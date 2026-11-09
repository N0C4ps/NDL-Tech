//! Buffers gráficos: vertex, index, constant.
//!
//! ## Estado atual
//!
//! [`Buffer`] já é real, com uso genérico (vertex/index/constant/
//! structured/indirect) marcado por [`BufferUsage`]. `BufferUsage::Storage`
//! é um structured buffer com leitura/escrita por compute shader (UAV) —
//! ver [`crate::compute`]. `BufferUsage::Indirect` guarda os argumentos de
//! um `draw`/`dispatch` indireto (ver [`DrawIndirectArgs`],
//! [`DrawIndexedIndirectArgs`] e [`crate::compute::DispatchIndirectArgs`]) —
//! hoje só escrito pela CPU (via `create_buffer_with_data` ou, se
//! `dynamic`, [`crate::context::Context::map_write_discard`]); um buffer
//! indirect gravável por compute shader (UAV) continua reservado. `RawBuffer`
//! continua reservado.
//!
//! Operações de atualização (`write_buffer`, `map_buffer`, ...) para
//! buffers não-dynamic também continuam reservadas: por ora um buffer
//! imutável é escrito uma vez, na criação, via
//! [`crate::core::Device::create_buffer_with_data`].

use crate::backend::{ActiveBackend, Backend};
use crate::core::{Device, GraphicsError};
use crate::view::UnorderedAccessView;

/// Para que um [`Buffer`] vai ser usado — determina a flag de bind do
/// backend (vertex/index/constant buffer do D3D11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferUsage {
    Vertex,
    Index,
    Constant,
    /// Structured buffer lido/escrito por compute shader (UAV) — usa
    /// `BufferDescriptor::stride` como o tamanho de cada elemento.
    Storage,
    /// Buffer de staging: sem bind a nenhum estágio, só para a CPU ler
    /// de volta dados que a GPU escreveu — ver
    /// [`crate::context::Context::copy_buffer`] +
    /// [`crate::context::Context::map_read`].
    Staging,
    /// Buffer de argumentos para `draw`/`dispatch` indireto — guarda os
    /// bytes lidos pela GPU ao emitir [`crate::context::Context::draw_indirect`],
    /// [`crate::context::Context::draw_indexed_indirect`] ou
    /// [`crate::context::Context::dispatch_indirect`], no lugar de contagens
    /// fixas passadas pela CPU a cada chamada.
    Indirect,
}

/// Configuração usada para criar um [`Buffer`].
#[derive(Debug, Clone, Copy)]
pub struct BufferDescriptor {
    pub size: u64,
    pub usage: BufferUsage,
    /// Se `true`, o buffer pode ser escrito pela CPU depois de criado
    /// (`D3D11_USAGE_DYNAMIC`); se `false`, é imutável depois da criação.
    pub dynamic: bool,
    /// Tamanho de cada elemento, em bytes — só usado (e obrigatório)
    /// quando `usage == BufferUsage::Storage`; ignorado nos outros casos.
    pub stride: u32,
}

/// Um buffer de GPU (vertex, index, constant ou structured).
pub struct Buffer {
    pub(crate) raw: <ActiveBackend as Backend>::RawBuffer,
    size: u64,
    usage: BufferUsage,
    stride: u32,
    dynamic: bool,
}

impl Buffer {
    pub(crate) fn from_raw(
        raw: <ActiveBackend as Backend>::RawBuffer,
        size: u64,
        usage: BufferUsage,
        stride: u32,
        dynamic: bool,
    ) -> Self {
        Self {
            raw,
            size,
            usage,
            stride,
            dynamic,
        }
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn usage(&self) -> BufferUsage {
        self.usage
    }

    /// Tamanho de cada elemento, em bytes — só relevante para
    /// `BufferUsage::Storage`.
    pub fn stride(&self) -> u32 {
        self.stride
    }

    pub fn is_dynamic(&self) -> bool {
        self.dynamic
    }
}

impl Device {
    /// Cria um buffer vazio (sem dados iniciais) — só faz sentido para
    /// buffers dinâmicos, que serão escritos depois.
    pub fn create_buffer(&self, desc: &BufferDescriptor) -> Result<Buffer, GraphicsError> {
        validate_descriptor(desc, self.limits().max_buffer_size)?;
        ActiveBackend::create_buffer(self, desc, None)
    }

    /// Cria um buffer já inicializado com `data` (deve ter `desc.size` bytes).
    pub fn create_buffer_with_data(
        &self,
        desc: &BufferDescriptor,
        data: &[u8],
    ) -> Result<Buffer, GraphicsError> {
        validate_descriptor(desc, self.limits().max_buffer_size)?;
        if data.len() as u64 != desc.size {
            return Err(GraphicsError::DataSizeMismatch {
                expected: desc.size,
                actual: data.len() as u64,
            });
        }
        ActiveBackend::create_buffer(self, desc, Some(data))
    }

