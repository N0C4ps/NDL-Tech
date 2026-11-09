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
//! [`CommandEncoder`] já é real: ele é um [`crate::context::Context`] por
//! baixo (os dois compartilham a mesma interface nativa), então todos os
//! métodos de desenho/bind de `Context` (`set_pipeline`, `draw`,
//! `set_vertex_buffer`, ...) já funcionam nele via `Deref` — a única
//! coisa exclusiva de `CommandEncoder` é [`CommandEncoder::finish`].
//! [`CommandBuffer`] também já é real.
//!
//! Operações mais específicas de gravação (`reset()`, `discard_recording()`,
//! `command_count()`, debug groups/markers) continuam reservadas.

use std::ops::Deref;

use crate::context::Context;
use crate::core::{Device, GraphicsError};

/// Grava comandos de desenho/bind num contexto diferido, sem executá-los
/// na GPU imediatamente — ver [`CommandEncoder::finish`].
///
/// Todos os métodos de [`Context`] (`set_pipeline`, `draw`,
/// `set_vertex_buffer`, `clear_render_target`, ...) estão disponíveis
/// aqui via `Deref`, porque gravar comandos é exatamente a mesma API de
/// emitir comandos — a diferença é só quando eles de fato rodam na GPU.
pub struct CommandEncoder {
    context: Context,
}

impl CommandEncoder {
    #[cfg(target_os = "windows")]
    pub(crate) fn from_context(context: Context) -> Self {
        Self { context }
    }

    /// Encerra a gravação e produz um [`CommandBuffer`], pronto para ser
    /// submetido numa [`crate::queue::GraphicsQueue`]. Nenhum comando
    /// gravado roda na GPU antes disso.
    pub fn finish(self) -> Result<CommandBuffer, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            crate::backend::directx::finish_command_list(&self.context)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
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
    #[cfg(target_os = "windows")]
    pub(crate) raw: windows::Win32::Graphics::Direct3D11::ID3D11CommandList,
}

impl CommandBuffer {
    #[cfg(target_os = "windows")]
    pub(crate) fn from_raw(raw: windows::Win32::Graphics::Direct3D11::ID3D11CommandList) -> Self {
        Self { raw }
    }
}

impl Device {
    /// Cria um novo [`CommandEncoder`] com um contexto diferido próprio.
    pub fn create_command_encoder(&self) -> Result<CommandEncoder, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            crate::backend::directx::create_command_encoder(self)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
    }
}
