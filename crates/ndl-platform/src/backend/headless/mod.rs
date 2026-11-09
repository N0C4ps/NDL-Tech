//! Backend "headless": não abre nenhuma janela real do sistema operacional.
//!
//! Útil para testes, CI, e como base funcional enquanto os backends nativos
//! (`windows`, `linux`, `macos`) ainda não estão implementados.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::backend::Backend;
use crate::core::PlatformCapabilities;
use crate::error::PlatformError;
use crate::event::WindowEvent;
use crate::monitor::MonitorInfo;
use crate::window::{WindowDescriptor, WindowId};

pub(crate) struct HeadlessBackend {
    next_window_id: AtomicU64,
}

impl HeadlessBackend {
    pub(crate) fn new() -> Self {
        Self {
            next_window_id: AtomicU64::new(1),
        }
    }
}

impl Backend for HeadlessBackend {
    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities::NONE
    }

    fn monitors(&self) -> Vec<MonitorInfo> {
        Vec::new()
    }

    fn primary_monitor(&self) -> Option<MonitorInfo> {
        None
    }

    fn create_window(&self, _descriptor: &WindowDescriptor) -> Result<WindowId, PlatformError> {
        let id = self.next_window_id.fetch_add(1, Ordering::Relaxed);
        Ok(WindowId::new(id))
    }

    fn destroy_window(&self, _id: WindowId) {}

    fn run(
        &self,
        _descriptor: WindowDescriptor,
        _on_event: Box<dyn FnMut(WindowEvent)>,
    ) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported(
            "backend headless não abre janelas reais; use PlatformBackendKind::Native",
        ))
    }
}
