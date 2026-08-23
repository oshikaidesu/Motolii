use std::collections::HashMap;
use std::sync::Arc;


use motolii_engine::ObservationCamera;
use motolii_store::{
    LayerSource, RationalTime,
};

use crate::stage_presenter::build_stage_presenter_rgba;
use crate::{
    metrics, stage, DisplaySource, PresenterSource,
    PreviewSnapshot, RenderedFrame, Shell,
};

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
                        build_stage_presenter_rgba(width, height, &frame.rgba, false, resolution_cap, colors, ui_scale)
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
                let presenter_generation =
                    self.frame.as_ref().map(|frame| frame.presenter_generation + 1).unwrap_or(0);
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
    /// どちらも同じ `text_documents` map から引く)だけ `view.text_document(id)`
    /// を引く。
    pub(crate) fn build_preview_snapshot(&self, playhead: i64) -> Option<PreviewSnapshot> {
        let view = self.doc.view();
        let composition = view.composition().ok().flatten()?;
        let t = RationalTime::try_from_frame(playhead, composition.fps).ok()?;
        let camera = view.resolve_camera(t).ok()?;
        let resolved = view.resolved_layers(t).ok()?;
        let mut text_documents = HashMap::new();
        for layer in &resolved {
            if layer.source == LayerSource::Text {
                if let Ok(Some(document)) = view.text_document(layer.id) {
                    text_documents.insert(layer.id, document);
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
    pub(crate) fn checkerboard_preview_source(&mut self, checkerboard: bool, playhead: i64) -> Option<Vec<u8>> {
        if !checkerboard {
            return None;
        }
        let composition = self.doc.view().composition().ok().flatten()?;
        let t = RationalTime::try_from_frame(playhead, composition.fps).ok()?;
        match self.engine.render_frame_without_background(&self.doc.view(), t) {
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
    pub(crate) fn observation_preview_source(&mut self, observation: &ObservationCamera, playhead: i64) -> Option<Vec<u8>> {
        match stage::observation_preview_source(&mut self.engine, &self.doc.view(), observation, playhead) {
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

