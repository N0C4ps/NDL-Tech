//! Entrada de teclado e mouse.
//!
//! Os tipos aqui descrevem o *estado* e os *dados* de input; a forma como
//! esses dados chegam até o código do jogo é via [`crate::event`] — este
//! módulo não lê o SO diretamente.

mod keyboard;
mod modifiers;
mod mouse;
mod state;

pub use keyboard::{KeyCode, KeyState};
pub use modifiers::Modifiers;
pub use mouse::{MouseButton, MouseState};
pub use state::InputState;
