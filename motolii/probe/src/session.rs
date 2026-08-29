use std::sync::{Arc, Mutex};

use motolii_store::{Document, LayerId};

use crate::playback::Clock;
use crate::tokens::UiScale;

/// 層選択の唯一の真実。chrome(Signal)とcustom paint(paint毎読み)の両方がここを読む。
/// chrome側の再描画はSignalのミラーが担う — ミラーを正にしない。
///
/// 並びの**末尾が主選択**。`get`は主選択だけを返すので、単数しか要らない呼び手は
/// 複数選択を意識しない。
#[derive(Clone, Default)]
pub struct Selection(Arc<Mutex<Vec<LayerId>>>);

impl Selection {
    /// 主選択(並びの末尾)。
    pub fn get(&self) -> Option<LayerId> {
        self.0.lock().unwrap().last().copied()
    }

    /// 単数で置き換える。
    pub fn set(&self, layer: Option<LayerId>) {
        let mut v = self.0.lock().unwrap();
        v.clear();
        if let Some(l) = layer {
            v.push(l);
        }
    }

    pub fn all(&self) -> Vec<LayerId> {
        self.0.lock().unwrap().clone()
    }

    pub fn contains(&self, layer: LayerId) -> bool {
        self.0.lock().unwrap().contains(&layer)
    }

    /// 既に居れば外し、居なければ主選択として足す(Cmd/Shiftクリックの意味)。
    pub fn toggle(&self, layer: LayerId) {
        let mut v = self.0.lock().unwrap();
        match v.iter().position(|l| *l == layer) {
            Some(i) => {
                v.remove(i);
            }
            None => v.push(layer),
        }
    }
}

pub struct Session {
    pub doc: Arc<Mutex<Document>>,
    pub clock: Arc<Clock>,
    pub scale: Arc<UiScale>,
    pub selection: Selection,
}

impl Session {
    pub fn new(doc: Document, duration_sec: f64) -> Self {
        Self {
            doc: Arc::new(Mutex::new(doc)),
            clock: Arc::new(Clock::new(duration_sec)),
            scale: Arc::new(UiScale::new(100)),
            selection: Selection::default(),
        }
    }
}

#[cfg(test)]
mod selection_invariants {
    use super::*;

    const A: LayerId = LayerId(1);
    const B: LayerId = LayerId(2);

    #[test]
    fn set_replaces_and_get_returns_it() {
        let s = Selection::default();
        s.set(Some(A));
        s.set(Some(B));
        assert_eq!(s.all(), vec![B]);
        assert_eq!(s.get(), Some(B));
    }

    #[test]
    fn set_none_empties() {
        let s = Selection::default();
        s.set(Some(A));
        s.set(None);
        assert!(s.all().is_empty());
        assert_eq!(s.get(), None);
    }

    #[test]
    fn toggle_twice_returns_to_start() {
        let s = Selection::default();
        s.set(Some(A));
        let before = s.all();
        s.toggle(B);
        s.toggle(B);
        assert_eq!(s.all(), before);
    }

    #[test]
    fn toggled_in_layer_becomes_primary() {
        let s = Selection::default();
        s.set(Some(A));
        s.toggle(B);
        assert_eq!(s.get(), Some(B));
        assert!(s.contains(A));
        assert_eq!(s.all().len(), 2);
    }

    #[test]
    fn toggling_out_the_primary_promotes_the_previous() {
        let s = Selection::default();
        s.set(Some(A));
        s.toggle(B);
        s.toggle(B);
        assert_eq!(s.get(), Some(A));
    }
}
