//! Mipmaps e texture arrays de verdade — o que faltava para uma cena 3D
//! com profundidade de câmera real não sofrer de aliasing/cache-miss de
//! textura à distância (ver `docs`/README: `Texture2D` não tinha mais do
//! que 1 mip, 1 camada).
//!
//! Três caminhos, cada um com uma assertiva de pixel que só passa se o
//! nível de mip certo (gerado na GPU ou fornecido explicitamente) ou a
//! camada de array certa foi de fato lida — não só "não quebrou":
//!
//! - `gpu_generated_mip_chain_filters_a_high_frequency_pattern`: cria uma
//!   textura com `mip_levels: 0` (cadeia completa), sobe só o nível base
//!   (colunas alternadas vermelho/azul — o pior caso de aliasing), chama
//!   `Context::generate_mips` e compara `SampleLevel` no nível 0 (deveria
//!   continuar "duro", sem mistura) contra o nível 1 (deveria estar
//!   filtrado/misturado — a prova de que o box filter da GPU rodou de
//!   verdade).
//! - `explicit_mip_chain_samples_the_exact_provided_level`: cadeia de 2
//!   níveis com conteúdo explícito e **diferente** por nível (vermelho no
//!   0, verde no 1) — prova que `create_texture_2d` grava cada
//!   subresource no lugar certo, não só o nível base.
//! - `texture_array_layers_sample_independently`: textura com 2 camadas
//!   de array, cada uma lida independentemente via `Texture2DArray`.
#![cfg(target_os = "windows")]

use ndl_render::buffer::{BufferDescriptor, BufferUsage};
use ndl_render::core::{Device, DeviceDescriptor};
use ndl_render::format::Format;
use ndl_render::input::{PrimitiveTopology, VertexAttribute, VertexStepMode};
use ndl_render::pipeline::GraphicsPipelineDescriptor;
use ndl_render::raster::Viewport;
use ndl_render::render::{ColorAttachment, LoadOp, RenderPassDescriptor};
use ndl_render::sampler::SamplerDescriptor;
use ndl_render::shader::compile_hlsl;
use ndl_render::texture::TextureDescriptor;

const RT_SIZE: u32 = 8;

const LOD_SHADER_SOURCE: &str = r#"
cbuffer LodCB : register(b0) {
    float lod;
    float3 _pad;
};

Texture2D tex : register(t0);
SamplerState samp : register(s0);

struct VsOutput {
    float4 position : SV_POSITION;
    float2 uv : TEXCOORD0;
};

VsOutput vs_main(float2 position : POSITION, float2 uv : TEXCOORD0) {
    VsOutput output;
    output.position = float4(position, 0.0, 1.0);
    output.uv = uv;
    return output;
}

float4 ps_main(VsOutput input) : SV_TARGET {
    return tex.SampleLevel(samp, input.uv, lod);
}
"#;

const ARRAY_SHADER_SOURCE: &str = r#"
cbuffer LayerCB : register(b0) {
    float layer_index;
    float3 _pad;
};

Texture2DArray tex : register(t0);
SamplerState samp : register(s0);

struct VsOutput {
    float4 position : SV_POSITION;
    float2 uv : TEXCOORD0;
};

VsOutput vs_main(float2 position : POSITION, float2 uv : TEXCOORD0) {
    VsOutput output;
    output.position = float4(position, 0.0, 1.0);
    output.uv = uv;
    return output;
}

float4 ps_main(VsOutput input) : SV_TARGET {
    return tex.SampleLevel(samp, float3(input.uv, layer_index), 0.0);
}
"#;

#[repr(C)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

fn as_bytes<T>(value: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(value.as_ptr() as *const u8, std::mem::size_of_val(value)) }
}

/// Um quad cobrindo a tela inteira (NDC -1..1), UV 0..1.
fn fullscreen_quad_vertices() -> [Vertex; 6] {
    [
        Vertex { position: [-1.0, 1.0], uv: [0.0, 0.0] },
        Vertex { position: [1.0, 1.0], uv: [1.0, 0.0] },
        Vertex { position: [-1.0, -1.0], uv: [0.0, 1.0] },
        Vertex { position: [-1.0, -1.0], uv: [0.0, 1.0] },
        Vertex { position: [1.0, 1.0], uv: [1.0, 0.0] },
        Vertex { position: [1.0, -1.0], uv: [1.0, 1.0] },
    ]
}

