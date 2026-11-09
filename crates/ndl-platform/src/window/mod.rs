//! Criação e gerenciamento de janelas.

mod attributes;
mod descriptor;
mod fullscreen;
mod handle;
mod native_handle;
mod window;

pub use attributes::WindowAttributes;
pub use descriptor::{LogicalSize, PhysicalPosition, WindowDescriptor};
pub use fullscreen::FullscreenMode;
pub use handle::WindowId;
pub(crate) use native_handle::NativeWindowOps;
pub use native_handle::{NativeWindowHandle, RawWindowHandle};
pub use window::Window;
