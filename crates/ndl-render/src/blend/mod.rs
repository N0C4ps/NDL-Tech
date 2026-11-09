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
//! (`opaque`/`alpha_blend`), fatores/operação separados para alpha e
//! máscara de escrita de canais. Operações lógicas (`LogicOp`) e blending
//! independente por render target continuam reservados.

use crate::backend::{ActiveBackend, Backend};
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

/// Canais de cor habilitados para escrita no attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ColorWriteMask(pub u8);

impl ColorWriteMask {
    pub const NONE: Self = Self(0);
    pub const RED: Self = Self(1);
    pub const GREEN: Self = Self(2);
    pub const BLUE: Self = Self(4);
    pub const ALPHA: Self = Self(8);
    pub const ALL: Self = Self(15);
}

impl std::ops::BitOr for ColorWriteMask {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ColorWriteMask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
impl std::ops::BitAnd for ColorWriteMask {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl std::ops::BitAndAssign for ColorWriteMask {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}
impl std::ops::BitXor for ColorWriteMask {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}
impl std::ops::BitXorAssign for ColorWriteMask {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}
impl std::ops::Not for ColorWriteMask {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0 & Self::ALL.0)
    }
}

/// Configuração usada para criar um [`BlendState`].
#[derive(Debug, Clone, Copy)]
pub struct BlendStateDescriptor {
    pub enabled: bool,
    pub src_factor: BlendFactor,
    pub dst_factor: BlendFactor,
    pub op: BlendOp,
    pub src_alpha_factor: BlendFactor,
    pub dst_alpha_factor: BlendFactor,
    pub alpha_op: BlendOp,
    pub write_mask: ColorWriteMask,
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
            src_alpha_factor: BlendFactor::One,
            dst_alpha_factor: BlendFactor::Zero,
            alpha_op: BlendOp::Add,
            write_mask: ColorWriteMask::ALL,
        }
    }

    /// Alpha blending convencional: `cor_final = src.a * src + (1 - src.a) * dst`.
    pub const fn alpha_blend() -> Self {
        Self {
            enabled: true,
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            op: BlendOp::Add,
            src_alpha_factor: BlendFactor::One,
            dst_alpha_factor: BlendFactor::OneMinusSrcAlpha,
            alpha_op: BlendOp::Add,
            write_mask: ColorWriteMask::ALL,
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
    pub(crate) raw: <ActiveBackend as Backend>::RawBlendState,
}

impl Device {
    pub fn create_blend_state(
        &self,
        desc: &BlendStateDescriptor,
    ) -> Result<BlendState, GraphicsError> {
        if desc.write_mask.0 & !ColorWriteMask::ALL.0 != 0 {
            return Err(GraphicsError::InvalidArgument {
                name: "BlendStateDescriptor::write_mask",
                reason: "a máscara só aceita os bits RED, GREEN, BLUE e ALPHA".into(),
            });
        }
        ActiveBackend::create_blend_state(self, desc)
    }
}

#[cfg(test)]
mod tests {
    use super::ColorWriteMask;

    #[test]
    fn channel_masks_compose_and_stay_within_rgba() {
        let rgb = ColorWriteMask::RED | ColorWriteMask::GREEN | ColorWriteMask::BLUE;
        assert_eq!(rgb.0, 0b0111);
        assert_eq!(!rgb, ColorWriteMask::ALPHA);
        assert_eq!(rgb & ColorWriteMask::GREEN, ColorWriteMask::GREEN);
    }
}
