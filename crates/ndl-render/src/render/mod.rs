//! Desenho e render passes — a principal interface de uso cotidiano da
//! crate.
//!
//! ## Planejado
//!
//! ```ignore
//! impl RenderPass<'_> {
//!     fn set_pipeline(&mut self, pipeline: &GraphicsPipeline);
//!     fn set_vertex_buffer(&mut self, slot: u32, buffer: &Buffer);
//!     fn set_index_buffer(&mut self, buffer: &Buffer, format: IndexFormat);
//!
//!     fn set_viewport(&mut self, viewport: Viewport);
//!     fn set_scissor(&mut self, rect: Rect);
//!     fn set_blend_constant(&mut self, color: [f32; 4]);
//!     fn set_stencil_reference(&mut self, reference: u32);
//!
//!     fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>);
//!     fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>);
//!     fn draw_indirect(&mut self, args: &Buffer, offset: u64);
//!     fn draw_indexed_indirect(&mut self, args: &Buffer, offset: u64);
//! }
//! ```
//!
//! Inventário completo de draw calls: `draw()`, `draw_indexed()`,
//! `draw_instanced()`, `draw_indexed_instanced()`, `draw_indirect()`,
//! `draw_indexed_indirect()`, `draw_auto()` — no D3D11 nativo, as formas
//! indiretas correspondem a `DrawInstancedIndirect` e
//! `DrawIndexedInstancedIndirect`.
//!
//! Operações de render pass: `begin_render_pass()`, `end_render_pass()`,
//! `set_render_targets()`, `clear_color_attachment()`,
//! `clear_depth_attachment()`, `clear_stencil_attachment()`,
//! `set_pipeline()`, `set_bind_group()`, `set_vertex_buffer()`,
//! `set_index_buffer()`, `set_viewport()`, `set_scissor()`,
//! `set_blend_constant()`, `set_stencil_reference()`, `draw()`,
//! `draw_indexed()`, `draw_indirect()`, `draw_indexed_indirect()`.
//!
//! `set_bind_group()` é uma abstração inspirada em APIs modernas: vai
//! precisar traduzir os grupos para slots de SRV/sampler/constant
//! buffer/UAV do D3D11.
//!
//! Attachment load/store planejados:
//!
//! ```ignore
//! enum LoadOp<T> { Load, Clear(T), DontCare }
//! enum StoreOp { Store, Discard }
//! ```
//!
//! O D3D11 não implementa o mesmo modelo de render pass do Vulkan;
//! `DontCare`/`Discard` exigem implementação cuidadosa conforme a versão
//! do contexto disponível.
//!
//! Ainda não implementado.
