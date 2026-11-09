//! Consultas e estatísticas da GPU.
//!
//! ## Estado atual
//!
//! [`Query`] e [`QueryType::Occlusion`] já são reais — dá para saber
//! quantos pixels de um draw passaram o depth/scissor test (útil para
//! occlusion culling: pular objetos totalmente escondidos). O evento de
//! sincronização que [`crate::sync::GpuFence`] usa por baixo também é um
//! `Query` (`QueryType::Event`), mas essa API fica melhor exposta por lá.
//!
//! `Timestamp`/`TimestampDisjoint`, `PipelineStatistics`, stream output
//! statistics/overflow, e predicação (`set_predication`) continuam
//! reservados.

use crate::core::{Device, GraphicsError};

/// O tipo de informação que uma [`Query`] mede.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum QueryType {
    /// Quantos pixels passaram o teste de profundidade/scissor entre
    /// `begin_query`/`end_query`.
    Occlusion,
    /// Sem `begin_query` — só `end_query`, sinalizado quando a GPU
    /// processa até aquele ponto. Usado internamente por
    /// [`crate::sync::GpuFence`].
    Event,
}

/// Uma consulta de GPU já criada no dispositivo.
pub struct Query {
    #[cfg(target_os = "windows")]
    pub(crate) raw: windows::Win32::Graphics::Direct3D11::ID3D11Query,
    pub(crate) kind: QueryType,
}

impl Device {
    pub fn create_query(&self, kind: QueryType) -> Result<Query, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            crate::backend::directx::create_query(self, kind)
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = kind;
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
    }
}

impl crate::context::Context {
    /// Marca o início do intervalo medido por `query` — só válido para
    /// [`QueryType::Occlusion`] (e tipos de contagem similares);
    /// [`QueryType::Event`] não tem `begin`, só `end_query`.
    pub fn begin_query(&self, query: &Query) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.Begin(&query.raw);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = query;
        }
    }

    /// Marca o fim do intervalo medido por `query`.
    pub fn end_query(&self, query: &Query) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.End(&query.raw);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = query;
        }
    }

    /// Se a GPU já processou até o ponto marcado por `end_query`, sem
    /// ler nenhum resultado — funciona para qualquer tipo de `Query`,
    /// inclusive [`QueryType::Event`] (que não tem resultado de dados,
    /// só "pronto ou não").
    pub fn is_query_ready(&self, query: &Query) -> bool {
        #[cfg(target_os = "windows")]
        {
            crate::backend::directx::is_query_ready(self, query)
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = query;
            false
        }
    }

    /// Lê o resultado de uma query de oclusão (contagem de pixels), sem
    /// bloquear — `None` enquanto a GPU ainda não terminou.
    pub fn get_occlusion_result(&self, query: &Query) -> Option<u64> {
        debug_assert_eq!(query.kind, QueryType::Occlusion);
        #[cfg(target_os = "windows")]
        {
            crate::backend::directx::get_occlusion_result(self, query)
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = query;
            None
        }
    }
}
