//! Backend Direct3D 11, via `windows-rs` (crate `windows`).
//!
//! Primeira implementação real de `ndl-render`: cria o dispositivo D3D11
//! e enumera adaptadores via DXGI. O resto da crate (buffer, texture,
//! pipeline, ...) ainda não tem contrapartida aqui — isso vai sendo
//! adicionado módulo a módulo.

use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;
use windows::Win32::Graphics::Direct3D::ID3DBlob;
use windows::Win32::Graphics::Direct3D::{
    D3D_DRIVER_TYPE, D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP, D3D_FEATURE_LEVEL,
    D3D_FEATURE_LEVEL_9_2, D3D_FEATURE_LEVEL_9_3, D3D_FEATURE_LEVEL_10_0, D3D_FEATURE_LEVEL_10_1,
    D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1,
};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_DEPTH_STENCIL, D3D11_BIND_FLAG, D3D11_BIND_INDEX_BUFFER,
    D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE, D3D11_BIND_UNORDERED_ACCESS,
    D3D11_BIND_VERTEX_BUFFER,
    D3D11_BUFFER_DESC, D3D11_COMPARISON_ALWAYS, D3D11_COMPARISON_EQUAL, D3D11_COMPARISON_FUNC,
    D3D11_COMPARISON_GREATER, D3D11_COMPARISON_GREATER_EQUAL, D3D11_COMPARISON_LESS,
    D3D11_COMPARISON_LESS_EQUAL, D3D11_COMPARISON_NEVER, D3D11_COMPARISON_NOT_EQUAL,
    D3D11_CPU_ACCESS_WRITE, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_DEBUG,
    D3D11_CREATE_DEVICE_FLAG, D3D11_DEPTH_STENCIL_DESC, D3D11_DEPTH_WRITE_MASK_ALL,
    D3D11_DEPTH_WRITE_MASK_ZERO, D3D11_FEATURE_D3D10_X_HARDWARE_OPTIONS,
    D3D11_FEATURE_D3D11_OPTIONS, D3D11_FEATURE_DATA_D3D10_X_HARDWARE_OPTIONS,
    D3D11_FEATURE_DATA_D3D11_OPTIONS, D3D11_FILTER_MIN_MAG_MIP_LINEAR,
    D3D11_FILTER_MIN_MAG_MIP_POINT, D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA,
    D3D11_MAP_WRITE_DISCARD, D3D11_MAPPED_SUBRESOURCE, D3D11_QUERY_DESC, D3D11_QUERY_EVENT,
    D3D11_INPUT_PER_INSTANCE_DATA, D3D11_RESOURCE_MISC_BUFFER_STRUCTURED,
    D3D11_RESOURCE_MISC_DRAWINDIRECT_ARGS, D3D11_RESOURCE_MISC_GENERATE_MIPS, D3D11_SAMPLER_DESC,
    D3D11_SDK_VERSION,
    D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE_ADDRESS_CLAMP, D3D11_TEXTURE_ADDRESS_MIRROR,
    D3D11_TEXTURE_ADDRESS_WRAP, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, D3D11_USAGE_DYNAMIC,
    D3D11CreateDevice, ID3D11Buffer, ID3D11CommandList, ID3D11DepthStencilState,
    ID3D11DepthStencilView, ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout,
    ID3D11PixelShader, ID3D11Query, ID3D11RasterizerState, ID3D11RenderTargetView,
    ID3D11SamplerState, ID3D11ShaderResourceView, ID3D11Texture2D, ID3D11VertexShader,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_ALPHA_MODE_IGNORE, DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_PRESENT, DXGI_SCALING_STRETCH,
    DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_CHAIN_FLAG, DXGI_SWAP_EFFECT_FLIP_DISCARD,
    DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGIDevice, IDXGIFactory1, IDXGIFactory2, IDXGISwapChain1,
};
use windows::core::Interface;
use windows::core::PCSTR;

use crate::core::{
    AdapterInfo, CapabilitySource, Device, DeviceCapabilities, DeviceDescriptor, DeviceLimits,
    FeatureSupport, GraphicsError, LimitsSource,
};
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

/// Traduz o nivel D3D11 para capacidades portaveis e limites garantidos.
/// A conversao permanece dentro deste modulo para nao expor Feature Level
/// na API publica do renderer.
pub(crate) fn device_properties(
    raw: &RawDevice,
) -> (DeviceCapabilities, DeviceLimits, LimitsSource) {
    let (supports_tessellation, mut limits) = match raw.feature_level {
        D3D_FEATURE_LEVEL_11_1 => (
            true,
            d3d11_limits(16384, 2048, 8, 16, 8, 16384, 65535, 128, 32),
        ),
        D3D_FEATURE_LEVEL_11_0 => (
            true,
            d3d11_limits(16384, 2048, 8, 16, 8, 16384, 65535, 128, 32),
        ),
        D3D_FEATURE_LEVEL_10_1 | D3D_FEATURE_LEVEL_10_0 => (
            false,
            d3d11_limits(8192, 2048, 8, 16, 1, 8192, 65535, 128, 16),
        ),
        D3D_FEATURE_LEVEL_9_3 => (false, d3d11_limits(4096, 256, 4, 16, 0, 4096, 0, 16, 16)),
        D3D_FEATURE_LEVEL_9_2 => (false, d3d11_limits(2048, 256, 1, 2, 0, 2048, 0, 16, 16)),
        _ => (false, d3d11_limits(2048, 256, 1, 2, 0, 2048, 0, 16, 16)),
    };

    let compute = query_compute_support(raw);
    if !compute.supported {
        limits.max_storage_buffer_bindings = 0;
        limits.max_compute_workgroups_per_dimension = 0;
    }
    let logic_ops = query_logic_op_support(raw);

    (
        DeviceCapabilities {
            compute,
            tessellation: FeatureSupport {
                supported: supports_tessellation,
                source: CapabilitySource::ApiGuarantee,
            },
            logic_ops,
        },
        limits,
        LimitsSource::ApiGuarantee,
    )
}

fn query_logic_op_support(raw: &RawDevice) -> FeatureSupport {
    let mut options = D3D11_FEATURE_DATA_D3D11_OPTIONS::default();
    let query = unsafe {
        raw.device.CheckFeatureSupport(
            D3D11_FEATURE_D3D11_OPTIONS,
            (&mut options as *mut D3D11_FEATURE_DATA_D3D11_OPTIONS).cast(),
            std::mem::size_of::<D3D11_FEATURE_DATA_D3D11_OPTIONS>() as u32,
        )
    };

    match query {
        Ok(()) => FeatureSupport {
            supported: options.OutputMergerLogicOp.as_bool(),
            source: CapabilitySource::DeviceQuery,
        },
        Err(_) if raw.feature_level == D3D_FEATURE_LEVEL_11_1 => FeatureSupport {
            supported: true,
            source: CapabilitySource::ApiGuarantee,
        },
        Err(_) => FeatureSupport {
            supported: false,
            source: CapabilitySource::ConservativeFallback,
        },
    }
}

fn query_compute_support(raw: &RawDevice) -> FeatureSupport {
    if raw.feature_level == D3D_FEATURE_LEVEL_11_1 || raw.feature_level == D3D_FEATURE_LEVEL_11_0 {
        return FeatureSupport {
            supported: true,
            source: CapabilitySource::ApiGuarantee,
        };
    }

    if raw.feature_level == D3D_FEATURE_LEVEL_10_0 || raw.feature_level == D3D_FEATURE_LEVEL_10_1 {
        let mut options = D3D11_FEATURE_DATA_D3D10_X_HARDWARE_OPTIONS::default();
        let query = unsafe {
            raw.device.CheckFeatureSupport(
                D3D11_FEATURE_D3D10_X_HARDWARE_OPTIONS,
                (&mut options as *mut D3D11_FEATURE_DATA_D3D10_X_HARDWARE_OPTIONS).cast(),
                std::mem::size_of::<D3D11_FEATURE_DATA_D3D10_X_HARDWARE_OPTIONS>() as u32,
            )
        };
        return match query {
            Ok(()) => FeatureSupport {
                supported: options
                    .ComputeShaders_Plus_RawAndStructuredBuffers_Via_Shader_4_x
                    .as_bool(),
                source: CapabilitySource::DeviceQuery,
            },
            Err(_) => FeatureSupport {
                supported: false,
                source: CapabilitySource::ConservativeFallback,
            },
        };
    }

    FeatureSupport {
        supported: false,
        source: CapabilitySource::ApiGuarantee,
    }
}

