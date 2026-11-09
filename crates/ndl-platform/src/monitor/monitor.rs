/// Identificador opaco de um monitor conectado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MonitorHandle(pub(crate) u32);

impl MonitorHandle {
    #[allow(dead_code)] // usado pelos backends nativos ao enumerar monitores reais
    pub(crate) fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn raw_id(self) -> u32 {
        self.0
    }
}

/// Informações descritivas de um monitor: nome, posição/tamanho e taxa de
/// atualização.
#[derive(Debug, Clone, PartialEq)]
pub struct MonitorInfo {
    pub handle: MonitorHandle,
    pub name: String,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub scale_factor: f64,
    pub refresh_rate_mhz: Option<u32>,
}
