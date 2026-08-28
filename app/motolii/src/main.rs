//! wraps: makepad_widgets の app host。面は `script_mod!` の宣言、再読込は makepad の
//! live reload(`--hot`)、窓の駆動は `--remote`。**ここに再読込機構を書き足さない。**
//! かつて `HotPanel` + `panel.splash` + 120ms タイマーで自前に作られており、makepad が
//! `Event::LiveEdit` で `script_mod!` を再実行すると面が空に戻る欠陥になっていた
//! (2026-08-27 撤去)。作り足すなら `wraps:` を `owns:` へ書き換えること — それが
//! 「上流に無い」という主張であり、`check.sh` の一覧に出て初めてレビューできる。

pub use makepad_widgets;

use makepad_widgets::*;
// 再生の背骨(発注 S2)。裁定276は duration probing(`admit_soundtrack`、
// `Engine::media_duration` 経由)限定のスコープで、再生セッションの結線とは別の
// 話 — front が `AudioProgram`/`PlaybackSession` を直接引くのはここだけ。
use motolii_audio::{AudioProgram, PcmCache, PlaybackSession};
use motolii_engine::{Engine, ObservationCamera};
use motolii_shell_state::Session;
use motolii_store::{
    property, AssetDraft, AssetId, Document, Fps, Interp, Intent, Keyframe, KeyframeTrack,
    LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming, Marker, PropertyId,
    RationalTime, SourceFingerprintV1, StoreView, Value,
};
use motolii_timeline_projection::{
    self as timeline_pane, stacking::restacked, waveform_bucket_range, StackDirection,
    WAVEFORM_BUCKETS,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

mod browser_surface;
mod theme_flat;
mod tokens;
mod chrome;
mod export_surface;
mod fx_stack;
mod gesture_input;
mod inspector_surface;
mod settings_surface;
mod stage_chrome;
mod stage_import;
mod stage_surface;
mod timeline_surface;
use browser_surface::{AssetKind, BrowserAsset, BrowserEditAction, BrowserSurface};
use fx_stack::{FxStack, FxStackAction, FxStackModel, FxWrite};
use inspector_surface::{InspectorSurface, InspectorSurfaceAction};
use settings_surface::{SettingsSurface, SettingsSurfaceAction};
use stage_chrome::{GizmoCommit, GizmoTarget, StageChrome, StageChromeAction, StageViewCamera};
use stage_surface::{SharedOsHandle, SharedSurfaceDesc, StagePresent, StageRoom, StageVerdict};
use timeline_surface::{
    ClipEdge, LaneFlag, TimelineEditAction, TimelineLane, TimelineMarker, TimelineModel,
    TimelinePropertyLane, TimelineSurface, TimelineSurfaceAction,
};

app_main!(App);

/// `browser_radio_groups` の `ids_array!` と同じ並び。索引が意味を運ぶので離さない。
/// `rail` の radio group。並びは `RAIL_HEADS` と対。`add_folder` は選択ではなく操作なので入れない。
macro_rules! browser_rail_ids {
    () => {
        ids_array!(
            browser_surface.browser_body.rail.all_media,
            browser_surface.browser_body.rail.video,
            browser_surface.browser_body.rail.images,
            browser_surface.browser_body.rail.audio,
            browser_surface.browser_body.rail.project,
            browser_surface.browser_body.rail.recent
        )
    };
}

const RAIL_ALL_MEDIA: usize = 0;

/// ドロップを音として引き受ける拡張子。**一覧の出所は
/// `browser_surface::asset_type_for` の audio 枝**(同じ6つ)— 新しい表を
/// 発明せず、既に台帳が `audio/*` と呼んでいる物にそのまま揃える。
const AUDIO_EXTENSIONS: &[&str] = &["wav", "mp3", "aac", "flac", "ogg", "m4a"];




script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // Source: Motolii next/reference/mocks/*.html (visual/semantic contract only).
    // This is a Makepad proof surface, not a second product state owner.

    let IconButton = ButtonFlatterIcon{
        margin: 0
        width: mod.tokens.size.menu
        height: mod.tokens.size.bar
        icon_walk: Walk{width: mod.tokens.size.icon height: mod.tokens.size.icon}
        padding: 0
        draw_icon +: {color: mod.tokens.ink.body}
    }

    let IconFlatButton = ButtonFlatIcon{
        margin: 0
        width: mod.tokens.size.menu
        height: mod.tokens.size.bar
        icon_walk: Walk{width: mod.tokens.size.icon height: mod.tokens.size.icon}
        padding: 0
        draw_icon +: {color: mod.tokens.ink.body}
    }

    // 窓の大きな面の境も同じ接地の文法: 暗い溝 + 次の面の上縁の光(Live 実測)
    let SurfaceDivider = SolidView{
        width: Fill
        height: mod.tokens.space.s2
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.desktop
        fill := View{width: Fill height: Fill}
        rim := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true new_batch: true draw_bg.color: mod.tokens.rule.rim}
    }

    // Timeline の面(TimeField / TimelineRow / RailToggle 等)はここから撤去した。
    // 描画は `timeline_surface.rs` の Rust 側へ移っており、この宣言群は誰も参照して
    // いなかった — 同じ意味の正本が2つあると、片方だけ直して食い違う。

    // Studio: kinds are Fill Views, then `Kind := Kind {}` on the Dock instance.
    let BrowserPane = View{
        width: Fill
        height: Fill
        browser_surface := BrowserSurface{}
    }

    let StagePane = View{
        width: Fill
        height: Fill
        stage_chrome := StageChrome{}
    }

    // FX STACK は `InspectorSurface` の中ではなく**その下の兄弟**として置く。
    // `FxStack` の param 欄は `ScrubValue`(`inspector_surface.rs` が登録する型)を
    // 使うので、`fx_stack` の script_mod は inspector より**後**に走らねばならず、
    // その順序では `InspectorSurface` の宣言の中から `FxStack` を名指せない
    // (未登録名の参照は葉が落ちる)。ここは全 mod の登録後に評価されるので、
    // 両方が揃っている唯一の場所である。
    let InspectorPane = View{
        width: Fill
        height: Fill
        flow: Down
        inspector_surface := InspectorSurface{height: Fit}
        fx_stack := FxStack{}
    }

    let ExportPane = View{
        width: Fill
        height: Fill
        export_surface := ExportSurface{}
    }

    let SettingsPane = View{
        width: Fill
        height: Fill
        settings_surface := SettingsSurface{}
    }

    let ChromePane = View{
        width: Fill
        height: Fill
        chrome_gallery := ChromeGallery{}
    }

    let TimelinePane = View{
        width: Fill
        height: Fill
        flow: Down
        transport := SolidView{
            width: Fill
            height: mod.tokens.size.transport
            flow: Right
            align: Align{x: 0.5 y: 0.5}
            show_bg: true
            new_batch: true
            draw_bg.color: mod.tokens.face.panel
            play_toggle := ButtonFlatIcon{
                width: 26.0 * mod.tokens.scale
                height: mod.tokens.size.status
                icon_walk: Walk{width: mod.tokens.size.icon height: mod.tokens.size.icon}
                padding: Inset{left: 0 right: 0}
                draw_bg.color: #4f4f4f
                draw_bg.border_size: 0.0
                draw_icon +: {svg: crate_resource("self://resources/icons/play.svg") color: mod.tokens.accent.on}
            }
        }
        timeline_surface := TimelineSurface{
            width: Fill
            height: Fill
        }
    }


    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(1440, 900)
                window.title: "Motolii Makepad Panel"
                body +: {
                    panel := SolidView{
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.well

            chrome := SolidView{
                width: Fill
                height: mod.tokens.size.chrome
                flow: Right
                align: Align{y: 0.5}
                padding: Inset{left: mod.tokens.space.s5 right: mod.tokens.space.s5}
                show_bg: true
                new_batch: true
                draw_bg +: { color: mod.tokens.face.area }

                brand := SolidView{
                    width: Fit
                    height: mod.tokens.size.menu
                    flow: Right
                    align: Align{y: 0.5}
                    spacing: mod.tokens.space.s3
                    mark := Icon{width: mod.tokens.size.icon_lg height: mod.tokens.size.icon_lg icon_walk: Walk{width: mod.tokens.size.icon_lg height: mod.tokens.size.icon_lg} draw_icon +: {svg: crate_resource("self://resources/icons/motolii.svg") color: mod.tokens.ink.body}}
                    name := InkLabel{text: "MOTOLII" width: Fit padding: Inset{right: mod.tokens.space.s3} draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.lg}}
                }
                file := ButtonFlatter{text: "File" width: 42.0 * mod.tokens.scale height: mod.tokens.size.menu draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.md}}
                edit := ButtonFlatter{text: "Edit" width: 42.0 * mod.tokens.scale height: mod.tokens.size.menu draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.md}}
                layer := ButtonFlatter{text: "Layer" width: 50.0 * mod.tokens.scale height: mod.tokens.size.menu draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.md}}
                view := ButtonFlatter{text: "View" width: 42.0 * mod.tokens.scale height: mod.tokens.size.menu draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.md}}
                spacer := SolidView{width: Fill height: 1}
                project := InkLabel{
                    text: "Untitled / Motion Study"
                    width: Fit
                    draw_text.color: mod.tokens.ink.glyph
                    draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}
                }
                // アイコンの言っている事をそのまま行う2本。splash の `on_click` からは
                // 届かない — Dock に `script_call` の口が無いのでタブ選択も splitter も
                // script 側から呼べず、状態行へ文言を書くだけの「見た目のボタン」に
                // なっていた。押下は Rust 側(`App::handle_actions`)で受ける
                // (`toggle_playback` / `radio_button_set` と同じ経路)
                browser_toggle := IconButton{width: 26.0 * mod.tokens.scale draw_icon +: {svg: crate_resource("self://resources/icons/panels.svg")}}
                settings := IconButton{width: 26.0 * mod.tokens.scale draw_icon +: {svg: crate_resource("self://resources/icons/filter.svg")}}
            }

            chrome_surface_divider := SurfaceDivider{}

            dock := DockFlat{
                width: Fill
                height: Fill
                // 境界は「同値の面の間の暗線」ではなく「暗い溝 + 次のパネルの縁の光」
                // (Live 実測: 面 #3e3f3c → 溝 ~5pt #2b2b29 → 明縁 1-2px #adaeae → 面)。
                // 溝の深さと縁のハイライトが板を接地させる — 暗線だけだと浮いて見える
                splitter: Splitter{
                    draw_bg +: {
                        color_bg: mod.tokens.face.desktop
                        pixel: fn() {
                            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                            sdf.clear(self.color_bg)
                            // 明縁は「後ろ側」= 次のパネルの先頭端。is_vertical>0.5 は
                            // 縦棒(列を割る = 横方向の splitter)なので rect_size.x が
                            // 細い側 — 縁は右端(次パネルの左端)に立てる。
                            // else は横棒(行を割る)で rect_size.y が細い側 — 縁は下端。
                            // (以前はこの2本が入れ替わっており、縁が細い辺の外へ
                            // 落ちて可視スキャン線に一切かからなかった — それが
                            // 「override が効いていない」ように見えた本当の原因)
                            // SDF のアンチエイリアスは幅1の矩形だと両端のぼかしが
                            // 中央で重なり、塗りが半分以下しか乗らない(実測:
                            // #2a2a2a→#9c9c9c のはずが #525252 止まりだった)。
                            // 縁だけ aa を締めてハードエッジに近づける
                            sdf.aa = sdf.aa * 4.0
                            if self.is_vertical > 0.5 {
                                sdf.rect(self.rect_size.x - 1.0, 0.0, 1.0, self.rect_size.y)
                            } else {
                                sdf.rect(0.0, self.rect_size.y - 1.0, self.rect_size.x, 1.0)
                            }
                            // rule.rim #x9c9c9c。shader 本体は mod.* を解決できないので
                            // リテラルで持つ(値の正本は tokens 側)
                            sdf.fill(vec4(0.612, 0.612, 0.612, 1.0))
                            sdf.aa = sdf.aa * 0.25
                            // 掴みの棒は既定のまま(hover まで不可視)
                            if self.is_vertical > 0.5 {
                                sdf.box(self.splitter_pad, self.rect_size.y * 0.5 - self.bar_size * 0.5, self.rect_size.x - 2.0 * self.splitter_pad, self.bar_size, self.border_radius)
                            } else {
                                sdf.box(self.rect_size.x * 0.5 - self.bar_size * 0.5, self.splitter_pad, self.bar_size, self.rect_size.y - 2.0 * self.splitter_pad, self.border_radius)
                            }
                            return sdf.fill_keep(mix(self.color, mix(self.color_hover, self.color_drag, self.drag), self.hover))
                        }
                    }
                }
                // makepad の丸角オーバーレイは makepad の顔。Ableton は直角
                round_corner.border_radius: 0.0
                // 既定の 33pt は、この密度の中では帯だけが太い。タブは掴む所なので
                // 消さずに詰める(makepad 側の下限は 25pt)
                tab_bar: TabBarFlat{
                    height: mod.tokens.size.tab_bar
                    // 帯の面は完全に透明。浮くのはタブの札だけで、下の chrome は読めたまま。
                    // 既定は `color_2: #0000` へのグラデーションで、下の文字が中途半端に
                    // 抜ける — 一番読みにくい状態なので、不透明か透明かに振り切る。
                    // ドロップ判定は幾何(`is_over_tab_bar`)なので見えなくても効く
                    draw_bg.color: #x00000000
                    // 既定の tab は帯より高く(36 > 25)、下がはみ出て切れる。
                    // align.y は元から中央なので、直すのは箱の高さ
                    PermanentTab := TabFlat{
                        height: mod.tokens.size.tab_bar
                        padding: Inset{left: mod.tokens.space.s5 right: mod.tokens.space.s5}
                        draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}
                    }
                }

                root := DockSplitter{
                    axis: SplitterAxis.Vertical
                    align: SplitterAlign.FromB(mod.tokens.size.pane)
                    a: @top_split
                    b: @timeline_tabs
                }

                top_split := DockSplitter{
                    axis: SplitterAxis.Horizontal
                    align: SplitterAlign.FromA(mod.tokens.size.pane)
                    a: @browser_tabs
                    b: @center_split
                }

                center_split := DockSplitter{
                    axis: SplitterAxis.Horizontal
                    align: SplitterAlign.FromB(mod.tokens.size.pane)
                    a: @stage_tabs
                    b: @inspector_tabs
                }

                browser_tabs := DockTabs{
                    tabs: [@browser]
                    selected: 0
                    closable: false
                    float_tab_bar: false
                }

                stage_tabs := DockTabs{
                    tabs: [@stage]
                    selected: 0
                    closable: false
                    float_tab_bar: false
                }

                inspector_tabs := DockTabs{
                    tabs: [@inspector @export @settings @chrome_tab]
                    selected: 0
                    closable: false
                    float_tab_bar: false
                }

                timeline_tabs := DockTabs{
                    tabs: [@timeline]
                    selected: 0
                    closable: false
                    float_tab_bar: false
                }

                browser := DockTab{
                    name: "Browser"
                    template: @PermanentTab
                    kind: @BrowserPane
                }

                stage := DockTab{
                    name: "Stage"
                    template: @PermanentTab
                    kind: @StagePane
                }

                inspector := DockTab{
                    name: "Inspector"
                    template: @PermanentTab
                    kind: @InspectorPane
                }

                export := DockTab{
                    name: "Export"
                    template: @PermanentTab
                    kind: @ExportPane
                }

                settings := DockTab{
                    name: "Settings"
                    template: @PermanentTab
                    kind: @SettingsPane
                }

                chrome_tab := DockTab{
                    name: "Chrome"
                    template: @PermanentTab
                    kind: @ChromePane
                }

                timeline := DockTab{
                    name: "Timeline"
                    template: @PermanentTab
                    kind: @TimelinePane
                }

                BrowserPane := BrowserPane{}
                StagePane := StagePane{}
                InspectorPane := InspectorPane{}
                ExportPane := ExportPane{}
                SettingsPane := SettingsPane{}
                ChromePane := ChromePane{}
                TimelinePane := TimelinePane{}
            }

            status_surface_divider := SurfaceDivider{}

            status := InkLabel{
                text: "READY  ·  RERUN STAGE  ·  FRAME 900 / 1800"
                width: Fill
                height: mod.tokens.size.status
                padding: Inset{left: mod.tokens.space.s5}
                draw_text.color: mod.tokens.ink.faint
                draw_text.text_style: theme.font_code{font_size: mod.tokens.text.sm}
            }

                    }
                }
            }
        }
    }
}

