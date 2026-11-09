//! Compute shader de verdade: dobra cada elemento de um structured buffer
//! na GPU, copia para um buffer de staging e lê de volta — diferente dos
//! outros testes (que só verificam "não panicou"), este confere os
//! valores calculados de verdade, provando que o compute shader rodou e
//! fez a conta certa.
#![cfg(target_os = "windows")]

use ndl_render::buffer::{BufferDescriptor, BufferUsage};
use ndl_render::core::{Device, DeviceDescriptor};
use ndl_render::pipeline::ComputePipelineDescriptor;
use ndl_render::shader::compile_hlsl;

const SHADER_SOURCE: &str = r#"
RWStructuredBuffer<float> data : register(u0);

[numthreads(8, 1, 1)]
void cs_main(uint3 id : SV_DispatchThreadID) {
    data[id.x] = data[id.x] * 2.0;
}
"#;

#[test]
fn compute_shader_doubles_each_element() {
    let device = Device::new(&DeviceDescriptor::default()).expect("falha ao criar device");

    let input: [f32; 8] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let input_bytes = unsafe {
        std::slice::from_raw_parts(input.as_ptr() as *const u8, std::mem::size_of_val(&input))
    };

    let storage_buffer = device
        .create_buffer_with_data(
            &BufferDescriptor {
                size: input_bytes.len() as u64,
                usage: BufferUsage::Storage,
                dynamic: false,
                stride: std::mem::size_of::<f32>() as u32,
            },
            input_bytes,
        )
        .expect("falha ao criar o structured buffer");

    let uav = device
        .create_unordered_access_view(&storage_buffer)
        .expect("falha ao criar a UAV");

    let cs_bytecode = compile_hlsl(SHADER_SOURCE, "cs_main", "cs_5_0")
        .expect("falha ao compilar o compute shader");
    let compute_shader = device
        .create_compute_shader(&cs_bytecode)
        .expect("falha ao criar o compute shader");
    let pipeline = device
        .create_compute_pipeline(&ComputePipelineDescriptor {
            compute_shader: &compute_shader,
        })
        .expect("falha ao criar o compute pipeline");

    let context = device.context();
    context.set_compute_pipeline(&pipeline);
    context.set_unordered_access_view(0, &uav);
    context.dispatch(1, 1, 1);

    let staging_buffer = device
        .create_buffer(&BufferDescriptor {
            size: input_bytes.len() as u64,
            usage: BufferUsage::Staging,
            dynamic: false,
            stride: 0,
        })
        .expect("falha ao criar o buffer de staging");
    context.copy_buffer(&storage_buffer, &staging_buffer);

    let mapped = context
        .map_read(&staging_buffer)
        .expect("falha ao mapear o buffer de staging para leitura");
    let bytes = mapped.as_slice();
    let result: &[f32] = unsafe {
        std::slice::from_raw_parts(bytes.as_ptr() as *const f32, input.len())
    };

    let expected: Vec<f32> = input.iter().map(|x| x * 2.0).collect();
    assert_eq!(
        result, expected.as_slice(),
        "o compute shader deveria ter dobrado cada elemento"
    );
}
