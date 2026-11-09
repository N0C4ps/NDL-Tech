//! `ndl` é a fachada pública do ecossistema NDL.
//!
//! Esta crate não implementa nada por conta própria — ela só reexporta as
//! demais crates do framework (`ndl-math`, `ndl-platform`, `ndl-render`,
//! ...) atrás de feature flags, para que o código do jogo possa escrever
//! `use ndl::platform::...` / `use ndl::math::...` importando só o que
//! habilitou em seu `Cargo.toml`, em vez de depender de cada crate
//! individualmente.
//!
//! ```toml
//! [dependencies]
//! ndl = { git = "https://github.com/N0C4ps/NDL-Tech.git", features = ["math", "platform"] }
//! ```
//!
//! O princípio de isolamento entre crates continua valendo: habilitar
//! `platform` e `render` juntos não cria nenhuma dependência entre os
//! domínios dessas duas crates — elas simplesmente ficam acessíveis sob o
//! mesmo namespace `ndl::`.

#[cfg(feature = "math")]
pub use ndl_math as math;

#[cfg(feature = "platform")]
pub use ndl_platform as platform;

#[cfg(feature = "render")]
pub use ndl_render as render;

#[cfg(feature = "config")]
pub use ndl_config as config;

#[cfg(feature = "dependencys")]
pub use ndl_dependencys as dependencys;

#[cfg(feature = "diagnostics")]
pub use ndl_diagnostics as diagnostics;

#[cfg(feature = "logger")]
pub use ndl_logger as logger;
