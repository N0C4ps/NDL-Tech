//! Configuração completa do pipeline gráfico e de computação.
//!
//! O D3D11 é uma API baseada em objetos de estado e bindings separados;
//! este módulo agrupa tudo isso em um único [`GraphicsPipeline`], que por
//! baixo aplica os estados D3D11 correspondentes.
//!
//! ## Estado atual
//!
//! [`GraphicsPipeline`] já é real, mas só com o essencial pra desenhar
//! geometria colorida: vertex shader, pixel shader, input layout e
//! topologia. Rasterizer/blend/depth-stencil states ainda não existem
//! (entram quando `raster`/`blend`/`depth_stencil` forem implementados) —
//! até lá, o pipeline usa os estados padrão do D3D11.
//! `ComputePipeline` continua reservado.

use crate::core::{Device, GraphicsError};
use crate::input::{InputLayout, PrimitiveTopology};
use crate::shader::{PixelShader, VertexShader};

/// Configuração usada para criar um [`GraphicsPipeline`].
pub struct GraphicsPipelineDescriptor<'a> {
    pub vertex_shader: &'a VertexShader,
    pub pixel_shader: &'a PixelShader,
    pub input_layout: &'a InputLayout,
    pub primitive_topology: PrimitiveTopology,
}

/// Um pipeline gráfico: todo o estado necessário para emitir um `draw()`
/// — hoje, vertex/pixel shader, input layout e topologia.
pub struct GraphicsPipeline {
    #[cfg(target_os = "windows")]
    pub(crate) vertex_shader: windows::Win32::Graphics::Direct3D11::ID3D11VertexShader,
    #[cfg(target_os = "windows")]
    pub(crate) pixel_shader: windows::Win32::Graphics::Direct3D11::ID3D11PixelShader,
    #[cfg(target_os = "windows")]
    pub(crate) input_layout: windows::Win32::Graphics::Direct3D11::ID3D11InputLayout,
    pub(crate) primitive_topology: PrimitiveTopology,
}

impl Device {
    pub fn create_graphics_pipeline(
        &self,
        desc: &GraphicsPipelineDescriptor<'_>,
    ) -> Result<GraphicsPipeline, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            return Ok(GraphicsPipeline {
                vertex_shader: desc.vertex_shader.raw.clone(),
                pixel_shader: desc.pixel_shader.raw.clone(),
                input_layout: desc.input_layout.raw.clone(),
                primitive_topology: desc.primitive_topology,
            });
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = desc;
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
    }
}
