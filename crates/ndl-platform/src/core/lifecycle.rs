/// Estado de ciclo de vida da aplicação dentro da plataforma.
///
/// Transições entre estados são notificadas via
/// [`crate::event::AppEvent`]; este tipo representa o estado atual, útil
/// para consulta direta sem precisar acompanhar o fluxo de eventos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationState {
    /// A plataforma foi inicializada mas a aplicação ainda não começou a rodar.
    Idle,
    /// A aplicação está em primeiro plano e rodando normalmente.
    Running,
    /// A aplicação foi suspensa pelo sistema operacional (ex: minimizada em mobile).
    Suspended,
    /// A aplicação recebeu um pedido de encerramento e está finalizando.
    Exiting,
    /// A aplicação terminou de executar.
    Exited,
}
