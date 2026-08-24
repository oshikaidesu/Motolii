use std::collections::HashMap;
use std::sync::Arc;

use motolii_engine::ObservationCamera;
use motolii_store::{DisplayRevision, LayerSource, RationalTime, ShapeNode};

use crate::stage_presenter::build_stage_presenter_rgba;
use crate::tokens::{Colors, Dimensions, Tokens};
use crate::{metrics, stage, Shell};

impl Shell {
    /// 描き上がった Stage フレームの生 RGBA。**常に背景込みの export 真値**
    /// (`Engine::render_frame`)— 市松トグルで一切変わらない。**screenshot
    /// 器具専用**(`screenshot.rs`)— 通常描画は shader Program(`stage_pane`)を
    /// 通る(裁定166 — GPU 高速路の間は `presenter_source: PresenterSource::Gpu`
    /// を渡す、`image::Handle` はもう作らない)。
    ///
    /// **裁定171 v2(M4)で `&mut self` になった** — GPU 高速路(`refresh_frame`)
    /// はこのフィールドを更新しない代わりに `rgba_stale` を立てるので、ここで
    /// 呼ばれた時だけ [`Self::ensure_rgba_fresh`] が CPU readback を1回払って
    /// 追いつかせる(EXACT TARGET 4「readback は要求された時だけ」)。呼び出し元は
    /// `screenshot.rs`(CLI 器具、`&mut Shell` は元から手元にある)と試験のみ —
    /// 通常描画(`Shell::view`)からは呼ばれない。
    pub fn frame_rgba(&mut self) -> Option<(u32, u32, &[u8])> {
        self.ensure_rgba_fresh();
        self.frame
            .as_ref()
            .map(|frame| (frame.width, frame.height, frame.rgba.as_slice()))
    }

    /// 市松 ON の間だけ `Some` — 裁定141「AE型の透明可視化モード」の入力
    /// (`Engine::render_frame_without_background` の結果そのもの、市松タイルは
    /// **まだ乗っていない**生値)。**screenshot 器具専用**(`screenshot.rs`)。
    /// `frame_rgba()` とは別物 — あちらは常に背景込みの export 真値。
    pub fn checkerboard_preview_rgba(&self) -> Option<(u32, u32, &[u8])> {
        self.frame.as_ref().and_then(|frame| {
            frame
                .checkerboard_preview_rgba
                .as_deref()
                .map(|rgba| (frame.width, frame.height, rgba))
        })
    }

    /// 今の観測カメラの状態(裁定157)。運転席/screenshot 器具が「カメラを通して
    /// 見る」(`None`)/「自由に見る」(`Some`)のどちらかを確かめる口
    /// (`checkerboard_enabled` と同じ形)。
    pub fn observation(&self) -> Option<ObservationCamera> {
        self.observation
    }

    /// 観測カメラ有効時の Stage 表示 RGBA(`Engine::render_frame_with_view_camera`
    /// の結果そのもの)。**`frame_rgba()`(export 真値)とは別物** —
    /// `checkerboard_preview_rgba` と同じ「screenshot 器具/試験専用」の形。
    /// `observation()` が `None` の間は常に `None`。
    pub fn observation_rgba(&self) -> Option<(u32, u32, &[u8])> {
        self.frame.as_ref().and_then(|frame| {
            frame
                .observation_rgba
                .as_deref()
                .map(|rgba| (frame.width, frame.height, rgba))
        })
    }

    /// Media 素材の実寸(表示上の寸法 — 回転メタデータ適用後、
    /// `motolii_media::MediaInfo` doc)。path ごとに1回だけ probe して
    /// `media_size_cache` に控える(失敗も `None` で控え、毎フレーム叩き直さ
    /// ない)。`Shell` 構造体フィールドの doc も参照。
    pub(crate) fn media_natural_size(&self, path: &str) -> Option<[f32; 2]> {
        let mut cache = self.media_size_cache.borrow_mut();
        if let Some(cached) = cache.get(path) {
            return *cached;
        }
        let probed = motolii_media::probe(path)
            .ok()
            .map(|info| [info.width as f32, info.height as f32]);
        cache.insert(path.to_owned(), probed);
        probed
    }

