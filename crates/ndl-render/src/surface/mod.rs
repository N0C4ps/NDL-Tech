//! Surface, swapchain e apresentação.
//!
//! Conecta uma janela (ex: um handle vindo de `ndl-platform::window::Window`)
//! a uma cadeia de troca de buffers e cuida de apresentação e resize.
//!
//! ## Planejado
//!
//! Tipos: `Surface`, `Swapchain`, `SwapchainDescriptor`, `PresentMode`,
//! `SurfaceFormat`, `PresentResult`.
//!
//! ```ignore
//! impl Surface {
//!     fn from_window(window: WindowHandle) -> Result<Self>;
//!     fn size(&self) -> Extent2D;
//!     fn is_valid(&self) -> bool;
//! }
//!
//! impl Swapchain {
//!     fn new(device: &Device, surface: &Surface, desc: &SwapchainDescriptor) -> Result<Self>;
//!     fn present(&mut self) -> Result<PresentResult>;
//!     fn present_with_options(&mut self, options: PresentOptions) -> Result<PresentResult>;
//!     fn resize(&mut self, width: u32, height: u32) -> Result<()>;
//!     fn back_buffer(&self) -> Result<Texture>;
//!     fn back_buffer_view(&self) -> Result<RenderTargetView>;
//!     fn format(&self) -> Format;
//!     fn dimensions(&self) -> Extent2D;
//!     fn buffer_count(&self) -> u32;
//!     fn set_fullscreen(&mut self, fullscreen: bool) -> Result<()>;
//!     fn is_fullscreen(&self) -> bool;
//! }
//! ```
//!
//! Operações complementares: `set_present_mode()`, `set_vsync()`,
//! `set_hdr_metadata()`, `set_color_space()`, `check_color_space_support()`,
//! `get_frame_latency()`, `set_max_frame_latency()`,
//! `get_frame_latency_waitable_object()`, `get_frame_statistics()`,
//! `get_last_present_count()`, `get_output()`.
//!
//! Enums/flags planejados: `PresentMode` (Immediate, VSync, Adaptive),
//! `SwapEffect` (Discard, Sequential, FlipSequential, FlipDiscard),
//! `PresentFlags` (ALLOW_TEARING, DO_NOT_WAIT, TEST, RESTART), `ColorSpace`
//! (sRGB, HDR10/PQ e outros espaços suportados pelo DXGI).
//!
//! `acquire_next_image()` pode existir por uniformidade com outras APIs,
//! mas no D3D11/DXGI não representa a mesma aquisição explícita de
//! imagens do Vulkan.
//!
//! Ainda não implementado.
