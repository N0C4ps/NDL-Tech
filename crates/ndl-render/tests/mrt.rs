//! MRT (multiple render targets) de verdade: um único draw call escreve
//! cores diferentes em dois attachments de cor simultâneos
//! (`SV_TARGET0`/`SV_TARGET1`), e o teste lê os dois de volta
//! independentemente (via `Device::capture_texture`) para confirmar que
//! cada um recebeu o valor certo — não só que o desenho não quebrou. Não
//! precisa de janela nenhuma: os attachments são texturas off-screen
//! criadas via `Device::create_render_target_texture`.
#![cfg(target_os = "windows")]

use ndl_render::core::{Device, DeviceDescriptor};
use ndl_render::format::Format;
use ndl_render::input::{PrimitiveTopology, VertexAttribute, VertexStepMode};
use ndl_render::pipeline::{GraphicsPipelineDescriptor, GraphicsPipelineStateDescriptor};
use ndl_render::raster::Viewport;
use ndl_render::render::{ColorAttachment, LoadOp, RenderPassDescriptor};
use ndl_render::shader::compile_hlsl;
use ndl_render::texture::TextureDescriptor;

const SHADER_SOURCE: &str = r#"
struct VsOutput {
    float4 position : SV_POSITION;
};

struct PsOutput {
    float4 target0 : SV_TARGET0;
    float4 target1 : SV_TARGET1;
};

VsOutput vs_main(float2 position : POSITION) {
    VsOutput output;
    output.position = float4(position, 0.0, 1.0);
    return output;
}

PsOutput ps_main(VsOutput input) {
    PsOutput output;
    output.target0 = float4(1.0, 0.0, 0.0, 1.0);
    output.target1 = float4(0.0, 1.0, 0.0, 1.0);
    return output;
}
"#;

#[repr(C)]
struct Vertex {
    position: [f32; 2],
}

fn as_bytes<T>(value: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(value.as_ptr() as *const u8, std::mem::size_of_val(value)) }
}

/// Um triângulo grande o bastante para cobrir o centro de uma textura
/// 64x64 — mesma orientação (clockwise em NDC) de `tests/triangle.rs`,
/// visível sob o `CullMode::Back` padrão.
fn covering_triangle_vertices() -> [Vertex; 3] {
    [
        Vertex { position: [0.0, 0.8] },
        Vertex { position: [0.8, -0.8] },
        Vertex { position: [-0.8, -0.8] },
    ]
}

#[test]
fn writes_different_colors_to_two_render_targets_in_one_draw_call() {
    let device = Device::new(&DeviceDescriptor::default()).expect("falha ao criar device");

    let (texture0, rtv0) = device
        .create_render_target_texture(&TextureDescriptor {
            width: 64,
            height: 64,
            format: Format::Bgra8Unorm,
        })
        .expect("falha ao criar render target 0");
    let (texture1, rtv1) = device
        .create_render_target_texture(&TextureDescriptor {
            width: 64,
            height: 64,
            format: Format::Bgra8Unorm,
        })
        .expect("falha ao criar render target 1");

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

    let attributes = [VertexAttribute {
        semantic: "POSITION",
        semantic_index: 0,
        format: Format::Rg32Float,
        offset: 0,
        input_slot: 0,
        step_mode: VertexStepMode::Vertex,
    }];
    let input_layout = device
        .create_input_layout(&attributes, &vertex_shader)
        .expect("falha ao criar input layout");

    // Dois formatos de cor — um por SV_TARGETn — é isso que torna este
    // pipeline um pipeline MRT; RenderPass::set_pipeline confere essa
    // contagem/ordem contra os attachments do pass antes de qualquer draw.
    let color_formats = [Format::Bgra8Unorm, Format::Bgra8Unorm];
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
                blend: ndl_render::blend::BlendStateDescriptor::opaque(),
                depth_stencil: ndl_render::depth_stencil::DepthStencilStateDescriptor::disabled(),
                color_formats: &color_formats,
                depth_format: None,
                stencil_reference: 0,
            },
        )
        .expect("falha ao criar pipeline MRT");

    let vertices = covering_triangle_vertices();
    let vertex_buffer = device
        .create_buffer_with_data(
            &ndl_render::buffer::BufferDescriptor {
                size: std::mem::size_of_val(&vertices) as u64,
                usage: ndl_render::buffer::BufferUsage::Vertex,
                dynamic: false,
                stride: 0,
            },
            as_bytes(&vertices),
        )
        .expect("falha ao criar vertex buffer");

    let context = device.context();
    let colors = [
        ColorAttachment {
            view: &rtv0,
            load: LoadOp::Clear([0.0, 0.0, 0.0, 1.0]),
        },
        ColorAttachment {
            view: &rtv1,
            load: LoadOp::Clear([0.0, 0.0, 0.0, 1.0]),
        },
    ];
    let mut pass = context
        .begin_render_pass(&RenderPassDescriptor {
            colors: &colors,
            depth: None,
        })
        .expect("falha ao iniciar render pass MRT");
    pass.set_viewport(Viewport::full(64.0, 64.0))
        .expect("invalid viewport");
    pass.set_pipeline(&pipeline).expect("pipeline MRT compatível");
    pass.set_vertex_buffer(0, &vertex_buffer, std::mem::size_of::<Vertex>() as u32)
        .expect("invalid vertex buffer binding");
    pass.draw(0..3).expect("falha ao desenhar nos dois alvos");
    pass.end();

    // Lê os dois attachments de volta *independentemente* — a prova real
    // de que o mesmo draw escreveu valores diferentes em cada um, não só
    // que o pass não lançou erro.
    let capture0 = device
        .capture_texture(&texture0)
        .expect("falha ao capturar render target 0");
    let capture1 = device
        .capture_texture(&texture1)
        .expect("falha ao capturar render target 1");

    let center0 = capture0.pixel_rgba(32, 40);
    assert!(
        center0[0] > 200 && center0[1] < 50 && center0[2] < 50,
        "target0 deveria ser vermelho no centro do triângulo, obtido {center0:?}"
    );
    let center1 = capture1.pixel_rgba(32, 40);
    assert!(
        center1[1] > 200 && center1[0] < 50 && center1[2] < 50,
        "target1 deveria ser verde no centro do triângulo, obtido {center1:?}"
    );

    // Fora do triângulo, os dois continuam com a cor de clear (preto) —
    // prova que o MRT não "vazou" cor fora da geometria rasterizada.
    let outside0 = capture0.pixel_rgba(2, 2);
    let outside1 = capture1.pixel_rgba(2, 2);
    assert!(
        outside0[0] < 10 && outside0[1] < 10 && outside0[2] < 10,
        "fora do triângulo, target0 deveria continuar preto, obtido {outside0:?}"
    );
    assert!(
        outside1[0] < 10 && outside1[1] < 10 && outside1[2] < 10,
        "fora do triângulo, target1 deveria continuar preto, obtido {outside1:?}"
    );
}