/// Makepad view adapter. Writes go to Document / Session; pixels come from Engine.
/// widgets never keep a second Document.
struct BackendBridge {
    doc: Document,
    session: Session,
    engine: Engine,
    frame: Option<(u32, u32, Vec<u8>)>,
    status: Option<String>,
    playing: bool,
    present: StagePresent,
    stage_texture: Option<Texture>,
    stage_gpu: Option<wgpu::Texture>,
    /// path → 素材の全長ぶんの波形 (min, max)。**取り込みの1回だけ**埋める。
    /// `motolii_media::waveform_peaks` は ffmpeg を素材の端から端まで走らせる I/O
    /// なので、投影(`timeline_model`、ズーム・スクラブのたびに走る)からは絶対に
    /// 呼ばない — 憲法3「プレイヘッドのカクつきは合否そのもの」。
    ///
    /// **空の Vec は「音が無い」の記録**でもある(静止画・音無し動画)。もう一度
    /// 聞き直さないために、失敗も空として憶えておく。
    waveforms: HashMap<String, Vec<(f32, f32)>>,
    /// path → 素材の総尺を comp の fps で数えたフレーム数。波形をトリム窓へ
    /// 切り出す時の分母(`waveform_bucket_range`)。
    source_frames: HashMap<String, i64>,
    /// 実デバイス再生セッション(発注 S2)。`playing` かつデバイスが開けた時だけ
    /// `Some` — 再生中の playhead の正はこの `clock()`(P07-C1D・憲法3)。`None`
    /// のまま再生することもある(TARGET 6: デバイス不可・comp 無しでの
    /// 静かな劣化、`start_audio_session` 参照)。Drop でストリームが閉じる。
    audio_session: Option<PlaybackSession>,
    /// `AudioProgram::from_view` が要る `(識別キー, ordinal) → 正準PCM` キャッシュ。
    /// 再生の開始/停止のたびに `AudioProgram` を組み直しても、同じ素材の decode を
    /// 使い回す(`program.rs` の doc が指す `caches` そのもの)。
    audio_pcm_cache: HashMap<(String, u32), Arc<PcmCache>>,
}

/// Stage ギズモ(S20 TARGET4)の書き手。「AE の指の規約」
/// (`fx_stack.rs::apply` の doc — キーが無い property は時刻0に Hold で静止値、
/// キーがある property はプレイヘッドへ直前の interp を写して打つ)をそのまま踏襲する
/// 自由関数 — `BackendBridge::apply_gizmo_commit` から成分ごとに1回呼ぶ。
fn write_gizmo_component(
    store: &StoreView<'_>,
    layer: LayerId,
    fps: Fps,
    playhead: i64,
    property_name: &str,
    value: Value,
) -> Result<Intent, String> {
    let property = PropertyId::new(property_name).map_err(|error| error.to_string())?;
    let Ok(t0) = RationalTime::try_from_frame(0, fps) else {
        return Err("Stage: frame 0 does not map to a time".to_owned());
    };
    let mut track = store.track(layer, &property).ok().flatten().unwrap_or_default();
    let (t, interp) = if track.keys().is_empty() {
        (t0, Interp::Hold)
    } else {
        let Ok(t) = RationalTime::try_from_frame(playhead.max(0), fps) else {
            return Err("Stage: the playhead does not map to a time".to_owned());
        };
        // 直前のキーから写す。区間イージングを勝手に発明しない(`fx_stack.rs` と同じ規約)。
        let interp = track
            .keys()
            .iter()
            .rev()
            .find(|key| key.t <= t)
            .or_else(|| track.keys().first())
            .map(|key| key.interp)
            .unwrap_or(Interp::Hold);
        (t, interp)
    };
    track.insert(Keyframe {
        t,
        value,
        interp,
        spatial: None,
    });
    Ok(Intent::SetTrack {
        layer,
        property,
        track,
    })
}

/// Inspector の行id(`"position"`/`"rotation"`/`"opacity"`)を書く先の `PropertyId` へ写す。
/// **vec 形が既定**(裁定61) — position の split(x/y 別 track)は値セルの成分ごとの話で、
/// 行いっぺんの ToggleKey/SetInterp は `write_gizmo_component` と同じ vec 側へ書く
/// (この app がドラッグで実際に作る唯一の形)。値セルの投影(split 判定込み)は
/// まだ口が無い("TRANSFORM の値投影は口が無いので次の波" — `selection_summary` の doc)。
fn inspector_row_property(prop: &str) -> Option<&'static str> {
    match prop {
        "position" => Some(property::POSITION),
        "rotation" => Some(property::ROTATION),
        "scale" => Some(property::SCALE),
        "opacity" => Some(property::OPACITY),
        _ => None,
    }
}

/// `track` の中で `t` を含む区間の開始キー index。区間が無ければ `None`
/// (`KeyframeTrack::eval` と同じ二分探索 — 「keys[i].interp が keys[i]→keys[i+1] を
/// 決める」という同じ規約を読み書き両方で使う)。
fn segment_start_index(track: &KeyframeTrack, t: RationalTime) -> Option<usize> {
    let keys = track.keys();
    let last = keys.len().checked_sub(1)?;
    if last == 0 || t < keys[0].t || t >= keys[last].t {
        return None;
    }
    Some(match keys.binary_search_by(|k| k.t.cmp(&t)) {
        Ok(i) => i,
        Err(i) => i - 1,
    })
}

/// Stage の空きクリックの当たり判定(S20 TARGET5、室の名前固定 — 口を変えずに
/// 中身だけ深められる)。**v1実装**: 寸法が分かるレイヤー
/// (`resolved.declared_size != [0,0]` — `motolii_store::LayerSource::declared_size`
/// が `Some` を返すのは実質 Solid のみ、Media は engine の probe を要するので
/// front からは `[0,0]` にしか見えない、Null/Shape/Text/Group はそもそも寸法という
/// 概念を持たない)の矩形(`declared_size` × `placement.transform`)を全走査して、
/// 当たった中で重ね順(`order`)が最も手前の物を返す。GPU instance picking への
/// 深化はこの関数の中身を差し替えるだけで、呼び手(`StageChromeAction::StagePicked`
/// の受け手)は変えなくてよい。
fn stage_pick(store: &StoreView<'_>, t: RationalTime, comp_point: [f32; 2]) -> Option<LayerId> {
    let point = glam::Vec2::new(comp_point[0], comp_point[1]);
    let mut best: Option<(LayerId, i16)> = None;
    for layer in store.layers() {
        let Ok(Some(resolved)) = store.resolve(layer, t) else {
            continue;
        };
        let [w, h] = resolved.declared_size;
        if w <= 0.0 || h <= 0.0 {
            continue;
        }
        let transform = resolved.placement.transform;
        let corners = [
            transform.transform_point2(glam::Vec2::new(0.0, 0.0)),
            transform.transform_point2(glam::Vec2::new(w, 0.0)),
            transform.transform_point2(glam::Vec2::new(w, h)),
            transform.transform_point2(glam::Vec2::new(0.0, h)),
        ];
        if !point_in_quad(point, corners) {
            continue;
        }
        let order = resolved.placement.order;
        if best.map(|(_, best_order)| order > best_order).unwrap_or(true) {
            best = Some((layer, order));
        }
    }
    best.map(|(layer, _)| layer)
}

/// 点が(任意の巻き順の)凸四角形の内側にあるか。全4辺の符号付き面積が同符号
/// (または0)なら内側 — `stage_chrome.rs` の三角形塗りつぶし判定(barycentric の
/// 符号)と同じ手口を辺の数だけ増やしただけ。
fn point_in_quad(p: glam::Vec2, corners: [glam::Vec2; 4]) -> bool {
    let mut sign = 0.0_f32;
    for i in 0..4 {
        let a = corners[i];
        let b = corners[(i + 1) % 4];
        let edge = b - a;
        let to_point = p - a;
        let cross = edge.x * to_point.y - edge.y * to_point.x;
        if cross.abs() > f32::EPSILON {
            if sign == 0.0 {
                sign = cross.signum();
            } else if cross.signum() != sign {
                return false;
            }
        }
    }
    true
}

enum TimelineUpdate {
    None,
    Stage(String),
    ModelAndStage(String),
    /// Timeline の投影だけを引き直す。**Stage は触らない** — 選択のように
    /// 絵を変えない操作で毎回 Stage を焼き直すのは、意図が名指していない物を
    /// 一緒に動かすこと(裁定271)。`render_frame` は `Session` を受け取らないので
    /// 選択は絵に写らない = 焼き直す理由が無い。
    Model(String),
    Status(String),
}

/// A one-slot mailbox for expensive projections. Producers may run at pointer
/// frequency; the consumer runs at display frequency and only observes the
/// newest request. The payload can later become a `StageSurfaceSlot` without
/// changing Timeline or App event routing.
#[derive(Default)]
struct LatestFrameRequest {
    pending: bool,
}

impl LatestFrameRequest {
    fn request(&mut self) -> bool {
        std::mem::replace(&mut self.pending, true)
    }

    fn take(&mut self) -> bool {
        std::mem::take(&mut self.pending)
    }
}

impl BackendBridge {
    fn new_fixture() -> Self {
        let built = motolii_fixture::build();
        Self {
            session: Session {
                playhead: built.playhead,
                selection: Some(built.selected),
                selected_layers: vec![built.selected],
                ..Session::default()
            },
            doc: built.doc,
            engine: Engine::new().expect("GPU を用意できない"),
            frame: None,
            status: Some(built.status),
            playing: false,
            present: StagePresent::FallbackCpu,
            stage_texture: None,
            stage_gpu: None,
            waveforms: HashMap::new(),
            source_frames: HashMap::new(),
            audio_session: None,
            audio_pcm_cache: HashMap::new(),
        }
    }

    fn display_name(name: &str) -> String {
        match name {
            "タイトルロゴ" => "Title Logo",
            "メインボーカル映像" => "Main Vocal",
            "Bロール_街並み" => "B-roll City",
            "1番Aメロ歌詞" => "Verse A Lyrics",
            "Bメロ歌詞" => "Pre-chorus Lyrics",
            "ダンスカット" => "Dance Cut",
            "サビ歌詞" => "Chorus Lyrics",
            "グリッチトランジション" => "Glitch Transition",
            "2番Aメロ歌詞" => "Verse 2 Lyrics",
            "波形ビジュアライザ" => "Wave Visualizer",
            "リリックモーション背景" => "Lyric Motion BG",
            "ラスサビ歌詞" => "Last Chorus Lyrics",
            "Bロール_夜景" => "B-roll Night",
            "エンドカード" => "End Card",
            "クレジット" => "Credits",
            other => other,
        }
        .to_owned()
    }

    fn timeline_model(&self) -> TimelineModel {
        let store = self.doc.view();
        let mut rows = timeline_pane::rows(&store, &self.session);
        // Timeline top means Stage front. Both are derived from the same
        // `LayerMeta.order`; no independent lane-order state exists here.
        rows.sort_by(|left, right| {
            let left_order = store
                .meta(left.id)
                .ok()
                .flatten()
                .map(|meta| meta.order)
                .unwrap_or(i16::MIN);
            let right_order = store
                .meta(right.id)
                .ok()
                .flatten()
                .map(|meta| meta.order)
                .unwrap_or(i16::MIN);
            right_order
                .cmp(&left_order)
                .then_with(|| right.id.cmp(&left.id))
        });
        drop(store);

        // 音のある行だけ、**クリップが見せている区間**の波形を切り出して渡す。
        // 切り出しの規則(`source_in`/`speed` からバケット範囲へ)は projection が
        // 持っている — front で書き直すと同じ規則の家が2つになる。
        // ここは HashMap を引くだけで I/O をしない(`waveforms` の doc 参照)。
        let mut waveforms: HashMap<u64, Vec<(f32, f32)>> = HashMap::new();
        for audio in timeline_pane::audio_rows(&self.doc.view()) {
            if !audio.may_have_audio {
                continue;
            }
            let Some(path) = audio.source_path.as_deref() else {
                continue;
            };
            let (Some(peaks), Some(&total)) =
                (self.waveforms.get(path), self.source_frames.get(path))
            else {
                continue;
            };
            let Some(range) = waveform_bucket_range(&audio.timing, total, peaks.len()) else {
                continue;
            };
            waveforms.insert(audio.layer.0, peaks[range].to_vec());
        }

        let lanes = rows
            .into_iter()
            .map(|row| TimelineLane {
                waveform: waveforms.remove(&row.id.0).unwrap_or_default(),
                id: row.id.0,
                name: Self::display_name(&row.name),
                hidden: row.hidden,
                solo: row.solo,
                locked: row.locked,
                label_color: row
                    .label_color
                    .map(usize::from)
                    .unwrap_or_else(|| row.id.0.saturating_sub(1) as usize),
                start: row.start,
                duration: row.duration,
                selected: row.selected,
            })
            .collect();
        let property_lanes = timeline_pane::property_rows(
            &self.doc.view(),
            &self.session,
            self.doc.view().composition().ok().flatten().map(|c| c.fps),
        )
        .into_iter()
        .map(|row| TimelinePropertyLane {
            layer_id: row.layer.0,
            name: format!("> {}", row.property.name()),
            keys: row.keys.into_iter().map(|key| key.frame).collect(),
        })
        .collect();
        let composition = self.doc.view().composition().ok().flatten();
        let (duration_frames, fps_num, fps_den) = composition
            .as_ref()
            .map(|composition| {
                (
                    composition.duration_frames,
                    composition.fps.num(),
                    composition.fps.den(),
                )
            })
            .unwrap_or((1, 30, 1));

        // ロケータ(発注 S5)。**宣言順のまま**投影する — `TimelineMarker` に安定 id は
        // 無いので、index が身分そのもの(`TimelineMarker` の doc 参照)。frame へ
        // 変換できない(fps が壊れている)マーカーは黙って落とす代わりに描かない —
        // それでも index がずれると `RemoveMarker` が別の marker を消してしまうので、
        // 変換に失敗した物は捨てず frame=0 として繋ぎ止める(消せなくなるより安全)。
        let markers = composition
            .as_ref()
            .map(|composition| {
                self.doc
                    .view()
                    .markers()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|marker| TimelineMarker {
                        frame: marker.time.try_to_frame_round(composition.fps).unwrap_or(0),
                    })
                    .collect()
            })
            .unwrap_or_default();

