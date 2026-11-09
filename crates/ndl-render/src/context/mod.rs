//! Contexto imediato e diferido.
//!
//! No modelo do Direct3D 11, o contexto imediato executa comandos
//! diretamente; contextos diferidos gravam listas de comandos executadas
//! depois pelo contexto imediato. Este módulo abstrai essa distinção sem
//! prometer o modelo de filas explícitas de APIs mais modernas.
//!
//! ## Planejado
//!
//! | Grupo | Métodos sugeridos |
//! |---|---|
//! | Estado global | `clear_state`, `flush`, `context_type` |
//! | Pipeline | `set_pipeline`, `set_compute_pipeline`, `unbind_pipeline` |
//! | Vertex input | `set_vertex_buffer(s)`, `set_index_buffer`, `set_input_layout`, `set_primitive_topology` |
//! | Shader | `set_vertex_shader`, `set_pixel_shader`, `set_geometry_shader`, `set_hull_shader`, `set_domain_shader`, `set_compute_shader` |
//! | Recursos | `set_constant_buffer`, `set_shader_resource`, `set_sampler`, `set_unordered_access_view` |
//! | Output merger | `set_render_targets`, `set_depth_stencil_target`, `set_blend_state`, `set_depth_stencil_state` |
//! | Rasterizer | `set_rasterizer_state`, `set_viewport(s)`, `set_scissor(s)` |
//! | Stream output | `set_stream_output_targets`, `reset_stream_output_targets` |
//! | Consultas | `begin_query`, `end_query`, `get_query_data`, `set_predication` |
//! | Inspeção | `get_pipeline_state`, `get_bound_resources`, `get_render_targets`, `get_viewports` |
//!
//! Ainda não implementado.
