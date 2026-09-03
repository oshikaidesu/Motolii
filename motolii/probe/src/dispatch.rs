//! 意図([`Intent`])の実行口。打鍵(`app.rs` の keydown)と右クリック menu
//! (`context_menu.rs`)が**同じ関数**を通る — 入口が2つあっても書き込み経路は1つ。

use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use dioxus_native::prelude::{ReadableExt, Signal, WritableExt};
use motolii_store::{Document, Intent as StoreIntent, LayerId};

use crate::fixture::{self, LayerRow};
use crate::keymap::Intent;
use crate::playback::Clock;
use crate::session::Selection;
use crate::timeline_widget::{attrs_to_patch, split_layer, TimelineMsg};

const DOC_FPS: f64 = 30.0;

/// 意図を撃つのに要る手元の物。全部 `Clone` が安い(Arc か Copy な Signal)。
#[derive(Clone)]
pub struct IntentCtx {
    pub doc: Arc<Mutex<Document>>,
    pub clock: Arc<Clock>,
    pub selection: Selection,
    pub timeline_tx: Sender<TimelineMsg>,
    pub layer_rows: Signal<Vec<LayerRow>>,
    pub attrs_state: Signal<Vec<(bool, bool, bool)>>,
    pub revision: Signal<u32>,
    pub selected: Signal<Option<LayerId>>,
    pub playing: Signal<bool>,
}

impl IntentCtx {
    fn comp_frame(&self) -> i64 {
        (self.clock.now_sec() * DOC_FPS).round() as i64
    }

    /// 層の増減の後に chrome(層列・M/S/L)と Timeline 帯を Document から読み直す。
    fn refresh_rows(&self) {
        let snapshot = self.doc.lock().unwrap();
        let rows = fixture::layer_rows_from_doc(&snapshot);
        let canvas = fixture::canvas_rows_from_doc(&snapshot);
        drop(snapshot);
        let mut attrs_state = self.attrs_state;
        let mut layer_rows = self.layer_rows;
        let mut revision = self.revision;
        attrs_state.set(rows.iter().map(|r| (r.hidden, r.solo, r.locked)).collect());
        layer_rows.set(rows);
        let _ = self.timeline_tx.send(TimelineMsg::SetRows(canvas));
        *revision.write() += 1;
    }

    fn set_selection(&self, layer: Option<LayerId>) {
        let mut selected = self.selected;
        self.selection.set(layer);
        selected.set(self.selection.get());
    }
}

/// 層を丸ごと写す。尺・属性・エフェクト・全 property track を同じ値で新しい id へ書く。
/// 重ね順も同じなので、写しは元と同じ位置に**重なって**現れる(Cmd+D の一般文法)。
pub fn duplicate_layer(doc: &Arc<Mutex<Document>>, layer: LayerId) -> Option<LayerId> {
    let mut doc = doc.lock().unwrap();
    let view = doc.view();
    let meta = view.meta(layer).ok().flatten()?;
    let copy = LayerId(view.next_layer_id());
    let attrs = view.attrs(layer).ok().flatten().unwrap_or_default();
    let effects = view.effects(layer).unwrap_or_default();
    let tracks: Vec<_> = view
        .properties(layer)
        .into_iter()
        .filter_map(|p| view.track(layer, &p).ok().flatten().map(|t| (p, t)))
        .collect();

    let mut intents = vec![
        StoreIntent::AddLayer(copy),
        StoreIntent::SetMeta { layer: copy, meta },
        StoreIntent::SetAttrs { layer: copy, patch: attrs_to_patch(&attrs) },
    ];
    if !effects.is_empty() {
        intents.push(StoreIntent::SetEffects { layer: copy, effects });
    }
    for (property, track) in tracks {
        intents.push(StoreIntent::SetTrack { layer: copy, property, track });
    }
    doc.apply_all(intents).ok()?;
    Some(copy)
}

