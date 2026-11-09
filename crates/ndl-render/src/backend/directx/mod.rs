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
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;
use windows::Win32::Graphics::Direct3D::ID3DBlob;
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout,
    ID3D11PixelShader, ID3D11RenderTargetView, ID3D11SamplerState, ID3D11ShaderResourceView,
    ID3D11Texture2D, ID3D11VertexShader, D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_FLAG,
    D3D11_BIND_INDEX_BUFFER, D3D11_BIND_SHADER_RESOURCE, D3D11_BIND_VERTEX_BUFFER,
    D3D11_BUFFER_DESC, D3D11_CPU_ACCESS_WRITE, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_CREATE_DEVICE_DEBUG, D3D11_CREATE_DEVICE_FLAG, D3D11_FILTER_MIN_MAG_MIP_LINEAR,
    D3D11_FILTER_MIN_MAG_MIP_POINT, D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA,
    D3D11_SAMPLER_DESC, D3D11_SDK_VERSION, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC,
    D3D11_TEXTURE_ADDRESS_CLAMP, D3D11_TEXTURE_ADDRESS_MIRROR, D3D11_TEXTURE_ADDRESS_WRAP,
    D3D11_USAGE_DEFAULT, D3D11_USAGE_DYNAMIC,
};
use windows::core::PCSTR;
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

fn bind_flag_for_usage(usage: crate::buffer::BufferUsage) -> D3D11_BIND_FLAG {
    use crate::buffer::BufferUsage::*;
    match usage {
        Vertex => D3D11_BIND_VERTEX_BUFFER,
        Index => D3D11_BIND_INDEX_BUFFER,
        Constant => D3D11_BIND_CONSTANT_BUFFER,
    }
}

