//! Dispositivo, adaptadores e capacidades.
//!
//! Responsável por inicializar o backend gráfico, descobrir a GPU
//! disponível e expor o que o hardware/driver suporta — a base de onde
//! todo o resto da crate parte.
//!
//! A API pública representa o subconjunto portátil do renderer. Recursos
//! específicos de um backend não são emulados silenciosamente: recebem um
//! erro de suporte ou são acessados por extensões nativas explícitas.

mod adapter;
mod capabilities;
mod device;
mod error;
mod instance;

/// Acesso nativo opcional ao Direct3D 11. Esta extensao nao faz parte do
/// contrato portavel de `Device` e so existe quando o backend DirectX 11
/// está ativo (feature `backend-directx11`).
#[cfg(feature = "backend-directx11")]
pub mod d3d11;

pub use adapter::{AdapterInfo, AdapterType};
pub use capabilities::{
    CapabilitySource, DeviceCapabilities, DeviceLimits, FeatureSupport, FormatFeatures,
    LimitsSource,
};
pub use device::{Device, DeviceDescriptor};
pub use error::GraphicsError;
pub use instance::Instance;

/// Identificador estável do backend ativo; não implica que todos os
/// backends tenham as mesmas capacidades.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BackendApi {
    Direct3D11,
    Vulkan,
    Direct3D12,
    Metal,
    OpenGl,
    Unknown,
}
