use super::adapter::AdapterInfo;
use super::error::GraphicsError;

/// Ponto de entrada para descobrir o que há disponível antes de criar um
/// [`super::Device`] — hoje, só enumeração de adaptadores (GPUs).
pub struct Instance;

impl Instance {
    pub fn new() -> Result<Self, GraphicsError> {
        Ok(Self)
    }

    /// Lista os adaptadores (GPUs) disponíveis no sistema.
    pub fn enumerate_adapters(&self) -> Result<Vec<AdapterInfo>, GraphicsError> {
        crate::backend::enumerate_adapters()
    }
}