        TimelineModel {
            lanes,
            property_lanes,
            markers,
            duration_frames,
            playhead: self.session.playhead,
            fps_num,
            fps_den,
        }
    }

    /// 落ちてきた1本を**サウンドトラックとして**この comp へ入れる
    /// (2026-08-18 裁定: 音声ファイルのドロップは offset 0 / gain 1.0)。
    ///
    /// **なぜ `browser_surface::place_media` を呼ばないか**: あちらは
    /// `motolii_media::probe`(先頭 video stream を要求する)で尺を取るので、
    /// audio-only ファイルでは必ず失敗する。音の尺は engine の口
    /// (`Engine::media_duration` = `probe_container` 経由、裁定274 (3) の修正)
    /// から取る。front は `motolii-audio` を直接引かない(裁定276)。
    ///
    /// **意図論(裁定271)**: 音を落とす人が求めているのは「この曲に合わせて作る」で
    /// あって、置き場所を名指してはいない。だから頭から・全長で・一番手前に置く。
    /// **名指していない物は変えない** — playhead も、comp の尺も、既存レイヤーの
    /// 順序も触らない。
    ///
    /// gain 1.0 は `property::LEVEL` の track を**置かないこと**で表す(裁定20:
    /// track が無ければ静止値)。値を書くと「1.0 というキーを打った」という別の
    /// 意味になる。
    ///
    /// 記帳(`AdmitAsset`)と配置(`AddLayer`+`SetMeta`)で **1 undo**。
    fn admit_soundtrack(&mut self, path: &Path) -> Result<String, String> {
        let label = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        let failed = |reason: &str| format!("AUDIO FAILED  ·  {label}  ·  {reason}");

        let store = self.doc.view();
        let composition = store
            .composition()
            .map_err(|error| failed(&error.to_string()))?
            .ok_or_else(|| failed("no composition yet"))?;
        let layer = LayerId(store.next_layer_id());
        let order = store
            .layers()
            .into_iter()
            .filter_map(|id| store.meta(id).ok().flatten().map(|meta| meta.order))
            .max()
            .map(|max| max.saturating_add(1))
            .unwrap_or(0);
        drop(store);

        let path_string = path.to_string_lossy().into_owned();
        let duration = self
            .engine
            .media_duration(&path_string)
            .ok_or_else(|| failed("cannot read its duration"))?;
        let source_frames = duration
            .try_to_frame_round(composition.fps)
            .map_err(|error| failed(&error.to_string()))?;
        // 頭から・素材の全長(comp の残りで頭打ち、`LayerTiming::place` の規則)。
        let timing = LayerTiming::place(0, Some(source_frames), composition.duration_frames);

        let mut intents = Vec::new();
        let content_hash = match std::fs::File::open(path)
            .map_err(|error| error.to_string())
            .and_then(|file| SourceFingerprintV1::from_reader(file).map_err(|e| e.to_string()))
        {
            Ok(fingerprint) => {
                let extension = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                let mut draft = AssetDraft::from_probed_source(
                    format!("audio/{extension}"),
                    &fingerprint,
                    path,
                    None,
                );
                draft.duration = Some(duration);
                intents.push(Intent::AdmitAsset { draft });
                Some(fingerprint.content_hash())
            }
            // 指紋が読めなくても配置は続ける(bin-first: 記帳と配置は別の判断、
            // 裁定162)。`browser_surface::place_media` と同じ割り切り。
            Err(_) => None,
        };
        intents.push(Intent::AddLayer(layer));
        intents.push(Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Media {
                    path: path_string.clone(),
                    fingerprint: content_hash,
                },
                order,
                timing,
            },
        });
        // 名前を付ける。既定は空文字なので、付けないと Timeline に**名前の無い行**
        // が生える(どれが自分の曲か読めない = Q0)。AE も取り込んだ物はファイル名で
        // 並ぶ。同じ `apply_all` の中なので undo は1回のまま。
        intents.push(Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch {
                name: Some(
                    path.file_stem()
                        .map(|stem| stem.to_string_lossy().into_owned())
                        .unwrap_or_else(|| label.clone()),
                ),
                ..Default::default()
            },
        });
        self.doc
            .apply_all(intents)
            .map_err(|error| failed(&error.to_string()))?;

        // 波形はここで**1回だけ**取る。失敗(音声 stream が無い)も空として憶える
        // — 憶えないと投影のたびに ffmpeg を叩き直す。
        let peaks = motolii_media::waveform_peaks(path, WAVEFORM_BUCKETS).unwrap_or_default();
        let silent = peaks.is_empty();
        self.waveforms.insert(path_string.clone(), peaks);
        self.source_frames.insert(path_string, source_frames);

        self.session.selection = Some(layer);
        self.session.selected_layers = vec![layer];

        let seconds = duration.as_seconds_f64();
        // 素材が comp より長ければ**そう言う**。黙って切ると、切れた側は
        // 「読み込めていない」ようにしか見えない(comp の尺を勝手に伸ばすのは
        // 利用者が名指していない変更なので、しない)。
        let clipped = if source_frames > timing.duration {
            format!("  ·  clipped to the comp ({} of {source_frames} frames)", timing.duration)
        } else {
            String::new()
        };
        let wave = if silent { "  ·  no audio stream" } else { "" };
        Ok(format!(
            "SOUNDTRACK  ·  {label}  ·  {seconds:.2}s from frame 0{clipped}{wave}"
        ))
    }

    fn scrub_to(&mut self, frame: i64) {
        let started = Instant::now();
        self.session.playhead = frame.max(0);
        // 再生中の seek(TARGET 4)。論理位置(`PlaybackClock`)は常に即時反映、
        // 実デバイスへの追従は `MixProducer` 側の既知の制約(`session.rs` doc の
        // `seek` 節参照)。
        if let Some(session) = self.audio_session.as_mut() {
            if let Some(composition) = self.doc.view().composition().ok().flatten() {
                if let Ok(at) = RationalTime::try_from_frame(self.session.playhead, composition.fps)
                {
                    session.seek(at);
                }
            }
        }
        self.frame = None;
        log!(
            "PERF store_scrub frame={} elapsed_us={}",
            frame,
            started.elapsed().as_micros()
        );
    }

    fn restack_from_timeline(&mut self, layer_id: u64, target_from_front: usize) -> String {
        let store = self.doc.view();
        let Some(layer) = timeline_pane::rows(&store, &self.session)
            .into_iter()
            .find(|row| row.id.0 == layer_id)
            .map(|row| row.id)
        else {
            return format!("Timeline: layer {layer_id} no longer exists");
        };
        let layer_count = store.layers().len();
        drop(store);
        let target_from_front = target_from_front.min(layer_count.saturating_sub(1));
        let target_from_back = layer_count
            .saturating_sub(1)
            .saturating_sub(target_from_front);

        self.session.selection = Some(layer);
        self.session.selected_layers = vec![layer];

        let store = self.doc.view();
        let stack: Vec<(LayerId, i16)> = store
            .layers()
            .into_iter()
            .filter_map(|id| store.meta(id).ok().flatten().map(|meta| (id, meta.order)))
            .collect();
        drop(store);
        let changes = restacked(
            &stack,
            &[layer],
            StackDirection::ToIndexFromBack(target_from_back),
        );
        if !changes.is_empty() {
            let intents: Vec<Intent> = changes
                .into_iter()
                .map(|(layer, order)| Intent::SetOrder { layer, order })
                .collect();
            if let Err(error) = self.doc.apply_all(intents) {
                return format!("重なりを書けない: {error}");
            }
            self.frame = None;
        }
        self.status = Some(format!(
            "Timeline: layer {} moved to lane {} / Stage stack updated",
            layer_id,
            target_from_front + 1
        ));
        self.status.clone().expect("just set")
    }

    /// レールグリフ(M/S/L)のクリック。3属性とも `motolii_store::LayerAttrsPatch`
    /// (hidden/solo/locked)に本物の書き口 `Intent::SetAttrs` が既にあるので、
    /// M/S/L のどれも Document へ書く — `TimelineModel` だけをその場でいじる
    /// フェイクは要らない(発注の「本物の口が無ければ捏造しない」の逆側:
    /// 本物があるのに使わない方が不自然)。
    fn toggle_lane_flag_from_timeline(&mut self, layer_id: u64, flag: LaneFlag) -> String {
        let layer = LayerId(layer_id);
        let store = self.doc.view();
        let Some(attrs) = store.attrs(layer).ok().flatten() else {
            return format!("Timeline: layer {layer_id} no longer exists");
        };
        drop(store);

        let (label, next, mut patch) = match flag {
            LaneFlag::Hidden => ("HIDDEN", !attrs.hidden, LayerAttrsPatch::default()),
            LaneFlag::Solo => ("SOLO", !attrs.solo, LayerAttrsPatch::default()),
            LaneFlag::Locked => ("LOCKED", !attrs.locked, LayerAttrsPatch::default()),
        };
        match flag {
            LaneFlag::Hidden => patch.hidden = Some(next),
            LaneFlag::Solo => patch.solo = Some(next),
            LaneFlag::Locked => patch.locked = Some(next),
        }
        if let Err(error) = self.doc.apply_all([Intent::SetAttrs { layer, patch }]) {
            return format!("{label} を書けない: {error}");
        }
        self.frame = None;
        self.status = Some(format!(
            "Timeline: layer {layer_id} {label} {}",
            if next { "ON" } else { "OFF" }
        ));
        self.status.clone().expect("just set")
    }

    /// # front から store へ書く時はこの形(2026-08-28、WIRE-1)
    ///
    /// トリム/移動の確定。`restack_from_timeline` / `toggle_lane_flag_from_timeline` と
    /// **同じ5手**で、動詞が `Intent::SetTiming` に変わるだけである。残りの動詞
    /// (`SetTrack` / `SetEffects` / `SetSource`)もこの形へ写す:
    ///
    /// 1. **今の値を store から読む**(`self.doc.view()` → `meta`/`attrs`/`track`)。
    ///    widget が持っている値は「見せていた形」であって正本ではない。読めなければ
    ///    layer はもう居ないので、書かずに理由を返す。
    /// 2. **`drop(store)`**。`view()` は借用なので、持ったまま `apply_all` は書けない。
    /// 3. **読んだ値の、意図が名指したフィールドだけを差し替える**。丸ごと組み直すと
    ///    名指していない物(ここでは `speed`)が黙って既定へ戻る。`Intent::SetMeta` を
    ///    使わず `SetTiming` を使うのはこのため(`SetMeta` の doc 参照)。
    /// 4. **`self.doc.apply_all([intent])` — 1ジェスチャ = 1呼び出し = 1 undo。**
    ///    ドラッグ中の途中経過は widget の中だけで動き、ここへは来ない。
    /// 5. **`self.frame = None`**(絵が変わったので Stage のキャッシュを捨てる)+
    ///    `status` を返す。呼び手は `TimelineUpdate` で投影の引き直しを決める。
    ///
    /// ## `source_in` を動かす場所
    ///
    /// 頭(`ClipEdge::Start`)を切った時だけ `source_in` が `start` と一緒に動く。
    /// Lottie の `st`(Start Time)は `st = start - source_in` としてこの2つに
    /// 分解されている(`app/reference/lottie-coverage.tsv`: `layers/layer/st` →
    /// 「代数的に等価な LayerTiming.source_in」)。頭を切っても素材はずれない
    /// ということは **`st` が動かない**ということで、`start` が動いた分 `source_in`
    /// も動く。丸ごと移動(`edge: None`)は逆に `st` ごと動くので `source_in` は
    /// そのまま、尻(`ClipEdge::End`)は `start` が動かないのでどちらも動かない。
    ///
    /// ## まだ無い壁(EVIDENCE_GAP)
    ///
    /// `LayerSource::Media` には上限がある(`source_in + duration × speed ≦ 素材の
    /// 総フレーム数`、裁定272)。**その総フレーム数は Document に無い** — 「大きさは
    /// probe が決めるので Document は持たない」(`LayerSource::Media` の doc)。
    /// `motolii-engine` は `probes: HashMap<String, MediaInfo>` を持っているが private で、
    /// front から聞く口が無い。だから Media の壁だけがここに無い。`Solid`/`Null`/
    /// `Shape`/`Text` は上限が無い(裁定272)ので、この経路は今そのまま本物である。
    fn set_clip_timing_from_timeline(
        &mut self,
        layer_id: u64,
        start: i64,
        duration: i64,
        edge: Option<ClipEdge>,
    ) -> String {
        let layer = LayerId(layer_id);
        let store = self.doc.view();
        let Some(meta) = store.meta(layer).ok().flatten() else {
            return format!("Timeline: layer {layer_id} no longer exists");
        };
        drop(store);

        let current = meta.timing;
        let start = start.max(0);
        let duration = duration.max(1);
        let source_in = match edge {
            // 頭を切る = 素材の頭出しが一緒に動く(`st` を保つ)。
            Some(ClipEdge::Start) => current.source_in + (start - current.start),
            // 尻を切る / 丸ごと動かす = 頭出しはそのまま。
            Some(ClipEdge::End) | None => current.source_in,
        };
        let timing = LayerTiming {
            start,
            duration,
            source_in,
            // 速度はトリムの意図が名指していない。タイムストレッチは別の動詞。
            speed: current.speed,
        };
        if timing == current {
            return format!("Timeline: layer {layer_id} timing unchanged");
        }
        if let Err(error) = self.doc.apply_all([Intent::SetTiming { layer, timing }]) {
            return format!("尺を書けない: {error}");
        }
        self.frame = None;
        let verb = match edge {
            Some(ClipEdge::Start) => "TRIM IN",
            Some(ClipEdge::End) => "TRIM OUT",
            None => "MOVE",
        };
        self.status = Some(format!(
            "Timeline: layer {layer_id} {verb}  ·  {start}..{}",
            start + duration
        ));
        self.status.clone().expect("just set")
    }

    /// 選択(A5)。行き先が `Document` ではなく `Session` なだけで、上の5手と同じ形
    /// (読む → 名指した物だけ差し替える → 投影を引き直す)。**Undo に乗らない** —
    /// 選択は Document に乗らない身分だから(`Session.selected_layers` の doc)。
    ///
    /// `layer_id: None` は**必ず全解除**。「修飾キー付きの空所クリックは何もしない」
    /// という判断はここではなく `apply_timeline_edit` にある — あちらが意図を読む
    /// 側で、こちらは名指された物を書く側。
    fn select_from_timeline(&mut self, layer_id: Option<u64>, additive: bool) -> String {
        let Some(layer_id) = layer_id else {
            self.session.selection = None;
            self.session.selected_layers.clear();
            self.status = Some("Timeline: selection cleared".to_owned());
            return self.status.clone().expect("just set");
        };
        let layer = LayerId(layer_id);
        let store = self.doc.view();
        let exists = store.meta(layer).ok().flatten().is_some();
        drop(store);
        if !exists {
            return format!("Timeline: layer {layer_id} no longer exists");
        }

        if additive {
            if let Some(index) = self
                .session
                .selected_layers
                .iter()
                .position(|selected| *selected == layer)
            {
                self.session.selected_layers.remove(index);
                if self.session.selection == Some(layer) {
                    self.session.selection = self.session.selected_layers.last().copied();
                }
            } else {
                self.session.selected_layers.push(layer);
                self.session.selection = Some(layer);
            }
        } else {
            self.session.selection = Some(layer);
            self.session.selected_layers = vec![layer];
        }
        self.status = Some(format!(
            "Timeline: {} layer(s) selected",
            self.session.selected_layers.len()
        ));
        self.status.clone().expect("just set")
    }

    /// ルーラーの右クリック(空所)。**その時刻へ、既定値のロケータを1つ**置く
    /// (発注 S5)。名前("")と尺(単発 = 0)は発明しない — Lottie の `cm`/`dr` の
    /// 既定と同じ。`SetMarkers` は丸ごと差し替え型(`document.rs` の doc)なので、
    /// 今の一覧を読んでから1件足して書き戻す(`SetMasks`/`AddMask` と同じ read-modify-write)。
    fn add_marker_from_timeline(&mut self, frame: i64) -> String {
        let store = self.doc.view();
        let Some(composition) = store.composition().ok().flatten() else {
            return "ロケータを書けない: composition is unreadable".to_owned();
        };
        let mut markers = store.markers().unwrap_or_default();
        drop(store);
        let Ok(time) = RationalTime::try_from_frame(frame.max(0), composition.fps) else {
            return format!("ロケータを書けない: frame {frame} does not map to a time");
        };
        markers.push(Marker {
            name: String::new(),
            time,
            duration: RationalTime::ZERO,
        });
        if let Err(error) = self.doc.apply_all([Intent::SetMarkers { markers }]) {
            return format!("ロケータを書けない: {error}");
        }
        self.status = Some(format!("Timeline: marker placed at frame {frame}"));
        self.status.clone().expect("just set")
    }

    /// 既存ロケータの上の右クリック。**そのロケータを消す**(発注 S5、置けるのに
    /// 消せないのは Q0 違反)。`index` は widget が持つ `TimelineModel.markers` の
    /// 宣言順そのもの — `TimelineMarker` に安定 id が無いので、これが唯一の名指し方
    /// (`marker.rs` の doc)。
    fn remove_marker_from_timeline(&mut self, index: usize) -> String {
        let store = self.doc.view();
        let mut markers = store.markers().unwrap_or_default();
        drop(store);
        if index >= markers.len() {
            return "Timeline: marker no longer exists".to_owned();
        }
        markers.remove(index);
        if let Err(error) = self.doc.apply_all([Intent::SetMarkers { markers }]) {
            return format!("ロケータを書けない: {error}");
        }
        self.status = Some("Timeline: marker removed".to_owned());
        self.status.clone().expect("just set")
    }

    /// Delete/Backspace(発注 S4)。選択された全レイヤーを**1回の `apply_all`**で
    /// 消す(= 1 undo)。削除は tombstone なので undo で戻せる(`Intent::RemoveLayer`
    /// の doc)。locked/frozen な層が混ざっていれば `apply_all` は原子的に失敗する
    /// (`Document::apply_all` の doc「バッチ全体を無かったことにする」)ので、
    /// 一部だけ消えて残りが残るという半端な結果にはならない。
    fn remove_selected_layers(&mut self) -> String {
        let layers = self.session.selected_layers.clone();
        if layers.is_empty() {
            return "Timeline: nothing selected to delete".to_owned();
        }
        let intents: Vec<Intent> = layers.iter().copied().map(Intent::RemoveLayer).collect();
        if let Err(error) = self.doc.apply_all(intents) {
            // 無反応ゼロ — locked/frozen を消そうとしたら理由が見える。
            return format!("削除を書けない: {error}");
        }
        self.frame = None;
        let count = layers.len();
        // 消えた層は選択からも外す(削除された物を選び続けさせない)。
        self.session.selection = None;
        self.session.selected_layers.clear();
        self.status = Some(format!("Timeline: {count} layer(s) deleted"));
        self.status.clone().expect("just set")
    }

    /// selection summary の投影(発注 S5b)。Inspector ヘッダの「名前+種別」だけ運ぶ —
    /// `inspector_surface::InspectorSurface::set_selection_summary` の**既存の口**で
    /// 運べる範囲だけ(TRANSFORM の値投影は口が無いので次の波、非目標)。
    fn selection_summary(&self) -> (String, String) {
        let Some(layer) = self.session.selection else {
            return (
                "No Selection".to_owned(),
                "Select a layer to inspect".to_owned(),
            );
        };
        let store = self.doc.view();
        let Some(meta) = store.meta(layer).ok().flatten() else {
            return (
                "No Selection".to_owned(),
                "Select a layer to inspect".to_owned(),
            );
        };
        let attrs = store.attrs(layer).ok().flatten().unwrap_or_default();
        let name = if attrs.name.is_empty() {
            format!("Layer {}", layer.0)
        } else {
            attrs.name
        };
        (name, Self::layer_kind_label(&meta.source).to_owned())
    }

    /// comp の寸法。選択の有無によらず — pick(TARGET5)が comp 空間の点を作るのに要る。
    fn gizmo_comp_dims(&self) -> Option<(f32, f32)> {
        let store = self.doc.view();
        let composition = store.composition().ok().flatten()?;
        Some((composition.width as f32, composition.height as f32))
    }

    /// 選択レイヤーの world transform(ギズモの対象、S20 TARGET2)。**新しい計算を
    /// しない** — `StoreView::resolve` は合成器と同じ経路
    /// (`ResolvedLayer.placement.transform`)なので、その値をそのまま渡す。hidden/
    /// timing 外/comp 無し/decompose 失敗のいずれかで `None`(ギズモは出ない —
    /// AE で見えない層を掴めないのと同じ)。skew が乗っていると
    /// `to_scale_angle_translation` の分解は近似になる(EVIDENCE_GAP)。
    fn gizmo_target(&self) -> Option<GizmoTarget> {
        let layer = self.session.selection?;
        let store = self.doc.view();
        let composition = store.composition().ok().flatten()?;
        let t = RationalTime::try_from_frame(self.session.playhead, composition.fps).ok()?;
        let resolved = store.resolve(layer, t).ok().flatten()?;
        let (scale, angle_radians, translation) =
            resolved.placement.transform.to_scale_angle_translation();
        Some(GizmoTarget {
            layer,
            translation: [translation.x, translation.y],
            rotation_degrees: angle_radians.to_degrees(),
            scale: [scale.x, scale.y],
        })
    }

    /// ドラッグを離した結果を Document へ書く(S20 TARGET4)。触った成分だけ
    /// (`GizmoCommit` の `Some`)を1つの property track として書く — 1回のドラッグは
    /// 1種類のハンドルしか動かさないので、`apply_all` に積む intent は常に1個 = 1 undo。
    fn apply_gizmo_commit(&mut self, commit: &GizmoCommit) -> String {
        let store = self.doc.view();
        if !store.has_layer(commit.layer) {
            return format!("Stage: layer {} no longer exists", commit.layer.0);
        }
        let Some(composition) = store.composition().ok().flatten() else {
            return "Stage: the composition is unreadable".to_owned();
        };
        let fps = composition.fps;
        let playhead = self.session.playhead;

        let mut intent = None;
        let mut label = "";
        if let Some(translation) = commit.translation {
            match write_gizmo_component(
                &store,
                commit.layer,
                fps,
                playhead,
                property::POSITION,
                Value::Vec2([translation[0] as f64, translation[1] as f64]),
            ) {
                Ok(built) => {
                    intent = Some(built);
                    label = "position";
                }
                Err(error) => return error,
            }
        } else if let Some(rotation) = commit.rotation_degrees {
            match write_gizmo_component(
                &store,
                commit.layer,
                fps,
                playhead,
                property::ROTATION,
                Value::F64(rotation as f64),
            ) {
                Ok(built) => {
                    intent = Some(built);
                    label = "rotation";
                }
                Err(error) => return error,
            }
        } else if let Some(scale) = commit.scale {
            match write_gizmo_component(
                &store,
                commit.layer,
                fps,
                playhead,
                property::SCALE,
                Value::Vec2([scale[0] as f64, scale[1] as f64]),
            ) {
                Ok(built) => {
                    intent = Some(built);
                    label = "scale";
                }
                Err(error) => return error,
            }
        }
        drop(store);
        let Some(intent) = intent else {
            return String::new();
        };
        if let Err(error) = self.doc.apply_all([intent]) {
            return format!("Stage: gizmo write failed: {error}");
        }
        format!("Stage: layer {} {label}", commit.layer.0)
    }

    /// Inspector の `SetInterp` を Document へ書く(wf4 INTERVAL EASING 板)。
    /// **playhead を含む区間だけ**を差し替える — キーの時刻・値は動かさない
    /// (`KeyframeTrack::eval` の「keys[i].interp が keys[i]→keys[i+1] を決める」規約)。
    fn apply_inspector_interp(&mut self, prop: &str, interp: Interp) -> String {
        let Some(property_name) = inspector_row_property(prop) else {
            return format!("Inspector: unwired property {prop}");
        };
        let Some(layer) = self.session.selection else {
            return "Inspector: no layer selected".to_owned();
        };
        let store = self.doc.view();
        let Some(composition) = store.composition().ok().flatten() else {
            return "Inspector: the composition is unreadable".to_owned();
        };
        let Ok(property) = PropertyId::new(property_name) else {
            return "Inspector: bad property id".to_owned();
        };
        let Ok(t) = RationalTime::try_from_frame(self.session.playhead.max(0), composition.fps)
        else {
            return "Inspector: the playhead does not map to a time".to_owned();
        };
        let track = store.track(layer, &property).ok().flatten().unwrap_or_default();
        let Some(index) = segment_start_index(&track, t) else {
            return format!("Inspector: {prop} has no segment at the playhead");
        };
        let mut keys = track.keys().to_vec();
        keys[index].interp = interp;
        let Ok(track) = KeyframeTrack::try_from_keys(keys) else {
            return "Inspector: interp write failed validation".to_owned();
        };
        drop(store);
        if let Err(error) = self.doc.apply_all([Intent::SetTrack {
            layer,
            property,
            track,
        }]) {
            return format!("Inspector: interp write failed: {error}");
        }
        format!("Inspector: layer {} {prop} interp", layer.0)
    }

    /// Inspector の ◆/◇ を Document へ書く。**`write_gizmo_component` と同じ
    /// 「AE の指の規約」**(`fx_stack.rs::apply` の doc): キーが無い property は時刻0に
    /// Hold で静止値、キーがある property はプレイヘッドへ直前の interp を写して打つ。
    /// 値は `StoreView::value_at` が読む「今の評価値」— 打つ瞬間に絵が動かない。
    fn apply_inspector_toggle_key(&mut self, prop: &str, keyed: bool) -> String {
        let Some(property_name) = inspector_row_property(prop) else {
            return format!("Inspector: unwired property {prop}");
        };
        let Some(layer) = self.session.selection else {
            return "Inspector: no layer selected".to_owned();
        };
        let store = self.doc.view();
        let Some(composition) = store.composition().ok().flatten() else {
            return "Inspector: the composition is unreadable".to_owned();
        };
        let Ok(property) = PropertyId::new(property_name) else {
            return "Inspector: bad property id".to_owned();
        };
        let Ok(t) = RationalTime::try_from_frame(self.session.playhead.max(0), composition.fps)
        else {
            return "Inspector: the playhead does not map to a time".to_owned();
        };
        let mut track = store.track(layer, &property).ok().flatten().unwrap_or_default();
        let track = if keyed {
            // track が無い property は `value_at` が `None` を返す(裁定20 の
            // 「静止値」は 0 キーの track の話で、track 自体が無い時の既定は
            // 呼び手が持つ — `resolve.rs` の `scalar(name, default)` と同じ形)。
            let value = store.value_at(layer, &property, t).ok().flatten().unwrap_or(
                match property_name {
                    p if p == property::SCALE => Value::Vec2([1.0, 1.0]),
                    p if p == property::POSITION => Value::Vec2([0.0, 0.0]),
                    p if p == property::OPACITY => Value::F64(1.0),
                    _ => Value::F64(0.0),
                },
            );
            let interp = track
                .keys()
                .iter()
                .rev()
                .find(|key| key.t <= t)
                .or_else(|| track.keys().first())
                .map(|key| key.interp)
                .unwrap_or(Interp::Hold);
            track.insert(Keyframe {
                t,
                value,
                interp,
                spatial: None,
            });
            track
        } else {
            let keys: Vec<_> = track.keys().iter().filter(|key| key.t != t).cloned().collect();
            match KeyframeTrack::try_from_keys(keys) {
                Ok(next) => next,
                Err(_) => return "Inspector: key removal failed validation".to_owned(),
            }
        };
        drop(store);
        if let Err(error) = self.doc.apply_all([Intent::SetTrack {
            layer,
            property,
            track,
        }]) {
            return format!("Inspector: key write failed: {error}");
        }
        format!("Inspector: layer {} {prop} key", layer.0)
    }

    /// [`InspectorSurface::set_property_keys`] へ押し込む3行ぶんの今
    /// (keyed / playhead を含む区間の interp)。`selection_summary` と同じ形
    /// (Document を読むだけ、面には触らない)。
    fn inspector_key_states(&self) -> Vec<(&'static str, bool, Option<Interp>)> {
        let mut states = Vec::new();
        let Some(layer) = self.session.selection else {
            return states;
        };
        let store = self.doc.view();
        let Some(composition) = store.composition().ok().flatten() else {
            return states;
        };
        let Ok(t) = RationalTime::try_from_frame(self.session.playhead.max(0), composition.fps)
        else {
            return states;
        };
        for prop in ["position", "rotation", "scale", "opacity"] {
            let property_name = inspector_row_property(prop).expect("listed above");
            let Ok(property) = PropertyId::new(property_name) else {
                continue;
            };
            let Some(track) = store.track(layer, &property).ok().flatten() else {
                states.push((prop, false, None));
                continue;
            };
            let keyed = track.keys().iter().any(|key| key.t == t);
            let interp = segment_start_index(&track, t).map(|i| track.keys()[i].interp);
            states.push((prop, keyed, interp));
        }
        states
    }

    /// `LayerSource` の人が読む種別名。捏造ではなく `LayerSource` の6 variant を
    /// そのまま名付けているだけ(新しい語彙を発明しない)。
    fn layer_kind_label(source: &LayerSource) -> &'static str {
        match source {
            LayerSource::Solid { .. } => "Solid layer",
            LayerSource::Media { .. } => "Media layer",
            LayerSource::Null => "Null layer",
            LayerSource::Shape => "Shape layer",
            LayerSource::Text => "Text layer",
            LayerSource::Group => "Group",
        }
    }

    /// Browser の棚(`Composition:assets`、裁定162 の bin-first 台帳)の投影。
    ///
    /// **front はカタログを持たない。** 以前 `browser_surface.rs` が持っていた
    /// 「手書き8件」は、取り込んだ物が Browser に現れない原因そのものだった
    /// (台帳「素材の配置は3本足りない」)。ここが唯一の出所である。
    ///
    /// `●`(= 配置済み)は**layer 側から導く** — 台帳に「置いたかどうか」を書き足すと、
    /// 同じ事実の家が2つになる。指紋(`content_hash`)が一致すれば同じ素材、
    /// 無ければ実体パスで見る(`LayerSource::Media.fingerprint` は任意)。
    fn browser_catalog(&self) -> Vec<BrowserAsset> {
        let store = self.doc.view();
        let placed: Vec<(Option<String>, String)> = store
            .layers()
            .into_iter()
            .filter_map(|id| store.meta(id).ok().flatten())
            .filter_map(|meta| match meta.source {
                LayerSource::Media { path, fingerprint } => Some((fingerprint, path)),
                _ => None,
            })
            .collect();
        store
            .assets()
            .unwrap_or_default()
            .into_iter()
            .map(|asset| BrowserAsset {
                id: asset.id.get(),
                // 一覧に出すのは拡張子まで含んだファイル名。`Asset::name` は stem なので
                // 「同じ名前の mp4 と wav」が同じ字面になる。
                name: asset
                    .file_name
                    .clone()
                    .unwrap_or_else(|| asset.name.clone()),
                kind: AssetKind::from_asset_type(&asset.asset_type),
                placed: placed.iter().any(|(fingerprint, path)| {
                    fingerprint.as_deref() == Some(asset.content_hash.as_str())
                        || Some(path.as_str()) == asset.path_absolute.as_deref()
                }),
                // store にタグの語彙がまだ無い。**発明しない**(`BrowserAsset::tags` の doc)。
                tags: Vec::new(),
            })
            .collect()
    }

    /// 棚 → タイムライン。カードの double-click([`BrowserEditAction::PlaceAsset`])の受け口。
    ///
    /// `set_clip_timing_from_timeline` の5手と同じ形で、動詞が新規配置なので
    /// `Intent::AddLayer` + `Intent::SetMeta` + `Intent::SetAttrs` の3本になる。
    /// **`SetSource` ではない** — `SetSource` は既存 `meta` を読んで書き換える口で、
    /// `meta` を持たない生まれたての layer には使えない(`Intent::SetMeta` の doc)。
    /// 3本は1回の `apply_all` = **1ジェスチャ = 1 undo**。
    ///
    /// ## 意図が名指していない物は既定で埋める(裁定271)
    ///
    /// double-click が言っているのは「これを使いたい」だけなので、置き場所は道具が決める:
    /// - **いつ**: playhead。聞きながら置く道具なので、手が居る場所が既定
    /// - **どこ**: 最前面(`order = max + 1`)。Timeline の一番上 = Stage の一番手前
    /// - **どれだけ**: [`LayerTiming::place`] = min(素材の尺, comp の残り)。
    ///   **この規則は store が持っている**(「shell に書かせない」— M4)ので写さない。
    ///   尺を持たない素材(静止画)は comp の残り全部 = AE の新規レイヤーと同じ
    ///
    /// ## 尺は engine に聞く
    ///
    /// front は `motolii-media` を直接引かない。`Engine::media_duration` は
    /// `probe_container` 経由なので **audio-only も答える**(裁定274 (3))。
    /// `RationalTime` のまま受けて comp の fps で丸めるので、素材 fps と comp fps が
    /// 食い違っても尺がずれない(`media_frames` は素材ネイティブ fps 単位なので使わない)。
    fn place_asset_from_browser(&mut self, asset_id: u64) -> String {
        let store = self.doc.view();
        let Some(asset) = store.asset(AssetId::from_raw(asset_id)).ok().flatten() else {
            return format!("Browser: asset {asset_id} は棚に無い");
        };
        let Some(composition) = store.composition().ok().flatten() else {
            return "Browser: comp が無い".to_owned();
        };
        let layer = LayerId(store.next_layer_id());
        let front = store
            .layers()
            .into_iter()
            .filter_map(|id| store.meta(id).ok().flatten().map(|meta| meta.order))
            .max();
        drop(store);

        // 実体を指さない素材(生成系)は、まだ layer にできない。黙って空の layer を
        // 作らず、理由を言って何も書かない。
        let Some(path) = asset.path_absolute.clone() else {
            return format!("Browser: {} は実体のパスを持たない", asset.name);
        };
        let source_frames = self
            .engine
            .media_duration(&path)
            .and_then(|duration| duration.try_to_frame_round(composition.fps).ok())
            .filter(|frames| *frames > 0);
        let timing = LayerTiming::place(
            self.session.playhead,
            source_frames,
            composition.duration_frames,
        );
        // comp の終端に playhead が居ると尺が 0 になる。尺ゼロの layer は
        // 置けたのに見えない = 「効いたように見えて黙って戻る」なので、書かない。
        if timing.duration <= 0 {
            return format!(
                "Browser: playhead が comp の終端({})に居るので置けない",
                composition.duration_frames
            );
        }
        let order = front.map_or(0, |order| order.saturating_add(1));

        let intents = vec![
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Media {
                        path,
                        fingerprint: Some(asset.content_hash.clone()),
                    },
                    order,
                    timing,
                },
            },
            Intent::SetAttrs {
                layer,
                patch: LayerAttrsPatch {
                    name: Some(asset.name.clone()),
                    ..Default::default()
                },
            },
        ];
        if let Err(error) = self.doc.apply_all(intents) {
            return format!("素材を置けない: {error}");
        }
        self.frame = None;
        // 置いた物を選ぶ。**意図の直接の産物**なので裁定271 に反しない
        // (AE / Premiere も新規レイヤーを選択状態にする)。Inspector が
        // 「今どれの話をしているか」を持てるのはここだけ。
        self.session.selection = Some(layer);
        self.session.selected_layers = vec![layer];
        self.status = Some(format!(
            "Browser: {} placed  ·  {}..{}",
            asset.name,
            timing.start,
            timing.start + timing.duration
        ));
        self.status.clone().expect("just set")
    }

    /// Settings 面の確定値([`SettingsSurfaceAction::SetField`])を `Intent::SetComposition`
    /// へ書く(S8)。**丸ごと差し替え型**なので、今の `Composition` を読んで名指された
    /// 1欄だけ書き換える(裁定271: 名指していない欄は既定へ戻さない)。
    ///
    /// `field` は `ScrubValue::prop` からそのまま来た名前(`"fps"`/`"width"`/
    /// `"height"`/`"duration"`)。書き先の対応表はここ1箇所にしか無い。
    fn apply_settings_action(&mut self, field: &str, value: f64) -> String {
        let store = self.doc.view();
        let Some(mut composition) = store.composition().ok().flatten() else {
            return "SETTINGS  ·  no composition yet".to_owned();
        };
        drop(store);
        match field {
            "fps" => match Fps::try_new(value.round() as i64, 1) {
                Ok(fps) => composition.fps = fps,
                Err(error) => return format!("SETTINGS FAILED  ·  Frame Rate  ·  {error}"),
            },
            "width" => composition.width = value.round() as u32,
            "height" => composition.height = value.round() as u32,
            "duration" => composition.duration_frames = value.round() as i64,
            other => return format!("SETTINGS  ·  unknown field {other}"),
        }
        if let Err(error) = self.doc.apply(Intent::SetComposition(composition)) {
            return format!("SETTINGS FAILED  ·  {field}  ·  {error}");
        }
        self.frame = None;
        format!("SETTINGS  ·  {field} = {value:.0}")
    }

    /// screenshot / export など明示 fallback 専用。通常の playhead / 再生からは呼ばない。
    fn frame_rgba(&mut self) -> Option<(u32, u32, &[u8])> {
        if self.frame.is_none() {
            let composition = self.doc.view().composition().ok().flatten()?;
            let t = RationalTime::try_from_frame(self.session.playhead, composition.fps).ok()?;
            let rgba = self.engine.render_frame(&self.doc.view(), t).ok()?;
            self.frame = Some((composition.width, composition.height, rgba));
        }
        self.frame
            .as_ref()
            .map(|(width, height, rgba)| (*width, *height, rgba.as_slice()))
    }

    /// 再生開始(TARGET 1)。`Document` の view から `AudioProgram::from_view` で
    /// program を組み、現在の playhead 時刻で `PlaybackSession::open_default` を
    /// 開く。**panic しない**(TARGET 6) — 失敗理由は `String` で返すだけで、
    /// `audio_session` は `None` のまま(呼び出し側は timer 駆動へ黙って劣化する)。
    fn start_audio_session(&mut self) -> Result<(), String> {
        let Some(composition) = self
            .doc
            .view()
            .composition()
            .map_err(|error| error.to_string())?
        else {
            // comp が無い Document: 音声 program を組む対象が無い。旧
            // `playback_tick` の「何もしない」と同じ扱いで、エラーではない。
            return Ok(());
        };
        let at = RationalTime::try_from_frame(self.session.playhead, composition.fps)
            .map_err(|error| error.to_string())?;
        let program = {
            let view = self.doc.view();
            AudioProgram::from_view(&view, &mut self.audio_pcm_cache)
                .map_err(|error| error.to_string())?
        };
        let session = PlaybackSession::open_default(Arc::new(program), at)
            .map_err(|error| error.to_string())?;
        self.audio_session = Some(session);
        Ok(())
    }

    /// 再生停止(TARGET 3)。session を閉じる前に clock の最終位置を playhead へ
    /// 写す — `PlaybackClock` が停止直前まで再生位置の正なので(P07-C1D・憲法3)、
    /// これが無いと直近の 60fps timer tick と実デバイス供給の間の端数が失われる。
    /// session を drop すると stream/producer スレッドが閉じる(`session.rs` doc)。
    fn stop_audio_session(&mut self) {
        let Some(session) = self.audio_session.take() else {
            return;
        };
        let Some(composition) = self.doc.view().composition().ok().flatten() else {
            return;
        };
        if let Ok(position) = session.clock().position() {
            if let Ok(frame) = position.try_to_frame_round(composition.fps) {
                self.session.playhead = frame.max(0);
            }
        }
    }

    /// 返り値: (再生中か, 音声デバイスが開けなかった理由)。理由が `Some` でも
    /// 再生自体は続ける(TARGET 6: 静かな劣化) — front はこれを状態行へ出すだけ。
    fn toggle_playback(&mut self) -> (bool, Option<String>) {
        self.playing = !self.playing;
        if self.playing {
            let issue = self.start_audio_session().err();
            (true, issue)
        } else {
            self.stop_audio_session();
            (false, None)
        }
    }

    fn playback_tick(&mut self) -> bool {
        if !self.playing {
            return false;
        }
        let started = Instant::now();
        let Some(composition) = self.doc.view().composition().ok().flatten() else {
            return false;
        };
        let duration = composition.duration_frames.max(1);

        // 再生中の playhead は音声クロックから導出する(P07-C1D・憲法3)。
        // `audio_session` が無い(デバイス不可・comp 無し等)場合だけ、従来どおり
        // timer 駆動で1フレームずつ進める(TARGET 6 の静かな劣化)。
        let next_frame = match self.audio_session.as_ref() {
            Some(session) => match session.clock().position() {
                Ok(position) => position
                    .try_to_frame_round(composition.fps)
                    .unwrap_or(self.session.playhead)
                    .max(0),
                Err(_) => self.session.playhead,
            },
            None => self.session.playhead + 1,
        };

        if next_frame >= duration {
            // comp 末尾で停止(TARGET 5) — ループしない。
            self.playing = false;
            self.stop_audio_session();
            self.session.playhead = duration.saturating_sub(1).max(0);
            self.frame = None;
            return true;
        }

        self.session.playhead = next_frame;
        self.frame = None;
        log!(
            "PERF store_playback frame={} elapsed_us={}",
            self.session.playhead,
            started.elapsed().as_micros()
        );
        true
    }

    fn apply_timeline_action(&mut self, action: &TimelineSurfaceAction) -> TimelineUpdate {
        match *action {
            TimelineSurfaceAction::None => TimelineUpdate::None,
            TimelineSurfaceAction::Scrub(frame) => {
                self.scrub_to(frame);
                TimelineUpdate::Stage(format!("SCRUB  ·  FRAME {frame}"))
            }
            TimelineSurfaceAction::Restack {
                layer_id,
                target_from_front,
            } => TimelineUpdate::ModelAndStage(
                self.restack_from_timeline(layer_id, target_from_front),
            ),
            TimelineSurfaceAction::ZoomChanged {
                start_frame,
                visible_frames,
            } => TimelineUpdate::Status(format!(
                "TIME ZOOM  ·  X ONLY  ·  START {start_frame}  ·  SPAN {visible_frames}F"
            )),
            TimelineSurfaceAction::ToggleLaneFlag { layer_id, flag } => TimelineUpdate::ModelAndStage(
                self.toggle_lane_flag_from_timeline(layer_id, flag),
            ),
        }
    }

    /// Timeline の**編集意図**の受け口([`TimelineEditAction`])。`apply_timeline_action`
    /// (スクラブ・ズーム・並べ替え)と口を分けているのは widget 側の都合ではなく、
    /// こちらが `Document`/`Session` を書く動詞だけを集めているから。
    fn apply_timeline_edit(&mut self, action: &TimelineEditAction) -> TimelineUpdate {
        match *action {
            TimelineEditAction::None => TimelineUpdate::None,
            // 修飾キー付きの空所クリックは何も名指していない。既存の選択を奪わない
            // (裁定271: 操作は意図が名指した物だけを変える)。
            TimelineEditAction::Select {
                layer_id: None,
                additive: true,
            } => TimelineUpdate::None,
            TimelineEditAction::Select { layer_id, additive } => {
                TimelineUpdate::Model(self.select_from_timeline(layer_id, additive))
            }
            TimelineEditAction::SetClipTiming {
                layer_id,
                start,
                duration,
                edge,
            } => TimelineUpdate::ModelAndStage(
                self.set_clip_timing_from_timeline(layer_id, start, duration, edge),
            ),
            // マーカーは絵を変えない(comp の合成に加わらない metadata)ので、選択と
            // 同じく Stage は焼き直さない(裁定271)。
            TimelineEditAction::AddMarker { frame } => {
                TimelineUpdate::Model(self.add_marker_from_timeline(frame))
            }
            TimelineEditAction::RemoveMarker { index } => {
                TimelineUpdate::Model(self.remove_marker_from_timeline(index))
            }
        }
    }

    /// 選択レイヤーの効果スタックの投影(`timeline_model` と同じ身分 — 読むだけ)。
    fn fx_model(&self) -> FxStackModel {
        fx_stack::model_for(&self.doc.view(), &self.session)
    }

    /// FX の編集意図を Document へ写す。**書く手順は `fx_stack::apply` が持つ** —
    /// `restack_from_timeline` 等と同じで、ここは投影の引き直しを決める側に
    /// 何が変わったかを返すだけ。
    fn apply_fx_action(&mut self, action: &FxStackAction) -> FxWrite {
        let write = fx_stack::apply(&mut self.doc, &self.session, action);
        if write.wrote {
            // 絵が変わったので Stage のキャッシュを捨てる
            // (`toggle_lane_flag_from_timeline` の `self.frame = None` と同じ)。
            self.frame = None;
            self.status = Some(write.status.clone());
        }
        write
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    playback_timer: Timer,
    /// Browser の選択。widget の animator ではなくここが正本。
    #[rust]
    browser_tab: usize,
    #[rust]
    browser_rail: usize,
    /// UI 全体の拡縮(%)。100 が等倍。
    #[rust(100)]
    ui_scale_percent: i32,
    /// いま明かしている浮くタブ行。開いた後の保持判定に使う。
    #[rust]
    revealed_bar: Option<LiveId>,
    /// Browser 面が畳まれているか。面の幅の正本は Dock の splitter align で、
    /// ここは「どちら向きに押すか」だけを持つ。
    #[rust]
    browser_collapsed: bool,
    /// 畳む直前の Browser 面の幅。戻す先が無いと畳みは片道になる。
    #[rust]
    browser_restore_width: f64,
    /// 状態行の文言。live edit は widget を宣言状態へ戻すので、ここが正本。
    #[rust]
    status_text: String,
    #[rust]
    stage_next_frame: NextFrame,
    /// 直前の室判定。変化したときだけ1行ログを出すため。
    #[rust]
    stage_verdict: Option<StageVerdict>,
    #[rust]
    stage_request: LatestFrameRequest,
    /// The existing product shell remains the sole Document/Engine owner. This
    /// probe only reads its compositor output for the Makepad Stage image.
    #[rust]
    backend: Option<BackendBridge>,
}

