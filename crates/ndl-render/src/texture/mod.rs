//! Texturas 1D, 2D, 3D, cube e arrays.
//!
//! `ndl-render` só cria recursos de GPU a partir de dados já carregados —
//! quem lê arquivos de imagem do disco é `ndl-assets`; a conexão entre as
//! duas crates é feita pelo código do jogo.
//!
//! ## Estado atual
//!
//! [`Texture2D`] já é real, com upload de dados na criação. `Texture1D`,
//! `Texture3D`, `TextureCube` e arrays continuam reservados, assim como
//! mipmaps, MSAA e as operações de atualização/cópia depois da criação
//! (`write_texture`, `generate_mipmaps`, `resolve_texture`, ...).

use crate::core::{Device, GraphicsError};
use crate::format::Format;
use crate::view::ShaderResourceView;

/// Configuração usada para criar uma [`Texture2D`].
#[derive(Debug, Clone, Copy)]
pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub format: Format,
}

/// Uma textura 2D — hoje sempre com 1 nível de mip, 1 camada de array e
/// sem multisampling.
pub struct Texture2D {
    #[cfg(target_os = "windows")]
    pub(crate) raw: windows::Win32::Graphics::Direct3D11::ID3D11Texture2D,
    width: u32,
    height: u32,
    format: Format,
}

impl Texture2D {
    #[cfg(target_os = "windows")]
    pub(crate) fn from_raw(
        raw: windows::Win32::Graphics::Direct3D11::ID3D11Texture2D,
        width: u32,
        height: u32,
        format: Format,
    ) -> Self {
        Self {
            raw,
            width,
            height,
            format,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn format(&self) -> Format {
        self.format
    }
}

impl Device {
    /// Cria uma textura 2D a partir de dados já carregados em memória
    /// (`data`, linha a linha, sem padding entre linhas — `width * height
    /// * bytes_per_pixel(format)` bytes).
    pub fn create_texture_2d(
        &self,
        desc: &TextureDescriptor,
        data: &[u8],
    ) -> Result<Texture2D, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::create_texture_2d(self, desc, data);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (desc, data);
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
    }

    /// Cria uma [`ShaderResourceView`] padrão (toda a textura, formato
    /// original) para amostrar `texture` num shader.
    pub fn create_shader_resource_view(
        &self,
        texture: &Texture2D,
    ) -> Result<ShaderResourceView, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::create_shader_resource_view(self, texture);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = texture;
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
    }
}
