use motolii_store::{
    AssetStatus, EffectId, EffectInstance, Intent, Interp, Keyframe, KeyframeTrack,
    LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming, Mask, MaskId, MaskMode, Path,
    PathSource, PathVertex, RationalTime, Shape as VectorShape, ShapeNode, Value, VectorPoint,
};
use motolii_vector::{Brush, Fill, Rgb, StarType};

use crate::{browser_pane, inspector_pane, tokens, Shell};

impl Shell {
    /// create タブのカード実体化(B36、`browser_pane::model::CreateKind` →
    /// `LayerSource::{Shape,Solid,Null,Text}`、`Message::AddLayer` と同じ
    /// 「AddLayer 合成」の流儀 — 1 `apply_all` = 1 undo)。Rectangle/Ellipse/
    /// PolyStar は `LayerSource::Shape` と `Intent::SetShapes` を同じ
    /// `apply_all` に積み、生成後は選ぶ(`Message::AddLayer`/`duplicate_layer`
    /// と同じ規律)。
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
            CreateKind::Rectangle | CreateKind::Ellipse | CreateKind::PolyStar => {
                LayerSource::Shape
            }
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

    /// [`Self::create_from_card`] の Rectangle/Ellipse/PolyStar 用パス源。
    /// **寸法は Solid の既定 footprint(240×135)をそのまま使う**(直上の doc
    /// 参照)。局所原点中心は `PathSource::Rectangle`/`Ellipse`/`PolyStar`
    /// 自身の契約(`motolii_vector::geom` doc)なのでここでは何もしない——
    /// 中心を自前で計算しないことが「中心ずれを起こしようがない」の実体。
    /// **PolyStar**(2026-08-24「ブラウザに8枚の札」発注 §1)は半径パラメータ
    /// なので、Rectangle/Ellipse の 240×135 の半分(120×67.5)に収まる
    /// `outer_radius`/`inner_radius` を既定にする——「新しく置いた物の大きさ」
    /// の見え方を Rectangle/Ellipse と揃える判断であって、`motolii_vector`
    /// 側に既定値の正本があるわけではない(演算子側の既定値と同じ立場、
    /// `shape_operator.rs` の既定値 doc 参照)。5点の星(`StarType::Star` が
    /// `#[default]`)を採る——多角形より「新しく置いた」ことが一目で分かる。
    pub(crate) fn default_shape_path_source(
        kind: browser_pane::model::CreateKind,
    ) -> Option<PathSource> {
        use browser_pane::model::CreateKind;
        let size = VectorPoint { x: 240.0, y: 135.0 };
        match kind {
            CreateKind::Rectangle => Some(PathSource::Rectangle { size }),
            CreateKind::Ellipse => Some(PathSource::Ellipse { size }),
            CreateKind::PolyStar => Some(PathSource::PolyStar {
                points: 5.0,
                outer_radius: 67.0,
                inner_radius: 33.0,
                star_type: StarType::Star,
            }),
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
    ///
    /// **A-3 `Asset::resolve_status` の結線(D-3、2026-08-23)。API 分析の
    /// 根拠(裁定199)**: シグネチャは
    /// `fn resolve_status(&self, project_root: Option<&Path>) -> AssetStatus`
    /// (`motolii_store::asset::Asset::resolve_status` doc 参照)——`canonicalize`
    /// (syscall)を呼ぶ純粋でない関数なので**毎フレーム呼ばない**。ここでは
    /// 「利用者が Replace を押した瞬間」に1回だけ呼ぶ——`resolve_status` の
    /// doc が明言する想定読者そのもの(「Asset::resolve_status を呼んだ側
    /// だけが更新する」)で、Replace は素材を選び直す操作だから「今その
    /// パスに実体があるか」を確かめる最も自然な発火点(create-card や
    /// timeline 描画のような毎フレーム経路とは違う、離散イベント)。
    /// `project_root` は `current_path`(保存済み project ファイルの場所、
    /// `document_io.rs` が唯一の書き手)の親ディレクトリ——`AssetDraft::
    /// from_probed_source` が `path_project_relative` を作る時に使う起点と
    /// 同じ規約に揃える。`AssetStatus::Missing`/`Unreadable` は理由付きで
    /// `status` 帯へ出し早期 return(裁定185: 説明文は status 帯へ、
    /// `add_mask_to_selected_layer` 等と同じ「拒否は必ず出す」規律)——
    /// `Present`/`Unchecked`(パスを持たない生成系 asset)はそのまま置換を
    /// 続行する(`Unchecked` を「無い」とみなさない、`AssetStatus` doc の
    /// 「判断が割れたら厳しい側へ」は Missing/Unreadable の話であって
    /// Unchecked を拒否理由にする根拠ではない)。
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
        let project_root = self
            .current_path
            .as_deref()
            .and_then(std::path::Path::parent);
        match asset.resolve_status(project_root) {
            AssetStatus::Missing => {
                self.status = Some(format!(
                    "素材 \"{}\" が見つかりません(パスが動いたか削除された可能性)",
                    asset.name
                ));
                return;
            }
            AssetStatus::Unreadable { reason } => {
                self.status = Some(format!("素材 \"{}\" を読めません: {reason}", asset.name));
                return;
            }
            AssetStatus::Present { .. } | AssetStatus::Unchecked => {}
        }
        let Some(source) = browser_pane::model::asset_to_layer_source(&asset) else {
            self.status = Some("この素材はパスを持たないため置換できません".to_owned());
            return;
        };
        if let Err(error) = self.doc.apply(Intent::SetSource { layer, source }) {
            self.status = Some(format!("素材を置き換えられません: {error}"));
        }
    }

    /// 素材を台帳から外す(`A01-entry.tsv` `RemoveAsset` 行 — pane 側は
    /// `next/ui/motolii-browser-pane/src/lib.rs` の `remove_affordance_row` が
    /// 押せるボタンを既に出しており(2026-08-23)、`state.rs` の
    /// `Message::RemoveAssetFromCard(_) => {}` は意図的な no-op(「pane は
    /// Intent を呼ばない」分業、`replace_selected_layer_source` と同型)。
    /// 台帳側の `Intent::RemoveAsset { asset }`(`motolii_store::document::Intent`)
    /// は既に実装・undo込みでテスト済み(`next/core/motolii-store/tests/asset.rs`)
    /// なので、ここは選択レイヤーの有無を問わずそのまま `apply` へ渡すだけ
    /// (`RemoveAsset` のdocどおり「この素材を指す layer が居るかは見ない」—
    /// レイヤー選択のゲートは要らない、`replace_selected_layer_source` の
    /// 単一選択ゲートとは別種の Intent)。拒否は必ず `status` へ出す
    /// (`replace_selected_layer_source`/`add_mask_to_selected_layer` と同じ
    /// 「拒否は必ず出す」規律)。
    pub(crate) fn remove_asset_from_card(&mut self, asset: motolii_store::AssetId) {
        if let Err(error) = self.doc.apply(Intent::RemoveAsset { asset }) {
            self.status = Some(format!("素材を削除できません: {error}"));
        }
    }

    /// **畳んだ口**(MC-1、2026-08-23)。`Message::Browser(msg)` 腕が
    /// カード発の意図(`CreateFromCard`/`AddMaskFromCard`/
    /// `ApplyEffectFromCard`/`ReplaceSelectedLayerSource`/
    /// `RemoveAssetFromCard`)を1つずつ `if let` で横取りしていた形
    /// (5本の別々の分岐がそれぞれ `lib.rs` を書き手に引きずり、`waves.md` の
    /// 連結成分を太らせていた)を、**1関数=1呼び出し**へ畳む。
    ///
    /// `lib.rs` 側の書き方は `self.dispatch_browser_card_intent(&msg);` の
    /// 1行だけになり、**カードの意図がもう1種類増えても `lib.rs` を触らずに
    /// この match へ腕を1本足すだけで済む**(write-set が `create.rs` 1枚に
    /// 収まる)。裁定5(pane は状態を持たない)は崩さない — pane 側の
    /// `Message` 変種は元から no-op のまま(`state.rs` のORACLE)で、
    /// ここは pane が発行した「意図の宣言」を読んで `Document` へ実際に
    /// 書き込む**唯一の書き手**(`self.doc.apply`/`apply_all`)であり続ける。
    /// pane は依然として `apply_all` を一切呼ばない — 単一書き手は維持。
    /// 網羅性は `_ => {}` で意図的に緩める(裁定6「口を増やさない」—
    /// pane 側に無害な新 variant が増えても shell 側の網羅 match を割らない、
    /// `next/reference/KNOWN.md` の「wildcard 無し網羅 match」問題をこの1点は
    /// 再発させない)。
    pub(crate) fn dispatch_browser_card_intent(&mut self, msg: &browser_pane::Message) {
        match msg {
            browser_pane::Message::CreateFromCard { kind } => self.create_from_card(*kind),
            browser_pane::Message::AddMaskFromCard => self.add_mask_to_selected_layer(),
            browser_pane::Message::ApplyEffectFromCard { plugin_id } => {
                self.apply_effect_to_selected_layer(plugin_id)
            }
            browser_pane::Message::ApplyOpFromCard { op } => self.apply_op_to_selected_layer(*op),
            browser_pane::Message::ReplaceSelectedLayerSource(asset_id) => {
                self.replace_selected_layer_source(*asset_id)
            }
            browser_pane::Message::RemoveAssetFromCard(asset_id) => {
                self.remove_asset_from_card(*asset_id)
            }
            _ => {}
        }
    }
}

use crate::Message;
use iced::Task;

impl Shell {
    /// `Shell::update` から委譲される領域別 dispatch(2026-08-23 SP-1 レーン、
    /// `docs/reviews/2026-08-23-shell-split-plan.md` の続き)。**中身は無改変** —
    /// 元の巨大な `update()` match の腕をそのままここへ移しただけ(裁定どおり
    /// 移送と委譲だけ、バグ修正・整形は混ぜない)。渡された `message` がこの
    /// 領域の variant でなければ `Err(message)` で突き返す — `crate::dispatch_message`
    /// の chain-of-responsibility が次の領域dispatchへ渡す。**新しい Message 枝は
    /// ここへ腕を1本足すだけで済み、`lib.rs` は触らない**(MC-1 と同じ効能)。
    pub(crate) fn dispatch_create(&mut self, message: Message) -> Result<Task<Message>, Message> {
        let mut task = Task::none();
        match message {
            Message::AddLayer => {
                let id = LayerId(self.next_layer_id());
                // **1操作 = 1 undo**。`AddLayer`/`SetMeta`/`SetAttrs`(差し色の
                // 自動割当)を別々に書くと利用者は Undo を複数回押すことになる
                // (ui-quality-bar Q2)。
                let placed = self.doc.apply_all([
                    Intent::AddLayer(id),
                    Intent::SetMeta {
                        layer: id,
                        meta: LayerMeta {
                            source: LayerSource::Solid {
                                rgba: [80, 160, 220, 255],
                                width: 240,
                                height: 135,
                            },
                            order: id.0 as i16,
                            // 尺の決め方は Document が持つ(M4)。
                            timing: LayerTiming::place(
                                self.session.playhead,
                                None,
                                self.comp_duration(),
                            ),
                        },
                    },
                    Intent::SetAttrs {
                        layer: id,
                        patch: LayerAttrsPatch {
                            label_color: Some(Some(Self::label_color_for_new_layer(id))),
                            ..Default::default()
                        },
                    },
                ]);
                match placed {
                    Ok(()) => self.select_single(id),
                    // 拒否は必ず出す。黙って消さない。
                    Err(error) => self.status = Some(format!("layer を置けない: {error}")),
                }
            }
            Message::Browser(msg) => {
                if let browser_pane::Message::PreviewMedia(request) = &msg {
                    return Ok(self.open_source_preview(*request));
                }
                // **畳んだ口**(MC-1、2026-08-23、`create.rs::
                // dispatch_browser_card_intent` doc 参照)。カード発の意図
                // (`CreateFromCard`/`AddMaskFromCard`/`ApplyEffectFromCard`/
                // `ReplaceSelectedLayerSource`/`RemoveAssetFromCard`)を
                // ここで1つずつ `if let` で横取りしていた5本の分岐は、
                // 1関数呼び出しへ畳んだ——pane側は元から no-op(`state.rs`の
                // ORACLE)なので、`&msg` を渡して先に処理しても
                // `self.browser.update(msg)` との二重処理にはならない。
                // カードの意図がもう1種類増えても、この行は変えず
                // `create.rs` の match へ腕を1本足すだけで済む
                // (write-set が `lib.rs` を引きずらなくなる)。
                self.dispatch_browser_card_intent(&msg);
                self.browser.update(msg);
                // pane_grid 側は `browser_pane::PaneState::is_open()` が唯一の
                // 真実源(`panes` フィールド doc 参照)——ここで追随させる。
                // `set_browser_open` は同値なら no-op(`pane_layout::Layout`
                // doc)なので、`ToggleBrowserPanel` 以外の3腕(rail/検索欄)で
                // 毎回呼んでも他 split の ratio・ドラッグ配置を潰さない。
                self.panes.set_browser_open(self.browser.is_open());
            }
            other => return Err(other),
        }
        Ok(task)
    }

    /// Browser パネルの開閉状態(B3)。**screenshot 器具専用**の読み口
    /// (`checkerboard_enabled` と同じ形) — `--browser-open` CLI フラグ
    /// (`main.rs`)経由で `Message::Browser(browser_pane::Message::
    /// ToggleBrowserPanel)` を実際に通した後の状態を screenshot.rs が読める
    /// ようにする。フラグそのものは `browser::PaneState::is_open` に住む
    /// (`state.rs` 冒頭 doc「Shell 側に per-variant 分岐を増やさない」) —
    /// この口は単なる薄い委譲。
    pub fn browser_panel_open(&self) -> bool {
        self.browser.is_open()
    }

    /// 素材台帳の一覧投影(裁定162 B1)。運転席/`browser_drive.rs` が
    /// 「AdmitPaths → 台帳に載る」を確かめる口(`timeline_rows`/`markers` と
    /// 同じ形 — pane 側の projection 関数をそのまま呼ぶだけ)。
    pub fn assets(&self) -> Vec<browser_pane::AssetListItem> {
        browser_pane::model::assets_with_status(&self.doc.view(), &|id| {
            self.asset_status.get(&id).cloned()
        })
    }
}
