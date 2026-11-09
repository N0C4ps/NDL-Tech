//! Implementações específicas de cada backend gráfico.
//!
//! Este módulo é **inteiramente privado**: nada aqui é exposto na API
//! pública da crate, do mesmo jeito que `ndl-platform::backend`. Os
//! módulos públicos acima (`core`, `buffer`, `pipeline`, ...) vão
//! programar contra um contrato interno comum implementado por cada
//! backend concreto, nunca contra um backend específico.
//!
//! Hoje só `directx` tem implementação real (Direct3D 11, via
//! `windows-rs`), disponível apenas no target Windows. Planejado se
//! estender depois a Vulkan, Metal e OpenGL, seguindo o mesmo padrão de
//! `ndl-platform` (um backend por plataforma/API, atrás de uma única API
//! pública consistente).

#[cfg(target_os = "windows")]
pub(crate) mod directx;

use crate::core::{AdapterInfo, GraphicsError};

pub(crate) fn enumerate_adapters() -> Result<Vec<AdapterInfo>, GraphicsError> {
    #[cfg(target_os = "windows")]
    {
        directx::enumerate_adapters()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(GraphicsError::Unsupported(
            "nenhum backend gráfico implementado para este SO ainda",
        ))
    }
}
