
use iced::widget::{
    button, column, container, pane_grid, row, stack, text, tooltip, Shader, Space,
};
use iced::{Element, Length};

use motolii_engine::{Engine, ObservationCamera};
use motolii_store::{
    Document, LayerId, LayerSource,
};

use crate::tokens::{Colors, Dimensions};
use crate::stage_presenter::StagePresenterProgram;
use crate::{
    browser_pane, export_pane, inspector_pane, pane_layout, settings_pane, stage, Message, PresenterSource, RenderedFrame, Shell,
};

impl Shell {
    /// `stage::StageOverlay` の組み立て(裁定157・S1〜S3)。`Shell::view` が
    /// 毎フレーム呼ぶ(`build_timeline_pane` と同じ「不変な投影を作り直す」形)。
    /// comp が無ければ `None`(`stage_pane` はその時 Stage 自体を出さないので
    /// 呼ばれないが、防御的に `Option` にして panic しない、M16)。
    ///
    /// **`screenshot.rs` も呼ぶ**(`pub` — `checkerboard_enabled` 等と同じ
    /// 「screenshot 器具専用」の公開理由)— 観測中のフレーム枠を同じ計算
    /// (`stage::StageOverlay::frame_corners_on_screen`)で再現するため。
    pub fn stage_overlay(&self) -> Option<stage::StageOverlay> {
        let composition = self.composition()?;
        let comp = motolii_core::CompSpec {
            width: composition.width,
            height: composition.height,
        };
        // レンダリングカメラ(`Composition.camera`、裁定113/115/116)。
        // track が無ければ既定値 — `resolve_camera` 自体がその規約を守るので
        // ここでは `unwrap_or_default` は「時刻が引けない」時のためだけの床。
        let render_camera = self
            .time_at_playhead()
            .and_then(|t| self.doc.view().resolve_camera(t).ok())
            .unwrap_or_default();
        Some(stage::StageOverlay::new(
            comp,
            render_camera,
            self.observation,
            self.dims(),
            self.tokens.colors,
        ))
    }

    /// `stage::GizmoOverlay` の組み立て(GZ 結線、第5波)。選択 layer の
    /// [`stage::GizmoTarget`] が組めた時だけ `Some` — `stage_overlay` と同じ
    /// 「毎フレーム不変な投影を作り直す」形で、`Shell::view` が `stack!` の
    /// 最上段へ積む。
    ///
    /// `size` の出典(`GizmoTarget::size` の契約「Document が寸法を知らない
    /// 素材は呼び出し側が実寸を渡す」):
    /// - Solid = `declared_size`(Document が知っている)
    /// - Media = 実寸([`Self::media_natural_size`] — path ごとに1回だけ probe)
    /// - Null/Shape/Text/Group = 寸法の正本が shell から引けない(演算子・組版
    ///   側が決める)ため**ギズモを出さない**(Q0「触れない物を描かない」の
    ///   安全側 — RETURN 逸脱報告)。
    pub fn stage_gizmo_overlay(&self) -> Option<stage::GizmoOverlay> {
        let layer = self.session.selection?;
        let composition = self.composition()?;
        let store = self.doc.view();
        let t = self.time_at_playhead()?;
        // 「今この時刻に見えているか」+ source/declared_size の読みは resolve
        // に任せる(`gizmo_target` 自身も resolve で門をかける — 二重だが読みは
        // 安い、判定の再実装をしない)。
        let resolved = store.resolve(layer, t).ok().flatten()?;
        let size = if resolved.declared_size[0] > 0.0 && resolved.declared_size[1] > 0.0 {
            resolved.declared_size
        } else if let LayerSource::Media { path, .. } = &resolved.source {
            self.media_natural_size(path)?
        } else {
            return None;
        };
        let target = stage::gizmo_target(&store, layer, self.session.playhead, size)?;
        let comp = motolii_core::CompSpec {
            width: composition.width,
            height: composition.height,
        };
        let render_camera = self
            .time_at_playhead()
            .and_then(|t| store.resolve_camera(t).ok())
            .unwrap_or_default();
        Some(stage::GizmoOverlay::new(
            comp,
            render_camera,
            self.observation,
            target,
            self.dims(),
            self.tokens.colors,
        ))
    }

    /// `stage::SheetOverlay` の組み立て(B22、第6波、`sheets.rs` 冒頭 doc
    /// 「家(結線は次波)」— この波で結線)。`stage_overlay`/`stage_gizmo_overlay`
    /// と同じ「毎フレーム不変な投影を作り直す」形。トグル状態
    /// ([`Shell::sheet_toggles`])は View メニューが動かす。
    pub fn stage_sheet_overlay(&self) -> Option<stage::SheetOverlay> {
        let composition = self.composition()?;
        let comp = motolii_core::CompSpec {
            width: composition.width,
            height: composition.height,
        };
        let render_camera = self
            .time_at_playhead()
            .and_then(|t| self.doc.view().resolve_camera(t).ok())
            .unwrap_or_default();
        Some(stage::SheetOverlay::new(
            comp,
            render_camera,
            self.observation,
            self.sheet_toggles,
            self.dims(),
            self.tokens.colors,
        ))
    }

