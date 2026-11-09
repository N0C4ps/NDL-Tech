//! `transfer` (cópia de buffer), `query` (oclusão), `sync` (fence) e
//! `debug` (grupos/marcadores), todos de verdade, ao redor de um desenho
//! real numa janela.
#![cfg(target_os = "windows")]

use std::num::NonZeroIsize;

use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, PeekMessageW,
    PostQuitMessage, RegisterClassW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    MSG, PM_REMOVE, WM_DESTROY, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

use ndl_render::buffer::{BufferDescriptor, BufferUsage};
use ndl_render::core::{Device, DeviceDescriptor};
use ndl_render::format::Format;
use ndl_render::input::{PrimitiveTopology, VertexAttribute};
use ndl_render::pipeline::GraphicsPipelineDescriptor;
use ndl_render::query::QueryType;
use ndl_render::raster::Viewport;
use ndl_render::shader::compile_hlsl;
use ndl_render::surface::{Extent2D, Surface, Swapchain, SwapchainDescriptor};

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_DESTROY {
        PostQuitMessage(0);
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn create_raw_window(width: i32, height: i32) -> HWND {
    unsafe {
        let hinstance: HINSTANCE = GetModuleHandleW(None).expect("GetModuleHandleW").into();
        let class_name = w!("NdlRenderTransferQuerySyncDebugTest");

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: hinstance,
            lpszClassName: class_name,
            ..Default::default()
        };
        RegisterClassW(&wc);

        CreateWindowExW(
            Default::default(),
            class_name,
            w!("ndl-render - transfer/query/sync/debug"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            width,
            height,
            None,
            None,
            hinstance,
            None,
        )
        .expect("CreateWindowExW")
    }
}

fn pump_messages() {
    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

const SHADER_SOURCE: &str = r#"
struct VsOutput {
    float4 position : SV_POSITION;
    float3 color : COLOR0;
};

VsOutput vs_main(float2 position : POSITION, float3 color : COLOR0) {
    VsOutput output;
    output.position = float4(position, 0.0, 1.0);
    output.color = color;
    return output;
}

float4 ps_main(VsOutput input) : SV_TARGET {
    return float4(input.color, 1.0);
}
"#;

#[repr(C)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

#[test]
fn transfer_query_sync_and_debug_markers() {
    let hwnd = create_raw_window(800, 600);

    let mut handle = Win32WindowHandle::new(NonZeroIsize::new(hwnd.0 as isize).unwrap());
    handle.hinstance = None;
    let raw_handle = RawWindowHandle::Win32(handle);

    let device = Device::new(&DeviceDescriptor::default()).expect("falha ao criar device");
    let surface = Surface::from_window_handle(raw_handle).expect("falha ao criar surface");

    let mut swapchain = Swapchain::new(
        &device,
        &surface,
        &SwapchainDescriptor {
            size: Extent2D::new(800, 600),
            ..Default::default()
        },
    )
    .expect("falha ao criar swapchain");

    let vs_bytecode =
        compile_hlsl(SHADER_SOURCE, "vs_main", "vs_5_0").expect("falha ao compilar vertex shader");
    let ps_bytecode =
        compile_hlsl(SHADER_SOURCE, "ps_main", "ps_5_0").expect("falha ao compilar pixel shader");
    let vertex_shader = device
        .create_vertex_shader(&vs_bytecode)
        .expect("falha ao criar vertex shader");
    let pixel_shader = device
        .create_pixel_shader(&ps_bytecode)
        .expect("falha ao criar pixel shader");

    let attributes = [
        VertexAttribute {
            semantic: "POSITION",
            semantic_index: 0,
            format: Format::Rg32Float,
            offset: 0,
        },
        VertexAttribute {
            semantic: "COLOR",
            semantic_index: 0,
            format: Format::Rgb32Float,
            offset: 8,
        },
    ];
    let input_layout = device
        .create_input_layout(&attributes, &vertex_shader)
        .expect("falha ao criar input layout");

    let pipeline = device
        .create_graphics_pipeline(&GraphicsPipelineDescriptor {
            vertex_shader: &vertex_shader,
            pixel_shader: &pixel_shader,
            input_layout: &input_layout,
            primitive_topology: PrimitiveTopology::TriangleList,
        })
        .expect("falha ao criar pipeline");

    let vertices = [
        Vertex { position: [0.0, 0.5], color: [1.0, 0.8, 0.2] },
        Vertex { position: [0.5, -0.5], color: [0.2, 0.8, 1.0] },
        Vertex { position: [-0.5, -0.5], color: [0.8, 0.2, 1.0] },
    ];
    let vertex_bytes = unsafe {
        std::slice::from_raw_parts(
            vertices.as_ptr() as *const u8,
            std::mem::size_of_val(&vertices),
        )
    };
    let vertex_buffer = device
        .create_buffer_with_data(
            &BufferDescriptor {
                size: vertex_bytes.len() as u64,
                usage: BufferUsage::Vertex,
                dynamic: false,
                stride: 0,
            },
            vertex_bytes,
        )
        .expect("falha ao criar vertex buffer");

    // transfer: copia o vertex buffer para um segundo buffer idêntico.
    let vertex_buffer_copy = device
        .create_buffer(&BufferDescriptor {
            size: vertex_bytes.len() as u64,
            usage: BufferUsage::Vertex,
            dynamic: false,
            stride: 0,
        })
        .expect("falha ao criar o buffer de destino da cópia");

    // query: uma query de oclusão ao redor do draw.
    let occlusion_query = device
        .create_query(QueryType::Occlusion)
        .expect("falha ao criar query de oclusão");

    // sync: uma fence sinalizada depois do desenho deste quadro.
    let fence = device.create_fence().expect("falha ao criar fence");

    let context = device.context();
    context.copy_buffer(&vertex_buffer, &vertex_buffer_copy);

    // Valida de verdade que a cópia levou os bytes certos, em vez de só
    // confiar que `CopyResource` não quebrou: copia o resultado pra um
    // buffer de staging (legível pela CPU) e compara byte a byte com os
    // dados originais.
    let staging_buffer = device
        .create_buffer(&BufferDescriptor {
            size: vertex_bytes.len() as u64,
            usage: BufferUsage::Staging,
            dynamic: false,
            stride: 0,
        })
        .expect("falha ao criar o buffer de staging");
    context.copy_buffer(&vertex_buffer_copy, &staging_buffer);
    {
        let mapped = context
            .map_read(&staging_buffer)
            .expect("falha ao mapear o buffer de staging para leitura");
        assert_eq!(
            mapped.as_slice(),
            vertex_bytes,
            "a cópia via CopyResource deveria ter levado os bytes originais do vertex buffer"
        );
    }

    let mut last_capture = None;
    let background = [0.05, 0.05, 0.08, 1.0];

    for i in 0..15 {
        pump_messages();

        let view = swapchain
            .render_target_view(&device)
            .expect("falha ao obter a render target view");

        // debug: grupo nomeado ao redor do desenho do quadro, com um
        // marcador pontual logo antes do draw em si.
        context.push_debug_group("frame");
        context.set_render_target(view);
        context.clear_render_target(view, background);
        context.set_viewport(Viewport::full(800.0, 600.0));
        context.set_pipeline(&pipeline);
        context.set_vertex_buffer(0, &vertex_buffer_copy, std::mem::size_of::<Vertex>() as u32);

        context.begin_query(&occlusion_query);
        context.insert_debug_marker("draw triangle");
        context.draw(3, 0);
        context.end_query(&occlusion_query);
        context.pop_debug_group();

        fence.signal(&context);
        fence.wait(&context);

        if i == 14 {
            last_capture = Some(swapchain.capture_frame(&device).expect("falha ao capturar o quadro"));
        }

        swapchain.present().expect("falha ao apresentar");
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    // O triângulo foi desenhado a partir de vertex_buffer_copy — se a
    // cópia (já validada acima) e o draw funcionaram, o centroide do
    // triângulo não deve ser a cor de fundo.
    let capture = last_capture.expect("deveria ter capturado o último quadro");
    let outside = capture.pixel_rgba(10, 10);
    let center = capture.pixel_rgba(400, 300);
    assert!(
        (center[0] as i16 - outside[0] as i16).abs() + (center[2] as i16 - outside[2] as i16).abs() > 20,
        "centro do triângulo (desenhado a partir da cópia) deveria ser diferente do fundo; centro={center:?} fundo={outside:?}"
    );

    // A query de oclusão deve estar pronta a essa altura — a fence já
    // esperou a GPU terminar o último quadro, que é posterior ao
    // end_query.
    let pixels_drawn = context.get_occlusion_result(&occlusion_query);
    assert!(pixels_drawn.is_some(), "query de oclusão deveria estar pronta");

    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}
