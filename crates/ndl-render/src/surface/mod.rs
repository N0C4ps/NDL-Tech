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
use crate::view::RenderTargetView;

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
    /// Cacheada na primeira vez que é pedida; invalidada em `resize`,
    /// porque o buffer físico por trás dela muda.
    render_target_view: Option<RenderTargetView>,
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
                render_target_view: None,
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
        // A view antiga aponta para o back buffer anterior — inválida
        // depois do resize, então é descartada e recriada sob demanda.
        self.render_target_view = None;

        #[cfg(target_os = "windows")]
        {
            crate::backend::directx::resize_swapchain(&mut self.raw, width, height)?;
        }
        self.size = Extent2D::new(width, height);
        Ok(())
    }

    /// A [`RenderTargetView`] do back buffer atual, criando-a (e
    /// cacheando) se ainda não existir.
    pub fn render_target_view(&mut self, device: &Device) -> Result<&RenderTargetView, GraphicsError> {
        if self.render_target_view.is_none() {
            #[cfg(target_os = "windows")]
            {
                let view = crate::backend::directx::create_swapchain_render_target_view(device, &self.raw)?;
                self.render_target_view = Some(view);
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = device;
                return Err(GraphicsError::Unsupported(
                    "nenhum backend gráfico implementado para este SO ainda",
                ));
            }
        }
        Ok(self.render_target_view.as_ref().expect("acabou de ser preenchida"))
    }

    /// Apresenta o quadro atual.
    pub fn present(&self) -> Result<PresentResult, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::present(&self.raw, self.present_mode);
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
    }

    /// Limpa o back buffer atual com a cor dada e apresenta o quadro.
    ///
    /// Conveniência por cima de [`Device::context`] + [`Swapchain::render_target_view`]
    /// + [`Swapchain::present`] — útil enquanto `render` (passes de desenho
    /// de verdade) ainda não existe. Jogos que precisam desenhar mais do
    /// que uma cor sólida devem usar esses três passos diretamente.
    pub fn clear_and_present(&mut self, device: &Device, color: [f32; 4]) -> Result<PresentResult, GraphicsError> {
        let context = device.context();
        let view = self.render_target_view(device)?;
        context.clear_render_target(view, color);
        self.present()
    }

    /// Lê de volta o conteúdo atual do back buffer, como pixels crus —
    /// para validar de verdade o que foi desenhado (ex: em testes), em
    /// vez de só confiar que o desenho não quebrou. Chame **antes** de
    /// `present()`, enquanto o back buffer ainda tem o quadro atual.
    pub fn capture_frame(&self, device: &Device) -> Result<FrameCapture, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            let (width, height, format, pixels) =
                crate::backend::directx::capture_swapchain_frame(device, &self.raw)?;
            return Ok(FrameCapture {
                width,
                height,
                format,
                pixels,
            });
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = device;
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
    }
}

/// Uma cópia dos pixels de um quadro, lida de volta da GPU — ver
/// [`Swapchain::capture_frame`].
pub struct FrameCapture {
    width: u32,
    height: u32,
    format: Format,
    /// Pixels compactados (sem padding de linha), `width * height *
    /// bytes_per_pixel(format)` bytes, linha a linha de cima para baixo.
    pixels: Vec<u8>,
}

impl FrameCapture {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn format(&self) -> Format {
        self.format
    }

    /// O pixel em `(x, y)`, como bytes crus no formato original (ex: 4
    /// bytes BGRA para `Format::Bgra8Unorm`, o formato comum de
    /// swapchain). Entra em pânico se `(x, y)` estiver fora dos limites.
    pub fn pixel(&self, x: u32, y: u32) -> &[u8] {
        assert!(x < self.width && y < self.height, "pixel ({x}, {y}) fora dos limites ({}, {})", self.width, self.height);
        let bytes_per_pixel = (self.format.bits_per_pixel() / 8) as usize;
        let offset = (y as usize * self.width as usize + x as usize) * bytes_per_pixel;
        &self.pixels[offset..offset + bytes_per_pixel]
    }

    /// `pixel(x, y)` interpretado como BGRA8 (`[b, g, r, a]`), convertido
    /// para `[r, g, b, a]` — conveniência para formatos comuns de
    /// swapchain (`Bgra8Unorm`/`Bgra8UnormSrgb`). Entra em pânico se o
    /// formato não tiver 4 bytes por pixel.
    pub fn pixel_rgba(&self, x: u32, y: u32) -> [u8; 4] {
        let raw = self.pixel(x, y);
        assert_eq!(raw.len(), 4, "pixel_rgba só funciona com formatos de 4 bytes por pixel");
        [raw[2], raw[1], raw[0], raw[3]]
    }
}