fn d3d11_limits(
    max_texture_dimension_2d: u32,
    max_texture_dimension_3d: u32,
    max_color_attachments: u32,
    max_sampler_anisotropy: u32,
    max_storage_buffer_bindings: u32,
    max_viewport_dimension: u32,
    max_compute_workgroups_per_dimension: u32,
    max_sampled_resource_bindings: u32,
    max_vertex_buffer_bindings: u32,
) -> DeviceLimits {
    DeviceLimits {
        max_buffer_size: u32::MAX as u64,
        max_texture_dimension_2d,
        max_texture_dimension_3d,
        max_color_attachments,
        max_sampler_anisotropy,
        max_storage_buffer_bindings,
        max_uniform_buffer_bindings: 14,
        max_sampled_resource_bindings,
        max_sampler_bindings: 16,
        max_vertex_buffer_bindings,
        max_viewport_dimension,
        max_compute_workgroups_per_dimension,
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

        let name_len = desc
            .Description
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(desc.Description.len());
        let name = String::from_utf16_lossy(&desc.Description[..name_len]);
        let is_software = (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0;

        adapters.push(AdapterInfo {
            name,
            vendor_id: desc.VendorId,
            device_id: desc.DeviceId,
            dedicated_video_memory: desc.DedicatedVideoMemory as u64,
            shared_system_memory: desc.SharedSystemMemory as u64,
            is_software,
            device_type: if is_software {
                crate::core::AdapterType::Software
            } else {
                crate::core::AdapterType::Hardware
            },
            driver_version: unsafe { adapter.CheckInterfaceSupport(&ID3D11Device::IID) }
                .ok()
                .map(format_driver_version),
        });
    }

    Ok(adapters)
}

pub(crate) fn adapter_info_for_device(raw: &RawDevice) -> Result<AdapterInfo, GraphicsError> {
    use windows::Win32::Graphics::Dxgi::{IDXGIAdapter1, IDXGIDevice};
    let dxgi_device: IDXGIDevice = raw
        .device
        .cast()
        .map_err(|e| GraphicsError::Backend(e.to_string()))?;
    let adapter =
        unsafe { dxgi_device.GetAdapter() }.map_err(|e| GraphicsError::Backend(e.to_string()))?;
    let adapter1: IDXGIAdapter1 = adapter
        .cast()
        .map_err(|e| GraphicsError::Backend(e.to_string()))?;
    let desc = unsafe { adapter1.GetDesc1() }.map_err(|e| GraphicsError::Backend(e.to_string()))?;
    let name_len = desc
        .Description
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(desc.Description.len());
    let is_software = (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0;
    Ok(AdapterInfo {
        name: String::from_utf16_lossy(&desc.Description[..name_len]),
        vendor_id: desc.VendorId,
        device_id: desc.DeviceId,
        dedicated_video_memory: desc.DedicatedVideoMemory as u64,
        shared_system_memory: desc.SharedSystemMemory as u64,
        is_software,
        device_type: if is_software {
            crate::core::AdapterType::Software
        } else {
            crate::core::AdapterType::Hardware
        },
        driver_version: unsafe { adapter1.CheckInterfaceSupport(&ID3D11Device::IID) }
            .ok()
            .map(format_driver_version),
    })
}

fn format_driver_version(version: i64) -> String {
    let v = version as u64;
    format!(
        "{}.{}.{}.{}",
        v >> 48,
        (v >> 32) & 0xffff,
        (v >> 16) & 0xffff,
        v & 0xffff
    )
}

pub(crate) fn query_format_features(
    device: &Device,
    format: crate::format::Format,
) -> Result<crate::core::FormatFeatures, GraphicsError> {
    use windows::Win32::Graphics::Direct3D11::{
        D3D11_FORMAT_SUPPORT_DEPTH_STENCIL, D3D11_FORMAT_SUPPORT_IA_VERTEX_BUFFER,
        D3D11_FORMAT_SUPPORT_RENDER_TARGET, D3D11_FORMAT_SUPPORT_SHADER_SAMPLE,
        D3D11_FORMAT_SUPPORT_TEXTURE2D, D3D11_FORMAT_SUPPORT_TYPED_UNORDERED_ACCESS_VIEW,
    };
    let dxgi = format
        .to_dxgi_format()
        .ok_or_else(|| GraphicsError::FeatureNotSupported {
            feature: format!("formato {format:?} sem mapeamento neste backend"),
        })?;
    let mask = unsafe { device.raw.device.CheckFormatSupport(dxgi) }
        .map_err(|e| GraphicsError::Backend(e.to_string()))?;
    let has = |flag: windows::Win32::Graphics::Direct3D11::D3D11_FORMAT_SUPPORT| {
        mask & flag.0 as u32 != 0
    };
    Ok(crate::core::FormatFeatures {
        texture_2d: has(D3D11_FORMAT_SUPPORT_TEXTURE2D),
        sampled: has(D3D11_FORMAT_SUPPORT_SHADER_SAMPLE),
        render_target: has(D3D11_FORMAT_SUPPORT_RENDER_TARGET),
        depth_stencil: has(D3D11_FORMAT_SUPPORT_DEPTH_STENCIL),
        vertex_buffer: has(D3D11_FORMAT_SUPPORT_IA_VERTEX_BUFFER),
        storage: has(D3D11_FORMAT_SUPPORT_TYPED_UNORDERED_ACCESS_VIEW),
    })
}

fn bind_flag_for_usage(usage: crate::buffer::BufferUsage) -> D3D11_BIND_FLAG {
    use crate::buffer::BufferUsage::*;
    match usage {
        Vertex => D3D11_BIND_VERTEX_BUFFER,
        Index => D3D11_BIND_INDEX_BUFFER,
        Constant => D3D11_BIND_CONSTANT_BUFFER,
        Storage => D3D11_BIND_FLAG(D3D11_BIND_SHADER_RESOURCE.0 | D3D11_BIND_UNORDERED_ACCESS.0),
        Staging => D3D11_BIND_FLAG(0),
        // Um buffer de argumentos indiretos não precisa de bind a nenhum
        // estágio — D3D11_RESOURCE_MISC_DRAWINDIRECT_ARGS (abaixo) já é o
        // que permite usá-lo em Draw*Indirect/DispatchIndirect.
        Indirect => D3D11_BIND_FLAG(0),
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

    let is_staging = matches!(desc.usage, crate::buffer::BufferUsage::Staging);

    let buffer_desc = D3D11_BUFFER_DESC {
        ByteWidth: desc.size as u32,
        Usage: if is_staging {
            windows::Win32::Graphics::Direct3D11::D3D11_USAGE_STAGING
        } else if desc.dynamic {
            D3D11_USAGE_DYNAMIC
        } else {
            D3D11_USAGE_DEFAULT
        },
        BindFlags: bind_flag_for_usage(desc.usage).0 as u32,
        CPUAccessFlags: if is_staging {
            (windows::Win32::Graphics::Direct3D11::D3D11_CPU_ACCESS_READ.0
                | D3D11_CPU_ACCESS_WRITE.0) as u32
        } else if desc.dynamic {
            D3D11_CPU_ACCESS_WRITE.0 as u32
        } else {
            0
        },
        MiscFlags: if matches!(desc.usage, crate::buffer::BufferUsage::Storage) {
            D3D11_RESOURCE_MISC_BUFFER_STRUCTURED.0 as u32
        } else if matches!(desc.usage, crate::buffer::BufferUsage::Indirect) {
            D3D11_RESOURCE_MISC_DRAWINDIRECT_ARGS.0 as u32
        } else {
            0
        },
        StructureByteStride: if matches!(desc.usage, crate::buffer::BufferUsage::Storage) {
            desc.stride
        } else {
            0
        },
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

    Ok(crate::buffer::Buffer::from_raw(
        buffer,
        desc.size,
        desc.usage,
        desc.stride,
        desc.dynamic,
    ))
}

pub(crate) fn create_unordered_access_view(
    device: &Device,
    buffer: &crate::buffer::Buffer,
) -> Result<crate::view::UnorderedAccessView, GraphicsError> {
    use windows::Win32::Graphics::Direct3D11::{
        D3D11_BUFFER_UAV, D3D11_UAV_DIMENSION_BUFFER, D3D11_UNORDERED_ACCESS_VIEW_DESC,
        D3D11_UNORDERED_ACCESS_VIEW_DESC_0, ID3D11UnorderedAccessView,
    };

    let num_elements = (buffer.size() / buffer.stride().max(1) as u64) as u32;
    let uav_desc = D3D11_UNORDERED_ACCESS_VIEW_DESC {
        Format: DXGI_FORMAT_UNKNOWN,
        ViewDimension: D3D11_UAV_DIMENSION_BUFFER,
        Anonymous: D3D11_UNORDERED_ACCESS_VIEW_DESC_0 {
            Buffer: D3D11_BUFFER_UAV {
                FirstElement: 0,
                NumElements: num_elements,
                Flags: 0,
            },
        },
    };

    let mut view: Option<ID3D11UnorderedAccessView> = None;
    unsafe {
        device
            .raw
            .device
            .CreateUnorderedAccessView(&buffer.raw, Some(&uav_desc), Some(&mut view))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let view = view.ok_or_else(|| {
        GraphicsError::Backend("CreateUnorderedAccessView não retornou uma view".into())
    })?;

    Ok(crate::view::UnorderedAccessView::from_raw(view))
}

pub(crate) fn create_compute_shader(
    device: &Device,
    bytecode: &[u8],
) -> Result<crate::shader::ComputeShader, GraphicsError> {
    use windows::Win32::Graphics::Direct3D11::ID3D11ComputeShader;

    let mut shader: Option<ID3D11ComputeShader> = None;
    unsafe {
        device
            .raw
            .device
            .CreateComputeShader(bytecode, None, Some(&mut shader))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;
    let shader = shader.ok_or_else(|| {
        GraphicsError::Backend("CreateComputeShader não retornou um shader".into())
    })?;
    Ok(crate::shader::ComputeShader { raw: shader })
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

    let code =
        code.ok_or_else(|| GraphicsError::Backend("D3DCompile não retornou bytecode".into()))?;
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
    let shader = shader
        .ok_or_else(|| GraphicsError::Backend("CreatePixelShader não retornou um shader".into()))?;
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
            let is_instance = attr.step_mode == crate::input::VertexStepMode::Instance;
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: PCSTR(name.as_ptr() as *const u8),
                SemanticIndex: attr.semantic_index,
                Format: format,
                InputSlot: attr.input_slot,
                AlignedByteOffset: attr.offset,
                InputSlotClass: if is_instance {
                    D3D11_INPUT_PER_INSTANCE_DATA
                } else {
                    D3D11_INPUT_PER_VERTEX_DATA
                },
                // 1 elemento de instância por instância desenhada — o caso
                // comum; uma taxa diferente (ex: 1 a cada N instâncias)
                // continua reservada enquanto não houver um uso real para ela.
                InstanceDataStepRate: if is_instance { 1 } else { 0 },
            }
        })
        .collect();

    let mut layout: Option<ID3D11InputLayout> = None;
    unsafe {
        device
            .raw
            .device
            .CreateInputLayout(&descs, &vertex_shader.bytecode, Some(&mut layout))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let layout = layout.ok_or_else(|| {
        GraphicsError::Backend("CreateInputLayout não retornou um input layout".into())
    })?;

    Ok(crate::input::InputLayout { raw: layout })
}

/// Dá um nome a qualquer recurso D3D11 (`ID3D11DeviceChild`) — visível em
/// ferramentas de debug gráfico. Resources que não suportam
/// `SetPrivateData` (não deveria acontecer para buffers/texturas) só
/// ignoram silenciosamente, já que um nome de debug nunca deveria
/// interromper o programa.
pub(crate) fn set_resource_label<T: windows::core::Interface>(resource: &T, label: &str) {
    use windows::Win32::Graphics::Direct3D::WKPDID_D3DDebugObjectName;
    use windows::Win32::Graphics::Direct3D11::ID3D11DeviceChild;

    let Ok(child) = resource.cast::<ID3D11DeviceChild>() else {
        return;
    };
    let Ok(name) = std::ffi::CString::new(label) else {
        return;
    };
    unsafe {
        let _ = child.SetPrivateData(
            &WKPDID_D3DDebugObjectName,
            name.as_bytes().len() as u32,
            Some(name.as_ptr() as *const _),
        );
    }
}

pub(crate) fn map_write_discard<'a>(
    context: &'a crate::context::Context,
    buffer: &'a crate::buffer::Buffer,
) -> Result<crate::memory::MappedBuffer<'a>, GraphicsError> {
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    unsafe {
        context.raw.Map(
            &buffer.raw,
            0,
            D3D11_MAP_WRITE_DISCARD,
            0,
            Some(&mut mapped),
        )
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    Ok(crate::memory::MappedBuffer {
        context,
        buffer,
        ptr: mapped.pData as *mut u8,
        size: buffer.size() as usize,
    })
}

pub(crate) fn map_read<'a>(
    context: &'a crate::context::Context,
    buffer: &'a crate::buffer::Buffer,
) -> Result<crate::memory::ReadMappedBuffer<'a>, GraphicsError> {
    use windows::Win32::Graphics::Direct3D11::D3D11_MAP_READ;

    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    unsafe {
        context
            .raw
            .Map(&buffer.raw, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    Ok(crate::memory::ReadMappedBuffer {
        context,
        buffer,
        ptr: mapped.pData as *const u8,
        size: buffer.size() as usize,
    })
}

pub(crate) fn unmap_buffer(context: &crate::context::Context, buffer: &crate::buffer::Buffer) {
    unsafe {
        context.raw.Unmap(&buffer.raw, 0);
    }
}

pub(crate) fn create_command_encoder(
    device: &Device,
) -> Result<crate::command::CommandEncoder, GraphicsError> {
    let mut deferred: Option<ID3D11DeviceContext> = None;
    unsafe {
        device
            .raw
            .device
            .CreateDeferredContext(0, Some(&mut deferred))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let deferred = deferred.ok_or_else(|| {
        GraphicsError::Backend("CreateDeferredContext não retornou um contexto".into())
    })?;

    Ok(crate::command::CommandEncoder::from_context(
        crate::context::Context::from_raw(deferred, device.limits()),
    ))
}

pub(crate) fn finish_command_list(
    context: &crate::context::Context,
) -> Result<crate::command::CommandBuffer, GraphicsError> {
    let mut command_list: Option<ID3D11CommandList> = None;
    unsafe {
        context
            .raw
            .FinishCommandList(false, Some(&mut command_list))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let command_list = command_list.ok_or_else(|| {
        GraphicsError::Backend("FinishCommandList não retornou uma lista de comandos".into())
    })?;

    Ok(crate::command::CommandBuffer::from_raw(command_list))
}

pub(crate) fn execute_command_list(
    context: &crate::context::Context,
    buffer: &crate::command::CommandBuffer,
) {
    unsafe {
        context.raw.ExecuteCommandList(&buffer.raw, false);
    }
}

pub(crate) fn wait_idle(context: &crate::context::Context) -> Result<(), GraphicsError> {
    let device: ID3D11Device = unsafe { context.raw.GetDevice() }
        .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let query_desc = D3D11_QUERY_DESC {
        Query: D3D11_QUERY_EVENT,
        MiscFlags: 0,
    };
    let mut query: Option<ID3D11Query> = None;
    unsafe { device.CreateQuery(&query_desc, Some(&mut query)) }
        .map_err(|err| GraphicsError::Backend(err.to_string()))?;
    let query = query.ok_or_else(|| GraphicsError::Backend("CreateQuery falhou".into()))?;

    unsafe {
        context.raw.End(&query);
    }

    // `result` começa zerado (FALSE) e GetData só escreve TRUE quando a
    // GPU realmente terminou — enquanto a query não estiver pronta,
    // GetData retorna S_FALSE sem tocar no buffer, então o polling padrão
    // é checar o valor em si, não só se a chamada "deu erro".
    let mut result = windows::Win32::Foundation::BOOL(0);
    while !result.as_bool() {
        let _ = unsafe {
            context.raw.GetData(
                &query,
                Some(&mut result as *mut _ as *mut _),
                std::mem::size_of_val(&result) as u32,
                0,
            )
        };
        std::thread::yield_now();
    }

    Ok(())
}

pub(crate) fn create_query(
    device: &Device,
    kind: crate::query::QueryType,
) -> Result<crate::query::Query, GraphicsError> {
    use windows::Win32::Graphics::Direct3D11::D3D11_QUERY_OCCLUSION;

    let query_type = match kind {
        crate::query::QueryType::Occlusion => D3D11_QUERY_OCCLUSION,
        crate::query::QueryType::Event => D3D11_QUERY_EVENT,
    };
    let query_desc = D3D11_QUERY_DESC {
        Query: query_type,
        MiscFlags: 0,
    };

    let mut query: Option<ID3D11Query> = None;
    unsafe { device.raw.device.CreateQuery(&query_desc, Some(&mut query)) }
        .map_err(|err| GraphicsError::Backend(err.to_string()))?;
    let query =
        query.ok_or_else(|| GraphicsError::Backend("CreateQuery não retornou uma query".into()))?;

    Ok(crate::query::Query { raw: query, kind })
}

/// `D3D11_ASYNC_GETDATA_DONOTFLUSH`: sem essa flag, `GetData` pode
/// bloquear internamente até o resultado ficar pronto — com ela, só
/// consulta o estado atual.
const ASYNC_GETDATA_DONOTFLUSH: u32 = 1;

pub(crate) fn is_query_ready(
    context: &crate::context::Context,
    query: &crate::query::Query,
) -> bool {
    let mut result = windows::Win32::Foundation::BOOL(0);
    let got_data = unsafe {
        context.raw.GetData(
            &query.raw,
            Some(&mut result as *mut _ as *mut _),
            std::mem::size_of_val(&result) as u32,
            ASYNC_GETDATA_DONOTFLUSH,
        )
    };
    got_data.is_ok() && result.as_bool()
}

pub(crate) fn get_occlusion_result(
    context: &crate::context::Context,
    query: &crate::query::Query,
) -> Option<u64> {
    let mut result: u64 = 0;
    let got_data = unsafe {
        context.raw.GetData(
            &query.raw,
            Some(&mut result as *mut _ as *mut _),
            std::mem::size_of_val(&result) as u32,
            ASYNC_GETDATA_DONOTFLUSH,
        )
    };
    if got_data.is_ok() { Some(result) } else { None }
}

fn user_defined_annotation(
    context: &crate::context::Context,
) -> Option<windows::Win32::Graphics::Direct3D11::ID3DUserDefinedAnnotation> {
    context.raw.cast().ok()
}

fn to_pcwstr_vec(label: &str) -> Vec<u16> {
    label.encode_utf16().chain(std::iter::once(0)).collect()
}

pub(crate) fn push_debug_group(context: &crate::context::Context, label: &str) {
    if let Some(annotation) = user_defined_annotation(context) {
        let wide = to_pcwstr_vec(label);
        unsafe {
            annotation.BeginEvent(windows::core::PCWSTR(wide.as_ptr()));
        }
    }
}

pub(crate) fn pop_debug_group(context: &crate::context::Context) {
    if let Some(annotation) = user_defined_annotation(context) {
        unsafe {
            annotation.EndEvent();
        }
    }
}

pub(crate) fn insert_debug_marker(context: &crate::context::Context, label: &str) {
    if let Some(annotation) = user_defined_annotation(context) {
        let wide = to_pcwstr_vec(label);
        unsafe {
            annotation.SetMarker(windows::core::PCWSTR(wide.as_ptr()));
        }
    }
}

fn blend_factor_to_d3d(
    factor: crate::blend::BlendFactor,
) -> windows::Win32::Graphics::Direct3D11::D3D11_BLEND {
    use crate::blend::BlendFactor::*;
    use windows::Win32::Graphics::Direct3D11::*;
    match factor {
        Zero => D3D11_BLEND_ZERO,
        One => D3D11_BLEND_ONE,
        SrcColor => D3D11_BLEND_SRC_COLOR,
        OneMinusSrcColor => D3D11_BLEND_INV_SRC_COLOR,
        DstColor => D3D11_BLEND_DEST_COLOR,
        OneMinusDstColor => D3D11_BLEND_INV_DEST_COLOR,
        SrcAlpha => D3D11_BLEND_SRC_ALPHA,
        OneMinusSrcAlpha => D3D11_BLEND_INV_SRC_ALPHA,
        DstAlpha => D3D11_BLEND_DEST_ALPHA,
        OneMinusDstAlpha => D3D11_BLEND_INV_DEST_ALPHA,
    }
}

fn blend_op_to_d3d(
    op: crate::blend::BlendOp,
) -> windows::Win32::Graphics::Direct3D11::D3D11_BLEND_OP {
    use crate::blend::BlendOp::*;
    use windows::Win32::Graphics::Direct3D11::*;
    match op {
        Add => D3D11_BLEND_OP_ADD,
        Subtract => D3D11_BLEND_OP_SUBTRACT,
        ReverseSubtract => D3D11_BLEND_OP_REV_SUBTRACT,
        Min => D3D11_BLEND_OP_MIN,
        Max => D3D11_BLEND_OP_MAX,
    }
}

pub(crate) fn create_blend_state(
    device: &Device,
    desc: &crate::blend::BlendStateDescriptor,
) -> Result<crate::blend::BlendState, GraphicsError> {
    use windows::Win32::Graphics::Direct3D11::{D3D11_BLEND_DESC, D3D11_RENDER_TARGET_BLEND_DESC};

    let render_target = D3D11_RENDER_TARGET_BLEND_DESC {
        BlendEnable: desc.enabled.into(),
        SrcBlend: blend_factor_to_d3d(desc.src_factor),
        DestBlend: blend_factor_to_d3d(desc.dst_factor),
        BlendOp: blend_op_to_d3d(desc.op),
        SrcBlendAlpha: blend_factor_to_d3d(desc.src_alpha_factor),
        DestBlendAlpha: blend_factor_to_d3d(desc.dst_alpha_factor),
        BlendOpAlpha: blend_op_to_d3d(desc.alpha_op),
        RenderTargetWriteMask: desc.write_mask.0,
    };

    let mut blend_desc = D3D11_BLEND_DESC::default();
    blend_desc.RenderTarget[0] = render_target;

    let mut state: Option<windows::Win32::Graphics::Direct3D11::ID3D11BlendState> = None;
    unsafe {
        device
            .raw
            .device
            .CreateBlendState(&blend_desc, Some(&mut state))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let state = state
        .ok_or_else(|| GraphicsError::Backend("CreateBlendState não retornou um estado".into()))?;

    Ok(crate::blend::BlendState { raw: state })
}

pub(crate) fn create_rasterizer_state(
    device: &Device,
    desc: &crate::raster::RasterizerStateDescriptor,
) -> Result<crate::raster::RasterizerState, GraphicsError> {
    use crate::raster::{CullMode, FillMode, FrontFace};
    use windows::Win32::Graphics::Direct3D11::{
        D3D11_CULL_BACK, D3D11_CULL_FRONT, D3D11_CULL_NONE, D3D11_FILL_SOLID, D3D11_FILL_WIREFRAME,
        D3D11_RASTERIZER_DESC,
    };
    let native = D3D11_RASTERIZER_DESC {
        FillMode: match desc.fill_mode {
            FillMode::Solid => D3D11_FILL_SOLID,
            FillMode::Wireframe => D3D11_FILL_WIREFRAME,
        },
        CullMode: match desc.cull_mode {
            CullMode::None => D3D11_CULL_NONE,
            CullMode::Front => D3D11_CULL_FRONT,
            CullMode::Back => D3D11_CULL_BACK,
        },
        FrontCounterClockwise: matches!(desc.front_face, FrontFace::CounterClockwise).into(),
        DepthBias: desc.depth_bias,
        DepthBiasClamp: desc.depth_bias_clamp,
        SlopeScaledDepthBias: desc.slope_scaled_depth_bias,
        DepthClipEnable: desc.depth_clip_enabled.into(),
        ScissorEnable: false.into(),
        MultisampleEnable: false.into(),
        AntialiasedLineEnable: false.into(),
    };
    let mut state: Option<ID3D11RasterizerState> = None;
    unsafe {
        device
            .raw
            .device
            .CreateRasterizerState(&native, Some(&mut state))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;
    let raw = state.ok_or_else(|| {
        GraphicsError::Backend("CreateRasterizerState não retornou um estado".into())
    })?;
    Ok(crate::raster::RasterizerState { raw })
}

fn compare_op_to_d3d(op: crate::depth_stencil::CompareOp) -> D3D11_COMPARISON_FUNC {
    use crate::depth_stencil::CompareOp::*;
    match op {
        Never => D3D11_COMPARISON_NEVER,
        Less => D3D11_COMPARISON_LESS,
        Equal => D3D11_COMPARISON_EQUAL,
        LessEqual => D3D11_COMPARISON_LESS_EQUAL,
        Greater => D3D11_COMPARISON_GREATER,
        NotEqual => D3D11_COMPARISON_NOT_EQUAL,
        GreaterEqual => D3D11_COMPARISON_GREATER_EQUAL,
        Always => D3D11_COMPARISON_ALWAYS,
    }
}

fn stencil_op_to_d3d(
    op: crate::depth_stencil::StencilOp,
) -> windows::Win32::Graphics::Direct3D11::D3D11_STENCIL_OP {
    use crate::depth_stencil::StencilOp::*;
    use windows::Win32::Graphics::Direct3D11::*;
    match op {
        Keep => D3D11_STENCIL_OP_KEEP,
        Zero => D3D11_STENCIL_OP_ZERO,
        Replace => D3D11_STENCIL_OP_REPLACE,
        IncrementClamp => D3D11_STENCIL_OP_INCR_SAT,
        DecrementClamp => D3D11_STENCIL_OP_DECR_SAT,
        Invert => D3D11_STENCIL_OP_INVERT,
        IncrementWrap => D3D11_STENCIL_OP_INCR,
        DecrementWrap => D3D11_STENCIL_OP_DECR,
    }
}

fn stencil_face_to_d3d(
    desc: crate::depth_stencil::StencilFaceDescriptor,
) -> windows::Win32::Graphics::Direct3D11::D3D11_DEPTH_STENCILOP_DESC {
    windows::Win32::Graphics::Direct3D11::D3D11_DEPTH_STENCILOP_DESC {
        StencilFailOp: stencil_op_to_d3d(desc.fail_op),
        StencilDepthFailOp: stencil_op_to_d3d(desc.depth_fail_op),
        StencilPassOp: stencil_op_to_d3d(desc.pass_op),
        StencilFunc: compare_op_to_d3d(desc.compare),
    }
}

pub(crate) fn create_depth_stencil_state(
    device: &Device,
    desc: &crate::depth_stencil::DepthStencilStateDescriptor,
) -> Result<crate::depth_stencil::DepthStencilState, GraphicsError> {
    let state_desc = D3D11_DEPTH_STENCIL_DESC {
        DepthEnable: desc.depth_test_enabled.into(),
        DepthWriteMask: if desc.depth_write_enabled {
            D3D11_DEPTH_WRITE_MASK_ALL
        } else {
            D3D11_DEPTH_WRITE_MASK_ZERO
        },
        DepthFunc: compare_op_to_d3d(desc.depth_compare),
        StencilEnable: desc.stencil_test_enabled.into(),
        StencilReadMask: desc.stencil_read_mask,
        StencilWriteMask: desc.stencil_write_mask,
        FrontFace: stencil_face_to_d3d(desc.front_face),
        BackFace: stencil_face_to_d3d(desc.back_face),
        ..Default::default()
    };

    let mut state: Option<ID3D11DepthStencilState> = None;
    unsafe {
        device
            .raw
            .device
            .CreateDepthStencilState(&state_desc, Some(&mut state))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let state = state.ok_or_else(|| {
        GraphicsError::Backend("CreateDepthStencilState não retornou um estado".into())
    })?;

    Ok(crate::depth_stencil::DepthStencilState { raw: state })
}

pub(crate) fn create_depth_stencil_view(
    device: &Device,
    width: u32,
    height: u32,
    format: crate::format::Format,
) -> Result<crate::view::DepthStencilView, GraphicsError> {
    let dxgi_format = format.to_dxgi_format().ok_or(GraphicsError::Unsupported(
        "formato não suportado para depth buffer",
    ))?;

    let texture_desc = D3D11_TEXTURE2D_DESC {
        Width: width,
        Height: height,
        MipLevels: 1,
        ArraySize: 1,
        Format: dxgi_format,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_DEPTH_STENCIL.0 as u32,
        CPUAccessFlags: 0,
        MiscFlags: 0,
    };

    let mut texture: Option<ID3D11Texture2D> = None;
    unsafe {
        device
            .raw
            .device
            .CreateTexture2D(&texture_desc, None, Some(&mut texture))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;
    let texture = texture.ok_or_else(|| {
        GraphicsError::Backend("CreateTexture2D (depth) não retornou uma textura".into())
    })?;

    let mut view: Option<ID3D11DepthStencilView> = None;
    unsafe {
        device
            .raw
            .device
            .CreateDepthStencilView(&texture, None, Some(&mut view))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;
    let view = view.ok_or_else(|| {
        GraphicsError::Backend("CreateDepthStencilView não retornou uma view".into())
    })?;

    Ok(crate::view::DepthStencilView::from_raw(
        view, format, width, height,
    ))
}

pub(crate) fn create_texture_2d(
    device: &Device,
    desc: &crate::texture::TextureDescriptor,
    mip_data: &[&[u8]],
) -> Result<crate::texture::Texture2D, GraphicsError> {
    use crate::texture::{mip_dimensions, mip_subresource_index};

    let format = desc
        .format
        .to_dxgi_format()
        .ok_or(GraphicsError::Unsupported(
            "formato não suportado para textura",
        ))?;
    let bytes_per_pixel = desc.format.bits_per_pixel() / 8;
    let generated_chain = desc.mip_levels == 0;

    let texture_desc = D3D11_TEXTURE2D_DESC {
        Width: desc.width,
        Height: desc.height,
        MipLevels: desc.mip_levels,
        ArraySize: desc.array_layers,
        Format: format,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        // GenerateMips exige o recurso também como render target (ver
        // Context::generate_mips) além de shader resource.
        BindFlags: if generated_chain {
            (D3D11_BIND_SHADER_RESOURCE.0 | D3D11_BIND_RENDER_TARGET.0) as u32
        } else {
            D3D11_BIND_SHADER_RESOURCE.0 as u32
        },
        CPUAccessFlags: 0,
        MiscFlags: if generated_chain {
            D3D11_RESOURCE_MISC_GENERATE_MIPS.0 as u32
        } else {
            0
        },
    };

    let mut texture: Option<ID3D11Texture2D> = None;
    unsafe {
        if generated_chain {
            // Drivers não aceitam inicializar todos os subresources de um
            // recurso GENERATE_MIPS na criação — cria vazio e sobe o nível
            // base de cada camada depois, via UpdateSubresource.
            device
                .raw
                .device
                .CreateTexture2D(&texture_desc, None, Some(&mut texture))
        } else {
            let initial_data: Vec<D3D11_SUBRESOURCE_DATA> = (0..desc.array_layers)
                .flat_map(|layer| {
                    (0..desc.mip_levels).map(move |mip| (mip, layer))
                })
                .map(|(mip, layer)| {
                    let index = mip_subresource_index(mip, layer, desc.mip_levels) as usize;
                    let (mip_w, _) = mip_dimensions(desc.width, desc.height, mip);
                    D3D11_SUBRESOURCE_DATA {
                        pSysMem: mip_data[index].as_ptr() as *const _,
                        SysMemPitch: mip_w * bytes_per_pixel,
                        SysMemSlicePitch: 0,
                    }
                })
                .collect();
            device.raw.device.CreateTexture2D(
                &texture_desc,
                Some(initial_data.as_ptr()),
                Some(&mut texture),
            )
        }
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let texture = texture
        .ok_or_else(|| GraphicsError::Backend("CreateTexture2D não retornou uma textura".into()))?;

    let mip_levels = if generated_chain {
        let mut created_desc = D3D11_TEXTURE2D_DESC::default();
        unsafe {
            texture.GetDesc(&mut created_desc);
        }
        unsafe {
            for layer in 0..desc.array_layers {
                let subresource =
                    mip_subresource_index(0, layer, created_desc.MipLevels);
                device.raw.context.UpdateSubresource(
                    &texture,
                    subresource,
                    None,
                    mip_data[layer as usize].as_ptr() as *const _,
                    desc.width * bytes_per_pixel,
                    0,
                );
            }
        }
        created_desc.MipLevels
    } else {
        desc.mip_levels
    };

    Ok(crate::texture::Texture2D::from_raw(
        texture,
        desc.width,
        desc.height,
        desc.format,
        mip_levels,
        desc.array_layers,
    ))
}

pub(crate) fn create_render_target_texture(
    device: &Device,
    desc: &crate::texture::TextureDescriptor,
) -> Result<(crate::texture::Texture2D, crate::view::RenderTargetView), GraphicsError> {
    let format = desc
        .format
        .to_dxgi_format()
        .ok_or(GraphicsError::Unsupported(
            "formato não suportado para textura",
        ))?;

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
        // Também SHADER_RESOURCE: um alvo de renderização em geral precisa
        // ser lido depois (post-processing, deferred shading), não só
        // escrito — ver create_shader_resource_view.
        BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
        CPUAccessFlags: 0,
        MiscFlags: 0,
    };

    let mut texture: Option<ID3D11Texture2D> = None;
    unsafe {
        device
            .raw
            .device
            .CreateTexture2D(&texture_desc, None, Some(&mut texture))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;
    let texture = texture
        .ok_or_else(|| GraphicsError::Backend("CreateTexture2D não retornou uma textura".into()))?;

    let mut render_target_view: Option<ID3D11RenderTargetView> = None;
    unsafe {
        device.raw.device.CreateRenderTargetView(
            &texture,
            None,
            Some(&mut render_target_view),
        )
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;
    let render_target_view = render_target_view.ok_or_else(|| {
        GraphicsError::Backend("CreateRenderTargetView não retornou uma view".into())
    })?;

    Ok((
        crate::texture::Texture2D::from_raw(
            texture.clone(),
            desc.width,
            desc.height,
            desc.format,
            1,
            1,
        ),
        crate::view::RenderTargetView::from_raw(
            render_target_view,
            desc.format,
            desc.width,
            desc.height,
        ),
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

    let sampler = sampler.ok_or_else(|| {
        GraphicsError::Backend("CreateSamplerState não retornou um sampler".into())
    })?;

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
    let factory: IDXGIFactory2 =
        unsafe { adapter.GetParent() }.map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let format = desc
        .format
        .to_dxgi_format()
        .ok_or(GraphicsError::Unsupported(
            "formato não suportado para swapchain",
        ))?;

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
    unsafe {
        raw.swapchain.ResizeBuffers(
            0,
            width,
            height,
            DXGI_FORMAT_UNKNOWN,
            DXGI_SWAP_CHAIN_FLAG(0),
        )
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))
}

/// Cria a [`crate::view::RenderTargetView`] do back buffer atual da
/// swapchain — precisa ser recriada após `resize_swapchain`, já que o
/// buffer físico por trás dela muda.
pub(crate) fn create_swapchain_render_target_view(
    device: &Device,
    raw: &RawSwapchain,
    size: crate::surface::Extent2D,
    format: crate::format::Format,
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

    Ok(crate::view::RenderTargetView::from_raw(
        render_target_view,
        format,
        size.width,
        size.height,
    ))
}

/// Apresenta o quadro atual da swapchain.
/// Lê de volta o back buffer atual da swapchain, como pixels crus e
/// compactados (sem o padding de linha que a GPU usa internamente) —
/// usado para validar de verdade o que foi desenhado (ver
/// `crate::surface::Swapchain::capture_frame`), em vez de só confiar que
/// a chamada não quebrou.
pub(crate) fn capture_swapchain_frame(
    device: &Device,
    raw: &RawSwapchain,
) -> Result<(u32, u32, crate::format::Format, Vec<u8>), GraphicsError> {
    let back_buffer: ID3D11Texture2D = unsafe { raw.swapchain.GetBuffer(0) }
        .map_err(|err| GraphicsError::Backend(err.to_string()))?;
    capture_texture_2d_raw(device, &back_buffer)
}

/// Lê de volta o conteúdo atual de `texture`, como pixels crus e
/// compactados — o mesmo caminho de `capture_swapchain_frame`, mas para
/// qualquer textura (ex: um attachment de cor extra de MRT), não só o
/// back buffer de uma swapchain. Ver
/// `crate::core::Device::capture_texture`.
pub(crate) fn capture_texture(
    device: &Device,
    texture: &crate::texture::Texture2D,
) -> Result<(u32, u32, crate::format::Format, Vec<u8>), GraphicsError> {
    capture_texture_2d_raw(device, &texture.raw)
}

fn capture_texture_2d_raw(
    device: &Device,
    source: &ID3D11Texture2D,
) -> Result<(u32, u32, crate::format::Format, Vec<u8>), GraphicsError> {
    use windows::Win32::Graphics::Direct3D11::D3D11_MAP_READ;

    let mut desc = D3D11_TEXTURE2D_DESC::default();
    unsafe {
        source.GetDesc(&mut desc);
    }

    let staging_desc = D3D11_TEXTURE2D_DESC {
        Width: desc.Width,
        Height: desc.Height,
        MipLevels: 1,
        ArraySize: 1,
        Format: desc.Format,
        SampleDesc: desc.SampleDesc,
        Usage: windows::Win32::Graphics::Direct3D11::D3D11_USAGE_STAGING,
        BindFlags: 0,
        CPUAccessFlags: windows::Win32::Graphics::Direct3D11::D3D11_CPU_ACCESS_READ.0 as u32,
        MiscFlags: 0,
    };

    let mut staging: Option<ID3D11Texture2D> = None;
    unsafe {
        device
            .raw
            .device
            .CreateTexture2D(&staging_desc, None, Some(&mut staging))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;
    let staging = staging.ok_or_else(|| {
        GraphicsError::Backend(
            "CreateTexture2D (staging de captura) não retornou uma textura".into(),
        )
    })?;

    unsafe {
        device.raw.context.CopyResource(&staging, source);
    }

    let format = crate::format::Format::from_dxgi_format(desc.Format).ok_or(
        GraphicsError::Unsupported("formato da textura sem mapeamento conhecido"),
    )?;
    let bytes_per_pixel = format.bits_per_pixel() / 8;

    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    unsafe {
        device
            .raw
            .context
            .Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
    }
    .map_err(|err| GraphicsError::Backend(err.to_string()))?;

    let mut pixels = vec![0u8; (desc.Width * desc.Height * bytes_per_pixel) as usize];
    unsafe {
        for row in 0..desc.Height {
            let src = (mapped.pData as *const u8).add((row * mapped.RowPitch) as usize);
            let dst_offset = (row * desc.Width * bytes_per_pixel) as usize;
            let row_bytes = (desc.Width * bytes_per_pixel) as usize;
            std::ptr::copy_nonoverlapping(src, pixels.as_mut_ptr().add(dst_offset), row_bytes);
        }
        device.raw.context.Unmap(&staging, 0);
    }

    Ok((desc.Width, desc.Height, format, pixels))
}

pub(crate) fn present(
    raw: &RawSwapchain,
    present_mode: PresentMode,
) -> Result<PresentResult, GraphicsError> {
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

// =======================================================================
// impl Backend for DirectX11 — o contrato único que o resto da crate usa
// (ver `crate::backend::Backend`). Os métodos abaixo ou delegam direto
// pras funções livres acima (criação de recursos — já existiam antes
// desta rodada, lógica inalterada), ou contêm a lógica que antes ficava
// inline em `context`/`compute`/`transfer`/`texture`/`query`, movida pra
// cá sem mudar o que cada chamada D3D11 faz.
// =======================================================================

use crate::blend::{BlendState, BlendStateDescriptor};
use crate::buffer::Buffer;
use crate::command::{CommandBuffer, CommandEncoder};
use crate::context::Context;
use crate::core::FormatFeatures;
use crate::depth_stencil::{DepthStencilState, DepthStencilStateDescriptor};
use crate::input::IndexFormat;
use crate::pipeline::{ComputePipeline, GraphicsPipeline};
use crate::query::{Query, QueryType};
use crate::raster::{RasterizerState, RasterizerStateDescriptor, Viewport};
use crate::sampler::{Sampler, SamplerDescriptor};
use crate::shader::{ComputeShader, PixelShader, VertexShader};
use crate::texture::TextureDescriptor;
use crate::view::{DepthStencilView, RenderTargetView, ShaderResourceView, UnorderedAccessView};

/// Marcador de tipo do backend Direct3D 11 — sem instância (nunca
/// construído), só o que `impl Backend for DirectX11` precisa pra existir.
pub(crate) enum DirectX11 {}

fn device_context_raw(raw: &RawDevice) -> ID3D11DeviceContext {
    raw.context.clone()
}

fn draw_raw(context: &Context, vertex_count: u32, first_vertex: u32) {
    unsafe {
        context.raw.Draw(vertex_count, first_vertex);
    }
}

fn draw_instanced_raw(
    context: &Context,
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
) {
    unsafe {
        context
            .raw
            .DrawInstanced(vertex_count, instance_count, first_vertex, first_instance);
    }
}

fn draw_indexed_raw(context: &Context, index_count: u32, first_index: u32, base_vertex: i32) {
    unsafe {
        context.raw.DrawIndexed(index_count, first_index, base_vertex);
    }
}

fn draw_indexed_instanced_raw(
    context: &Context,
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
) {
    unsafe {
        context.raw.DrawIndexedInstanced(
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
        );
    }
}

fn draw_indirect_raw(context: &Context, args_buffer: &Buffer, args_offset: u64) {
    unsafe {
        context
            .raw
            .DrawInstancedIndirect(&args_buffer.raw, args_offset as u32);
    }
}

fn draw_indexed_indirect_raw(context: &Context, args_buffer: &Buffer, args_offset: u64) {
    unsafe {
        context
            .raw
            .DrawIndexedInstancedIndirect(&args_buffer.raw, args_offset as u32);
    }
}

fn set_render_target_raw(context: &Context, view: &RenderTargetView) {
    unsafe {
        context
            .raw
            .OMSetRenderTargets(Some(&[Some(view.raw.clone())]), None);
    }
}

fn set_render_target_with_depth_raw(
    context: &Context,
    view: &RenderTargetView,
    depth: &DepthStencilView,
) {
    unsafe {
        context
            .raw
            .OMSetRenderTargets(Some(&[Some(view.raw.clone())]), Some(&depth.raw));
    }
}

fn set_render_targets_raw(context: &Context, views: &[&RenderTargetView]) {
    unsafe {
        let raw_views: Vec<Option<ID3D11RenderTargetView>> =
            views.iter().map(|view| Some(view.raw.clone())).collect();
        context.raw.OMSetRenderTargets(Some(&raw_views), None);
    }
}

fn set_render_targets_with_depth_raw(
    context: &Context,
    views: &[&RenderTargetView],
    depth: &DepthStencilView,
) {
    unsafe {
        let raw_views: Vec<Option<ID3D11RenderTargetView>> =
            views.iter().map(|view| Some(view.raw.clone())).collect();
        context
            .raw
            .OMSetRenderTargets(Some(&raw_views), Some(&depth.raw));
    }
}

fn clear_render_target_raw(context: &Context, view: &RenderTargetView, color: [f32; 4]) {
    unsafe {
        context.raw.ClearRenderTargetView(&view.raw, &color);
    }
}

fn clear_depth_raw(context: &Context, view: &DepthStencilView, depth: f32) {
    unsafe {
        context.raw.ClearDepthStencilView(
            &view.raw,
            windows::Win32::Graphics::Direct3D11::D3D11_CLEAR_DEPTH.0 as u32,
            depth,
            0,
        );
    }
}

fn flush_raw(context: &Context) {
    unsafe {
        context.raw.Flush();
    }
}

fn set_viewport_raw(context: &Context, viewport: Viewport) {
    use windows::Win32::Graphics::Direct3D11::D3D11_VIEWPORT;
    unsafe {
        context.raw.RSSetViewports(Some(&[D3D11_VIEWPORT {
            TopLeftX: viewport.x,
            TopLeftY: viewport.y,
            Width: viewport.width,
            Height: viewport.height,
            MinDepth: viewport.min_depth,
            MaxDepth: viewport.max_depth,
        }]));
    }
}

fn topology_to_d3d(
    topology: crate::input::PrimitiveTopology,
) -> windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY {
    use windows::Win32::Graphics::Direct3D::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST;
    match topology {
        crate::input::PrimitiveTopology::TriangleList => D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
    }
}

fn set_pipeline_raw(context: &Context, pipeline: &GraphicsPipeline) {
    unsafe {
        context.raw.RSSetState(&pipeline.rasterizer_state);
        context.raw.OMSetBlendState(
            &pipeline.blend_state,
            Some(&[1.0, 1.0, 1.0, 1.0]),
            0xffffffff,
        );
        context
            .raw
            .OMSetDepthStencilState(&pipeline.depth_stencil_state, pipeline.stencil_reference);
        context.raw.IASetInputLayout(&pipeline.input_layout);
        context.raw.VSSetShader(&pipeline.vertex_shader, None);
        context.raw.PSSetShader(&pipeline.pixel_shader, None);
        context
            .raw
            .IASetPrimitiveTopology(topology_to_d3d(pipeline.primitive_topology));
    }
}

fn set_vertex_buffer_raw(context: &Context, slot: u32, buffer: &Buffer, stride: u32) {
    unsafe {
        context.raw.IASetVertexBuffers(
            slot,
            1,
            Some(&Some(buffer.raw.clone())),
            Some(&stride),
            Some(&0),
        );
    }
}

fn set_index_buffer_raw(context: &Context, buffer: &Buffer, format: IndexFormat) {
    use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R16_UINT, DXGI_FORMAT_R32_UINT};
    let dxgi_format = match format {
        IndexFormat::Uint16 => DXGI_FORMAT_R16_UINT,
        IndexFormat::Uint32 => DXGI_FORMAT_R32_UINT,
    };
    unsafe {
        context.raw.IASetIndexBuffer(&buffer.raw, dxgi_format, 0);
    }
}

fn set_constant_buffer_raw(context: &Context, slot: u32, buffer: &Buffer) {
    unsafe {
        let slice = Some(&[Some(buffer.raw.clone())][..]);
        context.raw.VSSetConstantBuffers(slot, slice);
        context.raw.PSSetConstantBuffers(slot, slice);
    }
}

fn set_shader_resource_raw(context: &Context, slot: u32, view: &ShaderResourceView) {
    unsafe {
        context
            .raw
            .PSSetShaderResources(slot, Some(&[Some(view.raw.clone())]));
    }
}

fn set_sampler_raw(context: &Context, slot: u32, sampler: &Sampler) {
    unsafe {
        context
            .raw
            .PSSetSamplers(slot, Some(&[Some(sampler.raw.clone())]));
    }
}

fn set_blend_state_raw(context: &Context, state: &BlendState) {
    unsafe {
        context
            .raw
            .OMSetBlendState(&state.raw, Some(&[1.0, 1.0, 1.0, 1.0]), 0xffffffff);
    }
}

fn set_depth_stencil_state_raw(context: &Context, state: &DepthStencilState) {
    unsafe {
        context.raw.OMSetDepthStencilState(&state.raw, 0);
    }
}

fn update_buffer_raw(context: &Context, buffer: &Buffer, data: &[u8]) {
    unsafe {
        context
            .raw
            .UpdateSubresource(&buffer.raw, 0, None, data.as_ptr() as *const _, 0, 0);
    }
}

fn set_compute_pipeline_raw(context: &Context, pipeline: &ComputePipeline) {
    unsafe {
        context.raw.CSSetShader(&pipeline.compute_shader, None);
    }
}

fn set_compute_constant_buffer_raw(context: &Context, slot: u32, buffer: &Buffer) {
    unsafe {
        context
            .raw
            .CSSetConstantBuffers(slot, Some(&[Some(buffer.raw.clone())]));
    }
}

fn set_unordered_access_view_raw(context: &Context, slot: u32, view: &UnorderedAccessView) {
    unsafe {
        context
            .raw
            .CSSetUnorderedAccessViews(slot, 1, Some(&Some(view.raw.clone())), None);
    }
}

fn dispatch_raw(context: &Context, x: u32, y: u32, z: u32) {
    unsafe {
        context.raw.Dispatch(x, y, z);
    }
}

fn dispatch_indirect_raw(context: &Context, args_buffer: &Buffer, args_offset: u64) {
    unsafe {
        context
            .raw
            .DispatchIndirect(&args_buffer.raw, args_offset as u32);
    }
}

fn copy_buffer_raw(context: &Context, src: &Buffer, dst: &Buffer) {
    unsafe {
        context.raw.CopyResource(&dst.raw, &src.raw);
    }
}

fn copy_texture_raw(
    context: &Context,
    src: &crate::texture::Texture2D,
    dst: &crate::texture::Texture2D,
) {
    unsafe {
        context.raw.CopyResource(&dst.raw, &src.raw);
    }
}

fn generate_mips_raw(context: &Context, view: &ShaderResourceView) {
    unsafe {
        context.raw.GenerateMips(&view.raw);
    }
}

fn begin_query_raw(context: &Context, query: &Query) {
    unsafe {
        context.raw.Begin(&query.raw);
    }
}

fn end_query_raw(context: &Context, query: &Query) {
    unsafe {
        context.raw.End(&query.raw);
    }
}

impl crate::backend::Backend for DirectX11 {
    type RawDevice = RawDevice;
    type RawContext = ID3D11DeviceContext;
    type RawCommandBuffer = ID3D11CommandList;
    type RawBuffer = ID3D11Buffer;
    type RawTexture2D = ID3D11Texture2D;
    type RawRenderTargetView = ID3D11RenderTargetView;
    type RawShaderResourceView = ID3D11ShaderResourceView;
    type RawDepthStencilView = ID3D11DepthStencilView;
    type RawUnorderedAccessView = windows::Win32::Graphics::Direct3D11::ID3D11UnorderedAccessView;
    type RawSampler = ID3D11SamplerState;
    type RawInputLayout = ID3D11InputLayout;
    type RawVertexShader = ID3D11VertexShader;
    type RawPixelShader = ID3D11PixelShader;
    type RawComputeShader = windows::Win32::Graphics::Direct3D11::ID3D11ComputeShader;
    type RawRasterizerState = ID3D11RasterizerState;
    type RawBlendState = windows::Win32::Graphics::Direct3D11::ID3D11BlendState;
    type RawDepthStencilState = ID3D11DepthStencilState;
    type RawQuery = ID3D11Query;
    type RawSurface = windows::Win32::Foundation::HWND;
    type RawSwapchain = RawSwapchain;

    fn enumerate_adapters() -> Result<Vec<AdapterInfo>, GraphicsError> {
        enumerate_adapters()
    }
    fn create_default_device(
        desc: &DeviceDescriptor,
    ) -> Result<(Self::RawDevice, bool), GraphicsError> {
        create_default_device(desc)
    }
    fn device_properties(raw: &Self::RawDevice) -> (DeviceCapabilities, DeviceLimits, LimitsSource) {
        device_properties(raw)
    }
    fn adapter_info_for_device(raw: &Self::RawDevice) -> Result<AdapterInfo, GraphicsError> {
        adapter_info_for_device(raw)
    }
    fn is_device_lost(raw: &Self::RawDevice) -> bool {
        is_device_lost(raw)
    }
    fn query_format_features(
        device: &Device,
        format: crate::format::Format,
    ) -> Result<FormatFeatures, GraphicsError> {
        query_format_features(device, format)
    }
    fn device_context(raw: &Self::RawDevice) -> Self::RawContext {
        device_context_raw(raw)
    }

    fn create_buffer(
        device: &Device,
        desc: &crate::buffer::BufferDescriptor,
        mip_data: Option<&[u8]>,
    ) -> Result<Buffer, GraphicsError> {
        create_buffer(device, desc, mip_data)
    }
    fn create_unordered_access_view(
        device: &Device,
        buffer: &Buffer,
    ) -> Result<UnorderedAccessView, GraphicsError> {
        create_unordered_access_view(device, buffer)
    }
    fn map_write_discard<'a>(
        context: &'a Context,
        buffer: &'a Buffer,
    ) -> Result<crate::memory::MappedBuffer<'a>, GraphicsError> {
        map_write_discard(context, buffer)
    }
    fn map_read<'a>(
        context: &'a Context,
        buffer: &'a Buffer,
    ) -> Result<crate::memory::ReadMappedBuffer<'a>, GraphicsError> {
        map_read(context, buffer)
    }
    fn unmap_buffer(context: &Context, buffer: &Buffer) {
        unmap_buffer(context, buffer)
    }

    fn create_texture_2d(
        device: &Device,
        desc: &TextureDescriptor,
        mip_data: &[&[u8]],
    ) -> Result<crate::texture::Texture2D, GraphicsError> {
        create_texture_2d(device, desc, mip_data)
    }
    fn create_render_target_texture(
        device: &Device,
        desc: &TextureDescriptor,
    ) -> Result<(crate::texture::Texture2D, RenderTargetView), GraphicsError> {
        create_render_target_texture(device, desc)
    }
    fn create_shader_resource_view(
        device: &Device,
        texture: &crate::texture::Texture2D,
    ) -> Result<ShaderResourceView, GraphicsError> {
        create_shader_resource_view(device, texture)
    }
    fn create_depth_stencil_view(
        device: &Device,
        width: u32,
        height: u32,
        format: crate::format::Format,
    ) -> Result<DepthStencilView, GraphicsError> {
        create_depth_stencil_view(device, width, height, format)
    }
    fn generate_mips(context: &Context, view: &ShaderResourceView) {
        generate_mips_raw(context, view)
    }
    fn capture_texture(
        device: &Device,
        texture: &crate::texture::Texture2D,
    ) -> Result<(u32, u32, crate::format::Format, Vec<u8>), GraphicsError> {
        capture_texture(device, texture)
    }

    fn compile_hlsl(
        source: &str,
        entry_point: &str,
        target: &str,
    ) -> Result<Vec<u8>, GraphicsError> {
        compile_hlsl(source, entry_point, target)
    }
    fn create_vertex_shader(device: &Device, bytecode: &[u8]) -> Result<VertexShader, GraphicsError> {
        create_vertex_shader(device, bytecode)
    }
    fn create_pixel_shader(device: &Device, bytecode: &[u8]) -> Result<PixelShader, GraphicsError> {
        create_pixel_shader(device, bytecode)
    }
    fn create_compute_shader(
        device: &Device,
        bytecode: &[u8],
    ) -> Result<ComputeShader, GraphicsError> {
        create_compute_shader(device, bytecode)
    }
    fn create_input_layout(
        device: &Device,
        attributes: &[crate::input::VertexAttribute],
        vertex_shader: &VertexShader,
    ) -> Result<crate::input::InputLayout, GraphicsError> {
        create_input_layout(device, attributes, vertex_shader)
    }

    fn create_rasterizer_state(
        device: &Device,
        desc: &RasterizerStateDescriptor,
    ) -> Result<RasterizerState, GraphicsError> {
        create_rasterizer_state(device, desc)
    }
    fn create_blend_state(
        device: &Device,
        desc: &BlendStateDescriptor,
    ) -> Result<BlendState, GraphicsError> {
        create_blend_state(device, desc)
    }
    fn create_depth_stencil_state(
        device: &Device,
        desc: &DepthStencilStateDescriptor,
    ) -> Result<DepthStencilState, GraphicsError> {
        create_depth_stencil_state(device, desc)
    }

    fn set_render_target(context: &Context, view: &RenderTargetView) {
        set_render_target_raw(context, view)
    }
    fn set_render_target_with_depth(
        context: &Context,
        view: &RenderTargetView,
        depth: &DepthStencilView,
    ) {
        set_render_target_with_depth_raw(context, view, depth)
    }
    fn set_render_targets(context: &Context, views: &[&RenderTargetView]) {
        set_render_targets_raw(context, views)
    }
    fn set_render_targets_with_depth(
        context: &Context,
        views: &[&RenderTargetView],
        depth: &DepthStencilView,
    ) {
        set_render_targets_with_depth_raw(context, views, depth)
    }
    fn clear_render_target(context: &Context, view: &RenderTargetView, color: [f32; 4]) {
        clear_render_target_raw(context, view, color)
    }
    fn clear_depth(context: &Context, view: &DepthStencilView, depth: f32) {
        clear_depth_raw(context, view, depth)
    }
    fn flush(context: &Context) {
        flush_raw(context)
    }
    fn set_viewport(context: &Context, viewport: Viewport) {
        set_viewport_raw(context, viewport)
    }
    fn set_pipeline(context: &Context, pipeline: &GraphicsPipeline) {
        set_pipeline_raw(context, pipeline)
    }
    fn set_vertex_buffer(context: &Context, slot: u32, buffer: &Buffer, stride: u32) {
        set_vertex_buffer_raw(context, slot, buffer, stride)
    }
    fn set_index_buffer(context: &Context, buffer: &Buffer, format: IndexFormat) {
        set_index_buffer_raw(context, buffer, format)
    }
    fn set_constant_buffer(context: &Context, slot: u32, buffer: &Buffer) {
        set_constant_buffer_raw(context, slot, buffer)
    }
    fn set_shader_resource(context: &Context, slot: u32, view: &ShaderResourceView) {
        set_shader_resource_raw(context, slot, view)
    }
    fn set_sampler(context: &Context, slot: u32, sampler: &Sampler) {
        set_sampler_raw(context, slot, sampler)
    }
    fn set_blend_state(context: &Context, state: &BlendState) {
        set_blend_state_raw(context, state)
    }
    fn set_depth_stencil_state(context: &Context, state: &DepthStencilState) {
        set_depth_stencil_state_raw(context, state)
    }
    fn update_buffer(context: &Context, buffer: &Buffer, data: &[u8]) {
        update_buffer_raw(context, buffer, data)
    }
    fn draw(context: &Context, vertex_count: u32, first_vertex: u32) {
        draw_raw(context, vertex_count, first_vertex)
    }
    fn draw_instanced(
        context: &Context,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        draw_instanced_raw(context, vertex_count, instance_count, first_vertex, first_instance)
    }
    fn draw_indexed(context: &Context, index_count: u32, first_index: u32, base_vertex: i32) {
        draw_indexed_raw(context, index_count, first_index, base_vertex)
    }
    fn draw_indexed_instanced(
        context: &Context,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        base_vertex: i32,
        first_instance: u32,
    ) {
        draw_indexed_instanced_raw(
            context,
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
        )
    }
    fn draw_indirect(context: &Context, args_buffer: &Buffer, args_offset: u64) {
        draw_indirect_raw(context, args_buffer, args_offset)
    }
    fn draw_indexed_indirect(context: &Context, args_buffer: &Buffer, args_offset: u64) {
        draw_indexed_indirect_raw(context, args_buffer, args_offset)
    }

    fn set_compute_pipeline(context: &Context, pipeline: &ComputePipeline) {
        set_compute_pipeline_raw(context, pipeline)
    }
    fn set_compute_constant_buffer(context: &Context, slot: u32, buffer: &Buffer) {
        set_compute_constant_buffer_raw(context, slot, buffer)
    }
    fn set_unordered_access_view(context: &Context, slot: u32, view: &UnorderedAccessView) {
        set_unordered_access_view_raw(context, slot, view)
    }
    fn dispatch(context: &Context, x: u32, y: u32, z: u32) {
        dispatch_raw(context, x, y, z)
    }
    fn dispatch_indirect(context: &Context, args_buffer: &Buffer, args_offset: u64) {
        dispatch_indirect_raw(context, args_buffer, args_offset)
    }

    fn copy_buffer(context: &Context, src: &Buffer, dst: &Buffer) {
        copy_buffer_raw(context, src, dst)
    }
    fn copy_texture(
        context: &Context,
        src: &crate::texture::Texture2D,
        dst: &crate::texture::Texture2D,
    ) {
        copy_texture_raw(context, src, dst)
    }

    fn create_command_encoder(device: &Device) -> Result<CommandEncoder, GraphicsError> {
        create_command_encoder(device)
    }
    fn finish_command_list(context: &Context) -> Result<CommandBuffer, GraphicsError> {
        finish_command_list(context)
    }
    fn execute_command_list(context: &Context, buffer: &CommandBuffer) {
        execute_command_list(context, buffer)
    }
    fn wait_idle(context: &Context) -> Result<(), GraphicsError> {
        wait_idle(context)
    }

    fn create_query(device: &Device, kind: QueryType) -> Result<Query, GraphicsError> {
        create_query(device, kind)
    }
    fn begin_query(context: &Context, query: &Query) {
        begin_query_raw(context, query)
    }
    fn end_query(context: &Context, query: &Query) {
        end_query_raw(context, query)
    }
    fn is_query_ready(context: &Context, query: &Query) -> bool {
        is_query_ready(context, query)
    }
    fn get_occlusion_result(context: &Context, query: &Query) -> Option<u64> {
        get_occlusion_result(context, query)
    }

    fn push_debug_group(context: &Context, label: &str) {
        push_debug_group(context, label)
    }
    fn pop_debug_group(context: &Context) {
        pop_debug_group(context)
    }
    fn insert_debug_marker(context: &Context, label: &str) {
        insert_debug_marker(context, label)
    }
    fn set_buffer_label(buffer: &Buffer, label: &str) {
        set_resource_label(&buffer.raw, label)
    }
    fn set_texture_label(texture: &crate::texture::Texture2D, label: &str) {
        set_resource_label(&texture.raw, label)
    }

    fn create_sampler(device: &Device, desc: &SamplerDescriptor) -> Result<Sampler, GraphicsError> {
        create_sampler(device, desc)
    }

    fn surface_from_handle(
        handle: raw_window_handle::RawWindowHandle,
    ) -> Result<Surface, GraphicsError> {
        surface_from_handle(handle)
    }
    fn create_swapchain(
        device: &Device,
        surface: &Surface,
        desc: &SwapchainDescriptor,
    ) -> Result<Self::RawSwapchain, GraphicsError> {
        create_swapchain(device, surface, desc)
    }
    fn resize_swapchain(
        raw: &mut Self::RawSwapchain,
        width: u32,
        height: u32,
    ) -> Result<(), GraphicsError> {
        resize_swapchain(raw, width, height)
    }
    fn create_swapchain_render_target_view(
        device: &Device,
        raw: &Self::RawSwapchain,
        size: crate::surface::Extent2D,
        format: crate::format::Format,
    ) -> Result<RenderTargetView, GraphicsError> {
        create_swapchain_render_target_view(device, raw, size, format)
    }
    fn capture_swapchain_frame(
        device: &Device,
        raw: &Self::RawSwapchain,
    ) -> Result<(u32, u32, crate::format::Format, Vec<u8>), GraphicsError> {
        capture_swapchain_frame(device, raw)
    }
    fn present(
        raw: &Self::RawSwapchain,
        present_mode: crate::surface::PresentMode,
    ) -> Result<PresentResult, GraphicsError> {
        present(raw, present_mode)
    }
}
