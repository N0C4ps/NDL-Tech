//! Backend nativo do Windows, via `winit`.
//!
//! `winit` já abstrai a Win32 API (criação de janela, bomba de mensagens,
//! eventos de input) — esta camada só traduz os tipos e eventos do `winit`
//! para os tipos públicos do `ndl-platform`, mantendo a API da crate
//! independente de qual biblioteca está por baixo.

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent as WinitWindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode as WinitKeyCode, PhysicalKey};
use winit::window::{Window as WinitWindow, WindowAttributes, WindowId as WinitWindowId};

use crate::backend::Backend;
use crate::core::PlatformCapabilities;
use crate::error::PlatformError;
use crate::event::WindowEvent;
use crate::input::{KeyCode, KeyState, Modifiers, MouseButton, MouseState};
use crate::monitor::MonitorInfo;
use crate::window::{LogicalSize, WindowDescriptor, WindowId};

pub(crate) struct WindowsBackend;

impl WindowsBackend {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl Backend for WindowsBackend {
    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            multiple_windows: false,
            ..PlatformCapabilities::NONE
        }
    }

    fn monitors(&self) -> Vec<MonitorInfo> {
        // TODO: enumerar monitores reais via winit quando houver um
        // ActiveEventLoop disponível fora de `run`.
        Vec::new()
    }

    fn primary_monitor(&self) -> Option<MonitorInfo> {
        None
    }

    fn create_window(&self, _descriptor: &WindowDescriptor) -> Result<WindowId, PlatformError> {
        Err(PlatformError::Unsupported(
            "o backend nativo do Windows não cria janelas \"leves\" fora de um loop; use Platform::run_window",
        ))
    }

    fn destroy_window(&self, _id: WindowId) {}

    fn run(
        &self,
        descriptor: WindowDescriptor,
        on_event: Box<dyn FnMut(WindowEvent)>,
    ) -> Result<(), PlatformError> {
        let event_loop =
            EventLoop::new().map_err(|err| PlatformError::Backend(err.to_string()))?;

        let mut app = App {
            descriptor,
            on_event,
            window: None,
            winit_window_id: None,
        };

        event_loop
            .run_app(&mut app)
            .map_err(|err| PlatformError::Backend(err.to_string()))
    }
}