    /// Document・再生位置・市松トグルのいずれかが変わった時だけ描き直す。
    /// 判定は `display_revision()`(履歴 + transient overlay の世代の組) —
    /// front が「前回の Document」を自分で持たないため。drag-to-scrub 中は
    /// overlay だけが動いて履歴の `revision()` は不変なので、`display_revision()`
    /// を見ないと drag 中の再描画が起きない。
    ///
    /// **市松は Document・playhead に依存しない表示分岐**だが、裁定141以降は
    /// 「背景を敷かない」別入力(`Engine::render_frame_without_background`)を
    /// 見せるモードなので、市松の有無だけ変わった時でも
    /// [`Self::checkerboard_preview_source`] 経由で engine をもう一度だけ回す
    /// (`Document`/`StoreView` 自体の再評価が増えるわけではない — 合成の
    /// 入力差分を取り直すだけ、裁定141「同一合成器への入力の違い」)。
    pub(crate) fn refresh_frame(&mut self) {
        let revision = self.doc.display_revision();
        let playhead = self.session.playhead;
        let checkerboard = self.checkerboard;
        let observation = self.observation;
        let resolution_cap = self.resolution_cap;
        let colors = self.tokens.colors;
        let ui_scale = self.tokens.ui_scale;

        if let Some(frame) = &self.frame {
            if frame.revision == revision && frame.playhead == playhead {
                if frame.checkerboard == checkerboard
                    && frame.observation == observation
                    && frame.resolution_cap == resolution_cap
                {
                    return;
                }
                let width = frame.width;
                let height = frame.height;
                let display = self.compute_display_source(observation, checkerboard, playhead);
                let (presenter_width, presenter_height, presenter_rgba) = match &display.full_rgba {
                    Some(rgba) => build_stage_presenter_rgba(
                        width,
                        height,
                        rgba,
                        display.checkerboard,
                        resolution_cap,
                        colors,
                        ui_scale,
                    ),
                    None => {
                        let frame = self.frame.as_ref().expect("直前の if let で確認済み");
                        build_stage_presenter_rgba(
                            width,
                            height,
                            &frame.rgba,
                            false,
                            resolution_cap,
                            colors,
                            ui_scale,
                        )
                    }
                };
                if let Some(frame) = self.frame.as_mut() {
                    frame.presenter_source = PresenterSource::Cpu(Arc::new(presenter_rgba));
                    frame.presenter_width = presenter_width;
                    frame.presenter_height = presenter_height;
                    // 世代を進める(裁定166 EXACT TARGET 1) — shader Pipeline
                    // 側の「前回アップロードした世代」との比較でこれが鍵になる。
                    // ここへ来るのは中身が実際に変わった時だけ(市松/観測/cap の
                    // いずれかが変わった時 = このブロック自体が「変化があった」
                    // 早期return の否定側)なので、無条件に+1してよい
                    // (`metrics::record_handle_creation`はもう呼ばない — Stage
                    // 描画経路から `image::Handle` 生成そのものが無くなった)。
                    frame.presenter_generation += 1;
                    frame.checkerboard = checkerboard;
                    frame.checkerboard_preview_rgba = display.checkerboard_preview_rgba;
                    frame.observation = observation;
                    frame.observation_rgba = display.observation_rgba;
                    frame.resolution_cap = resolution_cap;
                }
                return;
            }

            // ---------------------------------------------------------------
            // 裁定171 v2(M4)GPU 高速路 — playhead だけが動いた時
            // (revision 不変・市松/観測がフォールバックを要求しない組み合わせの
            // 時)。**ここでは `self.engine.render_frame` を一切呼ばない**
            // (CPU readback ゼロ、ORACLE (a) の核心)—— `frame.rgba`(export
            // 真値)は更新せず `rgba_stale` を立てるだけ(`Self::ensure_rgba_fresh`
            // doc 参照)。
            //
            // 除外条件(いずれも裁定171 v2 §0-6 のフォールバックへ委ねる):
            // - `checkerboard`: CPU 合成フォールバック(市松の GPU 化は NON-GOAL)
            // - `observation.is_some()`: 観測視点は今回まだ zero-copy 経路に
            //   繋いでいない(NON-GOALS 外だが今回のスコープでもない、
            //   `render_resolved_to_texture` は camera を差し替えられる形なので
            //   将来はここを広げられる)
            //
            // **`resolution_cap`(½/¼)はもう除外条件ではない**(残コスト調査
            // `docs/reviews/2026-08-22-residual-bottleneck-survey.md` §1-4 の
            // 修理)。旧配線は cap≠Auto を理由にここを弾いて「フル再計算」
            // (CPU readback)へフォールスルーしていた——「速くするための cap」が
            // 実際には毎フレーム readback を払う遅い経路に自ら戻る bug だった。
            // GPU 高速路はここでは常に comp ネイティブ解像度のまま描く(cap は
            // GPU 側の描画コストを一切減らさない — r1 probe 実測「comp 出力の
            // 縮小はほぼ効かない、律速は素材帯域」と整合させたまま、無駄な
            // 縮小描画を足さない)。cap の見た目(粗さ)は presenter シェーダの
            // fragment 側サンプリング粒度で表現する(`StagePresenterProgram`
            // 構築側、`stage_pane` 関数の `pixel_scale` 参照)——CPU 側の
            // `stage_presenter_rgba` 縮小と同じ「明示的な縮小」を、テクスチャの
            // 実サイズは変えずに blit 時のサンプリングだけで再現する。
            //
            // 上のどれかに当たる、または snapshot が作れない(comp 消滅等)場合は
            // 下の「フル再計算」(既存、無改造)へフォールスルーする——
            // 「無反応より安全側」(M16)を保つ。
            if frame.revision == revision && !checkerboard && observation.is_none() {
                if let Some(snapshot) = self.build_preview_snapshot(playhead) {
                    if let Some(frame) = self.frame.as_mut() {
                        frame.playhead = playhead;
                        frame.width = snapshot.comp.width;
                        frame.height = snapshot.comp.height;
                        frame.presenter_width = snapshot.comp.width;
                        frame.presenter_height = snapshot.comp.height;
                        frame.presenter_source = PresenterSource::Gpu(Arc::new(snapshot));
                        frame.presenter_generation += 1;
                        frame.rgba_stale = true;
                        frame.checkerboard_preview_rgba = None;
                        frame.observation_rgba = None;
                    }
                    return;
                }
            }
        }

        let Ok(Some(composition)) = self.doc.view().composition() else {
            self.frame = None;
            return;
        };
        let Ok(t) = RationalTime::try_from_frame(playhead, composition.fps) else {
            self.status = Some("再生位置を時刻へ写せない".to_owned());
            return;
        };

        let render_start = std::time::Instant::now();
        // **export 真値**(`RenderedFrame::rgba`)— 観測カメラ・市松に一切
        // 影響されない唯一の経路(`Engine::render_frame`)。EXACT TARGET (d) の
        // 「export 用経路は observation 中でもレンダリングカメラの絵のまま」の
        // 直接の型的裏付け: この呼び出しは `observation`/`checkerboard` を
        // 一切引数に取らない。
        let render_result = self.engine.render_frame(&self.doc.view(), t);
        metrics::record_render_frame(render_start.elapsed());
        // A-4 結線(id A-4「壊れた素材があったことを status 帯へ出す」)。
        // **API 分析の根拠**(裁定199): `Engine::layer_failures(&self) -> &[String]`
        // は「直近の render_frame/render_frame_without_background/…呼び出し
        // 1回ぶん」の隔離理由を持ち、**次の render 系呼び出しの冒頭で
        // clear される**(`layer_failures` フィールド doc 実測)。この関数は
        // 直後に `compute_display_source` を呼び、それが観測カメラ/市松用に
        // engine をもう一度回す(`checkerboard_preview_source`/
        // `observation_preview_source` 経由)ため、そこへ進む前に読まないと
        // この export 真値レンダーぶんの隔離理由を取りこぼす——**ここが
        // 「clear されるタイミングを間違えない」唯一の場所**。`to_vec()` で
        // 複製するのは、この後の `compute_display_source` が同じ
        // `&mut self.engine` を借りて内部の Vec を上書きするため(借用のまま
        // 持ち越せない)。
        let layer_failures = self.engine.layer_failures().to_vec();
        match render_result {
            Ok(rgba) => {
                let display = self.compute_display_source(observation, checkerboard, playhead);
                let (presenter_width, presenter_height, presenter_rgba) = match &display.full_rgba {
                    Some(preview) => build_stage_presenter_rgba(
                        composition.width,
                        composition.height,
                        preview,
                        display.checkerboard,
                        resolution_cap,
                        colors,
                        ui_scale,
                    ),
                    None => build_stage_presenter_rgba(
                        composition.width,
                        composition.height,
                        &rgba,
                        false,
                        resolution_cap,
                        colors,
                        ui_scale,
                    ),
                };
                // 世代は前フレームから引き継いで+1する(裁定166 EXACT TARGET 1)。
                // **ここは scrub/edit のたびに毎回通る経路**(revision か
                // playhead が変わった時点でこの分岐に落ちる — 「新規フレーム
                // だから0にリセット」ではない、`self.frame` がまだ無い最初の
                // 1回だけ0になる)。固定で0を書くと presenter_generation が
                // 常に0のまま動かなくなる事故を踏んだので明示的に注意書きした。
                let presenter_generation = self
                    .frame
                    .as_ref()
                    .map(|frame| frame.presenter_generation + 1)
                    .unwrap_or(0);
                self.frame = Some(RenderedFrame {
                    revision,
                    playhead,
                    width: composition.width,
                    height: composition.height,
                    presenter_source: PresenterSource::Cpu(Arc::new(presenter_rgba)),
                    presenter_width,
                    presenter_height,
                    presenter_generation,
                    rgba,
                    rgba_stale: false,
                    checkerboard_preview_rgba: display.checkerboard_preview_rgba,
                    checkerboard,
                    observation,
                    observation_rgba: display.observation_rgba,
                    resolution_cap,
                });
                // 裁定185(説明文は status 帯へ)。**空なら書かない** — この
                // crate の既存慣習(`Err` 枝も成功時に `status` を戻さない)を
                // そのまま踏襲し、無関係な既存メッセージを毎フレーム無言で
                // 消さない(消えるのは何か別の動作が新しい `status` を積んだ
                // 時だけ)。
                if !layer_failures.is_empty() {
                    self.status = Some(format!(
                        "描けなかった layer {}件 — {}",
                        layer_failures.len(),
                        layer_failures.join(" / ")
                    ));
                }
            }
            Err(error) => {
                // 絵が出せなくても**画面は空にしない**(M16)。理由は帯に出す。
                self.status = Some(format!("Stage を描けない: {error}"));
            }
        }
    }

