//! Visualizações de recursos: SRV, RTV, DSV, UAV.
//!
//! O D3D11 separa o recurso físico de suas visualizações. Uma mesma
//! textura pode ter múltiplas views, desde que flags, formato e regras de
//! compatibilidade permitam.
//!
//! ## Planejado
//!
//! | Tipo | Finalidade |
//! |---|---|
//! | `ShaderResourceView` (SRV) | Ler buffers/texturas em shaders |
//! | `RenderTargetView` (RTV) | Escrever saídas de renderização |
//! | `DepthStencilView` (DSV) | Depth e stencil |
//! | `UnorderedAccessView` (UAV) | Leitura/escrita não ordenada |
//!
//! Métodos de conveniência: `resource()`, `descriptor()`, `format()`,
//! `dimension()`, `mip_range()`, `array_layer_range()`, `set_label()`.
//!
//! Para UAVs com contador: `initial_count()`, `reset_counter()`,
//! `preserve_counter()`, `copy_structure_count()`.
//!
//! Ainda não implementado.
