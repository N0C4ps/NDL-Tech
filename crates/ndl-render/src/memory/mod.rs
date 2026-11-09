//! Mapeamento, staging e upload.
//!
//! Separa duas coisas: acesso CPU a recursos do backend (mapeamento) e
//! gerenciamento de alocações de alto nível da própria crate.
//!
//! ## Estado atual
//!
//! [`MappedBuffer`] (guarda RAII) e [`crate::context::Context::map_write_discard`]
//! já são reais — desmapeiam automaticamente no `Drop`, como planejado
//! desde o início. [`ConstantBufferRing`] já é real, construído por cima
//! disso: evita que atualizar um constant buffer todo quadro force a GPU
//! a esperar a leitura anterior terminar, girando entre N buffers em vez
//! de reescrever sempre o mesmo.
//!
//! `UploadArena`, `StagingPool`, `TransientResourcePool` e `ResourceCache`
//! continuam reservados.

use crate::buffer::{Buffer, BufferDescriptor, BufferUsage};
use crate::context::Context;
use crate::core::{Device, GraphicsError};

/// Um buffer dinâmico mapeado para escrita da CPU — desmapeia
/// automaticamente quando sai de escopo.
///
/// ```ignore
/// let mut mapped = context.map_write_discard(&buffer)?;
/// mapped.copy_from_slice(&data);
/// // Unmap automático aqui, no Drop.
/// ```
pub struct MappedBuffer<'a> {
    pub(crate) context: &'a Context,
    pub(crate) buffer: &'a Buffer,
    #[cfg(target_os = "windows")]
    pub(crate) ptr: *mut u8,
    pub(crate) size: usize,
}

impl MappedBuffer<'_> {
    /// Copia `data` para o início da região mapeada. `data` não pode ser
    /// maior que o tamanho do buffer.
    pub fn copy_from_slice(&mut self, data: &[u8]) {
        assert!(
            data.len() <= self.size,
            "dados ({} bytes) maiores que o buffer mapeado ({} bytes)",
            data.len(),
            self.size
        );
        #[cfg(target_os = "windows")]
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.ptr, data.len());
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = data;
        }
    }
}

impl Drop for MappedBuffer<'_> {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        {
            crate::backend::directx::unmap_buffer(self.context, self.buffer);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (self.context, self.buffer);
        }
    }
}

impl Context {
    /// Mapeia `buffer` (que precisa ter sido criado com
    /// `BufferDescriptor::dynamic = true`) para escrita da CPU,
    /// descartando o conteúdo anterior — o backend pode devolver uma
    /// região de memória nova em vez de esperar a GPU liberar a antiga.
    pub fn map_write_discard<'a>(&'a self, buffer: &'a Buffer) -> Result<MappedBuffer<'a>, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            crate::backend::directx::map_write_discard(self, buffer)
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = buffer;
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
    }
}

/// Um conjunto de N constant buffers idênticos, usados em rotação —
/// evita que atualizar o mesmo constant buffer todo quadro force a GPU a
/// esperar o quadro anterior terminar de ler antes de aceitar a escrita.
pub struct ConstantBufferRing {
    buffers: Vec<Buffer>,
    next_index: usize,
}

impl ConstantBufferRing {
    /// Cria um ring com `frame_count` buffers de `buffer_size` bytes cada
    /// (ex: `frame_count = 2` ou `3`, casando com o número de buffers da
    /// swapchain).
    pub fn new(device: &Device, buffer_size: u64, frame_count: usize) -> Result<Self, GraphicsError> {
        let mut buffers = Vec::with_capacity(frame_count);
        for _ in 0..frame_count {
            buffers.push(device.create_buffer(&BufferDescriptor {
                size: buffer_size,
                usage: BufferUsage::Constant,
                dynamic: true,
            })?);
        }
        Ok(Self {
            buffers,
            next_index: 0,
        })
    }

    /// Escreve `data` no próximo buffer do ring (via
    /// [`Context::map_write_discard`]) e o retorna, pronto para
    /// `Context::set_constant_buffer`.
    pub fn write(&mut self, context: &Context, data: &[u8]) -> Result<&Buffer, GraphicsError> {
        let index = self.next_index;
        self.next_index = (self.next_index + 1) % self.buffers.len();

        let buffer = &self.buffers[index];
        let mut mapped = context.map_write_discard(buffer)?;
        mapped.copy_from_slice(data);
        drop(mapped);

        Ok(&self.buffers[index])
    }
}
