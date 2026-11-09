//! Configuração completa do pipeline gráfico e de computação.
//!
//! O D3D11 é uma API baseada em objetos de estado e bindings separados;
//! este módulo agrupa tudo isso em um único `GraphicsPipeline`/`ComputePipeline`,
//! que por baixo aplica os estados D3D11 correspondentes (não equivale a
//! um `ID3D12PipelineState`, que tem semântica diferente).
//!
//! ## Planejado
//!
//! ```ignore
//! impl Device {
//!     fn create_graphics_pipeline(&self, desc: &GraphicsPipelineDescriptor) -> Result<GraphicsPipeline>;
//!     fn create_compute_pipeline(&self, desc: &ComputePipelineDescriptor) -> Result<ComputePipeline>;
//! }
//!
//! pub struct GraphicsPipelineDescriptor {
//!     pub vertex_shader: VertexShader,
//!     pub pixel_shader: Option<PixelShader>,
//!     pub geometry_shader: Option<GeometryShader>,
//!     pub hull_shader: Option<HullShader>,
//!     pub domain_shader: Option<DomainShader>,
//!
//!     pub vertex_layout: VertexLayout,
//!     pub primitive_topology: PrimitiveTopology,
//!     pub rasterizer: RasterizerState,
//!     pub blend: BlendState,
//!     pub depth_stencil: DepthStencilState,
//! }
//! ```
//!
//! Outros métodos planejados: `create_pipeline_layout()`,
//! `validate_pipeline()`, `bind_pipeline()`, `pipeline_descriptor()`,
//! `pipeline_hash()`, `cache_pipeline()`, `get_cached_pipeline()`.
//!
//! Ainda não implementado.
