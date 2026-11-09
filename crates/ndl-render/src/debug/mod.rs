//! Diagnóstico e instrumentação de renderização.
//!
//! **Não confundir com `ndl-diagnostics`** (telemetria/profiling genérico
//! do ecossistema) — este módulo é especificamente sobre depurar o
//! backend gráfico: grupos e marcadores visíveis em ferramentas como
//! RenderDoc e PIX.
//!
//! ## Estado atual
//!
//! [`crate::context::Context::push_debug_group`]/`pop_debug_group`/
//! `insert_debug_marker` já são reais, via `ID3DUserDefinedAnnotation`.
//! A debug layer do D3D11 (`enable_debug_layer` — já parcialmente coberto
//! por [`crate::core::DeviceDescriptor::enable_debug_layer`]),
//! `InfoQueue`/mensagens de validação, e `set_debug_name`/`get_debug_name`
//! genéricos (ver [`crate::buffer::Buffer::set_label`] para o equivalente
//! já real, específico de buffer/textura) continuam reservados.

use crate::backend::{ActiveBackend, Backend};
use crate::context::Context;

impl Context {
    /// Abre um grupo de depuração nomeado — ferramentas como RenderDoc/PIX
    /// mostram os comandos entre este `push` e o `pop_debug_group`
    /// correspondente agrupados sob `label`. Grupos podem ser aninhados.
    pub fn push_debug_group(&self, label: &str) {
        ActiveBackend::push_debug_group(self, label);
    }

    /// Fecha o grupo de depuração mais recente aberto por
    /// [`Context::push_debug_group`].
    pub fn pop_debug_group(&self) {
        ActiveBackend::pop_debug_group(self);
    }

    /// Insere um marcador pontual (sem duração) na timeline de comandos,
    /// visível em ferramentas de debug gráfico.
    pub fn insert_debug_marker(&self, label: &str) {
        ActiveBackend::insert_debug_marker(self, label);
    }
}
