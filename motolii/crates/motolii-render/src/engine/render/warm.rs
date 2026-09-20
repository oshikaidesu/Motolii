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

/// 画面座標の矩形(左・上・右・下)。
type ScreenRect = [f32; 4];

fn rect_inside(inner: ScreenRect, outer: ScreenRect) -> bool {
    inner[0] >= outer[0] && inner[1] >= outer[1] && inner[2] <= outer[2] && inner[3] <= outer[3]
}

impl Engine {
    /// 復号の前に分かる範囲で層の画面上の矩形を出す。寸が分かる動画の層だけ。
    /// 軸に沿った矩形なら `Some((rect, true))`、回っていれば外接矩形と `false`。
    fn media_screen_rect(&self, comp: CompSpec, camera: ResolvedCamera, layer: &ResolvedLayer) -> Option<(ScreenRect, bool)> {
        let LayerSource::File { path, .. } = &layer.source else { return None };
        if crate::render::media::is_still_image_path(path)
            || crate::render::media::is_mesh_path(&path)
            || crate::render::media::is_point_cloud_path(path)
        {
            return None;
        }
        let info = self.probes.get(path)?;
        if info.rotation != 0 {
            return None;
        }
        let world = layer.placement.world_transform?;
        let size = layer_size(layer, [info.width as f32, info.height as f32]);
        let corners = crate::doc::core::projected_screen_corners(
            comp, camera, camera, layer.projection, world, [0.0, 0.0, 0.0], [size[0], size[1], 0.0],
        );
        if corners.iter().any(|c| !c.is_finite()) {
            return None;
        }
        let rect = corners.iter().fold([f32::MAX, f32::MAX, f32::MIN, f32::MIN], |r, c| {
            [r[0].min(c.x), r[1].min(c.y), r[2].max(c.x), r[3].max(c.y)]
        });
        // 4 隅が矩形の角に乗っていれば軸に沿っている(回転・傾き無し)。
        let eps = 0.5;
        let on_corner = |c: &glam::Vec2| {
            ((c.x - rect[0]).abs() < eps || (c.x - rect[2]).abs() < eps)
                && ((c.y - rect[1]).abs() < eps || (c.y - rect[3]).abs() < eps)
        };
        Some((rect, corners.iter().all(on_corner)))
    }

    /// 上から順に不透明な矩形を積み、丸ごと覆われた層と画面の外の層を集める(Blender VSE の型)。
    pub(super) fn unseen_layers(
        &self,
        comp: CompSpec,
        camera: ResolvedCamera,
        resolved: &[ResolvedLayer],
        needs_auxiliary_views: bool,
        matte_sources: &HashSet<LayerId>,
    ) -> HashSet<usize> {
        let mut unseen = HashSet::new();
        if needs_auxiliary_views {
            return unseen;
        }
        let screen: ScreenRect = [0.0, 0.0, comp.width as f32, comp.height as f32];
        let mut covers: Vec<ScreenRect> = Vec::new();
        for (index, layer) in resolved.iter().enumerate().rev() {
            // 他の層の入力になる物は消さない。解き手が動かす物も消さない — 画面の外から入って来る
            // (利用者 2026-09-16 の見本: 上から降ってくる切り抜き)。
            let feeds_others = matte_sources.contains(&layer.id) || layer.matte.is_some() || layer.clip_to_below
                || self.blocks.moves(layer.id) || self.analysing;
            let Some((rect, axis_aligned)) = self.media_screen_rect(comp, camera, layer) else { continue };
            let plain = layer.effects.is_empty() && layer.after_effects.is_empty() && layer.masks.is_empty();
            let offscreen = rect[2] < 0.0 || rect[3] < 0.0 || rect[0] > screen[2] || rect[1] > screen[3];
            if !feeds_others && plain && (offscreen || covers.iter().any(|cover| rect_inside(rect, *cover))) {
                unseen.insert(index);
                continue;
            }
            let opaque = axis_aligned
                && plain
                && layer.placement.opacity >= 1.0
                && layer.blend_mode == crate::doc::store::BlendMode::Normal
                && layer.matte.is_none()
                && !layer.clip_to_below
                && !layer.ghost
                // 面が画面に平行なら projection の種類は問わない(描画順は order のまま)。
                && layer.placement.rotation_x == 0.0
                && layer.placement.rotation_y == 0.0;
            if opaque {
                covers.push(rect);
            }
        }
        unseen
    }
}

/// 先に開いておく幅。30fps で半秒。
const WARM_AHEAD_FRAMES: i64 = 15;

impl Engine {
    /// 再生位置の少し先で現れる動画層の復号器を、今のうちに開く。timeline は未来を知っている。
    /// 待たず、失敗も記録しない(本番の描画で改めて分かる)。
    pub fn warm_upcoming(&mut self, view: &StoreView<'_>, t: RationalTime) -> Result<(), EngineError> {
        if !Self::has_file_layers(view)? { return Ok(()); }
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let fps = composition.fps;
        let now_frame = t.try_to_frame_floor(fps).map_err(|e| EngineError::Time(e.to_string()))?;
        let ahead = RationalTime::try_from_frame(now_frame + WARM_AHEAD_FRAMES, fps)
            .map_err(|e| EngineError::Time(e.to_string()))?;
        let now_ids: HashSet<LayerId> = crate::picture::resolve::resolved_layers(view, t)
            .map_err(|e| EngineError::Store(e.to_string()))?
            .iter()
            .map(|layer| layer.id)
            .collect();
        let upcoming = crate::picture::resolve::resolved_layers(view, ahead)
            .map_err(|e| EngineError::Store(e.to_string()))?;

        let failures = std::mem::take(&mut self.layer_failures);
        let was_realtime = self.realtime;
        self.realtime = true;
        for layer in upcoming.iter().filter(|layer| !now_ids.contains(&layer.id)) {
            let LayerSource::File { path, .. } = &layer.source else { continue };
            if crate::render::media::is_still_image_path(path)
                || crate::render::media::is_audio_path(path)
                || crate::render::media::is_mesh_path(&path)
                || crate::render::media::is_point_cloud_path(path)
            {
                continue;
            }
            let _ = self.media_texture_for(&path, layer.source_time, layer.id);
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
