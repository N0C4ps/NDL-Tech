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
}

impl Buffer {
    #[cfg(target_os = "windows")]
    pub(crate) fn from_raw(
        raw: windows::Win32::Graphics::Direct3D11::ID3D11Buffer,
        size: u64,
        usage: BufferUsage,
        stride: u32,
    ) -> Self {
        Self {
            raw,
            size,
            usage,
            stride,
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
}

impl Device {
    /// Cria um buffer vazio (sem dados iniciais) — só faz sentido para
    /// buffers dinâmicos, que serão escritos depois.
    pub fn create_buffer(&self, desc: &BufferDescriptor) -> Result<Buffer, GraphicsError> {
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
