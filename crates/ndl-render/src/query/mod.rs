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

use crate::backend::{ActiveBackend, Backend};
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
    pub(crate) raw: <ActiveBackend as Backend>::RawQuery,
    pub(crate) kind: QueryType,
}

impl Device {
    pub fn create_query(&self, kind: QueryType) -> Result<Query, GraphicsError> {
        ActiveBackend::create_query(self, kind)
    }
}

impl crate::context::Context {
    /// Marca o início do intervalo medido por `query` — só válido para
    /// [`QueryType::Occlusion`] (e tipos de contagem similares);
    /// [`QueryType::Event`] não tem `begin`, só `end_query`.
    pub fn begin_query(&self, query: &Query) {
        ActiveBackend::begin_query(self, query);
    }

    /// Marca o fim do intervalo medido por `query`.
    pub fn end_query(&self, query: &Query) {
        ActiveBackend::end_query(self, query);
    }

    /// Se a GPU já processou até o ponto marcado por `end_query`, sem
    /// ler nenhum resultado — funciona para qualquer tipo de `Query`,
    /// inclusive [`QueryType::Event`] (que não tem resultado de dados,
    /// só "pronto ou não").
    pub fn is_query_ready(&self, query: &Query) -> bool {
        ActiveBackend::is_query_ready(self, query)
    }

    /// Lê o resultado de uma query de oclusão (contagem de pixels), sem
    /// bloquear — `None` enquanto a GPU ainda não terminou.
    pub fn get_occlusion_result(&self, query: &Query) -> Option<u64> {
        debug_assert_eq!(query.kind, QueryType::Occlusion);
        ActiveBackend::get_occlusion_result(self, query)
    }
}