    /// Cria uma [`UnorderedAccessView`] para `buffer`, que precisa ter
    /// sido criado com `BufferUsage::Storage`.
    pub fn create_unordered_access_view(
        &self,
        buffer: &Buffer,
    ) -> Result<UnorderedAccessView, GraphicsError> {
        if buffer.usage() != BufferUsage::Storage {
            return Err(GraphicsError::IncompatibleResource {
                operation: "create_unordered_access_view",
                reason: "o buffer precisa ter sido criado com BufferUsage::Storage".into(),
            });
        }
        if self.limits().max_storage_buffer_bindings == 0 {
            return Err(GraphicsError::FeatureNotSupported {
                feature: "unordered access views neste dispositivo".into(),
            });
        }
        ActiveBackend::create_unordered_access_view(self, buffer)
    }
}

fn validate_descriptor(desc: &BufferDescriptor, max_size: u64) -> Result<(), GraphicsError> {
    if desc.size == 0 {
        return Err(GraphicsError::InvalidArgument {
            name: "BufferDescriptor::size",
            reason: "deve ser maior que zero".into(),
        });
    }
    if desc.size > max_size || desc.size > u32::MAX as u64 {
        return Err(GraphicsError::InvalidArgument {
            name: "BufferDescriptor::size",
            reason: format!("{} excede o limite do backend ({max_size})", desc.size),
        });
    }
    if desc.usage == BufferUsage::Constant && (desc.size % 16 != 0 || desc.size > 65_536) {
        return Err(GraphicsError::InvalidArgument {
            name: "BufferDescriptor::size",
            reason: "constant buffers precisam ter tamanho múltiplo de 16 e no máximo 65536 bytes"
                .into(),
        });
    }
    if desc.usage == BufferUsage::Storage
        && (desc.stride == 0 || desc.size % desc.stride as u64 != 0)
    {
        return Err(GraphicsError::InvalidArgument {
            name: "BufferDescriptor::stride",
            reason: "storage buffers exigem stride > 0 e tamanho divisível pelo stride".into(),
        });
    }
    if desc.usage != BufferUsage::Storage && desc.stride != 0 {
        return Err(GraphicsError::InvalidArgument {
            name: "BufferDescriptor::stride",
            reason: "stride só pode ser diferente de zero em storage buffers".into(),
        });
    }
    if desc.usage == BufferUsage::Staging && desc.dynamic {
        return Err(GraphicsError::InvalidArgument {
            name: "BufferDescriptor::dynamic",
            reason: "staging buffers não podem ser dynamic".into(),
        });
    }
    if desc.usage == BufferUsage::Storage && desc.dynamic {
        return Err(GraphicsError::InvalidArgument {
            name: "BufferDescriptor::dynamic",
            reason: "storage buffers não podem usar mapeamento dynamic neste backend".into(),
        });
    }
    if desc.usage == BufferUsage::Indirect && desc.size % 4 != 0 {
        return Err(GraphicsError::InvalidArgument {
            name: "BufferDescriptor::size",
            reason: "indirect buffers precisam ter tamanho múltiplo de 4 bytes".into(),
        });
    }
    Ok(())
}

/// Argumentos de um [`crate::context::Context::draw_indirect`] — mesmo
/// layout binário do `D3D11_DRAW_INSTANCED_INDIRECT_ARGS`. Escreva
/// [`DrawIndirectArgs::to_bytes`] num [`Buffer`] com `BufferUsage::Indirect`
/// (ex: via [`crate::context::Context::map_write_discard`]) no offset
/// passado a `draw_indirect`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct DrawIndirectArgs {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

impl DrawIndirectArgs {
    /// Tamanho em bytes de um `DrawIndirectArgs` no buffer indirect.
    pub const SIZE: u64 = 16;

    /// Serializa os argumentos na ordem/largura exata que o D3D11 espera
    /// encontrar no buffer indirect (little-endian, 4 `u32` sequenciais).
    pub fn to_bytes(&self) -> [u8; Self::SIZE as usize] {
        let mut out = [0u8; Self::SIZE as usize];
        out[0..4].copy_from_slice(&self.vertex_count.to_le_bytes());
        out[4..8].copy_from_slice(&self.instance_count.to_le_bytes());
        out[8..12].copy_from_slice(&self.first_vertex.to_le_bytes());
        out[12..16].copy_from_slice(&self.first_instance.to_le_bytes());
        out
    }
}

/// Argumentos de um [`crate::context::Context::draw_indexed_indirect`] —
/// mesmo layout binário do `D3D11_DRAW_INDEXED_INSTANCED_INDIRECT_ARGS`.
/// Ver [`DrawIndirectArgs`] para como escrevê-los num buffer indirect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct DrawIndexedIndirectArgs {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
}

impl DrawIndexedIndirectArgs {
    /// Tamanho em bytes de um `DrawIndexedIndirectArgs` no buffer indirect.
    pub const SIZE: u64 = 20;

    /// Serializa os argumentos na ordem/largura exata que o D3D11 espera
    /// encontrar no buffer indirect (little-endian, 5 campos de 4 bytes
    /// sequenciais).
    pub fn to_bytes(&self) -> [u8; Self::SIZE as usize] {
        let mut out = [0u8; Self::SIZE as usize];
        out[0..4].copy_from_slice(&self.index_count.to_le_bytes());
        out[4..8].copy_from_slice(&self.instance_count.to_le_bytes());
        out[8..12].copy_from_slice(&self.first_index.to_le_bytes());
        out[12..16].copy_from_slice(&self.base_vertex.to_le_bytes());
        out[16..20].copy_from_slice(&self.first_instance.to_le_bytes());
        out
    }
}
