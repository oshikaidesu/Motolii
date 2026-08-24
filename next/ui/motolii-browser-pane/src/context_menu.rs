//! Media-card context menu.
//!
//! This is deliberately a pane-local component.  It owns the transient anchor
//! (`CardKey`) and the small menu vocabulary, but it does not own an `Asset`,
//! a document selection, or an intent.  The only command currently available
//! from a Browser card is the already-connected `RemoveAssetFromCard` message.
//! A menu item is not emitted for preview-local cards because there is no real
//! removal meaning for those cards.

use crate::model::CardKey;
use crate::Message;
use iced::widget::{button, column, container, text};
use iced::{Element, Length};
use motolii_store::AssetId;
use motolii_tokens_rs::{Colors, Dimensions};

/// The transient anchor for the menu.  The card key is both the target passed
/// to the existing command and the placement anchor; no global cursor or
/// second selection is needed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct State {
    anchor: Option<CardKey>,
}

impl State {
    pub(crate) fn open(&mut self, anchor: CardKey) {
        self.anchor = supports(anchor).then_some(anchor);
    }

    pub(crate) fn close(&mut self) {
        self.anchor = None;
    }

    pub(crate) fn anchor(&self) -> Option<CardKey> {
        self.anchor
    }
}

/// A menu action is only declared when the pane has an existing meaning to
/// hand off.  Keeping this mapping here prevents the view from growing
/// placeholder actions as the Browser gets more cards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    RemoveAsset(AssetId),
}

pub(crate) fn supports(anchor: CardKey) -> bool {
    matches!(anchor, CardKey::Media(_))
}

pub(crate) fn actions_for(anchor: CardKey) -> Vec<Action> {
    match anchor {
        CardKey::Media(asset) => vec![Action::RemoveAsset(asset)],
        CardKey::Preview(_) => Vec::new(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Placement {
    BelowAnchor,
}

/// The Browser card is the stable anchor.  The fork's `mouse_area` gives us a
/// right-press callback but not a pointer position, so anchoring below the
/// target card is deterministic and keeps this component independent of Shell
/// coordinates.
pub(crate) fn placement_for(anchor: Option<CardKey>) -> Option<Placement> {
    anchor
        .filter(|key| supports(*key))
        .map(|_| Placement::BelowAnchor)
}

/// Draw the menu at the target card.  The menu intentionally contains one
/// action today: `RemoveAssetFromCard`, which Shell already translates to the
/// real `Intent::RemoveAsset` path.  No rename/favorite/tag entries are shown
/// because those meanings are not connected in this pane.
pub(crate) fn view(
    anchor: Option<CardKey>,
    card_width: Length,
    dims: Dimensions,
    colors: Colors,
) -> Option<Element<'static, Message>> {
    let anchor = anchor?;
    placement_for(Some(anchor))?;

    let items = actions_for(anchor)
        .into_iter()
        .map(|action| match action {
            Action::RemoveAsset(asset) => button(
                text("Remove from library")
                    .size(dims.micro_text)
                    .color(colors.text_primary),
            )
            .on_press(Message::RemoveAssetFromCard(asset))
            .width(card_width)
            .padding([dims.spacing_xs, dims.spacing_s])
            .style(move |_theme, status| {
                crate::search_view::chip_style(
                    dims,
                    colors,
                    false,
                    status,
                    crate::filter_view::FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO
                        * dims.row_height,
                )
            })
            .into(),
        })
        .collect::<Vec<Element<'static, Message>>>();

    Some(
        container(column(items).spacing(dims.spacing_xs))
            .width(card_width)
            .padding(dims.spacing_xs)
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(colors.surface_raised)),
                border: iced::Border {
                    color: colors.border_default,
                    width: dims.border_width,
                    radius: 0.0.into(),
                },
                ..container::Style::default()
            })
            .into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_and_closing_keeps_the_anchor_pane_local() {
        let asset = AssetId::from_raw(7);
        let mut state = State::default();

        state.open(CardKey::Media(asset));
        assert_eq!(state.anchor(), Some(CardKey::Media(asset)));

        state.close();
        assert_eq!(state.anchor(), None);
    }

    #[test]
    fn preview_cards_have_no_context_action() {
        let preview = CardKey::Preview("glow");

        assert!(!supports(preview));
        assert!(actions_for(preview).is_empty());
        assert_eq!(placement_for(Some(preview)), None);
    }

    #[test]
    fn media_card_layout_is_anchored_below_the_card() {
        let asset = CardKey::Media(AssetId::from_raw(11));

        assert_eq!(placement_for(Some(asset)), Some(Placement::BelowAnchor));
        assert_eq!(
            actions_for(asset),
            vec![Action::RemoveAsset(AssetId::from_raw(11))]
        );
    }
}
