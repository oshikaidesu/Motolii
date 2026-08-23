

use motolii_store::{
    EffectId,
    EffectInstance, Interp, Intent, Keyframe, KeyframeTrack, LayerAttrsPatch, LayerId,
    LayerMeta, LayerSource, LayerTiming, Mask, MaskId, MaskMode, Path, PathSource, PathVertex, RationalTime, Shape as VectorShape, ShapeNode, Value, VectorPoint,
};
use motolii_vector::{Brush, Fill, Rgb};

use crate::{
    browser_pane, inspector_pane, tokens, Shell,
};

impl Shell {
    /// create タブのカード実体化(B36、`browser_pane::model::CreateKind` →
    /// `LayerSource::{Shape,Solid,Null,Text}`、`Message::AddLayer` と同じ
    /// 「AddLayer 合成」の流儀 — 1 `apply_all` = 1 undo)。**Rectangle/Ellipse
    /// はどちらも `LayerSource::Shape` のまま**(発注 EXACT TARGET 7 の文言
    /// どおり — 図形の中身(`Layer:shapes` component の `ShapeNode`)を
    /// 書き分ける差はこの波の範囲外、RETURN 参照)。生成後は選ぶ
    /// (`Message::AddLayer`/`duplicate_layer` と同じ規律)。
    ///
    /// **Text**(2026-08-22 利用者裁定「追加するものは Browser の中に全部
    /// 入れる」— 歌詞動画/MV ペルソナの致命的欠落への対処、
    /// `docs/reviews/2026-08-22-persona-lyric-mv.md` 参照)は他3種と違い、
    /// `LayerSource::Text` に加えて **`Intent::SetTextDocument` も同じ
    /// `apply_all` へ積む** — text layer は `TextDocument` を持たなければ
    /// Inspector の TEXT section が [`inspector_pane::default_text_document`]
    /// を表示専用で仮組みするだけで store には何も無い状態になり(`text.rs`
    /// `apply_text_document_edit` doc 参照)、engine が描く字形が無い(空
    /// layer)。既定値は Inspector の投影が使うのと同じ関数
    /// (`inspector_pane::default_text_document`)を呼ぶ ── 「既定値」の
    /// 正本を2箇所で発明しない。1 `apply_all` = 1 undo は崩れない(AddLayer/
    /// SetMeta/SetAttrs/SetTextDocument の4 Intent が1回の undo 段に入る)。
    ///
    /// **Rectangle/Ellipse は `Intent::SetShapes` も同じ `apply_all` へ積む**
    /// (2026-08-22、engine レーンが `LayerSource::Shape` を実描画へ結線した
    /// 直後の引き継ぎ — `next/engine/motolii-engine/src/shape.rs`)。それまでは
    /// `LayerSource::Shape` の layer を置いても `Layer:shapes` component が
    /// 空のまま(`StoreView::shapes` が空 `Vec` を返す)で、`rasterize_shapes`
    /// が「描く物が無い」扱いにして何も描かれなかった——**Text と同型の欠陥**
    /// (店に置いたのに中身が無い)なので、Text と同じ「同じ apply_all へ積む」
    /// 直し方を踏襲する。座標は**局所原点中心**(`motolii_vector::geom::rect`/
    /// `ellipse` の doc「局所原点中央」、`shape.rs` の `Canvas::centered` 選択
    /// doc も参照)——手で頂点を組まず `PathSource::Rectangle`/`Ellipse` に
    /// `size` だけ渡すことで、中心ずれを起こしようがない形にした(engine 側の
    /// `shape_is_centered_on_the_canvas_not_anchored_to_the_top_left` テストが
    /// この前提を実測で固定している)。寸法は Solid の既定footprint(240×135)を
    /// そのまま流用 — 「新しく置いた物の大きさ」の正本を増やさない。塗りが
    /// 無いと「まだ塗り方が決まっていない図形」として何も描かれない
    /// (`motolii_vector::Shape::new` の doc)ので、既定 [`Fill`] も併せて積む
    /// (色は Solid の既定色と同じ差し色を使い、「新規に置いた物」の見た目を
    /// 揃える)。
    pub(crate) fn create_from_card(&mut self, kind: browser_pane::model::CreateKind) {
        use browser_pane::model::CreateKind;
        let id = LayerId(self.next_layer_id());
        let source = match kind {
            CreateKind::Rectangle | CreateKind::Ellipse => LayerSource::Shape,
            CreateKind::Solid => LayerSource::Solid {
                rgba: [80, 160, 220, 255],
                width: 240,
                height: 135,
            },
            CreateKind::Null => LayerSource::Null,
            CreateKind::Text => LayerSource::Text,
        };
        let mut intents = vec![
            Intent::AddLayer(id),
            Intent::SetMeta {
                layer: id,
                meta: LayerMeta {
                    source,
                    order: id.0 as i16,
                    timing: LayerTiming::place(self.session.playhead, None, self.comp_duration()),
                },
            },
            Intent::SetAttrs {
                layer: id,
                patch: LayerAttrsPatch {
                    label_color: Some(Some(Self::label_color_for_new_layer(id))),
                    ..Default::default()
                },
            },
        ];
        if matches!(kind, CreateKind::Text) {
            intents.push(Intent::SetTextDocument {
                layer: id,
                document: inspector_pane::default_text_document(),
            });
        }
        if let Some(path_source) = Self::default_shape_path_source(kind) {
            intents.push(Intent::SetShapes {
                layer: id,
                shapes: vec![ShapeNode::Leaf(VectorShape {
                    source: path_source,
                    ops: Vec::new(),
                    fill: Some(Self::default_new_object_fill()),
                    stroke: None,
                })],
            });
        }
        let placed = self.doc.apply_all(intents);
        match placed {
            Ok(()) => self.select_single(id),
            Err(error) => self.status = Some(format!("layer を作れない: {error}")),
        }
    }

