use super::fullscreen::FullscreenMode;

/// Tamanho em pixels lógicos (independentes do fator de escala do monitor).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogicalSize<T> {
    pub width: T,
    pub height: T,
}

impl<T> LogicalSize<T> {
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

/// Posição em pixels físicos (independentes do fator de escala do monitor).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicalPosition<T> {
    pub x: T,
    pub y: T,
}

impl<T> PhysicalPosition<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

/// Configuração usada para criar uma janela.
///
/// Representa a intenção de criação: depois de criada, o estado "vivo" da
/// janela (que pode mudar com o tempo) é consultado/alterado através de
/// [`super::Window`] e [`super::WindowAttributes`].
#[derive(Debug, Clone, PartialEq)]
pub struct WindowDescriptor {
    pub title: String,
    pub size: LogicalSize<u32>,
    pub min_size: Option<LogicalSize<u32>>,
    pub max_size: Option<LogicalSize<u32>>,
    pub position: Option<PhysicalPosition<i32>>,

    pub visible: bool,
    pub resizable: bool,
    pub decorated: bool,
    pub transparent: bool,
    pub maximized: bool,

    pub fullscreen: FullscreenMode,
    pub always_on_top: bool,
    pub focused: bool,
}

impl Default for WindowDescriptor {
    fn default() -> Self {
        Self {
            title: "NDL".to_string(),
            size: LogicalSize::new(1280, 720),
            min_size: None,
            max_size: None,
            position: None,
            visible: true,
            resizable: true,
            decorated: true,
            transparent: false,
            maximized: false,
            fullscreen: FullscreenMode::Windowed,
            always_on_top: false,
            focused: true,
        }
    }
}