    /// 裁定171 v2(M4)GPU 高速路専用。`playhead` の時刻の resolve 済み
    /// スナップショットを作る——GPU への実描画は Pipeline 側
    /// (`StagePresenterPipeline::prepare`)がやる、ここは `Document` を読んで
    /// **所有データ**へ変換するだけ(`motolii_engine::Engine::render_resolved_to_texture`
    /// の入力そのもの)。comp が無い/時刻を写せない/camera・layer が解決でき
    /// ない、のいずれかなら `None` — 呼び出し側([`Self::refresh_frame`])は
    /// フル再計算(既存の CPU 経路)へ安全側フォールバックする。
    ///
    /// **2026-08-22 でテキストも集める**(`motolii_engine` の `collect_text_documents`
    /// と同型の走査——`motolii-engine` は `Shell` の `view` を共有できないので、
    /// ここで自前にもう一度やる)。`resolved` の中の `LayerSource::Text` layer
    /// (matte 元も含めて区別しない——`layers_from_resolved` が `by_id` 越しに
    /// どちらも同じ `text_documents` map から引く)だけ `view.resolved_text_document(id, t)`
    /// を引く。
    ///
    /// **D-1 修正(2026-08-23)**: 以前は `view.text_document(id)`(静的値のみ)を
    /// 呼んでいたため、GPU 高速路(この関数の呼び出し元)だけ `text_style.{id}.*`/
    /// `text_justify` の track を無視していた——`engine::render_frame` の
    /// `collect_text_documents` は既に `resolved_text_document` を通っている
    /// (A-1b 着地分)ので、両者は同じ関数を通すのが Preview = Export の唯一の
    /// 評価経路(背骨2)。zero-copy 経路(GPU 高速路)だけ track を拾わない
    /// バグを埋める。
    pub(crate) fn build_preview_snapshot(&self, playhead: i64) -> Option<PreviewSnapshot> {
        let view = self.doc.view();
        let composition = view.composition().ok().flatten()?;
        let t = RationalTime::try_from_frame(playhead, composition.fps).ok()?;
        let camera = view.resolve_camera(t).ok()?;
        let resolved = view.resolved_layers(t).ok()?;
        let mut text_documents = HashMap::new();
        let mut shape_documents = HashMap::new();
        for layer in &resolved {
            if layer.source == LayerSource::Text {
                if let Ok(Some(document)) = view.resolved_text_document(layer.id, t) {
                    text_documents.insert(layer.id, document);
                }
            } else if layer.source == LayerSource::Shape {
                if let Ok(shapes) = view.shapes(layer.id) {
                    shape_documents.insert(layer.id, shapes);
                }
            }
        }
        Some(PreviewSnapshot {
            comp: composition.spec(),
            background: composition.background,
            camera,
            time: t,
            resolved,
            text_documents,
            shape_documents,
        })
    }

