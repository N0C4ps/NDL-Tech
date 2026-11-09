//! Contexto imediato e diferido.
//!
//! No modelo do Direct3D 11, o contexto imediato executa comandos
//! diretamente; contextos diferidos gravam listas de comandos executadas
//! depois pelo contexto imediato. Este módulo abstrai essa distinção sem
//! prometer o modelo de filas explícitas de APIs mais modernas.
//!
//! Cada handle criado por [`crate::core::Device::context`] refere o mesmo contexto imediato e
//! compartilha um lock de serialização por dispositivo. Cada chamada pública é
//! serializada individualmente; uma sequência de várias chamadas ainda pode ser
//! intercalada com outra thread. Para manter uma sequência indivisível, grave-a
//! num [`crate::command::CommandEncoder`] e submeta a lista pela fila.
//!
//! O estado usado pelas validações de bindings pertence ao contexto imediato e
//! acompanha os binds submetidos por esta crate. Chamadas nativas externas via
//! escape hatch podem invalidar esse rastreamento; nesse caso, as validações
//! dependentes dele não garantem o estado real do driver.
//!
//! Nenhuma chamada nativa aparece neste módulo — toda operação concreta
//! (`Draw`, `OMSetRenderTargets`, ...) vive atrás de [`crate::backend::Backend`],
//! implementado por cada backend em `crate::backend::<nome>`. Isso é o que
//! torna este módulo (e os outros do nível público da crate) igual para
//! qualquer backend ativo.
//!
//! ## Estado atual
//!
//! [`Context`] oferece comandos de baixo nivel; [`crate::render::RenderPass`]
//! organiza attachments, operacoes de carga e comandos de desenho. Contextos
//! diferidos e queries possuem APIs proprias. Permanecem reservados shaders
//! geometry/hull/domain, stream output e inspecao.
//!
//! | Grupo | Métodos ainda planejados |
//! |---|---|
//! | Shader | `set_geometry_shader`, `set_hull_shader`, `set_domain_shader` |
//! | Output merger | controle separado de depth target sem recriar o bind de cor |
//! | Stream output | `set_stream_output_targets`, `reset_stream_output_targets` |
//! | Consultas | predication e integracao direta de queries ao contexto |
//! | Inspecao | `get_pipeline_state`, `get_bound_resources`, `get_render_targets`, `get_viewports` |
//!
//! Já implementados: `set_blend_state` e `set_depth_stencil_state`, além dos
//! binds básicos, viewport, clear e draw. `draw_instanced` e
//! `draw_indexed_instanced` desenham N instâncias numa única chamada (ver
//! `VertexStepMode::Instance` em [`crate::input::VertexAttribute`]);
//! `draw_indirect` e `draw_indexed_indirect` leem os argumentos de um
//! [`crate::buffer::Buffer`] (`BufferUsage::Indirect`) em vez de recebê-los
//! da CPU — útil para GPU-driven rendering, ainda que o buffer indirect só
//! seja escrito pela CPU por enquanto. `set_render_targets`/
//! `set_render_targets_with_depth` vinculam múltiplos alvos de cor
//! simultâneos (MRT, até `DeviceLimits::max_color_attachments`) — o
//! `BlendState` único de `set_pipeline`/`set_blend_state` já se aplica a
//! todos eles igualmente (D3D11 sem `IndependentBlendEnable`); blend
//! independente por alvo continua reservado. Nenhum desses métodos faz
//! parte da lista de pendências.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::backend::{ActiveBackend, Backend};
use crate::buffer::Buffer;
use crate::core::DeviceLimits;
use crate::pipeline::GraphicsPipeline;
use crate::raster::Viewport;
use crate::sampler::Sampler;
use crate::view::{RenderTargetView, ShaderResourceView};

pub(crate) type VertexBufferBindings = Arc<Mutex<HashMap<u32, (u64, u32)>>>;
pub(crate) type ContextLock = Arc<Mutex<()>>;

