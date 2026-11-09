//! Computação de uso geral na GPU.
//!
//! ## Planejado
//!
//! ```ignore
//! impl ComputePass<'_> {
//!     fn set_pipeline(&mut self, pipeline: &ComputePipeline);
//!     fn set_constant_buffer(&mut self, slot: u32, buffer: &Buffer);
//!     fn set_shader_resource(&mut self, slot: u32, view: &ShaderResourceView);
//!     fn set_sampler(&mut self, slot: u32, sampler: &Sampler);
//!     fn set_uav(&mut self, slot: u32, view: &UnorderedAccessView);
//!
//!     fn dispatch(&mut self, x: u32, y: u32, z: u32);
//!     fn dispatch_indirect(&mut self, args: &Buffer, offset: u32);
//! }
//! ```
//!
//! Métodos adicionais: `clear_uav_float()`, `clear_uav_uint()`,
//! `copy_structure_count()`, `set_uav_initial_count()`,
//! `reset_uav_counter()`. Para sincronização de recursos, auxiliares como
//! `unbind_resource()` e `prepare_for_read()`/`prepare_for_write()` em vez
//! de uma barreira de memória estilo Vulkan — o D3D11 tem outro modelo de
//! gerenciamento de estados e dependências.
//!
//! Ainda não implementado.
