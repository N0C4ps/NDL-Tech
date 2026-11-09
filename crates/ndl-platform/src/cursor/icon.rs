/// Ícones de cursor padrão do sistema, equivalentes aos cursores nativos
/// mais comuns entre Windows/Linux/macOS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorIcon {
    #[default]
    Default,
    Pointer,
    Text,
    Crosshair,
    Move,
    Grab,
    Grabbing,
    Wait,
    Progress,
    NotAllowed,
    ResizeEastWest,
    ResizeNorthSouth,
    ResizeNorthEastSouthWest,
    ResizeNorthWestSouthEast,
}

/// Cursor customizado definido pelo jogo, como uma textura (e, no futuro,
/// possivelmente uma animação quadro a quadro).
#[derive(Debug, Clone, PartialEq)]
pub struct CustomCursor {
    pub width: u32,
    pub height: u32,
    /// Pixels RGBA8, `width * height * 4` bytes.
    pub rgba: Vec<u8>,
    /// Ponto do cursor que corresponde à posição real do clique.
    pub hotspot: (u32, u32),
}
