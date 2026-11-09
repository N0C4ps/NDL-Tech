//! Amostragem de texturas.
//!
//! ## Estado atual
//!
//! [`Sampler`] já é real, com [`FilterMode`] e [`AddressMode`] cobrindo o
//! essencial (ponto/linear, wrap/clamp/mirror). Comparação (`CompareOp`),
//! anisotropia, bias de LOD e cor de borda continuam reservados.

use crate::backend::{ActiveBackend, Backend};
use crate::core::{Device, GraphicsError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterMode {
    Point,
    #[default]
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AddressMode {
    #[default]
    Wrap,
    Mirror,
    Clamp,
}

/// Configuração usada para criar um [`Sampler`].
#[derive(Debug, Clone, Copy, Default)]
pub struct SamplerDescriptor {
    pub filter: FilterMode,
    pub address_mode: AddressMode,
}

impl SamplerDescriptor {
    pub const fn point_clamp() -> Self {
        Self {
            filter: FilterMode::Point,
            address_mode: AddressMode::Clamp,
        }
    }

    pub const fn point_repeat() -> Self {
        Self {
            filter: FilterMode::Point,
            address_mode: AddressMode::Wrap,
        }
    }

    pub const fn linear_clamp() -> Self {
        Self {
            filter: FilterMode::Linear,
            address_mode: AddressMode::Clamp,
        }
    }

    pub const fn linear_repeat() -> Self {
        Self {
            filter: FilterMode::Linear,
            address_mode: AddressMode::Wrap,
        }
    }
}

/// Um sampler de textura já criado no dispositivo.
pub struct Sampler {
    pub(crate) raw: <ActiveBackend as Backend>::RawSampler,
}

impl Device {
    pub fn create_sampler(&self, desc: &SamplerDescriptor) -> Result<Sampler, GraphicsError> {
        ActiveBackend::create_sampler(self, desc)
    }
}
