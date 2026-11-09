//! Backend nativo do Windows, via `winit`.
//!
//! `winit` já abstrai a Win32 API (criação de janela, bomba de mensagens,
//! eventos de input) — esta camada só traduz os tipos e eventos do `winit`
//! para os tipos públicos do `ndl-platform`, mantendo a API da crate
//! independente de qual biblioteca está por baixo.

use std::rc::Rc;

use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::event::{
    ElementState, Ime as WinitIme, Modifiers as WinitModifiers, MouseButton as WinitMouseButton,
    MouseScrollDelta, Touch as WinitTouch, TouchPhase as WinitTouchPhase,
    WindowEvent as WinitWindowEvent,
};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode as WinitKeyCode, PhysicalKey};
use winit::window::{Window as WinitWindow, WindowAttributes, WindowId as WinitWindowId};

use crate::backend::Backend;
use crate::core::PlatformCapabilities;
use crate::error::PlatformError;
use crate::event::{AppEvent, Event, TouchPhase, WindowEvent};
use crate::input::{KeyCode, KeyState, Modifiers, MouseButton, MouseState};
use crate::monitor::MonitorInfo;
use crate::window::{
    LogicalSize, NativeWindowHandle, NativeWindowOps, RawWindowHandle, WindowDescriptor, WindowId,
};

impl NativeWindowOps for WinitWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window_handle()
            .expect("janela nativa sem handle válido")
            .as_raw()
    }

    fn request_redraw(&self) {
        WinitWindow::request_redraw(self);
    }
}

/// Única janela suportada pelo backend nativo por enquanto
/// (`PlatformCapabilities::multiple_windows == false`).
const SINGLE_WINDOW_ID: WindowId = WindowId::new(1);

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
            touch: true,
            drag_and_drop: true,
            ime: true,
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
        on_event: Box<dyn FnMut(Event<()>, &NativeWindowHandle)>,
    ) -> Result<(), PlatformError> {
        let event_loop =
            EventLoop::new().map_err(|err| PlatformError::Backend(err.to_string()))?;

        let mut app = App {
            descriptor,
            on_event,
            window: None,
            native_handle: None,
            winit_window_id: None,
            modifiers: Modifiers::NONE,
            started: false,
        };

        event_loop
            .run_app(&mut app)
            .map_err(|err| PlatformError::Backend(err.to_string()))
    }
}

struct App {
    descriptor: WindowDescriptor,
    on_event: Box<dyn FnMut(Event<()>, &NativeWindowHandle)>,
    window: Option<Rc<WinitWindow>>,
    /// Handle estável entregue a `on_event` — cacheado na criação da
    /// janela para não recomputar o raw handle a cada evento.
    native_handle: Option<NativeWindowHandle>,
    winit_window_id: Option<WinitWindowId>,
    /// Último estado conhecido dos modificadores (shift/ctrl/alt/super),
    /// atualizado por `WindowEvent::ModifiersChanged` e aplicado aos
    /// eventos de teclado/mouse subsequentes — `winit` não inclui os
    /// modificadores em cada evento individual.
    modifiers: Modifiers,
    started: bool,
}

impl App {
    fn emit(&mut self, event: WindowEvent) {
        let Some(handle) = self.native_handle.as_ref() else {
            return;
        };
        (self.on_event)(
            Event::Window {
                window_id: SINGLE_WINDOW_ID,
                event,
            },
            handle,
        );
    }

    fn emit_app_event(&mut self, event: AppEvent) {
        let Some(handle) = self.native_handle.as_ref() else {
            return;
        };
        (self.on_event)(Event::App(event), handle);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // `Wait`: o loop só roda de novo quando houver um evento real do SO
        // (input, resize, ...), em vez do padrão `Poll`, que giraria sem
        // parar gerando `RedrawRequested` infinitamente mesmo sem nada
        // para redesenhar.
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

        if self.window.is_some() {
            // Não é a primeira vez (ex: app voltou de segundo plano) —
            // notifica como evento de ciclo de vida, não recria a janela.
            self.emit_app_event(AppEvent::Resumed);
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
                let window = Rc::new(window);
                let ops: Rc<dyn NativeWindowOps> = window.clone();
                self.native_handle = Some(NativeWindowHandle::new(ops));
                self.window = Some(window);

                if !self.started {
                    self.started = true;
                    self.emit_app_event(AppEvent::Started);
                }
            }
            Err(err) => {
                eprintln!("ndl-platform: falha ao criar janela nativa: {err}");
                event_loop.exit();
            }
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.emit_app_event(AppEvent::Suspended);
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

        if let WinitWindowEvent::ModifiersChanged(modifiers) = &event {
            self.modifiers = map_modifiers(modifiers);
        }

        // winit já entrega, junto da tecla, o texto que ela produz
        // (considerando layout de teclado); repassamos isso como um
        // `TextInput` separado, além do `KeyboardInput` em si.
        if let WinitWindowEvent::KeyboardInput { event: key_event, .. } = &event {
            if key_event.state == ElementState::Pressed {
                if let Some(text) = &key_event.text {
                    self.emit(WindowEvent::TextInput {
                        text: text.to_string(),
                    });
                }
            }
        }

        let should_exit = matches!(event, WinitWindowEvent::CloseRequested);

        if let Some(mapped) = self.map_window_event(event) {
            self.emit(mapped);
        }

        if should_exit {
            event_loop.exit();
        }
    }
}

