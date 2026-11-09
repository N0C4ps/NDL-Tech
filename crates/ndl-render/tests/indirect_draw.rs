//! Draw indireto de verdade: os argumentos de contagem/offset não são
//! passados pela CPU a cada chamada — ficam num `Buffer` com
//! `BufferUsage::Indirect` que a própria GPU lê ao executar
//! `draw_indirect`/`draw_indexed_indirect`. O teste confere o resultado
//! real (pixels do back buffer), não só que a chamada não falhou, e
//! também confere que a validação de buffer/offset rejeita entradas
//! inválidas antes de tocar a GPU.
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
use ndl_render::buffer::{
    BufferDescriptor, BufferUsage, DrawIndexedIndirectArgs, DrawIndirectArgs,
};
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

fn create_raw_window(width: i32, height: i32, title: windows::core::PCWSTR) -> HWND {
    unsafe {
        let hinstance: HINSTANCE = GetModuleHandleW(None).expect("GetModuleHandleW").into();
        let class_name = w!("NdlRenderIndirectDrawTest");

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
            title,
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

fn as_bytes<T>(value: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(value.as_ptr() as *const u8, std::mem::size_of_val(value)) }
}

fn attributes() -> [VertexAttribute; 2] {
    [
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
    ]
}

#[test]
fn draws_a_triangle_via_draw_indirect() {
    let hwnd = create_raw_window(800, 600, w!("ndl-render - draw_indirect"));

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

    let attrs = attributes();
    let input_layout = device
        .create_input_layout(&attrs, &vertex_shader)
        .expect("falha ao criar input layout");

    let pipeline = device
        .create_graphics_pipeline_with_state(
            &GraphicsPipelineDescriptor {
                vertex_shader: &vertex_shader,
                pixel_shader: &pixel_shader,
                input_layout: &input_layout,
                primitive_topology: PrimitiveTopology::TriangleList,
            },
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

    // Os argumentos do draw (3 vértices, 1 instância, sem offsets) vivem
    // num buffer indirect em vez de serem passados diretamente — é esse
    // buffer que `draw_indirect` lê na GPU.
    let args = DrawIndirectArgs {
        vertex_count: 3,
        instance_count: 1,
        first_vertex: 0,
        first_instance: 0,
    };
    let args_buffer = device
        .create_buffer_with_data(
            &BufferDescriptor {
                size: DrawIndirectArgs::SIZE,
                usage: BufferUsage::Indirect,
                dynamic: false,
                stride: 0,
            },
            &args.to_bytes(),
        )
        .expect("falha ao criar o buffer de argumentos indiretos");

    let context = device.context();
    let background = [0.05, 0.05, 0.08, 1.0];
    let mut last_capture = None;

    for i in 0..20 {
        pump_messages();

        {
            let view = swapchain
                .render_target_view(&device)
                .expect("falha ao obter a render target view");
            let mut pass = context
                .begin_render_pass(&RenderPassDescriptor {
                    color: ColorAttachment {
                        view,
                        load: LoadOp::Clear(background),
                    },
                    depth: None,
                })
                .expect("falha ao iniciar render pass");
            pass.set_viewport(Viewport::full(800.0, 600.0))
                .expect("invalid viewport");
            pass.set_pipeline(&pipeline).expect("pipeline incompatível");
            pass.set_vertex_buffer(0, &vertex_buffer, std::mem::size_of::<Vertex>() as u32)
                .expect("invalid vertex buffer binding");

            // O desenho em si: nenhuma contagem de vértices passada aqui,
            // só o buffer e o offset onde estão os argumentos.
            pass.draw_indirect(&args_buffer, 0)
                .expect("falha ao desenhar indiretamente");
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

    let outside = capture.pixel_rgba(10, 10);
    assert!(
        (outside[0] as i16 - 13).abs() <= 3 && (outside[2] as i16 - 20).abs() <= 3,
        "pixel fora do triângulo deveria ser a cor de fundo, obtido {outside:?}"
    );
    let near_top = capture.pixel_rgba(400, 170);
    assert!(
        near_top[0] > 150 && near_top[0] > near_top[1] + 30 && near_top[0] > near_top[2] + 30,
        "pixel perto do vértice de cima deveria ser predominantemente vermelho, obtido {near_top:?}"
    );

    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}

#[test]
fn draws_a_quad_via_draw_indexed_indirect() {
    let hwnd = create_raw_window(800, 600, w!("ndl-render - draw_indexed_indirect"));

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

    let attrs = attributes();
    let input_layout = device
        .create_input_layout(&attrs, &vertex_shader)
        .expect("falha ao criar input layout");

    let pipeline = device
        .create_graphics_pipeline_with_state(
            &GraphicsPipelineDescriptor {
                vertex_shader: &vertex_shader,
                pixel_shader: &pixel_shader,
                input_layout: &input_layout,
                primitive_topology: PrimitiveTopology::TriangleList,
            },
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

    const H: f32 = 0.4;
    let vertices = [
        Vertex {
            position: [-H, H],
            color: [1.0, 1.0, 0.0],
        },
        Vertex {
            position: [H, H],
            color: [1.0, 1.0, 0.0],
        },
        Vertex {
            position: [-H, -H],
            color: [1.0, 1.0, 0.0],
        },
        Vertex {
            position: [H, -H],
            color: [1.0, 1.0, 0.0],
        },
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

    let args = DrawIndexedIndirectArgs {
        index_count: 6,
        instance_count: 1,
        first_index: 0,
        base_vertex: 0,
        first_instance: 0,
    };
    let args_buffer = device
        .create_buffer_with_data(
            &BufferDescriptor {
                size: DrawIndexedIndirectArgs::SIZE,
                usage: BufferUsage::Indirect,
                dynamic: false,
                stride: 0,
            },
            &args.to_bytes(),
        )
        .expect("falha ao criar o buffer de argumentos indiretos");

    let context = device.context();
    let background = [0.05, 0.05, 0.08, 1.0];
    let mut last_capture = None;

    for i in 0..20 {
        pump_messages();

        {
            let view = swapchain
                .render_target_view(&device)
                .expect("falha ao obter a render target view");
            let mut pass = context
                .begin_render_pass(&RenderPassDescriptor {
                    color: ColorAttachment {
                        view,
                        load: LoadOp::Clear(background),
                    },
                    depth: None,
                })
                .expect("falha ao iniciar render pass");
            pass.set_viewport(Viewport::full(800.0, 600.0))
                .expect("invalid viewport");
            pass.set_pipeline(&pipeline).expect("pipeline incompatível");
            pass.set_vertex_buffer(0, &vertex_buffer, std::mem::size_of::<Vertex>() as u32)
                .expect("invalid vertex buffer binding");
            pass.set_index_buffer(&index_buffer, IndexFormat::Uint16)
                .expect("invalid index buffer binding");

            pass.draw_indexed_indirect(&args_buffer, 0)
                .expect("falha ao desenhar indiretamente com índices");
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

    // O quad cobre o centro da tela (400, 300) com amarelo (R e G altos,
    // B baixo); fora dele (canto) continua com a cor de fundo.
    let center = capture.pixel_rgba(400, 300);
    assert!(
        center[0] > 150 && center[1] > 150 && center[2] < 100,
        "centro do quad deveria ser amarelo, obtido {center:?}"
    );
    let outside = capture.pixel_rgba(10, 10);
    assert!(
        (outside[0] as i16 - 13).abs() <= 3 && (outside[2] as i16 - 20).abs() <= 3,
        "pixel fora do quad deveria ser a cor de fundo, obtido {outside:?}"
    );

    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}

#[test]
fn draw_indirect_rejects_a_buffer_that_is_not_indirect() {
    let device = Device::new(&DeviceDescriptor {
        prefer_software: true,
        ..Default::default()
    })
    .expect("falha ao criar device WARP");

    // Um vertex buffer comum não serve como fonte de argumentos
    // indiretos — a validação deve rejeitar antes de tocar a GPU.
    let not_indirect = device
        .create_buffer(&BufferDescriptor {
            size: DrawIndirectArgs::SIZE,
            usage: BufferUsage::Vertex,
            dynamic: false,
            stride: 0,
        })
        .expect("falha ao criar buffer de vertex");

    let context = device.context();
    assert!(context.draw_indirect(&not_indirect, 0).is_err());

    let indirect = device
        .create_buffer(&BufferDescriptor {
            size: DrawIndirectArgs::SIZE,
            usage: BufferUsage::Indirect,
            dynamic: false,
            stride: 0,
        })
        .expect("falha ao criar buffer indirect");

    // Offset não alinhado a 4 bytes também é rejeitado.
    assert!(context.draw_indirect(&indirect, 1).is_err());
    // Offset que não deixa espaço para os argumentos também é rejeitado.
    assert!(context.draw_indirect(&indirect, DrawIndirectArgs::SIZE).is_err());
}
