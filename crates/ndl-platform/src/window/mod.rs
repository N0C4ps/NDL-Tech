//! Criação e gerenciamento de janelas.

mod attributes;
mod descriptor;
mod fullscreen;
mod handle;
mod window;

pub use attributes::WindowAttributes;
pub use descriptor::{LogicalSize, PhysicalPosition, WindowDescriptor};
pub use fullscreen::FullscreenMode;
pub use handle::WindowId;
pub use window::Window;
