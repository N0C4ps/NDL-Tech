//! Cópias entre recursos — operações que não envolvem desenho de geometria.
//!
//! ## Estado atual
//!
//! [`crate::context::Context::copy_buffer`] e
//! [`crate::context::Context::copy_texture`] já são reais (cópia do
//! recurso inteiro via `CopyResource` — os dois precisam ter a mesma
//! descrição: tamanho para buffers, dimensões/formato para texturas).
//! [`Device::capture_texture`] lê de volta os pixels de qualquer textura
//! (mesmo caminho usado por `Swapchain::capture_frame`, generalizado) —
//! útil para validar um attachment de cor extra de MRT sem precisar
//! apresentá-lo numa swapchain. Cópias parciais (`copy_buffer_region`,
//! `copy_texture_region`, `copy_subresource_region`), resolução de MSAA
//! (`resolve_texture`) e geração de mipmaps (`generate_mips`) continuam
//! reservadas.

use crate::backend::{ActiveBackend, Backend};
use crate::buffer::Buffer;
use crate::context::Context;
use crate::core::{Device, GraphicsError};
use crate::surface::FrameCapture;
use crate::texture::Texture2D;

impl Context {
    /// Copia todo o conteúdo de `src` para `dst`. Os dois buffers
    /// precisam ter o mesmo tamanho.
    pub fn copy_buffer(&self, src: &Buffer, dst: &Buffer) {
        ActiveBackend::copy_buffer(self, src, dst);
    }

    /// Copia todo o conteúdo de `src` para `dst`. As duas texturas
    /// precisam ter as mesmas dimensões e formato.
    pub fn copy_texture(&self, src: &Texture2D, dst: &Texture2D) {
        ActiveBackend::copy_texture(self, src, dst);
    }
}

impl Device {
    /// Lê de volta o conteúdo atual de `texture`, como pixels crus — o
    /// mesmo caminho de [`crate::surface::Swapchain::capture_frame`], mas
    /// para qualquer textura (ex: um attachment de cor extra de MRT, ou um
    /// alvo de post-processing), não só o back buffer de uma swapchain.
    /// Para ler um attachment que a GPU acabou de escrever, chame depois
    /// do [`crate::render::RenderPass`] que o preencheu ter terminado
    /// (`RenderPass::end`).
    pub fn capture_texture(&self, texture: &Texture2D) -> Result<FrameCapture, GraphicsError> {
        let (width, height, format, pixels) = ActiveBackend::capture_texture(self, texture)?;
        Ok(FrameCapture::new(width, height, format, pixels))
    }
}
