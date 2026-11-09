//! Implementações específicas de cada sistema operacional.
//!
//! Este módulo é **inteiramente privado**: nada aqui é exposto na API
//! pública da crate. Cada backend concreto (Windows, Linux/X11, Linux/
//! Wayland, macOS, Headless) implementa o mesmo contrato interno
//! [`Backend`], e o restante da crate programa apenas contra esse
//! contrato — nunca contra um backend específico.
//!
//! Hoje `headless` (simulação em memória, sem SO) e `windows` (via `winit`,
//! com janela real) estão implementados; `linux`/`macos` existem como
//! stubs reservando o lugar na árvore de módulos.

mod headless;
#[allow(dead_code)]
mod linux;
#[allow(dead_code)]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use crate::core::{PlatformBackendKind, PlatformCapabilities};
use crate::error::PlatformError;
use crate::event::Event;
use crate::monitor::MonitorInfo;
use crate::window::{WindowDescriptor, WindowId};

/// Contrato interno que todo backend de sistema operacional deve implementar.
pub(crate) trait Backend {
    fn capabilities(&self) -> PlatformCapabilities;
    fn monitors(&self) -> Vec<MonitorInfo>;
    fn primary_monitor(&self) -> Option<MonitorInfo>;

    /// Cria uma janela "leve": sem superfície real de SO, com estado
    /// mantido apenas em memória (usado pelo backend `headless`, para
    /// testes e CI).
    fn create_window(&self, descriptor: &WindowDescriptor) -> Result<WindowId, PlatformError>;
    fn destroy_window(&self, id: WindowId);

    /// Abre uma janela real do sistema operacional e assume o controle da
    /// thread atual, bombeando o loop de mensagens do SO até que o jogo
    /// decida encerrar (normalmente ao receber `WindowEvent::CloseRequested`).
    ///
    /// `on_event` recebe tanto eventos de janela (`Event::Window`) quanto
    /// de ciclo de vida da aplicação (`Event::App`). Backends sem janela
    /// real (`headless`) retornam [`PlatformError::Unsupported`].
    fn run(
        &self,
        descriptor: WindowDescriptor,
        on_event: Box<dyn FnMut(Event<()>)>,
    ) -> Result<(), PlatformError>;
}

pub(crate) type BackendHandle = Box<dyn Backend>;

pub(crate) fn create(kind: PlatformBackendKind) -> Result<BackendHandle, PlatformError> {
    match kind {
        PlatformBackendKind::Headless => Ok(Box::new(headless::HeadlessBackend::new())),
        PlatformBackendKind::Native => {
            #[cfg(target_os = "windows")]
            {
                Ok(Box::new(windows::WindowsBackend::new()))
            }
            #[cfg(not(target_os = "windows"))]
            {
                // Nenhum backend nativo real está implementado ainda para
                // este SO; cai para headless como base funcional até
                // linux/macos existirem.
                Ok(Box::new(headless::HeadlessBackend::new()))
            }
        }
    }
}
