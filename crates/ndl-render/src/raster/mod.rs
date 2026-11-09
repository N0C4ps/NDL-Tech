//! Rasterização, viewport e scissor.
//!
//! ## Planejado
//!
//! Métodos: `create_rasterizer_state()`, `set_rasterizer_state()`,
//! `get_rasterizer_state()`, `set_viewport(s)()`, `get_viewports()`,
//! `set_scissor(s)()`, `get_scissors()`.
//!
//! Configurações:
//!
//! | Propriedade | Valores |
//! |---|---|
//! | FillMode | Solid, Wireframe |
//! | CullMode | None, Front, Back |
//! | FrontFace | Clockwise, CounterClockwise |
//! | DepthClip | Enabled, Disabled |
//! | Scissor | Enabled, Disabled |
//! | Multisample | Enabled, Disabled |
//! | AntialiasedLine | Enabled, Disabled |
//! | ConservativeRaster | Off, On (quando suportado) |
//!
//! Campos adicionais: `depth_bias`, `depth_bias_clamp`,
//! `slope_scaled_depth_bias`, `forced_sample_count`.
//!
//! Builders planejados: `RasterizerState::solid()`, `wireframe()`,
//! `cull_back()`, `cull_front()`, `no_culling()`, `with_depth_bias()`.
//!
//! Ainda não implementado.
