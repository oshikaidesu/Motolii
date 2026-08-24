//! Timeline markerの意味更新。Documentへのmarker Intentとdrag状態だけを持つ。

use motolii_store::Intent;

use crate::{timeline, Shell};

impl Shell {
    /// `Message::Marker` の畳み。**canvas 差し替え・input 優先順位・実際の
    /// mouse capture(`MarkerMessage::Grabbed`/`DragMoved`/`DragReleased`/
    /// `DragCancelled` を publish する側)は未結線**(`motolii-timeline-pane`
    /// の `canvas.rs`/`input.rs` が `pub(crate)` のため、EXACT TARGET
    /// 「pane crate は読み専用」の範囲で shell からは触れない — RETURN の
    /// API 要求参照)。この関数は Document 書き込みの意味だけを完結させる
    /// (keymap M=AddAtPlayhead は実際に届く経路、他は将来 canvas 側が
    /// publish するようになった時にそのまま機能する形で用意してある)。
    pub(crate) fn update_marker(&mut self, message: timeline::markers::MarkerMessage) {
        use timeline::markers::MarkerMessage;
        match message {
            MarkerMessage::AddAtPlayhead => {
                let Some(fps) = self.composition().map(|c| c.fps) else {
                    return;
                };
                let markers = self.markers();
                if let Some(next) =
                    timeline::markers::added_at_playhead(&markers, self.session.playhead, fps)
                {
                    if let Err(error) = self.doc.apply(Intent::SetMarkers { markers: next }) {
                        self.status = Some(format!("マーカーを置けない: {error}"));
                    }
                }
            }
            // S2 発注 #22 の2入口目(ルーラ locator lane 右クリック)。
            // `AddAtPlayhead` と同じ意味・同じ Intent、位置だけ呼び出し元
            // (`Message::AddMarkerAt(frame)`)が決める。
            MarkerMessage::AddAtFrame(frame) => {
                let Some(fps) = self.composition().map(|c| c.fps) else {
                    return;
                };
                let markers = self.markers();
                if let Some(next) = timeline::markers::added_at_frame(&markers, frame, fps) {
                    if let Err(error) = self.doc.apply(Intent::SetMarkers { markers: next }) {
                        self.status = Some(format!("マーカーを置けない: {error}"));
                    }
                }
            }
            // JumpTo は先取り(`ScrubTo`/`timeline_pane::Message::ScrubTo` と
            // 同じ経路 — playhead を直接書く、正典 §5「K/J ナビの補完」)。
            MarkerMessage::JumpTo(frame) => self.session.playhead = frame,
            MarkerMessage::Remove(index) => {
                let markers = self.markers();
                if let Some(next) = timeline::markers::removed(&markers, index) {
                    if let Err(error) = self.doc.apply(Intent::SetMarkers { markers: next }) {
                        self.status = Some(format!("マーカーを削除できない: {error}"));
                    }
                }
            }
            MarkerMessage::Grabbed { index, at_frame } => {
                let Some(fps) = self.composition().map(|c| c.fps) else {
                    return;
                };
                let markers = self.markers();
                self.marker_drag =
                    timeline::markers::MarkerDrag::start(&markers, index, at_frame, fps);
            }
            MarkerMessage::DragMoved { at_frame } => {
                let Some(fps) = self.composition().map(|c| c.fps) else {
                    return;
                };
                let duration = self.comp_duration();
                if let Some(drag) = self.marker_drag.as_mut() {
                    drag.dragged(at_frame, fps, duration);
                }
            }
            MarkerMessage::DragReleased => {
                if let Some(drag) = self.marker_drag.take() {
                    if let Some(next) = drag.finish() {
                        if let Err(error) = self.doc.apply(Intent::SetMarkers { markers: next }) {
                            self.status = Some(format!("マーカーを移動できない: {error}"));
                        }
                    }
                }
            }
            MarkerMessage::DragCancelled => {
                self.marker_drag = None;
            }
        }
    }
}
