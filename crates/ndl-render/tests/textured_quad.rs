//! Textura + sampler + constant buffer, todos de verdade: desenha um
//! quad texturizado (checkerboard 2x2) que pulsa de tamanho a cada
//! quadro, com a matriz de transform vindo de `ndl-math` — prova que o
//! jogo pode usar `ndl-math` para montar os bytes que `ndl-render` só
//! recebe como dados crus (`ndl-render` nunca depende de `ndl-math`).
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
use ndl_render::pipeline::GraphicsPipelineDescriptor;
use ndl_render::raster::Viewport;
use ndl_render::sampler::SamplerDescriptor;
use ndl_render::shader::compile_hlsl;
use ndl_render::surface::{Extent2D, Surface, Swapchain, SwapchainDescriptor};
use ndl_render::texture::TextureDescriptor;

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
        let class_name = w!("NdlRenderTexturedQuadTest");

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
            w!("ndl-render - quad texturizado"),
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

/// Extrai os 16 floats de uma `Mat4` em layout column-major (o mesmo que
/// o HLSL espera por padrão para `mul(matrix, vetor_coluna)`) — sem
/// depender de como a struct está empacotada na memória.
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

const SHADER_SOURCE: &str = r#"
cbuffer Transform : register(b0) {
    float4x4 transform;
};

Texture2D tex : register(t0);
SamplerState samp : register(s0);

struct VsOutput {
    float4 position : SV_POSITION;
    float2 uv : TEXCOORD0;
};

VsOutput vs_main(float2 position : POSITION, float2 uv : TEXCOORD0) {
    VsOutput output;
    output.position = mul(transform, float4(position, 0.0, 1.0));
    output.uv = uv;
    return output;
}

float4 ps_main(VsOutput input) : SV_TARGET {
    return tex.Sample(samp, input.uv);
}
"#;

#[repr(C)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

#[test]
fn draws_a_textured_pulsing_quad() {
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

    // --- textura: checkerboard 2x2, branco/preto ---
    let pixels: [u8; 16] = [
        255, 255, 255, 255, // branco
        0, 0, 0, 255, // preto
        0, 0, 0, 255, // preto
        255, 255, 255, 255, // branco
    ];
    let texture = device
        .create_texture_2d(
            &TextureDescriptor {
                width: 2,
                height: 2,
                format: Format::Rgba8Unorm,
            },
            &pixels,
        )
        .expect("falha ao criar textura");
    let texture_view = device
        .create_shader_resource_view(&texture)
        .expect("falha ao criar shader resource view");
    let sampler = device
        .create_sampler(&SamplerDescriptor::point_clamp())
        .expect("falha ao criar sampler");

    // --- shaders ---
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
            semantic: "TEXCOORD",
            semantic_index: 0,
            format: Format::Rg32Float,
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

    // --- quad (2 triângulos) ---
    let vertices = [
        Vertex {
            position: [-0.5, 0.5],
            uv: [0.0, 0.0],
        },
        Vertex {
            position: [0.5, 0.5],
            uv: [1.0, 0.0],
        },
        Vertex {
            position: [-0.5, -0.5],
            uv: [0.0, 1.0],
        },
        Vertex {
            position: [-0.5, -0.5],
            uv: [0.0, 1.0],
        },
        Vertex {
            position: [0.5, 0.5],
            uv: [1.0, 0.0],
        },
        Vertex {
            position: [0.5, -0.5],
            uv: [1.0, 1.0],
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

    // --- constant buffer: matriz de transform, montada com ndl-math ---
    let transform_buffer = device
        .create_buffer(&BufferDescriptor {
            size: 64, // float4x4
            usage: BufferUsage::Constant,
            dynamic: false,
            stride: 0,
        })
        .expect("falha ao criar constant buffer");

    let context = device.context();
    let background = [0.05, 0.05, 0.08, 1.0];
    let mut first_frame_capture = None;

    for frame in 0..20 {
        pump_messages();

        // Escala pulsando com o tempo — prova que a matriz vem de
        // ndl_math::Mat4 de verdade e está sendo recalculada a cada quadro.
        let t = frame as f32 * 0.1;
        let scale = 0.6 + 0.3 * t.sin();
        let transform = Mat4::from_scale(Vec3::new(scale, scale, scale));
        context
            .update_buffer(&transform_buffer, &mat4_to_bytes(&transform))
            .expect("invalid constant buffer update");

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
            .set_constant_buffer(0, &transform_buffer)
            .expect("invalid constant buffer binding");
        assert!(context
            .set_constant_buffer(
                device.limits().max_uniform_buffer_bindings,
                &transform_buffer
            )
            .is_err());
        assert!(context
            .set_shader_resource(device.limits().max_sampled_resource_bindings, &texture_view)
            .is_err());
        context
            .set_shader_resource(0, &texture_view)
            .expect("invalid shader resource slot");
        assert!(context
            .set_sampler(device.limits().max_sampler_bindings, &sampler)
            .is_err());
        context
            .set_sampler(0, &sampler)
            .expect("invalid sampler slot");
        context.draw(6, 0).expect("invalid vertex draw");

        // No quadro 0, `scale` é exatamente 0.6 (sin(0) = 0) — captura
        // aqui porque dá pra calcular de antemão onde o quad cai na tela.
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

    // Com scale = 0.6, o quad (NDC base [-0.5, 0.5]) ocupa NDC
    // [-0.3, 0.3] em x e y — em pixels (800x600): x em [280, 520], y em
    // [210, 390]. A textura é um checkerboard 2x2 (branco, preto / preto,
    // branco) amostrada com filtro de ponto, então cada quadrante do quad
    // deve mostrar uma cor sólida previsível.
    let capture = first_frame_capture.expect("deveria ter capturado o primeiro quadro");

    let outside = capture.pixel_rgba(10, 10);
    assert!(
        (outside[0] as i16 - 13).abs() <= 3,
        "fora do quad deveria ser a cor de fundo, obtido {outside:?}"
    );

    let top_left = capture.pixel_rgba(340, 255); // uv ~ (0.25, 0.25) -> branco
    assert!(
        top_left[0] > 200,
        "quadrante uv(0.25,0.25) deveria ser branco, obtido {top_left:?}"
    );

    let top_right = capture.pixel_rgba(460, 255); // uv ~ (0.75, 0.25) -> preto
    assert!(
        top_right[0] < 50,
        "quadrante uv(0.75,0.25) deveria ser preto, obtido {top_right:?}"
    );

    let bottom_left = capture.pixel_rgba(340, 345); // uv ~ (0.25, 0.75) -> preto
    assert!(
        bottom_left[0] < 50,
        "quadrante uv(0.25,0.75) deveria ser preto, obtido {bottom_left:?}"
    );

    let bottom_right = capture.pixel_rgba(460, 345); // uv ~ (0.75, 0.75) -> branco
    assert!(
        bottom_right[0] > 200,
        "quadrante uv(0.75,0.75) deveria ser branco, obtido {bottom_right:?}"
    );

    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}
