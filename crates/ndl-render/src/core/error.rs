use std::fmt;

/// Erros que podem ocorrer ao interagir com a camada gráfica.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphicsError {
    /// O backend não conseguiu inicializar o dispositivo gráfico.
    InitializationFailed(String),
    /// A operação pedida não é suportada pelo backend/hardware atual.
    Unsupported(&'static str),
    /// Nenhum adaptador (GPU) compatível foi encontrado.
    AdapterNotFound,
    /// O dispositivo foi perdido (ex: driver crashou, GPU removida).
    DeviceLost,
    /// Falha genérica de backend, com descrição livre.
    Backend(String),
}

impl fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphicsError::InitializationFailed(msg) => {
                write!(f, "falha ao inicializar o dispositivo gráfico: {msg}")
            }
            GraphicsError::Unsupported(what) => {
                write!(f, "operação não suportada neste backend/hardware: {what}")
            }
            GraphicsError::AdapterNotFound => write!(f, "nenhum adaptador gráfico encontrado"),
            GraphicsError::DeviceLost => write!(f, "o dispositivo gráfico foi perdido"),
            GraphicsError::Backend(msg) => write!(f, "erro de backend gráfico: {msg}"),
        }
    }
}

impl std::error::Error for GraphicsError {}
