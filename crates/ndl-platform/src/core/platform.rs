use super::{ApplicationState, PlatformCapabilities};
use crate::backend::{self, BackendHandle};
use crate::error::PlatformError;
use crate::event::WindowEvent;
use crate::window::WindowDescriptor;

/// Sistema operacional detectado (ou forçado) em que a plataforma está rodando.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingSystem {
    Windows,
    Linux,
    MacOs,
    Unknown,
}

impl OperatingSystem {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            OperatingSystem::Windows
        } else if cfg!(target_os = "linux") {
            OperatingSystem::Linux
        } else if cfg!(target_os = "macos") {
            OperatingSystem::MacOs
        } else {
            OperatingSystem::Unknown
        }
    }
}

/// Qual backend concreto de sistema operacional a plataforma deve usar.
///
/// Por padrão, a plataforma escolhe o backend nativo do SO atual. `Headless`
/// é útil para testes e ambientes sem servidor gráfico (CI, por exemplo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlatformBackendKind {
    #[default]
    Native,
    Headless,
}

/// Configuração usada para inicializar a [`Platform`].
#[derive(Debug, Clone)]
pub struct PlatformConfig {
    pub application_name: String,
    pub backend: PlatformBackendKind,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            application_name: "ndl-app".to_string(),
            backend: PlatformBackendKind::default(),
        }
    }
}

/// Ponto de entrada da camada de plataforma: inicializa o backend do
/// sistema operacional e expõe suas capacidades.
///
/// `Platform` não cria janelas por conta própria — veja [`crate::window`]
/// para isso, usando um [`Platform`] já inicializado.
pub struct Platform {
    config: PlatformConfig,
    capabilities: PlatformCapabilities,
    state: ApplicationState,
    pub(crate) backend: BackendHandle,
}

impl Platform {
    /// Inicializa a plataforma de acordo com a configuração dada.
    pub fn new(config: PlatformConfig) -> Result<Self, PlatformError> {
        let backend = backend::create(config.backend)?;
        let capabilities = backend.capabilities();
        Ok(Self {
            config,
            capabilities,
            state: ApplicationState::Idle,
            backend,
        })
    }

    pub fn config(&self) -> &PlatformConfig {
        &self.config
    }

    pub fn capabilities(&self) -> PlatformCapabilities {
        self.capabilities
    }

    pub fn operating_system(&self) -> OperatingSystem {
        OperatingSystem::current()
    }

    pub fn state(&self) -> ApplicationState {
        self.state
    }

    #[allow(dead_code)] // usado quando o event loop passar a atualizar o estado a partir de AppEvent
    pub(crate) fn set_state(&mut self, state: ApplicationState) {
        self.state = state;
    }

    /// Abre uma janela **real** do sistema operacional e assume o controle
    /// da thread atual até que o jogo peça para encerrar (normalmente ao
    /// receber `WindowEvent::CloseRequested` em `on_event`).
    ///
    /// Esse é o caminho usado por um jogo de verdade — em contraste com
    /// [`Platform::create_window`] + [`crate::event::EventLoop`], que
    /// continuam existindo como caminho leve/testável (usado pelo backend
    /// `headless`). Backends sem janela real retornam
    /// [`PlatformError::Unsupported`].
    pub fn run_window(
        &self,
        descriptor: WindowDescriptor,
        on_event: impl FnMut(WindowEvent) + 'static,
    ) -> Result<(), PlatformError> {
        self.backend.run(descriptor, Box::new(on_event))
    }
}
