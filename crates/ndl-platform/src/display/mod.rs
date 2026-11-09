//! Consultas de nível de "display" sobre o conjunto de monitores conectados,
//! construídas sobre [`crate::monitor`].

use crate::core::Platform;
use crate::monitor::MonitorInfo;

impl Platform {
    /// Lista todos os monitores conectados.
    pub fn monitors(&self) -> Vec<MonitorInfo> {
        self.backend.monitors()
    }

    /// Retorna o monitor primário, se o backend conseguir determiná-lo.
    pub fn primary_monitor(&self) -> Option<MonitorInfo> {
        self.backend.primary_monitor()
    }
}
