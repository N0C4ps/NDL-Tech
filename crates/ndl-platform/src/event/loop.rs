use std::sync::mpsc::{self, Receiver};

use super::event::Event;
use super::proxy::EventLoopProxy;
use super::queue::EventQueue;

/// Laço de eventos da aplicação.
///
/// `T` é o tipo de evento customizado do jogo (veja [`EventLoopProxy`]);
/// jogos sem eventos próprios usam `EventLoop<()>` (o padrão).
///
/// O `EventLoop` não decide como o jogo deve rodar — ele só entrega
/// eventos. O padrão mais comum é chamar [`EventLoop::poll_event`] dentro
/// do próprio game loop do jogo.
pub struct EventLoop<T = ()> {
    queue: EventQueue<T>,
    user_sender: mpsc::Sender<T>,
    user_receiver: Receiver<T>,
}

impl<T> Default for EventLoop<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> EventLoop<T> {
    pub fn new() -> Self {
        let (user_sender, user_receiver) = mpsc::channel();
        Self {
            queue: EventQueue::new(),
            user_sender,
            user_receiver,
        }
    }

    /// Cria um proxy que permite enviar eventos customizados para este loop
    /// a partir de outra thread.
    pub fn create_proxy(&self) -> EventLoopProxy<T>
    where
        T: Send + 'static,
    {
        EventLoopProxy::new(self.user_sender.clone())
    }

    /// Enfileira um evento (de janela, dispositivo ou aplicação) para ser
    /// entregue na próxima chamada de [`EventLoop::poll_event`].
    ///
    /// Usado internamente pelo backend de plataforma; também útil em testes
    /// com o backend `headless` para simular eventos.
    pub fn push_event(&mut self, event: Event<T>) {
        self.queue.push(event);
    }

    /// Retorna o próximo evento pendente, se houver, sem bloquear.
    pub fn poll_event(&mut self) -> Option<Event<T>> {
        if let Some(event) = self.queue.pop() {
            return Some(event);
        }
        match self.user_receiver.try_recv() {
            Ok(user_event) => Some(Event::User(user_event)),
            Err(_) => None,
        }
    }

    /// Número de eventos de janela/dispositivo/app pendentes na fila
    /// interna (não conta eventos customizados ainda não recebidos).
    pub fn pending_events(&self) -> usize {
        self.queue.len()
    }
}
