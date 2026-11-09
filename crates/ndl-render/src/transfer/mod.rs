//! Cópias, limpeza e resolução — operações que não envolvem desenho de
//! geometria.
//!
//! ## Planejado
//!
//! `copy_resource()`, `copy_buffer()`, `copy_buffer_region()`,
//! `copy_texture()`, `copy_texture_region()`, `copy_subresource_region()`,
//! `update_subresource()`, `update_buffer()`, `update_texture()`,
//! `resolve_subresource()`, `resolve_texture()`, `generate_mips()`,
//! `clear_render_target()`, `clear_depth()`, `clear_stencil()`,
//! `clear_depth_stencil()`, `clear_uav_float()`, `clear_uav_uint()`,
//! `copy_structure_count()`.
//!
//! Extensões D3D11.1+ (dependendo da interface disponível):
//! `copy_subresource_region1()`, `update_subresource1()`, `clear_view()`,
//! `discard_view()`, `discard_resource()`, `discard_view1()`. O suporte a
//! operações de descarte/mapeamento/atualização varia conforme o recurso,
//! sua configuração e a versão da interface disponível.
//!
//! Ainda não implementado.
