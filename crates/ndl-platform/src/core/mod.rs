//! Inicialização da plataforma e suas capacidades.
//!
//! O `core` administra a inicialização do backend do sistema operacional e
//! expõe quais recursos estão disponíveis através de [`PlatformCapabilities`].

mod capabilities;
mod lifecycle;
mod platform;

pub use capabilities::PlatformCapabilities;
pub use lifecycle::ApplicationState;
pub use platform::{OperatingSystem, Platform, PlatformBackendKind, PlatformConfig};
