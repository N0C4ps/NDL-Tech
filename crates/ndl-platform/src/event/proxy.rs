use std::fmt;
use std::sync::mpsc::{SendError, Sender};

/// Erro retornado ao enviar um evento customizado para um [`super::EventLoop`]
/// que já foi encerrado.
#[derive(Debug, Clone, PartialEq)]
pub struct EventLoopClosed;

impl fmt::Display for EventLoopClosed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "o event loop já foi encerrado")
    }
}

impl std::error::Error for EventLoopClosed {}

/// Permite enviar eventos customizados (`T`) para o [`super::EventLoop`] de
/// fora do loop principal — por exemplo, de outra thread (um worker de
/// rede ou de I/O que precisa acordar o loop de eventos).
#[derive(Clone)]
pub struct EventLoopProxy<T: Send + 'static> {
    sender: Sender<T>,
}

impl<T: Send + 'static> EventLoopProxy<T> {
    pub(crate) fn new(sender: Sender<T>) -> Self {
        Self { sender }
    }

    pub fn send_event(&self, event: T) -> Result<(), EventLoopClosed> {
        self.sender
            .send(event)
            .map_err(|SendError(_)| EventLoopClosed)
    }
}