impl App {
    fn map_window_event(&self, event: WinitWindowEvent) -> Option<WindowEvent> {
        match event {
            WinitWindowEvent::CloseRequested => Some(WindowEvent::CloseRequested),
            WinitWindowEvent::Resized(size) => {
                Some(WindowEvent::Resized(LogicalSize::new(size.width, size.height)))
            }
            WinitWindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                Some(WindowEvent::ScaleFactorChanged { scale_factor })
            }
            WinitWindowEvent::Focused(focused) => Some(WindowEvent::Focused(focused)),
            WinitWindowEvent::RedrawRequested => Some(WindowEvent::RedrawRequested),

            WinitWindowEvent::CursorMoved { position, .. } => Some(WindowEvent::CursorMoved {
                position: (position.x, position.y),
            }),
            WinitWindowEvent::CursorEntered { .. } => Some(WindowEvent::CursorEntered),
            WinitWindowEvent::CursorLeft { .. } => Some(WindowEvent::CursorLeft),
            WinitWindowEvent::MouseWheel { delta, .. } => {
                let (delta_x, delta_y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x as f64, y as f64),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x, pos.y),
                };
                Some(WindowEvent::MouseWheel { delta_x, delta_y })
            }
            WinitWindowEvent::MouseInput { state, button, .. } => Some(WindowEvent::MouseInput {
                button: map_mouse_button(button),
                state: map_mouse_state(state),
                modifiers: self.modifiers,
            }),

            WinitWindowEvent::KeyboardInput { event, .. } => {
                let key = match event.physical_key {
                    PhysicalKey::Code(code) => map_key_code(code),
                    PhysicalKey::Unidentified(_) => KeyCode::Unidentified,
                };
                Some(WindowEvent::KeyboardInput {
                    key,
                    state: map_key_state(event.state),
                    modifiers: self.modifiers,
                })
            }

            WinitWindowEvent::Ime(ime) => match ime {
                WinitIme::Preedit(text, _) => Some(WindowEvent::Ime { preedit: text }),
                WinitIme::Commit(text) => Some(WindowEvent::TextInput { text }),
                WinitIme::Enabled | WinitIme::Disabled => None,
            },

            WinitWindowEvent::DroppedFile(path) => Some(WindowEvent::FileDropped { path }),
            WinitWindowEvent::HoveredFile(path) => Some(WindowEvent::FileHovered { path }),
            WinitWindowEvent::HoveredFileCancelled => Some(WindowEvent::FileHoverCancelled),

            WinitWindowEvent::Touch(WinitTouch { id, phase, location, .. }) => {
                Some(WindowEvent::Touch {
                    id,
                    phase: map_touch_phase(phase),
                    position: (location.x, location.y),
                })
            }

            _ => None,
        }
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

fn map_touch_phase(phase: WinitTouchPhase) -> TouchPhase {
    match phase {
        WinitTouchPhase::Started => TouchPhase::Started,
        WinitTouchPhase::Moved => TouchPhase::Moved,
        WinitTouchPhase::Ended => TouchPhase::Ended,
        WinitTouchPhase::Cancelled => TouchPhase::Cancelled,
    }
}

fn map_modifiers(modifiers: &WinitModifiers) -> Modifiers {
    let state = modifiers.state();
    Modifiers {
        shift: state.shift_key(),
        ctrl: state.control_key(),
        alt: state.alt_key(),
        super_key: state.super_key(),
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
