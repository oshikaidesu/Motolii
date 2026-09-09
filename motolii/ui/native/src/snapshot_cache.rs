use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use crate::doc::store::{LayerId, PropertyBase, RationalTime, ResolvedLayer, Revision, StoreView};
use crate::EditorRuntime;
use serde_json::{json, Value};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
const REFERENCES: &[&str] = &["backgrounds", "assets", "fontFamilies", "easeKinds", "catalog", "capabilities", "importExtensions", "visualSamples", "blendSamples"];

#[derive(Default)]
pub(crate) struct SnapshotCache {
    key: Option<String>,
    body: Value,
    id: u64,
    references: Value,
    reference_id: u64,
    signature: Option<(String, Revision, String)>,
    /// 層ごとの status 行と、その行を作った入力の指紋。
    pub(crate) rows: HashMap<LayerId, (u64, Value)>,
}

fn digest(parts: impl Hash) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    parts.hash(&mut hasher);
    hasher.finish()
}

/// 群れの枠は子の枠で決まる。子の指紋を親へ畳む。
fn fold(id: LayerId, own: &HashMap<LayerId, u64>, children: &HashMap<LayerId, Vec<LayerId>>, seen: &mut Vec<LayerId>) -> u64 {
    if seen.contains(&id) { return 0; }
    seen.push(id);
    let kids: Vec<u64> = children.get(&id).into_iter().flatten().map(|&kid| fold(kid, own, children, seen)).collect();
    seen.pop();
    digest((own.get(&id).copied().unwrap_or(0), kids))
}

impl EditorRuntime {
    /// 層ごとの「入力の指紋」。同じ指紋なら `layer_json` の出す行も同じ。
    /// 覆うのは行が読む全て — 属性・meta・各属性の出所と *その時刻での評価値*(transient と
    /// preview 込み)・文字と解決済みの書式・図形・効果・マスク・切り抜きの基・解決済みの姿、
    /// そして子の指紋。値を時刻で評価して入れるので、時刻そのものは鍵に要らない。
    /// 例外は ◇ の点灯(`keyedNow`)—— キーを持つ層だけ frame も鍵に混ぜる。
    /// 観測者(カメラ・向き)は行に入らない。枠と x/y は `overlay_geometry` が毎回載せ直す。
    pub(crate) fn layer_keys(&self, view: &StoreView<'_>, at: RationalTime, resolved: &[ResolvedLayer], clipping: &HashMap<LayerId, Option<LayerId>>, live: bool) -> Result<HashMap<LayerId, u64>, String> {
        let e = |error: &dyn std::fmt::Display| error.to_string();
        let comp = view.composition().map_err(|x| e(&x))?;
        let global = digest((self.doc.identity(), live, format!("{comp:?}")));
        let ids = view.layers();
        let mut own = HashMap::new();
        let mut children: HashMap<LayerId, Vec<LayerId>> = HashMap::new();
        for &id in &ids {
            let mut properties = Vec::new();
            let mut keyed = false;
            for p in view.properties(id) {
                let source = view.property_source(id, &p).map_err(|x| e(&x))?;
                keyed |= source.as_ref().is_some_and(|s| matches!(&s.base, Some(PropertyBase::Track(track)) if !track.keys().is_empty()));
                properties.push(json!([
                    p.name(), source,
                    view.value_at(id, &p, at).map_err(|x| e(&x))?.as_ref().map(crate::snapshot::value),
                ]));
            }
            let attrs = view.attrs(id).map_err(|x| e(&x))?;
            if let Some(parent) = attrs.as_ref().and_then(|a| a.parent) { children.entry(parent).or_default().push(id); }
            let text = view.text_document(id).map_err(|x| e(&x))?;
            keyed |= text.as_ref().is_some_and(|t| !t.content.keys().is_empty());
            let resolved_text = match &text { Some(_) => view.resolved_text_document(id, at).map_err(|x| e(&x))?, None => None };
            let row = json!([
                id.0, attrs, view.meta(id).map_err(|x| e(&x))?, properties,
                text.as_ref().map(|t| t.content.eval(at)), text, resolved_text,
                view.shapes(id).map_err(|x| e(&x))?, view.effects(id).map_err(|x| e(&x))?, view.masks(id).map_err(|x| e(&x))?,
                clipping.get(&id).copied().flatten().map(|b| b.0), crate::editor::timeline_edit::ghostable(view, id),
                keyed.then_some(self.frame),
            ]);
            own.insert(id, digest((global, row.to_string())));
        }
        // 解決済みの姿(world 変換・配置効果の複製)も枠の入力。
        for layer in resolved {
            if let Some(key) = own.get_mut(&layer.id) { *key = digest((*key, format!("{layer:?}"))); }
        }
        Ok(ids.iter().map(|&id| (id, fold(id, &own, &children, &mut Vec::new()))).collect())
    }
}