/// O contexto imediato de um [`crate::core::Device`] — por onde comandos
/// de limpeza/desenho são de fato submetidos à GPU.
pub struct Context {
    pub(crate) raw: <ActiveBackend as Backend>::RawContext,
    bound_index_buffer: std::sync::Arc<std::sync::Mutex<Option<(u64, crate::input::IndexFormat)>>>,
    bound_vertex_buffers: VertexBufferBindings,
    context_lock: ContextLock,
    limits: DeviceLimits,
}

impl Context {
    pub(crate) fn limits(&self) -> DeviceLimits {
        self.limits
    }

    /// Contexto novo, com estado de bindings próprio (não compartilhado
    /// com o contexto imediato) — usado por
    /// [`crate::command::CommandEncoder`] para o contexto diferido.
    pub(crate) fn from_raw(raw: <ActiveBackend as Backend>::RawContext, limits: DeviceLimits) -> Self {
        Self::from_raw_with_shared(
            raw,
            std::sync::Arc::new(std::sync::Mutex::new(None)),
            std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            std::sync::Arc::new(std::sync::Mutex::new(())),
            limits,
        )
    }

    pub(crate) fn from_raw_with_shared(
        raw: <ActiveBackend as Backend>::RawContext,
        bound_index_buffer: std::sync::Arc<
            std::sync::Mutex<Option<(u64, crate::input::IndexFormat)>>,
        >,
        bound_vertex_buffers: VertexBufferBindings,
        context_lock: ContextLock,
        limits: DeviceLimits,
    ) -> Self {
        Self {
            raw,
            bound_index_buffer,
            bound_vertex_buffers,
            context_lock,
            limits,
        }
    }

    pub(crate) fn lock_context(&self) -> std::sync::MutexGuard<'_, ()> {
        self.context_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Define `view` como o único alvo de renderização atual, sem depth
    /// buffer — use [`Context::set_render_target_with_depth`] para 3D, ou
    /// [`Context::set_render_targets`] para vincular mais de um alvo de
    /// cor simultâneo (MRT).
    pub fn set_render_target(&self, view: &RenderTargetView) {
        let _guard = self.lock_context();
        ActiveBackend::set_render_target(self, view);
    }

    /// Define `view` como alvo de renderização e `depth` como o depth
    /// buffer usado pelo teste/escrita de profundidade — o caminho normal
    /// para desenhar geometria 3D com um único alvo de cor. Use
    /// [`Context::set_render_targets_with_depth`] para mais de um (MRT).
    pub fn set_render_target_with_depth(
        &self,
        view: &RenderTargetView,
        depth: &crate::view::DepthStencilView,
    ) {
        let _guard = self.lock_context();
        ActiveBackend::set_render_target_with_depth(self, view, depth);
    }

    /// Define `views` (1 a [`crate::core::DeviceLimits::max_color_attachments`])
    /// como os alvos de renderização simultâneos atuais (MRT — ex:
    /// deferred shading escrevendo albedo/normal/depth num só pass), sem
    /// depth buffer — use [`Context::set_render_targets_with_depth`] para
    /// 3D. `views[i]` corresponde ao `SV_TARGETn` de índice `i` no pixel
    /// shader.
    pub fn set_render_targets(
        &self,
        views: &[&RenderTargetView],
    ) -> Result<(), crate::core::GraphicsError> {
        let _guard = self.lock_context();
        validate_render_target_count(views.len(), self.limits.max_color_attachments)?;
        ActiveBackend::set_render_targets(self, views);
        Ok(())
    }

    /// Como [`Context::set_render_targets`], mas também vincula `depth`
    /// como o depth buffer usado pelo teste/escrita de profundidade — o
    /// caminho normal para desenhar geometria 3D com múltiplos alvos de
    /// cor simultâneos (MRT).
    pub fn set_render_targets_with_depth(
        &self,
        views: &[&RenderTargetView],
        depth: &crate::view::DepthStencilView,
    ) -> Result<(), crate::core::GraphicsError> {
        let _guard = self.lock_context();
        validate_render_target_count(views.len(), self.limits.max_color_attachments)?;
        ActiveBackend::set_render_targets_with_depth(self, views, depth);
        Ok(())
    }

