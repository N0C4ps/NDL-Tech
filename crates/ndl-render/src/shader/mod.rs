//! Compilação e estágios programáveis de shader.
//!
//! ## Estado atual
//!
//! [`compile_hlsl`], [`VertexShader`] e [`PixelShader`] já são reais —
//! cobrem o caminho mínimo pra desenhar geometria. Os demais estágios
//! (hull, domain, geometry, compute) e reflexão de shader (assinaturas de
//! entrada/saída, constant buffers, etc.) continuam reservados.
//!
//! | Estágio | Tipo Rust | Finalidade | Estado |
//! |---|---|---|---|
//! | Vertex | [`VertexShader`] | Processamento de vértices | real |
//! | Pixel | [`PixelShader`] | Fragmentos/pixels | real |
//! | Hull | `HullShader` | Controle de tesselação | reservado |
//! | Domain | `DomainShader` | Avaliação da tesselação | reservado |
//! | Geometry | `GeometryShader` | Processamento de primitivas | reservado |
//! | Compute | `ComputeShader` | Computação geral na GPU | reservado |

use crate::core::{Device, GraphicsError};

/// Compila código-fonte HLSL para bytecode de shader.
///
/// `target` é o "shader profile" do D3D11 (ex: `"vs_5_0"`, `"ps_5_0"`).
pub fn compile_hlsl(source: &str, entry_point: &str, target: &str) -> Result<Vec<u8>, GraphicsError> {
    #[cfg(target_os = "windows")]
    {
        crate::backend::directx::compile_hlsl(source, entry_point, target)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (source, entry_point, target);
        Err(GraphicsError::Unsupported(
            "nenhum backend gráfico implementado para este SO ainda",
        ))
    }
}

/// Um vertex shader já criado no dispositivo.
pub struct VertexShader {
    #[cfg(target_os = "windows")]
    pub(crate) raw: windows::Win32::Graphics::Direct3D11::ID3D11VertexShader,
    /// Guardado porque `CreateInputLayout` precisa do bytecode do vertex
    /// shader, não só do shader já criado.
    pub(crate) bytecode: Vec<u8>,
}

/// Um pixel shader já criado no dispositivo.
pub struct PixelShader {
    #[cfg(target_os = "windows")]
    pub(crate) raw: windows::Win32::Graphics::Direct3D11::ID3D11PixelShader,
}

impl Device {
    pub fn create_vertex_shader(&self, bytecode: &[u8]) -> Result<VertexShader, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::create_vertex_shader(self, bytecode);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = bytecode;
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
    }

    pub fn create_pixel_shader(&self, bytecode: &[u8]) -> Result<PixelShader, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::create_pixel_shader(self, bytecode);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = bytecode;
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
    }
}
