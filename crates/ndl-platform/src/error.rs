use std::fmt;

/// Erros que podem ocorrer ao interagir com a camada de plataforma.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformError {
    /// A plataforma não conseguiu inicializar o backend do sistema operacional.
    InitializationFailed(String),
    /// A operação pedida não é suportada no backend/OS atual.
    Unsupported(&'static str),
    /// A janela referenciada não existe (já foi destruída ou id inválido).
    WindowNotFound,
    /// O monitor referenciado não existe.
    MonitorNotFound,
    /// Falha genérica de backend, com descrição livre.
    Backend(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformError::InitializationFailed(msg) => {
                write!(f, "falha ao inicializar a plataforma: {msg}")
            }
            PlatformError::Unsupported(what) => {
                write!(f, "operação não suportada neste backend: {what}")
            }
            PlatformError::WindowNotFound => write!(f, "janela não encontrada"),
            PlatformError::MonitorNotFound => write!(f, "monitor não encontrado"),
            PlatformError::Backend(msg) => write!(f, "erro de backend: {msg}"),
        }
    }
}

impl std::error::Error for PlatformError {}
