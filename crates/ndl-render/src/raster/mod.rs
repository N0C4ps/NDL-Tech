//! Rasterização, viewport e scissor.
//!
//! ## Estado atual
//!
//! [`Viewport`] já é real — sem definir um viewport, o D3D11 não
//! rasteriza nada visível mesmo com o resto do pipeline correto. O resto
//! (`RasterizerState`, scissor, fill/cull mode, depth bias, ...) continua
//! reservado; até existir, o rasterizer usa os estados padrão do D3D11
//! (solid fill, back-face culling, sem scissor).
//!
//! | Propriedade | Valores planejados |
//! |---|---|
//! | FillMode | Solid, Wireframe |
//! | CullMode | None, Front, Back |
//! | FrontFace | Clockwise, CounterClockwise |
//! | DepthClip | Enabled, Disabled |
//! | Scissor | Enabled, Disabled |
//! | Multisample | Enabled, Disabled |

/// Região da superfície de destino em que a rasterização desenha,
/// mapeando o volume de clip ([-1,1]) para coordenadas de pixel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

impl Viewport {
    /// Viewport cobrindo toda a extensão `(width, height)`, começando em
    /// `(0, 0)`, com o range de profundidade padrão `[0, 1]`.
    pub fn full(width: f32, height: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width,
            height,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }
}
