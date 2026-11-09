//! Formatos gráficos — representação dos formatos suportados pelo backend
//! (DXGI, no caso do Direct3D).
//!
//! ## Planejado
//!
//! Famílias de formato planejadas:
//!
//! | Família | Exemplos |
//! |---|---|
//! | Unsigned normalized | R8Unorm, Rgba8Unorm |
//! | Signed normalized | R8Snorm, Rgba8Snorm |
//! | Unsigned integer | R8Uint, R32Uint |
//! | Signed integer | R8Sint, R32Sint |
//! | Floating point | R16Float, R32Float, Rgba16Float, Rgba32Float |
//! | sRGB | Rgba8UnormSrgb, Bgra8UnormSrgb |
//! | Depth/stencil | D16Unorm, D24UnormS8Uint, D32Float, D32FloatS8X24Uint |
//! | Block compression | BC1–BC7 |
//! | Packed | Rgb10a2Unorm, Rg11b10Float, Rgb9e5SharedExp |
//! | Typeless | famílias DXGI typeless para views compatíveis |
//! | Vídeo/YUV | NV12, P010, P016 e outros |
//!
//! Métodos utilitários planejados: `bytes_per_block()`,
//! `block_dimensions()`, `bits_per_pixel()`, `is_compressed()`,
//! `is_depth()`, `is_stencil()`, `is_srgb()`, `is_integer()`,
//! `is_float()`, `is_typeless()`, `is_filterable()`, `is_renderable()`,
//! `supports_msaa()`, `supports_uav()`, `to_dxgi_format()`,
//! `from_dxgi_format()`.
//!
//! A capacidade real de filtrar/renderizar/usar UAV precisa ser
//! consultada no dispositivo — não basta deduzi-la só do enum.
//!
//! Ainda não implementado.
