use std::sync::{Arc, Mutex};

use motolii_store::{Document, LayerId};

use crate::playback::Clock;
use crate::tokens::UiScale;

/// 層選択の唯一の真実。chrome(Signal)とcustom paint(paint毎読み)の両方がここを読む。
/// chrome側の再描画はSignalのミラーが担う — ミラーを正にしない。
#[derive(Clone, Default)]
pub struct Selection(Arc<Mutex<Option<LayerId>>>);

impl Selection {
    pub fn get(&self) -> Option<LayerId> {
        *self.0.lock().unwrap()
    }

    pub fn set(&self, layer: Option<LayerId>) {
        *self.0.lock().unwrap() = layer;
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
