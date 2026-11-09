/// Botão do mouse envolvido em um evento de input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

/// Estado de um botão do mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseState {
    Pressed,
    Released,
}
