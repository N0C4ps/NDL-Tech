//! Texturas 1D, 2D, 3D, cube e arrays.
//!
//! `ndl-render` só cria recursos de GPU a partir de dados já carregados —
//! quem lê arquivos de imagem do disco é `ndl-assets`; a conexão entre as
//! duas crates é feita pelo código do jogo.
//!
//! ## Estado atual
//!
//! [`Texture2D`] já é real, com upload de dados na criação via
//! [`Device::create_texture_2d`] — incluindo múltiplos níveis de mip
//! (`TextureDescriptor::mip_levels`) e múltiplas camadas de array
//! (`TextureDescriptor::array_layers`), os dois necessários para uma
//! cena 3D de verdade sem aliasing/cache-miss de textura à distância.
//! `mip_levels = 0` pede uma cadeia completa gerada na GPU a partir só do
//! nível base — ver [`Context::generate_mips`] — em vez de uma cadeia já
//! pronta fornecida pelo chamador. [`Device::create_render_target_texture`]
//! cria uma textura sem dados iniciais já pronta para ser usada como alvo
//! de cor (ex: um dos N attachments de um [`crate::render::RenderPass`]
//! com MRT) e, depois, lida como [`ShaderResourceView`] — o mesmo
//! raciocínio de [`crate::core::Device::create_depth_texture`]: o
//! conteúdo é escrito pela GPU durante o desenho, não enviado pela CPU.
//! `Texture1D`, `Texture3D`, `TextureCube`, MSAA e as operações de
//! atualização/cópia depois da criação (`write_texture`,
//! `resolve_texture`, ...) continuam reservados.

use crate::context::Context;
use crate::core::{Device, GraphicsError};
use crate::format::Format;
use crate::view::{RenderTargetView, ShaderResourceView};

/// Configuração usada para criar uma [`Texture2D`].
#[derive(Debug, Clone, Copy)]
pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub format: Format,
    /// Número de níveis de mip. `1` = só a base, sem cadeia. `0` = pede
    /// uma cadeia completa (até 1x1) gerada na GPU — os dados iniciais
    /// passados a [`Device::create_texture_2d`], nesse caso, cobrem só o
    /// nível base (um por camada de array); chame
    /// [`Context::generate_mips`] depois de criar a
    /// [`crate::view::ShaderResourceView`] para preencher o resto. Qualquer
    /// outro valor é uma cadeia já pronta — o chamador fornece todos os
    /// níveis.
    pub mip_levels: u32,
    /// Número de camadas de array. `1` = textura 2D simples.
    pub array_layers: u32,
}

/// Uma textura 2D — hoje sem multisampling.
pub struct Texture2D {
    #[cfg(target_os = "windows")]
    pub(crate) raw: windows::Win32::Graphics::Direct3D11::ID3D11Texture2D,
    width: u32,
    height: u32,
    format: Format,
    mip_levels: u32,
    array_layers: u32,
}