    /// Limpa `view` (um depth buffer) com o valor de profundidade dado
    /// (normalmente `1.0`, o mais longe possível).
    pub fn clear_depth(
        &self,
        view: &crate::view::DepthStencilView,
        depth: f32,
    ) -> Result<(), crate::core::GraphicsError> {
        let _guard = self.lock_context();
        validate_depth_value(depth)?;
        ActiveBackend::clear_depth(self, view, depth);
        Ok(())
    }

    /// Define o estado de teste/escrita de profundidade usado pelos
    /// próximos `draw()`.
    pub fn set_depth_stencil_state(&self, state: &crate::depth_stencil::DepthStencilState) {
        let _guard = self.lock_context();
        ActiveBackend::set_depth_stencil_state(self, state);
    }

    /// Define o estado de blending usado pelos próximos `draw()`.
    pub fn set_blend_state(&self, state: &crate::blend::BlendState) {
        let _guard = self.lock_context();
        ActiveBackend::set_blend_state(self, state);
    }

    /// Limpa `view` inteira com a cor dada (RGBA, 0.0–1.0).
    pub fn clear_render_target(&self, view: &RenderTargetView, color: [f32; 4]) {
        let _guard = self.lock_context();
        ActiveBackend::clear_render_target(self, view, color);
    }

    /// Força o envio de qualquer comando ainda pendente no contexto para a GPU.
    pub fn flush(&self) {
        let _guard = self.lock_context();
        ActiveBackend::flush(self);
    }

    /// Define a região da superfície de destino em que a rasterização
    /// desenha. Sem isso, nada é rasterizado mesmo com o resto do
    /// pipeline configurado corretamente.
    pub fn set_viewport(&self, viewport: Viewport) -> Result<(), crate::core::GraphicsError> {
        let _guard = self.lock_context();
        validate_viewport(viewport, self.limits.max_viewport_dimension)?;
        ActiveBackend::set_viewport(self, viewport);
        Ok(())
    }

    /// Define o pipeline gráfico (shaders, input layout, topologia) a ser
    /// usado pelos próximos `draw()`.
    pub fn set_pipeline(&self, pipeline: &GraphicsPipeline) {
        let _guard = self.lock_context();
        ActiveBackend::set_pipeline(self, pipeline);
    }

    /// Vincula `buffer` ao slot `slot` de vertex buffer, com o `stride`
    /// (tamanho de um vértice, em bytes) dado.
    pub fn set_vertex_buffer(
        &self,
        slot: u32,
        buffer: &Buffer,
        stride: u32,
    ) -> Result<(), crate::core::GraphicsError> {
        let _guard = self.lock_context();
        if slot >= self.limits.max_vertex_buffer_bindings {
            return Err(crate::core::GraphicsError::OutOfBounds {
                resource: "vertex buffer slots",
                offset: slot as u64,
                size: 1,
                limit: self.limits.max_vertex_buffer_bindings as u64,
            });
        }
        if buffer.usage() != crate::buffer::BufferUsage::Vertex {
            return Err(crate::core::GraphicsError::IncompatibleResource {
                operation: "set_vertex_buffer",
                reason: "o buffer precisa ter BufferUsage::Vertex".into(),
            });
        }
        if stride == 0 || stride as u64 > buffer.size() {
            return Err(crate::core::GraphicsError::InvalidArgument {
                name: "vertex stride",
                reason: format!(
                    "precisa estar entre 1 e o tamanho do buffer ({})",
                    buffer.size()
                ),
            });
        }
        ActiveBackend::set_vertex_buffer(self, slot, buffer, stride);
        self.bound_vertex_buffers
            .lock()
            .map_err(|_| {
                crate::core::GraphicsError::Backend(
                    "estado interno dos vertex buffers foi envenenado".into(),
                )
            })?
            .insert(slot, (buffer.size(), stride));
        Ok(())
    }