    /// `stage::marquee::MarqueeOverlay` の組み立て(B31、第6波、`marquee.rs`
    /// 冒頭 doc)。候補は「今この時刻に見えている全レイヤー」を
    /// `LayerMeta::order` 昇順(下→上、`marquee.rs` doc「候補列は可視レイヤー
    /// 下→上」— `StoreView::resolved_layers` の並べ方と同じ規約)で並べる —
    /// `stage_gizmo_overlay` と同じ「declared_size か media 実寸、どちらも
    /// 無ければスキップ」の門(gizmo を出さない layer は marquee の候補にも
    /// しない、Q0「触れない物を描かない」の対称)。
    pub fn stage_marquee_overlay(&self) -> Option<stage::marquee::MarqueeOverlay> {
        let composition = self.composition()?;
        let store = self.doc.view();
        let t = self.time_at_playhead()?;
        let mut ordered: Vec<(i16, stage::GizmoTarget)> = Vec::new();
        for layer in store.layers() {
            let Some(resolved) = store.resolve(layer, t).ok().flatten() else {
                continue;
            };
            let size = if resolved.declared_size[0] > 0.0 && resolved.declared_size[1] > 0.0 {
                resolved.declared_size
            } else if let LayerSource::Media { path, .. } = &resolved.source {
                let Some(size) = self.media_natural_size(path) else {
                    continue;
                };
                size
            } else {
                continue;
            };
            let Some(target) = stage::gizmo_target(&store, layer, self.session.playhead, size) else {
                continue;
            };
            ordered.push((resolved.placement.order, target));
        }
        ordered.sort_by_key(|(order, _)| *order);
        let candidates: Vec<stage::GizmoTarget> = ordered.into_iter().map(|(_, target)| target).collect();
        let comp = motolii_core::CompSpec {
            width: composition.width,
            height: composition.height,
        };
        let render_camera = store.resolve_camera(t).ok().unwrap_or_default();
        // 「今 gizmo を表示しているレイヤー」= 単一選択(gizmo は複数選択には
        // 出ない、`stage_gizmo_overlay` の `self.session.selection?` 参照)。
        let gizmo_layers: Vec<LayerId> = self.session.selection.into_iter().collect();
        Some(stage::marquee::MarqueeOverlay::new(
            comp,
            render_camera,
            self.observation,
            candidates,
            gizmo_layers,
            self.dims(),
            self.tokens.colors,
        ))
    }

