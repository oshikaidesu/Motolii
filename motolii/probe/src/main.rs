use std::any::Any;
use std::sync::Arc;

use dioxus_native::prelude::*;
use dioxus_native::CustomWidgetAttr;
use motolii_store::{property, Document, KeyframeTrack, PropertyId, Value};

mod playback;
mod stage_widget;
mod timeline_widget;
mod tokens;
use playback::Clock;
use tokens::UiScale;
use stage_widget::StageWidget;
use timeline_widget::{CanvasRow, TimelineWidget};

static STYLES: &str = include_str!("styles.css");

const FPS: f64 = 30.0;

/// LayerId % 12 → 差し色。長さはmotolii_tokens_rs::LABEL_PALETTE_LENと同じ12。
const LABEL_PALETTE: [&str; 12] = [
    "#d96b6b", "#d9985a", "#d9c95a", "#a3d95a", "#5ad98c", "#5ac6c6",
    "#5a96d9", "#7d7dd9", "#a86bd9", "#d96bbc", "#9a9a9a", "#cba97a",
];

fn main() {
    // re_rendererは共有deviceのuncaptured-errorハンドラを乗っ取りre_logへ流す。
    // ロガー未初期化だとwgpu検証エラーが無音で消えるので必ず先に立てる。
    re_log::setup_logging();
    let config: Vec<Box<dyn Any>> = vec![];
    dioxus_native::launch_cfg(app, Vec::new(), config);
}

/// Documentから行を読む。ドラッグcommit後の再読みにも同じ関数を使う。
fn canvas_rows_from_doc(doc: &Document) -> Vec<CanvasRow> {
    let view = doc.view();
    let props: Vec<PropertyId> = [property::OPACITY, property::POSITION]
        .iter()
        .filter_map(|p| PropertyId::new(p).ok())
        .collect();

    view.layers()
        .into_iter()
        .map(|layer| {
            let (start, duration) = view
                .meta(layer)
                .ok()
                .flatten()
                .map(|m| (m.timing.start, m.timing.duration))
                .unwrap_or((0, 0));
            let color_ix = view
                .attrs(layer)
                .ok()
                .flatten()
                .and_then(|a| a.label_color)
                .unwrap_or(10);
            let mut keys = Vec::new();
            for prop in &props {
                if let Ok(Some(track)) = view.track(layer, prop) {
                    keys.extend(track.keys().iter().map(|k| k.t.as_seconds_f64()));
                }
            }
            keys.sort_by(|a, b| a.partial_cmp(b).unwrap());
            CanvasRow {
                is_group: false,
                keys,
                span: Some((start as f64 / FPS, (start + duration) as f64 / FPS)),
                agg: Vec::new(),
                layer: Some(layer),
                color: label_rgb(color_ix),
            }
        })
        .collect()
}

fn label_rgb(ix: u8) -> [u8; 3] {
    let hex = LABEL_PALETTE[ix as usize % LABEL_PALETTE.len()];
    let v = u32::from_str_radix(&hex[1..], 16).unwrap_or(0x8c8c8c);
    [(v >> 16) as u8, (v >> 8) as u8, v as u8]
}

struct LayerRow {
    name: String,
    color: &'static str,
}

struct AssetRow {
    name: String,
    kind: String,
    thumb: &'static str,
}

/// Inspector 1行。cells[i]が空文字のセルは地なし(モックの空白セルと同じ)。
struct PropRow {
    label: &'static str,
    cells: [String; 3],
    dims: [bool; 3],
    keyed: bool,
}

struct InspectorData {
    ident_name: String,
    ident_sub: String,
    transform: Vec<PropRow>,
    appearance: Vec<PropRow>,
    has_effects: bool,
}

struct UiData {
    layer_rows: Vec<LayerRow>,
    assets: Vec<AssetRow>,
    comp_line: String,
    inspector: InspectorData,
    status: String,
}