    /// Vincula `buffer` como o index buffer atual, no formato `format`.
    pub fn set_index_buffer(
        &self,
        buffer: &Buffer,
        format: crate::input::IndexFormat,
    ) -> Result<(), crate::core::GraphicsError> {
        let _guard = self.lock_context();
        if buffer.usage() != crate::buffer::BufferUsage::Index {
            return Err(crate::core::GraphicsError::IncompatibleResource {
                operation: "set_index_buffer",
                reason: "o buffer precisa ter BufferUsage::Index".into(),
            });
        }
        let index_size = match format {
            crate::input::IndexFormat::Uint16 => 2,
            crate::input::IndexFormat::Uint32 => 4,
        };
        if buffer.size() % index_size != 0 {
            return Err(crate::core::GraphicsError::InvalidArgument {
                name: "index buffer size",
                reason: format!(
                    "o tamanho precisa ser múltiplo de {index_size} bytes para {format:?}"
                ),
            });
        }
        ActiveBackend::set_index_buffer(self, buffer, format);
        *self.bound_index_buffer.lock().map_err(|_| {
            crate::core::GraphicsError::Backend("estado interno do contexto envenenado".into())
        })? = Some((buffer.size(), format));
        Ok(())
    }

    /// Valida que `[first_index, first_index + index_count)` cabe no
    /// index buffer vinculado atualmente — compartilhado por `draw_indexed`
    /// e `draw_indexed_instanced`.
    fn check_index_range(
        &self,
        index_count: u32,
        first_index: u32,
    ) -> Result<(), crate::core::GraphicsError> {
        let bound = self.bound_index_buffer.lock().map_err(|_| {
            crate::core::GraphicsError::Backend("estado interno do contexto envenenado".into())
        })?;
        let (buffer_size, format) =
            bound.ok_or_else(|| crate::core::GraphicsError::InvalidArgument {
                name: "draw_indexed",
                reason: "nenhum index buffer foi vinculado".into(),
            })?;
        let element_size = match format {
            crate::input::IndexFormat::Uint16 => 2u64,
            crate::input::IndexFormat::Uint32 => 4u64,
        };
        let offset = (first_index as u64)
            .checked_mul(element_size)
            .ok_or_else(|| crate::core::GraphicsError::InvalidArgument {
                name: "first_index",
                reason: "overflow no cálculo do offset".into(),
            })?;
        let size = (index_count as u64)
            .checked_mul(element_size)
            .ok_or_else(|| crate::core::GraphicsError::InvalidArgument {
                name: "index_count",
                reason: "overflow no cálculo do tamanho".into(),
            })?;
        if offset
            .checked_add(size)
            .map_or(true, |end| end > buffer_size)
        {
            return Err(crate::core::GraphicsError::OutOfBounds {
                resource: "index buffer",
                offset,
                size,
                limit: buffer_size,
            });
        }
        Ok(())
    }

    /// Desenha `index_count` índices a partir de `first_index`, somando
    /// `base_vertex` a cada índice antes de buscar o vértice correspondente
    /// — usa o pipeline, os vertex buffers e o index buffer vinculados
    /// atualmente.
    pub fn draw_indexed(
        &self,
        index_count: u32,
        first_index: u32,
        base_vertex: i32,
    ) -> Result<(), crate::core::GraphicsError> {
        let _guard = self.lock_context();
        self.check_index_range(index_count, first_index)?;
        ActiveBackend::draw_indexed(self, index_count, first_index, base_vertex);
        Ok(())
    }

    /// Desenha `instance_count` instâncias de `index_count` índices a
    /// partir de `first_index`, somando `base_vertex` a cada índice —
    /// como `draw_indexed`, mas repetindo o desenho `instance_count` vezes
    /// sem reenviar geometria, variando apenas o `SV_InstanceID`
    /// (começando em `first_instance`) visível ao vertex shader e os
    /// atributos de vertex buffers com `VertexStepMode::Instance`. É o
    /// jeito de desenhar muitos objetos parecidos (vegetação, partículas,
    /// multidão) com uma única chamada em vez de um `draw_indexed` por
    /// objeto.
    pub fn draw_indexed_instanced(
        &self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        base_vertex: i32,
        first_instance: u32,
    ) -> Result<(), crate::core::GraphicsError> {
        let _guard = self.lock_context();
        if instance_count == 0 {
            return Ok(());
        }
        self.check_index_range(index_count, first_index)?;
        first_instance
            .checked_add(instance_count)
            .ok_or_else(|| crate::core::GraphicsError::InvalidArgument {
                name: "draw_indexed_instanced instance range",
                reason: "first_instance + instance_count excede u32::MAX".into(),
            })?;
        ActiveBackend::draw_indexed_instanced(
            self,
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
        );
        Ok(())
    }