impl EditorRuntime {
    fn content_key(&self) -> String {
        format!("{}:{:?}:{}:{:?}", self.doc.identity(), self.doc.display_revision(), self.user_stage, self.user_camera)
    }

    pub(crate) fn image_key(&self) -> String { format!("{}:{}", self.content_key(), self.frame) }

    pub(crate) fn is_dirty(&self) -> Result<bool, String> {
        let revision = self.doc.revision();
        let identity = self.doc.identity();
        if let Some((id, r, signature)) = &self.snapshot_cache.borrow().signature {
            if *id == identity && *r == revision { return Ok(signature != &self.saved_signature); }
        }
        let signature = crate::snapshot::authored_signature(&self.doc)?;
        let dirty = signature != self.saved_signature;
        self.snapshot_cache.borrow_mut().signature = Some((identity, revision, signature));
        Ok(dirty)
    }

    pub(crate) fn status(&self) -> Result<Value, String> { self.status_response(None, None) }

    pub(crate) fn status_response(&self, known: Option<u64>, known_references: Option<u64>) -> Result<Value, String> {
        // render は status の中身を変えない —— 変わるのは reply へ後から載せる renderCount/renderMs だけ。
        // 音の健康と波形は Document 版と無関係に動くので、鍵に入れて古い body を残さない。
        let key = format!("{}:{:?}:{}", self.image_key(), self.clock.health(), self.clock.waveform_tracks().len());
        let playing = self.clock.playing();
        if playing && known.is_some() && known != Some(self.snapshot_cache.borrow().id) {
            *self.full_status_revision.borrow_mut() = None;
        }
        if playing || self.snapshot_cache.borrow().key.as_ref() != Some(&key) {
            let mut built = self.build_status()?;
            if built.get("liveLayers").is_some() {
                built["contentRevision"] = json!(self.content_key());
                return Ok(built);
            }
            let mut references = json!({});
            for name in REFERENCES {
                if let Some(v) = built.as_object_mut().unwrap().remove(*name) { references[*name] = v; }
            }
            let mut cache = self.snapshot_cache.borrow_mut();
            // A full snapshot produced while playing can omit unchanged reference fields.
            let mut merged = cache.references.as_object().cloned().unwrap_or_default();
            merged.extend(references.as_object().unwrap().clone());
            let references = Value::Object(merged);
            if cache.reference_id == 0 || cache.references != references {
                cache.reference_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                cache.references = references;
            }
            cache.id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            cache.key = (!playing).then_some(key);
            cache.body = built;
        }
        let cache = self.snapshot_cache.borrow();
        let mut reply = if known == Some(cache.id) { json!({}) } else { cache.body.clone() };
        if known_references != Some(cache.reference_id) {
            reply.as_object_mut().unwrap().extend(cache.references.as_object().cloned().unwrap_or_default());
        }
        let selected = cache.body["layers"].as_array().and_then(|layers| layers.iter().find(|l| l["id"].as_u64() == self.selected.map(|id|id.0)));
        reply["selectedBounds"] = selected.map(|l| l["bounds"].clone()).unwrap_or(Value::Null);
        for axis in ["x", "y"] { reply[axis] = selected.map(|l|l[axis].clone()).unwrap_or(json!(0.0)); }
        for field in ["width", "height", "fps", "durationFrames", "documentRevision", "deviceId"] { reply[field] = cache.body[field].clone(); }
        reply["contentRevision"] = json!(self.content_key());
        reply["snapshotId"] = json!(cache.id);
        reply["referenceId"] = json!(cache.reference_id);
        drop(cache);
        let (undo, redo) = self.doc.history_depth();
        reply["selectedId"] = json!(self.selected.map(|id|id.0));
        reply["selectedIds"] = json!(self.selected_ids.iter().map(|id|id.0).collect::<Vec<_>>());
        let fps = self.doc.view().composition().map_err(|e|e.to_string())?.ok_or("No composition")?.fps.as_f64();
        reply["selectedKeys"] = json!(self.selected_keys.iter().map(|k|json!({"layer":k.layer.0,"property":k.property.as_ref().map(|p|p.name()),"frame":(k.at_sec*fps).round()as i64})).collect::<Vec<_>>());
        reply["colorTarget"] = self.color_target.as_ref().and_then(|slot| crate::editor::color::read_color(&self.doc,slot).map(|rgba|json!({"layer":slot.layer().0,"slot":slot,"label":"Color","rgba":rgba}))).unwrap_or(Value::Null);
        reply["undo"] = json!(undo); reply["redo"] = json!(redo);
        reply["path"] = json!(self.path); reply["dirty"] = json!(self.is_dirty()?);
        reply["frame"] = json!(self.frame); reply["playing"] = json!(playing);
        reply["animate"] = json!(self.animate); reply["error"] = json!(self.error);
        reply["preview"] = json!(self.preview.is_some()); reply["previewOwner"] = json!(self.preview.as_ref().map(|p|p.0));
        reply["renderCount"] = json!(self.render_count); reply["renderMs"] = json!(self.render_ms);
        reply["pickedColor"] = json!(self.picked_color); reply["pickSerial"] = json!(self.pick_serial);
        reply["export"] = self.exporter.status();
        Ok(reply)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::*;
    use std::ffi::{CStr, CString};
    use serde_json::Value;

    fn request(rt: &mut EditorRuntime, command: Value) -> Value {
        let command = CString::new(command.to_string()).unwrap();
        let reply = unsafe { crate::motolii_probe_request(rt, command.as_ptr()) };
        let reply: Value = serde_json::from_str(unsafe { CStr::from_ptr(reply) }.to_str().unwrap()).unwrap();
        assert!(reply["error"].is_null(), "{reply}");
        reply
    }

    /// 1 層だけ動いたら組み直すのも 1 行だけ。cache が黙って全組み直しへ戻る事故を止める。
    #[test]
    fn one_layer_moving_reprints_one_row() {
        let mut rt = EditorRuntime::open("").unwrap();
        let fps = rt.doc.view().composition().unwrap().unwrap().fps;
        for i in 0..8u64 {
            rt.doc.apply_all(crate::editor::create::new_layer_intents(LayerId(100 + i), i as i16, 0, 600, fps, (1920.0, 1080.0), crate::editor::create::NewKind::Rectangle, None)).unwrap();
        }
        let at = rt.time().unwrap();
        let keys = |rt: &EditorRuntime| {
            let view = rt.doc.view();
            let resolved = view.resolved_layers(at).unwrap();
            let clipping = view.clipping_bases().unwrap();
            rt.layer_keys(&view, at, &resolved, &clipping, false).unwrap()
        };
        let moved = |before: &HashMap<LayerId, u64>, after: &HashMap<LayerId, u64>| {
            let mut ids: Vec<u64> = after.iter().filter(|(id, key)| before.get(id) != Some(key)).map(|(id, _)| id.0).collect();
            ids.sort();
            ids
        };
        request(&mut rt, json!({"op":"select","ids":[100]}));
        request(&mut rt, json!({"op":"previewProperties","edits":[{"layer":100,"property":"opacity","value":0.5}]}));
        let before = keys(&rt);
        request(&mut rt, json!({"op":"previewProperties","edits":[{"layer":100,"property":"opacity","value":0.6}]}));
        assert_eq!(moved(&before, &keys(&rt)), vec![100], "掴んでいる層だけが動く");
        request(&mut rt, json!({"op":"cancelPreview"}));
        // キーの無い層は時刻が進んでも行が変わらない。
        let before = keys(&rt);
        request(&mut rt, json!({"op":"seek","frame":5}));
        assert_eq!(moved(&before, &keys(&rt)), Vec::<u64>::new(), "キーが無ければ時刻で行は動かない");
        // キーを持たせれば、その層だけが時刻で動く。
        request(&mut rt, json!({"op":"toggleKey","layer":100,"property":"opacity"}));
        let before = keys(&rt);
        request(&mut rt, json!({"op":"seek","frame":9}));
        assert_eq!(moved(&before, &keys(&rt)), vec![100], "キーを持つ層だけが時刻で動く");
        // 観測者は行に入らない —— 枠は毎回載せ直すので、回しても組み直しは起きない。
        let before = keys(&rt);
        rt.user_camera.orbit_degrees = [30.0, 12.0];
        assert_eq!(moved(&before, &keys(&rt)), Vec::<u64>::new(), "台を回しても行は組み直さない");
    }

    /// 層ごとの cache は、同じ編集列を cache 無しで組んだ status と 1 行も違ってはならない。
    #[test]
    fn cached_layer_rows_match_rows_built_without_the_cache() {
        let mut rt = EditorRuntime::open("").unwrap();
        let mut step = |rt: &mut EditorRuntime, label: &str, command: Value| {
            if !command.is_null() {
                let reply = request(rt, command.clone());
                assert!(reply["error"].is_null(), "{label}: {reply}");
            }
            let cached = rt.build_status().unwrap();
            rt.snapshot_cache.borrow_mut().rows.clear();
            let fresh = rt.build_status().unwrap();
            assert_eq!(cached["layers"], fresh["layers"], "{label}");
        };
        step(&mut rt, "text layer", json!({"op":"create","kind":"text"}));
        let text = rt.doc.view().layers()[0].0;
        step(&mut rt, "rectangle", json!({"op":"create","kind":"rectangle"}));
        step(&mut rt, "second rectangle", json!({"op":"create","kind":"rectangle"}));
        let ids: Vec<u64> = rt.doc.view().layers().iter().map(|l| l.0).collect();
        let (a, b) = (ids[ids.len() - 2], ids[ids.len() - 1]);
        step(&mut rt, "move one layer", json!({"op":"select","ids":[a]}));
        step(&mut rt, "position", json!({"op":"setProperty","layer":a,"property":"position","value":[300.0,200.0]}));
        step(&mut rt, "preview position", json!({"op":"previewProperties","edits":[{"layer":a,"property":"position","value":[420.0,90.0]}]}));
        step(&mut rt, "commit preview", json!({"op":"commitPreview"}));
        step(&mut rt, "rename", json!({"op":"setAttrs","layers":[a],"patch":{"name":"Renamed"}}));
        step(&mut rt, "hide", json!({"op":"setAttrs","layers":[a],"patch":{"hidden":true}}));
        step(&mut rt, "keyframe", json!({"op":"toggleKey","layer":a,"property":"opacity"}));
        step(&mut rt, "seek", json!({"op":"seek","frame":7}));
        step(&mut rt, "text content key at 7", json!({"op":"toggleKey","layer":text,"property":"content"}));
        step(&mut rt, "type at 7", json!({"op":"setText","layer":text,"content":"Later"}));
        step(&mut rt, "seek to 3", json!({"op":"seek","frame":3}));
        step(&mut rt, "seek to 9", json!({"op":"seek","frame":9}));
        step(&mut rt, "seek back to 7", json!({"op":"seek","frame":7}));
        step(&mut rt, "opacity at 7", json!({"op":"setProperty","layer":a,"property":"opacity","value":0.25}));
        step(&mut rt, "back to 0", json!({"op":"seek","frame":0}));
        let plugin = crate::render::engine::known_effects()[0].plugin_id.clone();
        step(&mut rt, "effect", json!({"op":"applyEffect","pluginId":plugin}));
        step(&mut rt, "text content", json!({"op":"setText","layer":text,"content":"Hello"}));
        step(&mut rt, "colour", json!({"op":"select","ids":[a]}));
        step(&mut rt, "palette", json!({"op":"applyPalette","rgba":[0.2,0.8,0.4,1.0]}));
        step(&mut rt, "group the pair", json!({"op":"select","ids":[a,b]}));
        step(&mut rt, "group", json!({"op":"group"}));
        let group = *rt.doc.view().layers().iter().find(|l| rt.doc.view().meta(**l).unwrap().is_some_and(|m| m.source == LayerSource::Group)).expect("group");
        // 子が動けば群れの枠も動く —— 親の行が古いままなら、ここで露見する。
        step(&mut rt, "child moves inside the group", json!({"op":"setProperty","layer":a,"property":"position","value":[900.0,700.0]}));
        step(&mut rt, "group moves", json!({"op":"setProperty","layer":group.0,"property":"position","value":[120.0,140.0]}));
        step(&mut rt, "child scale", json!({"op":"setProperty","layer":b,"property":"scale","value":[2.5,2.5]}));
        step(&mut rt, "ungroup", json!({"op":"select","ids":[group.0]}));
        step(&mut rt, "ungroup now", json!({"op":"ungroup"}));
        step(&mut rt, "undo", json!({"op":"undo"}));
        step(&mut rt, "redo", json!({"op":"redo"}));
        step(&mut rt, "reorder", json!({"op":"select","ids":[a]}));
        step(&mut rt, "reorder now", json!({"op":"reorder","delta":1}));
        step(&mut rt, "composition", json!({"op":"composition","width":960,"height":540}));
        step(&mut rt, "delete", json!({"op":"delete"}));
        rt.user_stage = false;
        step(&mut rt, "camera view", Value::Null);
        rt.user_stage = true;
        rt.user_camera.orbit_degrees = [22.0, 8.0];
        step(&mut rt, "orbited stage", Value::Null);
    }

    #[test]
    fn selection_delta_preview_cancel_and_bootstrap_preserve_the_contract() {
        let mut rt = EditorRuntime::open("").unwrap();
        let layer = LayerId(101);
        rt.doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 180) } },
            Intent::SetShapes { layer, shapes: vec![rect_shape([255,0,0,255],[40.0,40.0])] },
        ]).unwrap();
        let full = request(&mut rt, json!({"op":"status","bootstrap":true}));
        let known = full["snapshotId"].clone(); let references = full["referenceId"].clone();
        assert!(full["backgrounds"].is_array());
        let delta = request(&mut rt, json!({"op":"select","ids":[101],"knownSnapshotId":known,"knownReferenceId":references,"deferSnapshot":true}));
        assert_eq!(delta["needsRender"], false);
        assert_eq!(delta["selectedId"], 101);
        assert_eq!(delta["snapshotId"], known);
        assert!(delta.get("layers").is_none());
        assert!(delta.get("backgrounds").is_none());
        assert_eq!(delta["renderCount"], full["renderCount"]);
        let invalid = request(&mut rt, json!({"op":"status","knownSnapshotId":0,"knownReferenceId":0}));
        assert!(invalid["layers"].is_array()); assert!(invalid["backgrounds"].is_array());
        let preview = request(&mut rt, json!({"op":"previewProperties","edits":[{"layer":101,"property":"opacity","value":0.5}],"deferSnapshot":true}));
        assert_eq!(preview,json!({"needsRender":true}));
        let preview_status = rt.status().unwrap();
        assert_ne!(preview_status["snapshotId"], known);
        let cancel = request(&mut rt, json!({"op":"select","ids":[101],"deferSnapshot":true}));
        assert_eq!(cancel,json!({"needsRender":true}));
        assert!(rt.preview.is_none());
        let restored = rt.status().unwrap();
        let row = |v: &Value| v["layers"].as_array().unwrap().iter().find(|l|l["id"]==101).unwrap()["properties"].as_array().unwrap().iter().find(|p|p["id"]=="opacity").unwrap()["value"].clone();
        assert_eq!(row(&restored),row(&full));
        let moved = request(&mut rt,json!({"op":"seek","frame":8,"deferSnapshot":true}));
        assert_eq!(moved,json!({"needsRender":true}));
        let view = request(&mut rt,json!({"op":"stageView","mode":"Camera","deferSnapshot":true}));
        assert_eq!(view,json!({"needsRender":true}));
        request(&mut rt,json!({"op":"play"}));
        let joined = request(&mut rt,json!({"op":"status","bootstrap":true}));
        assert!(joined["layers"].is_array()); assert!(joined["backgrounds"].is_array());
        assert_eq!(joined["frame"],8);
    }

    #[test]
    fn saved_state_is_not_cached_across_save_undo_or_replacement() {
        let mut rt = EditorRuntime::open("").unwrap();
        assert!(!rt.is_dirty().unwrap());
        let mut comp = rt.doc.view().composition().unwrap().unwrap();
        comp.width += 1;
        rt.doc.apply(Intent::SetComposition(comp)).unwrap();
        assert!(rt.is_dirty().unwrap());
        rt.saved_signature = crate::snapshot::authored_signature(&rt.doc).unwrap();
        assert!(!rt.is_dirty().unwrap());
        assert!(rt.doc.undo()); assert!(rt.is_dirty().unwrap());
        assert!(rt.doc.redo()); assert!(!rt.is_dirty().unwrap());
        let old = rt.status().unwrap();
        let new = request(&mut rt,json!({"op":"new","knownSnapshotId":old["snapshotId"],"knownReferenceId":old["referenceId"]}));
        assert_ne!(new["snapshotId"],old["snapshotId"]);
        assert_eq!(new["dirty"],false);
        assert_eq!(new["layers"],json!([]));
        // render は文書を変えない。snapshot は据え置き、届くのは数だけ。
        let before = new["snapshotId"].clone();
        rt.render_count += 1;
        let after = rt.status_response(before.as_u64(), None).unwrap();
        assert_eq!(after["snapshotId"],before);
        assert_eq!(after["renderCount"],1);
        assert!(after.get("layers").is_none(),"{after}");
    }
}
