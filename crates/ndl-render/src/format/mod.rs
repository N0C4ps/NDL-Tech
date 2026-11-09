//! Formatos gráficos — representação dos formatos suportados pelo backend
//! (DXGI, no caso do Direct3D).
//!
//! `Format` é um tipo puramente de dados: não depende de nenhum backend
//! para existir, só a conversão para/do formato nativo do backend
//! (`to_dxgi_format`/`from_dxgi_format`) é específica do Windows.

/// Um formato de pixel/elemento, usado por texturas, buffers e views.
///
/// Cobre as famílias planejadas (normalizado, inteiro, float, sRGB,
/// depth/stencil, compressão em bloco) com um subconjunto representativo
/// de cada uma — mais variantes são adicionadas conforme a necessidade,
/// sem mudar a forma como o enum é consumido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Format {
    // Unsigned normalized
    R8Unorm,
    Rg8Unorm,
    Rgba8Unorm,
    Bgra8Unorm,

    // Signed normalized
    R8Snorm,
    Rgba8Snorm,

    // Unsigned integer
    R8Uint,
    R16Uint,
    R32Uint,
    Rgba8Uint,
    Rgba32Uint,

    // Signed integer
    R8Sint,
    R16Sint,
    R32Sint,
    Rgba8Sint,
    Rgba32Sint,

    // Floating point
    R16Float,
    R32Float,
    Rg16Float,
    Rg32Float,
    Rgba16Float,
    Rgba32Float,

    // sRGB
    Rgba8UnormSrgb,
    Bgra8UnormSrgb,

    // Depth/stencil
    D16Unorm,
    D24UnormS8Uint,
    D32Float,
    D32FloatS8X24Uint,

    // Block compression
    Bc1Unorm,
    Bc1UnormSrgb,
    Bc3Unorm,
    Bc3UnormSrgb,
    Bc4Unorm,
    Bc5Unorm,
    Bc6hUf16,
    Bc7Unorm,
    Bc7UnormSrgb,

    // Packed
    Rgb10a2Unorm,
    Rg11b10Float,
    Rgb9e5SharedExp,
}

impl Format {
    /// Quantos bytes cada bloco de compressão ocupa (só relevante para
    /// formatos BC); `1` para formatos não comprimidos (um "bloco" de
    /// 1x1 texel).
    pub fn bytes_per_block(self) -> u32 {
        use Format::*;
        match self {
            Bc1Unorm | Bc1UnormSrgb | Bc4Unorm => 8,
            Bc3Unorm | Bc3UnormSrgb | Bc5Unorm | Bc6hUf16 | Bc7Unorm | Bc7UnormSrgb => 16,
            _ => (self.bits_per_pixel() / 8).max(1),
        }
    }

    /// Dimensões (largura, altura) do bloco de compressão em texels; `(1, 1)`
    /// para formatos não comprimidos.
    pub fn block_dimensions(self) -> (u32, u32) {
        if self.is_compressed() {
            (4, 4)
        } else {
            (1, 1)
        }
    }

    pub fn bits_per_pixel(self) -> u32 {
        use Format::*;
        match self {
            R8Unorm | R8Snorm | R8Uint | R8Sint => 8,
            Rg8Unorm | R16Uint | R16Sint | R16Float | D16Unorm => 16,
            Rgba8Unorm | Bgra8Unorm | Rgba8Snorm | Rgba8Uint | Rgba8Sint | Rgba8UnormSrgb
            | Bgra8UnormSrgb | R32Uint | R32Sint | R32Float | Rg16Float | D24UnormS8Uint
            | D32Float | Rgb10a2Unorm | Rg11b10Float | Rgb9e5SharedExp => 32,
            Rg32Float | Rgba16Float | D32FloatS8X24Uint => 64,
            Rgba32Uint | Rgba32Sint | Rgba32Float => 128,
            // Formatos comprimidos: "bits por pixel" não é o conceito certo;
            // use `bytes_per_block()`/`block_dimensions()` para esses.
            Bc1Unorm | Bc1UnormSrgb | Bc4Unorm => 4,
            Bc3Unorm | Bc3UnormSrgb | Bc5Unorm | Bc6hUf16 | Bc7Unorm | Bc7UnormSrgb => 8,
        }
    }

    pub fn is_compressed(self) -> bool {
        use Format::*;
        matches!(
            self,
            Bc1Unorm
                | Bc1UnormSrgb
                | Bc3Unorm
                | Bc3UnormSrgb
                | Bc4Unorm
                | Bc5Unorm
                | Bc6hUf16
                | Bc7Unorm
                | Bc7UnormSrgb
        )
    }

    pub fn is_depth(self) -> bool {
        use Format::*;
        matches!(self, D16Unorm | D24UnormS8Uint | D32Float | D32FloatS8X24Uint)
    }

    pub fn is_stencil(self) -> bool {
        use Format::*;
        matches!(self, D24UnormS8Uint | D32FloatS8X24Uint)
    }

