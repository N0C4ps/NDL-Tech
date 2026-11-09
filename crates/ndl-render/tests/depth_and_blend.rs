//! Depth test + draw indexado + blend, todos de verdade: dois triângulos
//! sobrepostos em profundidades diferentes (prova que o depth buffer
//! decide o que fica visível, não a ordem de desenho) mais um quad
//! semi-transparente por cima (prova que o blend mistura em vez de
//! substituir) — tudo desenhado via `draw_indexed`, não `draw`.
#![cfg(target_os = "windows")]

use std::num::NonZeroIsize;

use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow,
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, PostQuitMessage, RegisterClassW,
    TranslateMessage, WM_DESTROY, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};
use windows::core::w;

use ndl_render::blend::BlendStateDescriptor;
use ndl_render::buffer::{BufferDescriptor, BufferUsage};
use ndl_render::core::{Device, DeviceDescriptor};
use ndl_render::depth_stencil::DepthStencilStateDescriptor;
use ndl_render::format::Format;
use ndl_render::input::{IndexFormat, PrimitiveTopology, VertexAttribute};
use ndl_render::pipeline::{GraphicsPipelineDescriptor, GraphicsPipelineStateDescriptor};
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
        let class_name = w!("NdlRenderDepthBlendTest");

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
            w!("ndl-render - depth + blend"),
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
    float4 color : COLOR0;
};

VsOutput vs_main(float3 position : POSITION, float4 color : COLOR0) {
    VsOutput output;
    output.position = float4(position, 1.0);
    output.color = color;
    return output;
}

float4 ps_main(VsOutput input) : SV_TARGET {
    return input.color;
}
"#;

#[repr(C)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}