    /// 裁定171 v2(M4)EXACT TARGET 4:「readback は要求された時だけ」。GPU
    /// 高速路(`refresh_frame` の早期 return 枝)は `frame.rgba`(export 真値)
    /// を更新せず [`RenderedFrame::rgba_stale`] を立てる——このメソッドが
    /// [`Self::frame_rgba`] から呼ばれた時だけ、その場で1回 CPU readback して
    /// 追いつかせる。`checkerboard`/観測カメラ/½・¼ cap のいずれかが有効な
    /// 間は GPU 高速路自体を通らない(`rgba_stale` は常に `false` のまま)ので、
    /// このパスは「GPU 高速路を経由した後」だけ実際に readback を1回払う。
    pub(crate) fn ensure_rgba_fresh(&mut self) {
        let Some(frame) = &self.frame else { return };
        if !frame.rgba_stale {
            return;
        }
        let playhead = frame.playhead;
        let Ok(Some(composition)) = self.doc.view().composition() else {
            return;
        };
        let Ok(t) = RationalTime::try_from_frame(playhead, composition.fps) else {
            return;
        };
        match self.engine.render_frame(&self.doc.view(), t) {
            Ok(rgba) => {
                if let Some(frame) = self.frame.as_mut() {
                    frame.rgba = rgba;
                    frame.rgba_stale = false;
                }
            }
            Err(error) => {
                self.status = Some(format!("Stage を描けない: {error}"));
            }
        }
    }

    /// 市松 ON の間だけ「背景を敷かない」合成をもう一度取る(裁定141)。
    /// `checkerboard` が `false` なら常に `None`(呼び出し側は `RenderedFrame::rgba`
    /// を使う)。comp が無い/時刻を写せない/engine が描けない、のいずれかなら
    /// `None` を返し、呼び出し側は背景込みへ**安全側にフォールバック**する
    /// (無反応より、背景込みのまま出す方が M16 に近い — 市松が一時的に効かない
    /// だけで Stage 自体は空にならない)。描けなかった時は理由を status へ出す
    /// (M13)。
    pub(crate) fn checkerboard_preview_source(
        &mut self,
        checkerboard: bool,
        playhead: i64,
    ) -> Option<Vec<u8>> {
        if !checkerboard {
            return None;
        }
        let composition = self.doc.view().composition().ok().flatten()?;
        let t = RationalTime::try_from_frame(playhead, composition.fps).ok()?;
        match self
            .engine
            .render_frame_without_background(&self.doc.view(), t)
        {
            Ok(rgba) => Some(rgba),
            Err(error) => {
                self.status = Some(format!("市松プレビューを描けない: {error}"));
                None
            }
        }
    }

    /// 観測カメラ(裁定157)が有効な間だけ、その視点で再合成する
    /// (`Engine::render_frame_with_view_camera`)。`checkerboard_preview_source`
    /// と同じ「無反応より安全側フォールバック」— comp が無い/時刻を写せない/
    /// engine が描けない、のいずれかなら `None` を返し、呼び出し側は従来経路
    /// (市松/背景込み)へフォールバックする。描けなかった理由は status へ出す
    /// (M13)。
    ///
    /// **裁定160 切片10**: 計算の実体は `stage::observation_preview_source`
    /// (`&mut Engine`/`&StoreView` を明示引数で受け取る自由関数、`motolii-stage-pane`
    /// crate 側)へ移設済み — ここは `self.engine`/`self.doc.view()` を貸し、
    /// `Some(Err(_))` の枝でだけ `self.status` へ書く glue(関数名・シグネチャは
    /// 無改名、`update_settings` と同じ glue の形)。
    pub(crate) fn observation_preview_source(
        &mut self,
        observation: &ObservationCamera,
        playhead: i64,
    ) -> Option<Vec<u8>> {
        match stage::observation_preview_source(
            &mut self.engine,
            &self.doc.view(),
            observation,
            playhead,
        ) {
            None => None,
            Some(Ok(rgba)) => Some(rgba),
            Some(Err(error)) => {
                self.status = Some(error);
                None
            }
        }
    }

