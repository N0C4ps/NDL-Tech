//! Texturas 1D, 2D, 3D, cube e arrays.
//!
//! `ndl-render` só cria recursos de GPU a partir de dados já carregados —
//! quem lê arquivos de imagem do disco é `ndl-assets`; a conexão entre as
//! duas crates é feita pelo código do jogo.
//!
//! ## Planejado
//!
//! Tipos: `Texture1D`, `Texture2D`, `Texture3D`, `TextureCube`,
//! `TextureDescriptor`, `TextureViewDescriptor`, `TextureDimension`,
//! `TextureAspect`, `TextureSubresource`, `TextureRegion`.
//!
//! ```ignore
//! impl Device {
//!     fn create_texture_1d(&self, desc: &TextureDescriptor) -> Result<Texture>;
//!     fn create_texture_2d(&self, desc: &TextureDescriptor) -> Result<Texture>;
//!     fn create_texture_3d(&self, desc: &TextureDescriptor) -> Result<Texture>;
//!
//!     fn create_shader_resource_view(&self, texture: &Texture, desc: &ShaderResourceViewDescriptor) -> Result<ShaderResourceView>;
//!     fn create_render_target_view(&self, texture: &Texture, desc: &RenderTargetViewDescriptor) -> Result<RenderTargetView>;
//!     fn create_depth_stencil_view(&self, texture: &Texture, desc: &DepthStencilViewDescriptor) -> Result<DepthStencilView>;
//!     fn create_unordered_access_view(&self, texture: &Texture, desc: &UnorderedAccessViewDescriptor) -> Result<UnorderedAccessView>;
//! }
//! ```
//!
//! Operações essenciais planejadas:
//!
//! | Operação | Métodos |
//! |---|---|
//! | Upload | `write_texture`, `update_texture`, `update_texture_region` |
//! | Leitura | `read_texture`, `read_texture_region` |
//! | Cópia | `copy_texture`, `copy_texture_region`, `copy_subresource` |
//! | Limpeza | `clear_render_target`, `clear_depth`, `clear_stencil`, `clear_depth_stencil`, `clear_uav_float`, `clear_uav_uint` |
//! | Mipmaps | `generate_mipmaps`, `set_resource_min_lod`, `get_resource_min_lod` |
//! | MSAA | `resolve_texture`, `resolve_subresource` |
//! | Mapeamento | `map_texture`, `unmap_texture` |
//! | Subrecursos | `subresource_index`, `mip_extent`, `array_layer_count`, `mip_level_count` |
//!
//! Outros métodos úteis: `format()`, `width()`, `height()`, `depth()`,
//! `sample_count()`, `dimension()`, `is_cube()`, `is_multisampled()`,
//! `descriptor()`.
//!
//! `generate_mipmaps()`/`resolve_texture()` exigem texturas, views e
//! formatos compatíveis — não são válidas para toda textura.
//!
//! Ainda não implementado.