struct Loaded {
    doc: Document,
    ui: UiData,
    duration_sec: f64,
    /// サビ歌詞 position(Bezierイージング入り)。Stageのカメラを駆動する。
    sabi_position: Option<KeyframeTrack>,
    /// タイトルロゴ opacity。Stageのカメラ距離を駆動する。
    logo_opacity: Option<KeyframeTrack>,
}

fn load_fixture() -> Loaded {
    let fx = motolii_fixture::build();
    let view = fx.doc.view();

    let mut layer_rows = Vec::new();
    let mut sabi_position = None;
    let mut logo_opacity = None;
    for layer in view.layers() {
        let attrs = view.attrs(layer).ok().flatten().unwrap_or_default();
        if attrs.name == "サビ歌詞" {
            if let Ok(Some(track)) =
                view.track(layer, &PropertyId::new(property::POSITION).unwrap())
            {
                sabi_position = Some(track);
            }
        }
        if attrs.name == "タイトルロゴ" {
            if let Ok(Some(track)) =
                view.track(layer, &PropertyId::new(property::OPACITY).unwrap())
            {
                logo_opacity = Some(track);
            }
        }
        let color = attrs
            .label_color
            .map(|ix| LABEL_PALETTE[ix as usize % LABEL_PALETTE.len()])
            .unwrap_or("#8c8c8c");
        layer_rows.push(LayerRow { name: attrs.name, color });
    }

    let assets = view
        .assets()
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(i, a)| AssetRow {
            name: a.name,
            kind: a.asset_type,
            thumb: ["#6f8fb5", "#8f7fb8", "#6fb58a", "#b59a6f"][i % 4],
        })
        .collect();

    let comp_line = view
        .composition()
        .ok()
        .flatten()
        .map(|c| {
            format!(
                "{}×{} · {}fps · {}s",
                c.width,
                c.height,
                c.fps.num(),
                c.duration_frames / c.fps.num()
            )
        })
        .unwrap_or_default();

    let t = motolii_store::RationalTime::try_new(fx.playhead, 30)
        .unwrap_or(motolii_store::RationalTime::ZERO);
    let value_of = |prop: &str| {
        PropertyId::new(prop)
            .ok()
            .and_then(|p| view.value_at(fx.selected, &p, t).ok().flatten())
    };
    let keyed = |prop: &str| {
        PropertyId::new(prop)
            .ok()
            .and_then(|p| view.track(fx.selected, &p).ok().flatten())
            .is_some()
    };
    let f = |v: f64| format!("{v:.3}");

    // comp px級の値は38px幅セルに3桁小数が収まらないので1桁へ落とす。
    let f1 = |v: f64| format!("{v:.1}");
    let (px, py) = match value_of(property::POSITION) {
        Some(Value::Vec2([x, y])) => (f1(x), f1(y)),
        _ => (f1(0.0), f1(0.0)),
    };
    let opacity = match value_of(property::OPACITY) {
        Some(Value::F64(v)) => f(v),
        _ => f(1.0),
    };

    let sel_attrs = view.attrs(fx.selected).ok().flatten().unwrap_or_default();
    let key_count: usize = [property::POSITION, property::OPACITY]
        .iter()
        .filter_map(|p| PropertyId::new(p).ok())
        .filter_map(|p| view.track(fx.selected, &p).ok().flatten())
        .map(|tr| tr.keys().len())
        .sum();
    let has_effects = !view.effects(fx.selected).unwrap_or_default().is_empty();

    let inspector = InspectorData {
        ident_name: sel_attrs.name,
        ident_sub: format!("solid · {key_count} keys"),
        transform: vec![
            PropRow {
                label: "Position",
                cells: [px, py, f(0.0)],
                dims: [false, false, true],
                keyed: keyed(property::POSITION),
            },
            PropRow {
                label: "Scale",
                cells: [f(1.0), f(1.0), f(1.0)],
                dims: [false, false, true],
                keyed: keyed(property::SCALE),
            },
            PropRow {
                label: "Anchor",
                cells: [f(0.0), f(0.0), f(0.0)],
                dims: [false, false, true],
                keyed: keyed(property::ANCHOR),
            },
            PropRow {
                label: "Rotation",
                cells: [String::new(), String::new(), f(0.0)],
                dims: [false, false, true],
                keyed: keyed(property::ROTATION),
            },
        ],
        appearance: vec![PropRow {
            label: "Opacity",
            cells: [String::new(), String::new(), opacity],
            dims: [false, false, false],
            keyed: keyed(property::OPACITY),
        }],
        has_effects,
    };
    drop(view);

    Loaded {
        doc: fx.doc,
        ui: UiData { layer_rows, assets, comp_line, inspector, status: fx.status },
        duration_sec: 60.0,
        sabi_position,
        logo_opacity,
    }
}