    /// Stage 表示(presenter)用の入力を決める。**`rgba`(export 真値)そのものには
    /// 一切触れない** — ここが返す物は表示専用の複製(`build_stage_presenter_rgba`
    /// へそのまま渡すか、`full_rgba: None` の時は呼び出し側が `RenderedFrame::rgba`
    /// を使う、既存の市松分岐と同じ形)。
    ///
    /// **優先順位**(裁定157): 観測カメラが有効なら観測視点の再合成を最優先で
    /// 使う([`Self::observation_preview_source`])。描けなければ(comp が無い等)
    /// 安全側で従来経路へフォールバックする。観測カメラが無効(`None`)なら
    /// 従来どおり市松の有無で分岐する([`Self::checkerboard_preview_source`]、
    /// 裁定141)。
    ///
    /// **既知の限界**: 観測カメラ有効中は市松プレビューを試みない
    /// (`Engine::render_frame_with_view_camera` は常に背景込み — 裁定157 の
    /// engine 側実装がそう組んである、`motolii_engine` のモジュール doc 参照)。
    /// 観測カメラは Stage 表示専用の別軸機能で、この2軸を同時に満たす engine
    /// エントリは今回のスコープ外(NON-GOALS外だが、必要になれば
    /// `render_frame_without_background_with_view_camera` 相当を engine 側へ
    /// 追加する形で拡張できる)。
    pub(crate) fn compute_display_source(
        &mut self,
        observation: Option<ObservationCamera>,
        checkerboard: bool,
        playhead: i64,
    ) -> DisplaySource {
        if let Some(observation) = observation {
            if let Some(rgba) = self.observation_preview_source(&observation, playhead) {
                return DisplaySource {
                    full_rgba: Some(rgba.clone()),
                    checkerboard: false,
                    checkerboard_preview_rgba: None,
                    observation_rgba: Some(rgba),
                };
            }
        }
        match self.checkerboard_preview_source(checkerboard, playhead) {
            Some(preview) => DisplaySource {
                full_rgba: Some(preview.clone()),
                checkerboard: true,
                checkerboard_preview_rgba: Some(preview),
                observation_rgba: None,
            },
            None => DisplaySource {
                full_rgba: None,
                checkerboard: false,
                checkerboard_preview_rgba: None,
                observation_rgba: None,
            },
        }
    }
}

#[cfg(test)]
mod text_track_preview_tests {
    use super::*;
    use motolii_store::{
        ContentTrack, FontRef, Intent, Interp, Keyframe, KeyframeTrack, LayerId, LayerMeta,
        LayerSource, LayerTiming, PathSource, PropertyId, Shape as VectorShape, ShapeNode,
        TextAlignmentOptions, TextDocument, TextDocumentStyle, TextJustify, TextStyleId, Value,
    };

    fn default_text_style() -> TextDocumentStyle {
        TextDocumentStyle {
            id: TextStyleId(0),
            font: FontRef::default(),
            size: 12.0,
            fill: [0.0, 0.0, 0.0, 1.0],
            line_height: None,
            tracking: 0.0,
            stroke_color: None,
            stroke_width: 0.0,
            stroke_over_fill: false,
            axes: Vec::new(),
            features: Vec::new(),
        }
    }

    fn default_text_document() -> TextDocument {
        TextDocument {
            content: ContentTrack::new(),
            justify: TextJustify::Left,
            wrap_size: None,
            styles: vec![default_text_style()],
            slot_id: None,
            ranges: Vec::new(),
            alignment: TextAlignmentOptions::default(),
            runs: Vec::new(),
        }
    }

