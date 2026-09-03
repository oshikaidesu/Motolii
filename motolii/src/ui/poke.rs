use blitz_shell::{BlitzShellEvent, BlitzShellProxy};
use crate::ui::host::Woken;

/// 窓の外の糸が持つ、窓を起こすだけの口。
#[derive(Clone)]
pub(crate) struct Poke(pub(crate) Option<BlitzShellProxy>);

/// 起こす口はどれも同じ物(props の比較用)。
impl PartialEq for Poke {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Poke {
    pub(crate) fn poke(&self) {
        if let Some(proxy) = &self.0 {
            proxy.send_event(BlitzShellEvent::embedder_event(Woken));
        }
    }
}

pub(crate) fn wakes_shared_state(event: &BlitzShellEvent) -> bool {
    matches!(event, BlitzShellEvent::Embedder(value) if value.is::<Woken>())
}

#[cfg(test)]
#[test]
fn only_explicit_external_state_events_wake_every_window() {
    assert!(wakes_shared_state(&BlitzShellEvent::embedder_event(Woken)));
    assert!(!wakes_shared_state(&BlitzShellEvent::embedder_event(())));
}
