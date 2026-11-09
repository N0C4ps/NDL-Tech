//! Consultas e estatísticas da GPU.
//!
//! ## Planejado
//!
//! ```ignore
//! pub enum QueryType {
//!     Event,
//!     Occlusion,
//!     OcclusionPredicate,
//!     Timestamp,
//!     TimestampDisjoint,
//!     PipelineStatistics,
//!     StreamOutputStatistics,
//!     StreamOutputOverflowPredicate,
//! }
//! ```
//!
//! (Também existem variantes por stream para certas consultas de stream
//! output.)
//!
//! Métodos: `create_query()`, `begin_query()`, `end_query()`,
//! `get_query_data()`, `try_get_query_data()`, `is_query_ready()`,
//! `begin_occlusion_query()`, `end_occlusion_query()`,
//! `get_occlusion_result()`, `write_timestamp()`,
//! `begin_timestamp_disjoint()`, `end_timestamp_disjoint()`,
//! `get_gpu_timestamp_frequency()`, `begin_pipeline_statistics()`,
//! `end_pipeline_statistics()`, `get_pipeline_statistics()`,
//! `set_predication()`, `get_predication()`.
//!
//! Algumas queries (timestamps, eventos) são encerradas com `End` sem um
//! `Begin` correspondente.
//!
//! Estatísticas úteis planejadas: vértices/primitivas processados pelo
//! input assembler; invocações de vertex/geometry/hull/domain/pixel/
//! compute shaders; primitivas emitidas pelo geometry shader; estatísticas
//! de clipping e stream output; tempo estimado de GPU via diferença entre
//! timestamps válidos.
//!
//! Ainda não implementado.
