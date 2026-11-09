/// Identificador opaco de uma janela, estável durante toda a vida da janela.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub(crate) u64);

impl WindowId {
    pub(crate) fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn raw_id(self) -> u64 {
        self.0
    }
}