fn quad_attributes() -> [VertexAttribute; 2] {
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
            semantic: "TEXCOORD",
            semantic_index: 0,
            format: Format::Rg32Float,
            offset: 8,
            input_slot: 0,
            step_mode: VertexStepMode::Vertex,
        },
    ]
}

#[test]
fn gpu_generated_mip_chain_filters_a_high_frequency_pattern() {
    let device = Device::new(&DeviceDescriptor::default()).expect("falha ao criar device");

    // Pior caso de aliasing: colunas de 1 texel alternando vermelho/azul.
    // Largura 64 garante uma cadeia de mips longa o bastante (7 níveis)
    // para o nível 1 (32 de largura) ser claramente diferente do nível 0.
    const WIDTH: u32 = 64;
    const HEIGHT: u32 = 2;
    let mut base_pixels = vec![0u8; (WIDTH * HEIGHT * 4) as usize];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let offset = ((y * WIDTH + x) * 4) as usize;
            if x % 2 == 0 {
                base_pixels[offset..offset + 4].copy_from_slice(&[255, 0, 0, 255]); // vermelho
            } else {
                base_pixels[offset..offset + 4].copy_from_slice(&[0, 0, 255, 255]); // azul
            }
        }
    }

    let texture = device
        .create_texture_2d(
            &TextureDescriptor {
                width: WIDTH,
                height: HEIGHT,
                format: Format::Rgba8Unorm,
                mip_levels: 0, // cadeia completa gerada na GPU
                array_layers: 1,
            },
            &[&base_pixels], // só o nível base
        )
        .expect("falha ao criar textura com cadeia de mips gerada");
    assert_eq!(
        texture.mip_levels(),
        7,
        "64x2 deveria resultar em 7 níveis de mip (64,32,16,8,4,2,1)"
    );

    let texture_view = device
        .create_shader_resource_view(&texture)
        .expect("falha ao criar shader resource view");

    let context = device.context();
    context.generate_mips(&texture_view);

    let (color_pixel_lod0, color_pixel_lod1) =
        render_with_lod_scalar(&device, LOD_SHADER_SOURCE, &texture_view, 0.0, 1.0);

    // Nível 0: ponto de amostra exato no centro da tela cai sobre um
    // texel inteiro — com sampler de ponto, continua uma cor pura (sem
    // mistura), exatamente o problema de aliasing que motivou mips.
    assert!(
        (color_pixel_lod0[0] > 200 && color_pixel_lod0[2] < 50)
            || (color_pixel_lod0[2] > 200 && color_pixel_lod0[0] < 50),
        "LOD 0 deveria ser uma cor pura (vermelho ou azul), sem mistura — obtido {color_pixel_lod0:?}"
    );

    // Nível 1: cada texel do mip 1 é a média de dois texels do mip 0 (um
    // vermelho, um azul) — se `generate_mips` rodou de verdade, o
    // resultado é uma mistura, não mais uma cor pura.
    assert!(
        color_pixel_lod1[0] > 60
            && color_pixel_lod1[0] < 200
            && color_pixel_lod1[2] > 60
            && color_pixel_lod1[2] < 200,
        "LOD 1 deveria estar filtrado (mistura de vermelho/azul), obtido {color_pixel_lod1:?} — \
         generate_mips não preencheu a cadeia corretamente"
    );
}

