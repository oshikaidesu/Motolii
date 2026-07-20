//! Motolii layout intentと一時runtime mutationの原子的な権限往復。

use crate::layout::{LayoutAction, LayoutConstraints, LayoutError, PanelLayout};
use crate::layout_runtime::RuntimeLayout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeFrameEdit {
    None,
    Continuous,
    Commit,
}

pub(crate) struct LayoutAuthority {
    intent: PanelLayout,
    runtime: RuntimeLayout,
    gesture_baseline: Option<PanelLayout>,
}

impl LayoutAuthority {
    pub(crate) fn built_in() -> Result<Self, LayoutError> {
        let intent = PanelLayout::built_in();
        let runtime = RuntimeLayout::project(&intent)?;
        Ok(Self {
            intent,
            runtime,
            gesture_baseline: None,
        })
    }

    pub(crate) fn intent(&self) -> &PanelLayout {
        &self.intent
    }

    pub(crate) fn runtime(&self) -> &RuntimeLayout {
        &self.runtime
    }

    pub(crate) fn runtime_mut(&mut self) -> &mut RuntimeLayout {
        &mut self.runtime
    }

    #[cfg(test)]
    pub(crate) fn replace_runtime_for_test(
        &mut self,
        proposal: PanelLayout,
    ) -> Result<(), LayoutError> {
        self.runtime = RuntimeLayout::project(&proposal)?;
        Ok(())
    }

    pub(crate) fn gesture_in_flight(&self) -> bool {
        self.gesture_baseline.is_some()
    }

    pub(crate) fn apply(
        &mut self,
        action: LayoutAction,
        constraints: LayoutConstraints,
    ) -> Result<(), LayoutError> {
        let mut candidate = self.intent.clone();
        candidate.apply(action, constraints)?;
        let runtime = RuntimeLayout::project(&candidate)?;
        self.intent = candidate;
        self.runtime = runtime;
        Ok(())
    }

    pub(crate) fn reconcile_runtime_frame(
        &mut self,
        cancelled: bool,
        edit: RuntimeFrameEdit,
        gesture_finished: bool,
        constraints: LayoutConstraints,
    ) -> Result<(), LayoutError> {
        if cancelled {
            self.cancel_gesture()?;
            return Ok(());
        }

        if edit != RuntimeFrameEdit::None {
            let proposal = match self.runtime.extract_proposal() {
                Ok(proposal) => proposal,
                Err(error) => {
                    self.reproject_authority()?;
                    return Err(error);
                }
            };
            let mut candidate = self.intent.clone();
            if let Err(error) = candidate.accept_runtime_proposal(proposal, constraints) {
                self.reproject_authority()?;
                return Err(error);
            }
            let runtime = RuntimeLayout::project(&candidate)?;
            if edit == RuntimeFrameEdit::Continuous && self.gesture_baseline.is_none() {
                self.gesture_baseline = Some(self.intent.clone());
            }
            self.intent = candidate;
            self.runtime = runtime;
        }

        if gesture_finished {
            self.gesture_baseline = None;
        }
        Ok(())
    }

    fn cancel_gesture(&mut self) -> Result<(), LayoutError> {
        if let Some(baseline) = self.gesture_baseline.take() {
            self.intent = baseline;
        }
        self.reproject_authority()
    }

    fn reproject_authority(&mut self) -> Result<(), LayoutError> {
        self.runtime = RuntimeLayout::project(&self.intent)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{PanelRole, SplitAxis};

    fn constraints() -> LayoutConstraints {
        LayoutConstraints {
            viewport_width: 1_000.0,
            stage_min_width: 320.0,
        }
    }

    #[test]
    fn product_reducer_accepts_split_tab_hide_restore_and_reset_sequence() {
        let mut authority = LayoutAuthority::built_in().unwrap();
        let mut proposal = authority.intent().clone();
        proposal
            .move_tab_for_test(PanelRole::Browser, PanelRole::Inspector, constraints())
            .unwrap();
        proposal.select_tab_for_test(PanelRole::Browser).unwrap();
        authority.replace_runtime_for_test(proposal).unwrap();
        authority
            .reconcile_runtime_frame(false, RuntimeFrameEdit::Commit, true, constraints())
            .unwrap();
        assert_eq!(
            authority.runtime().canonical_signature().unwrap(),
            authority.intent().canonical_signature()
        );

        let mut proposal = authority.intent().clone();
        proposal
            .move_split_for_test(
                PanelRole::Timeline,
                PanelRole::Stage,
                SplitAxis::Vertical,
                false,
                constraints(),
            )
            .unwrap();
        authority.replace_runtime_for_test(proposal).unwrap();
        authority
            .reconcile_runtime_frame(false, RuntimeFrameEdit::Commit, true, constraints())
            .unwrap();

        for action in [
            LayoutAction::Hide(PanelRole::Inspector),
            LayoutAction::Restore(PanelRole::Inspector),
            LayoutAction::ResetPreset,
        ] {
            authority.apply(action, constraints()).unwrap();
        }
        assert_eq!(
            authority.runtime().canonical_signature().unwrap(),
            PanelLayout::built_in().canonical_signature()
        );
    }

    #[test]
    fn invalid_runtime_proposal_is_rejected_and_reprojected_atomically() {
        let mut authority = LayoutAuthority::built_in().unwrap();
        let before = authority.intent().canonical_signature();
        authority.runtime_mut().remove_stage_for_test();
        assert!(authority
            .reconcile_runtime_frame(false, RuntimeFrameEdit::Commit, true, constraints())
            .is_err());
        assert_eq!(authority.intent().canonical_signature(), before);
        assert_eq!(authority.runtime().canonical_signature().unwrap(), before);
    }

    #[test]
    fn cancelled_frame_cannot_recommit_its_mutated_runtime() {
        let mut authority = LayoutAuthority::built_in().unwrap();
        let baseline = authority.intent().canonical_signature();
        let mut resized = authority.intent().clone();
        resized
            .apply(
                LayoutAction::Separator {
                    path: vec![0],
                    boundary: 0,
                    action: crate::layout::SeparatorAction::IncreaseLeading,
                },
                constraints(),
            )
            .unwrap();
        authority.replace_runtime_for_test(resized).unwrap();
        authority
            .reconcile_runtime_frame(false, RuntimeFrameEdit::Continuous, false, constraints())
            .unwrap();
        assert!(authority.gesture_in_flight());

        let mut same_frame_mutation = authority.intent().clone();
        same_frame_mutation
            .apply(
                LayoutAction::Separator {
                    path: vec![0],
                    boundary: 0,
                    action: crate::layout::SeparatorAction::IncreaseLeading,
                },
                constraints(),
            )
            .unwrap();
        authority
            .replace_runtime_for_test(same_frame_mutation)
            .unwrap();
        authority
            .reconcile_runtime_frame(true, RuntimeFrameEdit::Continuous, false, constraints())
            .unwrap();
        assert_eq!(authority.intent().canonical_signature(), baseline);
        assert_eq!(authority.runtime().canonical_signature().unwrap(), baseline);
        assert!(!authority.gesture_in_flight());
    }
}
