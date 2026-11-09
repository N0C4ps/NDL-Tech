//! Implementações específicas de cada backend gráfico.
//!
//! Este módulo é **inteiramente privado**: nada aqui é exposto na API
//! pública da crate, do mesmo jeito que `ndl-platform::backend`. O resto
//! da crate (`core`, `buffer`, `context`, `pipeline`, ...) programa
//! inteiramente contra o contrato [`Backend`] definido aqui — nunca
//! importa um tipo nativo (`ID3D11Buffer`, `IDXGISwapChain1`, ...) nem
//! chama um backend concreto por nome. Isso é o que torna um backend
//! novo (D3D12, Vulkan, ...) um trabalho isolado em `backend/<nome>/`:
//! nenhum outro módulo precisa mudar.
//!
//! ## O contrato [`Backend`]
//!
//! Um único trait com tipos associados (`type RawBuffer`, `type RawContext`,
//! ...) para cada recurso nativo guardado dentro de um tipo público
//! (`Buffer::raw`, `Context::raw`, ...), e um método por operação que hoje
//! vira uma chamada nativa (`create_buffer`, `draw`, `set_pipeline`, ...).
//! O backend ativo é escolhido **em tempo de compilação**, via feature do
//! Cargo (`backend-directx11` hoje; `backend-directx12`/`backend-vulkan`
//! reservados) — isso é o que permite `ActiveBackend` abaixo resolver pra
//! um tipo concreto sem nenhum `dyn`/vtable: tudo monomorphiza, sem custo
//! de indireção numa chamada de draw.
//!
//! Os tipos públicos (`Buffer`, `Context`, `Texture2D`, ...) continuam
//! structs comuns, não genéricas — só o *campo* `raw` interno deles é
//! `<ActiveBackend as Backend>::RawX`, resolvido em tempo de compilação
//! para o tipo nativo do backend escolhido.
//!
//! ## Backends disponíveis
//!
//! Hoje só `directx` (Direct3D 11, via `windows-rs`) tem implementação
//! real, atrás da feature `backend-directx11` — a única habilitada por
//! padrão. Reservado para `d3d12`/`vulkan`/`metal` depois, seguindo o
//! mesmo padrão de `ndl-platform` (um backend por plataforma/API, atrás
//! de uma única API pública consistente).

#[cfg(feature = "backend-directx11")]
pub(crate) mod directx;

#[cfg(all(feature = "backend-directx11", target_os = "windows"))]
pub(crate) type ActiveBackend = directx::DirectX11;

#[cfg(not(all(feature = "backend-directx11", target_os = "windows")))]
compile_error!(
    "nenhum backend gráfico ativo: habilite a feature \"backend-directx11\" (hoje, o único \
     backend real, disponível só em builds Windows) — ver crates/ndl-render/Cargo.toml"
);

use crate::blend::{BlendState, BlendStateDescriptor};
use crate::buffer::{Buffer, BufferDescriptor};
use crate::command::{CommandBuffer, CommandEncoder};
use crate::context::Context;
use crate::core::{
    AdapterInfo, Device, DeviceCapabilities, DeviceDescriptor, DeviceLimits, FormatFeatures,
    GraphicsError, LimitsSource,
};
use crate::depth_stencil::{DepthStencilState, DepthStencilStateDescriptor};
use crate::format::Format;
use crate::input::{IndexFormat, VertexAttribute};
use crate::pipeline::{ComputePipeline, GraphicsPipeline};
use crate::query::{Query, QueryType};
use crate::raster::{RasterizerState, RasterizerStateDescriptor, Viewport};
use crate::sampler::{Sampler, SamplerDescriptor};
use crate::shader::{ComputeShader, PixelShader, VertexShader};
use crate::surface::{Extent2D, PresentMode, PresentResult, Surface, SwapchainDescriptor};
use crate::texture::{Texture2D, TextureDescriptor};
use crate::view::{DepthStencilView, RenderTargetView, ShaderResourceView, UnorderedAccessView};

/// Contrato interno que todo backend gráfico deve implementar — ver o
/// `//!` deste módulo. Puramente associativo (sem `&self`): a própria
/// crate escolhe `ActiveBackend` em tempo de compilação, então um backend
/// é só um marcador de tipo (`enum DirectX11 {}`, sem instância).
#[allow(dead_code)] // o resto do crate ainda está sendo migrado pra usar isto
pub(crate) trait Backend: Sized {
    // --- tipos nativos guardados dentro dos tipos públicos ---
    //
    // `Clone` porque os tipos públicos (`GraphicsPipeline`, binds de
    // `Context`, ...) clonam esses handles livremente, do mesmo jeito que
    // já fazem hoje com `ID3D11X::clone()` (ref-count de COM) — isso é o
    // que permite `pipeline.rs` montar `GraphicsPipeline` sem nenhum
    // método de trait próprio, só clonando os `raw` já abstraídos de
    // shaders/estados já criados.
    type RawDevice;
    type RawContext;
    type RawCommandBuffer;
    type RawBuffer: Clone;
    type RawTexture2D: Clone;
    type RawRenderTargetView: Clone;
    type RawShaderResourceView: Clone;
    type RawDepthStencilView: Clone;
    type RawUnorderedAccessView: Clone;
    type RawSampler: Clone;
    type RawInputLayout: Clone;
    type RawVertexShader: Clone;
    type RawPixelShader: Clone;
    type RawComputeShader: Clone;
    type RawRasterizerState: Clone;
    type RawBlendState: Clone;
    type RawDepthStencilState: Clone;
    type RawQuery;
    type RawSurface;
    type RawSwapchain;