    /// **D-1 の回帰柵**: `build_preview_snapshot`(GPU 高速路、`refresh_frame` の
    /// zero-copy 分岐が使う)が `text_style.{id}.size` の track を engine と
    /// 同じ経路(`StoreView::resolved_text_document`)で読むこと。修正前は
    /// `view.text_document(layer)`(静的値)を呼んでいたため、track を打っても
    /// この経路の Stage 表示だけ既定値のまま止まっていた(engine 本経路
    /// `Engine::render_frame` は既に A-1b で resolved 側を読んでいたので、
    /// 「同じ Document・同じ時刻で違う絵」が生じていた——本試験はこの2経路の
    /// 値が一致することを直接検査する)。
    #[test]
    fn build_preview_snapshot_reads_the_text_style_size_track_like_the_engine_does() {
        let (mut shell, _) = Shell::new_fixture();
        let layer = LayerId(90_001);
        shell.doc.apply(Intent::AddLayer(layer)).unwrap();
        shell
            .doc
            .apply(Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Text,
                    order: 0,
                    timing: LayerTiming::place(0, None, 60),
                },
            })
            .unwrap();
        shell
            .doc
            .apply(Intent::SetTextDocument {
                layer,
                document: default_text_document(),
            })
            .unwrap();

        let property = PropertyId::text_style_size(TextStyleId(0));
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::F64(48.0),
            interp: Interp::Hold,
            spatial: None,
        });
        shell
            .doc
            .apply(Intent::SetTrack {
                layer,
                property,
                track,
            })
            .unwrap();

        // engine が実際に読む値(A-1b が既に結線済みの本経路)。
        let expected = shell
            .doc
            .view()
            .resolved_text_document(layer, RationalTime::ZERO)
            .unwrap()
            .unwrap()
            .styles[0]
            .size;
        assert_eq!(
            expected, 48.0,
            "resolved_text_document 自体が track を読めていない(前提が壊れている)"
        );

        let snapshot = shell
            .build_preview_snapshot(0)
            .expect("fixture には comp があるはず");
        let document = snapshot
            .text_documents
            .get(&layer)
            .expect("text layer が snapshot の text_documents に無い");
        assert_eq!(
            document.styles[0].size, expected,
            "GPU 高速路(build_preview_snapshot)が engine と違う値を見ている"
        );
    }

    #[test]
    fn build_preview_snapshot_carries_shape_documents_to_gpu_path() {
        let (mut shell, _) = Shell::new_fixture();
        let layer = LayerId(90_002);
        shell.doc.apply(Intent::AddLayer(layer)).unwrap();
        shell
            .doc
            .apply(Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Shape,
                    order: 0,
                    timing: LayerTiming::place(0, None, 60),
                },
            })
            .unwrap();
        shell
            .doc
            .apply(Intent::SetShapes {
                layer,
                shapes: vec![ShapeNode::Leaf(VectorShape {
                    source: PathSource::Rectangle {
                        size: motolii_store::VectorPoint { x: 240.0, y: 135.0 },
                    },
                    ops: Vec::new(),
                    fill: Some(Shell::default_new_object_fill()),
                    stroke: None,
                })],
            })
            .unwrap();

        let snapshot = shell
            .build_preview_snapshot(0)
            .expect("fixture には comp があるはず");
        let shapes = snapshot
            .shape_documents
            .get(&layer)
            .expect("shape layer が GPU snapshot の shape_documents に無い");
        assert_eq!(shapes.len(), 1);
    }
}

use motolii_core::{CompSpec, ResolvedCamera};
use motolii_store::{LayerId, ResolvedLayer, TextDocument};

/// 裁定171 v2(M4)。GPU zero-copy 経路で使う resolve 済みスナップショット。
/// `motolii_store::Document` を直接共有できない(`re_entity_db::EntityDb` が
/// `testing` feature 外では `Clone` を持たない)ので、`Shell::build_preview_snapshot`
/// が `StoreView` から抜き出した**所有データ**をここへ積む——
/// `motolii_engine::Engine::render_resolved_to_texture` の入力そのもの。
///
    /// **`time`/`text_documents` は2026-08-22(ゼロコピー経路にも matte とテキストを
    /// 通す発注)で新設**——`render_resolved_to_texture_with_shapes` がテキストの Hold
    /// 評価と `TextDocument` 本体を要るようになったのに合わせた(`motolii_engine::Engine`
    /// の doc 参照)。`shape_documents` も同じ所有データの束として加わり、`resolved`
    /// の `Text`/`Shape` layer(matte 元も含む)に対応する本体を `StoreView` からここで
    /// 抜き出す。`resolved_layers(t)` を呼ぶのと同じ `view` から取るため、追加の
    /// Document 走査は増えない。
#[derive(Clone, Debug)]
pub(crate) struct PreviewSnapshot {
    pub(crate) comp: CompSpec,
    pub(crate) background: [f32; 4],
    pub(crate) camera: ResolvedCamera,
    pub(crate) time: RationalTime,
    pub(crate) resolved: Vec<ResolvedLayer>,
    pub(crate) text_documents: HashMap<LayerId, TextDocument>,
    pub(crate) shape_documents: HashMap<LayerId, Vec<ShapeNode>>,
}

/// Stage presenter shader へ渡す実体(裁定171 v2 M4)。
#[derive(Clone, Debug)]
pub(crate) enum PresenterSource {
    /// **高速路**(EXACT TARGET 1〜3)。`StagePresenterPipeline::prepare` が
    /// 世代ゲート越しに [`PreviewSnapshot`] を GPU 直接描画する——CPU
    /// readback をしない。
    Gpu(Arc<PreviewSnapshot>),
    /// **フォールバック**(裁定171 v2 §0-6: 市松 ON、または観測カメラ/½・¼
    /// resolution cap のように CPU 側で作った RGBA をそのまま見せたい場合)。
    /// 旧 `presenter_rgba: Arc<Vec<u8>>` と同じ形——`queue.write_texture`
    /// 経由で永続テクスチャへ上げる(裁定166 の経路、無改造で残す)。
    Cpu(Arc<Vec<u8>>),
}