    /// [`Self::create_from_card`] の Rectangle/Ellipse 用パス源。**寸法は
    /// Solid の既定 footprint(240×135)をそのまま使う**(直上の doc 参照)。
    /// 局所原点中心は `PathSource::Rectangle`/`Ellipse` 自身の契約
    /// (`motolii_vector::geom` doc)なのでここでは何もしない——中心を自前で
    /// 計算しないことが「中心ずれを起こしようがない」の実体。
    pub(crate) fn default_shape_path_source(kind: browser_pane::model::CreateKind) -> Option<PathSource> {
        use browser_pane::model::CreateKind;
        let size = VectorPoint {
            x: 240.0,
            y: 135.0,
        };
        match kind {
            CreateKind::Rectangle => Some(PathSource::Rectangle { size }),
            CreateKind::Ellipse => Some(PathSource::Ellipse { size }),
            _ => None,
        }
    }

    /// 「新規に置いた物」の既定塗り。Solid の既定色([80,160,220])と同じ
    /// 差し色を使う——「作った直後に何か置かれたと分かる」ための色であって
    /// shape 固有の正本ではない(色を変えたければ Inspector の FILL section
    /// が引き続き正本)。
    pub(crate) fn default_new_object_fill() -> Fill {
        Fill {
            brush: Brush::Solid(Rgb {
                r: 80.0 / 255.0,
                g: 160.0 / 255.0,
                b: 220.0 / 255.0,
            }),
            ..Fill::default()
        }
    }

    /// effects タブの Mask カード実体化(裁定205 施工第2号 §A)。単一選択の
    /// 時だけ意味を持つ——Text カード直前に入った「単一選択の時だけ素材差替が
    /// 効く」判定と同型(`browser_pane::model::can_replace_source` doc、
    /// `self.session.selected_layers.as_slice()` の `[only] => Some(*only), _
    /// => None` 分岐)。**`Intent::AddMask` 1本だけを使う** — 「一覧への追加」
    /// と「shape の初期値」を同じ `write()` へ束ねる原子操作なので、
    /// `SetMasks`(一覧)+`SetTrack`(shape)の2 intent 手組みはしない
    /// (`motolii_store::document::Intent::AddMask` doc、壁7の恒久修正の狙い
    /// そのもの)。
    pub(crate) fn add_mask_to_selected_layer(&mut self) {
        let target = match self.session.selected_layers.as_slice() {
            [only] => Some(*only),
            _ => None,
        };
        let Some(layer) = target else {
            self.status = Some("マスクを追加するレイヤーを1つ選んでください".to_owned());
            return;
        };
        let masks = self.doc.view().masks(layer).unwrap_or_default();
        let id = MaskId(
            masks
                .iter()
                .map(|mask| mask.id.0)
                .max()
                .map(|max| max + 1)
                .unwrap_or(0),
        );
        let shape = self.default_mask_shape(layer);
        let placed = self.doc.apply(Intent::AddMask {
            layer,
            mask: Mask {
                id,
                mode: MaskMode::Add,
                inverted: false,
            },
            shape,
        });
        if let Err(error) = placed {
            self.status = Some(format!("マスクを追加できない: {error}"));
        }
    }

