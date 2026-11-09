//! Input assembler: input layout e topologia de vértices.
//!
//! **Não confundir com `ndl-platform::input`** (teclado/mouse) — este é o
//! estágio de montagem de vértices da GPU, totalmente sem relação com
//! input de dispositivos.
//!
//! ## Estado atual
//!
//! [`VertexAttribute`], [`InputLayout`], [`PrimitiveTopology`] (só
//! `TriangleList` por enquanto) e [`IndexFormat`] já são reais.
//! [`VertexAttribute::input_slot`]/[`VertexAttribute::step_mode`] permitem
//! atributos vindos de um vertex buffer separado avançado por instância
//! em vez de por vértice ([`VertexStepMode::Instance`]) — a base para
//! instanced rendering (ver
//! [`crate::context::Context::draw_instanced`]). O resto das topologias e
//! os builders de `VertexBufferLayout` continuam reservados.

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

/// O tipo de cada índice num index buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexFormat {
    Uint16,
    Uint32,
}

/// Com que frequência o input assembler avança para o próximo elemento de
/// um atributo de vertex buffer — a diferença entre geometria normal e
/// dados de instância.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VertexStepMode {
    /// Avança um elemento por vértice desenhado — o comportamento normal
    /// de geometria (posição, normal, UV, ...).
    #[default]
    Vertex,
    /// Avança um elemento por instância desenhada, não por vértice — usado
    /// para dados que variam por instância (ex: matriz de transform, cor)
    /// num `draw_instanced`/`draw_indexed_instanced`. O mesmo elemento é
    /// reusado para todos os vértices da instância atual.
    Instance,
}

/// Um atributo de vértice: onde ele fica no buffer e o que o shader espera
/// encontrar ali (semântica HLSL, ex: `"POSITION"`, `"COLOR"`).
#[derive(Debug, Clone, Copy)]
pub struct VertexAttribute {
    pub semantic: &'static str,
    pub semantic_index: u32,
    pub format: Format,
    pub offset: u32,
    /// Em qual slot de vertex buffer (ver
    /// [`crate::context::Context::set_vertex_buffer`]) este atributo é
    /// lido — atributos de instância normalmente ficam num slot separado
    /// dos atributos por vértice, já que vêm de um buffer diferente.
    pub input_slot: u32,
    /// Se este atributo avança por vértice ou por instância.
    pub step_mode: VertexStepMode,
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
