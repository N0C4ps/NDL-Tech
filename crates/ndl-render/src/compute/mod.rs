//! Computação de uso geral na GPU.
//!
//! ## Estado atual
//!
//! [`crate::context::Context::set_compute_pipeline`],
//! `set_compute_constant_buffer`, `set_unordered_access_view` e
//! `dispatch` já são reais — o essencial para rodar um compute shader
//! lendo/escrevendo um [`crate::buffer::Buffer`] (`BufferUsage::Storage`,
//! via [`crate::view::UnorderedAccessView`]).
//!
//! `dispatch_indirect`, limpeza de UAV (`clear_uav_float`/`clear_uav_uint`),
//! contadores de UAV (`set_uav_initial_count`/`reset_uav_counter`) e
//! texturas como alvo de compute (hoje só buffers) continuam reservados.

use crate::context::Context;
use crate::pipeline::ComputePipeline;
use crate::view::UnorderedAccessView;

impl Context {
    /// Define o compute shader a ser usado pelos próximos `dispatch()`.
    pub fn set_compute_pipeline(&self, pipeline: &ComputePipeline) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.CSSetShader(&pipeline.compute_shader, None);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = pipeline;
        }
    }

    /// Vincula `buffer` ao slot `slot` de constant buffer do compute
    /// shader.
    pub fn set_compute_constant_buffer(&self, slot: u32, buffer: &crate::buffer::Buffer) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.CSSetConstantBuffers(slot, Some(&[Some(buffer.raw.clone())]));
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (slot, buffer);
        }
    }

    /// Vincula `view` ao slot `slot` de leitura/escrita não ordenada do
    /// compute shader.
    pub fn set_unordered_access_view(&self, slot: u32, view: &UnorderedAccessView) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw
                .CSSetUnorderedAccessViews(slot, 1, Some(&Some(view.raw.clone())), None);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (slot, view);
        }
    }

    /// Despacha o compute shader atual com `x * y * z` grupos de thread.
    pub fn dispatch(&self, x: u32, y: u32, z: u32) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.Dispatch(x, y, z);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (x, y, z);
        }
    }
}
