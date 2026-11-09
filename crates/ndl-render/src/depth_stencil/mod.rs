//! Teste e escrita de profundidade e stencil.
//!
//! O D3D11 usa objetos de estado imutáveis — métodos como `set_depth_test()`
//! são implementados como modificação de um descriptor seguida da seleção
//! de um novo estado, não mutação direta de um estado existente.
//!
//! ## Planejado
//!
//! Métodos: `create_depth_stencil_state()`, `set_depth_stencil_state()`,
//! `get_depth_stencil_state()`, `set_depth_test()`, `set_depth_write()`,
//! `set_depth_compare()`, `set_stencil_test()`, `set_stencil_reference()`,
//! `set_stencil_read_mask()`, `set_stencil_write_mask()`,
//! `set_stencil_front_face()`, `set_stencil_back_face()`,
//! `clear_depth()`, `clear_stencil()`, `clear_depth_stencil()`.
//!
//! `CompareOp`: Never, Less, Equal, LessEqual, Greater, NotEqual,
//! GreaterEqual, Always.
//!
//! `StencilOp`: Keep, Zero, Replace, IncrementClamp, DecrementClamp,
//! Invert, IncrementWrap, DecrementWrap.
//!
//! Presets planejados: `DepthStencilState::disabled()`,
//! `depth_read_write()`, `depth_read_only()`, `reverse_z()`,
//! `stencil_write()`, `stencil_test_equal()`.
//!
//! Ainda não implementado.
