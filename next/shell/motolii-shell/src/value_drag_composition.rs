//! Composition値のdrag意味。
//!
//! 共通gestureは [`crate::value_drag`] が所有し、このmoduleはCompositionの
//! 表示単位・下書き・確定だけを所有する。

use motolii_core::Fps;

use crate::value_drag::ValueDragTarget;
use crate::{settings_pane, Shell};

impl Shell {
    pub(crate) fn composition_value_drag_start_value(
        &self,
        target: ValueDragTarget,
    ) -> Option<f64> {
        let composition = self.doc.view().composition().ok().flatten()?;
        Some(match target {
            ValueDragTarget::CompWidth => f64::from(composition.width),
            ValueDragTarget::CompHeight => f64::from(composition.height),
            ValueDragTarget::CompFps => composition.fps.as_f64(),
            ValueDragTarget::CompDuration => composition.duration_frames as f64,
            _ => return None,
        })
    }

    pub(crate) fn write_composition_value_drag_draft(&mut self, target: ValueDragTarget, raw: f64) {
        use settings_pane::sections::{self, CompField};

        let Ok(Some(mut composition)) = self.doc.view().composition() else {
            return;
        };
        let field = match target {
            ValueDragTarget::CompWidth => CompField::Width,
            ValueDragTarget::CompHeight => CompField::Height,
            ValueDragTarget::CompFps => CompField::Fps,
            ValueDragTarget::CompDuration => CompField::DurationFrames,
            _ => return,
        };
        match field {
            // 下限1・上限 MAX_COMP_DIMENSION_PX(`parse_comp_dimension` と同じクランプ)。
            CompField::Width => {
                composition.width = raw
                    .round()
                    .clamp(1.0, f64::from(sections::MAX_COMP_DIMENSION_PX))
                    as u32;
            }
            CompField::Height => {
                composition.height = raw
                    .round()
                    .clamp(1.0, f64::from(sections::MAX_COMP_DIMENSION_PX))
                    as u32;
            }
            CompField::Fps => {
                let clamped = raw.clamp(1.0, sections::MAX_COMP_FPS);
                let per_mille = (clamped * 1000.0).round() as i64;
                if let Ok(fps) = Fps::try_new(per_mille, 1000) {
                    composition.fps = fps;
                }
            }
            CompField::DurationFrames => {
                composition.duration_frames = raw
                    .round()
                    .clamp(1.0, sections::MAX_COMP_DURATION_FRAMES as f64)
                    as i64;
            }
        }
        let text = sections::comp_field_display(field, &composition);
        self.comp_draft = Some(sections::CompFieldDraft { field, text });
    }

    pub(crate) fn finish_composition_value_drag(&mut self, target: ValueDragTarget) {
        let field = match target {
            ValueDragTarget::CompWidth => settings_pane::sections::CompField::Width,
            ValueDragTarget::CompHeight => settings_pane::sections::CompField::Height,
            ValueDragTarget::CompFps => settings_pane::sections::CompField::Fps,
            ValueDragTarget::CompDuration => settings_pane::sections::CompField::DurationFrames,
            _ => return,
        };
        if let Err(error) =
            settings_pane::sections::commit_comp_field(&mut self.doc, &mut self.comp_draft, field)
        {
            self.status = Some(error);
        }
    }
}