#[test]
fn depth_test_and_blend_with_indexed_draws() {
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

    let depth_view = device
        .create_depth_texture(800, 600, Format::D32Float)
        .expect("falha ao criar depth buffer");
    let opaque_blend = device
        .create_blend_state(&BlendStateDescriptor::opaque())
        .expect("falha ao criar blend state opaco");
    let alpha_blend = device
        .create_blend_state(&BlendStateDescriptor::alpha_blend())
        .expect("falha ao criar blend state com alpha");

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
            format: Format::Rgb32Float,
            offset: 0,
        },
        VertexAttribute {
            semantic: "COLOR",
            semantic_index: 0,
            format: Format::Rgba32Float,
            offset: 12,
        },
    ];
    let input_layout = device
        .create_input_layout(&attributes, &vertex_shader)
        .expect("falha ao criar input layout");

    let pipeline_desc = GraphicsPipelineDescriptor {
        vertex_shader: &vertex_shader,
        pixel_shader: &pixel_shader,
        input_layout: &input_layout,
        primitive_topology: PrimitiveTopology::TriangleList,
    };
    let pipeline = device
        .create_graphics_pipeline_with_state(
            &pipeline_desc,
            &GraphicsPipelineStateDescriptor {
                rasterizer: Default::default(),
                blend: BlendStateDescriptor::opaque(),
                depth_stencil: DepthStencilStateDescriptor::depth_read_write(),
                color_format: Format::Bgra8Unorm,
                depth_format: Some(Format::D32Float),
                stencil_reference: 0,
            },
        )
        .expect("falha ao criar pipeline com depth explícito");

    // Vértices 0..3: triângulo vermelho, longe (z = 0.8).
    // Vértices 3..6: triângulo verde, perto (z = 0.2), mesma área na tela
    //                — com depth test, o verde deve "ganhar" do vermelho.
    // Vértices 6..10: quad azul semi-transparente, ainda mais perto
    //                 (z = 0.1), por cima dos dois via blend.
    let vertices = [
        Vertex {
            position: [-0.6, 0.6, 0.8],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.6, 0.6, 0.8],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.0, -0.6, 0.8],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex {
            position: [-0.6, 0.6, 0.2],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.6, 0.6, 0.2],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.0, -0.6, 0.2],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            position: [-0.3, 0.3, 0.1],
            color: [0.0, 0.3, 1.0, 0.4],
        },
        Vertex {
            position: [0.3, 0.3, 0.1],
            color: [0.0, 0.3, 1.0, 0.4],
        },
        Vertex {
            position: [-0.3, -0.3, 0.1],
            color: [0.0, 0.3, 1.0, 0.4],
        },
        Vertex {
            position: [0.3, -0.3, 0.1],
            color: [0.0, 0.3, 1.0, 0.4],
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

    let indices: [u16; 12] = [
        0, 1, 2, // triângulo vermelho
        3, 4, 5, // triângulo verde
        6, 7, 8, 7, 9, 8, // quad azul (2 triângulos)
    ];
    let index_bytes = unsafe {
        std::slice::from_raw_parts(
            indices.as_ptr() as *const u8,
            std::mem::size_of_val(&indices),
        )
    };
    let index_buffer = device
        .create_buffer_with_data(
            &BufferDescriptor {
                size: index_bytes.len() as u64,
                usage: BufferUsage::Index,
                dynamic: false,
                stride: 0,
            },
            index_bytes,
        )
        .expect("falha ao criar index buffer");

    let context = device.context();
    let background = [0.05, 0.05, 0.08, 1.0];
    let mut last_capture = None;

    for i in 0..20 {
        pump_messages();

        let color_view = swapchain
            .render_target_view(&device)
            .expect("falha ao obter a render target view");
        context.set_render_target_with_depth(color_view, &depth_view);
        context.clear_render_target(color_view, background);
        context
            .clear_depth(&depth_view, 1.0)
            .expect("invalid depth clear");
        context
            .set_viewport(Viewport::full(800.0, 600.0))
            .expect("invalid viewport");
        context.set_pipeline(&pipeline);
        context
            .set_vertex_buffer(0, &vertex_buffer, std::mem::size_of::<Vertex>() as u32)
            .expect("invalid vertex buffer binding");
        context
            .set_index_buffer(&index_buffer, IndexFormat::Uint16)
            .expect("invalid index buffer binding");

        // Os dois triângulos opacos: a ordem de desenho não deveria
        // importar, porque é o depth test que decide o que fica visível.
        context.set_blend_state(&opaque_blend);
        context
            .draw_indexed(3, 0, 0)
            .expect("indexed draw out of bounds"); // vermelho, longe
        context
            .draw_indexed(3, 3, 0)
            .expect("indexed draw out of bounds"); // verde, perto - deve "ganhar"

        // O quad semi-transparente por cima, via blend em vez de
        // substituição direta.
        context.set_blend_state(&alpha_blend);
        context
            .draw_indexed(6, 6, 0)
            .expect("indexed draw out of bounds");

        if i == 19 {
            last_capture = Some(
                swapchain
                    .capture_frame(&device)
                    .expect("falha ao capturar o quadro"),
            );
        }

        swapchain.present().expect("falha ao apresentar");
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    // Os dois triângulos (vermelho em z=0.8, verde em z=0.2) cobrem
    // exatamente a mesma região da tela — se o depth test estiver
    // funcionando de verdade (e não só "o último desenhado venceu"),
    // o verde (mais perto) tem que aparecer, nunca o vermelho, apesar do
    // vermelho ter sido desenhado DEPOIS na ordem dos draw_indexed.
    //
    // Ponto perto do ápice do triângulo (NDC (0, 0.5) -> pixel (400, 150)),
    // fora da área do quad azul (NDC y em [-0.3, 0.3]).
    let capture = last_capture.expect("deveria ter capturado o último quadro");

    let depth_test_point = capture.pixel_rgba(400, 150);
    assert!(
        depth_test_point[1] > 150 && depth_test_point[0] < 80,
        "depth test deveria mostrar o triângulo verde (mais perto), não o vermelho (mais longe); obtido {depth_test_point:?}"
    );

    // Fora de tudo (canto) continua com a cor de fundo.
    let outside = capture.pixel_rgba(10, 10);
    assert!(
        (outside[0] as i16 - 13).abs() <= 3,
        "fora de toda a geometria deveria ser a cor de fundo, obtido {outside:?}"
    );

    // Centro: dentro do triângulo verde E do quad azul semi-transparente
    // (alpha 0.4). Se o blend estivesse substituindo em vez de misturar,
    // o pixel seria azul puro (0, 76, 255) ou, se o blend não fizesse
    // nada, verde puro (0, 255, 0). Uma mistura real tem os dois canais
    // (verde e azul) claramente presentes, nenhum dos dois nos extremos.
    let blended = capture.pixel_rgba(400, 300);
    assert!(
        blended[1] > 40 && blended[1] < 230,
        "canal verde deveria estar misturado, não puro nem ausente; obtido {blended:?}"
    );
    assert!(
        blended[2] > 40 && blended[2] < 230,
        "canal azul deveria estar misturado, não puro nem ausente; obtido {blended:?}"
    );

    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}