impl Texture2D {
    #[cfg(target_os = "windows")]
    pub(crate) fn from_raw(
        raw: windows::Win32::Graphics::Direct3D11::ID3D11Texture2D,
        width: u32,
        height: u32,
        format: Format,
        mip_levels: u32,
        array_layers: u32,
    ) -> Self {
        Self {
            raw,
            width,
            height,
            format,
            mip_levels,
            array_layers,
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

    /// Número real de níveis de mip (nunca `0`, mesmo quando
    /// `TextureDescriptor::mip_levels` pediu `0` — já resolvido para a
    /// contagem completa calculada pela GPU).
    pub fn mip_levels(&self) -> u32 {
        self.mip_levels
    }

    pub fn array_layers(&self) -> u32 {
        self.array_layers
    }
}

/// Quantos níveis de mip uma textura `max(width, height)` texels no maior
/// lado tem até chegar em 1x1 — o mesmo valor que o D3D11 calcula quando
/// `MipLevels = 0`.
fn full_mip_chain_levels(width: u32, height: u32) -> u32 {
    32 - width.max(height).max(1).leading_zeros()
}

/// Índice de subresource D3D11 para o nível `mip` da camada `layer`,
/// numa textura com `mip_levels` níveis — a mesma convenção usada por
/// `pInitialData` em `CreateTexture2D` (todos os mips da camada 0, depois
/// todos os mips da camada 1, ...).
pub(crate) fn mip_subresource_index(mip: u32, layer: u32, mip_levels: u32) -> u32 {
    mip + layer * mip_levels
}

/// Dimensões do nível de mip `level` de uma textura `width`x`height` — a
/// base dividida por `2^level`, nunca menor que `1`.
pub(crate) fn mip_dimensions(width: u32, height: u32, level: u32) -> (u32, u32) {
    ((width >> level).max(1), (height >> level).max(1))
}

impl Device {
    /// Cria uma textura 2D a partir de dados já carregados em memória.
    ///
    /// `mip_data` tem um slice por subresource, na convenção do D3D11:
    /// índice `mip + layer * mip_levels` (todos os níveis da camada 0,
    /// depois todos os da camada 1, ...) — ver
    /// [`TextureDescriptor::mip_levels`]/[`TextureDescriptor::array_layers`].
    /// Cada slice tem `largura_do_nível * altura_do_nível *
    /// bytes_per_pixel(format)` bytes (a largura/altura do nível já
    /// divididas por `2^mip`, nunca menores que `1`), sem padding de
    /// linha. Para o caso comum (sem mip chain, sem array) é um
    /// `&[dados]` de um elemento só.
    ///
    /// Quando `desc.mip_levels == 0`, `mip_data` tem um elemento por
    /// camada (só o nível base) — o resto da cadeia fica para
    /// [`Context::generate_mips`] preencher na GPU.
    pub fn create_texture_2d(
        &self,
        desc: &TextureDescriptor,
        mip_data: &[&[u8]],
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
        if desc.array_layers == 0 {
            return Err(GraphicsError::InvalidArgument {
                name: "TextureDescriptor::array_layers",
                reason: "precisa ser maior que zero".into(),
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
        let max_levels = full_mip_chain_levels(desc.width, desc.height);
        let generated_chain = desc.mip_levels == 0;
        if !generated_chain && desc.mip_levels > max_levels {
            return Err(GraphicsError::InvalidArgument {
                name: "TextureDescriptor::mip_levels",
                reason: format!(
                    "{}x{} só comporta até {max_levels} níveis de mip, pedido {}",
                    desc.width, desc.height, desc.mip_levels
                ),
            });
        }
        if generated_chain {
            if !self.format_features(desc.format)?.render_target {
                return Err(GraphicsError::FeatureNotSupported {
                    feature: format!(
                        "geração de mipmaps na GPU no formato {:?} (precisa também ser usável como render target)",
                        desc.format
                    ),
                });
            }
        } else {
            let format_features = self.format_features(desc.format)?;
            if !format_features.texture_2d || !format_features.sampled {
                return Err(GraphicsError::FeatureNotSupported {
                    feature: format!("textura 2D amostrável no formato {:?}", desc.format),
                });
            }
        }

        let expected_subresources = if generated_chain {
            desc.array_layers
        } else {
            desc.mip_levels * desc.array_layers
        } as usize;
        if mip_data.len() != expected_subresources {
            return Err(GraphicsError::InvalidArgument {
                name: "mip_data",
                reason: format!(
                    "esperava {expected_subresources} subresource(s) ({}), recebeu {}",
                    if generated_chain {
                        "um nível base por camada".to_string()
                    } else {
                        format!("{} mip(s) x {} camada(s)", desc.mip_levels, desc.array_layers)
                    },
                    mip_data.len()
                ),
            });
        }
        let bytes_per_pixel = (desc.format.bits_per_pixel() / 8) as u64;
        for (index, slice) in mip_data.iter().enumerate() {
            let mip = if generated_chain { 0 } else { index as u32 % desc.mip_levels };
            let (mip_w, mip_h) = mip_dimensions(desc.width, desc.height, mip);
            let expected = (mip_w as u64)
                .checked_mul(mip_h as u64)
                .and_then(|n| n.checked_mul(bytes_per_pixel))
                .ok_or_else(|| GraphicsError::InvalidArgument {
                    name: "TextureDescriptor",
                    reason: "dimensões causam overflow no cálculo do tamanho".into(),
                })?;
            if slice.len() as u64 != expected {
                return Err(GraphicsError::DataSizeMismatch {
                    expected,
                    actual: slice.len() as u64,
                });
            }
        }

        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::create_texture_2d(self, desc, mip_data);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (desc, mip_data);
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
    ///
    /// Hoje só `desc.mip_levels == 1` e `desc.array_layers == 1` são
    /// aceitos — um alvo de renderização com mip chain própria (ex: para
    /// post-processing com downsample progressivo) ou em array continua
    /// reservado.
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
        if desc.mip_levels != 1 || desc.array_layers != 1 {
            return Err(GraphicsError::FeatureNotSupported {
                feature: "render target texture com mip_levels/array_layers != 1".into(),
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
    /// original, todos os níveis de mip e camadas de array existentes)
    /// para amostrar `texture` num shader.
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

impl Context {
    /// Preenche os níveis de mip de `view` acima do nível base a partir
    /// dele, filtrando na GPU — o complemento de criar uma textura com
    /// `TextureDescriptor::mip_levels == 0` em
    /// [`Device::create_texture_2d`] (que só recebe os dados do nível
    /// base). Chame depois de ter escrito o nível base (na criação, ou
    /// via `update_buffer`-equivalente de textura, quando existir), e
    /// antes de desenhar usando `view` à distância da câmera.
    pub fn generate_mips(&self, view: &ShaderResourceView) {
        let _guard = self.lock_context();
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.GenerateMips(&view.raw);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = view;
        }
    }
}
