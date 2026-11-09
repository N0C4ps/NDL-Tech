//! Computação de uso geral na GPU.
//!
//! ## Estado atual
//!
//! [`crate::context::Context::set_compute_pipeline`],
//! `set_compute_constant_buffer`, `set_unordered_access_view`,
//! `dispatch` e `dispatch_indirect` já são reais — o essencial para rodar
//! um compute shader lendo/escrevendo um [`crate::buffer::Buffer`]
//! (`BufferUsage::Storage`, via [`crate::view::UnorderedAccessView`]).
//! `dispatch_indirect` lê a contagem de grupos de
//! [`DispatchIndirectArgs`] num buffer `BufferUsage::Indirect` — hoje só
//! escrito pela CPU, já que um buffer indirect gravável por compute (UAV)
//! continua reservado (ver [`crate::buffer`]).
//!
//! Limpeza de UAV (`clear_uav_float`/`clear_uav_uint`), contadores de UAV
//! (`set_uav_initial_count`/`reset_uav_counter`) e texturas como alvo de
//! compute (hoje só buffers) continuam reservados.

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

    /// Despacha o compute shader atual lendo `x`/`y`/`z` de
    /// `args_buffer` (um [`crate::buffer::Buffer`] com
    /// `BufferUsage::Indirect`) a partir de `args_offset`, no lugar de
    /// contagens fixas — ver [`DispatchIndirectArgs`] para o layout
    /// esperado.
    pub fn dispatch_indirect(
        &self,
        args_buffer: &crate::buffer::Buffer,
        args_offset: u64,
    ) -> Result<(), crate::core::GraphicsError> {
        validate_indirect_args(
            args_buffer,
            args_offset,
            DispatchIndirectArgs::SIZE,
            "dispatch_indirect",
        )?;
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw
                .DispatchIndirect(&args_buffer.raw, args_offset as u32);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (args_buffer, args_offset);
        }
        Ok(())
    }
}

pub(crate) fn validate_indirect_args(
    buffer: &crate::buffer::Buffer,
    offset: u64,
    args_size: u64,
    operation: &'static str,
) -> Result<(), crate::core::GraphicsError> {
    if buffer.usage() != crate::buffer::BufferUsage::Indirect {
        return Err(crate::core::GraphicsError::IncompatibleResource {
            operation,
            reason: "o buffer precisa ter BufferUsage::Indirect".into(),
        });
    }
    if offset % 4 != 0 {
        return Err(crate::core::GraphicsError::InvalidArgument {
            name: "args_offset",
            reason: "precisa ser múltiplo de 4 bytes".into(),
        });
    }
    let end = offset
        .checked_add(args_size)
        .ok_or_else(|| crate::core::GraphicsError::InvalidArgument {
            name: "args_offset",
            reason: "overflow ao calcular o fim dos argumentos indiretos".into(),
        })?;
    if end > buffer.size() {
        return Err(crate::core::GraphicsError::OutOfBounds {
            resource: "indirect args buffer",
            offset,
            size: args_size,
            limit: buffer.size(),
        });
    }
    Ok(())
}

/// Argumentos de um [`Context::dispatch_indirect`] — mesmo layout binário
/// do `D3D11_DISPATCH_INDIRECT_ARGS`. Escreva
/// [`DispatchIndirectArgs::to_bytes`] num buffer `BufferUsage::Indirect`
/// no offset passado a `dispatch_indirect`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct DispatchIndirectArgs {
    pub thread_group_count_x: u32,
    pub thread_group_count_y: u32,
    pub thread_group_count_z: u32,
}

impl DispatchIndirectArgs {
    /// Tamanho em bytes de um `DispatchIndirectArgs` no buffer indirect.
    pub const SIZE: u64 = 12;

    /// Serializa os argumentos na ordem/largura exata que o D3D11 espera
    /// encontrar no buffer indirect (little-endian, 3 `u32` sequenciais).
    pub fn to_bytes(&self) -> [u8; Self::SIZE as usize] {
        let mut out = [0u8; Self::SIZE as usize];
        out[0..4].copy_from_slice(&self.thread_group_count_x.to_le_bytes());
        out[4..8].copy_from_slice(&self.thread_group_count_y.to_le_bytes());
        out[8..12].copy_from_slice(&self.thread_group_count_z.to_le_bytes());
        out
    }
}
