//! Gravação e execução de comandos.
//!
//! Decisão arquitetural: oferecer um `CommandEncoder` no estilo de APIs
//! modernas (que grava passes e comandos) por cima do modelo real do
//! backend (contextos imediato/diferido no D3D11), em vez de expor filas
//! de comando explícitas que o D3D11 não tem nativamente.
//!
//! ## Planejado
//!
//! ```ignore
//! impl CommandEncoder {
//!     fn begin_render_pass(&mut self, desc: &RenderPassDescriptor) -> RenderPass<'_>;
//!     fn begin_compute_pass(&mut self) -> ComputePass<'_>;
//!
//!     fn copy_buffer(&mut self, src: &Buffer, dst: &Buffer);
//!     fn copy_texture(&mut self, src: &Texture, dst: &Texture);
//!     fn clear_texture(&mut self, texture: &Texture, color: Color);
//!
//!     fn begin_debug_group(&mut self, label: &str);
//!     fn end_debug_group(&mut self);
//!     fn insert_debug_marker(&mut self, label: &str);
//!
//!     fn finish(self) -> Result<CommandBuffer>;
//! }
//! ```
//!
//! Outras operações consideradas: `reset()`, `execute()`,
//! `execute_command_list()`, `finish_command_list()`,
//! `restore_context_state()`, `discard_recording()`, `command_count()`.
//! Nem todas precisam ser públicas — `reset()`/`discard_recording()` só
//! fazem sentido se o encoder permitir reuso/cancelamento.
//!
//! Ainda não implementado.
