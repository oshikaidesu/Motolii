//! 先に開けておく物と、見えない物を捨てる係。

use super::*;

/// 使われなくなった動画の再生機(ffmpeg 子プロセス)を掃く猶予。30fps で 5 秒。
const VIDEO_PLAYER_PURGE_EVERY: u32 = 150;

impl Engine {
    pub(super) fn purge_idle_video_players(&mut self) {
        self.frame_cache_tick += 1;
        self.flush_pending_frame_copies();
        self.renders_since_video_purge += 1;
        if self.renders_since_video_purge < VIDEO_PLAYER_PURGE_EVERY {
            return;
        }
        self.renders_since_video_purge = 0;
        for (_, video) in self.videos.values() {
            video.begin_frame();
        }
    }
}

impl Engine {

}

/// 先に開いておく幅。30fps で半秒。
const WARM_AHEAD_FRAMES: i64 = 15;

impl Engine {
    /// 再生位置の少し先で現れる動画層の復号器を、今のうちに開く。timeline は未来を知っている。
    /// 待たず、失敗も記録しない(本番の描画で改めて分かる)。
    pub fn warm_upcoming(&mut self, view: &StoreView<'_>, t: RationalTime) -> Result<(), EngineError> {
        let warmed = self.warm_upcoming_frames(view, t);
        // Warming runs between ticks: what it uploaded goes out now, not across the next frame's start.
        self.compositor.flush_pending();
        warmed
    }

    fn warm_upcoming_frames(&mut self, view: &StoreView<'_>, t: RationalTime) -> Result<(), EngineError> {
        if !Self::has_file_layers(view)? { return Ok(()); }
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let fps = composition.fps;
        let now_frame = t.try_to_frame_floor(fps).map_err(|e| EngineError::Time(e.to_string()))?;
        let ahead = RationalTime::try_from_frame(now_frame + WARM_AHEAD_FRAMES, fps)
            .map_err(|e| EngineError::Time(e.to_string()))?;

        // Warming is execution policy, not a second document resolver. Evaluate
        // only the graph values needed to decide whether a timed media source
        // participates now and at the look-ahead time.
        let properties = crate::frame_graph::PropertyProgram::compile(view)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let visibility = crate::frame_graph::VisibilityProgram::compile(view, &properties)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let content = crate::frame_graph::ContentProgram::compile(view, &properties)
            .map_err(|e| EngineError::Store(e.to_string()))?;

        let mut targets = Vec::new();
        let mut roots = Vec::new();
        for layer in view.layers() {
            let Some(meta) = view.meta(layer).map_err(|e| EngineError::Store(e.to_string()))? else { continue };
            let LayerSource::File { path, .. } = &meta.source else { continue };
            if crate::render::media::is_still_image_path(path)
                || crate::render::media::is_audio_path(path)
                || crate::render::media::is_mesh_path(path)
                || crate::render::media::is_point_cloud_path(path)
            {
                continue;
            }
            let Some(visible) = visibility.binding(layer).map(|binding| binding.node) else { continue };
            let Some(media) = content.binding(layer).and_then(|binding| binding.content) else { continue };
            roots.push(visible);
            roots.push(media);
            targets.push((layer, visible, media));
        }
        if targets.is_empty() { return Ok(()); }

        let nodes = properties.nodes().chain(visibility.nodes()).chain(content.nodes());
        let topology = crate::frame_graph::GraphTopology::try_new(nodes, roots)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let mut graph = crate::frame_graph::CompiledGraph::with_topology(
            crate::frame_graph::GraphRevision::new(view.revision_key()),
            topology,
        );

        struct WarmExecutor<'a> {
            properties: &'a crate::frame_graph::PropertyProgram,
            visibility: &'a crate::frame_graph::VisibilityProgram,
            content: &'a crate::frame_graph::ContentProgram,
        }
        impl crate::frame_graph::NodeExecutor for WarmExecutor<'_> {
            type Error = EngineError;
            fn execute(
                &mut self,
                node: &crate::frame_graph::GraphNode,
                inputs: crate::frame_graph::NodeInputs,
                context: crate::frame_graph::EvaluationContext,
            ) -> Result<crate::frame_graph::NodeValue, Self::Error> {
                if let Some(value) = self.properties.execute(node, &inputs, &context) {
                    return value.map_err(|e| EngineError::Store(e.to_string()));
                }
                if let Some(value) = self.visibility.execute(node, &inputs, &context) {
                    return value.map_err(|e| EngineError::Store(e.to_string()));
                }
                if let Some(value) = self.content.execute(node, &inputs, &context) {
                    return value.map_err(|e| EngineError::Store(e.to_string()));
                }
                Err(EngineError::Store(format!("warm-up graph does not support {:?}", node.identity().kind)))
            }
        }
        let mut executor = WarmExecutor { properties: &properties, visibility: &visibility, content: &content };
        let now = graph.evaluate(
            &mut executor,
            t,
            crate::frame_graph::FrameQuality::Preview { scale: 1 },
            crate::frame_graph::Generation::new(1),
        )?;
        let future = graph.evaluate(
            &mut executor,
            ahead,
            crate::frame_graph::FrameQuality::Preview { scale: 1 },
            crate::frame_graph::Generation::new(2),
        )?;

        let any_solo = |frame: &crate::frame_graph::EvaluatedFrame| {
            targets.iter().any(|(_, visible, _)| {
                frame.value(*visible)
                    .and_then(|value| value.downcast_ref::<crate::frame_graph::VisibilityValue>())
                    .is_some_and(|value| value.solo)
            })
        };
        let now_solo = any_solo(&now);
        let future_solo = any_solo(&future);
        let participates = |frame: &crate::frame_graph::EvaluatedFrame, visible: crate::frame_graph::NodeKey, solo_gate: bool| {
            frame.value(visible)
                .and_then(|value| value.downcast_ref::<crate::frame_graph::VisibilityValue>())
                .is_some_and(|value| value.active && (!solo_gate || value.solo))
        };
        let now_ids: HashSet<LayerId> = targets.iter()
            .filter(|(_, visible, _)| participates(&now, *visible, now_solo))
            .map(|(layer, _, _)| *layer)
            .collect();

        let failures = std::mem::take(&mut self.layer_failures);
        let was_realtime = self.realtime;
        self.realtime = true;
        for (layer, visible, media) in targets {
            if now_ids.contains(&layer) || !participates(&future, visible, future_solo) { continue; }
            let Some(frame) = future.value(media)
                .and_then(|value| value.downcast_ref::<crate::frame_graph::MediaFrameValue>())
            else { continue };
            let Some(source_time) = frame.time else { continue };
            let _ = self.media_texture_for(&frame.source.path, source_time, layer);
        }
        self.realtime = was_realtime;
        self.layer_failures = failures;
        Ok(())
    }

    pub(in crate::engine) fn has_file_layers(view: &StoreView<'_>) -> Result<bool, EngineError> {
        for id in view.layers() {
            if view.meta(id).map_err(|e| EngineError::Store(e.to_string()))?
                .is_some_and(|meta| matches!(meta.source, LayerSource::File { .. })) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