    /// daemon の窓別 view dispatcher(S1、裁定182/188)。[`Shell::view`]
    /// (main 窓の絵)は**改名も改形もしない** — 既存の `.view()` 呼び出し
    /// (tests/`screenshot.rs`/`transport.rs` 等 78 箇所)を無傷に保つための
    /// 薄い分岐だけをここに置く(probe §Q3 の設計どおり)。台帳に無い Id
    /// (開閉の境目の1フレームで来うる)は main の絵 — probe の fallback と
    /// 同じ扱い。
    pub fn view_window(&self, window: iced::window::Id) -> Element<'_, Message> {
        if self.settings_window == Some(window) {
            self.view_settings_window()
        } else if self.export_window == Some(window) {
            self.view_export_window()
        } else {
            self.view()
        }
    }

    /// daemon の窓別 title(`main.rs` の `.title(...)`)。main 窓(と台帳に
    /// 無い Id)は従来どおり [`Shell::title`]、Settings 窓は "Settings"
    /// (S2 — pane 名の常設(題帯レーン)の役は OS 窓の titlebar が担う)。
    /// Export 窓(B09、第6波)は同型で "Export"。
    pub fn window_title(&self, window: iced::window::Id) -> String {
        if self.settings_window == Some(window) {
            "Settings".to_owned()
        } else if self.export_window == Some(window) {
            "Export".to_owned()
        } else {
            self.title()
        }
    }

    /// Settings 窓の絵(S2、裁定182/188)。中身は SET+(B12 第1切片)の
    /// [`settings_pane::sections::view`] — section 分け(COMPOSITION /
    /// APPEARANCE / PLAYBACK)+ Composition の W/H/FPS/尺 編集。旧
    /// `settings_pane::view` の直呼びは第5波結線で撤去済み(sections.rs 冒頭
    /// doc の手順3 — 旧関数自体の撤去は settings-pane crate 側の後続)。
    /// PLAYBACK 節の実測値(`Engine::cached_frame_count()` /
    /// `Engine::FRAME_CACHE_LIMIT`)はここで注入する(sections が GPU 系依存を
    /// 持たないための注入形)。root の余白は main 窓の [`Shell::view`] と
    /// 同じ `spacing_l` — 窓が違っても余白文法は同じ。旧・全幅ストリップに
    /// 積んでいた題帯(`panel_title_band`)は置かない: pane 名の名札の役は
    /// OS 窓の titlebar("Settings"、[`Shell::window_title`])が担う —
    /// 同じ名札を窓内へ重ねると二重表示になる。
    pub(crate) fn view_settings_window(&self) -> Element<'_, Message> {
        let dims = self.dims();
        let colors = self.tokens.colors;
        let composition = self.composition();
        container(
            settings_pane::sections::view(
                settings_pane::sections::ViewModel {
                    composition: composition.as_ref(),
                    background_draft: self.background_draft.as_ref(),
                    comp_draft: self.comp_draft.as_ref(),
                    ui_scale: self.tokens.ui_scale,
                    ui_scale_draft: self.ui_scale_draft.as_deref(),
                    preview_cache: Some(settings_pane::sections::PreviewCacheStats {
                        held_frames: self.engine.cached_frame_count(),
                        limit: Engine::FRAME_CACHE_LIMIT,
                    }),
                    auto_save_enabled: self.auto_save_enabled,
                    auto_save_config: self.auto_save_config,
                    auto_save_draft: self.auto_save_draft.as_ref(),
                },
                dims,
                colors,
            )
            .map(Message::Settings),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(dims.spacing_l)
        .into()
    }

    /// Export 窓の絵(B09、第6波、`export_pane` crate doc「shell 結線」手順2)。
    /// Settings 窓(`view_settings_window`)と同型の第2窓 — 中身は
    /// [`export_pane::view`](投影を受ける純関数)。作業範囲は TL+ の
    /// `WorkArea`(`timeline_work_area()`)を `export_pane::WorkAreaFrames` へ
    /// 座標だけ写す(pane 同士は依存しない、crate doc 参照)。
    pub(crate) fn view_export_window(&self) -> Element<'_, Message> {
        let dims = self.dims();
        let colors = self.tokens.colors;
        let composition = self.composition();
        let work_area = self.timeline_work_area().map(|area| export_pane::WorkAreaFrames {
            start: area.start,
            end: area.end,
        });
        container(
            export_pane::view(
                export_pane::ViewModel {
                    composition: composition.as_ref(),
                    out_path: self.export_out_path.as_deref(),
                    quality: self.export_quality,
                    range: self.export_range,
                    work_area,
                    progress: self.export_progress,
                },
                dims,
                colors,
            )
            .map(Message::Export),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(dims.spacing_l)
        .into()
    }

    pub fn view(&self) -> Element<'_, Message> {
        // pane が受け取るのは不変の投影だけ。
        let dims = self.dims();
        let colors = self.tokens.colors;
        let store = self.doc.view();

        // Settings は S2(裁定182/188)で OS 窓へ移住した — 旧「header 直下の
        // 全幅ストリップ」の表示分岐はここから退去([`Shell::
        // view_settings_window`] が窓の絵の正本)。main の絵は Settings 窓の
        // 開閉と無関係(Q0: 閉じた道具は木に現れない、の窓版)。
        let layout = column![self.header()];
        // 旧 MB-0/MB-1 のドロップダウン表示分岐(file_menu_open/edit_menu_open)
        // は MB-2 で廃止 — menubar の開いた menu は widget 自身の overlay
        // (`motolii_menubar` の vendored `MenuBarOverlay`)として木に現れる。

        // Browser/Inspector/Stage/Timeline は `pane_grid`(shell の pane_grid
        // 化、2026-08-22 実装レーン、`pane_layout.rs` 冒頭 doc 参照)。
        // Browser パネル(裁定162 切片 B3)は**表示だけの分岐ではなくなった**
        // — `self.panes.state` 自体が「開いていれば木にある・閉じていれば
        // 無い」を体現する(`pane_layout::build_configuration` doc、Q0)。
        // 各 pane の内容は closure の中で組み立てる(`Element` は `Clone` が
        // 無いので、外側で1回だけ作って使い回すことができない——`Fn` closure
        // は `state.panes.iter()` の各エントリごとに1回ずつ呼ばれるので、
        // 各腕がその場で組み立てれば十分・複製にはならない)。
        let browser_items = browser_pane::model::assets(&store);
        let grid = pane_grid::PaneGrid::new(&self.panes.state, |_pane, kind, _is_maximized| {
            let content: Element<'_, Message> = match kind {
                pane_layout::PaneKind::Browser => browser_pane::pane_view_with_modifiers(
                    &self.browser,
                    &browser_items,
                    // 素材差替(map 616/617)は「単一選択の時だけ」意味を持つ
                    // — 複数選択で差し替えると「どの層か」が決まらない。
                    match self.session.selected_layers.as_slice() {
                        [only] => Some(*only),
                        _ => None,
                    },
                    browser_pane::CardSelectionModifiers::new(
                        self.keyboard_modifiers.shift(),
                        self.keyboard_modifiers.command(),
                    ),
                    dims,
                    colors,
                )
                .map(Message::Browser),
                pane_layout::PaneKind::Inspector => {
                    // Inspector は canvas を使わない標準 widget 構成
                    // (inspector_pane crate 冒頭の doc comment)。Content 行
                    // (S4、#46 の穴塞ぎ)の `text_editor` だけは永続バッファ
                    // (`self.inspector_content_editor`)を借用するので、
                    // このクロージャが返す `Element<'_, _>` の寿命はもう
                    // `'static` ではない(`view_with_content_editor` doc 参照)
                    // — `content: Element<'_, Message>` の宣言どおりで問題ない。
                    let inspector_selection = inspector_pane::project(&store, &self.session)
                        .ok()
                        .flatten();
                    inspector_pane::view_with_content_editor(
                        inspector_selection.as_ref(),
                        self.inspector_field_draft.as_ref(),
                        self.inspector_name_draft.as_deref(),
                        self.inspector_speed_draft.as_deref(),
                        self.inspector_text_field_draft.as_ref(),
                        self.inspector_color_field_draft.as_ref(),
                        Some(&self.inspector_content_editor),
                        dims,
                        colors,
                    )
                    .map(Message::Inspector)
                }
                pane_layout::PaneKind::Stage => stage_pane(
                    self.frame.as_ref(),
                    self.stage_overlay(),
                    self.stage_sheet_overlay(),
                    self.stage_marquee_overlay(),
                    self.stage_gizmo_overlay(),
                    self.observation,
                    self.resolution_cap,
                    self.checkerboard,
                    dims,
                    colors,
                ),
                pane_layout::PaneKind::Timeline => {
                    // pane crate 化(裁定160 切片7)で `timeline.view()` は
                    // `Element<'static, timeline_pane::Message>` を返す
                    // (root の `Message` を pane crate から参照できないため
                    // — 循環回避)。`.map(Message::Timeline)` で1回だけ畳む
                    // (§3.1 の「pane-local Message を親が畳む」構成そのもの)。
                    // transport 帯込み(裁定180 — 下部 Play バーは撤去済み、
                    // 再生系の顔は timeline pane 上端の帯が正本)。
                    self.build_timeline_pane()
                        .with_playing(self.is_playing())
                        .view_with_transport()
                        .map(Message::Timeline)
                }
            };
            pane_grid::Content::new(content).title_bar(Self::pane_title_bar(*kind, dims, colors))
        })
        .width(Length::Fill)
        .height(Length::Fill)
        // フラット文法: リサイズグリップ = 8px(装飾余白としては使用不可、
        // `docs/reviews/2026-08-19-flat-grammar-canon-revision.md`)。
        // `spacing_m` が既にその値(8.0、`motolii-tokens-rs` 既定)——新しい
        // token を作らず既存を読む。`on_resize` の leeway=0 なので掴める幅は
        // `spacing + leeway` = `spacing_m` ちょうど(`PaneGrid::on_resize` doc)。
        .spacing(dims.spacing_m)
        // 退化(潰れて使えなくなる)パネルを防ぐ床(M13 無反応ゼロの一環)。
        .min_size(dims.row_height * 3.0)
        // Q0 適合に必須(`Message::PaneClicked` doc 参照) — pane_grid は
        // これを配線しないと本体全域が「capture されるのに無反応」になる。
        .on_click(Message::PaneClicked)
        .on_resize(0.0, Message::PaneResized)
        .on_drag(Message::PaneDragged)
        // drop 先の可視化(題帯レーン #3): drag 中、cursor が乗っている
        // 受け入れ region を pane_grid 自身が塗る(`widget/src/pane_grid.rs::
        // draw` の hovered_region 描画、fork rev 73e686e 実測)。色は既存
        // ロールのみ(S4): 面=`surface_hover`(「hover」の意味役割そのもの —
        // drag 中に cursor が乗っている受け入れ面)、縁=`focus`(操作が着地
        // する場所の合図)。split 線(picked/hovered)も `focus` — 太さは
        // `border_width * 2.0`(ln 器具の強調線と同じ導出、
        // `tests/suite/tonmana_token_fence.rs` の許容形)。
        .style(move |_theme| pane_grid::Style {
            hovered_region: pane_grid::Highlight {
                background: iced::Background::Color(colors.surface_hover),
                border: iced::Border {
                    color: colors.focus,
                    width: dims.border_width * 2.0,
                    radius: 0.0.into(),
                },
            },
            picked_split: pane_grid::Line {
                color: colors.focus,
                width: dims.border_width * 2.0,
            },
            hovered_split: pane_grid::Line {
                color: colors.focus,
                width: dims.border_width * 2.0,
            },
        });

        let main: Element<'_, Message> = layout
            .push(container(grid).width(Length::Fill).height(Length::Fill))
            .push(status_band(self.status.as_deref(), &self.doc, dims, colors))
            .spacing(dims.spacing_m)
            .padding(dims.spacing_l)
            .into()

        // Source Preview は Browser の上に責任を戻さず、main window 上の一時的な
        // owner として表示する。閉じれば元の pane tree へ戻り、Document は変えない。
        ;
        if self.source_preview.is_open() {
            let preview = self
                .source_preview
                .view(dims, colors)
                .map(Message::SourcePreview);
            stack![main, preview].into()
        } else {
            main
        }
    }

    /// pane_grid の各 pane の題帯(pane 名入りの薄い常設帯 = drag ハンドル、
    /// 2026-08-22 題帯レーン。`view()` から呼ぶ)。
    ///
    /// **必須である理由**(fork rev 73e686e の pane_grid を実測): `Content`
    /// の `Draggable` 実装(`widget/src/pane_grid/content.rs::
    /// can_be_dragged_at`)は `title_bar` が無いと常に `false` を返す —
    /// `.on_drag(...)` を配線しただけではドラッグは一切始まらない(掴む
    /// 場所が無い)。
    ///
    /// **旧 grip 帯(匿名 8px `Space`)からの置き換え理由**: (1) S6 —
    /// 見えない帯はつかめない(利用者実窓検分「レイアウト変更ができない。
    /// ハンドルが無いせいか」)。(2) **旧帯は構造的にも死んでいた** —
    /// `TitleBar::is_over_pick_area`(`title_bar.rs` 実測)は title content の
    /// bounds を pick 対象から**除外**するため、全幅 `Space` を content に
    /// していた旧帯は pick 面積ゼロ=ドラッグが一切始まらなかった
    /// (`tests/suite/pane_band_drive.rs` が red→green で検分)。
    ///
    /// 新帯: pane 名(`pane_layout::title` 正本)を左端に置き、**残りの全幅が
    /// pick area**(S1 — 帯全体が大きい的。ラベル矩形だけは上記実測理由で
    /// pick 対象外という構造的限界が残る)。pick area の hover では pane_grid
    /// 自身が `Interaction::Grab` を返す(`content.rs::grid_interaction`
    /// 実測 — カーソル予告は追加配線なしで効く)。寸法は全て tokens 由来
    /// (裁定 2026-08-22「デザイン値の外出し徹底」): 帯高=
    /// `pane_header_height`(導出は `tokens/dimensions.json` の
    /// `_note_pane_header_height`)、文字=`micro_text`(正典バンド最小段 —
    /// 本帯が最初の消費者)、左右余白=`spacing_m`(ident/cols 帯と同段)。
    /// 色は既存ロールのみ(S4): 地=`surface_raised`(旧 grip と同じ)、
    /// 文字=`text_secondary`(章立ての控えめな見出し)。リサイズは従来どおり
    /// pane 間の 8px 境界(`PaneGrid::spacing`)が担う — drag 責務はこの帯へ
    /// 一本化。
    pub(crate) fn pane_title_bar<'a>(
        kind: pane_layout::PaneKind,
        dims: Dimensions,
        colors: Colors,
    ) -> pane_grid::TitleBar<'a, Message> {
        pane_grid::TitleBar::new(
            container(
                text(pane_layout::title(kind))
                    .size(dims.micro_text)
                    .color(colors.text_secondary),
            )
            .height(Length::Fixed(dims.pane_header_height))
            .align_y(iced::alignment::Vertical::Center)
            .padding([0.0, dims.spacing_m]),
        )
        .padding(0)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(colors.surface_raised)),
            ..container::Style::default()
        })
    }

    // 旧 `panel_title_band`(pane_grid 外の Settings 全幅ストリップ用の名札帯)
    // は S2(裁定182/188)で撤去 — Settings は OS 窓になり、名札の役は窓の
    // titlebar([`Shell::window_title`])が担う(`view_settings_window` doc)。

    /// shell chrome の線化(裁定137/139 の Inspector 以外の面への展開)。
    /// 旧実装はこの帯にコンテナが無く、地(背景)も境界(hairline)も持たない
    /// 生の `row!` だった — 帯の下の Stage/Inspector 行とは `spacing_m` の
    /// gap だけで離れており「面色の塗り分けで区切る」違反ではなかったが、
    /// 帯自身が「パネル」だと分かる縁を持っていなかった。Timeline の `.tp`
    /// (transport 帯、background=panel + border-bottom hairline)と同じ
    /// grammar をここへも延長する — 新しい視覚言語の発明ではない。
    /// MB-2(裁定179 D1 根治): 旧「輪郭箱ボタンの列」(File/Edit/Undo/Redo/
    /// + Layer/Browser/Settings)を `motolii-menubar::menu_bar`(左)+icon
    /// ボタン2つ(右端 — Browser トグル/Settings、裁定187 の icon+tooltip
    /// ペア第1号)へ差し替えた。メニューの中身(全て既存 `Message` の露出)は
    /// `menu.rs::menus()` が正本、見た目は menubar crate の「枠の文法」
    /// (裁定179: 輪郭なし・hover で面)。旧 Undo/Redo 箱ボタンは廃止 —
    /// 入口は Edit メニューと Cmd+Z/Cmd+Shift+Z の2本(S6 併存)。
    pub(crate) fn header(&self) -> Element<'_, Message> {
        let dims = self.dims();
        let colors = self.tokens.colors;
        let content = row![
            motolii_menubar::menu_bar(crate::menu::menus(), dims, colors),
            Space::new().width(Length::Fill),
            // **Browser トグル**(裁定162 切片 B3、normal-map id980 — panel 型
            // 出典のみなので S6 併設要件は無い)。Icon::GridView+tooltip
            // "Browser"。
            Self::header_icon_action(
                motolii_icons::Icon::GridView,
                "Browser",
                Message::Browser(browser_pane::Message::ToggleBrowserPanel),
                dims,
                colors,
            ),
            // **Settings**(歯車)。Icon::Settings+tooltip "Settings"。旧腕は
            // SET+ 結線で `sections::Message::Legacy` が包む(sections.rs 冒頭
            // doc「結線互換の縫い目」)。
            Self::header_icon_action(
                motolii_icons::Icon::Settings,
                "Settings",
                Message::Settings(settings_pane::sections::Message::Legacy(
                    settings_pane::Message::ToggleSettingsPanel,
                )),
                dims,
                colors,
            ),
        ]
        .spacing(dims.spacing_m)
        .align_y(iced::alignment::Vertical::Center);

        // 線化 D5(裁定179 文法1): 帯の輪郭線は廃止 — `surface_panel` の面が
        // app 地から明度1段浮くことが帯の輪郭([`band_chrome_style`] doc 参照)。
        container(content)
            .width(Length::Fill)
            .height(Length::Fixed(dims.panel_header_height))
            .padding([0.0, dims.spacing_s])
            .align_y(iced::alignment::Vertical::Center)
            .style(move |_theme| band_chrome_style(dims, colors))
            .into()
    }

    /// header 右端の icon ボタン(裁定187 icon+tooltip ペア第1号)。輪郭なし・
    /// hover/press で面(裁定179 — `timeline_pane::transport` の
    /// `transport_button` と同じ枠の文法)。アイコン枠寸は旧文言ボタンの字寸
    /// (`body_text`)を [`motolii_icons::frame_px_for_glyph_px`](Material
    /// live area 比 24/20)で写した視覚同等寸 — 中間比の発明ではなく上流定数の
    /// 転写(transport のアイコン化と同じ判断)。tooltip が語(動詞名)を運ぶ
    /// (裁定187「アイコンは tooltip と対で使うのが標準」)— 面は menubar の
    /// 開いた menu と同じ `surface_raised`+hairline。
    pub(crate) fn header_icon_action<'a>(
        icon: motolii_icons::Icon,
        label: &'a str,
        message: Message,
        dims: Dimensions,
        colors: Colors,
    ) -> Element<'a, Message> {
        let glyph = motolii_icons::icon(
            icon,
            motolii_icons::frame_px_for_glyph_px(dims.body_text),
            colors.text_secondary,
        );
        let action = button(glyph)
            // 踏面はアイコンより大きく(S1、transport_button と同じ判断)。
            .padding(dims.spacing_s)
            .on_press(message)
            .style(move |_theme, status| {
                let background = match status {
                    // hover/押下: 面が浮く(輪郭は出さない — 裁定179)。
                    button::Status::Pressed | button::Status::Hovered => {
                        Some(iced::Background::Color(colors.surface_hover))
                    }
                    // 常時: 素のアイコン(輪郭なし・面なし)。
                    _ => None,
                };
                button::Style {
                    background,
                    // svg には効かない(tint が正)が、契約として ink を宣言しておく
                    // (`transport_button` と同じ注記)。
                    text_color: colors.text_secondary,
                    ..button::Style::default()
                }
            });
        tooltip(
            action,
            container(text(label).size(dims.caption_text).color(colors.text_primary))
                .padding([dims.spacing_xs, dims.spacing_s])
                .style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(colors.surface_raised)),
                    border: iced::Border {
                        color: colors.border_default,
                        width: dims.border_width,
                        radius: 0.0.into(),
                    },
                    ..container::Style::default()
                }),
            tooltip::Position::Bottom,
        )
        .gap(dims.spacing_xs)
        .into()
    }

}