fn fmt_timecode(sec: f64) -> String {
    let f = (sec * FPS).round() as i64;
    format!("{}:{:02}:{:02}", f / 1800, (f / 30) % 60, f % 30)
}

#[derive(Clone, Copy, PartialEq)]
enum DragTarget {
    Browser,
    Inspector,
    Timeline,
}

struct DragSplit {
    target: DragTarget,
    start: f64,
    orig: f64,
}

fn prop_row(p: &PropRow) -> Element {
    let cells = p.cells.iter().zip(p.dims).map(|(c, dim)| {
        let class = if c.is_empty() {
            "v blank"
        } else if dim {
            "v z"
        } else {
            "v"
        };
        rsx!(span { class: "{class}", "{c}" })
    });
    let key_class = if p.keyed { "glyph on" } else { "glyph" };
    rsx!(
        div { class: "prow",
            span { class: "n", "{p.label}" }
            {cells}
            span { class: "{key_class}", if p.keyed { "◆" } else { "◇" } }
        }
    )
}

fn app() -> Element {
    let mut playing = use_signal(|| false);
    let mut browser_w = use_signal(|| 300.0f64);
    let mut inspector_w = use_signal(|| 270.0f64);
    let mut timeline_h = use_signal(|| 300.0f64);
    let mut drag = use_signal(|| Option::<DragSplit>::None);
    let mut msl = use_signal(std::collections::HashSet::<(usize, u8)>::new);

    let mut scale_pct = use_signal(|| 100u32);

    let (clock, ui_scale, timeline_attr, stage_attr, loaded) = use_hook(|| {
        let Loaded { doc, ui, duration_sec, sabi_position, logo_opacity } = load_fixture();
        let clock = Arc::new(Clock::new(duration_sec));
        let ui_scale = Arc::new(UiScale::new(100));

        let canvas_rows = canvas_rows_from_doc(&doc);
        let timeline = TimelineWidget::new(canvas_rows)
            .with_clock(clock.clone())
            .with_scale(ui_scale.clone())
            .with_document(doc, canvas_rows_from_doc);
        let stage = StageWidget::new(clock.clone(), sabi_position, logo_opacity);
        (
            clock,
            ui_scale,
            CustomWidgetAttr::new(timeline),
            CustomWidgetAttr::new(stage),
            Arc::new(ui),
        )
    });

    let layer_rows = loaded.layer_rows.iter().enumerate().map(|(i, row)| {
        let glyph = |bit: u8, label: &'static str| {
            let lit = msl.read().contains(&(i, bit));
            let class = if lit { "glyph lit" } else { "glyph" };
            rsx!(
                span {
                    class: "{class}",
                    onclick: move |_| {
                        let mut set = msl.write();
                        if !set.remove(&(i, bit)) {
                            set.insert((i, bit));
                        }
                    },
                    "{label}"
                }
            )
        };
        rsx!(
            div { class: "lrow",
                span { class: "lsurface", style: "background:{row.color};", "{row.name}" }
                div { class: "lctrl",
                    {glyph(0, "M")}
                    {glyph(1, "S")}
                    {glyph(2, "L")}
                }
            }
        )
    });

    let asset_cards = loaded.assets.iter().map(|a| {
        rsx!(
            div { class: "tcard",
                div { class: "thumb", style: "background:{a.thumb};" }
                span { class: "tname", "{a.name}" }
                span { class: "tmeta", "{a.kind}" }
            }
        )
    });
    let first_asset = loaded.assets.first().map(|a| a.name.clone()).unwrap_or_default();
    let asset_count = loaded.assets.len();

    let transform_rows = loaded.inspector.transform.iter().map(prop_row);
    let appearance_rows = loaded.inspector.appearance.iter().map(prop_row);
    let fx_label = if loaded.inspector.has_effects { "" } else { "No shared FX" };

    let timecode = fmt_timecode(clock.now_sec());
    let layer_count = loaded.layer_rows.len();

    let bw = browser_w();
    let iw = inspector_w();
    let th = timeline_h();
    let css = format!("{}{}", tokens::css_root(scale_pct()), STYLES);

    rsx!(
        style { {css} }
        div {
            id: "app",
            style: "grid-template-rows: var(--section) 1fr 8px {th}px calc(20 * var(--s) * 1px);",
            onmousemove: move |evt| {
                if let Some(d) = drag.read().as_ref() {
                    let p = evt.data().client_coordinates();
                    match d.target {
                        DragTarget::Browser => {
                            *browser_w.write() = (d.orig + (p.x - d.start)).clamp(140.0, 420.0);
                        }
                        DragTarget::Inspector => {
                            *inspector_w.write() = (d.orig - (p.x - d.start)).clamp(180.0, 420.0);
                        }
                        DragTarget::Timeline => {
                            *timeline_h.write() = (d.orig - (p.y - d.start)).clamp(120.0, 600.0);
                        }
                    }
                }
            },
            onmouseup: move |_| {
                *drag.write() = None;
            },

            div { id: "menubar",
                span { class: "appname", "Motolii" }
                span { class: "menu", "File" }
                span { class: "menu", "Edit" }
                span { class: "menu", "Layer" }
                span { class: "menu", "Effect" }
                span { class: "menu", "View" }
                span { class: "menu", "Help" }
                div { class: "zoomctl",
                    span {
                        class: "zbtn",
                        onclick: {
                            let ui_scale = ui_scale.clone();
                            move |_| {
                                ui_scale.set_percent(ui_scale.percent().saturating_sub(1));
                                *scale_pct.write() = ui_scale.percent();
                            }
                        },
                        "−"
                    }
                    span { class: "zval", "{scale_pct()}%" }
                    span {
                        class: "zbtn",
                        onclick: {
                            let ui_scale = ui_scale.clone();
                            move |_| {
                                ui_scale.set_percent(ui_scale.percent() + 1);
                                *scale_pct.write() = ui_scale.percent();
                            }
                        },
                        "+"
                    }
                }
            }

            div {
                id: "main",
                style: "grid-template-columns: {bw}px 8px 1fr 8px {iw}px;",

                div { id: "browser",
                    div { class: "ptitle",
                        span { class: "way", style: "background:var(--way-browser);" }
                        "Browser"
                        em { "LOCAL LIBRARY" }
                    }
                    div { class: "btoolbar",
                        span { class: "hbtn", "‹" }
                        span { class: "hbtn", "›" }
                        span { class: "search", "Search files and tags" }
                        span { class: "tbtn", "Filters" }
                        span { class: "tbtn", "Tags" }
                    }
                    div { class: "btabs",
                        span { class: "btab on", "Media" }
                        span { class: "btab", "Effects" }
                        span { class: "btab", "Create" }
                        span { class: "btab", "Panels" }
                    }
                    div { class: "bwork",
                        div { class: "bside",
                            div { class: "sh", "LIBRARY" }
                            div { class: "srow on", "All media" }
                            div { class: "srow", "Video" }
                            div { class: "srow", "Images" }
                            div { class: "srow", "Audio" }
                            div { class: "sh", "PLACES" }
                            div { class: "srow", "Comp 1" }
                        }
                        div { class: "bresults",
                            div { class: "rhead",
                                div {
                                    b { "All media" }
                                    span { class: "sub", "Library · fixture" }
                                }
                                div { class: "sp",
                                    span { class: "glyph", "▦" }
                                    span { class: "glyph on", "▤" }
                                }
                            }
                            div { class: "rcount",
                                "Results"
                                em { "{asset_count}" }
                            }
                            div { class: "tgrid", {asset_cards} }
                            div { class: "bfoot",
                                span { class: "dot", style: "background:var(--accent);" }
                                "{first_asset}"
                                em { "Edit tags" }
                            }
                        }
                    }
                }

                div {
                    class: "vgrip",
                    onmousedown: move |evt| {
                        let p = evt.data().client_coordinates();
                        *drag.write() = Some(DragSplit {
                            target: DragTarget::Browser,
                            start: p.x,
                            orig: browser_w(),
                        });
                    },
                }

                div { id: "stagecol",
                    div { id: "stagehead",
                        span { class: "way", style: "background:var(--way-stage);" }
                        "Stage"
                        em { "{loaded.comp_line}" }
                    }
                    div { id: "stage",
                        object { "data": stage_attr }
                    }
                }

                div {
                    class: "vgrip",
                    onmousedown: move |evt| {
                        let p = evt.data().client_coordinates();
                        *drag.write() = Some(DragSplit {
                            target: DragTarget::Inspector,
                            start: p.x,
                            orig: inspector_w(),
                        });
                    },
                }

                div { id: "inspector",
                    div { class: "ptitle",
                        span { class: "way", style: "background:var(--way-inspector);" }
                        "Inspector"
                        em { "solid" }
                    }
                    div { class: "ident",
                        div {
                            b { "{loaded.inspector.ident_name}" }
                            span { class: "sub", "{loaded.inspector.ident_sub}" }
                        }
                        div { class: "sp",
                            span { class: "glyph", "M" }
                            span { class: "glyph on", "S" }
                        }
                    }
                    div { class: "cols",
                        span { class: "pn", "Property" }
                        span { "X" }
                        span { "Y" }
                        span { "Z" }
                        span { class: "k", "Key" }
                    }
                    div { class: "sec", "TRANSFORM" }
                    {transform_rows}
                    div { class: "sec", "APPEARANCE" }
                    {appearance_rows}
                    div { class: "sec", "EFFECTS" }
                    div { class: "prow",
                        span { class: "n empty", "{fx_label}" }
                        span { class: "v blank", "" }
                        span { class: "v blank", "" }
                        span { class: "v blank", "" }
                        span { "" }
                    }
                    div { class: "hint", "Drag to scrub · double-click to type · Esc to cancel" }
                }
            }

            div {
                class: "hgrip",
                onmousedown: move |evt| {
                    let p = evt.data().client_coordinates();
                    *drag.write() = Some(DragSplit {
                        target: DragTarget::Timeline,
                        start: p.y,
                        orig: timeline_h(),
                    });
                },
            }

            div { id: "timelineshell",
                div { id: "tp",
                    button {
                        id: "play",
                        onclick: {
                            let clock = clock.clone();
                            move |_| {
                                clock.toggle();
                                *playing.write() = clock.playing();
                            }
                        },
                        if playing() { "■" } else { "▶" }
                    }
                    span { class: "tc", "{timecode}" }
                    em { "{layer_count} rows · 30fps · 60s" }
                }
                div { id: "timeline",
                    div { id: "layers",
                        div { class: "lhead", "OBJECT" }
                        {layer_rows}
                    }
                    object { "data": timeline_attr }
                }
            }

            div { id: "status", "{loaded.status}" }
        }
    )
}
