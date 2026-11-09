//! Contexto imediato e diferido.
//!
//! No modelo do Direct3D 11, o contexto imediato executa comandos
//! diretamente; contextos diferidos gravam listas de comandos executadas
//! depois pelo contexto imediato. Este módulo abstrai essa distinção sem
//! prometer o modelo de filas explícitas de APIs mais modernas.
//!
//! ## Estado atual
//!
//! [`Context`] já cobre o essencial pra desenhar com texturas e constant
//! buffers: `set_render_target`, `clear_render_target`, `flush`,
//! `set_viewport`, `set_pipeline`, `set_vertex_buffer`,
//! `set_constant_buffer`, `set_shader_resource`, `set_sampler`,
//! `update_buffer` e `draw`. O resto do que estava planejado (geometry/
//! hull/domain shader, output merger com múltiplos alvos, stream output,
//! consultas, inspeção, e contextos diferidos) continua reservado — ver
//! tabela abaixo.
//!
//! | Grupo | Métodos planejados |
//! |---|---|
//! | Shader | `set_geometry_shader`, `set_hull_shader`, `set_domain_shader`, `set_compute_shader` |
//! | Output merger | múltiplos alvos simultâneos, `set_depth_stencil_target`, `set_blend_state`, `set_depth_stencil_state` |
//! | Stream output | `set_stream_output_targets`, `reset_stream_output_targets` |
//! | Consultas | `begin_query`, `end_query`, `get_query_data`, `set_predication` |
//! | Inspeção | `get_pipeline_state`, `get_bound_resources`, `get_render_targets`, `get_viewports` |

#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext;

use crate::buffer::Buffer;
use crate::pipeline::GraphicsPipeline;
use crate::raster::Viewport;
use crate::sampler::Sampler;
use crate::view::{RenderTargetView, ShaderResourceView};

/// O contexto imediato de um [`crate::core::Device`] — por onde comandos
/// de limpeza/desenho são de fato submetidos à GPU.
pub struct Context {
    #[cfg(target_os = "windows")]
    pub(crate) raw: ID3D11DeviceContext,
}

impl Context {
    #[cfg(target_os = "windows")]
    pub(crate) fn from_raw(raw: ID3D11DeviceContext) -> Self {
        Self { raw }
    }

    /// Define `view` como o único alvo de renderização atual.
    ///
    /// Múltiplos alvos simultâneos (MRT) ainda não são suportados — isso
    /// entra quando `view`/`texture` tiverem mais do que `RenderTargetView`.
    pub fn set_render_target(&self, view: &RenderTargetView) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.OMSetRenderTargets(Some(&[Some(view.raw.clone())]), None);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = view;
        }
    }

    /// Limpa `view` inteira com a cor dada (RGBA, 0.0–1.0).
    pub fn clear_render_target(&self, view: &RenderTargetView, color: [f32; 4]) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.ClearRenderTargetView(&view.raw, &color);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (view, color);
        }
    }

    /// Força o envio de qualquer comando ainda pendente no contexto para a GPU.
    pub fn flush(&self) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.Flush();
        }
    }

    /// Define a região da superfície de destino em que a rasterização
    /// desenha. Sem isso, nada é rasterizado mesmo com o resto do
    /// pipeline configurado corretamente.
    pub fn set_viewport(&self, viewport: Viewport) {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::Graphics::Direct3D11::D3D11_VIEWPORT;
            self.raw.RSSetViewports(Some(&[D3D11_VIEWPORT {
                TopLeftX: viewport.x,
                TopLeftY: viewport.y,
                Width: viewport.width,
                Height: viewport.height,
                MinDepth: viewport.min_depth,
                MaxDepth: viewport.max_depth,
            }]));
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = viewport;
        }
    }

    /// Define o pipeline gráfico (shaders, input layout, topologia) a ser
    /// usado pelos próximos `draw()`.
    pub fn set_pipeline(&self, pipeline: &GraphicsPipeline) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.IASetInputLayout(&pipeline.input_layout);
            self.raw.VSSetShader(&pipeline.vertex_shader, None);
            self.raw.PSSetShader(&pipeline.pixel_shader, None);
            self.raw.IASetPrimitiveTopology(topology_to_d3d(pipeline.primitive_topology));
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = pipeline;
        }
    }

    /// Vincula `buffer` ao slot `slot` de vertex buffer, com o `stride`
    /// (tamanho de um vértice, em bytes) dado.
    pub fn set_vertex_buffer(&self, slot: u32, buffer: &Buffer, stride: u32) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.IASetVertexBuffers(
                slot,
                1,
                Some(&Some(buffer.raw.clone())),
                Some(&stride),
                Some(&0),
            );
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (slot, buffer, stride);
        }
    }

    /// Desenha `vertex_count` vértices a partir de `first_vertex`, usando
    /// o pipeline e os vertex buffers vinculados atualmente.
    pub fn draw(&self, vertex_count: u32, first_vertex: u32) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.Draw(vertex_count, first_vertex);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (vertex_count, first_vertex);
        }
    }

    /// Vincula `buffer` ao slot `slot` de constant buffer, visível tanto
    /// para o vertex quanto para o pixel shader.
    pub fn set_constant_buffer(&self, slot: u32, buffer: &Buffer) {
        #[cfg(target_os = "windows")]
        unsafe {
            let slice = Some(&[Some(buffer.raw.clone())][..]);
            self.raw.VSSetConstantBuffers(slot, slice);
            self.raw.PSSetConstantBuffers(slot, slice);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (slot, buffer);
        }
    }

    /// Vincula `view` ao slot `slot` de recurso de leitura do pixel shader.
    pub fn set_shader_resource(&self, slot: u32, view: &ShaderResourceView) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.PSSetShaderResources(slot, Some(&[Some(view.raw.clone())]));
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (slot, view);
        }
    }

    /// Vincula `sampler` ao slot `slot` de sampler do pixel shader.
    pub fn set_sampler(&self, slot: u32, sampler: &Sampler) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.PSSetSamplers(slot, Some(&[Some(sampler.raw.clone())]));
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (slot, sampler);
        }
    }

    /// Sobrescreve todo o conteúdo de `buffer` com `data` (que deve ter
    /// exatamente `buffer.size()` bytes) — o jeito de atualizar, por
    /// exemplo, um constant buffer de transform a cada quadro.
    pub fn update_buffer(&self, buffer: &Buffer, data: &[u8]) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.UpdateSubresource(
                &buffer.raw,
                0,
                None,
                data.as_ptr() as *const _,
                0,
                0,
            );
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (buffer, data);
        }
    }
}

#[cfg(target_os = "windows")]
fn topology_to_d3d(
    topology: crate::input::PrimitiveTopology,
) -> windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY {
    use windows::Win32::Graphics::Direct3D::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST;
    match topology {
        crate::input::PrimitiveTopology::TriangleList => D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
    }
}
