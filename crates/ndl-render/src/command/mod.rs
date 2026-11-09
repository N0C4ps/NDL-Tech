//! Gravação e execução de comandos.
//!
//! No D3D11, um contexto diferido é a mesma interface `ID3D11DeviceContext`
//! do contexto imediato — só que criado via `CreateDeferredContext` em vez
//! de ser o contexto principal do device, e seus comandos não executam na
//! GPU até serem passados para o contexto imediato via
//! `ExecuteCommandList`. Isso permite gravar comandos em outra thread
//! (ex: culling/batching de uma cena grande) enquanto a thread principal
//! segue desenhando.
//!
//! ## Estado atual
//!
//! [`CommandEncoder`] já grava comandos num contexto diferido próprio; sua API
//! reaproveita os métodos de [`crate::context::Context`] via `Deref`, mas seu
//! estado de bindings é independente do contexto imediato. `finish` produz
//! uma lista que a fila executa no contexto imediato, serializada com as
//! demais chamadas feitas por esta crate.
//! [`CommandBuffer`] também já é real.
//!
//! Operações mais específicas de gravação (`reset()`, `discard_recording()`,
//! `command_count()`, debug groups/markers) continuam reservadas.

use std::ops::Deref;

use crate::backend::{ActiveBackend, Backend};
use crate::context::Context;
use crate::core::{Device, GraphicsError};

/// Grava comandos de desenho/bind num contexto diferido, sem executá-los
/// na GPU imediatamente — ver [`CommandEncoder::finish`].
///
/// Os métodos de [`Context`] (`set_pipeline`, `draw`, `set_vertex_buffer`,
/// `clear_render_target`, ...) estão disponíveis via `Deref`. O encoder possui
/// um contexto e estado próprios; só a execução da lista final entra na
/// sequência compartilhada do contexto imediato.
pub struct CommandEncoder {
    context: Context,
}

impl CommandEncoder {
    pub(crate) fn from_context(context: Context) -> Self {
        Self { context }
    }

    /// Encerra a gravação e produz um [`CommandBuffer`], pronto para ser
    /// submetido numa [`crate::queue::GraphicsQueue`]. Nenhum comando
    /// gravado roda na GPU antes disso.
    pub fn finish(self) -> Result<CommandBuffer, GraphicsError> {
        ActiveBackend::finish_command_list(&self.context)
    }
}

impl Deref for CommandEncoder {
    type Target = Context;

    fn deref(&self) -> &Context {
        &self.context
    }
}

/// Uma lista de comandos já gravada e pronta para execução — ver
/// [`crate::queue::GraphicsQueue::submit`].
pub struct CommandBuffer {
    pub(crate) raw: <ActiveBackend as Backend>::RawCommandBuffer,
}

impl CommandBuffer {
    pub(crate) fn from_raw(raw: <ActiveBackend as Backend>::RawCommandBuffer) -> Self {
        Self { raw }
    }
}

impl Device {
    /// Cria um novo [`CommandEncoder`] com um contexto diferido próprio.
    pub fn create_command_encoder(&self) -> Result<CommandEncoder, GraphicsError> {
        ActiveBackend::create_command_encoder(self)
    }
}
