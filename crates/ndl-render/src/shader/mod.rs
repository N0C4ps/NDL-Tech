//! Compilação e estágios programáveis de shader.
//!
//! ## Estado atual
//!
//! [`compile_hlsl`], [`VertexShader`], [`PixelShader`] e [`ComputeShader`]
//! já são reais. Hull/domain/geometry shader e reflexão de shader
//! (assinaturas de entrada/saída, constant buffers, etc.) continuam
//! reservados.
//!
//! | Estágio | Tipo Rust | Finalidade | Estado |
//! |---|---|---|---|
//! | Vertex | [`VertexShader`] | Processamento de vértices | real |
//! | Pixel | [`PixelShader`] | Fragmentos/pixels | real |
//! | Compute | [`ComputeShader`] | Computação geral na GPU | real |
//! | Hull | `HullShader` | Controle de tesselação | reservado |
//! | Domain | `DomainShader` | Avaliação da tesselação | reservado |
//! | Geometry | `GeometryShader` | Processamento de primitivas | reservado |

use crate::backend::{ActiveBackend, Backend};
use crate::core::{Device, GraphicsError};

/// Compila código-fonte HLSL para bytecode de shader.
///
/// `target` é o "shader profile" do D3D11 (ex: `"vs_5_0"`, `"ps_5_0"`).
pub fn compile_hlsl(
    source: &str,
    entry_point: &str,
    target: &str,
) -> Result<Vec<u8>, GraphicsError> {
    ActiveBackend::compile_hlsl(source, entry_point, target)
}

/// Um vertex shader já criado no dispositivo.
pub struct VertexShader {
    pub(crate) raw: <ActiveBackend as Backend>::RawVertexShader,
    /// Guardado porque `CreateInputLayout` precisa do bytecode do vertex
    /// shader, não só do shader já criado.
    pub(crate) bytecode: Vec<u8>,
}

/// Um pixel shader já criado no dispositivo.
pub struct PixelShader {
    pub(crate) raw: <ActiveBackend as Backend>::RawPixelShader,
}

/// Um compute shader já criado no dispositivo.
pub struct ComputeShader {
    pub(crate) raw: <ActiveBackend as Backend>::RawComputeShader,
}

impl Device {
    pub fn create_vertex_shader(&self, bytecode: &[u8]) -> Result<VertexShader, GraphicsError> {
        ActiveBackend::create_vertex_shader(self, bytecode)
    }

    pub fn create_pixel_shader(&self, bytecode: &[u8]) -> Result<PixelShader, GraphicsError> {
        ActiveBackend::create_pixel_shader(self, bytecode)
    }

    pub fn create_compute_shader(&self, bytecode: &[u8]) -> Result<ComputeShader, GraphicsError> {
        ActiveBackend::create_compute_shader(self, bytecode)
    }
}