#[test]
fn begin_render_pass_rejects_zero_color_attachments() {
    let device = Device::new(&DeviceDescriptor {
        prefer_software: true,
        ..Default::default()
    })
    .expect("falha ao criar device WARP");
    let context = device.context();
    let colors: [ColorAttachment; 0] = [];
    let result = context.begin_render_pass(&RenderPassDescriptor {
        colors: &colors,
        depth: None,
    });
    assert!(result.is_err());
}

#[test]
fn begin_render_pass_rejects_mismatched_attachment_dimensions() {
    let device = Device::new(&DeviceDescriptor {
        prefer_software: true,
        ..Default::default()
    })
    .expect("falha ao criar device WARP");

    let (_t0, rtv0) = device
        .create_render_target_texture(&TextureDescriptor {
            width: 64,
            height: 64,
            format: Format::Bgra8Unorm,
        })
        .expect("falha ao criar render target 0");
    let (_t1, rtv1) = device
        .create_render_target_texture(&TextureDescriptor {
            width: 32,
            height: 32,
            format: Format::Bgra8Unorm,
        })
        .expect("falha ao criar render target 1 (dimensão diferente)");

    let context = device.context();
    let colors = [
        ColorAttachment {
            view: &rtv0,
            load: LoadOp::Clear([0.0, 0.0, 0.0, 1.0]),
        },
        ColorAttachment {
            view: &rtv1,
            load: LoadOp::Clear([0.0, 0.0, 0.0, 1.0]),
        },
    ];
    let result = context.begin_render_pass(&RenderPassDescriptor {
        colors: &colors,
        depth: None,
    });
    assert!(result.is_err());
}

#[test]
fn begin_render_pass_rejects_more_attachments_than_the_device_supports() {
    let device = Device::new(&DeviceDescriptor {
        prefer_software: true,
        ..Default::default()
    })
    .expect("falha ao criar device WARP");
    let max = device.limits().max_color_attachments as usize;

    let textures: Vec<_> = (0..=max)
        .map(|_| {
            device
                .create_render_target_texture(&TextureDescriptor {
                    width: 4,
                    height: 4,
                    format: Format::Bgra8Unorm,
                })
                .expect("falha ao criar render target")
        })
        .collect();
    let colors: Vec<ColorAttachment> = textures
        .iter()
        .map(|(_, view)| ColorAttachment {
            view,
            load: LoadOp::Clear([0.0, 0.0, 0.0, 1.0]),
        })
        .collect();

    let context = device.context();
    // max + 1 attachments: um a mais do que o dispositivo suporta.
    let result = context.begin_render_pass(&RenderPassDescriptor {
        colors: &colors,
        depth: None,
    });
    assert!(result.is_err());
}

#[test]
fn set_pipeline_rejects_attachment_count_mismatch() {
    let device = Device::new(&DeviceDescriptor {
        prefer_software: true,
        ..Default::default()
    })
    .expect("falha ao criar device WARP");

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
    let attributes = [VertexAttribute {
        semantic: "POSITION",
        semantic_index: 0,
        format: Format::Rg32Float,
        offset: 0,
        input_slot: 0,
        step_mode: VertexStepMode::Vertex,
    }];
    let input_layout = device
        .create_input_layout(&attributes, &vertex_shader)
        .expect("falha ao criar input layout");

    // Pipeline declara 2 alvos de cor...
    let color_formats = [Format::Bgra8Unorm, Format::Bgra8Unorm];
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
                blend: ndl_render::blend::BlendStateDescriptor::opaque(),
                depth_stencil: ndl_render::depth_stencil::DepthStencilStateDescriptor::disabled(),
                color_formats: &color_formats,
                depth_format: None,
                stencil_reference: 0,
            },
        )
        .expect("falha ao criar pipeline MRT");

    // ...mas o pass só tem 1.
    let (_texture, rtv) = device
        .create_render_target_texture(&TextureDescriptor {
            width: 16,
            height: 16,
            format: Format::Bgra8Unorm,
        })
        .expect("falha ao criar render target");
    let context = device.context();
    let colors = [ColorAttachment {
        view: &rtv,
        load: LoadOp::Clear([0.0, 0.0, 0.0, 1.0]),
    }];
    let mut pass = context
        .begin_render_pass(&RenderPassDescriptor {
            colors: &colors,
            depth: None,
        })
        .expect("falha ao iniciar render pass de 1 attachment");
    assert!(pass.set_pipeline(&pipeline).is_err());
}
