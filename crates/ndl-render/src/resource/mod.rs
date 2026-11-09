//! Recursos e suas descrições.
//!
//! Conceitos compartilhados por todos os tipos de recurso de GPU
//! (`buffer`, `texture`, ...). Os tipos concretos vivem em seus próprios
//! módulos; este módulo reúne o que é comum entre eles.
//!
//! ## Estado atual
//!
//! `set_label` já é real em [`crate::buffer::Buffer`] e
//! [`crate::texture::Texture2D`] — dá um nome ao recurso nativo, visível
//! em ferramentas de debug gráfico (RenderDoc, PIX, a debug layer do
//! D3D11). `resource()`, `descriptor()` e um trait comum unificando os
//! tipos de recurso continuam reservados.

use crate::backend::{ActiveBackend, Backend};
use crate::buffer::Buffer;
use crate::texture::Texture2D;

impl Buffer {
    /// Dá um nome a este buffer, visível em ferramentas de debug gráfico.
    pub fn set_label(&self, label: &str) {
        ActiveBackend::set_buffer_label(self, label);
    }
}

impl Texture2D {
    /// Dá um nome a esta textura, visível em ferramentas de debug gráfico.
    pub fn set_label(&self, label: &str) {
        ActiveBackend::set_texture_label(self, label);
    }
}
