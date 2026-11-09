//! `command`/`queue` de verdade: grava os comandos de desenho num
//! `CommandEncoder` (contexto diferido), termina a gravação num
//! `CommandBuffer`, submete na `GraphicsQueue` e espera a GPU terminar
//! via `wait_idle` antes de apresentar — em vez de desenhar diretamente
//! no contexto imediato, como os outros testes fazem.
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

use ndl_render::buffer::{BufferDescriptor, BufferUsage};
use ndl_render::core::{Device, DeviceDescriptor};
use ndl_render::format::Format;
use ndl_render::input::{PrimitiveTopology, VertexAttribute, VertexStepMode};
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
        let class_name = w!("NdlRenderCommandQueueTest");

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
            w!("ndl-render - command/queue"),
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
fn records_draw_in_a_deferred_context_and_submits_to_the_queue() {
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
            input_slot: 0,
            step_mode: VertexStepMode::Vertex,
        },
        VertexAttribute {
            semantic: "COLOR",
            semantic_index: 0,
            format: Format::Rgb32Float,
            offset: 8,
            input_slot: 0,
            step_mode: VertexStepMode::Vertex,
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
            color: [1.0, 1.0, 0.0],
        },
        Vertex {
            position: [0.5, -0.5],
            color: [0.0, 1.0, 1.0],
        },
        Vertex {
            position: [-0.5, -0.5],
            color: [1.0, 0.0, 1.0],
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

    let queue = device.queue();
    let background = [0.08, 0.05, 0.1, 1.0];
    let mut last_capture = None;

    for i in 0..15 {
        pump_messages();

        let view = swapchain
            .render_target_view(&device)
            .expect("falha ao obter a render target view");

        // Grava os comandos num contexto diferido — nada roda na GPU
        // ainda nesse ponto.
        let encoder = device
            .create_command_encoder()
            .expect("falha ao criar command encoder");
        encoder.set_render_target(view);
        encoder.clear_render_target(view, background);
        encoder
            .set_viewport(Viewport::full(800.0, 600.0))
            .expect("invalid viewport");
        encoder.set_pipeline(&pipeline);
        encoder
            .set_vertex_buffer(0, &vertex_buffer, std::mem::size_of::<Vertex>() as u32)
            .expect("invalid vertex buffer binding");
        encoder.draw(3, 0).expect("invalid vertex draw");
        let command_buffer = encoder.finish().expect("falha ao finalizar a gravação");

        // Só agora o trabalho gravado de fato roda na GPU.
        queue.submit(&[command_buffer]).expect("falha ao submeter");
        queue.wait_idle().expect("falha ao esperar a GPU");

        if i == 14 {
            last_capture = Some(
                swapchain
                    .capture_frame(&device)
                    .expect("falha ao capturar o quadro"),
            );
        }

        swapchain.present().expect("falha ao apresentar");
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    // O desenho foi gravado num CommandEncoder (contexto diferido) e só
    // executou na GPU depois do submit — se isso não funcionasse de
    // verdade, nada teria sido desenhado e o centro seria a cor de fundo.
    let capture = last_capture.expect("deveria ter capturado o último quadro");
    let outside = capture.pixel_rgba(10, 10);
    let center = capture.pixel_rgba(400, 300);
    assert!(
        (center[0] as i16 - outside[0] as i16).abs() + (center[2] as i16 - outside[2] as i16).abs() > 20,
        "centro do triângulo (desenhado via CommandEncoder/Queue) deveria ser diferente do fundo; centro={center:?} fundo={outside:?}"
    );

    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}
