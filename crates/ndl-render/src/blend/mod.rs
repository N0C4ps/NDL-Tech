//! Mistura de cores (blending).
//!
//! A equação geral é `C_out = C_src * F_src + C_dst * F_dst`. Essencial
//! para transparência (vidro, partículas, UI com alpha) — sem um
//! [`BlendState`] habilitado, tudo que é desenhado substitui o que já
//! estava lá, mesmo com alpha menor que 1.
//!
//! ## Estado atual
//!
//! [`BlendState`] já é real, com os dois presets mais comuns
//! (`opaque`/`alpha_blend`). Operações lógicas (`LogicOp`), máscara de
//! escrita de cor e blending independente por render target continuam
//! reservados.

use crate::core::{Device, GraphicsError};

/// Fator multiplicado pela cor de origem/destino na equação de blend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendFactor {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
}

/// Como os fatores de origem/destino são combinados.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendOp {
    #[default]
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

/// Configuração usada para criar um [`BlendState`].
#[derive(Debug, Clone, Copy)]
pub struct BlendStateDescriptor {
    pub enabled: bool,
    pub src_factor: BlendFactor,
    pub dst_factor: BlendFactor,
    pub op: BlendOp,
}

impl BlendStateDescriptor {
    /// Sem blending — o padrão do D3D11. O que é desenhado substitui o
    /// que já estava no alvo, ignorando o canal alpha.
    pub const fn opaque() -> Self {
        Self {
            enabled: false,
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::Zero,
            op: BlendOp::Add,
        }
    }

    /// Alpha blending convencional: `cor_final = src.a * src + (1 - src.a) * dst`.
    pub const fn alpha_blend() -> Self {
        Self {
            enabled: true,
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            op: BlendOp::Add,
        }
    }
}

impl Default for BlendStateDescriptor {
    fn default() -> Self {
        Self::opaque()
    }
}

/// Um estado de blending já criado no dispositivo.
pub struct BlendState {
    #[cfg(target_os = "windows")]
    pub(crate) raw: windows::Win32::Graphics::Direct3D11::ID3D11BlendState,
}

impl Device {
    pub fn create_blend_state(
        &self,
        desc: &BlendStateDescriptor,
    ) -> Result<BlendState, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::create_blend_state(self, desc);
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
