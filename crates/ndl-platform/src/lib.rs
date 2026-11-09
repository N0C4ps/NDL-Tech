//! `ndl-platform` é a camada de abstração do NDL sobre o sistema operacional.
//!
//! Ela oferece uma interface **consistente** para janelas, eventos, entrada
//! de dispositivos, monitores, cursores, temporização e integração com APIs
//! gráficas — mas, assim como as demais crates do NDL, é **infraestrutura
//! pura**: importar `ndl-platform` não abre janela nenhuma sozinha. Quem usa
//! a crate decide quando criar uma janela, que eventos observar e como
//! reagir a eles.
//!
//! A API pública é a mesma em qualquer sistema operacional. Diferenças reais
//! de capacidade entre plataformas são expostas via [`PlatformCapabilities`],
//! nunca vazando detalhes de implementação por OS — esses detalhes vivem no
//! módulo `backend`, que é inteiramente privado a esta crate.

mod backend;

pub mod core;
pub mod cursor;
pub mod display;
pub mod error;
pub mod event;
pub mod input;
pub mod monitor;
pub mod window;

pub use core::{ApplicationState, OperatingSystem, Platform, PlatformCapabilities, PlatformConfig};
pub use error::PlatformError;
