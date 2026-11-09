//! Dispositivo, adaptadores e capacidades.
//!
//! Responsável por inicializar o backend gráfico, descobrir a GPU
//! disponível e expor o que o hardware/driver suporta — a base de onde
//! todo o resto da crate parte.

mod adapter;
mod capabilities;
mod device;
mod error;
mod instance;

pub use adapter::AdapterInfo;
pub use capabilities::{DeviceCapabilities, DeviceLimits, FeatureLevel};
pub use device::{Device, DeviceDescriptor};
pub use error::GraphicsError;
pub use instance::Instance;
