//! Instanced rendering de verdade: um único `draw_indexed_instanced`
//! desenha três quads em posições/cores diferentes, lendo um segundo
//! vertex buffer (`VertexStepMode::Instance`) em vez de três draw calls
//! separados — e o teste confere, pixel a pixel, que as três instâncias
//! apareceram nos lugares certos com as cores certas (não só "não
//! quebrou").
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
use ndl_render::input::{IndexFormat, PrimitiveTopology, VertexAttribute, VertexStepMode};
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
        let class_name = w!("NdlRenderInstancingTest");

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
            w!("ndl-render - instancing"),
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

VsOutput vs_main(
    float2 position : POSITION,
    float2 instance_offset : INSTANCE_OFFSET,
    float3 instance_color : INSTANCE_COLOR
) {
    VsOutput output;
    output.position = float4(position + instance_offset, 0.0, 1.0);
    output.color = instance_color;
    return output;
}

float4 ps_main(VsOutput input) : SV_TARGET {
    return float4(input.color, 1.0);
}
"#;

#[repr(C)]
struct Vertex {
    position: [f32; 2],
}

#[repr(C)]
struct InstanceData {
    offset: [f32; 2],
    color: [f32; 3],
}

fn as_bytes<T>(value: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(value.as_ptr() as *const u8, std::mem::size_of_val(value)) }
}

#[test]
fn draws_three_instanced_quads_with_one_draw_call() {
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

    // Atributo por vértice (slot 0, posição local do quad) e dois
    // atributos por instância (slot 1, avançam uma vez por instância em
    // vez de uma vez por vértice) — é essa combinação que faz
    // `draw_indexed_instanced` variar posição/cor por instância sem o
    // vertex shader receber nenhum estado extra explícito.
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
            semantic: "INSTANCE_OFFSET",
            semantic_index: 0,
            format: Format::Rg32Float,
            offset: 0,
            input_slot: 1,
            step_mode: VertexStepMode::Instance,
        },
        VertexAttribute {
            semantic: "INSTANCE_COLOR",
            semantic_index: 0,
            format: Format::Rgb32Float,
            offset: 8,
            input_slot: 1,
            step_mode: VertexStepMode::Instance,
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
                depth_stencil: DepthStencilStateDescriptor::disabled(),
                color_format: Format::Bgra8Unorm,
                depth_format: None,
                stencil_reference: 0,
            },
        )
        .expect("falha ao criar pipeline");

    // Um quad pequeno em coordenadas locais — a mesma geometria é
    // reaproveitada pelas três instâncias, cada uma só deslocando/
    // colorindo via o segundo vertex buffer.
    const H: f32 = 0.12;
    let vertices = [
        Vertex { position: [-H, H] },  // 0: topo-esquerda
        Vertex { position: [H, H] },   // 1: topo-direita
        Vertex { position: [-H, -H] }, // 2: baixo-esquerda
        Vertex { position: [H, -H] },  // 3: baixo-direita
    ];
    let vertex_buffer = device
        .create_buffer_with_data(
            &BufferDescriptor {
                size: std::mem::size_of_val(&vertices) as u64,
                usage: BufferUsage::Vertex,
                dynamic: false,
                stride: 0,
            },
            as_bytes(&vertices),
        )
        .expect("falha ao criar vertex buffer");

    let indices: [u16; 6] = [0, 3, 2, 0, 1, 3];
    let index_buffer = device
        .create_buffer_with_data(
            &BufferDescriptor {
                size: std::mem::size_of_val(&indices) as u64,
                usage: BufferUsage::Index,
                dynamic: false,
                stride: 0,
            },
            as_bytes(&indices),
        )
        .expect("falha ao criar index buffer");

    // Três instâncias lado a lado, cada uma com cor própria.
    let instances = [
        InstanceData {
            offset: [-0.5, 0.0],
            color: [1.0, 0.0, 0.0],
        },
        InstanceData {
            offset: [0.0, 0.0],
            color: [0.0, 1.0, 0.0],
        },
        InstanceData {
            offset: [0.5, 0.0],
            color: [0.0, 0.0, 1.0],
        },
    ];
    let instance_buffer = device
        .create_buffer_with_data(
            &BufferDescriptor {
                size: std::mem::size_of_val(&instances) as u64,
                usage: BufferUsage::Vertex,
                dynamic: false,
                stride: 0,
            },
            as_bytes(&instances),
        )
        .expect("falha ao criar instance buffer");

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
            pass.set_viewport(Viewport::full(800.0, 600.0))
                .expect("invalid viewport");
            pass.set_pipeline(&pipeline)
                .expect("pipeline compatível com o attachment");
            pass.set_vertex_buffer(0, &vertex_buffer, std::mem::size_of::<Vertex>() as u32)
                .expect("invalid vertex buffer binding");
            pass.set_vertex_buffer(1, &instance_buffer, std::mem::size_of::<InstanceData>() as u32)
                .expect("invalid instance buffer binding");
            pass.set_index_buffer(&index_buffer, IndexFormat::Uint16)
                .expect("invalid index buffer binding");

            // As três instâncias inteiras num único draw call — nenhum
            // `draw_indexed` é emitido três vezes aqui.
            pass.draw_indexed_instanced(0..6, 0, 3, 0)
                .expect("falha ao desenhar instanced");
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

    let capture = last_capture.expect("deveria ter capturado o último quadro");

    // Centros esperados em pixel (NDC -> pixel, 800x600):
    //   offset (-0.5, 0) -> (200, 300), vermelho
    //   offset ( 0.0, 0) -> (400, 300), verde
    //   offset ( 0.5, 0) -> (600, 300), azul
    let left = capture.pixel_rgba(200, 300);
    assert!(
        left[0] > 150 && left[0] > left[1] + 30 && left[0] > left[2] + 30,
        "instância esquerda deveria ser predominantemente vermelha, obtido {left:?}"
    );

    let center = capture.pixel_rgba(400, 300);
    assert!(
        center[1] > 150 && center[1] > center[0] + 30 && center[1] > center[2] + 30,
        "instância central deveria ser predominantemente verde, obtido {center:?}"
    );

    let right = capture.pixel_rgba(600, 300);
    assert!(
        right[2] > 150 && right[2] > right[0] + 30 && right[2] > right[1] + 30,
        "instância direita deveria ser predominantemente azul, obtido {right:?}"
    );

    // Entre instâncias (gap) continua com a cor de fundo — prova que são
    // três quads distintos, não uma faixa única cobrindo a tela inteira.
    let gap = capture.pixel_rgba(300, 300);
    assert!(
        (gap[0] as i16 - 13).abs() <= 3 && (gap[2] as i16 - 20).abs() <= 3,
        "pixel entre instâncias deveria ser a cor de fundo, obtido {gap:?}"
    );

    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}
