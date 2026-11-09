//! Fila gráfica lógica e submissão.
//!
//! O D3D11 não tem filas gráficas, de computação e de cópia separadas
//! como o D3D12/Vulkan — este módulo oferece uma [`GraphicsQueue`] lógica
//! única, por cima do contexto imediato do device, em vez de fingir que
//! existem filas nativas independentes.
//!
//! ## Estado atual
//!
//! [`GraphicsQueue`] já é real: `submit` executa
//! [`crate::command::CommandBuffer`]s gravados por um
//! [`crate::command::CommandEncoder`], `flush` força o envio de comandos
//! pendentes, e `wait_idle` espera a GPU terminar via uma query de evento
//! (não existe um `Flush()` que bloqueie — isso é construído por cima de
//! `End`/`GetData` polling).

use crate::backend::{ActiveBackend, Backend};
use crate::command::CommandBuffer;
use crate::core::{Device, GraphicsError};

/// Fila lógica por onde o jogo submete trabalho gravado para a GPU.
pub struct GraphicsQueue {
    context: crate::context::Context,
}

impl GraphicsQueue {
    pub(crate) fn from_context(context: crate::context::Context) -> Self {
        Self { context }
    }

    /// Executa, em ordem, cada [`CommandBuffer`] gravado por um
    /// [`crate::command::CommandEncoder`].
    pub fn submit(&self, buffers: &[CommandBuffer]) -> Result<(), GraphicsError> {
        let _guard = self.context.lock_context();
        for buffer in buffers {
            ActiveBackend::execute_command_list(&self.context, buffer);
        }
        Ok(())
    }

    /// Força o envio de qualquer comando ainda pendente no contexto
    /// imediato para a GPU, sem esperar a execução terminar.
    pub fn flush(&self) {
        self.context.flush();
    }

    /// Bloqueia até a GPU terminar de processar todo o trabalho submetido
    /// até agora.
    pub fn wait_idle(&self) -> Result<(), GraphicsError> {
        let _guard = self.context.lock_context();
        ActiveBackend::wait_idle(&self.context)
    }
}

impl Device {
    /// A fila gráfica deste dispositivo, por cima do seu contexto
    /// imediato.
    pub fn queue(&self) -> GraphicsQueue {
        GraphicsQueue::from_context(self.context())
    }
}
