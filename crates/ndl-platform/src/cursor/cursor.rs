use std::fmt;

use super::icon::{CursorIcon, CustomCursor};

/// Erros específicos de operações de cursor.
#[derive(Debug, Clone, PartialEq)]
pub enum CursorError {
    Unsupported(&'static str),
}

impl fmt::Display for CursorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CursorError::Unsupported(what) => {
                write!(f, "operação de cursor não suportada: {what}")
            }
        }
    }
}

impl std::error::Error for CursorError {}

/// Controla a aparência e o comportamento do cursor dentro de uma janela.
///
/// Jogos que trocam o cursor por sprites próprios (ex: um jogo de RPG com
/// cursor temático) ou que precisam capturar o mouse (ex: câmera em
/// primeira pessoa) usam este tipo — nada disso acontece por padrão.
#[derive(Debug, Clone)]
pub struct CursorManager {
    icon: CursorIcon,
    custom: Option<CustomCursor>,
    visible: bool,
    grabbed: bool,
}

impl Default for CursorManager {
    fn default() -> Self {
        Self {
            icon: CursorIcon::default(),
            custom: None,
            visible: true,
            grabbed: false,
        }
    }
}

impl CursorManager {
    pub fn set_icon(&mut self, icon: CursorIcon) {
        self.custom = None;
        self.icon = icon;
    }

    pub fn icon(&self) -> CursorIcon {
        self.icon
    }

    pub fn set_custom(&mut self, cursor: CustomCursor) {
        self.custom = Some(cursor);
    }

    pub fn custom(&self) -> Option<&CustomCursor> {
        self.custom.as_ref()
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_grab(&mut self, grab: bool) -> Result<(), CursorError> {
        self.grabbed = grab;
        Ok(())
    }

    pub fn is_grabbed(&self) -> bool {
        self.grabbed
    }
}
