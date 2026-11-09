//! Sistema de eventos: suporta múltiplas janelas, múltiplos dispositivos e
//! eventos internos da aplicação.
//!
//! A plataforma **possui** todos esses tipos de evento, mas isso não
//! significa que o jogo precisa tratar todos eles — observar só o
//! subconjunto relevante (ex: só `KeyboardInput` e `CloseRequested`) é
//! escolha de quem programa o jogo.

mod event;
#[path = "loop.rs"]
mod event_loop;
mod proxy;
mod queue;

pub use event::{AppEvent, DeviceEvent, DeviceId, Event, TouchPhase, WindowEvent};
pub use event_loop::EventLoop;
pub use proxy::{EventLoopClosed, EventLoopProxy};
