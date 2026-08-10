use rerun::Archetype as _;
use rerun::external::re_view::{DataResultQuery as _, VisualizerInstructionQueryResults};
use rerun::external::re_viewer_context::{
    self, IdentifiedViewSystem, ViewClass as _, ViewContext, ViewContextCollection, ViewQuery,
    ViewSystemExecutionError, ViewSystemIdentifier, VisualizerExecutionOutput, VisualizerQueryInfo,
    VisualizerSystem,
};
use rerun::external::{re_renderer, re_view_spatial};

use crate::path_archetype::Path2DFill;
use crate::path_renderer::{Path2DDrawData, PathConfig};
use crate::path2d::FillContribution;

#[derive(Default)]
pub struct Path2DVisualizer;

impl IdentifiedViewSystem for Path2DVisualizer {
    fn identifier() -> ViewSystemIdentifier {
        "motolii.Path2DVisualizer".into()
    }
}

impl VisualizerSystem for Path2DVisualizer {
    fn visualizer_query_info(
        &self,
        _app_options: &re_viewer_context::AppOptions,
    ) -> VisualizerQueryInfo {
        VisualizerQueryInfo::single_required_component::<rerun::components::Blob>(
            &Path2DFill::descriptor_payload(),
            &Path2DFill::all_components(),
        )
    }

    fn affinity(&self) -> Option<rerun::external::re_sdk_types::ViewClassIdentifier> {
        Some(re_view_spatial::SpatialView2D::identifier())
    }

    fn execute(
        &self,
        ctx: &ViewContext<'_>,
        query: &ViewQuery<'_>,
        context_systems: &ViewContextCollection,
    ) -> Result<VisualizerExecutionOutput, ViewSystemExecutionError> {
        let mut output = VisualizerExecutionOutput::default();
        let transforms = context_systems.get::<re_view_spatial::TransformTreeContext>(&output)?;
        let mut draw_data = Path2DDrawData::new(ctx.render_ctx());

        for (data_result, instruction) in query.iter_visualizer_instruction_for(Self::identifier())
        {
            let entity_path = &data_result.entity_path;
            let Some(Ok(transform_info)) = transforms.target_from_entity_path(entity_path.hash())
            else {
                continue;
            };
            let results =
                data_result.query_archetype_with_history::<Path2DFill>(ctx, query, instruction);
            let results = VisualizerInstructionQueryResults::new(instruction, &results, &output);
            let payloads = results.iter_required(Path2DFill::descriptor_payload().component);
            let transform = transform_info
                .single_transform_required_for_entity(entity_path, Path2DFill::name())
                .as_affine3a();
            let picking_object_id = re_renderer::PickingLayerObjectId(entity_path.hash64());
            let outline_mask = query
                .highlights
                .entity_outline_mask(entity_path.hash())
                .index_outline_mask(0_u64.into());

            for (_, payloads) in payloads.slice::<&[u8]>() {
                let Some(payload) = payloads.first() else {
                    continue;
                };
                let Ok(contribution) = FillContribution::decode(payload) else {
                    continue;
                };
                let Ok(mesh) = contribution.tessellate_convex() else {
                    continue;
                };
                draw_data.add_path(
                    ctx.render_ctx(),
                    PathConfig {
                        world_from_obj: transform,
                        mesh: &mesh,
                        color: contribution.color,
                        draw_order: contribution.draw_order,
                        picking_object_id,
                        outline_mask,
                    },
                );
            }
        }

        output.draw_data = vec![draw_data.into()];
        Ok(output)
    }
}