// ---------------------------------------------------------------------------
// pane — **`StoreView`(不変)・`&Session`・`Tokens`(読み取り専用の意匠値)しか
// 取らない**。書ける物を持たない。`timeline_pane::TimelinePane` も同じ制約。
// ---------------------------------------------------------------------------

/// **裁定163 S 空間スコア — 発注書 EXACT TARGET**: Stage pane の下縁に1行の
/// 状態帯を追加した(S5「下縁=状態帯」・S6「状態は隠れない」の初適用)。
/// `body`(ヒーロー、S5a 占有率)は `Length::Fill` のまま、帯は自然高
/// (`stage::state_band_view` 自身が `.padding`/`.spacing` だけで決める、
/// `status_band` と同じ「明示 `.height()` を持たない」形)——ヒーローの縁へ
/// 退く低重み要素として全体高を食わない。
fn stage_pane(
    frame: Option<&RenderedFrame>,
    overlay: Option<stage::StageOverlay>,
    sheets: Option<stage::SheetOverlay>,
    marquee: Option<stage::marquee::MarqueeOverlay>,
    gizmo: Option<stage::GizmoOverlay>,
    observation: Option<ObservationCamera>,
    resolution_cap: stage::PreviewResolutionCap,
    checkerboard: bool,
    dims: Dimensions,
    colors: Colors,
) -> Element<'_, Message> {
    let body: Element<'_, Message> = match frame {
        Some(frame) => {
            // 裁定166: Stage の絵は shader Program の永続テクスチャで提示する
            // (旧 `image(frame.handle.clone())` の置き換え — image widget の
            // 非同期アップロード「その間 draw_image は何も描かない」穴を構造で
            // 消す)。letterbox は `Program::draw` が widget bounds を受け取った
            // 時点で `stage::letterboxed_rect` を呼んで組む(2箇所目の
            // letterbox 実装を作らない、EXACT TARGET 1)。
            // 残コスト調査(§1-4)の修理: GPU 高速路(`PresenterSource::Gpu`)は
            // テクスチャ自体を comp ネイティブ解像度のまま描く(§refresh_frame
            // 参照)ので、cap の「明示的な縮小」は fragment 側のサンプリング
            // 粒度で再現する(`pixel_scale` uniform、下記 WGSL `fs_main` 参照)。
            // CPU 経路(`PresenterSource::Cpu`)は `build_stage_presenter_rgba`
            // が既にテクスチャ自体を cap 相当の寸法へ縮めてアップロード済みな
            // ので、ここでさらに縮小粒度を足すと二重適用になる——常に `1.0`
            // (無 no-op、`fs_main` 側の grid はテクスチャ実寸そのものになり、
            // 通常のサンプリングと事実上同じ)を渡す。
            let pixel_scale = match &frame.presenter_source {
                PresenterSource::Cpu(_) => 1.0,
                PresenterSource::Gpu(_) => stage::effective_preview_scale(1.0, resolution_cap) as f32,
            };
            let picture: Element<'_, Message> = Shader::new(StagePresenterProgram {
                source: frame.presenter_source.clone(),
                width: frame.presenter_width,
                height: frame.presenter_height,
                generation: frame.presenter_generation,
                pixel_scale,
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
            // 観測カメラの入力(ホイール/中ボタンドラッグ)とフレーム枠 overlay
            // (裁定157)、その上に方眼シート(B22、描画のみ・入力ゼロ)、その上に
            // マーキー(B31)、最上段にギズモ(GZ 結線、第5波)。shader widget の
            // 上に重ねるだけ — 変形はしない(Stage は letterbox 貼りのまま、
            // `stage.rs` モジュール doc 参照)。
            // 裁定160 切片10: `StageOverlay::view()` は `stage::Message`
            // (pane ローカル)を返す — `.map(Message::Stage)` で root
            // `Message` へ畳んでから `picture` と同じ `stack!` へ積む
            // (`timeline.view().map(Message::Timeline)` と同じ形)。
            // 第6波: sheets/marquee も同型の独立 message → `.map` 畳み。
            // 積む順は各モジュール冒頭 doc の指定どおり
            // (`sheets.rs`/`marquee.rs`「StageOverlay の上・GizmoOverlay の
            // 下」) — gizmo が最優先で入力を capture し(GZ 契約「gizmo が
            // 勝つ」)、marquee がその補集合(`press_starts_marquee`)、sheets
            // は入力ゼロ(`pointer-events:none` の転写)なので順序は描画にしか
            // 効かない。動的な組み合わせ(4 overlay 全部が Option)は
            // `if let` を積み上げる形にした — `(Option, Option)` の全組み合わせ
            // 網羅より読みやすい(見えない overlay を stack へ積まないのは
            // 前波までと同じ「無い物は木に無い」Q0 の型)。
            let mut layered: Element<'_, Message> = picture;
            if let Some(overlay) = overlay {
                layered = stack![layered, overlay.view().map(Message::Stage)].into();
            }
            if let Some(sheets) = sheets {
                layered = stack![layered, sheets.view().map(Message::Sheet)].into();
            }
            if let Some(marquee) = marquee {
                layered = stack![layered, marquee.view().map(Message::Marquee)].into();
            }
            if let Some(gizmo) = gizmo {
                layered = stack![layered, gizmo.view().map(Message::Gizmo)].into();
            }
            layered
        }
        None => text("comp がまだ無い")
            .size(dims.body_text)
            .color(colors.text_muted)
            .into(),
    };

    // 裁定166: Auto は 1.0 固定(iced 同期アップロード予算からの自動縮小柵は
    // 撤去 — `stage_auto_scale` は無くなった、フル解像度復帰)。状態帯の
    // 実効値表示は `effective_preview_scale(1.0, cap)` へそのまま追随する
    // (発注書 EXACT TARGET 2「常時表示」・「実効値表示の追随を確認」)。
    let auto_scale = 1.0;
    let band = stage::state_band_view(observation, resolution_cap, auto_scale, checkerboard, dims, colors)
        .map(Message::Stage);

    // letterbox は neutral dark(D8: 装飾 gradient 禁止・余白は neutral)。raw 値ではなく
    // token 経由の面色 + 罫線幅。
    // **高さは `Length::Fill`**(Inspector と並ぶ `row!` の中にいるため、以前の
    // `FillPortion(3)` は `Shell::view` 側のその `row!` 自身が持つ — 2箇所で
    // portion を重ねて割合をずらさない)。
    // 線化 D5(裁定179 文法1): Stage 容器の輪郭線も透明化(幅だけ残す=幾何
    // 不変)。letterbox は neutral dark(D8)のまま app 地と同族 — Stage の
    // 範囲は上の pane 題帯(`surface_raised`)・下の状態帯・隣接 pane の
    // `surface_panel` 明度段が読ませる(AE=「暗い隙間」の viewer と同文法)。
    container(column![container(body).width(Length::Fill).height(Length::Fill), band].spacing(0.0))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(colors.surface_app)),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}


