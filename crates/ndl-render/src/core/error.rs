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
    /// Um descritor ou argumento viola as regras da API.
    InvalidArgument { name: &'static str, reason: String },
    /// Os dados recebidos não têm o tamanho exigido pelo recurso.
    DataSizeMismatch { expected: u64, actual: u64 },
    /// Uma operação foi solicitada para um recurso incompatível.
    IncompatibleResource {
        operation: &'static str,
        reason: String,
    },
    /// Um índice ou intervalo está fora dos limites do recurso.
    OutOfBounds {
        resource: &'static str,
        offset: u64,
        size: u64,
        limit: u64,
    },
    /// O dispositivo ou backend não concede o recurso solicitado.
    FeatureNotSupported { feature: String },
    /// O recurso ainda está em uso e precisa de sincronização antes da operação.
    ResourceBusy { resource: &'static str },
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
            GraphicsError::InvalidArgument { name, reason } => write!(f, "argumento inválido `{name}`: {reason}"),
            GraphicsError::DataSizeMismatch { expected, actual } => write!(f, "tamanho de dados inválido: esperado {expected} bytes, recebido {actual} bytes"),
            GraphicsError::IncompatibleResource { operation, reason } => write!(f, "recurso incompatível para `{operation}`: {reason}"),
            GraphicsError::OutOfBounds { resource, offset, size, limit } => write!(f, "intervalo fora dos limites de `{resource}`: offset {offset} + tamanho {size}, limite {limit}"),
            GraphicsError::FeatureNotSupported { feature } => write!(f, "recurso não suportado pelo dispositivo/backend: {feature}"),
            GraphicsError::ResourceBusy { resource } => write!(f, "recurso `{resource}` ainda está em uso pela GPU"),
        }
    }
}

impl std::error::Error for GraphicsError {}
