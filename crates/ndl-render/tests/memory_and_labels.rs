//! `memory::ConstantBufferRing` e `resource::set_label`, de verdade: a
//! matriz de transform é escrita a cada quadro via
//! `ConstantBufferRing::write` (que usa `Context::map_write_discard` por
//! baixo, desmapeando automaticamente no `Drop`), girando entre os
//! buffers do ring em vez de reescrever sempre o mesmo.
#![cfg(target_os = "windows")]

use std::num::NonZeroIsize;

use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, PeekMessageW,
    PostQuitMessage, RegisterClassW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, MSG,
    PM_REMOVE, WM_DESTROY, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

use ndl_math::{Mat4, Vec3};
use ndl_render::buffer::{BufferDescriptor, BufferUsage};
use ndl_render::core::{Device, DeviceDescriptor};
use ndl_render::format::Format;
use ndl_render::input::{PrimitiveTopology, VertexAttribute};
use ndl_render::memory::ConstantBufferRing;
use ndl_render::pipeline::GraphicsPipelineDescriptor;
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
        let class_name = w!("NdlRenderMemoryLabelsTest");

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
            w!("ndl-render - memory + labels"),
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
cbuffer Transform : register(b0) {
    float4x4 transform;
};

struct VsOutput {
    float4 position : SV_POSITION;
    float3 color : COLOR0;
};

VsOutput vs_main(float2 position : POSITION, float3 color : COLOR0) {
    VsOutput output;
    output.position = mul(transform, float4(position, 0.0, 1.0));
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

fn mat4_to_bytes(m: &Mat4) -> [u8; 64] {
    let mut bytes = [0u8; 64];
    for (i, col) in m.cols.iter().enumerate() {
        let floats = [col.x, col.y, col.z, col.w];
        for (j, f) in floats.iter().enumerate() {
            bytes[(i * 4 + j) * 4..(i * 4 + j) * 4 + 4].copy_from_slice(&f.to_le_bytes());
        }
    }
    bytes
}

#[test]
fn constant_buffer_ring_and_resource_labels() {
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
        Vertex {
            position: [0.0, 0.5],
            color: [1.0, 0.5, 0.0],
        },
        Vertex {
            position: [0.5, -0.5],
            color: [0.0, 0.5, 1.0],
        },
        Vertex {
            position: [-0.5, -0.5],
            color: [0.5, 1.0, 0.0],
        },
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

    // set_label: só precisa não paniciar e ser visível em ferramentas de
    // debug gráfico (RenderDoc/PIX) — não há como verificar isso em
    // código, então o teste garante que a chamada em si é segura.
    vertex_buffer.set_label("ndl-render-test-triangle-vertices");

    // O ring: 2 constant buffers, girando a cada escrita.
    let mut transform_ring =
        ConstantBufferRing::new(&device, 64, 2).expect("falha ao criar o constant buffer ring");

    let context = device.context();
    let background = [0.05, 0.05, 0.08, 1.0];
    let mut first_frame_capture = None;

    for frame in 0..20 {
        pump_messages();

        let t = frame as f32 * 0.1;
        let scale = 0.5 + 0.3 * t.sin();
        let transform = Mat4::from_scale(Vec3::new(scale, scale, scale));
        let transform_buffer = transform_ring
            .write(&context, &mat4_to_bytes(&transform))
            .expect("falha ao escrever no constant buffer ring");

        let view = swapchain
            .render_target_view(&device)
            .expect("falha ao obter a render target view");
        context.set_render_target(view);
        context.clear_render_target(view, background);
        context
            .set_viewport(Viewport::full(800.0, 600.0))
            .expect("invalid viewport");
        context.set_pipeline(&pipeline);
        context
            .set_vertex_buffer(0, &vertex_buffer, std::mem::size_of::<Vertex>() as u32)
            .expect("invalid vertex buffer binding");
        context
            .set_constant_buffer(0, transform_buffer)
            .expect("invalid constant buffer binding");
        context.draw(3, 0).expect("invalid vertex draw");

        // No quadro 0, scale = 0.5 (sin(0) = 0) — captura aqui porque dá
        // pra calcular de antemão onde o triângulo cai na tela.
        if frame == 0 {
            first_frame_capture = Some(
                swapchain
                    .capture_frame(&device)
                    .expect("falha ao capturar o quadro"),
            );
        }

        swapchain.present().expect("falha ao apresentar");
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    // Com scale = 0.5, o triângulo escalado tem vértices em NDC
    // (0, 0.25), (0.25, -0.25) e (-0.25, -0.25) — o centroide, bem
    // dentro do triângulo (não na borda nem no vértice, onde a área é
    // zero), cai em NDC (0, -0.0833) -> pixel (400, 325). Isso só é
    // verdade se o ConstantBufferRing realmente escreveu a matriz via
    // map_write_discard e o shader aplicou a transformação.
    let capture = first_frame_capture.expect("deveria ter capturado o primeiro quadro");
    let outside = capture.pixel_rgba(10, 10);
    let centroid = capture.pixel_rgba(400, 325);
    assert!(
        (centroid[0] as i16 - outside[0] as i16).abs()
            + (centroid[2] as i16 - outside[2] as i16).abs()
            > 20,
        "centroide do triângulo transformado deveria ser diferente do fundo; obtido {centroid:?} vs fundo {outside:?}"
    );

    // Fora do triângulo (mesmo escalado, a 0.5 ele não chega nem perto
    // dos cantos) continua com a cor de fundo.
    assert!(
        (outside[0] as i16 - 13).abs() <= 3,
        "fora do triângulo deveria ser a cor de fundo, obtido {outside:?}"
    );

    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}
