//! Contexto imediato e diferido.
//!
//! No modelo do Direct3D 11, o contexto imediato executa comandos
//! diretamente; contextos diferidos gravam listas de comandos executadas
//! depois pelo contexto imediato. Este módulo abstrai essa distinção sem
//! prometer o modelo de filas explícitas de APIs mais modernas.
//!
//! ## Estado atual
//!
//! [`Context`] já é real para o contexto imediato: `set_render_target`,
//! `clear_render_target` e `flush` funcionam de verdade contra o D3D11.
//! O resto do que estava planejado (pipeline, vertex input, shader,
//! recursos, rasterizer, stream output, consultas, inspeção, e contextos
//! diferidos) continua reservado — ver tabela abaixo.
//!
//! | Grupo | Métodos planejados |
//! |---|---|
//! | Pipeline | `set_pipeline`, `set_compute_pipeline`, `unbind_pipeline` |
//! | Vertex input | `set_vertex_buffer(s)`, `set_index_buffer`, `set_input_layout`, `set_primitive_topology` |
//! | Shader | `set_vertex_shader`, `set_pixel_shader`, `set_geometry_shader`, `set_hull_shader`, `set_domain_shader`, `set_compute_shader` |
//! | Recursos | `set_constant_buffer`, `set_shader_resource`, `set_sampler`, `set_unordered_access_view` |
//! | Output merger | `set_render_targets` (múltiplos), `set_depth_stencil_target`, `set_blend_state`, `set_depth_stencil_state` |
//! | Rasterizer | `set_rasterizer_state`, `set_viewport(s)`, `set_scissor(s)` |
//! | Stream output | `set_stream_output_targets`, `reset_stream_output_targets` |
//! | Consultas | `begin_query`, `end_query`, `get_query_data`, `set_predication` |
//! | Inspeção | `get_pipeline_state`, `get_bound_resources`, `get_render_targets`, `get_viewports` |

#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext;

use crate::view::RenderTargetView;

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
}
