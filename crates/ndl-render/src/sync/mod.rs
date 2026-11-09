//! Sincronização CPU/GPU.
//!
//! O D3D11 não oferece, em todas as versões, o mesmo modelo de fences e
//! semáforos do Vulkan — [`GpuFence`] é construído sobre uma
//! [`crate::query::Query`] de evento (`End`/`GetData` polling), não
//! promete fences nativas.
//!
//! ## Estado atual
//!
//! [`GpuFence`] já é real: `signal`, `is_signaled` (não bloqueia) e
//! `wait` (bloqueia). [`crate::queue::GraphicsQueue::wait_idle`] usa a
//! mesma técnica internamente. Fences nativas D3D11.4
//! (`create_fence`/`signal_fence`/`wait_fence`, quando a interface
//! estiver disponível) continuam reservadas.

use crate::context::Context;
use crate::core::{Device, GraphicsError};
use crate::query::{Query, QueryType};

/// Um ponto de sincronização entre CPU e GPU: `signal` marca um ponto no
/// fluxo de comandos, `wait`/`is_signaled` checam quando a GPU chegou lá.
pub struct GpuFence {
    query: Query,
}

impl Device {
    pub fn create_fence(&self) -> Result<GpuFence, GraphicsError> {
        Ok(GpuFence {
            query: self.create_query(QueryType::Event)?,
        })
    }
}

impl GpuFence {
    /// Marca este ponto no fluxo de comandos de `context` — a GPU sinaliza
    /// a fence quando terminar de processar tudo até aqui.
    pub fn signal(&self, context: &Context) {
        context.end_query(&self.query);
    }

    /// Se a GPU já processou até o ponto marcado por [`GpuFence::signal`].
    /// Não bloqueia.
    pub fn is_signaled(&self, context: &Context) -> bool {
        context.is_query_ready(&self.query)
    }

    /// Bloqueia a thread atual até a GPU processar até o ponto marcado
    /// por [`GpuFence::signal`].
    pub fn wait(&self, context: &Context) {
        while !self.is_signaled(context) {
            std::thread::yield_now();
        }
    }
}
