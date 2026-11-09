//! Surface, swapchain e apresentação.
//!
//! Conecta uma janela a uma cadeia de troca de buffers e cuida de
//! apresentação e resize. **`ndl-render` nunca depende de `ndl-platform`
//! (ou de qualquer outra lib de janela) diretamente** — [`Surface`] recebe
//! um [`raw_window_handle::RawWindowHandle`] genérico, e é o código do
//! jogo que extrai esse handle de onde quer que a janela tenha sido
//! criada (seja `ndl-platform`, `winit` puro, etc.) e o entrega aqui.
//!
//! ## Desempenho e `PresentMode`
//!
//! Por padrão, [`Swapchain::present`] usa [`PresentMode::VSync`] — o
//! "liso" numa GPU topo de linha não é o mesmo que numa GPU mais modesta,
//! então a crate nunca assume `Immediate` (sem limite de quadros) como
//! padrão. Jogos que sabem o que estão fazendo podem pedir `Immediate`
//! explicitamente via [`SwapchainDescriptor::present_mode`].

use raw_window_handle::RawWindowHandle;

use crate::core::{Device, GraphicsError};
use crate::format::Format;

/// Dimensões em pixels de uma superfície/swapchain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extent2D {
    pub width: u32,
    pub height: u32,
}

impl Extent2D {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// Como os quadros são apresentados na tela.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PresentMode {
    /// Espera o próximo vertical sync — sem tearing, taxa de quadros
    /// consistente entre hardwares diferentes. Padrão.
    #[default]
    VSync,
    /// Apresenta assim que possível, sem esperar o vsync — pode causar
    /// tearing e consome GPU sem limite; só use se o jogo já tem seu
    /// próprio limitador de quadros.
    Immediate,
}

/// Resultado de uma apresentação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentResult {
    Ok,
    /// A swapchain não é mais válida (ex: janela minimizada/ocluída em
    /// alguns backends) — o jogo deve pular o present deste quadro.
    Suboptimal,
}

/// Configuração usada para criar uma [`Swapchain`].
#[derive(Debug, Clone, Copy)]
pub struct SwapchainDescriptor {
    pub size: Extent2D,
    pub format: Format,
    pub buffer_count: u32,
    pub present_mode: PresentMode,
}

impl Default for SwapchainDescriptor {
    fn default() -> Self {
        Self {
            size: Extent2D::new(1, 1),
            format: Format::Bgra8Unorm,
            buffer_count: 2,
            present_mode: PresentMode::VSync,
        }
    }
}

/// Uma superfície de apresentação, associada a uma janela via
/// [`raw_window_handle::RawWindowHandle`].
pub struct Surface {
    #[cfg(target_os = "windows")]
    pub(crate) hwnd: windows::Win32::Foundation::HWND,
}

impl Surface {
    pub fn from_window_handle(handle: RawWindowHandle) -> Result<Self, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::surface_from_handle(handle);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = handle;
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
    }
}

/// Cadeia de troca de buffers (swapchain) de uma [`Surface`].
pub struct Swapchain {
    #[cfg(target_os = "windows")]
    pub(crate) raw: crate::backend::directx::RawSwapchain,
    size: Extent2D,
    format: Format,
    present_mode: PresentMode,
}

impl Swapchain {
    pub fn new(
        device: &Device,
        surface: &Surface,
        desc: &SwapchainDescriptor,
    ) -> Result<Self, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            let raw = crate::backend::directx::create_swapchain(device, surface, desc)?;
            return Ok(Self {
                raw,
                size: desc.size,
                format: desc.format,
                present_mode: desc.present_mode,
            });
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (device, surface, desc);
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub fn dimensions(&self) -> Extent2D {
        self.size
    }

    pub fn present_mode(&self) -> PresentMode {
        self.present_mode
    }

    pub fn set_present_mode(&mut self, mode: PresentMode) {
        self.present_mode = mode;
    }

    /// Redimensiona os buffers da swapchain — deve ser chamado quando a
    /// janela associada muda de tamanho (ex: em resposta a
    /// `WindowEvent::Resized` vindo de `ndl-platform`, repassado pelo
    /// código do jogo).
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            crate::backend::directx::resize_swapchain(&mut self.raw, width, height)?;
        }
        self.size = Extent2D::new(width, height);
        Ok(())
    }

    /// Limpa o back buffer atual com a cor dada e apresenta o quadro.
    ///
    /// Isso é um atalho temporário enquanto `context`/`render` ainda não
    /// existem — quando esses módulos forem implementados, limpar e
    /// desenhar passam a ser passos separados de um `RenderPass` real.
    pub fn clear_and_present(&mut self, device: &Device, color: [f32; 4]) -> Result<PresentResult, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::clear_and_present(
                device,
                &self.raw,
                color,
                self.present_mode,
            );
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (device, color);
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
    }
}