/// header 帯・status 帯の共通スタイル。線化 D5(裁定179 文法1、
/// `docs/reviews/2026-08-22-chrome-grammar-audit.md`): 帯は `surface_panel` の
/// 面で app 地(`surface_app`)から**明度1段**浮く — 輪郭線は描かない(透明
/// border で幅だけ残す=幾何不変)。参照3製品の「区切りは明度1段+間隔」の
/// shell chrome への適用(旧: 裁定139 の hairline 縁)。`pub`:
/// `tests/suite/band_line_fence.rs` が機械照合する。
pub fn band_chrome_style(dims: Dimensions, colors: Colors) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(colors.surface_panel)),
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

/// shell chrome の status 帯。線化 D5(裁定179 文法1)で旧「border のみ・背景は
/// 塗らない」(裁定139 の hairline grammar)を上書き — 帯は
/// [`band_chrome_style`](`surface_panel` の明度1段+透明 border)で header と
/// 同じ器になり、「今どこからが summary か」は線でなく面の段差が示す。
fn status_band<'a>(
    status: Option<&str>,
    doc: &Document,
    dims: Dimensions,
    colors: Colors,
) -> Element<'a, Message> {
    let layers = doc.view().layers().len();
    // 拒否・警告は status 帯の警告色(D2/D7: 文脈連動の status 帯文法)。
    // 通常の要約(layer数/edit位置)は弱文字 — 警告と同格に見せない。
    let (message, color) = match status {
        Some(status) => (status.to_owned(), colors.status_warning),
        None => (
            format!("layer {layers} / edit {}", doc.edit_head()),
            colors.text_muted,
        ),
    };
    container(text(message).size(dims.caption_text).color(color))
        .width(Length::Fill)
        .padding([dims.spacing_xs, dims.spacing_m])
        .style(move |_theme| band_chrome_style(dims, colors))
        .into()
}

// `button_style` は裁定160 切片5(pane split survey §2.4/§6)で
// `chrome::button_style` へ移設した(純粋な再配置・挙動ゼロ変更)。
