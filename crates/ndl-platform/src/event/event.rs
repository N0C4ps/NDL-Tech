use std::path::PathBuf;

use crate::input::{KeyCode, KeyState, Modifiers, MouseButton, MouseState};
use crate::window::{LogicalSize, PhysicalPosition, WindowId};

/// Identificador opaco de um dispositivo de input (gamepad, caneta, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceId(pub(crate) u32);

/// Fase de um toque (touchscreen).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

/// Eventos relativos a uma janela específica.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    Resized(LogicalSize<u32>),
    Moved(PhysicalPosition<i32>),
    ScaleFactorChanged { scale_factor: f64 },
    RedrawRequested,
    CloseRequested,
    Focused(bool),

    KeyboardInput {
        key: KeyCode,
        state: KeyState,
        modifiers: Modifiers,
    },
    TextInput {
        text: String,
    },
    Ime {
        preedit: String,
    },

    MouseInput {
        button: MouseButton,
        state: MouseState,
        modifiers: Modifiers,
    },
    CursorMoved {
        position: (f64, f64),
    },
    CursorEntered,
    CursorLeft,
    MouseWheel {
        delta_x: f64,
        delta_y: f64,
    },

    FileDropped {
        path: PathBuf,
    },
    FileHovered {
        path: PathBuf,
    },
    FileHoverCancelled,

    Touch {
        id: u64,
        phase: TouchPhase,
        position: (f64, f64),
    },
    Pen {
        position: (f64, f64),
        pressure: f32,
    },
}

/// Eventos de ciclo de vida da aplicação como um todo (não de uma janela
/// específica).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEvent {
    Started,
    Resumed,
    Suspended,
    ExitRequested,
    LowMemory,
}

/// Eventos de conexão/desconexão de dispositivos de input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceEvent {
    Connected,
    Disconnected,
}

/// Qualquer evento que pode ser emitido pela plataforma.
///
/// `T` é o tipo de evento customizado do jogo, enviado via
/// [`crate::event::EventLoopProxy`]; jogos que não precisam de eventos
/// próprios podem usar `Event<()>`.
#[derive(Debug, Clone, PartialEq)]
pub enum Event<T = ()> {
    Window { window_id: WindowId, event: WindowEvent },
    Device { device_id: DeviceId, event: DeviceEvent },
    App(AppEvent),
    User(T),
}
