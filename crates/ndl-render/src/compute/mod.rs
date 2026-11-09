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
    pub fn set_compute_constant_buffer(
        &self,
        slot: u32,
        buffer: &crate::buffer::Buffer,
    ) -> Result<(), crate::core::GraphicsError> {
        let limit = self.limits().max_uniform_buffer_bindings;
        if slot >= limit {
            return Err(crate::core::GraphicsError::OutOfBounds {
                resource: "compute uniform buffer slots",
                offset: slot as u64,
                size: 1,
                limit: limit as u64,
            });
        }
        if buffer.usage() != crate::buffer::BufferUsage::Constant {
            return Err(crate::core::GraphicsError::IncompatibleResource {
                operation: "set_compute_constant_buffer",
                reason: "o buffer precisa ter BufferUsage::Constant".into(),
            });
        }
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw
                .CSSetConstantBuffers(slot, Some(&[Some(buffer.raw.clone())]));
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (slot, buffer);
        }
        Ok(())
    }

    /// Vincula `view` ao slot `slot` de leitura/escrita não ordenada do
    /// compute shader.
    pub fn set_unordered_access_view(
        &self,
        slot: u32,
        view: &UnorderedAccessView,
    ) -> Result<(), crate::core::GraphicsError> {
        let limit = self.limits().max_storage_buffer_bindings;
        if slot >= limit {
            return Err(crate::core::GraphicsError::OutOfBounds {
                resource: "storage buffer slots",
                offset: slot as u64,
                size: 1,
                limit: limit as u64,
            });
        }
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw
                .CSSetUnorderedAccessViews(slot, 1, Some(&Some(view.raw.clone())), None);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (slot, view);
        }
        Ok(())
    }

    /// Despacha o compute shader atual com `x * y * z` grupos de thread.
    pub fn dispatch(&self, x: u32, y: u32, z: u32) -> Result<(), crate::core::GraphicsError> {
        let limit = self.limits().max_compute_workgroups_per_dimension;
        if x > limit || y > limit || z > limit {
            return Err(crate::core::GraphicsError::OutOfBounds {
                resource: "compute workgroup count per dimension",
                offset: x.max(y).max(z) as u64,
                size: 0,
                limit: limit as u64,
            });
        }
        if x == 0 || y == 0 || z == 0 {
            return Ok(());
        }
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.Dispatch(x, y, z);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (x, y, z);
        }
        Ok(())
    }
}