    /// Desenha via [`crate::buffer::Buffer`] com `BufferUsage::Indirect`,
    /// lendo `index_count`/`instance_count`/`first_index`/`base_vertex`/
    /// `first_instance` de `args_buffer` a partir de `args_offset` (mesmo
    /// layout binário de [`crate::buffer::DrawIndexedIndirectArgs`]) —
    /// equivalente a `draw_indexed_instanced`, mas os argumentos são lidos
    /// pela própria GPU em vez de enviados pela CPU a cada chamada. Isso
    /// permite que um compute shader (culling, LOD) decida quantas
    /// instâncias desenhar sem a CPU precisar ler o resultado de volta
    /// antes de desenhar.
    ///
    /// Hoje `args_buffer` só pode ser escrito pela CPU (buffer indirect
    /// gravável por UAV continua reservado), mas isso já evita reemitir um
    /// `draw_indexed_instanced` por lote quando a contagem é decidida fora
    /// do laço de desenho principal.
    pub fn draw_indexed_indirect(
        &self,
        args_buffer: &Buffer,
        args_offset: u64,
    ) -> Result<(), crate::core::GraphicsError> {
        let _guard = self.lock_context();
        crate::compute::validate_indirect_args(
            args_buffer,
            args_offset,
            crate::buffer::DrawIndexedIndirectArgs::SIZE,
            "draw_indexed_indirect",
        )?;
        ActiveBackend::draw_indexed_indirect(self, args_buffer, args_offset);
        Ok(())
    }

    /// Valida que `[first_vertex, first_vertex + vertex_count)` cabe no
    /// vertex buffer vinculado no slot 0 — compartilhado por `draw` e
    /// `draw_instanced`.
    fn check_vertex_range(
        &self,
        vertex_count: u32,
        first_vertex: u32,
    ) -> Result<(), crate::core::GraphicsError> {
        let end_vertex = first_vertex.checked_add(vertex_count).ok_or_else(|| {
            crate::core::GraphicsError::InvalidArgument {
                name: "draw vertex range",
                reason: "first_vertex + vertex_count excede u32::MAX".into(),
            }
        })?;
        if let Some(&(buffer_size, stride)) = self
            .bound_vertex_buffers
            .lock()
            .map_err(|_| {
                crate::core::GraphicsError::Backend(
                    "estado interno dos vertex buffers foi envenenado".into(),
                )
            })?
            .get(&0)
        {
            let offset = (first_vertex as u64)
                .checked_mul(stride as u64)
                .ok_or_else(|| crate::core::GraphicsError::InvalidArgument {
                    name: "first_vertex",
                    reason: "overflow ao calcular o offset do vertex buffer".into(),
                })?;
            let end = (end_vertex as u64)
                .checked_mul(stride as u64)
                .ok_or_else(|| crate::core::GraphicsError::InvalidArgument {
                    name: "vertex_count",
                    reason: "overflow ao calcular o fim do vertex buffer".into(),
                })?;
            if end > buffer_size {
                return Err(crate::core::GraphicsError::OutOfBounds {
                    resource: "vertex buffer slot 0",
                    offset,
                    size: end - offset,
                    limit: buffer_size,
                });
            }
        }
        Ok(())
    }

    /// Desenha `vertex_count` vértices a partir de `first_vertex`, usando
    /// o pipeline e os vertex buffers vinculados atualmente.
    pub fn draw(
        &self,
        vertex_count: u32,
        first_vertex: u32,
    ) -> Result<(), crate::core::GraphicsError> {
        let _guard = self.lock_context();
        if vertex_count == 0 {
            return Ok(());
        }
        self.check_vertex_range(vertex_count, first_vertex)?;
        ActiveBackend::draw(self, vertex_count, first_vertex);
        Ok(())
    }

