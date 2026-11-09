use std::collections::VecDeque;

use super::event::Event;

/// Fila FIFO interna de eventos, usada pelo [`super::EventLoop`] e
/// alimentada pelo backend de plataforma (ou, no backend headless, por
/// quem estiver simulando eventos em testes).
#[derive(Debug, Default)]
pub(crate) struct EventQueue<T> {
    events: VecDeque<Event<T>>,
}

impl<T> EventQueue<T> {
    pub(crate) fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }

    pub(crate) fn push(&mut self, event: Event<T>) {
        self.events.push_back(event);
    }

    pub(crate) fn pop(&mut self) -> Option<Event<T>> {
        self.events.pop_front()
    }

    pub(crate) fn len(&self) -> usize {
        self.events.len()
    }
}