/// 描き上がった1フレーム。**Document の写しではなく、描画の成果物**。
///
/// いつ捨てるかは [`Document::revision`] が決める(store 世代 + edit 位置)。
/// front が「前回の値」を自分で持たないための口がこれ。
pub(crate) struct RenderedFrame {
    /// `Document::display_revision()`(履歴 + transient overlay の世代の組)。
    /// **`revision()` ではない** — drag-to-scrub 中は overlay だけが動き、履歴の
    /// `revision()` は不変のままなので、`revision()` だけを見ていると drag 中の
    /// 再描画が起きない(transient overlay 化の要点そのもの)。
    pub(crate) revision: DisplayRevision,
    pub(crate) playhead: i64,
    pub(crate) width: u32,
    pub(crate) height: u32,
    /// Stage 表示用の実体(裁定171 v2 — 高速路/フォールバックの両対応、上記
    /// [`PresenterSource`] 参照)。**裁定166**: 旧 `handle: image::Handle` の
    /// 置き換え — shader Program の `Primitive`(`StagePresenterPrimitive`)が
    /// 毎フレーム `Arc::clone`/`clone()` するだけで、内容が変わらない限り
    /// 複製しない(`Program::draw` は描画のたびに呼ばれる、
    /// `iced_widget::shader::Program` doc 参照)。
    pub(crate) presenter_source: PresenterSource,
    pub(crate) presenter_width: u32,
    pub(crate) presenter_height: u32,
    /// `presenter_source` を新しく作り直した回数(単調増加)。shader Pipeline
    /// 側(`StagePresenterPipeline::upload`/`resolve`)が「前回描いた世代と
    /// 同じか」をこれで比較し、違う時だけ実際に描く/アップロードする
    /// (EXACT TARGET 1/2 の核心 — oracle (a) の直接の鍵)。
    pub(crate) presenter_generation: u64,
    /// `Engine::render_frame`(背景込み)の生 RGBA。**export/screenshot 真値専用**
    /// (`screenshot.rs`・`frame_rgba()`)— 通常描画(GPU 高速路)は一切読まない。
    /// **市松は絶対にここへ乗せない**し、市松トグルで一切変わらない
    /// (`settings_pane` doc「合成器が出せる」と「書き出しが吐く」は別問題、参照)。
    ///
    /// **裁定171 v2 EXACT TARGET 4**: GPU 高速路(`refresh_frame` の新しい早期
    /// return 枝)はこのフィールドを更新しない——古いままにしておき、
    /// [`rgba_stale`](RenderedFrame::rgba_stale)を立てる。`frame_rgba()` が
    /// 実際に呼ばれた時だけ [`Shell::ensure_rgba_fresh`] が追いつかせる
    /// (「readback は要求された時だけ」を型で保つ)。
    pub(crate) rgba: Vec<u8>,
    /// `rgba` が今の `playhead` を反映していない(GPU 高速路がここを飛ばした)
    /// ことを示す。`frame_rgba()`(screenshot 器具・試験専用)が呼ばれた時だけ
    /// [`Shell::ensure_rgba_fresh`] がこれを見て CPU readback を1回だけ行う。
    pub(crate) rgba_stale: bool,
    /// 市松 ON の間だけ `Some` — 裁定141「AE型の透明可視化モード」の入力
    /// (`Engine::render_frame_without_background`、背景 layer を省いた合成結果)。
    /// `presenter_rgba`(Stage 表示)と `screenshot.rs` は市松 ON の間、`rgba` の
    /// 代わりにこれへ [`settings_pane::composite_checkerboard`] を当てる。
    /// 市松 OFF の間は `None`(`rgba` をそのまま使う)。**export 真値(`rgba`)
    /// には一切影響しない** — 別フィールド。
    pub(crate) checkerboard_preview_rgba: Option<Vec<u8>>,
    /// `presenter_rgba` が市松込みで作られているか。**Document・playhead
    /// 非依存**の表示分岐なので、`revision()`/`playhead` が同じでもここが
    /// 変わっていれば `refresh_frame` は Document の再評価をせず presenter
    /// だけ作り直す(市松 ON の間は `checkerboard_preview_rgba` を取り直すため
    /// engine を1回追加で回すが、`Document`/`StoreView` の評価が増える
    /// わけではない)。
    pub(crate) checkerboard: bool,
    /// この `presenter_rgba` を作った時点の観測カメラ(裁定157)。
    /// `display_revision()`/`playhead`/`checkerboard` と同じ「キャッシュを
    /// 落とすかどうか」の鍵の一部 — `refresh_frame` の早期 return はこれも
    /// 比較する(`checkerboard` と同格の表示専用の鍵拡張)。
    pub(crate) observation: Option<ObservationCamera>,
    /// 観測カメラ有効時の Stage 表示 RGBA(`Engine::render_frame_with_view_camera`
    /// の結果そのもの)。**`rgba`(export 真値)とは別物** — `checkerboard_preview_rgba`
    /// と同じ「表示専用の複製」の形。`observation` が `None` の間は常に `None`。
    pub(crate) observation_rgba: Option<Vec<u8>>,
    /// この `presenter_rgba` を作った時点のプレビュー解像度 cap(裁定163 Stage
    /// 下縁状態帯)。**`checkerboard`/`observation` と同格の鍵拡張** —
    /// `stage_presenter_rgba` へ渡す実効スケールを変えるだけの表示専用の値
    /// なので、`revision()`/`playhead` が同じでもここが変わっていれば
    /// presenter だけ作り直す(Document・engine の再評価は増えない)。
    pub(crate) resolution_cap: stage::PreviewResolutionCap,
}

