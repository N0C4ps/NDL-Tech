//! Input assembler: input layout e topologia de vértices.
//!
//! **Não confundir com `ndl-platform::input`** (teclado/mouse) — este é o
//! estágio de montagem de vértices da GPU, totalmente sem relação com
//! input de dispositivos.
//!
//! ## Planejado
//!
//! Métodos: `create_input_layout()`, `set_input_layout()`,
//! `set_vertex_buffer(s)()`, `set_index_buffer()`,
//! `set_primitive_topology()`, `get_input_layout()`,
//! `get_vertex_buffers()`, `get_index_buffer()`,
//! `get_primitive_topology()`.
//!
//! Topologias planejadas:
//!
//! ```ignore
//! pub enum PrimitiveTopology {
//!     PointList,
//!     LineList,
//!     LineStrip,
//!     TriangleList,
//!     TriangleStrip,
//!     LineListAdjacency,
//!     LineStripAdjacency,
//!     TriangleListAdjacency,
//!     TriangleStripAdjacency,
//!     PatchList(u8), // 1 a 32 pontos de controle
//! }
//! ```
//!
//! Layout de vértices — tipos: `VertexAttribute`, `VertexBufferLayout`,
//! `VertexFormat`, `VertexStepMode`, `IndexFormat`, `InputClassification`.
//! Métodos de builder: `add_attribute()`, `set_stride()`, `set_slot()`,
//! `set_format()`, `set_offset()`, `set_step_mode()`,
//! `set_instance_step_rate()`, `build()`.
//!
//! Ainda não implementado.