    /// Desenha `instance_count` instâncias de `vertex_count` vértices a
    /// partir de `first_vertex` — como `draw`, mas repetindo o desenho
    /// `instance_count` vezes sem reenviar geometria, variando apenas o
    /// `SV_InstanceID` (começando em `first_instance`) visível ao vertex
    /// shader e os atributos de vertex buffers com
    /// `VertexStepMode::Instance` (ver [`crate::input::VertexAttribute`]).
    pub fn draw_instanced(
        &self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<(), crate::core::GraphicsError> {
        let _guard = self.lock_context();
        if vertex_count == 0 || instance_count == 0 {
            return Ok(());
        }
        self.check_vertex_range(vertex_count, first_vertex)?;
        first_instance
            .checked_add(instance_count)
            .ok_or_else(|| crate::core::GraphicsError::InvalidArgument {
                name: "draw_instanced instance range",
                reason: "first_instance + instance_count excede u32::MAX".into(),
            })?;
        ActiveBackend::draw_instanced(self, vertex_count, instance_count, first_vertex, first_instance);
        Ok(())
    }

    /// Desenha via [`crate::buffer::Buffer`] com `BufferUsage::Indirect`,
    /// lendo `vertex_count`/`instance_count`/`first_vertex`/`first_instance`
    /// de `args_buffer` a partir de `args_offset` (mesmo layout binário de
    /// [`crate::buffer::DrawIndirectArgs`]) — equivalente a `draw_instanced`,
    /// mas os argumentos são lidos pela própria GPU em vez de enviados pela
    /// CPU a cada chamada.
    ///
    /// Hoje `args_buffer` só pode ser escrito pela CPU (buffer indirect
    /// gravável por UAV continua reservado).
    pub fn draw_indirect(
        &self,
        args_buffer: &Buffer,
        args_offset: u64,
    ) -> Result<(), crate::core::GraphicsError> {
        let _guard = self.lock_context();
        crate::compute::validate_indirect_args(
            args_buffer,
            args_offset,
            crate::buffer::DrawIndirectArgs::SIZE,
            "draw_indirect",
        )?;
        ActiveBackend::draw_indirect(self, args_buffer, args_offset);
        Ok(())
    }

    /// Vincula `buffer` ao slot `slot` de constant buffer, visível tanto
    /// para o vertex quanto para o pixel shader.
    pub fn set_constant_buffer(
        &self,
        slot: u32,
        buffer: &Buffer,
    ) -> Result<(), crate::core::GraphicsError> {
        let _guard = self.lock_context();
        if slot >= self.limits.max_uniform_buffer_bindings {
            return Err(crate::core::GraphicsError::OutOfBounds {
                resource: "uniform buffer slots",
                offset: slot as u64,
                size: 1,
                limit: self.limits.max_uniform_buffer_bindings as u64,
            });
        }
        if buffer.usage() != crate::buffer::BufferUsage::Constant {
            return Err(crate::core::GraphicsError::IncompatibleResource {
                operation: "set_constant_buffer",
                reason: "o buffer precisa ter BufferUsage::Constant".into(),
            });
        }
        ActiveBackend::set_constant_buffer(self, slot, buffer);
        Ok(())
    }

    /// Vincula `view` ao slot `slot` de recurso de leitura do pixel shader.
    pub fn set_shader_resource(
        &self,
        slot: u32,
        view: &ShaderResourceView,
    ) -> Result<(), crate::core::GraphicsError> {
        let _guard = self.lock_context();
        if slot >= self.limits.max_sampled_resource_bindings {
            return Err(crate::core::GraphicsError::OutOfBounds {
                resource: "sampled resource slots",
                offset: slot as u64,
                size: 1,
                limit: self.limits.max_sampled_resource_bindings as u64,
            });
        }
        ActiveBackend::set_shader_resource(self, slot, view);
        Ok(())
    }

    /// Vincula `sampler` ao slot `slot` de sampler do pixel shader.
    pub fn set_sampler(
        &self,
        slot: u32,
        sampler: &Sampler,
    ) -> Result<(), crate::core::GraphicsError> {
        let _guard = self.lock_context();
        if slot >= self.limits.max_sampler_bindings {
            return Err(crate::core::GraphicsError::OutOfBounds {
                resource: "pixel shader sampler slots",
                offset: slot as u64,
                size: 1,
                limit: self.limits.max_sampler_bindings as u64,
            });
        }
        ActiveBackend::set_sampler(self, slot, sampler);
        Ok(())
    }

    /// Sobrescreve todo o conteúdo de `buffer` com `data` (que deve ter
    /// exatamente `buffer.size()` bytes) — o jeito de atualizar, por
    /// exemplo, um constant buffer de transform a cada quadro. No D3D11 o
    /// driver gerencia dependências com trabalho em voo; backends explícitos
    /// deverão exigir sincronização ou retornar [`GraphicsError::ResourceBusy`].
    pub fn update_buffer(
        &self,
        buffer: &Buffer,
        data: &[u8],
    ) -> Result<(), crate::core::GraphicsError> {
        let _guard = self.lock_context();
        if data.len() as u64 != buffer.size() {
            return Err(crate::core::GraphicsError::DataSizeMismatch {
                expected: buffer.size(),
                actual: data.len() as u64,
            });
        }
        if buffer.usage() == crate::buffer::BufferUsage::Staging {
            return Err(crate::core::GraphicsError::IncompatibleResource {
                operation: "update_buffer",
                reason: "staging buffers são atualizados por mapeamento, não por UpdateSubresource"
                    .into(),
            });
        }
        if buffer.is_dynamic() {
            return Err(crate::core::GraphicsError::IncompatibleResource {
                operation: "update_buffer",
                reason: "buffers dynamic devem ser atualizados com map_write_discard".into(),
            });
        }
        ActiveBackend::update_buffer(self, buffer, data);
        Ok(())
    }
}

fn validate_render_target_count(
    count: usize,
    max_color_attachments: u32,
) -> Result<(), crate::core::GraphicsError> {
    if count == 0 {
        return Err(crate::core::GraphicsError::InvalidArgument {
            name: "render target count",
            reason: "precisa de ao menos um alvo de renderização".into(),
        });
    }
    if count as u64 > max_color_attachments as u64 {
        return Err(crate::core::GraphicsError::OutOfBounds {
            resource: "render target slots",
            offset: 0,
            size: count as u64,
            limit: max_color_attachments as u64,
        });
    }
    Ok(())
}

fn validate_viewport(
    viewport: Viewport,
    max_dimension: u32,
) -> Result<(), crate::core::GraphicsError> {
    let right = viewport.x + viewport.width;
    let bottom = viewport.y + viewport.height;
    if !viewport.x.is_finite()
        || !viewport.y.is_finite()
        || !viewport.width.is_finite()
        || !viewport.height.is_finite()
        || viewport.x < 0.0
        || viewport.y < 0.0
        || viewport.width <= 0.0
        || viewport.height <= 0.0
        || !right.is_finite()
        || !bottom.is_finite()
        || right > max_dimension as f32
        || bottom > max_dimension as f32
    {
        return Err(crate::core::GraphicsError::InvalidArgument {
            name: "Viewport",
            reason: format!(
                "x/y devem ser finitos e nao negativos, largura/altura positivas, e a extensao deve caber no limite {max_dimension}"
            ),
        });
    }
    if !viewport.min_depth.is_finite()
        || !viewport.max_depth.is_finite()
        || viewport.min_depth < 0.0
        || viewport.max_depth > 1.0
        || viewport.min_depth > viewport.max_depth
    {
        return Err(crate::core::GraphicsError::InvalidArgument {
            name: "Viewport::depth_range",
            reason: "deve satisfazer 0.0 <= min_depth <= max_depth <= 1.0".into(),
        });
    }
    Ok(())
}

fn validate_depth_value(depth: f32) -> Result<(), crate::core::GraphicsError> {
    if !depth.is_finite() || !(0.0..=1.0).contains(&depth) {
        return Err(crate::core::GraphicsError::InvalidArgument {
            name: "depth",
            reason: "deve ser um valor finito entre 0.0 e 1.0".into(),
        });
    }
    Ok(())
}
