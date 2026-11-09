//! Controle do cursor: ícone, visibilidade, captura ("grab") e posição.

mod cursor;
mod icon;

pub use cursor::{CursorError, CursorManager};
pub use icon::{CursorIcon, CustomCursor};
