use std::collections::{BTreeMap, BTreeSet};

use crate::doc::store::AssetId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum BrowserScope {
    Media,
    Effects,
    Create,
    Colors,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum CreateItem {
    Text,
    Rectangle,
    Bezier,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum BrowserItemId {
    Media(AssetId),
    Effect(String),
    Create(CreateItem),
    Color([u8; 4]),
}

impl BrowserItemId {
    /// Stable key used by the DOM focus registry. It names the Browser scope as
    /// well as the item, so equal-looking labels in different faces never alias.
    pub(super) fn focus_key(&self) -> String {
        match self {
            Self::Media(id) => format!("browser:media:{}", id.get()),
            Self::Effect(id) => format!("browser:effect:{id}"),
            Self::Create(kind) => format!("browser:create:{kind:?}"),
            Self::Color(rgba) => format!(
                "browser:color:{:02x}{:02x}{:02x}{:02x}",
                rgba[0], rgba[1], rgba[2], rgba[3]
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MoveActive {
    Previous,
    Next,
    PagePrevious,
    PageNext,
    First,
    Last,
}

/// The CSS grid is responsive and does not expose its computed column count to
/// Dioxus. Page navigation therefore advances a deterministic eight results.
pub(super) const PAGE_STEP: usize = 8;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ScopeSelection {
    selected: BTreeSet<BrowserItemId>,
    active: Option<BrowserItemId>,
    anchor: Option<BrowserItemId>,
}

/// Shared Browser selection. IDs survive result reordering and panel moves;
/// each Browser face keeps an independent active item and selection.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct BrowserSelection {
    scopes: BTreeMap<BrowserScope, ScopeSelection>,
    queries: BTreeMap<BrowserScope, String>,
    select_modes: BTreeSet<BrowserScope>,
}

impl BrowserSelection {
    fn state_mut(&mut self, scope: BrowserScope) -> &mut ScopeSelection {
        self.scopes.entry(scope).or_default()
    }

    fn state(&self, scope: BrowserScope) -> Option<&ScopeSelection> {
        self.scopes.get(&scope)
    }

    pub(super) fn query(&self, scope: BrowserScope) -> &str {
        self.queries.get(&scope).map(String::as_str).unwrap_or("")
    }

    pub(super) fn set_query(&mut self, scope: BrowserScope, query: String) -> bool {
        if self.query(scope) == query {
            return false;
        }
        if query.is_empty() {
            self.queries.remove(&scope);
        } else {
            self.queries.insert(scope, query);
        }
        true
    }

    pub(super) fn clear_query(&mut self, scope: BrowserScope) -> bool {
        self.queries.remove(&scope).is_some()
    }

    pub(super) fn select_mode(&self, scope: BrowserScope) -> bool {
        self.select_modes.contains(&scope)
    }

    pub(super) fn toggle_select_mode(&mut self, scope: BrowserScope) -> bool {
        if self.select_modes.remove(&scope) {
            false
        } else {
            self.select_modes.insert(scope);
            true
        }
    }

    /// Drop IDs which no longer exist without changing surviving selection when
    /// results are reordered or filtered.
    pub(super) fn reconcile(&mut self, scope: BrowserScope, all: &[BrowserItemId]) {
        let available: BTreeSet<_> = all.iter().cloned().collect();
        let state = self.state_mut(scope);
        state.selected.retain(|id| available.contains(id));
        if state
            .active
            .as_ref()
            .is_some_and(|id| !available.contains(id))
        {
            state.active = all.iter().find(|id| state.selected.contains(*id)).cloned();
        }
        if state
            .anchor
            .as_ref()
            .is_some_and(|id| !available.contains(id))
        {
            state.anchor = state.active.clone();
        }
    }

    /// Select an item using desktop collection conventions: plain replaces,
    /// Command toggles, Shift replaces with the anchor range, and Command+Shift
    /// adds the anchor range.
    pub(super) fn click(
        &mut self,
        scope: BrowserScope,
        id: &BrowserItemId,
        ordered: &[BrowserItemId],
        command: bool,
        shift: bool,
    ) -> bool {
        let Some(clicked) = ordered.iter().find(|candidate| *candidate == id).cloned() else {
            return false;
        };
        let state = self.state_mut(scope);
        let before = state.clone();

        if shift {
            let anchor = state
                .anchor
                .as_ref()
                .filter(|anchor| ordered.contains(anchor))
                .cloned()
                .or_else(|| {
                    state
                        .active
                        .as_ref()
                        .filter(|active| ordered.contains(active))
                        .cloned()
                })
                .unwrap_or_else(|| clicked.clone());
            if !command {
                state.selected.clear();
            }
            extend_range(&mut state.selected, ordered, &anchor, &clicked);
            state.anchor = Some(anchor);
        } else if command {
            if !state.selected.remove(&clicked) {
                state.selected.insert(clicked.clone());
            }
            state.anchor = Some(clicked.clone());
        } else {
            state.selected.clear();
            state.selected.insert(clicked.clone());
            state.anchor = Some(clicked.clone());
        }
        state.active = Some(clicked);
        *state != before
    }

    pub(super) fn move_active(
        &mut self,
        scope: BrowserScope,
        ordered: &[BrowserItemId],
        movement: MoveActive,
        extend: bool,
    ) -> Option<BrowserItemId> {
        if ordered.is_empty() {
            return None;
        }

        let state = self.state_mut(scope);
        let current = state
            .active
            .as_ref()
            .and_then(|active| ordered.iter().position(|id| id == active));
        let next = match movement {
            MoveActive::First => 0,
            MoveActive::Last => ordered.len() - 1,
            MoveActive::Previous => current.unwrap_or(1).saturating_sub(1),
            MoveActive::Next => current.map_or(0, |index| (index + 1).min(ordered.len() - 1)),
            MoveActive::PagePrevious => current.unwrap_or(PAGE_STEP).saturating_sub(PAGE_STEP),
            MoveActive::PageNext => {
                current.map_or(0, |index| (index + PAGE_STEP).min(ordered.len() - 1))
            }
        };
        let id = ordered[next].clone();
        if extend {
            let anchor = state
                .anchor
                .as_ref()
                .filter(|anchor| ordered.contains(anchor))
                .cloned()
                .or_else(|| current.map(|index| ordered[index].clone()))
                .unwrap_or_else(|| id.clone());
            state.selected.clear();
            extend_range(&mut state.selected, ordered, &anchor, &id);
            state.anchor = Some(anchor);
        } else {
            state.selected.clear();
            state.selected.insert(id.clone());
            state.anchor = Some(id.clone());
        }
        state.active = Some(id.clone());
        Some(id)
    }

    pub(super) fn select_all(&mut self, scope: BrowserScope, ordered: &[BrowserItemId]) -> bool {
        let state = self.state_mut(scope);
        let before = state.clone();
        state.selected = ordered.iter().cloned().collect();
        if state
            .active
            .as_ref()
            .is_none_or(|active| !ordered.contains(active))
        {
            state.active = ordered.first().cloned();
        }
        state.anchor = state.active.clone();
        *state != before
    }

    pub(super) fn clear(&mut self, scope: BrowserScope) -> bool {
        let state = self.state_mut(scope);
        let changed =
            !state.selected.is_empty() || state.active.is_some() || state.anchor.is_some();
        *state = ScopeSelection::default();
        changed
    }

    pub(super) fn is_selected(&self, scope: BrowserScope, id: &BrowserItemId) -> bool {
        self.state(scope)
            .is_some_and(|state| state.selected.contains(id))
    }

    pub(super) fn active(&self, scope: BrowserScope) -> Option<&BrowserItemId> {
        self.state(scope).and_then(|state| state.active.as_ref())
    }

    pub(super) fn active_in(
        &self,
        scope: BrowserScope,
        ordered: &[BrowserItemId],
    ) -> Option<BrowserItemId> {
        self.active(scope)
            .filter(|active| ordered.contains(active))
            .cloned()
    }

    pub(super) fn activate(
        &mut self,
        scope: BrowserScope,
        id: &BrowserItemId,
        ordered: &[BrowserItemId],
    ) -> bool {
        let Some(id) = ordered.iter().find(|candidate| *candidate == id).cloned() else {
            return false;
        };
        let state = self.state_mut(scope);
        let changed = state.active.as_ref() != Some(&id) || state.anchor.as_ref() != Some(&id);
        state.active = Some(id.clone());
        state.anchor = Some(id);
        changed
    }

    /// Keep one keyboard target in the visible result set without selecting it.
    pub(super) fn ensure_active_in(
        &mut self,
        scope: BrowserScope,
        ordered: &[BrowserItemId],
    ) -> bool {
        let state = self.state_mut(scope);
        if state
            .active
            .as_ref()
            .is_some_and(|active| ordered.contains(active))
        {
            return false;
        }
        let next = ordered.first().cloned();
        let changed = state.active != next;
        state.active = next.clone();
        state.anchor = next;
        changed
    }

    pub(super) fn selected_in_order(
        &self,
        scope: BrowserScope,
        ordered: &[BrowserItemId],
    ) -> Vec<BrowserItemId> {
        let Some(state) = self.state(scope) else {
            return Vec::new();
        };
        ordered
            .iter()
            .filter(|id| state.selected.contains(*id))
            .cloned()
            .collect()
    }

    pub(super) fn selected_or_active_in_order(
        &self,
        scope: BrowserScope,
        ordered: &[BrowserItemId],
    ) -> Vec<BrowserItemId> {
        let selected = self.selected_in_order(scope, ordered);
        if selected.is_empty() {
            self.active_in(scope, ordered).into_iter().collect()
        } else {
            selected
        }
    }

    pub(super) fn marquee(
        &mut self,
        scope: BrowserScope,
        ordered: &[BrowserItemId],
        hits: &[BrowserItemId],
        additive: bool,
    ) -> bool {
        let before = self.clone();
        let state = self.state_mut(scope);
        if !additive {
            state.selected.clear();
        }
        let hit_set: BTreeSet<_> = hits.iter().collect();
        let hits_in_order: Vec<_> = ordered
            .iter()
            .filter(|id| hit_set.contains(id))
            .cloned()
            .collect();
        state.selected.extend(hits_in_order.iter().cloned());
        if let Some(active) = hits_in_order.last().cloned() {
            state.active = Some(active);
            state.anchor = hits_in_order.first().cloned();
        } else if !additive {
            state.active = None;
            state.anchor = None;
        }
        *self != before
    }
}

fn extend_range(
    selected: &mut BTreeSet<BrowserItemId>,
    ordered: &[BrowserItemId],
    anchor: &BrowserItemId,
    focus: &BrowserItemId,
) {
    let Some(anchor) = ordered.iter().position(|id| id == anchor) else {
        return;
    };
    let Some(focus) = ordered.iter().position(|id| id == focus) else {
        return;
    };
    let (start, end) = if anchor <= focus {
        (anchor, focus)
    } else {
        (focus, anchor)
    };
    selected.extend(ordered[start..=end].iter().cloned());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn effects(names: &[&str]) -> Vec<BrowserItemId> {
        names
            .iter()
            .map(|name| BrowserItemId::Effect((*name).to_owned()))
            .collect()
    }

    #[test]
    fn plain_toggle_range_and_add_range_follow_desktop_collection_rules() {
        let ids = effects(&["a", "b", "c", "d"]);
        let mut selection = BrowserSelection::default();

        assert!(selection.click(BrowserScope::Effects, &ids[1], &ids, false, false));
        assert_eq!(
            selection.selected_in_order(BrowserScope::Effects, &ids),
            vec![ids[1].clone()]
        );

        selection.click(BrowserScope::Effects, &ids[3], &ids, true, false);
        assert_eq!(
            selection.selected_in_order(BrowserScope::Effects, &ids),
            vec![ids[1].clone(), ids[3].clone()]
        );

        selection.click(BrowserScope::Effects, &ids[2], &ids, false, true);
        assert_eq!(
            selection.selected_in_order(BrowserScope::Effects, &ids),
            vec![ids[2].clone(), ids[3].clone()]
        );

        selection.click(BrowserScope::Effects, &ids[0], &ids, true, true);
        assert_eq!(
            selection.selected_in_order(BrowserScope::Effects, &ids),
            ids
        );
    }

    #[test]
    fn selection_is_scoped_and_stable_across_reordering() {
        let media = [
            BrowserItemId::Media(AssetId::from_raw(7)),
            BrowserItemId::Media(AssetId::from_raw(9)),
        ];
        let create = [BrowserItemId::Create(CreateItem::Rectangle)];
        let mut selection = BrowserSelection::default();
        selection.click(BrowserScope::Media, &media[0], &media, false, false);
        selection.click(BrowserScope::Create, &create[0], &create, false, false);

        selection.reconcile(BrowserScope::Media, &[media[1].clone(), media[0].clone()]);
        assert!(selection.is_selected(BrowserScope::Media, &media[0]));
        assert_eq!(selection.active(BrowserScope::Media), Some(&media[0]));
        assert!(selection.is_selected(BrowserScope::Create, &create[0]));
    }

    #[test]
    fn reconciliation_drops_vanished_ids_and_keeps_a_surviving_active_item() {
        let ids = effects(&["a", "b", "c"]);
        let mut selection = BrowserSelection::default();
        selection.click(BrowserScope::Effects, &ids[0], &ids, false, false);
        selection.click(BrowserScope::Effects, &ids[1], &ids, true, false);
        selection.reconcile(BrowserScope::Effects, &[ids[1].clone(), ids[2].clone()]);

        assert_eq!(selection.active(BrowserScope::Effects), Some(&ids[1]));
        assert_eq!(
            selection.selected_in_order(BrowserScope::Effects, &ids),
            vec![ids[1].clone()]
        );
    }

    #[test]
    fn arrows_home_end_extend_select_all_and_escape_are_deterministic() {
        let ids = effects(&["a", "b", "c", "d"]);
        let mut selection = BrowserSelection::default();

        assert_eq!(
            selection.move_active(BrowserScope::Effects, &ids, MoveActive::Next, false),
            Some(ids[0].clone())
        );
        assert_eq!(
            selection.move_active(BrowserScope::Effects, &ids, MoveActive::Last, false),
            Some(ids[3].clone())
        );
        assert_eq!(
            selection.move_active(BrowserScope::Effects, &ids, MoveActive::Previous, true),
            Some(ids[2].clone())
        );
        assert_eq!(
            selection.selected_in_order(BrowserScope::Effects, &ids),
            vec![ids[2].clone(), ids[3].clone()]
        );

        assert!(selection.select_all(BrowserScope::Effects, &ids));
        assert_eq!(
            selection.selected_in_order(BrowserScope::Effects, &ids),
            ids
        );
        assert!(selection.clear(BrowserScope::Effects));
        assert!(selection
            .selected_in_order(BrowserScope::Effects, &ids)
            .is_empty());
        assert_eq!(selection.active(BrowserScope::Effects), None);
    }

    #[test]
    fn command_toggle_can_leave_an_active_but_unselected_item() {
        let ids = effects(&["a"]);
        let mut selection = BrowserSelection::default();
        selection.click(BrowserScope::Effects, &ids[0], &ids, false, false);
        selection.click(BrowserScope::Effects, &ids[0], &ids, true, false);

        assert_eq!(selection.active(BrowserScope::Effects), Some(&ids[0]));
        assert!(!selection.is_selected(BrowserScope::Effects, &ids[0]));
    }

    #[test]
    fn activating_an_already_selected_context_target_keeps_the_multi_selection() {
        let ids = effects(&["a", "b", "c"]);
        let mut selection = BrowserSelection::default();
        selection.click(BrowserScope::Effects, &ids[0], &ids, false, false);
        selection.click(BrowserScope::Effects, &ids[2], &ids, true, false);

        assert!(selection.activate(BrowserScope::Effects, &ids[0], &ids));
        assert_eq!(selection.active(BrowserScope::Effects), Some(&ids[0]));
        assert_eq!(
            selection.selected_in_order(BrowserScope::Effects, &ids),
            vec![ids[0].clone(), ids[2].clone()]
        );
    }

    #[test]
    fn focus_keys_include_scope_and_stable_identity() {
        assert_eq!(
            BrowserItemId::Media(AssetId::from_raw(42)).focus_key(),
            "browser:media:42"
        );
        assert_ne!(
            BrowserItemId::Effect("Rectangle".into()).focus_key(),
            BrowserItemId::Create(CreateItem::Rectangle).focus_key()
        );
    }

    #[test]
    fn page_navigation_uses_the_documented_linear_step() {
        let ids: Vec<_> = (0..20)
            .map(|n| BrowserItemId::Effect(n.to_string()))
            .collect();
        let mut selection = BrowserSelection::default();
        selection.click(BrowserScope::Effects, &ids[2], &ids, false, false);
        assert_eq!(
            selection.move_active(BrowserScope::Effects, &ids, MoveActive::PageNext, false,),
            Some(ids[2 + PAGE_STEP].clone())
        );
        assert_eq!(
            selection.move_active(BrowserScope::Effects, &ids, MoveActive::PagePrevious, false,),
            Some(ids[2].clone())
        );
    }

    #[test]
    fn queries_and_selections_are_shared_but_scoped_by_face() {
        let effects = effects(&["blur"]);
        let media = [BrowserItemId::Media(AssetId::from_raw(3))];
        let mut selection = BrowserSelection::default();
        selection.click(BrowserScope::Effects, &effects[0], &effects, false, false);
        selection.click(BrowserScope::Media, &media[0], &media, false, false);

        assert!(selection.set_query(BrowserScope::Effects, "bl".into()));
        assert!(selection.set_query(BrowserScope::Media, "song".into()));
        assert_eq!(selection.query(BrowserScope::Effects), "bl");
        assert_eq!(selection.query(BrowserScope::Media), "song");
        assert!(selection.clear_query(BrowserScope::Effects));
        assert_eq!(selection.query(BrowserScope::Media), "song");
        assert!(selection.is_selected(BrowserScope::Effects, &effects[0]));
        assert!(selection.is_selected(BrowserScope::Media, &media[0]));
    }

    #[test]
    fn marquee_replaces_or_adds_visible_hits_and_select_mode_is_scoped() {
        let ids = effects(&["a", "b", "c", "d"]);
        let mut selection = BrowserSelection::default();
        assert!(selection.toggle_select_mode(BrowserScope::Effects));
        assert!(selection.select_mode(BrowserScope::Effects));
        assert!(!selection.select_mode(BrowserScope::Media));

        selection.marquee(
            BrowserScope::Effects,
            &ids,
            &[ids[1].clone(), ids[2].clone()],
            false,
        );
        assert_eq!(
            selection.selected_in_order(BrowserScope::Effects, &ids),
            vec![ids[1].clone(), ids[2].clone()]
        );
        selection.marquee(BrowserScope::Effects, &ids, &[ids[0].clone()], true);
        assert_eq!(
            selection.selected_in_order(BrowserScope::Effects, &ids),
            vec![ids[0].clone(), ids[1].clone(), ids[2].clone()]
        );
        selection.marquee(BrowserScope::Effects, &ids, &[], false);
        assert!(selection
            .selected_in_order(BrowserScope::Effects, &ids)
            .is_empty());
    }

    #[test]
    fn visible_results_always_have_one_active_keyboard_target_without_selecting_it() {
        let ids = effects(&["a", "b", "c"]);
        let mut selection = BrowserSelection::default();
        assert!(selection.ensure_active_in(BrowserScope::Effects, &ids));
        assert_eq!(selection.active(BrowserScope::Effects), Some(&ids[0]));
        assert!(selection
            .selected_in_order(BrowserScope::Effects, &ids)
            .is_empty());

        assert!(selection.ensure_active_in(BrowserScope::Effects, &ids[1..]));
        assert_eq!(selection.active(BrowserScope::Effects), Some(&ids[1]));
    }
}