    pub fn is_srgb(self) -> bool {
        matches!(self, Format::Rgba8UnormSrgb | Format::Bgra8UnormSrgb | Format::Bc1UnormSrgb | Format::Bc3UnormSrgb | Format::Bc7UnormSrgb)
    }

    pub fn is_integer(self) -> bool {
        use Format::*;
        matches!(
            self,
            R8Uint
                | R16Uint
                | R32Uint
                | Rgba8Uint
                | Rgba32Uint
                | R8Sint
                | R16Sint
                | R32Sint
                | Rgba8Sint
                | Rgba32Sint
        )
    }

    pub fn is_float(self) -> bool {
        use Format::*;
        matches!(
            self,
            R16Float | R32Float | Rg16Float | Rg32Float | Rgba16Float | Rgba32Float | D32Float
        )
    }
}

#[cfg(target_os = "windows")]
mod dxgi {
    use super::Format;
    use windows::Win32::Graphics::Dxgi::Common::*;

    impl Format {
        /// Converte para o `DXGI_FORMAT` nativo correspondente, quando existe
        /// um mapeamento direto.
        pub fn to_dxgi_format(self) -> Option<DXGI_FORMAT> {
            use Format::*;
            Some(match self {
                R8Unorm => DXGI_FORMAT_R8_UNORM,
                Rg8Unorm => DXGI_FORMAT_R8G8_UNORM,
                Rgba8Unorm => DXGI_FORMAT_R8G8B8A8_UNORM,
                Bgra8Unorm => DXGI_FORMAT_B8G8R8A8_UNORM,
                R8Snorm => DXGI_FORMAT_R8_SNORM,
                Rgba8Snorm => DXGI_FORMAT_R8G8B8A8_SNORM,
                R8Uint => DXGI_FORMAT_R8_UINT,
                R16Uint => DXGI_FORMAT_R16_UINT,
                R32Uint => DXGI_FORMAT_R32_UINT,
                Rgba8Uint => DXGI_FORMAT_R8G8B8A8_UINT,
                Rgba32Uint => DXGI_FORMAT_R32G32B32A32_UINT,
                R8Sint => DXGI_FORMAT_R8_SINT,
                R16Sint => DXGI_FORMAT_R16_SINT,
                R32Sint => DXGI_FORMAT_R32_SINT,
                Rgba8Sint => DXGI_FORMAT_R8G8B8A8_SINT,
                Rgba32Sint => DXGI_FORMAT_R32G32B32A32_SINT,
                R16Float => DXGI_FORMAT_R16_FLOAT,
                R32Float => DXGI_FORMAT_R32_FLOAT,
                Rg16Float => DXGI_FORMAT_R16G16_FLOAT,
                Rg32Float => DXGI_FORMAT_R32G32_FLOAT,
                Rgba16Float => DXGI_FORMAT_R16G16B16A16_FLOAT,
                Rgba32Float => DXGI_FORMAT_R32G32B32A32_FLOAT,
                Rgba8UnormSrgb => DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
                Bgra8UnormSrgb => DXGI_FORMAT_B8G8R8A8_UNORM_SRGB,
                D16Unorm => DXGI_FORMAT_D16_UNORM,
                D24UnormS8Uint => DXGI_FORMAT_D24_UNORM_S8_UINT,
                D32Float => DXGI_FORMAT_D32_FLOAT,
                D32FloatS8X24Uint => DXGI_FORMAT_D32_FLOAT_S8X24_UINT,
                Bc1Unorm => DXGI_FORMAT_BC1_UNORM,
                Bc1UnormSrgb => DXGI_FORMAT_BC1_UNORM_SRGB,
                Bc3Unorm => DXGI_FORMAT_BC3_UNORM,
                Bc3UnormSrgb => DXGI_FORMAT_BC3_UNORM_SRGB,
                Bc4Unorm => DXGI_FORMAT_BC4_UNORM,
                Bc5Unorm => DXGI_FORMAT_BC5_UNORM,
                Bc6hUf16 => DXGI_FORMAT_BC6H_UF16,
                Bc7Unorm => DXGI_FORMAT_BC7_UNORM,
                Bc7UnormSrgb => DXGI_FORMAT_BC7_UNORM_SRGB,
                Rgb10a2Unorm => DXGI_FORMAT_R10G10B10A2_UNORM,
                Rg11b10Float => DXGI_FORMAT_R11G11B10_FLOAT,
                Rgb9e5SharedExp => DXGI_FORMAT_R9G9B9E5_SHAREDEXP,
            })
        }