impl App {
    fn install_stage_frame(&mut self, cx: &mut Cx) {
        let verdict = self.try_present_shared(cx);
        self.set_stage_error(cx, &verdict.message());
        // 室が変わったときだけ1行。黒い Stage を見たらこの行だけ読めばよい。
        if self.stage_verdict != Some(verdict) {
            self.stage_verdict = Some(verdict);
            match verdict {
                StageVerdict::Shown => log!("STAGE room=- zero_copy=true shown"),
                StageVerdict::Stalled { room, reason } => {
                    log!(
                        "STAGE room={} owner={} reason={}",
                        room.tag(),
                        room.owner(),
                        reason
                    )
                }
            }
        }
    }

    fn try_present_shared(&mut self, cx: &mut Cx) -> StageVerdict {
        // 観測視点は front ローカル状態(store には無い)。`backend` を可変で借りる
        // **前**に読む — 借用の順序であって、意味の順序ではない。
        let view_camera = self.stage_view_camera(cx);
        let Some(backend) = self.backend.as_mut() else {
            return StageVerdict::stalled(StageRoom::Seam, "backend is not up yet");
        };
        let Some(composition) = backend.doc.view().composition().ok().flatten() else {
            return StageVerdict::stalled(StageRoom::Host, "composition is unreadable");
        };
        let desc = SharedSurfaceDesc::from_comp(composition.width, composition.height);
        let recreate = backend.present.needs_recreate(desc) || backend.stage_gpu.is_none();
        if recreate {
            let (texture, handle) = cx.create_presentable_texture(
                desc.width,
                desc.height,
                SharedPresentablePixel::Rgba8Srgb,
            );
            let handle = match handle {
                makepad_widgets::SharedOsHandle::IoSurfaceId(id) => SharedOsHandle::IoSurfaceId(id),
                makepad_widgets::SharedOsHandle::DxgiSharedHandle(v) => {
                    SharedOsHandle::DxgiSharedHandle(v)
                }
                makepad_widgets::SharedOsHandle::DmaBufFd(fd) => SharedOsHandle::DmaBufFd(fd),
            };
            let Some(present) = StagePresent::shared(desc, handle) else {
                return StageVerdict::stalled(StageRoom::Leaf, "the shared surface handle is unusable");
            };
            let Some(gpu) =
                stage_import::import_presentable(backend.engine.gpu_device(), desc, handle)
            else {
                return StageVerdict::stalled(StageRoom::Seam, "cannot import the shared surface into wgpu");
            };
            backend.stage_texture = Some(texture);
            backend.stage_gpu = Some(gpu);
            backend.present = present;
        }
        let Some(gpu) = backend.stage_gpu.as_ref() else {
            return StageVerdict::stalled(StageRoom::Seam, "no shared surface is held");
        };
        let Ok(t) = RationalTime::try_from_frame(backend.session.playhead, composition.fps) else {
            return StageVerdict::stalled(StageRoom::Host, "playhead does not map to a time");
        };
        // **カメラ分離の継ぎ目はこの1箇所**(裁定157)。
        //   Camera タブ = 出力カメラ(`view.resolve_camera(t)`)= 書き出しと同じ絵。
        //   User View タブ = 観測カメラ。engine の別入口を通り、Document を1文字も
        //   触らないので、どれだけ見回しても `render_frame`(export の経路)が
        //   作る絵は変わらない。
        // world 画素への換算はここでしかできない — comp の寸法を知っているのは
        // store 側で、面(StageChrome)は比しか持っていない。
        let observation = view_camera.map(|view| ObservationCamera {
            pan: [
                (view.pan_fraction[0] * composition.width as f64) as f32,
                (view.pan_fraction[1] * composition.height as f64) as f32,
            ],
            zoom: view.zoom as f32,
        });
        let written = match observation {
            Some(observation) => backend.engine.render_frame_into_with_view_camera(
                &backend.doc.view(),
                t,
                gpu,
                &observation,
                true,
            ),
            None => backend.engine.render_frame_into(&backend.doc.view(), t, gpu),
        };
        // A05 隔離の読み出し口(発注 S3)。**frame を引いた後に読む、この1本だけ** —
        // engine の文字列をそのまま持ち帰る(意味を発明しない)。`written` の成否とは
        // 別の話(層ごとの隔離は render 呼び出し自体は成功させたまま起きる)なので、
        // 早期リターンの前に読んでおく。
        let layer_failures: Vec<String> = backend.engine.layer_failures().to_vec();
        if written.is_err() {
            return StageVerdict::stalled(StageRoom::Host, "writing into the shared surface failed");
        }
        let present = backend.present;
        let Some(texture) = backend.stage_texture.clone() else {
            return StageVerdict::stalled(StageRoom::Seam, "no shared Texture is held");
        };
        let stage_image = self.stage_image(cx);
        if stage_image.is_empty() {
            return StageVerdict::stalled(StageRoom::Seam, "the Stage Image is not in the panel");
        }
        // Stage chrome の常設帯へ渡す(裁定済みの置き場所)。加工せず列挙するのは
        // `StageChrome::set_failures` の仕事。
        self.set_stage_failures(cx, &layer_failures);
        // 「表示側が答えた寸法」を持って帰る。書けたことは見えたことではない。
        let displayed = texture
            .get_format(cx)
            .vec_width_height()
            .map(|(width, height)| (width as u32, height as u32));
        stage_image.set_texture(cx, Some(texture));
        cx.redraw_all();
        // 「書けた」で終わらせない。出たかどうかは表示寸法が答える。
        stage_surface::check_shown(present, desc, displayed)
    }