pub fn run_intent(ctx: &IntentCtx, intent: Intent) {
    match intent {
        Intent::Split => {
            let Some(layer) = ctx.selection.get() else {
                println!("PROBE room=write verdict=split-noop reason=no-selection");
                return;
            };
            let comp_frame = ctx.comp_frame();
            match split_layer(&ctx.doc, layer, comp_frame) {
                Some(tail) => {
                    ctx.refresh_rows();
                    println!(
                        "PROBE room=write verdict=applied Split layer={layer:?} tail={tail:?} comp_frame={comp_frame}"
                    );
                }
                None => println!("PROBE room=write verdict=split-noop layer={layer:?} comp_frame={comp_frame}"),
            }
        }
        Intent::Duplicate => {
            let layers = ctx.selection.all();
            if layers.is_empty() {
                println!("PROBE room=write verdict=duplicate-noop reason=no-selection");
                return;
            }
            let mut last = None;
            for layer in layers {
                match duplicate_layer(&ctx.doc, layer) {
                    Some(copy) => {
                        last = Some(copy);
                        println!("PROBE room=write verdict=applied Duplicate layer={layer:?} copy={copy:?}");
                    }
                    None => println!("PROBE room=write verdict=duplicate-noop layer={layer:?}"),
                }
            }
            if last.is_some() {
                ctx.set_selection(last);
                ctx.refresh_rows();
            }
        }
        Intent::Delete => {
            let layers = ctx.selection.all();
            if layers.is_empty() {
                println!("PROBE room=write verdict=delete-noop reason=no-selection");
                return;
            }
            let mut removed = 0;
            for layer in layers {
                let mut doc = ctx.doc.lock().unwrap();
                match doc.apply(StoreIntent::RemoveLayer(layer)) {
                    Ok(_) => {
                        removed += 1;
                        println!("PROBE room=write verdict=applied RemoveLayer layer={layer:?}");
                    }
                    Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                }
            }
            if removed > 0 {
                ctx.set_selection(None);
                ctx.refresh_rows();
            }
        }
        Intent::StepFrame(delta) => {
            let frame = ctx.comp_frame() + delta;
            ctx.clock.seek(frame as f64 / DOC_FPS);
        }
        Intent::Home => ctx.clock.seek(0.0),
        Intent::End => ctx.clock.seek(ctx.clock.duration),
        Intent::Deselect => ctx.set_selection(None),
        Intent::PlayPause => {
            let mut playing = ctx.playing;
            ctx.clock.toggle();
            playing.set(ctx.clock.playing());
        }
        Intent::SelectAll => {
            let mut selected = ctx.selected;
            let mut revision = ctx.revision;
            let layers = ctx.doc.lock().unwrap().view().layers();
            for layer in layers {
                if !ctx.selection.contains(layer) {
                    ctx.selection.toggle(layer);
                }
            }
            selected.set(ctx.selection.get());
            *revision.write() += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_store::LayerId;

    fn doc_with_layer() -> (Arc<Mutex<Document>>, LayerId) {
        let fx = motolii_fixture::build();
        let doc = fx.doc;
        let layer = doc.view().layers().into_iter().next().expect("fixture has layers");
        (Arc::new(Mutex::new(doc)), layer)
    }

    /// 不変量: 写しは元と同じ尺・属性・エフェクト・track を持ち、元は変わらない。
    #[test]
    fn duplicate_copies_everything_and_leaves_the_original_alone() {
        let (doc, layer) = doc_with_layer();
        let before = {
            let d = doc.lock().unwrap();
            let v = d.view();
            (v.meta(layer).unwrap(), v.attrs(layer).unwrap(), v.effects(layer).unwrap(), v.layers().len())
        };
        let copy = duplicate_layer(&doc, layer).expect("duplicate");
        assert_ne!(copy, layer);
        let d = doc.lock().unwrap();
        let v = d.view();
        assert_eq!(v.layers().len(), before.3 + 1);
        assert_eq!(v.meta(layer).unwrap(), before.0);
        assert_eq!(v.attrs(layer).unwrap(), before.1);
        assert_eq!(v.effects(layer).unwrap(), before.2);
        assert_eq!(v.meta(copy).unwrap(), before.0);
        assert_eq!(v.attrs(copy).unwrap().unwrap_or_default(), before.1.unwrap_or_default());
        assert_eq!(v.effects(copy).unwrap(), before.2);
        for p in v.properties(layer) {
            assert_eq!(v.track(copy, &p).unwrap(), v.track(layer, &p).unwrap(), "track {p:?}");
        }
    }
}
