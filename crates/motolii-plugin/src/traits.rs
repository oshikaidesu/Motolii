//! 種別ごとのplugin trait。

use motolii_core::RationalTime;
use motolii_eval::DataTrack;
use motolii_gpu::{GpuCtx, PipelineCache};

use crate::context::{LayerSourceContext, ParamDriverContext, RenderCtx, TextureRef};
use crate::contract::{NodeDesc, PluginError};
use crate::params::ResolvedParams;

pub trait FilterPlugin: Send + Sync {
    fn desc(&self) -> &NodeDesc;

    // プラグイン契約の引数集合(GPU/文脈/params/入出力)が閾値を超えるのは構造上のもの。
    #[allow(clippy::too_many_arguments)]
    fn render(
        &self,
        gpu: &GpuCtx,
        pipelines: &mut PipelineCache,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &RenderCtx,
        params: &ResolvedParams,
        input: TextureRef<'_>,
        output: TextureRef<'_>,
    ) -> Result<(), PluginError>;
}

pub trait LayerSourcePlugin: Send + Sync {
    fn desc(&self) -> &NodeDesc;

    #[allow(clippy::too_many_arguments)]
    fn render(
        &self,
        gpu: &GpuCtx,
        pipelines: &mut PipelineCache,
        encoder: &mut wgpu::CommandEncoder,
        t: RationalTime,
        params: &ResolvedParams,
        ctx: LayerSourceContext,
        output: TextureRef<'_>,
    ) -> Result<(), PluginError>;
}

pub trait ParamDriverPlugin: Send + Sync {
    fn desc(&self) -> &NodeDesc;

    fn build_track(
        &self,
        ctx: ParamDriverContext,
        params: &ResolvedParams,
    ) -> Result<DataTrack, PluginError>;
}

pub trait CompositePlugin: Send + Sync {
    fn desc(&self) -> &NodeDesc;

    #[allow(clippy::too_many_arguments)]
    fn render(
        &self,
        gpu: &GpuCtx,
        pipelines: &mut PipelineCache,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &RenderCtx,
        params: &ResolvedParams,
        inputs: &[TextureRef<'_>],
        output: TextureRef<'_>,
    ) -> Result<(), PluginError>;
}
