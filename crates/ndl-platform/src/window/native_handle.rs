use std::rc::Rc;

pub use raw_window_handle::RawWindowHandle;

/// Operações que um backend precisa fornecer para que sua janela nativa
/// possa ser exposta como [`NativeWindowHandle`] — implementado pelo tipo
/// de janela concreto de cada backend (ex: `winit::window::Window`).
pub(crate) trait NativeWindowOps {
    fn raw_window_handle(&self) -> RawWindowHandle;
    fn request_redraw(&self);
}

/// Handle de uma janela nativa aberta via [`crate::core::Platform::run_window`],
/// entregue ao callback de eventos.
///
/// Expõe um [`RawWindowHandle`] genérico (da crate `raw-window-handle`),
/// não um tipo de `ndl-render` ou de qualquer outra lib gráfica —
/// `ndl-platform` não conhece essas crates. É o código do jogo que recebe
/// este handle e decide o que fazer com ele (por exemplo, entregá-lo a
/// `ndl-render::surface::Surface::from_window_handle`).
#[derive(Clone)]
pub struct NativeWindowHandle {
    ops: Rc<dyn NativeWindowOps>,
}

impl NativeWindowHandle {
    pub(crate) fn new(ops: Rc<dyn NativeWindowOps>) -> Self {
        Self { ops }
    }

    pub fn raw_window_handle(&self) -> RawWindowHandle {
        self.ops.raw_window_handle()
    }

    /// Pede ao SO para emitir outro `WindowEvent::RedrawRequested` em
    /// breve. Sem isso, depois do primeiro redraw o loop (que usa
    /// `ControlFlow::Wait`) só desperta de novo com input real — um jogo
    /// que precisa renderizar continuamente chama isso a cada quadro,
    /// tipicamente dentro do próprio handler de `RedrawRequested`.
    pub fn request_redraw(&self) {
        self.ops.request_redraw();
    }
}
