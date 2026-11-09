//! Implementações específicas de cada sistema operacional.
//!
//! Este módulo é **inteiramente privado**: nada aqui é exposto na API
//! pública da crate. Cada backend concreto (Windows, Linux/X11, Linux/
//! Wayland, macOS, Headless) implementa o mesmo contrato interno
//! [`BackendHandle`], e o restante da crate programa apenas contra esse
//! contrato — nunca contra um backend específico.
//!
//! Hoje só o backend `headless` está implementado; os demais existem como
//! stubs reservando o lugar na árvore de módulos para quando a integração
//! real com cada SO (provavelmente via `winit` ou chamadas nativas) for
//! escrita.

mod headless;
#[allow(dead_code)]
mod linux;
#[allow(dead_code)]
mod macos;
#[allow(dead_code)]
mod windows;

use crate::core::{PlatformBackendKind, PlatformCapabilities};
use crate::error::PlatformError;
use crate::monitor::MonitorInfo;
use crate::window::{WindowDescriptor, WindowId};

/// Contrato interno que todo backend de sistema operacional deve implementar.
pub(crate) trait Backend {
    fn capabilities(&self) -> PlatformCapabilities;
    fn monitors(&self) -> Vec<MonitorInfo>;
    fn primary_monitor(&self) -> Option<MonitorInfo>;
    fn create_window(&self, descriptor: &WindowDescriptor) -> Result<WindowId, PlatformError>;
    fn destroy_window(&self, id: WindowId);
}

pub(crate) type BackendHandle = Box<dyn Backend>;

pub(crate) fn create(kind: PlatformBackendKind) -> Result<BackendHandle, PlatformError> {
    match kind {
        PlatformBackendKind::Headless => Ok(Box::new(headless::HeadlessBackend::new())),
        PlatformBackendKind::Native => {
            // Nenhum backend nativo real está implementado ainda; cai para
            // headless como base funcional até windows/linux/macos existirem.
            Ok(Box::new(headless::HeadlessBackend::new()))
        }
    }
}
