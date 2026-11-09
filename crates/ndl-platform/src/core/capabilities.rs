/// Recursos de plataforma que podem ou não estar disponíveis dependendo do
/// sistema operacional e do backend ativo.
///
/// A API pública do `ndl-platform` é idêntica em qualquer SO; é
/// `PlatformCapabilities` que informa, em tempo de execução, se um recurso
/// específico pode ser usado — em vez de o código do jogo precisar saber
/// detalhes de cada plataforma.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformCapabilities {
    pub multiple_windows: bool,
    pub transparency: bool,
    pub decorations_toggle: bool,
    pub fullscreen_exclusive: bool,
    pub fullscreen_borderless: bool,
    pub custom_cursor: bool,
    pub cursor_grab: bool,
    pub touch: bool,
    pub pen: bool,
    pub ime: bool,
    pub gamepad: bool,
    pub drag_and_drop: bool,
    pub multiple_monitors: bool,
}

impl PlatformCapabilities {
    /// Nenhum recurso opcional disponível — usado como base conservadora
    /// por backends que ainda não implementam tudo (ex: `headless`).
    pub const NONE: PlatformCapabilities = PlatformCapabilities {
        multiple_windows: false,
        transparency: false,
        decorations_toggle: false,
        fullscreen_exclusive: false,
        fullscreen_borderless: false,
        custom_cursor: false,
        cursor_grab: false,
        touch: false,
        pen: false,
        ime: false,
        gamepad: false,
        drag_and_drop: false,
        multiple_monitors: false,
    };
}
