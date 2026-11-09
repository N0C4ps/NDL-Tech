//! Texturas 1D, 2D, 3D, cube e arrays.
//!
//! `ndl-render` só cria recursos de GPU a partir de dados já carregados —
//! quem lê arquivos de imagem do disco é `ndl-assets`; a conexão entre as
//! duas crates é feita pelo código do jogo.
//!
//! ## Estado atual
//!
//! [`Texture2D`] já é real, com upload de dados na criação via
//! [`Device::create_texture_2d`]. [`Device::create_render_target_texture`]
//! cria uma textura sem dados iniciais já pronta para ser usada como alvo
//! de cor (ex: um dos N attachments de um [`crate::render::RenderPass`]
//! com MRT) e, depois, lida como [`ShaderResourceView`] — o mesmo
//! raciocínio de [`crate::core::Device::create_depth_texture`]: o
//! conteúdo é escrito pela GPU durante o desenho, não enviado pela CPU.
//! `Texture1D`, `Texture3D`, `TextureCube` e arrays continuam reservados,
//! assim como mipmaps, MSAA e as operações de atualização/cópia depois da
//! criação (`write_texture`, `generate_mipmaps`, `resolve_texture`, ...).

use crate::core::{Device, GraphicsError};
use crate::format::Format;
use crate::view::{RenderTargetView, ShaderResourceView};

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
        if desc.width == 0 || desc.height == 0 {
            return Err(GraphicsError::InvalidArgument {
                name: "TextureDescriptor::extent",
                reason: "largura e altura precisam ser maiores que zero".into(),
            });
        }
        let limits = self.limits();
        if desc.width > limits.max_texture_dimension_2d
            || desc.height > limits.max_texture_dimension_2d
        {
            return Err(GraphicsError::InvalidArgument {
                name: "TextureDescriptor::extent",
                reason: format!(
                    "{}x{} excede o limite 2D {}x{}",
                    desc.width,
                    desc.height,
                    limits.max_texture_dimension_2d,
                    limits.max_texture_dimension_2d
                ),
            });
        }
        if desc.format.is_depth() {
            return Err(GraphicsError::IncompatibleResource {
                operation: "create_texture_2d",
                reason: "formatos depth/stencil requerem create_depth_texture".into(),
            });
        }
        if desc.format.is_compressed() {
            return Err(GraphicsError::FeatureNotSupported {
                feature: "upload inicial de formatos comprimidos ainda não implementado".into(),
            });
        }
        let expected = (desc.width as u64)
            .checked_mul(desc.height as u64)
            .and_then(|n| n.checked_mul((desc.format.bits_per_pixel() / 8) as u64))
            .ok_or_else(|| GraphicsError::InvalidArgument {
                name: "TextureDescriptor",
                reason: "dimensões causam overflow no cálculo do tamanho".into(),
            })?;
        if data.len() as u64 != expected {
            return Err(GraphicsError::DataSizeMismatch {
                expected,
                actual: data.len() as u64,
            });
        }
        let format_features = self.format_features(desc.format)?;
        if !format_features.texture_2d || !format_features.sampled {
            return Err(GraphicsError::FeatureNotSupported {
                feature: format!("textura 2D amostrável no formato {:?}", desc.format),
            });
        }
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

    /// Cria uma textura 2D sem dados iniciais, já com uma
    /// [`RenderTargetView`] pronta — para usar como alvo de desenho (ex:
    /// um attachment de cor extra num [`crate::render::RenderPass`] com
    /// MRT, ou um alvo intermediário de post-processing), e depois,
    /// opcionalmente, ler via [`Device::create_shader_resource_view`].
    pub fn create_render_target_texture(
        &self,
        desc: &TextureDescriptor,
    ) -> Result<(Texture2D, RenderTargetView), GraphicsError> {
        if desc.width == 0 || desc.height == 0 {
            return Err(GraphicsError::InvalidArgument {
                name: "TextureDescriptor::extent",
                reason: "largura e altura precisam ser maiores que zero".into(),
            });
        }
        let limits = self.limits();
        if desc.width > limits.max_texture_dimension_2d
            || desc.height > limits.max_texture_dimension_2d
        {
            return Err(GraphicsError::InvalidArgument {
                name: "TextureDescriptor::extent",
                reason: format!(
                    "{}x{} excede o limite 2D {}x{}",
                    desc.width,
                    desc.height,
                    limits.max_texture_dimension_2d,
                    limits.max_texture_dimension_2d
                ),
            });
        }
        if desc.format.is_depth() || desc.format.is_compressed() {
            return Err(GraphicsError::InvalidArgument {
                name: "TextureDescriptor::format",
                reason: "um alvo de renderização precisa de um formato colorido e não comprimido"
                    .into(),
            });
        }
        if !self.format_features(desc.format)?.render_target {
            return Err(GraphicsError::FeatureNotSupported {
                feature: format!("render target no formato {:?}", desc.format),
            });
        }
        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::create_render_target_texture(self, desc);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = desc;
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