    fn dock(&self, cx: &mut Cx) -> DockRef {
        self.ui.widget(cx, ids!(panel.dock)).as_dock()
    }

    fn stage_image(&self, cx: &mut Cx) -> ImageRef {
        self.dock(cx)
            .item(id!(stage))
            .child_by_path(ids!(stage_frame))
            .as_image()
    }

    fn set_stage_error(&self, cx: &mut Cx, text: &str) {
        self.dock(cx)
            .item(id!(stage))
            .child_by_path(ids!(stage_error))
            .as_label()
            .set_text(cx, text);
        cx.redraw_all();
    }

    /// A05 隔離の読み出し口を Stage chrome の常設帯へ渡す(発注 S3)。**この1本だけ**
    /// が `Engine::layer_failures()` の読み手 — 加工は `StageChrome::set_failures`
    /// (空なら帯ごと隠す、非空なら engine の文字列をそのまま列挙)。
    fn set_stage_failures(&self, cx: &mut Cx, failures: &[String]) {
        if let Some(mut chrome) = self.stage_chrome_ref(cx).borrow_mut::<StageChrome>() {
            chrome.set_failures(cx, failures);
        }
    }

    fn request_stage_frame(&mut self, cx: &mut Cx) {
        if !self.stage_request.request() {
            self.stage_next_frame = cx.new_next_frame();
        }
    }

