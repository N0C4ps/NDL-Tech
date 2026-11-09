use super::attributes::WindowAttributes;
use super::descriptor::{LogicalSize, WindowDescriptor};
use super::fullscreen::FullscreenMode;
use super::handle::WindowId;
use crate::core::Platform;
use crate::cursor::CursorManager;
use crate::error::PlatformError;

/// Uma janela criada através de [`Platform::create_window`].
///
/// `Window` não sabe nada sobre o conteúdo que será desenhado dentro dela —
/// isso é responsabilidade de `ndl-render`, que recebe um handle desta
/// janela para criar sua superfície gráfica.
pub struct Window {
    id: WindowId,
    attributes: WindowAttributes,
    cursor: CursorManager,
}

impl Window {
    pub(crate) fn from_descriptor(id: WindowId, descriptor: &WindowDescriptor) -> Self {
        Self {
            id,
            attributes: WindowAttributes {
                title: descriptor.title.clone(),
                inner_size: descriptor.size,
                min_size: descriptor.min_size,
                max_size: descriptor.max_size,
                visible: descriptor.visible,
                resizable: descriptor.resizable,
                decorated: descriptor.decorated,
                transparent: descriptor.transparent,
                focused: descriptor.focused,
                minimized: false,
                maximized: descriptor.maximized,
                always_on_top: descriptor.always_on_top,
                fullscreen: descriptor.fullscreen,
            },
            cursor: CursorManager::default(),
        }
    }

    /// Controle de cursor específico desta janela.
    pub fn cursor(&self) -> &CursorManager {
        &self.cursor
    }

    /// Controle de cursor específico desta janela (acesso mutável).
    pub fn cursor_mut(&mut self) -> &mut CursorManager {
        &mut self.cursor
    }

    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.attributes.title
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.attributes.title = title.into();
    }

    pub fn inner_size(&self) -> LogicalSize<u32> {
        self.attributes.inner_size
    }

    /// Tamanho externo da janela, incluindo decorações (barra de título,
    /// bordas). Em backends sem janela real (headless), é igual ao tamanho
    /// interno.
    pub fn outer_size(&self) -> LogicalSize<u32> {
        self.attributes.inner_size
    }

    pub fn set_min_size(&mut self, size: Option<LogicalSize<u32>>) {
        self.attributes.min_size = size;
    }

    pub fn set_max_size(&mut self, size: Option<LogicalSize<u32>>) {
        self.attributes.max_size = size;
    }

    pub fn show(&mut self) {
        self.attributes.visible = true;
    }

    pub fn hide(&mut self) {
        self.attributes.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.attributes.visible
    }

    pub fn focus(&mut self) {
        self.attributes.focused = true;
    }

    pub fn is_focused(&self) -> bool {
        self.attributes.focused
    }

    /// Pede atenção do usuário (ex: piscar o ícone na barra de tarefas),
    /// sem necessariamente roubar o foco.
    pub fn request_user_attention(&mut self) {
        // Sem efeito no backend headless; backends nativos vão delegar ao SO.
    }

    pub fn minimize(&mut self) {
        self.attributes.minimized = true;
        self.attributes.maximized = false;
    }

    pub fn maximize(&mut self) {
        self.attributes.maximized = true;
        self.attributes.minimized = false;
    }

    pub fn restore(&mut self) {
        self.attributes.minimized = false;
        self.attributes.maximized = false;
    }

    pub fn is_minimized(&self) -> bool {
        self.attributes.minimized
    }

    pub fn is_maximized(&self) -> bool {
        self.attributes.maximized
    }

    pub fn set_fullscreen(&mut self, mode: FullscreenMode) {
        self.attributes.fullscreen = mode;
    }

    pub fn fullscreen(&self) -> FullscreenMode {
        self.attributes.fullscreen
    }

    pub fn set_decorations(&mut self, decorated: bool) {
        self.attributes.decorated = decorated;
    }

    pub fn is_decorated(&self) -> bool {
        self.attributes.decorated
    }

    pub fn set_always_on_top(&mut self, always_on_top: bool) {
        self.attributes.always_on_top = always_on_top;
    }

    pub fn is_always_on_top(&self) -> bool {
        self.attributes.always_on_top
    }

    pub fn attributes(&self) -> &WindowAttributes {
        &self.attributes
    }
}

impl Platform {
    /// Cria uma nova janela a partir de um [`WindowDescriptor`].
    pub fn create_window(&self, descriptor: WindowDescriptor) -> Result<Window, PlatformError> {
        let id = self.backend.create_window(&descriptor)?;
        Ok(Window::from_descriptor(id, &descriptor))
    }

    /// Destrói uma janela previamente criada por esta plataforma.
    pub fn destroy_window(&self, window: Window) {
        self.backend.destroy_window(window.id());
    }
}