    // --- core: instância, adaptadores, dispositivo ---
    fn enumerate_adapters() -> Result<Vec<AdapterInfo>, GraphicsError>;
    fn create_default_device(
        desc: &DeviceDescriptor,
    ) -> Result<(Self::RawDevice, bool), GraphicsError>;
    fn device_properties(
        raw: &Self::RawDevice,
    ) -> (DeviceCapabilities, DeviceLimits, LimitsSource);
    fn adapter_info_for_device(raw: &Self::RawDevice) -> Result<AdapterInfo, GraphicsError>;
    fn is_device_lost(raw: &Self::RawDevice) -> bool;
    fn query_format_features(
        device: &Device,
        format: Format,
    ) -> Result<FormatFeatures, GraphicsError>;
    fn device_context(raw: &Self::RawDevice) -> Self::RawContext;

    // --- buffers ---
    fn create_buffer(
        device: &Device,
        desc: &BufferDescriptor,
        mip_data: Option<&[u8]>,
    ) -> Result<Buffer, GraphicsError>;
    fn create_unordered_access_view(
        device: &Device,
        buffer: &Buffer,
    ) -> Result<UnorderedAccessView, GraphicsError>;
    fn map_write_discard<'a>(
        context: &'a Context,
        buffer: &'a Buffer,
    ) -> Result<crate::memory::MappedBuffer<'a>, GraphicsError>;
    fn map_read<'a>(
        context: &'a Context,
        buffer: &'a Buffer,
    ) -> Result<crate::memory::ReadMappedBuffer<'a>, GraphicsError>;
    fn unmap_buffer(context: &Context, buffer: &Buffer);

    // --- texturas ---
    fn create_texture_2d(
        device: &Device,
        desc: &TextureDescriptor,
        mip_data: &[&[u8]],
    ) -> Result<Texture2D, GraphicsError>;
    fn create_render_target_texture(
        device: &Device,
        desc: &TextureDescriptor,
    ) -> Result<(Texture2D, RenderTargetView), GraphicsError>;
    fn create_shader_resource_view(
        device: &Device,
        texture: &Texture2D,
    ) -> Result<ShaderResourceView, GraphicsError>;
    fn create_depth_stencil_view(
        device: &Device,
        width: u32,
        height: u32,
        format: Format,
    ) -> Result<DepthStencilView, GraphicsError>;
    fn generate_mips(context: &Context, view: &ShaderResourceView);
    fn capture_texture(
        device: &Device,
        texture: &Texture2D,
    ) -> Result<(u32, u32, Format, Vec<u8>), GraphicsError>;

    // --- shaders ---
    fn compile_hlsl(source: &str, entry_point: &str, target: &str) -> Result<Vec<u8>, GraphicsError>;
    fn create_vertex_shader(device: &Device, bytecode: &[u8]) -> Result<VertexShader, GraphicsError>;
    fn create_pixel_shader(device: &Device, bytecode: &[u8]) -> Result<PixelShader, GraphicsError>;
    fn create_compute_shader(
        device: &Device,
        bytecode: &[u8],
    ) -> Result<ComputeShader, GraphicsError>;
    fn create_input_layout(
        device: &Device,
        attributes: &[VertexAttribute],
        vertex_shader: &VertexShader,
    ) -> Result<crate::input::InputLayout, GraphicsError>;

    // --- estados fixos e pipelines ---
    fn create_rasterizer_state(
        device: &Device,
        desc: &RasterizerStateDescriptor,
    ) -> Result<RasterizerState, GraphicsError>;
    fn create_blend_state(
        device: &Device,
        desc: &BlendStateDescriptor,
    ) -> Result<BlendState, GraphicsError>;
    fn create_depth_stencil_state(
        device: &Device,
        desc: &DepthStencilStateDescriptor,
    ) -> Result<DepthStencilState, GraphicsError>;

    // --- contexto: binds, clears, draws ---
    fn set_render_target(context: &Context, view: &RenderTargetView);
    fn set_render_target_with_depth(context: &Context, view: &RenderTargetView, depth: &DepthStencilView);
    fn set_render_targets(context: &Context, views: &[&RenderTargetView]);
    fn set_render_targets_with_depth(
        context: &Context,
        views: &[&RenderTargetView],
        depth: &DepthStencilView,
    );
    fn clear_render_target(context: &Context, view: &RenderTargetView, color: [f32; 4]);
    fn clear_depth(context: &Context, view: &DepthStencilView, depth: f32);
    fn flush(context: &Context);
    fn set_viewport(context: &Context, viewport: Viewport);
    fn set_pipeline(context: &Context, pipeline: &GraphicsPipeline);
    fn set_vertex_buffer(context: &Context, slot: u32, buffer: &Buffer, stride: u32);
    fn set_index_buffer(context: &Context, buffer: &Buffer, format: IndexFormat);
    fn set_constant_buffer(context: &Context, slot: u32, buffer: &Buffer);
    fn set_shader_resource(context: &Context, slot: u32, view: &ShaderResourceView);
    fn set_sampler(context: &Context, slot: u32, sampler: &Sampler);
    fn set_blend_state(context: &Context, state: &BlendState);
    fn set_depth_stencil_state(context: &Context, state: &DepthStencilState);
    fn update_buffer(context: &Context, buffer: &Buffer, data: &[u8]);
    fn draw(context: &Context, vertex_count: u32, first_vertex: u32);
    fn draw_instanced(
        context: &Context,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    );
    fn draw_indexed(context: &Context, index_count: u32, first_index: u32, base_vertex: i32);
    fn draw_indexed_instanced(
        context: &Context,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        base_vertex: i32,
        first_instance: u32,
    );
    fn draw_indirect(context: &Context, args_buffer: &Buffer, args_offset: u64);
    fn draw_indexed_indirect(context: &Context, args_buffer: &Buffer, args_offset: u64);

    // --- compute ---
    fn set_compute_pipeline(context: &Context, pipeline: &ComputePipeline);
    fn set_compute_constant_buffer(context: &Context, slot: u32, buffer: &Buffer);
    fn set_unordered_access_view(context: &Context, slot: u32, view: &UnorderedAccessView);
    fn dispatch(context: &Context, x: u32, y: u32, z: u32);
    fn dispatch_indirect(context: &Context, args_buffer: &Buffer, args_offset: u64);

    // --- cópias ---
    fn copy_buffer(context: &Context, src: &Buffer, dst: &Buffer);
    fn copy_texture(context: &Context, src: &Texture2D, dst: &Texture2D);

    // --- command encoder / queue ---
    fn create_command_encoder(device: &Device) -> Result<CommandEncoder, GraphicsError>;
    fn finish_command_list(context: &Context) -> Result<CommandBuffer, GraphicsError>;
    fn execute_command_list(context: &Context, buffer: &CommandBuffer);
    fn wait_idle(context: &Context) -> Result<(), GraphicsError>;

    // --- queries ---
    fn create_query(device: &Device, kind: QueryType) -> Result<Query, GraphicsError>;
    fn begin_query(context: &Context, query: &Query);
    fn end_query(context: &Context, query: &Query);
    fn is_query_ready(context: &Context, query: &Query) -> bool;
    fn get_occlusion_result(context: &Context, query: &Query) -> Option<u64>;

    // --- debug ---
    fn push_debug_group(context: &Context, label: &str);
    fn pop_debug_group(context: &Context);
    fn insert_debug_marker(context: &Context, label: &str);
    fn set_buffer_label(buffer: &Buffer, label: &str);
    fn set_texture_label(texture: &Texture2D, label: &str);

    // --- sampler ---
    fn create_sampler(device: &Device, desc: &SamplerDescriptor) -> Result<Sampler, GraphicsError>;

    // --- surface / swapchain ---
    fn surface_from_handle(handle: raw_window_handle::RawWindowHandle) -> Result<Surface, GraphicsError>;
    fn create_swapchain(
        device: &Device,
        surface: &Surface,
        desc: &SwapchainDescriptor,
    ) -> Result<Self::RawSwapchain, GraphicsError>;
    fn resize_swapchain(
        raw: &mut Self::RawSwapchain,
        width: u32,
        height: u32,
    ) -> Result<(), GraphicsError>;
    fn create_swapchain_render_target_view(
        device: &Device,
        raw: &Self::RawSwapchain,
        size: Extent2D,
        format: Format,
    ) -> Result<RenderTargetView, GraphicsError>;
    fn capture_swapchain_frame(
        device: &Device,
        raw: &Self::RawSwapchain,
    ) -> Result<(u32, u32, Format, Vec<u8>), GraphicsError>;
    fn present(raw: &Self::RawSwapchain, present_mode: PresentMode) -> Result<PresentResult, GraphicsError>;
}
