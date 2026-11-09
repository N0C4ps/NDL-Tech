//! O primeiro triângulo de verdade: compila HLSL, cria buffer/shaders/
//! input layout/pipeline, e desenha num back buffer real — numa janela
//! Win32 crua, sem tocar em `ndl-platform` (mesmo princípio de
//! `surface_triangle.rs`: a conexão entre janela e render é sempre
//! trabalho do jogo, nunca uma dependência entre as crates).
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
use ndl_render::input::{PrimitiveTopology, VertexAttribute, VertexStepMode};
use ndl_render::pipeline::{GraphicsPipelineDescriptor, GraphicsPipelineStateDescriptor};
use ndl_render::raster::Viewport;
use ndl_render::render::{ColorAttachment, LoadOp, RenderPassDescriptor};
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
        let class_name = w!("NdlRenderTriangleTest");

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
            w!("ndl-render - triangulo"),
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
fn draws_a_colored_triangle() {
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

    // --- input layout ---
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

    // --- pipeline ---
    let pipeline_desc = GraphicsPipelineDescriptor {
        vertex_shader: &vertex_shader,
        pixel_shader: &pixel_shader,
        input_layout: &input_layout,
        primitive_topology: PrimitiveTopology::TriangleList,
    };
    let pipeline_state = GraphicsPipelineStateDescriptor {
        rasterizer: Default::default(),
        blend: BlendStateDescriptor::opaque(),
        depth_stencil: DepthStencilStateDescriptor::disabled(),
        color_format: Format::Bgra8Unorm,
        depth_format: None,
        stencil_reference: 0,
    };
    let pipeline = device
        .create_graphics_pipeline_with_state(&pipeline_desc, &pipeline_state)
        .expect("falha ao criar pipeline com estados explícitos");
    assert!(
        device
            .create_graphics_pipeline_with_state(
                &pipeline_desc,
                &GraphicsPipelineStateDescriptor {
                    depth_stencil: DepthStencilStateDescriptor::depth_read_write(),
                    ..pipeline_state
                },
            )
            .is_err()
    );
    let mut stencil_state = DepthStencilStateDescriptor::depth_read_write();
    stencil_state.stencil_test_enabled = true;
    assert!(
        device
            .create_graphics_pipeline_with_state(
                &pipeline_desc,
                &GraphicsPipelineStateDescriptor {
                    depth_stencil: stencil_state,
                    depth_format: Some(Format::D32Float),
                    ..pipeline_state
                },
            )
            .is_err()
    );
    let incompatible_pipeline = device
        .create_graphics_pipeline_with_state(
            &pipeline_desc,
            &GraphicsPipelineStateDescriptor {
                color_format: Format::Rgba8Unorm,
                ..pipeline_state
            },
        )
        .expect("falha ao criar pipeline para teste de incompatibilidade");

    // --- vertex buffer: um triângulo vermelho/verde/azul ---
    let vertices = [
        Vertex {
            position: [0.0, 0.5],
            color: [1.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.5, -0.5],
            color: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [-0.5, -0.5],
            color: [0.0, 0.0, 1.0],
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

    let context = device.context();
    let background = [0.05, 0.05, 0.08, 1.0];
    let mut last_capture = None;

    for i in 0..20 {
        pump_messages();

        {
            let view = swapchain
                .render_target_view(&device)
                .expect("falha ao obter a render target view");
            let pass_desc = RenderPassDescriptor {
                color: ColorAttachment {
                    view,
                    load: LoadOp::Clear(background),
                },
                depth: None,
            };
            let mut pass = context
                .begin_render_pass(&pass_desc)
                .expect("falha ao iniciar render pass");
            assert!(pass.set_viewport(Viewport::full(0.0, 600.0)).is_err());
            pass.set_viewport(Viewport::full(800.0, 600.0))
                .expect("invalid viewport");
            assert!(pass.set_pipeline(&incompatible_pipeline).is_err());
            pass.set_pipeline(&pipeline)
                .expect("pipeline compatível com o attachment");
            pass.set_vertex_buffer(0, &vertex_buffer, std::mem::size_of::<Vertex>() as u32)
                .expect("invalid vertex buffer binding");
            assert!(pass.draw(0..4).is_err());
            pass.draw(0..3).expect("falha ao desenhar o triangulo");
            pass.end();
        }

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

    // Valida de verdade que o triângulo foi desenhado no lugar certo com
    // as cores certas, em vez de só confiar que os draw calls não
    // quebraram.
    //
    // Vértices em coordenadas de tela (NDC -> pixel, 800x600):
    //   topo    (0.0,  0.5) -> (400, 150), vermelho
    //   dir-bx  (0.5, -0.5) -> (600, 450), verde
    //   esq-bx (-0.5, -0.5) -> (200, 450), azul
    let capture = last_capture.expect("deveria ter capturado o último quadro");

    // Fora do triângulo (canto) continua com a cor de fundo.
    let outside = capture.pixel_rgba(10, 10);
    assert!(
        (outside[0] as i16 - 13).abs() <= 3 && (outside[2] as i16 - 20).abs() <= 3,
        "pixel fora do triângulo deveria ser a cor de fundo, obtido {outside:?}"
    );

    // Perto do vértice de cima, a cor deve ser predominantemente vermelha.
    let near_top = capture.pixel_rgba(400, 170);
    assert!(
        near_top[0] > 150 && near_top[0] > near_top[1] + 30 && near_top[0] > near_top[2] + 30,
        "pixel perto do vértice de cima deveria ser predominantemente vermelho, obtido {near_top:?}"
    );

    // O centroide do triângulo (dentro dele) não deve ser a cor de fundo.
    let center = capture.pixel_rgba(400, 350);
    let diff_from_background =
        (center[0] as i16 - outside[0] as i16).abs() + (center[2] as i16 - outside[2] as i16).abs();
    assert!(
        diff_from_background > 20,
        "centro do triângulo deveria ser visivelmente diferente do fundo, obtido {center:?} vs fundo {outside:?}"
    );

    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}
