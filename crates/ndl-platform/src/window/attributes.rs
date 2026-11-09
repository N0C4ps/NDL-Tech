use super::descriptor::LogicalSize;
use super::fullscreen::FullscreenMode;

/// Estado "vivo" de uma janela já criada — em contraste com
/// [`super::WindowDescriptor`], que representa a configuração inicial.
///
/// É atualizado conforme a janela muda (resize do usuário, minimizar, etc.)
/// e consultado pelos métodos de [`super::Window`].
#[derive(Debug, Clone, PartialEq)]
pub struct WindowAttributes {
    pub title: String,
    pub inner_size: LogicalSize<u32>,
    pub min_size: Option<LogicalSize<u32>>,
    pub max_size: Option<LogicalSize<u32>>,

    pub visible: bool,
    pub resizable: bool,
    pub decorated: bool,
    pub transparent: bool,

    pub focused: bool,
    pub minimized: bool,
    pub maximized: bool,
    pub always_on_top: bool,

    pub fullscreen: FullscreenMode,
}
