//! Compilação, reflexão e estágios programáveis de shader.
//!
//! O Direct3D 11 tem seis estágios principais: vertex, hull, domain,
//! geometry, pixel e compute.
//!
//! ## Planejado
//!
//! | Estágio | Tipo Rust | Finalidade |
//! |---|---|---|
//! | Vertex | `VertexShader` | Processamento de vértices |
//! | Hull | `HullShader` | Controle de tesselação |
//! | Domain | `DomainShader` | Avaliação da tesselação |
//! | Geometry | `GeometryShader` | Processamento de primitivas |
//! | Pixel | `PixelShader` | Fragmentos/pixels |
//! | Compute | `ComputeShader` | Computação geral na GPU |
//!
//! Criação e compilação: `compile_hlsl()`, `compile_hlsl_file()`,
//! `compile_shader_source()`, `create_shader_module()`,
//! `create_vertex_shader()`, `create_pixel_shader()`,
//! `create_geometry_shader()`,
//! `create_geometry_shader_with_stream_output()`, `create_hull_shader()`,
//! `create_domain_shader()`, `create_compute_shader()`,
//! `load_shader_bytecode()`, `validate_shader_bytecode()`.
//!
//! Reflexão e metadados: `reflect_shader()`, `get_entry_point()`,
//! `get_shader_stage()`, `get_shader_model()`, `get_input_signature()`,
//! `get_output_signature()`, `get_resource_bindings()`,
//! `get_constant_buffers()`, `get_constant_buffer_variables()`,
//! `get_thread_group_size()`, `get_shader_bytecode()`,
//! `get_shader_hash()`, `get_compilation_diagnostics()`.
//!
//! Binding por estágio: cada estágio oferece algo equivalente a
//! `set_shader()`, `unbind_shader()`, `set_constant_buffer(s)()`,
//! `set_shader_resource(s)()`, `set_sampler(s)()`; compute (e, quando
//! suportado, outros estágios) também `set_unordered_access_view(s)()`.
//!
//! Enums planejados: `ShaderStage`, `ShaderStages` (bitflags),
//! `ShaderModel`, `ShaderCompilationFlags`, `ShaderOptimizationLevel`,
//! `ShaderResourceType`, `ShaderVariableType`, `ShaderInputType`.
//!
//! Para HLSL, um submódulo de compilação separado (bytecode já compilado
//! vs. compilação em tempo de desenvolvimento), e um `ShaderCache`
//! (`get()`, `insert()`, `invalidate()`, `clear()`, `reload()`) como
//! conveniência da crate, não funcionalidade nativa de shaders.
//!
//! Ainda não implementado.
