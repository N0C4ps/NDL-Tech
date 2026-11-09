use crate::monitor::MonitorHandle;

/// Modo de tela cheia de uma janela.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullscreenMode {
    /// Janela normal, não em tela cheia.
    Windowed,
    /// Tela cheia "sem borda", ocupando o monitor dado sem trocar o modo de vídeo.
    Borderless(MonitorHandle),
    /// Tela cheia exclusiva no monitor dado (pode trocar resolução/taxa de atualização).
    Exclusive(MonitorHandle),
}
