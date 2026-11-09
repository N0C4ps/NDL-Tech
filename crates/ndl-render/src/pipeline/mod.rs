//! Configuração completa do pipeline gráfico e de computação.
//!
//! O D3D11 é uma API baseada em objetos de estado e bindings separados;
//! este módulo agrupa tudo isso em um único [`GraphicsPipeline`], que por
//! baixo aplica os estados D3D11 correspondentes.
//!
//! ## Estado atual
//!
//! [`GraphicsPipeline`] agrupa shaders, input layout, topologia e estados
//! explícitos de rasterizer/blend/depth-stencil. A variante configurada
//! declara também os formatos esperados dos attachments — um ou vários,
//! via `GraphicsPipelineStateDescriptor::color_formats` — verificados ao
//! vincular o pipeline a um [`crate::render::RenderPass`] com o mesmo
//! número de alvos de cor, na mesma ordem (MRT). MSAA ainda não faz parte
//! do contrato. [`ComputePipeline`] também já é real.

use crate::backend::{ActiveBackend, Backend};
use crate::blend::BlendStateDescriptor;
use crate::core::{Device, GraphicsError};
use crate::depth_stencil::DepthStencilStateDescriptor;
use crate::format::Format;
use crate::input::{InputLayout, PrimitiveTopology};
use crate::raster::RasterizerStateDescriptor;
use crate::shader::{ComputeShader, PixelShader, VertexShader};

/// Configuração usada para criar um [`GraphicsPipeline`].
pub struct GraphicsPipelineDescriptor<'a> {
    pub vertex_shader: &'a VertexShader,
    pub pixel_shader: &'a PixelShader,
    pub input_layout: &'a InputLayout,
    pub primitive_topology: PrimitiveTopology,
}

/// Um pipeline gráfico: todo o estado necessário para emitir um `draw()`
/// — hoje, vertex/pixel shader, input layout e topologia.
///
/// `raw` é opaco de propósito: cada backend decide a forma interna (ver
/// [`crate::backend::Backend::assemble_graphics_pipeline`]) — D3D11 agrupa
/// os objetos de estado já criados separadamente; D3D12 monta um PSO numa
/// única chamada. `color_formats`/`depth_format` continuam campos comuns
/// porque [`crate::render::RenderPass`] precisa deles pra validar
/// attachments sem conhecer o backend.
pub struct GraphicsPipeline {
    pub(crate) raw: <ActiveBackend as Backend>::RawGraphicsPipeline,
    /// Vazio = nenhum formato declarado (caminho de
    /// [`Device::create_graphics_pipeline`], sem checagem de attachment).
    /// Não vazio = um formato por alvo de cor, na ordem vinculada por
    /// [`crate::render::RenderPass`] — `len() > 1` é MRT.
    pub(crate) color_formats: Vec<Format>,
    pub(crate) depth_format: Option<Format>,
}

/// Estados fixos e formatos dos attachments esperados pelo pipeline.
///
/// `color_formats` tem um formato por alvo de cor simultâneo — `&[fmt]`
/// para um único attachment, `&[fmt0, fmt1, ...]` para MRT (até
/// [`crate::core::DeviceLimits::max_color_attachments`]).
#[derive(Debug, Clone, Copy)]
pub struct GraphicsPipelineStateDescriptor<'a> {
    pub rasterizer: RasterizerStateDescriptor,
    pub blend: BlendStateDescriptor,
    pub depth_stencil: DepthStencilStateDescriptor,
    pub color_formats: &'a [Format],
    pub depth_format: Option<Format>,
    pub stencil_reference: u8,
}

impl Device {
    pub fn create_graphics_pipeline(
        &self,
        desc: &GraphicsPipelineDescriptor<'_>,
    ) -> Result<GraphicsPipeline, GraphicsError> {
        let rasterizer = self.create_rasterizer_state(&RasterizerStateDescriptor::default())?;
        let blend = self.create_blend_state(&BlendStateDescriptor::default())?;
        let depth_stencil =
            self.create_depth_stencil_state(&DepthStencilStateDescriptor::disabled())?;
        let raw = ActiveBackend::assemble_graphics_pipeline(
            self, desc, rasterizer, blend, depth_stencil, &[], None, 0,
        )?;
        Ok(GraphicsPipeline {
            raw,
            color_formats: Vec::new(),
            depth_format: None,
        })
    }