#[test]
fn explicit_mip_chain_samples_the_exact_provided_level() {
    let device = Device::new(&DeviceDescriptor::default()).expect("falha ao criar device");

    // 2 níveis (4x4 e 2x2), cada um com uma cor **diferente** e sem
    // relação matemática com o outro — só uma cadeia explícita, gravada
    // por quem chama, pode produzir esse resultado (o GPU nunca
    // "inventaria" verde a partir de vermelho).
    let mip0: Vec<u8> = (0..4 * 4).flat_map(|_| [200u8, 0, 0, 255]).collect(); // vermelho
    let mip1: Vec<u8> = (0..2 * 2).flat_map(|_| [0u8, 200, 0, 255]).collect(); // verde

    let texture = device
        .create_texture_2d(
            &TextureDescriptor {
                width: 4,
                height: 4,
                format: Format::Rgba8Unorm,
                mip_levels: 2,
                array_layers: 1,
            },
            &[&mip0, &mip1],
        )
        .expect("falha ao criar textura com cadeia de mips explícita");
    assert_eq!(texture.mip_levels(), 2);

    let texture_view = device
        .create_shader_resource_view(&texture)
        .expect("falha ao criar shader resource view");

    let (level0, level1) =
        render_with_lod_scalar(&device, LOD_SHADER_SOURCE, &texture_view, 0.0, 1.0);

    assert!(
        level0[0] > 150 && level0[1] < 50,
        "LOD 0 deveria ser o vermelho fornecido explicitamente, obtido {level0:?}"
    );
    assert!(
        level1[1] > 150 && level1[0] < 50,
        "LOD 1 deveria ser o verde fornecido explicitamente, obtido {level1:?}"
    );
}

#[test]
fn texture_array_layers_sample_independently() {
    let device = Device::new(&DeviceDescriptor::default()).expect("falha ao criar device");

    let layer0: Vec<u8> = (0..4 * 4).flat_map(|_| [200u8, 0, 0, 255]).collect(); // vermelho
    let layer1: Vec<u8> = (0..4 * 4).flat_map(|_| [0u8, 0, 200, 255]).collect(); // azul

    let texture = device
        .create_texture_2d(
            &TextureDescriptor {
                width: 4,
                height: 4,
                format: Format::Rgba8Unorm,
                mip_levels: 1,
                array_layers: 2,
            },
            &[&layer0, &layer1],
        )
        .expect("falha ao criar textura em array");
    assert_eq!(texture.array_layers(), 2);

    let texture_view = device
        .create_shader_resource_view(&texture)
        .expect("falha ao criar shader resource view");

    let (layer0_pixel, layer1_pixel) =
        render_with_lod_scalar(&device, ARRAY_SHADER_SOURCE, &texture_view, 0.0, 1.0);

    assert!(
        layer0_pixel[0] > 150 && layer0_pixel[2] < 50,
        "camada 0 deveria ser vermelha, obtido {layer0_pixel:?}"
    );
    assert!(
        layer1_pixel[2] > 150 && layer1_pixel[0] < 50,
        "camada 1 deveria ser azul, obtido {layer1_pixel:?}"
    );
}

#[test]
fn create_texture_2d_rejects_too_many_mip_levels_for_the_extent() {
    let device = Device::new(&DeviceDescriptor {
        prefer_software: true,
        ..Default::default()
    })
    .expect("falha ao criar device WARP");

    // 4x4 só comporta 3 níveis (4, 2, 1) — pedir 5 é inválido.
    let result = device.create_texture_2d(
        &TextureDescriptor {
            width: 4,
            height: 4,
            format: Format::Rgba8Unorm,
            mip_levels: 5,
            array_layers: 1,
        },
        &[],
    );
    assert!(result.is_err());
}

#[test]
fn create_texture_2d_rejects_mip_data_count_mismatch() {
    let device = Device::new(&DeviceDescriptor {
        prefer_software: true,
        ..Default::default()
    })
    .expect("falha ao criar device WARP");

    let mip0 = vec![0u8; 4 * 4 * 4];
    // mip_levels: 2 exige 2 slices; só 1 foi fornecido.
    let result = device.create_texture_2d(
        &TextureDescriptor {
            width: 4,
            height: 4,
            format: Format::Rgba8Unorm,
            mip_levels: 2,
            array_layers: 1,
        },
        &[&mip0],
    );
    assert!(result.is_err());
}

#[test]
fn create_texture_2d_rejects_zero_array_layers() {
    let device = Device::new(&DeviceDescriptor {
        prefer_software: true,
        ..Default::default()
    })
    .expect("falha ao criar device WARP");

    let result = device.create_texture_2d(
        &TextureDescriptor {
            width: 4,
            height: 4,
            format: Format::Rgba8Unorm,
            mip_levels: 1,
            array_layers: 0,
        },
        &[],
    );
    assert!(result.is_err());
}