    /// `TimelineUpdate` → 窓の引き直し。**書いた側ではなくここが「何を引き直すか」を
    /// 決める** — `BackendBridge` は store を書いて何が変わったかを言うだけで、
    /// 面の都合を知らない。動詞が増えても触るのはこの1箇所。
    fn apply_timeline_update(&mut self, cx: &mut Cx, update: TimelineUpdate) {
        match update {
            TimelineUpdate::None => {}
            TimelineUpdate::Stage(status) => {
                // param の値は時刻の関数。プレイヘッドが動いたら FX の欄も引き直す。
                self.install_fx_model(cx);
                self.install_inspector_selection(cx);
                self.install_stage_gizmo(cx);
                self.request_stage_frame(cx);
                self.set_status(cx, &status);
            }
            TimelineUpdate::ModelAndStage(status) => {
                self.install_timeline_model(cx);
                self.install_fx_model(cx);
                self.install_inspector_selection(cx);
                self.install_stage_gizmo(cx);
                self.request_stage_frame(cx);
                self.set_status(cx, &status);
            }
            // 選択が動いた。FX STACK は**選択レイヤーの**効果を映すので、
            // 絵が変わらなくても面は引き直す。Inspector のヘッダ(発注 S5b)も同じ理由で
            // ここに揃える — 選択は Document を書かないので `Model` 以外の枝を通らない。
            // Stage ギズモ(S20)も選択が動くたび引き直す(同じ理由)。
            TimelineUpdate::Model(status) => {
                self.install_timeline_model(cx);
                self.install_fx_model(cx);
                self.install_inspector_selection(cx);
                self.install_stage_gizmo(cx);
                self.set_status(cx, &status);
            }
            TimelineUpdate::Status(status) => {
                self.set_status(cx, &status);
            }
        }
    }

    fn install_timeline_model(&mut self, cx: &mut Cx) {
        let started = Instant::now();
        let Some(model) = self.backend.as_ref().map(BackendBridge::timeline_model) else {
            return;
        };
        let timeline = self.timeline_ref(cx);
        let timeline_found = !timeline.is_empty();
        if let Some(mut timeline) = timeline.borrow_mut::<TimelineSurface>() {
            timeline.set_model(cx, model);
        }
        log!(
            "PERF timeline_projection elapsed_us={} timeline_found={}",
            started.elapsed().as_micros(),
            timeline_found,
        );
    }

    /// FX の面。Inspector 面の**兄弟**として `InspectorPane` に居る
    /// (置き場所の理由は `InspectorPane` の宣言のコメント参照)。
    fn fx_ref(&self, cx: &mut Cx) -> WidgetRef {
        self.dock(cx)
            .item(id!(inspector))
            .child_by_path(ids!(fx_stack))
    }

    fn fx_uid(&self, cx: &mut Cx) -> Option<WidgetUid> {
        let fx = self.fx_ref(cx);
        (!fx.is_empty()).then(|| fx.widget_uid())
    }

    /// `install_timeline_model` と同じ形。選択・プレイヘッド・Document のどれが
    /// 動いても、効果の面は Document から引き直す(面は正本を持たない)。
    fn install_fx_model(&mut self, cx: &mut Cx) {
        let Some(model) = self.backend.as_ref().map(BackendBridge::fx_model) else {
            return;
        };
        let fx = self.fx_ref(cx);
        if let Some(mut inner) = fx.borrow_mut::<FxStack>() {
            inner.set_model(cx, model);
        };
    }

    /// Inspector 面の `InspectorSurface`(`InspectorPane` の中、`fx_stack` の兄弟)。
    fn inspector_ref(&self, cx: &mut Cx) -> WidgetRef {
        self.dock(cx)
            .item(id!(inspector))
            .child_by_path(ids!(inspector_surface))
    }

    fn inspector_uid(&self, cx: &mut Cx) -> Option<WidgetUid> {
        let inspector = self.inspector_ref(cx);
        (!inspector.is_empty()).then(|| inspector.widget_uid())
    }

    /// 選択の投影を Inspector ヘッダへ押し込む(発注 S5b)。**FX STACK が既に選択を
    /// 読んでいる経路(`install_fx_model`)と同じ形** — 呼ぶ場所も同じにして揃える。
    /// 選択が無ければ `InspectorSurface::set_selection_summary` がその状態を
    /// 正直に映す("No Selection" / "Select a layer to inspect")。
    ///
    /// **KeyEase への投影もここで一緒に行う**(wf4 easing 板)。選択と playhead の
    /// どちらが動いてもこの関数を通るので、呼び場所を増やさずに済む。
    fn install_inspector_selection(&mut self, cx: &mut Cx) {
        let Some((name, kind)) = self.backend.as_ref().map(BackendBridge::selection_summary) else {
            return;
        };
        let key_states = self
            .backend
            .as_ref()
            .map(BackendBridge::inspector_key_states)
            .unwrap_or_default();
        let inspector = self.inspector_ref(cx);
        if let Some(mut surface) = inspector.borrow_mut::<InspectorSurface>() {
            surface.set_selection_summary(cx, &name, &kind);
            for (prop, keyed, interp) in key_states {
                surface.set_property_keys(cx, prop, keyed, interp);
            }
        };
    }

    /// 選択レイヤーの world transform を StageChrome へ渡す(ギズモの対象、S20)。
    /// StageChrome は Document を持たないので、ここが唯一の継ぎ目
    /// (`install_inspector_selection`/`set_stage_failures` と同じ形)。
    fn install_stage_gizmo(&mut self, cx: &mut Cx) {
        let Some(backend) = self.backend.as_ref() else {
            return;
        };
        let comp_dims = backend.gizmo_comp_dims();
        let target = backend.gizmo_target();
        if let Some(mut chrome) = self.stage_chrome_ref(cx).borrow_mut::<StageChrome>() {
            chrome.set_stage_gizmo(cx, comp_dims, target);
        }
    }

    /// Stage の空きクリック(S20 TARGET5)。当たり判定(`stage_pick`)は Document を
    /// 読むだけなのでここで行う — StageChrome から運ばれるのは comp 空間の点だけ。
    /// 当たれば選択、外せば解除(`select_from_timeline` の `layer_id: None` 規約が
    /// そのまま「必ず全解除」を担う)。
    fn apply_stage_pick(&mut self, cx: &mut Cx, comp_point: [f32; 2], additive: bool) {
        let Some(backend) = self.backend.as_mut() else {
            return;
        };
        let store = backend.doc.view();
        let Some(composition) = store.composition().ok().flatten() else {
            return;
        };
        let Ok(t) = RationalTime::try_from_frame(backend.session.playhead, composition.fps) else {
            return;
        };
        let picked = stage_pick(&store, t, comp_point).map(|layer| layer.0);
        drop(store);
        let status = backend.select_from_timeline(picked, additive);
        self.apply_timeline_update(cx, TimelineUpdate::Model(status));
    }

    fn browser(&self, cx: &mut Cx) -> WidgetRef {
        self.dock(cx).item(id!(browser))
    }

    fn browser_surface_ref(&self, cx: &mut Cx) -> WidgetRef {
        self.browser(cx).child_by_path(ids!(browser_surface))
    }

    /// 棚の投影を widget へ流す(`install_timeline_model` と同じ形)。
    /// **hot reload 後も呼ぶ** — `script_mod!` の再実行は widget を宣言状態へ戻すので、
    /// 呼ばないとカードが消えたまま黙る。
    fn install_browser_catalog(&mut self, cx: &mut Cx) {
        let Some(catalog) = self.backend.as_ref().map(BackendBridge::browser_catalog) else {
            return;
        };
        let items = catalog.len();
        let browser = self.browser_surface_ref(cx);
        let browser_found = !browser.is_empty();
        if let Some(mut surface) = browser.borrow_mut::<BrowserSurface>() {
            surface.set_catalog(cx, catalog);
        };
        // 棚が空に見えた時、原因が「台帳が空」か「widget に届いていない」かを
        // `/log` の1行で分ける(黙って空にしない)。
        log!("PERF browser_catalog items={items} browser_found={browser_found}");
    }