pub(crate) fn create_buffer(
    device: &Device,
    desc: &crate::buffer::BufferDescriptor,
    data: Option<&[u8]>,
) -> Result<crate::buffer::Buffer, GraphicsError> {
    if let Some(data) = data {
        if data.len() as u64 != desc.size {
            return Err(GraphicsError::Backend(format!(
                "dados fornecidos ({} bytes) não têm o mesmo tamanho do descriptor ({} bytes)",
                data.len(),
                desc.size
            )));
        }
    }

    let buffer_desc = D3D11_BUFFER_DESC {
        ByteWidth: desc.size as u32,
        Usage: if desc.dynamic {
            D3D11_USAGE_DYNAMIC
        } else {
            D3D11_USAGE_DEFAULT
        },
        BindFlags: bind_flag_for_usage(desc.usage).0 as u32,
        CPUAccessFlags: if desc.dynamic {
            D3D11_CPU_ACCESS_WRITE.0 as u32
        } else {
            0
        },
        MiscFlags: 0,
        StructureByteStride: 0,
    };

    let initial_data = data.map(|bytes| D3D11_SUBRESOURCE_DATA {
        pSysMem: bytes.as_ptr() as *const _,
        SysMemPitch: 0,
        SysMemSlicePitch: 0,
    });

    let mut buffer: Option<ID3D11Buffer> = None;
    unsafe {
        device.raw.device.CreateBuffer(
            &buffer_desc,
            initial_data.as_ref().map(|d| d as *const _),
            Some(&mut buffer),
        )
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let buffer = buffer
        .ok_or_else(|| GraphicsError::Backend("CreateBuffer não retornou um buffer".into()))?;

    Ok(crate::buffer::Buffer::from_raw(buffer, desc.size, desc.usage))
}

pub(crate) fn compile_hlsl(
    source: &str,
    entry_point: &str,
    target: &str,
) -> Result<Vec<u8>, GraphicsError> {
    let entry_point_c = std::ffi::CString::new(entry_point)
        .map_err(|err| GraphicsError::Backend(err.to_string()))?;
    let target_c =
        std::ffi::CString::new(target).map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let mut code: Option<ID3DBlob> = None;
    let mut errors: Option<ID3DBlob> = None;

    let result = unsafe {
        D3DCompile(
            source.as_ptr() as *const _,
            source.len(),
            None,
            None,
            None,
            PCSTR(entry_point_c.as_ptr() as *const u8),
            PCSTR(target_c.as_ptr() as *const u8),
            0,
            0,
            &mut code,
            Some(&mut errors),
        )
    };

    if let Err(err) = result {
        let message = errors
            .map(|blob| blob_to_string(&blob))
            .unwrap_or_else(|| err.message());
        return Err(GraphicsError::Backend(message));
    }

    let code = code
        .ok_or_else(|| GraphicsError::Backend("D3DCompile não retornou bytecode".into()))?;
    Ok(blob_to_bytes(&code))
}

fn blob_to_bytes(blob: &ID3DBlob) -> Vec<u8> {
    unsafe {
        let ptr = blob.GetBufferPointer() as *const u8;
        let len = blob.GetBufferSize();
        std::slice::from_raw_parts(ptr, len).to_vec()
    }
}

fn blob_to_string(blob: &ID3DBlob) -> String {
    String::from_utf8_lossy(&blob_to_bytes(blob)).into_owned()
}

pub(crate) fn create_vertex_shader(
    device: &Device,
    bytecode: &[u8],
) -> Result<crate::shader::VertexShader, GraphicsError> {
    let mut shader: Option<ID3D11VertexShader> = None;
    unsafe {
        device
            .raw
            .device
            .CreateVertexShader(bytecode, None, Some(&mut shader))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;
    let shader = shader.ok_or_else(|| {
        GraphicsError::Backend("CreateVertexShader não retornou um shader".into())
    })?;
    Ok(crate::shader::VertexShader {
        raw: shader,
        bytecode: bytecode.to_vec(),
    })
}

pub(crate) fn create_pixel_shader(
    device: &Device,
    bytecode: &[u8],
) -> Result<crate::shader::PixelShader, GraphicsError> {
    let mut shader: Option<ID3D11PixelShader> = None;
    unsafe {
        device
            .raw
            .device
            .CreatePixelShader(bytecode, None, Some(&mut shader))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;
    let shader = shader.ok_or_else(|| {
        GraphicsError::Backend("CreatePixelShader não retornou um shader".into())
    })?;
    Ok(crate::shader::PixelShader { raw: shader })
}

pub(crate) fn create_input_layout(
    device: &Device,
    attributes: &[crate::input::VertexAttribute],
    vertex_shader: &crate::shader::VertexShader,
) -> Result<crate::input::InputLayout, GraphicsError> {
    // Os nomes de semântica precisam ficar vivos (como C strings) durante
    // toda a chamada a CreateInputLayout.
    let semantic_names: Vec<std::ffi::CString> = attributes
        .iter()
        .map(|attr| std::ffi::CString::new(attr.semantic).expect("semântica sem bytes nulos"))
        .collect();

    let descs: Vec<D3D11_INPUT_ELEMENT_DESC> = attributes
        .iter()
        .zip(&semantic_names)
        .map(|(attr, name)| {
            let format = attr.format.to_dxgi_format().unwrap_or(DXGI_FORMAT_UNKNOWN);
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: PCSTR(name.as_ptr() as *const u8),
                SemanticIndex: attr.semantic_index,
                Format: format,
                InputSlot: 0,
                AlignedByteOffset: attr.offset,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            }
        })
        .collect();

    let mut layout: Option<ID3D11InputLayout> = None;
    unsafe {
        device.raw.device.CreateInputLayout(
            &descs,
            &vertex_shader.bytecode,
            Some(&mut layout),
        )
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let layout = layout.ok_or_else(|| {
        GraphicsError::Backend("CreateInputLayout não retornou um input layout".into())
    })?;

    Ok(crate::input::InputLayout { raw: layout })
}

pub(crate) fn create_texture_2d(
    device: &Device,
    desc: &crate::texture::TextureDescriptor,
    data: &[u8],
) -> Result<crate::texture::Texture2D, GraphicsError> {
    let format = desc
        .format
        .to_dxgi_format()
        .ok_or(GraphicsError::Unsupported("formato não suportado para textura"))?;

    let texture_desc = D3D11_TEXTURE2D_DESC {
        Width: desc.width,
        Height: desc.height,
        MipLevels: 1,
        ArraySize: 1,
        Format: format,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
        CPUAccessFlags: 0,
        MiscFlags: 0,
    };

    let bytes_per_pixel = desc.format.bits_per_pixel() / 8;
    let initial_data = D3D11_SUBRESOURCE_DATA {
        pSysMem: data.as_ptr() as *const _,
        SysMemPitch: desc.width * bytes_per_pixel,
        SysMemSlicePitch: 0,
    };

    let mut texture: Option<ID3D11Texture2D> = None;
    unsafe {
        device
            .raw
            .device
            .CreateTexture2D(&texture_desc, Some(&initial_data), Some(&mut texture))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let texture = texture
        .ok_or_else(|| GraphicsError::Backend("CreateTexture2D não retornou uma textura".into()))?;

    Ok(crate::texture::Texture2D::from_raw(
        texture,
        desc.width,
        desc.height,
        desc.format,
    ))
}

pub(crate) fn create_shader_resource_view(
    device: &Device,
    texture: &crate::texture::Texture2D,
) -> Result<crate::view::ShaderResourceView, GraphicsError> {
    let mut view: Option<ID3D11ShaderResourceView> = None;
    unsafe {
        device
            .raw
            .device
            .CreateShaderResourceView(&texture.raw, None, Some(&mut view))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let view = view.ok_or_else(|| {
        GraphicsError::Backend("CreateShaderResourceView não retornou uma view".into())
    })?;

    Ok(crate::view::ShaderResourceView::from_raw(view))
}

pub(crate) fn create_sampler(
    device: &Device,
    desc: &crate::sampler::SamplerDescriptor,
) -> Result<crate::sampler::Sampler, GraphicsError> {
    use crate::sampler::{AddressMode, FilterMode};

    let filter = match desc.filter {
        FilterMode::Point => D3D11_FILTER_MIN_MAG_MIP_POINT,
        FilterMode::Linear => D3D11_FILTER_MIN_MAG_MIP_LINEAR,
    };
    let address = match desc.address_mode {
        AddressMode::Wrap => D3D11_TEXTURE_ADDRESS_WRAP,
        AddressMode::Mirror => D3D11_TEXTURE_ADDRESS_MIRROR,
        AddressMode::Clamp => D3D11_TEXTURE_ADDRESS_CLAMP,
    };

    let sampler_desc = D3D11_SAMPLER_DESC {
        Filter: filter,
        AddressU: address,
        AddressV: address,
        AddressW: address,
        MipLODBias: 0.0,
        MaxAnisotropy: 1,
        ComparisonFunc: windows::Win32::Graphics::Direct3D11::D3D11_COMPARISON_NEVER,
        BorderColor: [0.0; 4],
        MinLOD: 0.0,
        MaxLOD: f32::MAX,
    };

    let mut sampler: Option<ID3D11SamplerState> = None;
    unsafe {
        device
            .raw
            .device
            .CreateSamplerState(&sampler_desc, Some(&mut sampler))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let sampler = sampler
        .ok_or_else(|| GraphicsError::Backend("CreateSamplerState não retornou um sampler".into()))?;

    Ok(crate::sampler::Sampler { raw: sampler })
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

/// Cria a [`crate::view::RenderTargetView`] do back buffer atual da
/// swapchain — precisa ser recriada após `resize_swapchain`, já que o
/// buffer físico por trás dela muda.
pub(crate) fn create_swapchain_render_target_view(
    device: &Device,
    raw: &RawSwapchain,
) -> Result<crate::view::RenderTargetView, GraphicsError> {
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

    Ok(crate::view::RenderTargetView::from_raw(render_target_view))
}

/// Apresenta o quadro atual da swapchain.
pub(crate) fn present(raw: &RawSwapchain, present_mode: PresentMode) -> Result<PresentResult, GraphicsError> {
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