    /// 新規マスクの既定 shape(選択レイヤーの矩形いっぱい程度の素直な既定 —
    /// 発注「打った直後に『何も起きていない』ように見えるのが最悪」への対処)。
    /// `Intent::AddMask` 自体が shape 無しの中間状態を構造的に許さないので
    /// 壊れはしないが、見た目にも「何か置かれた」と分かる大きさが要る。
    ///
    /// **座標は局所原点中心**(`motolii_vector::geom::rect` の「局所原点中央の
    /// 軸平行矩形」と同じ慣習に揃える——shape の既定 shape がこの規約を持つ
    /// のと理由は同じ: layer 自身の位置は transform が持つので、shape/mask の
    /// 記述自体は常に原点基準)。**LayerSource の大半(Media/Null/Shape/Text/
    /// Group)は intrinsic な width/height を持たない**(`LayerSource` の
    /// variant 一覧、`Solid` だけが例外)——どの source でも一様に使える既定
    /// として、layer 固有の寸法ではなく **comp の解像度**を使う(comp が無い
    /// 状態は起こり得ない——マスクは既存 layer への追加なので comp は既に
    /// 設定済みのはず。念のための fallback は Solid と同じ 240×135)。
    pub(crate) fn default_mask_shape(&self, layer: LayerId) -> KeyframeTrack {
        let _ = layer; // 将来 layer 固有の寸法(Solid の width/height 等)を
                        // 使う拡張の余地を残すための明示引数——今回は使わない。
        let (width, height) = self
            .composition()
            .map(|c| (c.width as f64, c.height as f64))
            .unwrap_or((240.0, 135.0));
        let hx = width * 0.5;
        let hy = height * 0.5;
        let corner = |x: f64, y: f64| PathVertex {
            point: [x, y],
            in_tangent: [0.0, 0.0],
            out_tangent: [0.0, 0.0],
        };
        let path = Path {
            vertices: vec![
                corner(-hx, -hy),
                corner(hx, -hy),
                corner(hx, hy),
                corner(-hx, hy),
            ],
            closed: true,
        };
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::Path(path),
            interp: Interp::Hold,
            spatial: None,
        });
        track
    }

    /// effects タブの Glow カード実体化(裁定205 施工第2号 §B)。単一選択の
    /// 時だけ意味を持つ([`Self::add_mask_to_selected_layer`] と同じ選択
    /// ゲート)。**新しい原子 Intent は増やさない** — 既存の
    /// `Intent::SetEffects`(丸ごと差し替え)を「現在の一覧を読んで1件足して
    /// 書き戻す」形で使う。`AddMask` のような専用の原子操作が要らない理由:
    /// track の無い effect param は engine 側が既定値で埋めるだけ
    /// (`motolii-engine::translate_glow_params` の「track の無い param は
    /// proof の既定値」)なので、`AddMask` が壁7で踏んだ「一覧だけ更新されて
    /// 中身(shape)が無いと `resolved_masks` が `Err` になる」に相当する
    /// エラー状態がそもそも存在しない。
    pub(crate) fn apply_effect_to_selected_layer(&mut self, plugin_id: &str) {
        let target = match self.session.selected_layers.as_slice() {
            [only] => Some(*only),
            _ => None,
        };
        let Some(layer) = target else {
            self.status = Some("effect を追加するレイヤーを1つ選んでください".to_owned());
            return;
        };
        let mut effects = self.doc.view().effects(layer).unwrap_or_default();
        let id = EffectId(
            effects
                .iter()
                .map(|effect| effect.id.0)
                .max()
                .map(|max| max + 1)
                .unwrap_or(0),
        );
        effects.push(EffectInstance {
            id,
            plugin_id: plugin_id.to_owned(),
        });
        if let Err(error) = self.doc.apply(Intent::SetEffects { layer, effects }) {
            self.status = Some(format!("effect を追加できない: {error}"));
        }
    }

    /// 採番の正本は store 側([`StoreView::next_layer_id`])。**墓標を含む最大 id + 1**
    /// を返すので、削除した layer の id が再利用されない(2026-08-20 の敵対的レビュー修正)。
    pub(crate) fn next_layer_id(&self) -> u64 {
        self.doc.view().next_layer_id()
    }

    /// レイヤー差し色の自動割当(利用者裁定2026-08-21「色が足りない。Ableton は
    /// レイヤー全部に色」)。**決定論**(`LayerId % パレット長`) — Session に依存
    /// しない・undo/redo で結果が変わらない・同じ layer は常に同じ色になる。
    /// パレットの実体色は `tokens::Colors::label_palette`(トンマナ従属パレット、
    /// 発注書の候補C)にあり、ここは index を計算するだけ(色そのものはここに
    /// 埋め込まない)。生成点(`Message::AddLayer` 腕・`admit`)専用 — 既存 layer
    /// の色を後から変えるための関数ではない(その UI は後続波)。
    pub(crate) fn label_color_for_new_layer(id: LayerId) -> u8 {
        (id.0 % tokens::LABEL_PALETTE_LEN as u64) as u8
    }

    /// A01 id616/617(map「Replace selected footage item」/「Replace selected
    /// source footage for selected layers」)の shell 側実体化。
    /// `browser_pane::state::Message::ReplaceSelectedLayerSource` doc が明記する
    /// 契約どおり — pane は no-op、**supervisor が `AssetId` → `Asset` を引き、
    /// [`browser_pane::model::asset_to_layer_source`] を呼んで `Some` なら
    /// `Intent::SetSource` を dispatch する**。
    ///
    /// **API 分析の根拠**(裁定199):
    /// - `self.doc.view().asset(id) -> Result<Option<Asset>, StoreError>`
    ///   (`motolii_store::view::StoreView::asset`)— 台帳に無ければ `Ok(None)`、
    ///   store 内部エラーなら `Err`。どちらも「置換できない」なので早期 return
    ///   で畳む(`Err` を無視するのではなく、以後どのみち `status` へ理由を出す
    ///   経路が無い致命度ではないため — 台帳に無い asset_id は事実上起こらない
    ///   はずの経路で、`add_mask_to_selected_layer` 等の既存腕も store 内部
    ///   エラーを黙って諦める前例が無いのでここでは `status` に一言残す)。
    /// - `browser_pane::model::asset_to_layer_source(&Asset) -> Option<LayerSource>`
    ///   ── `path_absolute` 優先・`path_project_relative` へ落ちる・両方無ければ
    ///   `None`(crate doc「置換できる実体が無い」)。`None` は非ファイル素材
    ///   (生成系)を指す正常系であって IO 障害ではないので、専用の文言で
    ///   `status` へ出す。
    /// - `Intent::SetSource { layer, source }`(`motolii_store::document::Intent`)
    ///   ── layer に既存 `meta` が無ければ `Err`(「先に SetMeta で配置する
    ///   こと」)。この腕は `single_selected_layer` を通すので選択レイヤーは
    ///   既に配置済みのはず(未配置 layer は選択できない、既存 UI 慣習)だが、
    ///   `apply` の `Result` はそのまま `status` へ落として黙殺しない
    ///   (`add_mask_to_selected_layer`/`apply_effect_to_selected_layer` と同じ
    ///   「拒否は必ず出す」規律)。
    ///
    /// 単一選択のゲーティングは [`browser_pane::model::can_replace_source`] の
    /// 契約(「0件・2件以上の選択には何もしない」)をそのままここで踏襲する
    /// (pane 側の `Replace` ボタン自体が `single_selected_layer` 前提で描画
    /// されている、`replace_affordance_row` 参照)。
    pub(crate) fn replace_selected_layer_source(&mut self, asset_id: motolii_store::AssetId) {
        let target = match self.session.selected_layers.as_slice() {
            [only] => Some(*only),
            _ => None,
        };
        let Some(layer) = target else {
            self.status = Some("素材を置き換えるレイヤーを1つ選んでください".to_owned());
            return;
        };
        let asset = match self.doc.view().asset(asset_id) {
            Ok(Some(asset)) => asset,
            Ok(None) => {
                self.status = Some("置き換え元の素材が台帳に見つかりません".to_owned());
                return;
            }
            Err(error) => {
                self.status = Some(format!("素材を読めません: {error}"));
                return;
            }
        };
        let Some(source) = browser_pane::model::asset_to_layer_source(&asset) else {
            self.status = Some("この素材はパスを持たないため置換できません".to_owned());
            return;
        };
        if let Err(error) = self.doc.apply(Intent::SetSource { layer, source }) {
            self.status = Some(format!("素材を置き換えられません: {error}"));
        }
    }

}

