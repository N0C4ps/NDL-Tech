//! Render passes para organizar os comandos que desenham um quadro.
//!
//! Um pass vincula seus attachments ao [`crate::context::Context`], aplica
//! as operações de carga solicitadas e oferece uma interface agrupada para
//! configurar estado e emitir draw calls. O backend D3D11 executa os comandos
//! imediatamente; `end` fecha o escopo lógico da API, sem emitir uma chamada
//! nativa correspondente.
//!
//! ## Estado atual
//!
//! Suporta de um a [`crate::core::DeviceLimits::max_color_attachments`]
//! attachments de cor simultâneos (MRT — ex: deferred shading escrevendo
//! albedo/normal num só pass) e, opcionalmente, um attachment de depth. As
//! operações de carga são `Load` e `Clear`. Stores sempre preservam o
//! conteúdo; [`RenderPass::draw_instanced`], [`RenderPass::draw_indexed_instanced`],
//! [`RenderPass::draw_indirect`] e [`RenderPass::draw_indexed_indirect`] já
//! são reais (ver [`crate::context::Context`] para o que cada um exige).
//! Scissor, bind groups e operações `DontCare`/`Discard` ainda não fazem
//! parte da API.

use std::ops::Range;

use crate::blend::BlendState;
use crate::buffer::Buffer;
use crate::context::Context;
use crate::core::GraphicsError;
use crate::depth_stencil::DepthStencilState;
use crate::input::IndexFormat;
use crate::pipeline::GraphicsPipeline;
use crate::raster::Viewport;
use crate::sampler::Sampler;
use crate::view::{DepthStencilView, RenderTargetView, ShaderResourceView};

/// Operação feita sobre um attachment ao iniciar um render pass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadOp<T> {
    /// Preserva o conteúdo que já está no attachment.
    Load,
    /// Limpa o attachment com o valor fornecido.
    Clear(T),
}

/// Configuração do attachment de cor do pass.
#[derive(Clone, Copy)]
pub struct ColorAttachment<'a> {
    pub view: &'a RenderTargetView,
    pub load: LoadOp<[f32; 4]>,
}

/// Configuração do attachment de depth do pass.
#[derive(Clone, Copy)]
pub struct DepthAttachment<'a> {
    pub view: &'a DepthStencilView,
    pub load: LoadOp<f32>,
}

/// Attachments e operações de carga para iniciar um [`RenderPass`].
///
/// `colors` tem um attachment por alvo de cor simultâneo — um elemento
/// para o caso comum, vários para MRT (até
/// [`crate::core::DeviceLimits::max_color_attachments`]). Todos os
/// attachments (de cor e, se houver, de depth) precisam ter as mesmas
/// dimensões.
#[derive(Clone, Copy)]
pub struct RenderPassDescriptor<'a> {
    pub colors: &'a [ColorAttachment<'a>],
    pub depth: Option<DepthAttachment<'a>>,
}

/// Escopo de comandos de desenho sobre os attachments configurados.
///
/// O pass mantém um empréstimo do contexto até ser consumido por
/// [`RenderPass::end`] ou sair de escopo. Isso organiza a API Rust, mas não
/// desfaz bindings do backend ao terminar: no D3D11 o estado permanece
/// vinculado até outro comando substituí-lo.
pub struct RenderPass<'a> {
    context: &'a Context,
    colors: &'a [ColorAttachment<'a>],
    depth: Option<&'a DepthStencilView>,
}

impl Context {
    /// Inicia um render pass, vincula seus attachments (um ou vários alvos
    /// de cor — MRT quando `desc.colors.len() > 1`) e aplica as operações
    /// de carga.
    pub fn begin_render_pass<'a>(
        &'a self,
        desc: &RenderPassDescriptor<'a>,
    ) -> Result<RenderPass<'a>, GraphicsError> {
        let Some(first_color) = desc.colors.first() else {
            return Err(GraphicsError::InvalidArgument {
                name: "RenderPassDescriptor::colors",
                reason: "precisa de ao menos um attachment de cor".into(),
            });
        };
        let dimensions = first_color.view.dimensions();
        for color in desc.colors {
            if color.view.dimensions() != dimensions {
                return Err(GraphicsError::IncompatibleResource {
                    operation: "begin_render_pass",
                    reason: format!(
                        "todos os attachments de cor precisam ter as mesmas dimensões ({:?} != {:?})",
                        color.view.dimensions(),
                        dimensions
                    ),
                });
            }
        }
        if let Some(depth) = desc.depth {
            if dimensions != depth.view.dimensions() {
                return Err(GraphicsError::IncompatibleResource {
                    operation: "begin_render_pass",
                    reason: format!(
                        "color {:?} e depth {:?} precisam ter as mesmas dimensões",
                        dimensions,
                        depth.view.dimensions()
                    ),
                });
            }
        }
        if let Some(DepthAttachment {
            load: LoadOp::Clear(depth),
            ..
        }) = desc.depth
        {
            validate_depth(depth)?;
        }

        let color_views: Vec<&RenderTargetView> = desc.colors.iter().map(|c| c.view).collect();
        if let Some(depth) = desc.depth {
            self.set_render_targets_with_depth(&color_views, depth.view)?;
        } else {
            self.set_render_targets(&color_views)?;
        }

        for color in desc.colors {
            if let LoadOp::Clear(value) = color.load {
                self.clear_render_target(color.view, value);
            }
        }
        if let Some(DepthAttachment {
            view,
            load: LoadOp::Clear(depth),
        }) = desc.depth
        {
            self.clear_depth(view, depth)?;
        }

        Ok(RenderPass {
            context: self,
            colors: desc.colors,
            depth: desc.depth.map(|depth| depth.view),
        })
    }
}

