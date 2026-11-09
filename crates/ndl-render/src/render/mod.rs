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
//! Esta primeira versão suporta um attachment de cor e, opcionalmente, um
//! attachment de depth. As operações de carga são `Load` e `Clear`. Stores
//! sempre preservam o conteúdo; múltiplos attachments, scissor, bind groups,
//! draws indiretos e operações `DontCare`/`Discard` ainda não fazem parte da
//! API.

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
#[derive(Clone, Copy)]
pub struct RenderPassDescriptor<'a> {
    pub color: ColorAttachment<'a>,
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
    color: &'a RenderTargetView,
    depth: Option<&'a DepthStencilView>,
}

impl Context {
    /// Inicia um render pass, vincula seus attachments e aplica as operações
    /// de carga.
    pub fn begin_render_pass<'a>(
        &'a self,
        desc: &RenderPassDescriptor<'a>,
    ) -> Result<RenderPass<'a>, GraphicsError> {
        if let Some(depth) = desc.depth {
            if desc.color.view.dimensions() != depth.view.dimensions() {
                return Err(GraphicsError::IncompatibleResource {
                    operation: "begin_render_pass",
                    reason: format!(
                        "color {:?} e depth {:?} precisam ter as mesmas dimensões",
                        desc.color.view.dimensions(),
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

        self.set_render_target(desc.color.view);
        if let Some(depth) = desc.depth {
            self.set_render_target_with_depth(desc.color.view, depth.view);
        }

        if let LoadOp::Clear(color) = desc.color.load {
            self.clear_render_target(desc.color.view, color);
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
            color: desc.color.view,
            depth: desc.depth.map(|depth| depth.view),
        })
    }
}

impl RenderPass<'_> {
    /// Vincula o pipeline gráfico para os próximos draws.
    pub fn set_pipeline(&mut self, pipeline: &GraphicsPipeline) -> Result<(), GraphicsError> {
        if let Some(expected) = pipeline.color_format {
            if self.color.format() != expected {
                return Err(GraphicsError::IncompatibleResource {
                    operation: "RenderPass::set_pipeline",
                    reason: format!(
                        "pipeline espera color {expected:?}, attachment usa {:?}",
                        self.color.format()
                    ),
                });
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

    /// Limpa novamente o attachment de cor atual.
    pub fn clear_color_attachment(&mut self, color: [f32; 4]) {
        self.context.clear_render_target(self.color, color);
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