    /// Cria pipeline com estados fixos e formatos de attachment explícitos.
    pub fn create_graphics_pipeline_with_state(
        &self,
        desc: &GraphicsPipelineDescriptor<'_>,
        state: &GraphicsPipelineStateDescriptor<'_>,
    ) -> Result<GraphicsPipeline, GraphicsError> {
        if state.color_formats.is_empty() {
            return Err(GraphicsError::InvalidArgument {
                name: "color_formats",
                reason: "precisa declarar ao menos um formato de alvo de cor".into(),
            });
        }
        let max_color_attachments = self.limits().max_color_attachments as usize;
        if state.color_formats.len() > max_color_attachments {
            return Err(GraphicsError::OutOfBounds {
                resource: "pipeline color_formats",
                offset: 0,
                size: state.color_formats.len() as u64,
                limit: max_color_attachments as u64,
            });
        }
        for color_format in state.color_formats {
            if color_format.is_depth() || color_format.is_compressed() {
                return Err(GraphicsError::InvalidArgument {
                    name: "color_formats",
                    reason: "cada formato precisa ser colorido e não comprimido".into(),
                });
            }
            if !self.format_features(*color_format)?.render_target {
                return Err(GraphicsError::FeatureNotSupported {
                    feature: format!("color attachment no formato {color_format:?}"),
                });
            }
        }
        if let Some(format) = state.depth_format {
            if !format.is_depth() {
                return Err(GraphicsError::InvalidArgument {
                    name: "depth_format",
                    reason: "o formato precisa ser depth/stencil".into(),
                });
            }
            if !self.format_features(format)?.depth_stencil {
                return Err(GraphicsError::FeatureNotSupported {
                    feature: format!("depth attachment no formato {format:?}"),
                });
            }
        }
        if (state.depth_stencil.depth_test_enabled
            || state.depth_stencil.depth_write_enabled
            || state.depth_stencil.stencil_test_enabled)
            && state.depth_format.is_none()
        {
            return Err(GraphicsError::IncompatibleResource {
                operation: "create_graphics_pipeline",
                reason: "depth test, escrita de depth ou stencil exige depth_format".into(),
            });
        }
        if state.depth_stencil.stencil_test_enabled
            && !state.depth_format.is_some_and(Format::is_stencil)
        {
            return Err(GraphicsError::IncompatibleResource {
                operation: "create_graphics_pipeline",
                reason: "stencil test exige um formato de depth/stencil com canal stencil".into(),
            });
        }
        let rasterizer = self.create_rasterizer_state(&state.rasterizer)?;
        let blend = self.create_blend_state(&state.blend)?;
        let depth_stencil = self.create_depth_stencil_state(&state.depth_stencil)?;
        let raw = ActiveBackend::assemble_graphics_pipeline(
            self,
            desc,
            rasterizer,
            blend,
            depth_stencil,
            state.color_formats,
            state.depth_format,
            state.stencil_reference as u32,
        )?;
        Ok(GraphicsPipeline {
            raw,
            color_formats: state.color_formats.to_vec(),
            depth_format: state.depth_format,
        })
    }

    pub fn create_compute_pipeline(
        &self,
        desc: &ComputePipelineDescriptor<'_>,
    ) -> Result<ComputePipeline, GraphicsError> {
        if !self.capabilities().supports_compute() {
            return Err(GraphicsError::FeatureNotSupported {
                feature: "compute shaders não suportados pelo dispositivo".into(),
            });
        }
        Ok(ComputePipeline {
            raw: ActiveBackend::assemble_compute_pipeline(self, desc)?,
        })
    }
}

/// Configuração usada para criar um [`ComputePipeline`].
pub struct ComputePipelineDescriptor<'a> {
    pub compute_shader: &'a ComputeShader,
}

/// Um pipeline de computação — ver `raw` em [`GraphicsPipeline`] pro porquê
/// de ser opaco.
pub struct ComputePipeline {
    pub(crate) raw: <ActiveBackend as Backend>::RawComputePipeline,
}
