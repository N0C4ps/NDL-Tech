//! Buffers gráficos: vertex, index, constant.
//!
//! ## Estado atual
//!
//! [`Buffer`] já é real, com uso genérico (vertex/index/constant/
//! structured) marcado por [`BufferUsage`]. `BufferUsage::Storage` é um
//! structured buffer com leitura/escrita por compute shader (UAV) — ver
//! [`crate::compute`]. `RawBuffer`, `IndirectBuffer` e `StagingBuffer`
//! continuam reservados.
//!
//! Operações de atualização (`write_buffer`, `map_buffer`, ...) também
//! continuam reservadas: por ora um buffer é escrito uma vez, na criação,
//! via [`crate::core::Device::create_buffer_with_data`].

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
    #[cfg(target_os = "windows")]
    pub(crate) raw: windows::Win32::Graphics::Direct3D11::ID3D11Buffer,
    size: u64,
    usage: BufferUsage,
    stride: u32,
    dynamic: bool,
}

impl Buffer {
    #[cfg(target_os = "windows")]
    pub(crate) fn from_raw(
        raw: windows::Win32::Graphics::Direct3D11::ID3D11Buffer,
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
        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::create_buffer(self, desc, None);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = desc;
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
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
        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::create_buffer(self, desc, Some(data));
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (desc, data);
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
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
        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::create_unordered_access_view(self, buffer);
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
    Ok(())
}