struct App {
    descriptor: WindowDescriptor,
    on_event: Box<dyn FnMut(WindowEvent)>,
    window: Option<WinitWindow>,
    winit_window_id: Option<WinitWindowId>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title(self.descriptor.title.clone())
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.descriptor.size.width as f64,
                self.descriptor.size.height as f64,
            ))
            .with_resizable(self.descriptor.resizable)
            .with_decorations(self.descriptor.decorated)
            .with_transparent(self.descriptor.transparent)
            .with_visible(self.descriptor.visible)
            .with_maximized(self.descriptor.maximized);

        match event_loop.create_window(attrs) {
            Ok(window) => {
                self.winit_window_id = Some(window.id());
                self.window = Some(window);
            }
            Err(err) => {
                eprintln!("ndl-platform: falha ao criar janela nativa: {err}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WinitWindowId,
        event: WinitWindowEvent,
    ) {
        if Some(window_id) != self.winit_window_id {
            return;
        }

        let should_exit = matches!(event, WinitWindowEvent::CloseRequested);
        let should_redraw = matches!(event, WinitWindowEvent::RedrawRequested);

        if let Some(mapped) = map_window_event(&event) {
            (self.on_event)(mapped);
        }

        if should_redraw {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }

        if should_exit {
            event_loop.exit();
        }
    }
}

fn map_window_event(event: &WinitWindowEvent) -> Option<WindowEvent> {
    match event {
        WinitWindowEvent::CloseRequested => Some(WindowEvent::CloseRequested),
        WinitWindowEvent::Resized(size) => {
            Some(WindowEvent::Resized(LogicalSize::new(size.width, size.height)))
        }
        WinitWindowEvent::Focused(focused) => Some(WindowEvent::Focused(*focused)),
        WinitWindowEvent::RedrawRequested => Some(WindowEvent::RedrawRequested),
        WinitWindowEvent::CursorMoved { position, .. } => Some(WindowEvent::CursorMoved {
            position: (position.x, position.y),
        }),
        WinitWindowEvent::CursorEntered { .. } => Some(WindowEvent::CursorEntered),
        WinitWindowEvent::CursorLeft { .. } => Some(WindowEvent::CursorLeft),
        WinitWindowEvent::MouseInput { state, button, .. } => Some(WindowEvent::MouseInput {
            button: map_mouse_button(*button),
            state: map_mouse_state(*state),
            modifiers: Modifiers::NONE,
        }),
        WinitWindowEvent::KeyboardInput { event, .. } => {
            let key = match event.physical_key {
                PhysicalKey::Code(code) => map_key_code(code),
                PhysicalKey::Unidentified(_) => KeyCode::Unidentified,
            };
            Some(WindowEvent::KeyboardInput {
                key,
                state: map_key_state(event.state),
                modifiers: Modifiers::NONE,
            })
        }
        _ => None,
    }
}

fn map_mouse_button(button: WinitMouseButton) -> MouseButton {
    match button {
        WinitMouseButton::Left => MouseButton::Left,
        WinitMouseButton::Right => MouseButton::Right,
        WinitMouseButton::Middle => MouseButton::Middle,
        WinitMouseButton::Back => MouseButton::Back,
        WinitMouseButton::Forward => MouseButton::Forward,
        WinitMouseButton::Other(id) => MouseButton::Other(id),
    }
}

fn map_mouse_state(state: ElementState) -> MouseState {
    match state {
        ElementState::Pressed => MouseState::Pressed,
        ElementState::Released => MouseState::Released,
    }
}

fn map_key_state(state: ElementState) -> KeyState {
    match state {
        ElementState::Pressed => KeyState::Pressed,
        ElementState::Released => KeyState::Released,
    }
}

fn map_key_code(code: WinitKeyCode) -> KeyCode {
    use WinitKeyCode::*;
    match code {
        KeyA => KeyCode::KeyA,
        KeyB => KeyCode::KeyB,
        KeyC => KeyCode::KeyC,
        KeyD => KeyCode::KeyD,
        KeyE => KeyCode::KeyE,
        KeyF => KeyCode::KeyF,
        KeyG => KeyCode::KeyG,
        KeyH => KeyCode::KeyH,
        KeyI => KeyCode::KeyI,
        KeyJ => KeyCode::KeyJ,
        KeyK => KeyCode::KeyK,
        KeyL => KeyCode::KeyL,
        KeyM => KeyCode::KeyM,
        KeyN => KeyCode::KeyN,
        KeyO => KeyCode::KeyO,
        KeyP => KeyCode::KeyP,
        KeyQ => KeyCode::KeyQ,
        KeyR => KeyCode::KeyR,
        KeyS => KeyCode::KeyS,
        KeyT => KeyCode::KeyT,
        KeyU => KeyCode::KeyU,
        KeyV => KeyCode::KeyV,
        KeyW => KeyCode::KeyW,
        KeyX => KeyCode::KeyX,
        KeyY => KeyCode::KeyY,
        KeyZ => KeyCode::KeyZ,
        Digit0 => KeyCode::Digit0,
        Digit1 => KeyCode::Digit1,
        Digit2 => KeyCode::Digit2,
        Digit3 => KeyCode::Digit3,
        Digit4 => KeyCode::Digit4,
        Digit5 => KeyCode::Digit5,
        Digit6 => KeyCode::Digit6,
        Digit7 => KeyCode::Digit7,
        Digit8 => KeyCode::Digit8,
        Digit9 => KeyCode::Digit9,
        ArrowUp => KeyCode::ArrowUp,
        ArrowDown => KeyCode::ArrowDown,
        ArrowLeft => KeyCode::ArrowLeft,
        ArrowRight => KeyCode::ArrowRight,
        Space => KeyCode::Space,
        Enter => KeyCode::Enter,
        Escape => KeyCode::Escape,
        Tab => KeyCode::Tab,
        Backspace => KeyCode::Backspace,
        Delete => KeyCode::Delete,
        ShiftLeft => KeyCode::ShiftLeft,
        ShiftRight => KeyCode::ShiftRight,
        ControlLeft => KeyCode::ControlLeft,
        ControlRight => KeyCode::ControlRight,
        AltLeft => KeyCode::AltLeft,
        AltRight => KeyCode::AltRight,
        SuperLeft => KeyCode::SuperLeft,
        SuperRight => KeyCode::SuperRight,
        F1 => KeyCode::F1,
        F2 => KeyCode::F2,
        F3 => KeyCode::F3,
        F4 => KeyCode::F4,
        F5 => KeyCode::F5,
        F6 => KeyCode::F6,
        F7 => KeyCode::F7,
        F8 => KeyCode::F8,
        F9 => KeyCode::F9,
        F10 => KeyCode::F10,
        F11 => KeyCode::F11,
        F12 => KeyCode::F12,
        _ => KeyCode::Unidentified,
    }
}
