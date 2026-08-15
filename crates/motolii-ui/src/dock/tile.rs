// Ported from `egui_tiles` (https://github.com/rerun-io/egui_tiles) by rerun.io / Emil Ernerfeldt.
// Licensed under MIT OR Apache-2.0. Source: upstream commit fddb9fd (0.17.0 pre-release), `src/tile.rs`.
//
// なぜ: `TileId::egui_id` だけを落としてある。ドラッグ対象の同定は egui のメモリストアではなく
// `dock::drag::DragState` が持つため(C4 capsule の「HashMap 一つ」)。

use super::{Container, ContainerKind};

/// An identifier for a [`Tile`] in the tree, be it a [`Container`] or a pane.
///
/// This id is unique within the tree, but not across trees.
#[derive(Clone, Copy, Hash, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct TileId(pub u64);

impl TileId {
    #[inline]
    pub fn from_u64(n: u64) -> Self {
        Self(n)
    }
}

impl std::fmt::Debug for TileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

// ----------------------------------------------------------------------------

/// A tile in the tree. Either a pane (leaf) or a [`Container`] of more tiles.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Tile<Pane> {
    /// A leaf. This is where the user puts their UI, using the [`super::Behavior`] trait.
    Pane(Pane),

    /// A container of more tiles, e.g. a horizontal layout or a tab layout.
    Container(Container),
}

impl<T> From<Container> for Tile<T> {
    #[inline]
    fn from(container: Container) -> Self {
        Self::Container(container)
    }
}

impl<Pane> Tile<Pane> {
    /// Returns `None` if this is a [`Self::Pane`].
    #[inline]
    pub fn kind(&self) -> Option<ContainerKind> {
        match self {
            Self::Pane(_) => None,
            Self::Container(container) => Some(container.kind()),
        }
    }

    #[inline]
    pub fn is_pane(&self) -> bool {
        matches!(self, Self::Pane(_))
    }

    #[inline]
    pub fn is_container(&self) -> bool {
        matches!(self, Self::Container(_))
    }

    #[inline]
    pub fn container_kind(&self) -> Option<ContainerKind> {
        match self {
            Self::Pane(_) => None,
            Self::Container(container) => Some(container.kind()),
        }
    }
}
