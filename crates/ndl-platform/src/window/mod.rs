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
pub use native_handle::{NativeWindowHandle, RawWindowHandle};
pub(crate) use native_handle::NativeWindowOps;
pub use window::Window;
