//! Diagnóstico e instrumentação de renderização.
//!
//! **Não confundir com `ndl-diagnostics`** (telemetria/profiling genérico
//! do ecossistema) — este módulo é especificamente sobre depurar o
//! backend gráfico (debug layer, debug markers, mensagens de validação).
//!
//! ## Planejado
//!
//! Métodos: `enable_debug_layer()`, `report_live_objects()`,
//! `set_debug_name()`, `get_debug_name()`, `push_debug_group()`,
//! `pop_debug_group()`, `insert_debug_marker()`, `get_debug_messages()`,
//! `clear_debug_messages()`, `set_message_filter()`,
//! `set_message_break_severity()`, `set_message_break_id()`,
//! `get_device_removed_reason()`.
//!
//! Tipos: `DebugSeverity`, `DebugCategory`, `DebugMessage`, `DebugFilter`,
//! `DebugGroup`, `DebugDevice`, `InfoQueue`.
//!
//! `capture_frame()`/`begin_capture()` podem existir como integração
//! opcional com ferramentas externas (RenderDoc, PIX) quando a ferramenta
//! e a API de captura permitirem — não são operações fundamentais do
//! D3D11.
//!
//! Ainda não implementado.
