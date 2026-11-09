//! Implementações específicas de cada backend gráfico.
//!
//! Este módulo é **inteiramente privado**: nada aqui é exposto na API
//! pública da crate, do mesmo jeito que `ndl-platform::backend`. Os
//! módulos públicos acima (`core`, `buffer`, `pipeline`, ...) vão
//! programar contra um contrato interno comum implementado por cada
//! backend concreto, nunca contra um backend específico.
//!
//! Hoje só `directx` existe, como reserva — nenhum backend está
//! implementado ainda. Planejado se estender depois a Vulkan, Metal e
//! OpenGL, seguindo o mesmo padrão de `ndl-platform` (um backend por
//! plataforma/API, atrás de uma única API pública consistente).

pub(crate) mod directx;
