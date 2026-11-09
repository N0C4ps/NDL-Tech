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

use crate::backend::{ActiveBackend, Backend};
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
    pub(crate) ptr: *mut u8,
    pub(crate) size: usize,
}

impl MappedBuffer<'_> {
    /// Copia `data` para o início da região mapeada. `data` não pode ser
    /// maior que o tamanho do buffer.
    pub fn copy_from_slice(&mut self, data: &[u8]) -> Result<(), GraphicsError> {
        if data.len() > self.size {
            return Err(GraphicsError::OutOfBounds {
                resource: "MappedBuffer",
                offset: 0,
                size: data.len() as u64,
                limit: self.size as u64,
            });
        }
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.ptr, data.len());
        }
        Ok(())
    }
}

impl Drop for MappedBuffer<'_> {
    fn drop(&mut self) {
        ActiveBackend::unmap_buffer(self.context, self.buffer);
    }
}

impl Context {
    /// Mapeia `buffer` (que precisa ter sido criado com
    /// `BufferDescriptor::dynamic = true`) para escrita da CPU,
    /// descartando o conteúdo anterior — o backend pode devolver uma
    /// região de memória nova em vez de esperar a GPU liberar a antiga.
    pub fn map_write_discard<'a>(
        &'a self,
        buffer: &'a Buffer,
    ) -> Result<MappedBuffer<'a>, GraphicsError> {
        if !buffer.is_dynamic() || buffer.usage() == BufferUsage::Staging {
            return Err(GraphicsError::IncompatibleResource {
                operation: "map_write_discard",
                reason: "exige buffer dynamic que não seja staging".into(),
            });
        }
        ActiveBackend::map_write_discard(self, buffer)
    }

    /// Mapeia `buffer` (que precisa ter sido criado com
    /// `BufferUsage::Staging`) para leitura da CPU — o jeito de ler de
    /// volta dados que a GPU escreveu (ex: o resultado de um compute
    /// shader), depois de um [`Context::copy_buffer`] para um buffer de
    /// staging.
    pub fn map_read<'a>(
        &'a self,
        buffer: &'a Buffer,
    ) -> Result<ReadMappedBuffer<'a>, GraphicsError> {
        if buffer.usage() != BufferUsage::Staging {
            return Err(GraphicsError::IncompatibleResource {
                operation: "map_read",
                reason: "leitura pela CPU exige BufferUsage::Staging".into(),
            });
        }
        ActiveBackend::map_read(self, buffer)
    }
}

/// Um buffer de staging mapeado para leitura da CPU — desmapeia
/// automaticamente quando sai de escopo.
pub struct ReadMappedBuffer<'a> {
    pub(crate) context: &'a Context,
    pub(crate) buffer: &'a Buffer,
    pub(crate) ptr: *const u8,
    pub(crate) size: usize,
}

impl ReadMappedBuffer<'_> {
    /// Os bytes do buffer mapeado.
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.size) }
    }
}

impl Drop for ReadMappedBuffer<'_> {
    fn drop(&mut self) {
        ActiveBackend::unmap_buffer(self.context, self.buffer);
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
    pub fn new(
        device: &Device,
        buffer_size: u64,
        frame_count: usize,
    ) -> Result<Self, GraphicsError> {
        if frame_count == 0 {
            return Err(GraphicsError::InvalidArgument {
                name: "frame_count",
                reason: "deve ser maior que zero".into(),
            });
        }
        let mut buffers = Vec::with_capacity(frame_count);
        for _ in 0..frame_count {
            buffers.push(device.create_buffer(&BufferDescriptor {
                size: buffer_size,
                usage: BufferUsage::Constant,
                dynamic: true,
                stride: 0,
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
        let buffer_size = self.buffers.first().map(Buffer::size).unwrap_or(0);
        if data.len() as u64 > buffer_size {
            return Err(GraphicsError::OutOfBounds {
                resource: "ConstantBufferRing",
                offset: 0,
                size: data.len() as u64,
                limit: buffer_size,
            });
        }
        let index = self.next_index;
        self.next_index = (self.next_index + 1) % self.buffers.len();

        let buffer = &self.buffers[index];
        let mut mapped = context.map_write_discard(buffer)?;
        mapped.copy_from_slice(data)?;
        drop(mapped);

        Ok(&self.buffers[index])
    }
}