/// [`Shell::compute_display_source`] の戻り値。Stage 表示用の入力を1箇所へ
/// まとめただけの内部型 — `RenderedFrame` のフィールドへの書き戻しと
/// `build_stage_presenter_rgba` への引数の両方をこれ1つから作る(呼び出し側の
/// `refresh_frame` が2箇所(キャッシュヒット/フル再計算)で同じ分岐を書かずに
/// 済む)。
pub(crate) struct DisplaySource {
    /// `build_stage_presenter_rgba` へ渡す実体。`None` なら呼び出し側は
    /// `RenderedFrame::rgba`(export 真値)をそのまま使う(市松・観測カメラの
    /// どちらも効いていない既定の場合)。
    pub(crate) full_rgba: Option<Vec<u8>>,
    /// `full_rgba` を市松タイルで覆うかどうか(`build_stage_presenter_rgba` の
    /// 第4引数)。
    pub(crate) checkerboard: bool,
    /// `RenderedFrame::checkerboard_preview_rgba` へそのまま書き戻す値。
    pub(crate) checkerboard_preview_rgba: Option<Vec<u8>>,
    /// `RenderedFrame::observation_rgba` へそのまま書き戻す値。
    pub(crate) observation_rgba: Option<Vec<u8>>,
}

impl Shell {
    /// 描き上がったフレームの識別。同じなら描き直していない。
    pub fn frame_token(&self) -> Option<(DisplayRevision, i64)> {
        self.frame
            .as_ref()
            .map(|frame| (frame.revision.clone(), frame.playhead))
    }

    /// 市松トグルの今の状態。**screenshot 器具**が「実際に画面へ出る絵」を
    /// 再現するのに使う(`frame_rgba()` は市松を絶対に乗せない生値なので、
    /// この状態と `settings_pane::composite_checkerboard` を screenshot 側が
    /// 自分で組み合わせる必要がある — `lib.rs::build_stage_presenter_rgba` と
    /// 同じ形)。
    pub fn checkerboard_enabled(&self) -> bool {
        self.checkerboard
    }

    /// Stage 方眼シート束のトグル状態の読み口(B22、第6波)。運転席が
    /// 「View メニューを押す → トグルが反転する」を確かめる口。
    pub fn sheet_toggles(&self) -> stage::SheetToggles {
        self.sheet_toggles
    }

    /// Stage 下縁状態帯(裁定163)の今のプレビュー解像度 cap。運転席/試験が
    /// 見るための口(`checkerboard_enabled`/`observation` と同じ形)。
    pub fn resolution_cap(&self) -> stage::PreviewResolutionCap {
        self.resolution_cap
    }

    /// **裁定166 EXACT TARGET (b) の読み口**: shader Primitive へ実際に渡る
    /// RGBA の寸法。`frame_rgba()`(常に comp 解像度の export 真値)とは別に、
    /// 「今 Stage へ upload する寸法」だけを独立に確かめられるようにする
    /// (`resolution_cap()` と同じ形の試験用アクセサ)。
    pub fn stage_presenter_dims(&self) -> Option<(u32, u32)> {
        self.frame
            .as_ref()
            .map(|frame| (frame.presenter_width, frame.presenter_height))
    }

    /// Stage presenter の内容が変わった回数(裁定166 EXACT TARGET 1 の CPU 側
    /// の鍵)。shader Pipeline はこれを「前回アップロードした世代」と比較して
    /// `queue.write_texture` を省くかどうか決める(`StagePresenterPipeline::
    /// upload` 参照)。運転席/試験が「同じ内容の再描画では世代が動かない」
    /// ことを確かめる口。
    pub fn stage_presenter_generation(&self) -> Option<u64> {
        self.frame.as_ref().map(|frame| frame.presenter_generation)
    }

    /// **裁定171 v2 M4 / 残コスト調査(§1-4)の読み口**: 今の presenter が
    /// GPU 高速路(`PresenterSource::Gpu`)か CPU フォールバック
    /// (`PresenterSource::Cpu`)かを、実際に GPU device を動かさずに確かめる
    /// (`metrics::presenter_blits()` は shader Pipeline の実描画時にしか
    /// 増えない——`iced_test::simulator` は `Widget::draw` を叩かないため
    /// headless 試験では観測できない、`STAGE_PRESENTER_WGSL` doc 参照。この
    /// アクセサは `Shell::refresh_frame` が選んだ経路を `RenderedFrame` から
    /// 直接読むだけなので headless でも確かな証拠になる)。
    pub fn stage_presenter_is_gpu_backed(&self) -> Option<bool> {
        self.frame
            .as_ref()
            .map(|frame| matches!(frame.presenter_source, PresenterSource::Gpu(_)))
    }

    /// 今のデザイン値。運転席がトークン再読込を確かめる口。
    pub fn tokens(&self) -> &Tokens {
        &self.tokens
    }

    /// `ui_scale` を適用した寸法。**全 pane・全 instrument(`screenshot.rs` 含む)は
    /// ここ経由で寸法を読む** — `tokens.dims` を直接読まない。`ui_scale` を掛ける
    /// のはこの関数(=[`tokens::Dimensions::scaled`] を呼ぶ唯一の場所)だけ
    /// (発注書「適用点1箇所」)。
    pub fn dims(&self) -> Dimensions {
        self.tokens.dims.scaled(self.tokens.ui_scale)
    }

    /// 現在の色トークン。`main.rs` の `iced::application(...).theme(...)` 結線
    /// (`tokens::theme_from_colors` 参照)が窓の外から読む唯一の口 — `dims()`
    /// と対になる公開アクセサ(`tokens` フィールド自体は private のまま)。
    pub fn colors(&self) -> Colors {
        self.tokens.colors
    }
}