impl RenderPass<'_> {
    /// Vincula o pipeline gráfico para os próximos draws.
    pub fn set_pipeline(&mut self, pipeline: &GraphicsPipeline) -> Result<(), GraphicsError> {
        if !pipeline.color_formats.is_empty() {
            if pipeline.color_formats.len() != self.colors.len() {
                return Err(GraphicsError::IncompatibleResource {
                    operation: "RenderPass::set_pipeline",
                    reason: format!(
                        "pipeline declara {} alvo(s) de cor, pass tem {}",
                        pipeline.color_formats.len(),
                        self.colors.len()
                    ),
                });
            }
            for (index, (&expected, attachment)) in
                pipeline.color_formats.iter().zip(self.colors.iter()).enumerate()
            {
                if attachment.view.format() != expected {
                    return Err(GraphicsError::IncompatibleResource {
                        operation: "RenderPass::set_pipeline",
                        reason: format!(
                            "alvo de cor {index}: pipeline espera {expected:?}, attachment usa {:?}",
                            attachment.view.format()
                        ),
                    });
                }
            }
        }
        match (pipeline.depth_format, self.depth) {
            (Some(expected), Some(view)) if view.format() == expected => {}
            (Some(expected), Some(view)) => {
                return Err(GraphicsError::IncompatibleResource {
                    operation: "RenderPass::set_pipeline",
                    reason: format!(
                        "pipeline espera depth {expected:?}, attachment usa {:?}",
                        view.format()
                    ),
                });
            }
            (Some(_), None) => {
                return Err(GraphicsError::IncompatibleResource {
                    operation: "RenderPass::set_pipeline",
                    reason: "pipeline exige depth attachment".into(),
                });
            }
            (None, Some(_)) => {
                return Err(GraphicsError::IncompatibleResource {
                    operation: "RenderPass::set_pipeline",
                    reason: "pipeline não declara depth attachment".into(),
                });
            }
            (None, None) => {}
        }
        self.context.set_pipeline(pipeline);
        Ok(())
    }

    /// Vincula um vertex buffer ao slot indicado.
    pub fn set_vertex_buffer(
        &mut self,
        slot: u32,
        buffer: &Buffer,
        stride: u32,
    ) -> Result<(), GraphicsError> {
        self.context.set_vertex_buffer(slot, buffer, stride)
    }

    /// Vincula o index buffer usado por [`RenderPass::draw_indexed`].
    pub fn set_index_buffer(
        &mut self,
        buffer: &Buffer,
        format: IndexFormat,
    ) -> Result<(), GraphicsError> {
        self.context.set_index_buffer(buffer, format)
    }

    /// Define a região do attachment em que a rasterização desenha.
    pub fn set_viewport(&mut self, viewport: Viewport) -> Result<(), GraphicsError> {
        self.context.set_viewport(viewport)
    }

    /// Define o estado de blend para os próximos draws.
    pub fn set_blend_state(&mut self, state: &BlendState) {
        self.context.set_blend_state(state);
    }

    /// Define o estado de teste e escrita de depth/stencil.
    pub fn set_depth_stencil_state(&mut self, state: &DepthStencilState) {
        self.context.set_depth_stencil_state(state);
    }

    /// Vincula um constant buffer aos shaders de vértice e pixel.
    pub fn set_constant_buffer(&mut self, slot: u32, buffer: &Buffer) -> Result<(), GraphicsError> {
        self.context.set_constant_buffer(slot, buffer)
    }

    /// Vincula uma textura como recurso de leitura do pixel shader.
    pub fn set_shader_resource(
        &mut self,
        slot: u32,
        view: &ShaderResourceView,
    ) -> Result<(), GraphicsError> {
        self.context.set_shader_resource(slot, view)
    }

    /// Vincula um sampler ao pixel shader.
    pub fn set_sampler(&mut self, slot: u32, sampler: &Sampler) -> Result<(), GraphicsError> {
        self.context.set_sampler(slot, sampler)
    }

    /// Limpa novamente o attachment de cor de índice `index` (0 para o
    /// caso comum de um único alvo; em MRT, o índice corresponde ao
    /// `SV_TARGETn` do pixel shader).
    pub fn clear_color_attachment(&mut self, index: usize, color: [f32; 4]) -> Result<(), GraphicsError> {
        let attachment = self.colors.get(index).ok_or_else(|| GraphicsError::OutOfBounds {
            resource: "RenderPass color attachments",
            offset: index as u64,
            size: 1,
            limit: self.colors.len() as u64,
        })?;
        self.context.clear_render_target(attachment.view, color);
        Ok(())
    }

    /// Limpa novamente o attachment de depth indicado.
    pub fn clear_depth_attachment(&mut self, depth: f32) -> Result<(), GraphicsError> {
        validate_depth(depth)?;
        let view = self.depth.ok_or_else(|| GraphicsError::InvalidArgument {
            name: "RenderPass::clear_depth_attachment",
            reason: "este render pass não possui attachment de depth".into(),
        })?;
        self.context.clear_depth(view, depth)?;
        Ok(())
    }

    /// Desenha o intervalo de vértices `[vertices.start, vertices.end)`.
    /// Um intervalo vazio é um no-op.
    pub fn draw(&mut self, vertices: Range<u32>) -> Result<(), GraphicsError> {
        let count = checked_range_count("vertices", &vertices)?;
        if count != 0 {
            self.context.draw(count, vertices.start)?;
        }
        Ok(())
    }

    /// Desenha índices do intervalo `[indices.start, indices.end)`, somando
    /// `base_vertex` a cada índice. Um intervalo vazio é um no-op.
    pub fn draw_indexed(
        &mut self,
        indices: Range<u32>,
        base_vertex: i32,
    ) -> Result<(), GraphicsError> {
        let count = checked_range_count("indices", &indices)?;
        if count != 0 {
            self.context
                .draw_indexed(count, indices.start, base_vertex)?;
        }
        Ok(())
    }

    /// Desenha `instance_count` instâncias do intervalo de vértices
    /// `[vertices.start, vertices.end)`, com `SV_InstanceID` começando em
    /// `first_instance` — ver [`crate::context::Context::draw_instanced`].
    pub fn draw_instanced(
        &mut self,
        vertices: Range<u32>,
        instance_count: u32,
        first_instance: u32,
    ) -> Result<(), GraphicsError> {
        let count = checked_range_count("vertices", &vertices)?;
        self.context
            .draw_instanced(count, instance_count, vertices.start, first_instance)
    }

    /// Desenha `instance_count` instâncias de índices do intervalo
    /// `[indices.start, indices.end)`, somando `base_vertex` a cada índice
    /// e com `SV_InstanceID` começando em `first_instance` — ver
    /// [`crate::context::Context::draw_indexed_instanced`].
    pub fn draw_indexed_instanced(
        &mut self,
        indices: Range<u32>,
        base_vertex: i32,
        instance_count: u32,
        first_instance: u32,
    ) -> Result<(), GraphicsError> {
        let count = checked_range_count("indices", &indices)?;
        self.context.draw_indexed_instanced(
            count,
            instance_count,
            indices.start,
            base_vertex,
            first_instance,
        )
    }

    /// Desenha lendo os argumentos de `args_buffer` a partir de
    /// `args_offset` — ver [`crate::context::Context::draw_indirect`].
    pub fn draw_indirect(&mut self, args_buffer: &Buffer, args_offset: u64) -> Result<(), GraphicsError> {
        self.context.draw_indirect(args_buffer, args_offset)
    }

    /// Desenha com índices lendo os argumentos de `args_buffer` a partir
    /// de `args_offset` — ver
    /// [`crate::context::Context::draw_indexed_indirect`].
    pub fn draw_indexed_indirect(
        &mut self,
        args_buffer: &Buffer,
        args_offset: u64,
    ) -> Result<(), GraphicsError> {
        self.context.draw_indexed_indirect(args_buffer, args_offset)
    }

    /// Fecha o escopo lógico do pass. O backend mantém o estado atual
    /// vinculado até uma chamada posterior substituí-lo.
    pub fn end(self) {}
}

fn validate_depth(depth: f32) -> Result<(), GraphicsError> {
    if !depth.is_finite() || !(0.0..=1.0).contains(&depth) {
        return Err(GraphicsError::InvalidArgument {
            name: "depth",
            reason: "deve ser um valor finito entre 0.0 e 1.0".into(),
        });
    }
    Ok(())
}

fn checked_range_count(name: &'static str, range: &Range<u32>) -> Result<u32, GraphicsError> {
    range
        .end
        .checked_sub(range.start)
        .ok_or_else(|| GraphicsError::InvalidArgument {
            name,
            reason: format!(
                "o fim do intervalo ({}) não pode ser menor que o início ({})",
                range.end, range.start
            ),
        })
}
