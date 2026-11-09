//! Rasterização, viewport e scissor.
//!
//! ## Estado atual
//!
//! [`Viewport`] e [`RasterizerState`] são reais, incluindo fill/cull mode,
//! orientação da face e depth bias. Scissor dinâmico e controle de
//! multisampling permanecem reservados.
//!
//! | Propriedade | Valores planejados |
//! |---|---|
//! | Propriedade | Estado |
//! |---|---|
//! | FillMode | implementado |
//! | CullMode | implementado |
//! | FrontFace | implementado |
//! | DepthClip e depth bias | implementados |
//! | Scissor e multisampling | reservados |

use crate::backend::{ActiveBackend, Backend};
use crate::core::{Device, GraphicsError};

/// Modo de preenchimento dos triângulos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FillMode {
    #[default]
    Solid,
    Wireframe,
}

/// Faces descartadas antes de rasterizar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CullMode {
    None,
    Front,
    #[default]
    Back,
}

/// Convenção de orientação da face frontal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FrontFace {
    #[default]
    CounterClockwise,
    Clockwise,
}

/// Estado fixo de rasterização, sem scissor dinâmico.
#[derive(Debug, Clone, Copy)]
pub struct RasterizerStateDescriptor {
    pub fill_mode: FillMode,
    pub cull_mode: CullMode,
    pub front_face: FrontFace,
    pub depth_clip_enabled: bool,
    pub depth_bias: i32,
    pub depth_bias_clamp: f32,
    pub slope_scaled_depth_bias: f32,
}

impl Default for RasterizerStateDescriptor {
    fn default() -> Self {
        Self {
            fill_mode: FillMode::Solid,
            cull_mode: CullMode::Back,
            front_face: FrontFace::Clockwise,
            depth_clip_enabled: true,
            depth_bias: 0,
            depth_bias_clamp: 0.0,
            slope_scaled_depth_bias: 0.0,
        }
    }
}

/// Estado de rasterização criado no dispositivo.
pub struct RasterizerState {
    pub(crate) raw: <ActiveBackend as Backend>::RawRasterizerState,
}

impl Device {
    pub fn create_rasterizer_state(
        &self,
        desc: &RasterizerStateDescriptor,
    ) -> Result<RasterizerState, GraphicsError> {
        if !desc.depth_bias_clamp.is_finite() || !desc.slope_scaled_depth_bias.is_finite() {
            return Err(GraphicsError::InvalidArgument {
                name: "RasterizerStateDescriptor",
                reason: "depth bias precisa conter valores finitos".into(),
            });
        }
        ActiveBackend::create_rasterizer_state(self, desc)
    }
}

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
