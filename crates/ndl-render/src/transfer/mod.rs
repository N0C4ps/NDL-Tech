//! Cópias entre recursos — operações que não envolvem desenho de geometria.
//!
//! ## Estado atual
//!
//! [`crate::context::Context::copy_buffer`] e
//! [`crate::context::Context::copy_texture`] já são reais (cópia do
//! recurso inteiro via `CopyResource` — os dois precisam ter a mesma
//! descrição: tamanho para buffers, dimensões/formato para texturas).
//! Cópias parciais (`copy_buffer_region`, `copy_texture_region`,
//! `copy_subresource_region`), resolução de MSAA (`resolve_texture`) e
//! geração de mipmaps (`generate_mips`) continuam reservadas.

use crate::buffer::Buffer;
use crate::context::Context;
use crate::texture::Texture2D;

impl Context {
    /// Copia todo o conteúdo de `src` para `dst`. Os dois buffers
    /// precisam ter o mesmo tamanho.
    pub fn copy_buffer(&self, src: &Buffer, dst: &Buffer) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.CopyResource(&dst.raw, &src.raw);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (src, dst);
        }
    }

    /// Copia todo o conteúdo de `src` para `dst`. As duas texturas
    /// precisam ter as mesmas dimensões e formato.
    pub fn copy_texture(&self, src: &Texture2D, dst: &Texture2D) {
        #[cfg(target_os = "windows")]
        unsafe {
            self.raw.CopyResource(&dst.raw, &src.raw);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (src, dst);
        }
    }
}
