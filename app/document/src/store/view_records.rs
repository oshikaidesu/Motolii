//! 書類ぜんたいを訊く口。**1 コマに何度訊かれても、store を引くのは 1 回**。
//!
//! ここは層ごとの読み(`meta` / `attrs`)と違い、呼ぶ側が「ついでに」何度でも訊く口で、
//! 文字の道だけでも 1 コマに数百回来る。素で引くと `latest_at` + JSON parse + entity path の
//! 整形が積み上がり、2026-09-21 の標本では再生の render thread の約 3 割がこれだった。
//! 手控えは既にある `RecordCache`(版で捨てる)へ、時刻も鍵にして載せる。

use super::*;

impl<'a> StoreView<'a> {
    pub fn layers(&self) -> Vec<LayerId> {
        {
            let mut cache = self.record_cache.borrow_mut();
            cache.sync(&self.revision);
            if let Some((at, hit)) = &cache.layers {
                if *at == self.at {
                    return hit.as_ref().clone();
                }
            }
        }
        let out = self.layers_uncached();
        self.record_cache.borrow_mut().layers = Some((self.at, std::sync::Arc::new(out.clone())));
        out
    }

    fn layers_uncached(&self) -> Vec<LayerId> {
        let query = self.query();
        let mut out: Vec<LayerId> = self
            .db
            .sorted_entity_paths()
            .filter_map(|path| {
                let id = layer_id_of(path)?;
                let results = self
                    .db
                    .latest_at(&query, path, [descriptor_present().component]);
                let present = results
                    .component_batch::<LayerPresent>(descriptor_present().component)?
                    .first()
                    .copied()?;
                present.0.then_some(id)
            })
            .collect();
        out.sort();
        out
    }

    pub fn composition(&self) -> Result<Option<Composition>, StoreError> {
        {
            let mut cache = self.record_cache.borrow_mut();
            cache.sync(&self.revision);
            if let Some((at, hit)) = &cache.composition {
                if *at == self.at {
                    return Ok(hit.clone());
                }
            }
        }
        let value = self.composition_uncached()?;
        self.record_cache.borrow_mut().composition = Some((self.at, value.clone()));
        Ok(value)
    }

    fn composition_uncached(&self) -> Result<Option<Composition>, StoreError> {
        let descriptor = descriptor_composition();
        let path = composition_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(None);
        };
        serde_json::from_str(&json.0)
            .map(Some)
            .map_err(StoreError::Encode)
    }
}

/// 層の entity path(`/layer/<番号>`)から番号を読む。**文字列に整形しない** —
/// 書類の path を全部舐めるので、`to_string()` は 1 コマに数千回の format + 確保になる
/// (2026-09-21 の標本で `EntityPath as Display::fmt` が 439 標本)。
pub(super) fn layer_id_of(path: &EntityPath) -> Option<LayerId> {
    let [head, id] = path.as_slice() else { return None };
    (head.unescaped_str() == "layer")
        .then(|| id.unescaped_str().parse::<u64>().ok())
        .flatten()
        .map(LayerId)
}
