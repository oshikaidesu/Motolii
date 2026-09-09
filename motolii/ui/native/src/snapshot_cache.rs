use std::sync::atomic::{AtomicU64, Ordering};
use crate::{doc::store::Revision, EditorRuntime};
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
        let key = format!("{}:{}", self.image_key(), self.render_count);
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
        let before = new["snapshotId"].clone();
        rt.render_count += 1;
        let after = rt.status().unwrap();
        assert_ne!(after["snapshotId"],before);
    }
}