    fn browser_surface_uid(&self, cx: &mut Cx) -> Option<WidgetUid> {
        let browser = self.browser_surface_ref(cx);
        (!browser.is_empty()).then(|| browser.widget_uid())
    }

    /// Settings タブ(`inspector_tabs` の兄弟、`main.rs` 宣言の `settings := DockTab{}`)
    /// の中身。`fx_ref`/`inspector_ref` と同じ形 — Dock のタブ id で辿る。
    fn settings_ref(&self, cx: &mut Cx) -> WidgetRef {
        self.dock(cx)
            .item(id!(settings))
            .child_by_path(ids!(settings_surface))
    }

    fn settings_uid(&self, cx: &mut Cx) -> Option<WidgetUid> {
        let settings = self.settings_ref(cx);
        (!settings.is_empty()).then(|| settings.widget_uid())
    }

    /// comp 設定の投影(S8、`install_fx_model` と同じ形)。**hot reload 後も呼ぶ** —
    /// `script_mod!` の再実行は widget を宣言値へ戻す(`install_browser_catalog` の doc
    /// と同じ理由)。タブが選ばれる前でも安全に no-op する(`SettingsSurface` が
    /// 見つからなければ `borrow_mut` が `None` を返すだけ)。
    fn install_settings(&mut self, cx: &mut Cx) {
        let Some(composition) = self
            .backend
            .as_ref()
            .and_then(|backend| backend.doc.view().composition().ok().flatten())
        else {
            return;
        };
        let settings = self.settings_ref(cx);
        if let Some(mut surface) = settings.borrow_mut::<SettingsSurface>() {
            surface.set_composition(cx, &composition);
        };
    }

    /// Browser の「N のうち1つ」は makepad の radio group が持つ。排他と選択移動は
    /// `RadioButtonSet::selected` の担当で、色をこちらで塗り替えない
    /// (`active` は instance、`draw_bg.color*` は group 共有の uniform)。
    fn browser_radio_groups(&mut self, cx: &mut Cx, actions: &Actions) {
        let browser = self.browser(cx);
        browser
            .radio_button_set(
                cx,
                ids_array!(
                    browser_surface.tabs.media,
                    browser_surface.tabs.effects,
                    browser_surface.tabs.create,
                    browser_surface.tabs.panels
                ),
            )
            .selected(cx, actions)
            .map(|index| self.browser_tab = index);
        if let Some(index) = browser
            .radio_button_set(cx, browser_rail_ids!())
            .selected(cx, actions)
        {
            self.browser_rail = index;
        }
    }


    /// 選択は App が持つ。widget は投影であって正本ではない — `script_mod!` の
    /// 再実行(hot reload)は animator を宣言状態へ戻すので、そのたび投影し直す。
    fn apply_browser_selection(&self, cx: &mut Cx) {
        let browser = self.browser(cx);
        let tabs = browser.radio_button_set(
            cx,
            ids_array!(
                browser_surface.tabs.media,
                browser_surface.tabs.effects,
                browser_surface.tabs.create,
                browser_surface.tabs.panels
            ),
        );
        for (index, item) in tabs.iter().enumerate() {
            item.set_active(cx, index == self.browser_tab, Animate::No);
        }
        let rail = browser.radio_button_set(cx, browser_rail_ids!());
        for (index, item) in rail.iter().enumerate() {
            item.set_active(cx, index == self.browser_rail, Animate::No);
        }
    }

    /// UI 全体の拡縮。寸法トークンが1箇所に集まっているので、倍率もここ1つで済む。
    ///
    /// 窓の `dpi_override` でも同じ絵は作れるが、実行時に差し替えると `--remote` の
    /// grab が Metal のアサーションで落ちる(drawable と grab テクスチャの寸法不一致、
    /// 実測 2026-08-27)。検証手段を壊さない方を採る。
    /// 浮くタブ行を明かす判断。**機構は Dock、判断はここ**(fork 差分に製品の判断を
    /// 入れない — gesture fork と同じ切り方)。
    ///
    /// 開く引き金はセルの左上の隅だけにする。帯の全幅を引き金にすると、上端の操作へ
    /// 手を伸ばしただけで開いてしまう。開いた後は帯の全幅で保持する — でないと
    /// タブへ向かって右へ動いた瞬間に閉じる。
    fn reveal_tab_bars_under(&mut self, cx: &mut Cx, abs: Vec2d) {
        let dock = self.dock(cx);
        let bar = 25.0 * tokens::ui_scale();
        let corner = 140.0 * tokens::ui_scale();
        for (id, cell) in dock.floating_tab_bar_cells() {
            let open_zone = Rect {
                pos: cell.pos,
                size: dvec2(corner.min(cell.size.x), bar),
            };
            // 帯はセルの**上**に生えるので、保持ゾーンは境界をまたぐ
            let hold_zone = Rect {
                pos: dvec2(cell.pos.x, cell.pos.y - bar),
                size: dvec2(cell.size.x, bar * 2.0),
            };
            let shown = if self.revealed_bar == Some(id) {
                hold_zone.contains(abs)
            } else {
                open_zone.contains(abs)
            };
            dock.set_tab_bar_revealed(cx, id, shown);
            if shown {
                self.revealed_bar = Some(id);
            } else if self.revealed_bar == Some(id) {
                self.revealed_bar = None;
            }
        }
    }

    fn set_ui_scale(&mut self, cx: &mut Cx, percent: i32) {
        let percent = tokens::set_ui_scale_percent(percent);
        if percent == self.ui_scale_percent {
            return;
        }
        self.ui_scale_percent = percent;
        // トークンは `script_mod!` の式へ焼き込まれている。焼き直しは live edit の仕事
        // (makepad が iOS の safe-area inset に使っているのと同じ経路)。
        cx.request_live_edit();
        self.set_status(cx, &format!("UI SCALE  ·  {percent}%"));
    }

    fn set_status(&mut self, cx: &mut Cx, status: &str) {
        self.status_text = status.to_string();
        self.project_status(cx);
    }

    fn project_status(&self, cx: &mut Cx) {
        if self.status_text.is_empty() {
            return;
        }
        self.ui
            .widget(cx, ids!(panel.status))
            .as_label()
            .set_text(cx, &self.status_text);
    }

    fn timeline_ref(&self, cx: &mut Cx) -> WidgetRef {
        self.dock(cx)
            .item(id!(timeline))
            .child_by_path(ids!(timeline_surface))
    }

    fn stage_chrome_ref(&self, cx: &mut Cx) -> WidgetRef {
        self.dock(cx)
            .item(id!(stage))
            .child_by_path(ids!(stage_chrome))
    }

    fn stage_chrome_uid(&self, cx: &mut Cx) -> Option<WidgetUid> {
        let chrome = self.stage_chrome_ref(cx);
        (!chrome.is_empty()).then(|| chrome.widget_uid())
    }

    /// User View の観測視点。**Camera タブなら `None`** — 出力カメラは front が
    /// 持たないので、渡す物が無いのが正しい状態(裁定157)。
    /// 視点の持ち主は StageChrome 1つで、ここは写しを持たない(2つ持つと片方が古くなる)。
    fn stage_view_camera(&self, cx: &mut Cx) -> Option<StageViewCamera> {
        self.stage_chrome_ref(cx)
            .borrow::<StageChrome>()
            .and_then(|chrome| chrome.view_camera())
    }

    fn timeline_uid(&self, cx: &mut Cx) -> Option<WidgetUid> {
        let timeline = self.timeline_ref(cx);
        (!timeline.is_empty()).then(|| timeline.widget_uid())
    }

    fn play_uid(&self, cx: &mut Cx) -> Option<WidgetUid> {
        let play = self
            .dock(cx)
            .item(id!(timeline))
            .child_by_path(ids!(play_toggle));
        (!play.is_empty()).then(|| play.widget_uid())
    }

    /// パネル切替(panels.svg)。Browser 面を畳む/戻す。
    ///
    /// 面の幅の正本は Dock の splitter align 1つで、こちらは畳む前の幅だけ覚える。
    /// `FromA(0.0)` は掴み棒だけを残して A 側を潰す — 帯ごと消さないので、
    /// 畳んだ後もドラッグで戻せる(押した物が消えて戻せない、を作らない)。
    fn toggle_browser_panel(&mut self, cx: &mut Cx) {
        let dock = self.dock(cx);
        if self.browser_collapsed {
            // 畳む前が既に 0 幅だったなら宣言の既定へ戻す(そうしないと戻らない)
            let width = if self.browser_restore_width > 1.0 {
                self.browser_restore_width
            } else {
                300.0 * tokens::ui_scale()
            };
            dock.set_splitter_align(cx, id!(top_split), SplitterAlign::FromA(width), true);
            self.browser_collapsed = false;
            self.set_status(cx, "BROWSER  ·  SHOWN");
        } else {
            self.browser_restore_width = dock.splitter_position(id!(top_split)).unwrap_or(0.0);
            dock.set_splitter_align(cx, id!(top_split), SplitterAlign::FromA(0.0), true);
            self.browser_collapsed = true;
            self.set_status(cx, "BROWSER  ·  HIDDEN");
        }
    }

    /// 設定(filter.svg)。`SettingsPane` は inspector 側のタブとして既に居るので、
    /// 開くとは「そのタブを選ぶ」こと。新しい面を作らない。
    fn open_settings(&mut self, cx: &mut Cx) {
        self.dock(cx).select_tab(cx, id!(settings));
        // タブが今初めて実体化した場合の保険。Dock がタブ内容を先に持っていても
        // 後で持っていても、ここで一度合わせておけば黙って古い値のまま出ない。
        self.install_settings(cx);
        self.set_status(cx, "SETTINGS");
    }

    /// いま文字を打っている最中か。
    ///
    /// 欄を名指ししない — 面は他レーンが増やし続けるので、木を辿って
    /// 「キーフォーカスを持つ `TextInput` が居るか」だけを聞く。名指しにすると
    /// 欄が増えるたびにここが古くなり、Space が再生へ抜ける穴が戻る。
    fn text_entry_has_focus(&self, cx: &Cx) -> bool {
        fn focused_text_input(cx: &Cx, node: &WidgetRef) -> bool {
            if node.borrow::<TextInput>().is_some() && node.key_focus(cx) {
                return true;
            }
            let mut found = false;
            node.children(&mut |_id, child| {
                if !found {
                    found = focused_text_input(cx, &child);
                }
            });
            found
        }
        focused_text_input(cx, &self.ui)
    }

