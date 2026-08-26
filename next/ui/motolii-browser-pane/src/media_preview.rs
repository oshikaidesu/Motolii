//! Media-card preview handoff.
//!
//! The Browser owns the gesture and the typed asset identity only.  It does
//! not create a player, a second asset record, or a fake preview surface.
//! `motolii-shell` can later consume [`PreviewMedia`] at the existing
//! `Message::Browser` boundary and route it to a real source-preview owner.

/* motolii-component
id = "browser.media_card_preview"
kind = "semantic"
weight = "render_export"
maps = []
entry = ["PreviewMedia"]
meaning = ["preview_media_request"]
evaluation = ["preview_media_target"]
render = ["media_card_preview"]
observable = ["media_card_double_click_publishes_preview"]
*/

use crate::model::CardKey;
use crate::Message;
use iced::widget::{container, mouse_area};
use iced::{Element, Length};
use motolii_store::AssetId;
use motolii_tokens_rs::{Colors, Dimensions};

/// A request to inspect one admitted media asset.
///
/// The `AssetId` is the only payload on purpose.  The Shell already owns the
/// Document/Asset table and can resolve the current path there; carrying a
/// copied path through Browser would create a second source of truth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreviewMedia {
    asset_id: AssetId,
}

impl PreviewMedia {
    pub const fn new(asset_id: AssetId) -> Self {
        Self { asset_id }
    }

    pub const fn asset_id(self) -> AssetId {
        self.asset_id
    }
}

/// Resolve a card identity into the only preview target this component owns.
/// Preview-local Effects/Create/Panel cards deliberately cannot become media
/// preview requests.
pub(crate) fn preview_media_target(key: CardKey) -> Option<PreviewMedia> {
    match key {
        CardKey::Media(asset_id) => Some(PreviewMedia::new(asset_id)),
        CardKey::Preview(_) => None,
    }
}

/// Publish the typed handoff at the pane boundary.
pub(crate) fn preview_media_request(asset_id: AssetId) -> Message {
    Message::PreviewMedia(PreviewMedia::new(asset_id))
}

/// Media card surface with the existing selection/recent grammar and a real
/// double-click handoff.  The surface deliberately has no play button: the
/// source-preview player belongs to the Shell/preview owner, not Browser.
#[allow(clippy::too_many_arguments)]
pub(crate) fn media_card_preview(
    body: Element<'static, Message>,
    asset_id: AssetId,
    selected: bool,
    recent: bool,
    hovered: bool,
    select_message: Message,
    card_width: Length,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let key = CardKey::Media(asset_id);
    let face = container(body)
        .width(card_width)
        .padding(dims.theme().space.xs)
        .style(move |_theme| media_card_face(dims, colors, selected, recent, hovered));

    mouse_area(face)
        .on_press(select_message)
        .on_double_click(preview_media_request(asset_id))
        .on_enter(Message::CardHovered(key))
        .on_exit(Message::CardUnhovered(key))
        .interaction(iced::mouse::Interaction::Pointer)
        .into()
}

/// Container equivalent of the existing button-card style.  The state order
/// stays identical: selected wins over hover, and recent controls only the
/// focus-colored border.
pub(crate) fn media_card_face(
    dims: Dimensions,
    colors: Colors,
    selected: bool,
    recent: bool,
    hovered: bool,
) -> container::Style {
    let background = if selected {
        Some(iced::Background::Color(colors.state_selected))
    } else if hovered {
        Some(iced::Background::Color(colors.surface_hover))
    } else {
        None
    };
    let border_color = if recent {
        colors.focus
    } else {
        iced::Color::TRANSPARENT
    };
    container::Style {
        background,
        border: iced::Border {
            color: border_color,
            width: dims.theme().stroke.hairline,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_media_keeps_the_asset_identity() {
        let request = PreviewMedia::new(AssetId::from_raw(17));
        assert_eq!(request.asset_id(), AssetId::from_raw(17));
        assert_eq!(
            preview_media_target(CardKey::Media(AssetId::from_raw(17))),
            Some(request)
        );
    }

    #[test]
    fn preview_local_cards_have_no_media_preview_target() {
        assert_eq!(preview_media_target(CardKey::Preview("glow")), None);
    }

    #[test]
    fn selected_face_wins_over_hover_without_losing_recent_border() {
        let dims = Dimensions::default();
        let colors = Colors::default();
        let style = media_card_face(dims, colors, true, true, true);
        assert_eq!(
            style.background,
            Some(iced::Background::Color(colors.state_selected))
        );
        assert_eq!(style.border.color, colors.focus);
        assert_eq!(style.border.width, dims.theme().stroke.hairline);
    }
}
