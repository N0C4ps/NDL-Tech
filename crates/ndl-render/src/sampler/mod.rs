//! Amostragem de texturas.
//!
//! ## Planejado
//!
//! Métodos: `create_sampler()`, `set_sampler()`, `set_samplers()`,
//! `get_sampler()`.
//!
//! | Categoria | Opções |
//! |---|---|
//! | FilterMode | Point, Linear |
//! | MipFilterMode | Point, Linear, None |
//! | AddressMode | Wrap, Mirror, Clamp, Border, MirrorOnce |
//! | Comparison | os oito valores de `CompareOp` |
//! | ReductionMode | Standard, Comparison, Minimum, Maximum (conforme suporte) |
//! | Anisotropia | limitada pelo dispositivo |
//! | LOD | `min_lod`, `max_lod`, `mip_lod_bias` |
//! | Border | `border_color` |
//!
//! Presets planejados: `Sampler::point_clamp()`, `point_repeat()`,
//! `linear_clamp()`, `linear_repeat()`, `trilinear()`, `anisotropic()`,
//! `shadow_comparison()`. Também útil: `create_sampler_from_descriptor()`
//! para configurações personalizadas.
//!
//! Ainda não implementado.
