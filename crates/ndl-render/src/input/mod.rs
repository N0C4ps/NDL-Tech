//! Input assembler: input layout e topologia de vértices.
//!
//! **Não confundir com `ndl-platform::input`** (teclado/mouse) — este é o
//! estágio de montagem de vértices da GPU, totalmente sem relação com
//! input de dispositivos.
//!
//! ## Estado atual
//!
//! [`VertexAttribute`], [`InputLayout`] e [`PrimitiveTopology`] (só
//! `TriangleList` por enquanto) já são reais. O resto das topologias e os
//! builders de `VertexBufferLayout` continuam reservados.

use crate::core::{Device, GraphicsError};
use crate::format::Format;
use crate::shader::VertexShader;

/// Como os vértices enviados em `draw()` são agrupados em primitivas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PrimitiveTopology {
    TriangleList,
    // TODO: PointList, LineList, LineStrip, TriangleStrip, *Adjacency,
    // PatchList(u8) — adicionados conforme forem usados de verdade.
}

/// Um atributo de vértice: onde ele fica no buffer e o que o shader espera
/// encontrar ali (semântica HLSL, ex: `"POSITION"`, `"COLOR"`).
#[derive(Debug, Clone, Copy)]
pub struct VertexAttribute {
    pub semantic: &'static str,
    pub semantic_index: u32,
    pub format: Format,
    pub offset: u32,
}

/// Descreve o layout de um vértice para o input assembler — criado a
/// partir de uma lista de [`VertexAttribute`] e do bytecode do vertex
/// shader que vai consumi-los (o D3D11 precisa casar os dois).
pub struct InputLayout {
    #[cfg(target_os = "windows")]
    pub(crate) raw: windows::Win32::Graphics::Direct3D11::ID3D11InputLayout,
}

impl Device {
    pub fn create_input_layout(
        &self,
        attributes: &[VertexAttribute],
        vertex_shader: &VertexShader,
    ) -> Result<InputLayout, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::create_input_layout(self, attributes, vertex_shader);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (attributes, vertex_shader);
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
    }
}
