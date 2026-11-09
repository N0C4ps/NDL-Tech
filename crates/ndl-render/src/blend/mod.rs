//! Mistura de cores (blending) e operações lógicas.
//!
//! A equação geral de blending de cor, nas operações aditivas, é
//! `C_out = C_src * F_src + C_dst * F_dst`; outras operações substituem a
//! soma por subtração, mínimo ou máximo.
//!
//! ## Planejado
//!
//! `BlendOp`: Add, Subtract, ReverseSubtract, Min, Max.
//!
//! ```ignore
//! pub enum BlendFactor {
//!     Zero, One,
//!     SrcColor, OneMinusSrcColor, DstColor, OneMinusDstColor,
//!     SrcAlpha, OneMinusSrcAlpha, DstAlpha, OneMinusDstAlpha,
//!     SrcAlphaSaturated,
//!     ConstantColor, OneMinusConstantColor,
//!     Src1Color, OneMinusSrc1Color, Src1Alpha, OneMinusSrc1Alpha,
//! }
//! ```
//!
//! `ConstantColor`/`OneMinusConstantColor` representam, nesta API, o fator
//! de blend configurado pelo aplicativo — o D3D11 usa `BLEND_FACTOR` e
//! `INV_BLEND_FACTOR`.
//!
//! Métodos: `create_blend_state()`, `set_blend_state()`,
//! `get_blend_state()`, `set_blend_constant()`, `get_blend_constant()`,
//! `set_sample_mask()`, `set_alpha_to_coverage()`,
//! `set_independent_blending()`, `set_color_write_mask()`.
//!
//! Builders/presets planejados: `BlendState::opaque()`,
//! `alpha_blend()`, `premultiplied_alpha()`, `additive()`,
//! `subtractive()`, `multiply()`, `screen()`, `min()`, `max()`,
//! `disabled()` — `multiply()`/`screen()` são presets expressos com
//! fatores adequados, não novos `BlendOp` nativos.
//!
//! `ColorWriteMask`: RED, GREEN, BLUE, ALPHA, RGB, ALL, NONE.
//!
//! `LogicOp` (hardware/feature level compatíveis, distinto do blending
//! aritmético): Clear, Set, Copy, CopyInverted, Noop, Invert, And, Nand,
//! Or, Nor, Xor, Equiv, AndReverse, AndInverted, OrReverse, OrInverted.
//!
//! Ainda não implementado.