/// Desenha um quad fullscreen amostrando `texture_view` duas vezes (com
/// `scalar_a` e `scalar_b` no `register(b0)` do shader — LOD ou índice de
/// camada, dependendo de `shader_source`), capturando o pixel central de
/// um render target off-screen depois de cada desenho. Sem janela
/// nenhuma: usa `Device::create_render_target_texture` +
/// `Device::capture_texture`, como `tests/mrt.rs`.
fn render_with_lod_scalar(
    device: &Device,
    shader_source: &str,
    texture_view: &ndl_render::view::ShaderResourceView,
    scalar_a: f32,
    scalar_b: f32,
) -> ([u8; 4], [u8; 4]) {
    use ndl_render::blend::BlendStateDescriptor;
    use ndl_render::depth_stencil::DepthStencilStateDescriptor;
    use ndl_render::pipeline::GraphicsPipelineStateDescriptor;

    let (rt_texture, rtv) = device
        .create_render_target_texture(&TextureDescriptor {
            width: RT_SIZE,
            height: RT_SIZE,
            format: Format::Bgra8Unorm,
            mip_levels: 1,
            array_layers: 1,
        })
        .expect("falha ao criar render target");

    let vs_bytecode =
        compile_hlsl(shader_source, "vs_main", "vs_5_0").expect("falha ao compilar vertex shader");
    let ps_bytecode =
        compile_hlsl(shader_source, "ps_main", "ps_5_0").expect("falha ao compilar pixel shader");
    let vertex_shader = device
        .create_vertex_shader(&vs_bytecode)
        .expect("falha ao criar vertex shader");
    let pixel_shader = device
        .create_pixel_shader(&ps_bytecode)
        .expect("falha ao criar pixel shader");
    let attributes = quad_attributes();
    let input_layout = device
        .create_input_layout(&attributes, &vertex_shader)
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
                color_formats: &[Format::Bgra8Unorm],
                depth_format: None,
                stencil_reference: 0,
            },
        )
        .expect("falha ao criar pipeline");

    let vertices = fullscreen_quad_vertices();
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

    let sampler = device
        .create_sampler(&SamplerDescriptor::point_clamp())
        .expect("falha ao criar sampler");

    let scalar_buffer = device
        .create_buffer(&BufferDescriptor {
            size: 16,
            usage: BufferUsage::Constant,
            dynamic: false,
            stride: 0,
        })
        .expect("falha ao criar constant buffer");

    let context = device.context();
    let center = RT_SIZE / 2;

    let draw_and_capture = |scalar: f32| -> [u8; 4] {
        let mut cb_bytes = [0u8; 16];
        cb_bytes[0..4].copy_from_slice(&scalar.to_le_bytes());
        context
            .update_buffer(&scalar_buffer, &cb_bytes)
            .expect("falha ao atualizar o constant buffer");

        let colors = [ColorAttachment {
            view: &rtv,
            load: LoadOp::Clear([0.0, 0.0, 0.0, 1.0]),
        }];
        let mut pass = context
            .begin_render_pass(&RenderPassDescriptor {
                colors: &colors,
                depth: None,
            })
            .expect("falha ao iniciar render pass");
        pass.set_viewport(Viewport::full(RT_SIZE as f32, RT_SIZE as f32))
            .expect("invalid viewport");
        pass.set_pipeline(&pipeline).expect("pipeline incompatível");
        pass.set_vertex_buffer(0, &vertex_buffer, std::mem::size_of::<Vertex>() as u32)
            .expect("invalid vertex buffer binding");
        pass.set_constant_buffer(0, &scalar_buffer)
            .expect("invalid constant buffer binding");
        pass.set_shader_resource(0, texture_view)
            .expect("invalid shader resource slot");
        pass.set_sampler(0, &sampler).expect("invalid sampler slot");
        pass.draw(0..6).expect("falha ao desenhar o quad");
        pass.end();

        let capture = device
            .capture_texture(&rt_texture)
            .expect("falha ao capturar o render target");
        capture.pixel_rgba(center, center)
    };

    let pixel_a = draw_and_capture(scalar_a);
    let pixel_b = draw_and_capture(scalar_b);
    (pixel_a, pixel_b)
}
