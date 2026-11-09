//! `ndl-math` é a fundação matemática do ecossistema NDL.
//!
//! Fornece os tipos básicos (vetores, matrizes, quaternions) usados por outras
//! crates do framework (como `ndl-render` ou `ndl-physics`). Esta crate não
//! conhece nada sobre renderização, física ou qualquer outro domínio — é
//! infraestrutura pura, consumida por quem precisar de matemática 3D/2D.

mod mat4;
mod quat;
mod vec2;
mod vec3;
mod vec4;

pub use mat4::Mat4;
pub use quat::Quat;
pub use vec2::Vec2;
pub use vec3::Vec3;
pub use vec4::Vec4;