    /// 窓へ落ちてきたファイル。**音だけを引き受ける**(2026-08-18 裁定の
    /// 「音声ファイルのドロップ」)。
    ///
    /// **Q0**: 動画・画像は引き受けない — その道(`browser_surface::place_media`)は
    /// まだ窓から到達できる形になっていないので、ここで半分だけ配線すると
    /// 「棚を素通りする2本目の入口」が生える。引き受けないことを**状態行で言う**:
    /// 黙って落とすと、利用者からは「落としたのに何も起きない」としか見えない。
    fn handle_file_drop(&mut self, cx: &mut Cx, items: &[DragItem]) -> bool {
        // 音を先に片付ける。音と動画が一緒に落ちてきた時、状態行に残るべきなのは
        // 「入った物」であって「入らなかった物」ではない。
        let mut status: Option<String> = None;
        for item in items.iter().filter(|item| Self::is_audio_drag_item(item)) {
            let DragItem::FilePath { path, .. } = item else {
                continue;
            };
            let Some(backend) = self.backend.as_mut() else {
                continue;
            };
            status = Some(
                backend
                    .admit_soundtrack(Path::new(path))
                    .unwrap_or_else(|reason| reason),
            );
            backend.frame = None;
        }
        let Some(status) = status else {
            // 何も入らなかった。**外から来たファイル**なら、黙って落とさずに
            // 理由を言う(落としたのに何も起きない、が一番読めない)。窓の中の
            // ドラッグ(Dock のタブ = `internal_id` を持つ)には触らない —
            // ここで `handled` を立てるとタブの並べ替えを横取りする。
            let outsider = items.iter().find_map(|item| match item {
                DragItem::FilePath {
                    path,
                    internal_id: None,
                } if !path.is_empty() => Path::new(path)
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned()),
                _ => None,
            });
            if let Some(label) = outsider {
                self.set_status(cx, &format!("DROP  ·  {label}  ·  only audio can be dropped yet"));
            }
            // 受け取っていないので `handled` は立てない。
            return false;
        };
        self.install_timeline_model(cx);
        self.request_stage_frame(cx);
        self.set_status(cx, &status);
        true
    }

    /// 引き受ける気があるか(拡張子だけで決める)。`Event::Drag` は指が窓の上を
    /// 動くたびに来るので、ここでは**ファイルを開かない**。
    ///
    /// `internal_id` を持つ物は窓の中のドラッグ(Dock のタブ)なので対象外 —
    /// 素材の取り込みと、面の並べ替えは別の話。
    fn is_audio_drag_item(item: &DragItem) -> bool {
        let DragItem::FilePath { path, internal_id: None } = item else {
            return false;
        };
        let extension = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        AUDIO_EXTENSIONS.contains(&extension.as_str())
    }

    fn toggle_playback(&mut self, cx: &mut Cx) {
        let (playing, audio_issue) = self
            .backend
            .as_mut()
            .map(BackendBridge::toggle_playback)
            .unwrap_or((false, None));
        let status = if playing {
            match audio_issue {
                // TARGET 6: デバイスが開けない環境でも再生自体は止めない —
                // 理由だけ状態行へ出す(憲法3の許容型、静かな劣化)。
                Some(reason) => format!("PLAYING  ·  NO AUDIO DEVICE  ·  {reason}"),
                None => "PLAYING  ·  SPACE TO PAUSE".to_string(),
            }
        } else {
            "PAUSED".to_string()
        };
        self.set_status(cx, &status);
        self.install_timeline_model(cx);
        self.request_stage_frame(cx);
    }

    /// Delete/Backspace(発注 S4)。`toggle_playback` と同じ形の直接キー操作 —
    /// TimelineSurface を経由しないので `TimelineUpdate` は経由せず、ここで
    /// 投影を引き直す。選択も変わるので Inspector(発注 S5b)も一緒に引き直す。
    fn delete_selected_layers(&mut self, cx: &mut Cx) {
        let Some(backend) = self.backend.as_mut() else {
            return;
        };
        let status = backend.remove_selected_layers();
        self.set_status(cx, &status);
        self.install_timeline_model(cx);
        self.install_fx_model(cx);
        self.install_inspector_selection(cx);
        self.install_stage_gizmo(cx);
        self.request_stage_frame(cx);
    }

}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        self.backend = Some(BackendBridge::new_fixture());
        self.playback_timer = cx.start_interval(1.0 / 60.0);
        self.install_timeline_model(cx);
        self.install_fx_model(cx);
        self.install_inspector_selection(cx);
        self.install_stage_gizmo(cx);
        self.install_browser_catalog(cx);
        self.install_settings(cx);
        self.request_stage_frame(cx);
        self.browser_rail = RAIL_ALL_MEDIA;
        self.apply_browser_selection(cx);
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        self.browser_radio_groups(cx, actions);
        if self
            .ui
            .widget(cx, ids!(panel.chrome.browser_toggle))
            .as_button()
            .clicked(actions)
        {
            self.toggle_browser_panel(cx);
        }
        if self
            .ui
            .widget(cx, ids!(panel.chrome.settings))
            .as_button()
            .clicked(actions)
        {
            self.open_settings(cx);
        }
        // 棚 → タイムライン(カードの double-click)。書いた後は3つとも引き直す —
        // タイムラインに行が増え、Stage の絵が変わり、棚の ● が点く。
        if let Some(uid) = self.browser_surface_uid(cx) {
            let browser_edits: Vec<BrowserEditAction> =
                actions.filter_widget_actions_cast(uid).collect();
            for action in browser_edits {
                let BrowserEditAction::PlaceAsset { asset } = action else {
                    continue;
                };
                let Some(status) = self
                    .backend
                    .as_mut()
                    .map(|backend| backend.place_asset_from_browser(asset))
                else {
                    continue;
                };
                self.install_timeline_model(cx);
                self.install_browser_catalog(cx);
                self.request_stage_frame(cx);
                self.set_status(cx, &status);
            }
        }
        // 素材の口(WIRE-2)。`browser_surface::handle_import_actions` が唯一の受け口 —
        // Import ボタンの意図(`BrowserSurfaceAction::ImportMedia`)をここで doc へ書く。
        // 失敗理由も含めて戻り値をそのまま状態行へ流す(黙って落とさない)。
        let browser = self.browser(cx);
        let import_status = self.backend.as_mut().and_then(|backend| {
            browser_surface::handle_import_actions(
                cx,
                &browser,
                actions,
                &mut backend.doc,
                &mut backend.session,
            )
        });
        if let Some(status) = import_status {
            if let Some(backend) = self.backend.as_mut() {
                backend.frame = None;
            }
            self.install_timeline_model(cx);
            self.install_browser_catalog(cx);
            self.request_stage_frame(cx);
            self.set_status(cx, &status);
        }
        if let Some(uid) = self.play_uid(cx) {
            if actions
                .filter_widget_actions(uid)
                .any(|action| matches!(action.cast(), ButtonAction::Clicked { .. }))
            {
                self.toggle_playback(cx);
            }
        }
        // FX の編集意図。Timeline と同じ流儀(uid で絞って1つずつ Document へ写す)。
        // `filter_widget_actions_cast` は型の合わない action を `Default`(= `None`)へ
        // 落とすので、その `None` は素通しする。
        if let Some(uid) = self.fx_uid(cx) {
            let fx_actions: Vec<FxStackAction> = actions.filter_widget_actions_cast(uid).collect();
            for action in fx_actions {
                if matches!(action, FxStackAction::None) {
                    continue;
                }
                let Some(backend) = self.backend.as_mut() else {
                    continue;
                };
                let write = backend.apply_fx_action(&action);
                if write.wrote {
                    // 効果の param track は Timeline の property 行にも出る。
                    self.install_timeline_model(cx);
                    self.request_stage_frame(cx);
                }
                self.install_fx_model(cx);
                self.install_inspector_selection(cx);
                self.install_stage_gizmo(cx);
                if !write.status.is_empty() {
                    self.set_status(cx, &write.status);
                }
            }
        }

        // Inspector の ◆/緩急(wf4 INTERVAL EASING 板)。FX の action ループと同じ形
        // (uid で絞って1つずつ Document へ写し、書けたら選択の投影をやり直す —
        // `install_inspector_selection` が KeyEase への投影も一緒に持っている)。
        if let Some(uid) = self.inspector_uid(cx) {
            let inspector_actions: Vec<InspectorSurfaceAction> =
                actions.filter_widget_actions_cast(uid).collect();
            for action in inspector_actions {
                let Some(backend) = self.backend.as_mut() else {
                    continue;
                };
                let status = match action {
                    InspectorSurfaceAction::None | InspectorSurfaceAction::SetValue { .. } => {
                        continue;
                    }
                    InspectorSurfaceAction::ToggleKey { prop, keyed } => {
                        backend.apply_inspector_toggle_key(&prop, keyed)
                    }
                    InspectorSurfaceAction::SetInterp { prop, interp } => {
                        backend.apply_inspector_interp(&prop, interp)
                    }
                };
                self.install_inspector_selection(cx);
                self.install_timeline_model(cx);
                self.request_stage_frame(cx);
                self.set_status(cx, &status);
            }
        }

        // comp 設定(S8)。SettingsSurface が出す確定値
        // ([`SettingsSurfaceAction::SetField`])を `Intent::SetComposition` へ書く。
        // fps/尺が動けば Timeline の目盛りと再生範囲も、寸法が動けば共有 surface も
        // 付いてくるので、書けた時は Timeline の投影と Stage をまとめて引き直す
        // (FX の書き込みと同じ形)。
        if let Some(uid) = self.settings_uid(cx) {
            let settings_actions: Vec<SettingsSurfaceAction> =
                actions.filter_widget_actions_cast(uid).collect();
            for action in settings_actions {
                let SettingsSurfaceAction::SetField { field, value } = action else {
                    continue;
                };
                let Some(status) = self
                    .backend
                    .as_mut()
                    .map(|backend| backend.apply_settings_action(field, value))
                else {
                    continue;
                };
                self.install_timeline_model(cx);
                self.install_settings(cx);
                self.request_stage_frame(cx);
                self.set_status(cx, &status);
            }
        }

        // Stage chrome の action(視点変更・ギズモ確定・pick、S20)。3種とも同じ
        // uid から出るので1回だけ集めて種類ごとに分ける(FX の action ループと同じ形)。
        if let Some(uid) = self.stage_chrome_uid(cx) {
            let stage_actions: Vec<StageChromeAction> =
                actions.filter_widget_actions_cast(uid).collect();
            for action in stage_actions {
                match action {
                    StageChromeAction::None => {}
                    // 視点が動いたら Stage を描き直す。**Document は触らない** —
                    // 見回しは意味を変えないので `install_timeline_model` も undo も
                    // 通らない(裁定157/271)。
                    StageChromeAction::ViewChanged => {
                        self.request_stage_frame(cx);
                    }
                    // ギズモのドラッグが確定した(S20 TARGET4)。値は
                    // `take_gizmo_commit` から1回だけ取り出す — action 自体は運ばない。
                    StageChromeAction::GizmoCommitted => {
                        let commit = self
                            .stage_chrome_ref(cx)
                            .borrow_mut::<StageChrome>()
                            .and_then(|mut chrome| chrome.take_gizmo_commit());
                        let Some(commit) = commit else {
                            continue;
                        };
                        let status = self
                            .backend
                            .as_mut()
                            .map(|backend| backend.apply_gizmo_commit(&commit))
                            .unwrap_or_default();
                        if status.is_empty() {
                            continue;
                        }
                        self.install_timeline_model(cx);
                        self.install_stage_gizmo(cx);
                        self.request_stage_frame(cx);
                        self.set_status(cx, &status);
                    }
                    // 空きクリック(S20 TARGET5)。当たり判定は Document を読む
                    // main.rs の仕事 — StageChrome は comp 空間の点しか運ばない。
                    StageChromeAction::StagePicked {
                        comp_point,
                        additive,
                    } => {
                        self.apply_stage_pick(cx, comp_point, additive);
                    }
                }
            }
        }

        let Some(uid) = self.timeline_uid(cx) else {
            return;
        };
        // Timeline は2種の action を同じ uid から出す(入力と編集意図)。
        // `filter_widget_actions_cast` は型の合わない action を `Default`(= `None`)へ
        // 落とすので、それぞれの型で1回ずつ拾って、片方の `None` は素通りさせる。
        let timeline_actions: Vec<TimelineSurfaceAction> =
            actions.filter_widget_actions_cast(uid).collect();
        let edit_actions: Vec<TimelineEditAction> =
            actions.filter_widget_actions_cast(uid).collect();
        for action in timeline_actions {
            let update = self
                .backend
                .as_mut()
                .map(|backend| backend.apply_timeline_action(&action))
                .unwrap_or(TimelineUpdate::None);
            self.apply_timeline_update(cx, update);
        }
        for action in edit_actions {
            let update = self
                .backend
                .as_mut()
                .map(|backend| backend.apply_timeline_edit(&action))
                .unwrap_or(TimelineUpdate::None);
            self.apply_timeline_update(cx, update);
        }
    }

    fn handle_timer(&mut self, cx: &mut Cx, event: &TimerEvent) {
        if self.playback_timer.is_timer(event).is_none() {
            return;
        }
        let was_playing = self.backend.as_ref().map(|backend| backend.playing).unwrap_or(false);
        let changed = self
            .backend
            .as_mut()
            .map(BackendBridge::playback_tick)
            .unwrap_or(false);
        if changed {
            let still_playing = self.backend.as_ref().map(|backend| backend.playing).unwrap_or(false);
            if was_playing && !still_playing {
                // comp 末尾での自然停止(TARGET 5)。状態行を PLAYING のまま
                // 残さない — `toggle_playback` が Space で止めた時と同じ表示。
                self.set_status(cx, "PAUSED");
            }
            self.install_timeline_model(cx);
            self.install_fx_model(cx);
            self.install_inspector_selection(cx);
            self.install_stage_gizmo(cx);
            self.request_stage_frame(cx);
        }
    }

    fn handle_key_down(&mut self, cx: &mut Cx, event: &KeyEvent) {
        // `MatchEvent` は `ui.handle_event` より先に来る。フォーカスのある欄より先に
        // ここが Space を食べると、名前を打っている最中に再生が始まる。
        // 文字を打っている間は窓のショートカットを名乗らない — 手前の欄が持ち主。
        if self.text_entry_has_focus(cx) {
            return;
        }
        if event.modifiers.logo || event.modifiers.control {
            let step = if event.modifiers.shift { 10 } else { 1 };
            match event.key_code {
                KeyCode::Equals => {
                    self.set_ui_scale(cx, self.ui_scale_percent + step);
                    return;
                }
                KeyCode::Minus => {
                    self.set_ui_scale(cx, self.ui_scale_percent - step);
                    return;
                }
                KeyCode::Key0 => {
                    self.set_ui_scale(cx, 100);
                    return;
                }
                _ => {}
            }
        }
        if event.key_code == KeyCode::Space && !event.is_repeat {
            self.toggle_playback(cx);
        }
        // レイヤー削除(発注 S4)。Delete と Backspace の両方を受ける — キーボードの
        // 種類で片方しか無い機種があるので、どちらも同じ意図として通す。
        if matches!(event.key_code, KeyCode::Delete | KeyCode::Backspace) && !event.is_repeat {
            self.delete_selected_layers(cx);
        }
    }

    fn handle_next_frame(&mut self, cx: &mut Cx, event: &NextFrameEvent) {
        if event.set.contains(&self.stage_next_frame) && self.stage_request.take() {
            self.install_stage_frame(cx);
        }
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        // Ableton の identity は palette ではなく形の文法にある(18テーマ同梱 =
        // 色を全部差し替えても Ableton に見える、が証拠)。テーマ横断で不変なのは:
        // 矩形のみ・角丸ゼロ・ベベルゼロ・影ゼロ、分離は 1px の暗線と明度段。
        // makepad の既定(corner_radius 2.5 / beveling 0.75)はその全部に反するので、
        // widget が theme.* を読む**前**に根を書き換える。現場の数百箇所を触らない。
        crate::makepad_widgets::theme_mod(vm);
        crate::theme_flat::script_mod(vm);
        crate::makepad_widgets::widgets_mod(vm);
        // 目盛りは誰よりも先。surface はこれを引く。
        crate::tokens::script_mod(vm);
        // Widget modules register before the UI modules that import them (DSL 正史)。
        // chrome (parts / gallery 含む) が先、surface 群が後。
        crate::chrome::script_mod(vm);
        crate::browser_surface::script_mod(vm);
        crate::stage_chrome::script_mod(vm);
        crate::inspector_surface::script_mod(vm);
        // FX の param 欄は inspector が登録する `ScrubValue` なので、必ずその後。
        crate::fx_stack::script_mod(vm);
        crate::export_surface::script_mod(vm);
        crate::settings_surface::script_mod(vm);
        crate::timeline_surface::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        // hot reload は `script_mod!` を再実行して widget を宣言状態へ戻す。
        // 選択は App が持っているので、投影し直すのはこちらの責任。
        if let Event::MouseMove(move_event) = event {
            let abs = move_event.abs;
            self.reveal_tab_bars_under(cx, abs);
        }
        // 窓の外へ出たら浮きタブを畳む。MouseMove は窓内でしか来ないので、
        // 出しっぱなしで固まるのはここを聞いていない時だけ
        if matches!(event, Event::MouseLeave(_)) {
            self.reveal_tab_bars_under(cx, dvec2(-1.0e6, -1.0e6));
        }
        if matches!(event, Event::LiveEdit) {
            self.apply_browser_selection(cx);
            self.install_browser_catalog(cx);
            self.install_settings(cx);
            self.project_status(cx);
        }
        // ドロップは**2段**。`Event::Drag` へ `Copy` と答えないと macOS の
        // `draggingEntered:` が `NSDragOperation::None` を返し、`Event::Drop` は
        // そもそも来ない(カーソルは「不可」のまま指が離せない)。受け取る気が
        // あることを先に言う — Q0「触れそうで触れない物を作らない」がここに効く。
        if let Event::Drag(drag_event) = event {
            if drag_event.items.iter().any(Self::is_audio_drag_item) {
                *drag_event.response.lock().unwrap() = DragResponse::Copy;
            }
        }
        if let Event::Drop(drop_event) = event {
            if self.handle_file_drop(cx, &drop_event.items) {
                // `performDragOperation:` はこの値をそのまま OS へ返す。false の
                // ままだと、取り込みは成功しているのにファイルが元の場所へ
                // 弾かれて戻るアニメーションが出る(効いたのに効かなく見える)。
                *drop_event.handled.lock().unwrap() = true;
            }
        }
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

#[cfg(test)]
mod backend_tests {
    use super::*;

    #[test]
    fn stage_present_starts_as_named_cpu_fallback() {
        let backend = BackendBridge::new_fixture();
        assert_eq!(backend.present, StagePresent::FallbackCpu);
        assert!(!backend.present.is_zero_copy());
    }

    #[test]
    fn scrub_action_reaches_session() {
        let mut backend = BackendBridge::new_fixture();
        let update = backend.apply_timeline_action(&TimelineSurfaceAction::Scrub(600));

        assert!(matches!(update, TimelineUpdate::Stage(_)));
        assert_eq!(backend.session.playhead, 600);
    }

    #[test]
    fn lane_restack_action_changes_document_derived_stage_order() {
        let mut backend = BackendBridge::new_fixture();
        let layer_id = backend
            .timeline_model()
            .lanes
            .last()
            .expect("fixture lane")
            .id;
        let update = backend.apply_timeline_action(&TimelineSurfaceAction::Restack {
            layer_id,
            target_from_front: 0,
        });

        assert!(matches!(update, TimelineUpdate::ModelAndStage(_)));
        assert_eq!(backend.timeline_model().lanes[0].id, layer_id);
    }

    #[test]
    fn stage_requests_coalesce_to_one_latest_delivery() {
        let mut requests = LatestFrameRequest::default();
        assert!(!requests.request(), "first request schedules a consumer");
        assert!(
            requests.request(),
            "later requests reuse the pending consumer"
        );
        assert!(requests.take());
        assert!(
            !requests.take(),
            "one delivery consumes every coalesced request"
        );
    }
}
