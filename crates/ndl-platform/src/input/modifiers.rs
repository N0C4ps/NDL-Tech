/// Teclas modificadoras ativas no momento de um evento de input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

impl Modifiers {
    pub const NONE: Modifiers = Modifiers {
        shift: false,
        ctrl: false,
        alt: false,
        super_key: false,
    };

    pub fn any(self) -> bool {
        self.shift || self.ctrl || self.alt || self.super_key
    }
}