        /// Converte a partir de um `DXGI_FORMAT` nativo, quando existe um
        /// mapeamento direto para este enum.
        pub fn from_dxgi_format(format: DXGI_FORMAT) -> Option<Format> {
            use Format::*;
            Some(match format {
                DXGI_FORMAT_R8_UNORM => R8Unorm,
                DXGI_FORMAT_R8G8_UNORM => Rg8Unorm,
                DXGI_FORMAT_R8G8B8A8_UNORM => Rgba8Unorm,
                DXGI_FORMAT_B8G8R8A8_UNORM => Bgra8Unorm,
                DXGI_FORMAT_R8_SNORM => R8Snorm,
                DXGI_FORMAT_R8G8B8A8_SNORM => Rgba8Snorm,
                DXGI_FORMAT_R8_UINT => R8Uint,
                DXGI_FORMAT_R16_UINT => R16Uint,
                DXGI_FORMAT_R32_UINT => R32Uint,
                DXGI_FORMAT_R8G8B8A8_UINT => Rgba8Uint,
                DXGI_FORMAT_R32G32B32A32_UINT => Rgba32Uint,
                DXGI_FORMAT_R8_SINT => R8Sint,
                DXGI_FORMAT_R16_SINT => R16Sint,
                DXGI_FORMAT_R32_SINT => R32Sint,
                DXGI_FORMAT_R8G8B8A8_SINT => Rgba8Sint,
                DXGI_FORMAT_R32G32B32A32_SINT => Rgba32Sint,
                DXGI_FORMAT_R16_FLOAT => R16Float,
                DXGI_FORMAT_R32_FLOAT => R32Float,
                DXGI_FORMAT_R16G16_FLOAT => Rg16Float,
                DXGI_FORMAT_R32G32_FLOAT => Rg32Float,
                DXGI_FORMAT_R16G16B16A16_FLOAT => Rgba16Float,
                DXGI_FORMAT_R32G32B32A32_FLOAT => Rgba32Float,
                DXGI_FORMAT_R8G8B8A8_UNORM_SRGB => Rgba8UnormSrgb,
                DXGI_FORMAT_B8G8R8A8_UNORM_SRGB => Bgra8UnormSrgb,
                DXGI_FORMAT_D16_UNORM => D16Unorm,
                DXGI_FORMAT_D24_UNORM_S8_UINT => D24UnormS8Uint,
                DXGI_FORMAT_D32_FLOAT => D32Float,
                DXGI_FORMAT_D32_FLOAT_S8X24_UINT => D32FloatS8X24Uint,
                DXGI_FORMAT_BC1_UNORM => Bc1Unorm,
                DXGI_FORMAT_BC1_UNORM_SRGB => Bc1UnormSrgb,
                DXGI_FORMAT_BC3_UNORM => Bc3Unorm,
                DXGI_FORMAT_BC3_UNORM_SRGB => Bc3UnormSrgb,
                DXGI_FORMAT_BC4_UNORM => Bc4Unorm,
                DXGI_FORMAT_BC5_UNORM => Bc5Unorm,
                DXGI_FORMAT_BC6H_UF16 => Bc6hUf16,
                DXGI_FORMAT_BC7_UNORM => Bc7Unorm,
                DXGI_FORMAT_BC7_UNORM_SRGB => Bc7UnormSrgb,
                DXGI_FORMAT_R10G10B10A2_UNORM => Rgb10a2Unorm,
                DXGI_FORMAT_R11G11B10_FLOAT => Rg11b10Float,
                DXGI_FORMAT_R9G9B9E5_SHAREDEXP => Rgb9e5SharedExp,
                _ => return None,
            })
        }
    }
}

// TODO: is_filterable()/is_renderable()/supports_msaa()/supports_uav()
// precisam consultar o Device (a capacidade real depende do hardware/
// driver, não só do enum) — ficam para quando `core::Device` existir.

#[cfg(test)]
mod tests {
    use super::Format;

    #[test]
    fn bits_per_pixel_matches_known_formats() {
        assert_eq!(Format::R8Unorm.bits_per_pixel(), 8);
        assert_eq!(Format::Rgba8Unorm.bits_per_pixel(), 32);
        assert_eq!(Format::Rgba32Float.bits_per_pixel(), 128);
    }

    #[test]
    fn classification_flags() {
        assert!(Format::D32Float.is_depth());
        assert!(!Format::D32Float.is_stencil());
        assert!(Format::D24UnormS8Uint.is_stencil());

        assert!(Format::Bc7Unorm.is_compressed());
        assert!(!Format::Rgba8Unorm.is_compressed());

        assert!(Format::Rgba8UnormSrgb.is_srgb());
        assert!(!Format::Rgba8Unorm.is_srgb());

        assert!(Format::R32Uint.is_integer());
        assert!(Format::R32Float.is_float());
        assert!(!Format::R32Uint.is_float());
    }

    #[test]
    fn block_dimensions_for_compressed_vs_plain() {
        assert_eq!(Format::Bc1Unorm.block_dimensions(), (4, 4));
        assert_eq!(Format::Rgba8Unorm.block_dimensions(), (1, 1));
        assert_eq!(Format::Bc1Unorm.bytes_per_block(), 8);
        assert_eq!(Format::Bc7Unorm.bytes_per_block(), 16);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn dxgi_roundtrip() {
        for format in [
            Format::Rgba8Unorm,
            Format::Bgra8Unorm,
            Format::D32Float,
            Format::Bc7Unorm,
            Format::Rgba32Float,
        ] {
            let dxgi = format.to_dxgi_format().expect("mapeamento dxgi esperado");
            assert_eq!(Format::from_dxgi_format(dxgi), Some(format));
        }
    }
}
