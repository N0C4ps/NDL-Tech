//! Backend Direct3D 11, via `windows-rs` (crate `windows`).
//!
//! Primeira implementação real de `ndl-render`: cria o dispositivo D3D11
//! e enumera adaptadores via DXGI. O resto da crate (buffer, texture,
//! pipeline, ...) ainda não tem contrapartida aqui — isso vai sendo
//! adicionado módulo a módulo.

use windows::Win32::Graphics::Direct3D::{
    D3D_DRIVER_TYPE, D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP, D3D_FEATURE_LEVEL,
    D3D_FEATURE_LEVEL_10_0, D3D_FEATURE_LEVEL_10_1, D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1,
    D3D_FEATURE_LEVEL_9_2, D3D_FEATURE_LEVEL_9_3,
};
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_CREATE_DEVICE_DEBUG, D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION,
};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE};

use crate::core::{AdapterInfo, DeviceDescriptor, FeatureLevel, GraphicsError};

/// Dispositivo D3D11 já criado: o device, o contexto imediato e o nível
/// de recurso que a GPU/driver realmente concedeu.
pub(crate) struct RawDevice {
    #[allow(dead_code)] // usado pelos módulos de recurso (buffer/texture/...) quando existirem
    pub(crate) device: ID3D11Device,
    #[allow(dead_code)] // usado por `context`/`command` quando existirem
    pub(crate) context: ID3D11DeviceContext,
    pub(crate) feature_level: D3D_FEATURE_LEVEL,
}

pub(crate) fn map_feature_level(level: D3D_FEATURE_LEVEL) -> FeatureLevel {
    match level {
        D3D_FEATURE_LEVEL_11_1 => FeatureLevel::Level11_1,
        D3D_FEATURE_LEVEL_11_0 => FeatureLevel::Level11_0,
        D3D_FEATURE_LEVEL_10_1 => FeatureLevel::Level10_1,
        D3D_FEATURE_LEVEL_10_0 => FeatureLevel::Level10_0,
        D3D_FEATURE_LEVEL_9_3 => FeatureLevel::Level9_3,
        D3D_FEATURE_LEVEL_9_2 => FeatureLevel::Level9_2,
        _ => FeatureLevel::Level9_1,
    }
}

pub(crate) fn create_default_device(
    descriptor: &DeviceDescriptor,
) -> Result<(RawDevice, bool), GraphicsError> {
    let driver_type: D3D_DRIVER_TYPE = if descriptor.prefer_software {
        D3D_DRIVER_TYPE_WARP
    } else {
        D3D_DRIVER_TYPE_HARDWARE
    };

    let mut flags = D3D11_CREATE_DEVICE_FLAG(D3D11_CREATE_DEVICE_BGRA_SUPPORT.0);
    if descriptor.enable_debug_layer {
        flags.0 |= D3D11_CREATE_DEVICE_DEBUG.0;
    }

    let mut device: Option<ID3D11Device> = None;
    let mut context: Option<ID3D11DeviceContext> = None;
    let mut feature_level = D3D_FEATURE_LEVEL::default();

    let result = unsafe {
        D3D11CreateDevice(
            None,
            driver_type,
            None,
            flags,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            Some(&mut feature_level),
            Some(&mut context),
        )
    };

    result.map_err(|err| GraphicsError::InitializationFailed(err.to_string()))?;

    let device = device.ok_or_else(|| {
        GraphicsError::InitializationFailed("D3D11CreateDevice não retornou um device".into())
    })?;
    let context = context.ok_or_else(|| {
        GraphicsError::InitializationFailed("D3D11CreateDevice não retornou um contexto".into())
    })?;

    Ok((
        RawDevice {
            device,
            context,
            feature_level,
        },
        descriptor.prefer_software,
    ))
}

pub(crate) fn is_device_lost(raw: &RawDevice) -> bool {
    unsafe { raw.device.GetDeviceRemovedReason() }.is_err()
}

pub(crate) fn enumerate_adapters() -> Result<Vec<AdapterInfo>, GraphicsError> {
    let factory: IDXGIFactory1 =
        unsafe { CreateDXGIFactory1() }.map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let mut adapters = Vec::new();
    let mut index = 0;
    loop {
        let adapter = match unsafe { factory.EnumAdapters1(index) } {
            Ok(adapter) => adapter,
            Err(_) => break,
        };
        index += 1;

        let desc = match unsafe { adapter.GetDesc1() } {
            Ok(desc) => desc,
            Err(_) => continue,
        };

        let name_len = desc.Description.iter().position(|&c| c == 0).unwrap_or(desc.Description.len());
        let name = String::from_utf16_lossy(&desc.Description[..name_len]);
        let is_software = (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0;

        adapters.push(AdapterInfo {
            name,
            vendor_id: desc.VendorId,
            device_id: desc.DeviceId,
            dedicated_video_memory: desc.DedicatedVideoMemory as u64,
            shared_system_memory: desc.SharedSystemMemory as u64,
            is_software,
        });
    }

    Ok(adapters)
}

// ---------------------------------------------------------------------
// Recursos avançados do D3D11 considerados para o futuro:
//
// | Módulo | Funcionalidades |
// |---|---|
// | interop | Recursos compartilhados, handles DXGI, keyed mutex |
// | video | Video device, decoder, processor, encoder |
// | tiled | Tiled resources, mapeamento e atualização de tiles |
// | stream_output | Geometry shader stream output, buffers e contadores |
// | indirect | Buffers de argumentos e execução indireta |
// | predication | Renderização condicional baseada em consultas |
// | multithreading | Proteção multithread, deferred contexts |
// | dxgi | Adaptadores, outputs, modos de exibição, formatos |
// | debug | InfoQueue, debug layer, annotations |
// | interop_d3d | Acesso controlado aos objetos COM nativos |
//
// Interop planejado (funções unsafe, com contratos de ownership e
// segurança rigorosos onde exponham/recebam ponteiros COM): raw_device(),
// raw_context(), raw_resource(), from_raw_device(), from_raw_resource(),
// open_shared_resource(), get_shared_handle(), acquire_keyed_mutex(),
// release_keyed_mutex().
//
// Tiled resources planejado: update_tile_mappings(), copy_tile_mappings(),
// copy_tiles(), update_tiles(), resize_tile_pool(), tiled_resource_barrier().
//
// Multithreading planejado: create_deferred_context(),
// finish_command_list(), execute_command_list(),
// set_multithread_protected(), get_multithread_protected() — o contexto
// imediato exige cuidado especial com acesso concorrente.
// ---------------------------------------------------------------------
