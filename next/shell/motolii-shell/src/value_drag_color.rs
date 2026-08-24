//! Text style色のdrag意味。
//!
//! 色の投影・draft・commitだけを所有する。gestureの寿命や他の値域は扱わない。

use crate::value_drag::ValueDragTarget;
use crate::{inspector_pane, Shell};

impl Shell {
    pub(crate) fn color_value_drag_start_value(&self, target: ValueDragTarget) -> Option<f64> {
        let ValueDragTarget::Color(color_target, channel) = target else {
            return None;
        };
        let layer = self.session.selection?;
        let current = self.doc.view().text_document(layer).ok()?;
        let document = current.unwrap_or_else(inspector_pane::default_text_document);
        let style = document
            .styles
            .first()
            .cloned()
            .unwrap_or_else(inspector_pane::default_text_style);
        let rgba = inspector_pane::color::text_style_color(&style, color_target);
        Some(rgba[channel.index()] * 255.0)
    }

    pub(crate) fn write_color_value_drag_draft(&mut self, target: ValueDragTarget, raw: f64) {
        let ValueDragTarget::Color(color_target, channel) = target else {
            return;
        };
        let Some(layer) = self.session.selection else {
            return;
        };
        let Ok(current) = self.doc.view().text_document(layer) else {
            return;
        };
        let document = current.unwrap_or_else(inspector_pane::default_text_document);
        let mut style = document
            .styles
            .first()
            .cloned()
            .unwrap_or_else(inspector_pane::default_text_style);
        let clamped = raw.clamp(0.0, 255.0);
        inspector_pane::color::set_text_style_color_channel(
            &mut style,
            color_target,
            channel,
            clamped / 255.0,
        );
        let text = inspector_pane::color::color_channel_display(&style, color_target, channel);
        self.inspector_color_field_draft = Some(inspector_pane::color::ColorFieldDraft {
            target: color_target,
            channel,
            text,
        });
    }

    pub(crate) fn finish_color_value_drag(&mut self, target: ValueDragTarget) {
        let ValueDragTarget::Color(color_target, channel) = target else {
            return;
        };
        if let Err(error) = inspector_pane::color::commit_text_style_color(
            &mut self.doc,
            &mut self.inspector_color_field_draft,
            self.session.selection,
            color_target,
            channel,
        ) {
            self.status = Some(error);
        }
    }
}
