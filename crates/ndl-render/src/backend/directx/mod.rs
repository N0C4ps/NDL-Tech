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
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Texture2D,
    D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_DEBUG, D3D11_CREATE_DEVICE_FLAG,
    D3D11_SDK_VERSION,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_ALPHA_MODE_IGNORE, DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIDevice, IDXGIFactory1, IDXGIFactory2, IDXGISwapChain1,
    DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_PRESENT, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1,
    DXGI_SWAP_CHAIN_FLAG, DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
};
use windows::core::Interface;

use crate::core::{AdapterInfo, Device, DeviceDescriptor, FeatureLevel, GraphicsError};
use crate::surface::{PresentMode, PresentResult, Surface, SwapchainDescriptor};

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

pub(crate) fn surface_from_handle(
    handle: raw_window_handle::RawWindowHandle,
) -> Result<Surface, GraphicsError> {
    match handle {
        raw_window_handle::RawWindowHandle::Win32(win32) => {
            let hwnd = windows::Win32::Foundation::HWND(win32.hwnd.get() as *mut _);
            Ok(Surface { hwnd })
        }
        _ => Err(GraphicsError::Unsupported(
            "o backend Direct3D só aceita handles de janela Win32",
        )),
    }
}

/// Swapchain D3D11 já criada.
pub(crate) struct RawSwapchain {
    swapchain: IDXGISwapChain1,
}

pub(crate) fn create_swapchain(
    device: &Device,
    surface: &Surface,
    desc: &SwapchainDescriptor,
) -> Result<RawSwapchain, GraphicsError> {
    let dxgi_device: IDXGIDevice = device
        .raw
        .device
        .cast()
        .map_err(|err| GraphicsError::Backend(err.to_string()))?;
    let adapter = unsafe { dxgi_device.GetAdapter() }
        .map_err(|err| GraphicsError::Backend(err.to_string()))?;
    let factory: IDXGIFactory2 = unsafe { adapter.GetParent() }
        .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let format = desc
        .format
        .to_dxgi_format()
        .ok_or(GraphicsError::Unsupported("formato não suportado para swapchain"))?;

    let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
        Width: desc.size.width,
        Height: desc.size.height,
        Format: format,
        Stereo: false.into(),
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: desc.buffer_count,
        Scaling: DXGI_SCALING_STRETCH,
        SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
        AlphaMode: DXGI_ALPHA_MODE_IGNORE,
        Flags: 0,
    };

    let swapchain = unsafe {
        factory.CreateSwapChainForHwnd(
            &device.raw.device,
            surface.hwnd,
            &swap_chain_desc,
            None,
            None,
        )
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    Ok(RawSwapchain { swapchain })
}

pub(crate) fn resize_swapchain(
    raw: &mut RawSwapchain,
    width: u32,
    height: u32,
) -> Result<(), GraphicsError> {
    unsafe { raw.swapchain.ResizeBuffers(0, width, height, DXGI_FORMAT_UNKNOWN, DXGI_SWAP_CHAIN_FLAG(0)) }
        .map_err(|err| GraphicsError::Backend(err.to_string()))
}

/// Limpa o back buffer atual e apresenta — atalho temporário até
/// `context`/`render` existirem de verdade (ver doc de
/// `Swapchain::clear_and_present`).
pub(crate) fn clear_and_present(
    device: &Device,
    raw: &RawSwapchain,
    color: [f32; 4],
    present_mode: PresentMode,
) -> Result<PresentResult, GraphicsError> {
    let back_buffer: ID3D11Texture2D = unsafe { raw.swapchain.GetBuffer(0) }
        .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let mut render_target_view: Option<ID3D11RenderTargetView> = None;
    unsafe {
        device
            .raw
            .device
            .CreateRenderTargetView(&back_buffer, None, Some(&mut render_target_view))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;
    let render_target_view = render_target_view.ok_or_else(|| {
        GraphicsError::Backend("CreateRenderTargetView não retornou uma view".into())
    })?;

    unsafe {
        device.raw.context.ClearRenderTargetView(&render_target_view, &color);
    }

    // VSync por padrão: "liso" numa GPU topo de linha não significa nada
    // numa GPU mais modesta — nunca presumimos Immediate sem o jogo pedir.
    let sync_interval = match present_mode {
        PresentMode::VSync => 1,
        PresentMode::Immediate => 0,
    };

    let present_result = unsafe { raw.swapchain.Present(sync_interval, DXGI_PRESENT(0)) };
    if present_result.is_err() {
        return Err(GraphicsError::Backend(present_result.message()));
    }

    Ok(PresentResult::Ok)
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
