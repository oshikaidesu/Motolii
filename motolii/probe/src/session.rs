use std::sync::{Arc, Mutex};

use motolii_store::Document;

use crate::playback::Clock;
use crate::tokens::UiScale;

pub struct Session {
    pub doc: Arc<Mutex<Document>>,
    pub clock: Arc<Clock>,
    pub scale: Arc<UiScale>,
}

impl Session {
    pub fn new(doc: Document, duration_sec: f64) -> Self {
        Self {
            doc: Arc::new(Mutex::new(doc)),
            clock: Arc::new(Clock::new(duration_sec)),
            scale: Arc::new(UiScale::new(100)),
        }
    }
}
