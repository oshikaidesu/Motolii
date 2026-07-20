//! U1a-2のtoolkit非依存panel layout正本。

use std::collections::BTreeSet;

const SHARE_UNITS: u32 = 1_000_000;
const KEYBOARD_SHARE_STEP: u32 = 50_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum PanelRole {
    Browser,
    Stage,
    Inspector,
    Timeline,
}

impl PanelRole {
    pub(crate) const ALL: [Self; 4] = [Self::Browser, Self::Stage, Self::Inspector, Self::Timeline];
    pub(crate) const AUXILIARY: [Self; 3] = [Self::Browser, Self::Inspector, Self::Timeline];

    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Browser => "Browser",
            Self::Stage => "Stage",
            Self::Inspector => "Inspector",
            Self::Timeline => "Timeline",
        }
    }

    pub(crate) const fn is_auxiliary(self) -> bool {
        !matches!(self, Self::Stage)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LayoutNode {
    Pane(PanelRole),
    Split {
        axis: SplitAxis,
        children: Vec<LayoutNode>,
        shares: Vec<u32>,
        default_shares: Vec<u32>,
    },
    Tabs {
        children: Vec<LayoutNode>,
        active: PanelRole,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PanelLayout {
    root: LayoutNode,
    hidden: BTreeSet<PanelRole>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LayoutConstraints {
    pub(crate) viewport_width: f32,
    pub(crate) stage_min_width: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SeparatorAction {
    DecreaseLeading,
    IncreaseLeading,
    Reset,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LayoutAction {
    Hide(PanelRole),
    Restore(PanelRole),
    ResetPreset,
    Separator {
        path: Vec<usize>,
        boundary: usize,
        action: SeparatorAction,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum LayoutError {
    #[error("layout role {role:?} is missing")]
    MissingRole { role: PanelRole },
    #[error("layout role {role:?} occurs more than once")]
    DuplicateRole { role: PanelRole },
    #[error("layout container is empty")]
    EmptyContainer,
    #[error("layout split has mismatched child and share counts")]
    ShareCountMismatch,
    #[error("layout split contains a zero share")]
    ZeroShare,
    #[error("layout tab group must contain panes directly")]
    NestedTabChild,
    #[error("layout tab group contains Stage")]
    StageInTabs,
    #[error("layout tab active role is not a child")]
    InvalidActiveTab,
    #[error("only auxiliary panes may be layout operation subjects")]
    NonAuxiliarySubject,
    #[cfg(test)]
    #[error("layout operation target does not exist")]
    MissingTarget,
    #[error("separator path does not identify a split")]
    InvalidSeparator,
    #[error("layout proposal has a non-finite or non-positive share")]
    InvalidRuntimeShare,
    #[error("layout runtime tree contains a dangling tile")]
    DanglingRuntimeTile,
    #[error("layout runtime tree contains a cycle or repeated tile")]
    RepeatedRuntimeTile,
    #[error("layout runtime grid is outside the U1a-2 closed set")]
    UnsupportedRuntimeGrid,
    #[error("layout viewport constraints are invalid")]
    InvalidConstraints,
}

impl Default for PanelLayout {
    fn default() -> Self {
        Self::built_in()
    }
}

impl PanelLayout {
    pub(crate) fn built_in() -> Self {
        Self {
            root: LayoutNode::Split {
                axis: SplitAxis::Vertical,
                children: vec![
                    LayoutNode::Split {
                        axis: SplitAxis::Horizontal,
                        children: vec![
                            LayoutNode::Pane(PanelRole::Browser),
                            LayoutNode::Pane(PanelRole::Stage),
                            LayoutNode::Pane(PanelRole::Inspector),
                        ],
                        shares: vec![1, 3, 1],
                        default_shares: vec![1, 3, 1],
                    },
                    LayoutNode::Pane(PanelRole::Timeline),
                ],
                shares: vec![3, 1],
                default_shares: vec![3, 1],
            },
            hidden: BTreeSet::new(),
        }
    }

    pub(crate) fn from_parts(
        root: LayoutNode,
        hidden: BTreeSet<PanelRole>,
    ) -> Result<Self, LayoutError> {
        let layout = Self { root, hidden };
        layout.validate()?;
        Ok(layout)
    }

    pub(crate) fn root(&self) -> &LayoutNode {
        &self.root
    }

    pub(crate) fn is_visible(&self, role: PanelRole) -> bool {
        !self.hidden.contains(&role)
    }

    pub(crate) fn canonical_signature(&self) -> String {
        fn write_node(node: &LayoutNode, output: &mut String) {
            match node {
                LayoutNode::Pane(role) => {
                    output.push_str("P:");
                    output.push_str(role.title());
                }
                LayoutNode::Split {
                    axis,
                    children,
                    shares,
                    ..
                } => {
                    output.push_str(match axis {
                        SplitAxis::Horizontal => "H[",
                        SplitAxis::Vertical => "V[",
                    });
                    for (index, (child, share)) in children.iter().zip(shares).enumerate() {
                        if index > 0 {
                            output.push(',');
                        }
                        output.push_str(&share.to_string());
                        output.push(':');
                        write_node(child, output);
                    }
                    output.push(']');
                }
                LayoutNode::Tabs { children, active } => {
                    output.push_str("T(");
                    output.push_str(active.title());
                    output.push_str(")[");
                    for (index, child) in children.iter().enumerate() {
                        if index > 0 {
                            output.push(',');
                        }
                        write_node(child, output);
                    }
                    output.push(']');
                }
            }
        }

        let mut output = String::new();
        write_node(&self.root, &mut output);
        output.push_str("|hidden=");
        for role in &self.hidden {
            output.push_str(role.title());
            output.push(',');
        }
        output.push_str("|status=Status");
        output
    }

    pub(crate) fn validate(&self) -> Result<(), LayoutError> {
        fn visit(node: &LayoutNode, seen: &mut BTreeSet<PanelRole>) -> Result<(), LayoutError> {
            match node {
                LayoutNode::Pane(role) => {
                    if !seen.insert(*role) {
                        return Err(LayoutError::DuplicateRole { role: *role });
                    }
                }
                LayoutNode::Split {
                    children,
                    shares,
                    default_shares,
                    ..
                } => {
                    if children.is_empty() {
                        return Err(LayoutError::EmptyContainer);
                    }
                    if children.len() != shares.len() || children.len() != default_shares.len() {
                        return Err(LayoutError::ShareCountMismatch);
                    }
                    if shares.iter().chain(default_shares).any(|share| *share == 0) {
                        return Err(LayoutError::ZeroShare);
                    }
                    for child in children {
                        visit(child, seen)?;
                    }
                }
                LayoutNode::Tabs { children, active } => {
                    if children.is_empty() {
                        return Err(LayoutError::EmptyContainer);
                    }
                    let mut has_active = false;
                    for child in children {
                        let LayoutNode::Pane(role) = child else {
                            return Err(LayoutError::NestedTabChild);
                        };
                        if *role == PanelRole::Stage {
                            return Err(LayoutError::StageInTabs);
                        }
                        has_active |= role == active;
                        visit(child, seen)?;
                    }
                    if !has_active {
                        return Err(LayoutError::InvalidActiveTab);
                    }
                }
            }
            Ok(())
        }

        if self.hidden.contains(&PanelRole::Stage) {
            return Err(LayoutError::NonAuxiliarySubject);
        }
        let mut seen = BTreeSet::new();
        visit(&self.root, &mut seen)?;
        for role in PanelRole::ALL {
            if !seen.contains(&role) {
                return Err(LayoutError::MissingRole { role });
            }
        }
        Ok(())
    }

    pub(crate) fn apply(
        &mut self,
        action: LayoutAction,
        constraints: LayoutConstraints,
    ) -> Result<(), LayoutError> {
        let mut candidate = self.clone();
        match action {
            LayoutAction::Hide(role) => {
                require_auxiliary(role)?;
                candidate.hidden.insert(role);
            }
            LayoutAction::Restore(role) => {
                require_auxiliary(role)?;
                candidate.hidden.remove(&role);
            }
            LayoutAction::ResetPreset => candidate = Self::built_in(),
            LayoutAction::Separator {
                path,
                boundary,
                action,
            } => {
                if action == SeparatorAction::Cancel {
                    return Ok(());
                }
                adjust_separator(&mut candidate.root, &path, boundary, action)?;
            }
        }
        candidate.clamp_stage_minimum(constraints)?;
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    pub(crate) fn accept_runtime_proposal(
        &mut self,
        mut candidate: PanelLayout,
        constraints: LayoutConstraints,
    ) -> Result<(), LayoutError> {
        candidate.validate()?;
        inherit_default_shares(&mut candidate.root, &self.root);
        candidate.clamp_stage_minimum(constraints)?;
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    fn clamp_stage_minimum(&mut self, constraints: LayoutConstraints) -> Result<(), LayoutError> {
        if !constraints.viewport_width.is_finite()
            || !constraints.stage_min_width.is_finite()
            || constraints.viewport_width <= 0.0
            || constraints.stage_min_width < 0.0
            || constraints.stage_min_width >= constraints.viewport_width
        {
            return Err(LayoutError::InvalidConstraints);
        }
        let required = f64::from(constraints.stage_min_width / constraints.viewport_width);
        let horizontal_depth = horizontal_stage_depth(&self.root);
        if horizontal_depth == 0 || stage_width_fraction(&self.root) >= required {
            return Ok(());
        }
        let per_split = required.powf(1.0 / horizontal_depth as f64);
        clamp_stage_path(&mut self.root, per_split)?;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn move_split_for_test(
        &mut self,
        subject: PanelRole,
        target: PanelRole,
        axis: SplitAxis,
        before: bool,
        constraints: LayoutConstraints,
    ) -> Result<(), LayoutError> {
        require_auxiliary(subject)?;
        if subject == target {
            return Err(LayoutError::MissingTarget);
        }
        let mut candidate = self.clone();
        let removed =
            remove_role(&mut candidate.root, subject).ok_or(LayoutError::MissingTarget)?;
        insert_split(&mut candidate.root, target, removed, axis, before)?;
        candidate.clamp_stage_minimum(constraints)?;
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn move_tab_for_test(
        &mut self,
        subject: PanelRole,
        target: PanelRole,
        constraints: LayoutConstraints,
    ) -> Result<(), LayoutError> {
        require_auxiliary(subject)?;
        require_auxiliary(target)?;
        if subject == target {
            return Err(LayoutError::MissingTarget);
        }
        let mut candidate = self.clone();
        let removed =
            remove_role(&mut candidate.root, subject).ok_or(LayoutError::MissingTarget)?;
        insert_tab(&mut candidate.root, target, removed)?;
        candidate.clamp_stage_minimum(constraints)?;
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn select_tab_for_test(&mut self, role: PanelRole) -> Result<(), LayoutError> {
        require_auxiliary(role)?;
        if select_tab(&mut self.root, role) {
            Ok(())
        } else {
            Err(LayoutError::MissingTarget)
        }
    }
}

pub(crate) fn normalize_runtime_shares(shares: &[f32]) -> Result<Vec<u32>, LayoutError> {
    if shares.is_empty() || shares.len() > SHARE_UNITS as usize {
        return Err(LayoutError::InvalidRuntimeShare);
    }
    if shares
        .iter()
        .any(|share| !share.is_finite() || *share <= 0.0)
    {
        return Err(LayoutError::InvalidRuntimeShare);
    }
    let sum: f64 = shares.iter().map(|share| f64::from(*share)).sum();
    let mut units = vec![0_u32; shares.len()];
    let mut remainders = Vec::with_capacity(shares.len());
    let mut assigned = 0_u32;
    for (index, share) in shares.iter().enumerate() {
        let exact = f64::from(*share) / sum * f64::from(SHARE_UNITS);
        let floor = exact.floor() as u32;
        units[index] = floor;
        assigned = assigned.saturating_add(floor);
        remainders.push((index, exact - f64::from(floor)));
    }
    remainders.sort_by(|(left_index, left), (right_index, right)| {
        right
            .total_cmp(left)
            .then_with(|| left_index.cmp(right_index))
    });
    for (index, _) in remainders
        .into_iter()
        .take(SHARE_UNITS.saturating_sub(assigned) as usize)
    {
        units[index] = units[index].saturating_add(1);
    }
    for zero_index in 0..units.len() {
        if units[zero_index] != 0 {
            continue;
        }
        let donor = units
            .iter()
            .enumerate()
            .filter(|(_, units)| **units > 1)
            .max_by_key(|(index, units)| (**units, std::cmp::Reverse(*index)))
            .map(|(index, _)| index)
            .ok_or(LayoutError::InvalidRuntimeShare)?;
        units[donor] -= 1;
        units[zero_index] = 1;
    }
    reduce_shares(&mut units);
    Ok(units)
}

fn require_auxiliary(role: PanelRole) -> Result<(), LayoutError> {
    if role.is_auxiliary() {
        Ok(())
    } else {
        Err(LayoutError::NonAuxiliarySubject)
    }
}

fn reduce_shares(shares: &mut [u32]) {
    fn gcd(mut left: u32, mut right: u32) -> u32 {
        while right != 0 {
            (left, right) = (right, left % right);
        }
        left
    }

    let divisor = shares.iter().copied().reduce(gcd).unwrap_or(1).max(1);
    for share in shares {
        *share /= divisor;
    }
}

#[cfg(test)]
fn remove_role(node: &mut LayoutNode, role: PanelRole) -> Option<LayoutNode> {
    match node {
        LayoutNode::Pane(_) => None,
        LayoutNode::Split {
            children,
            shares,
            default_shares,
            ..
        } => {
            if let Some(index) = children
                .iter()
                .position(|child| matches!(child, LayoutNode::Pane(found) if *found == role))
            {
                shares.remove(index);
                default_shares.remove(index);
                let removed = children.remove(index);
                collapse_single_child(node);
                return Some(removed);
            }
            for child in children.iter_mut() {
                if let Some(removed) = remove_role(child, role) {
                    collapse_single_child(node);
                    return Some(removed);
                }
            }
            None
        }
        LayoutNode::Tabs { children, active } => {
            if let Some(index) = children
                .iter()
                .position(|child| matches!(child, LayoutNode::Pane(found) if *found == role))
            {
                let removed = children.remove(index);
                if *active == role {
                    if let Some(LayoutNode::Pane(next)) = children.first() {
                        *active = *next;
                    }
                }
                collapse_single_child(node);
                return Some(removed);
            }
            None
        }
    }
}

#[cfg(test)]
fn collapse_single_child(node: &mut LayoutNode) {
    match node {
        LayoutNode::Split { children, .. } | LayoutNode::Tabs { children, .. }
            if children.len() == 1 =>
        {
            *node = children.remove(0);
        }
        _ => {}
    }
}

#[cfg(test)]
fn insert_split(
    node: &mut LayoutNode,
    target: PanelRole,
    subject: LayoutNode,
    axis: SplitAxis,
    before: bool,
) -> Result<(), LayoutError> {
    if matches!(node, LayoutNode::Pane(role) if *role == target) {
        let target_node = std::mem::replace(node, LayoutNode::Pane(PanelRole::Stage));
        let children = if before {
            vec![subject, target_node]
        } else {
            vec![target_node, subject]
        };
        *node = LayoutNode::Split {
            axis,
            children,
            shares: vec![1, 1],
            default_shares: vec![1, 1],
        };
        return Ok(());
    }
    match node {
        LayoutNode::Split { children, .. } | LayoutNode::Tabs { children, .. } => {
            for child in children {
                if insert_split(child, target, subject.clone(), axis, before).is_ok() {
                    return Ok(());
                }
            }
            Err(LayoutError::MissingTarget)
        }
        LayoutNode::Pane(_) => Err(LayoutError::MissingTarget),
    }
}

#[cfg(test)]
fn insert_tab(
    node: &mut LayoutNode,
    target: PanelRole,
    subject: LayoutNode,
) -> Result<(), LayoutError> {
    if matches!(node, LayoutNode::Pane(role) if *role == target) {
        let target_node = std::mem::replace(node, LayoutNode::Pane(PanelRole::Stage));
        let LayoutNode::Pane(subject_role) = subject else {
            return Err(LayoutError::NestedTabChild);
        };
        *node = LayoutNode::Tabs {
            children: vec![target_node, LayoutNode::Pane(subject_role)],
            active: subject_role,
        };
        return Ok(());
    }
    match node {
        LayoutNode::Split { children, .. } => {
            for child in children {
                if insert_tab(child, target, subject.clone()).is_ok() {
                    return Ok(());
                }
            }
            Err(LayoutError::MissingTarget)
        }
        LayoutNode::Tabs { children, active } => {
            if children
                .iter()
                .any(|child| matches!(child, LayoutNode::Pane(role) if *role == target))
            {
                let LayoutNode::Pane(subject_role) = subject else {
                    return Err(LayoutError::NestedTabChild);
                };
                children.push(LayoutNode::Pane(subject_role));
                *active = subject_role;
                Ok(())
            } else {
                Err(LayoutError::MissingTarget)
            }
        }
        LayoutNode::Pane(_) => Err(LayoutError::MissingTarget),
    }
}

#[cfg(test)]
fn select_tab(node: &mut LayoutNode, role: PanelRole) -> bool {
    match node {
        LayoutNode::Pane(_) => false,
        LayoutNode::Split { children, .. } => {
            children.iter_mut().any(|child| select_tab(child, role))
        }
        LayoutNode::Tabs { children, active } => {
            if children
                .iter()
                .any(|child| matches!(child, LayoutNode::Pane(found) if *found == role))
            {
                *active = role;
                true
            } else {
                false
            }
        }
    }
}

fn adjust_separator(
    node: &mut LayoutNode,
    path: &[usize],
    boundary: usize,
    action: SeparatorAction,
) -> Result<(), LayoutError> {
    let mut current = node;
    for &index in path {
        current = match current {
            LayoutNode::Split { children, .. } => children
                .get_mut(index)
                .ok_or(LayoutError::InvalidSeparator)?,
            LayoutNode::Tabs { children, .. } => children
                .get_mut(index)
                .ok_or(LayoutError::InvalidSeparator)?,
            LayoutNode::Pane(_) => return Err(LayoutError::InvalidSeparator),
        };
    }
    let LayoutNode::Split {
        shares,
        default_shares,
        ..
    } = current
    else {
        return Err(LayoutError::InvalidSeparator);
    };
    if boundary + 1 >= shares.len() {
        return Err(LayoutError::InvalidSeparator);
    }
    match action {
        SeparatorAction::Reset => {
            shares[boundary] = default_shares[boundary];
            shares[boundary + 1] = default_shares[boundary + 1];
        }
        SeparatorAction::IncreaseLeading | SeparatorAction::DecreaseLeading => {
            let total = shares[boundary].saturating_add(shares[boundary + 1]);
            let step = KEYBOARD_SHARE_STEP.min(total.saturating_sub(2));
            let leading = match action {
                SeparatorAction::IncreaseLeading => shares[boundary].saturating_add(step),
                SeparatorAction::DecreaseLeading => shares[boundary].saturating_sub(step),
                SeparatorAction::Reset | SeparatorAction::Cancel => unreachable!(),
            }
            .clamp(1, total.saturating_sub(1));
            shares[boundary] = leading;
            shares[boundary + 1] = total - leading;
            reduce_shares(shares);
        }
        SeparatorAction::Cancel => {}
    }
    Ok(())
}

fn inherit_default_shares(candidate: &mut LayoutNode, previous: &LayoutNode) {
    match (candidate, previous) {
        (
            LayoutNode::Split {
                axis: candidate_axis,
                children: candidate_children,
                default_shares,
                ..
            },
            LayoutNode::Split {
                axis: previous_axis,
                children: previous_children,
                default_shares: previous_defaults,
                ..
            },
        ) if candidate_axis == previous_axis
            && candidate_children.len() == previous_children.len()
            && same_role_shape(candidate_children, previous_children) =>
        {
            *default_shares = previous_defaults.clone();
            for (candidate_child, previous_child) in
                candidate_children.iter_mut().zip(previous_children)
            {
                inherit_default_shares(candidate_child, previous_child);
            }
        }
        _ => {}
    }
}

fn same_role_shape(left: &[LayoutNode], right: &[LayoutNode]) -> bool {
    left.iter().map(first_role).eq(right.iter().map(first_role))
}

fn first_role(node: &LayoutNode) -> PanelRole {
    match node {
        LayoutNode::Pane(role) => *role,
        LayoutNode::Split { children, .. } | LayoutNode::Tabs { children, .. } => {
            first_role(&children[0])
        }
    }
}

fn contains_stage(node: &LayoutNode) -> bool {
    match node {
        LayoutNode::Pane(role) => *role == PanelRole::Stage,
        LayoutNode::Split { children, .. } | LayoutNode::Tabs { children, .. } => {
            children.iter().any(contains_stage)
        }
    }
}

fn horizontal_stage_depth(node: &LayoutNode) -> usize {
    match node {
        LayoutNode::Pane(_) => 0,
        LayoutNode::Split { axis, children, .. } => {
            let child_depth = children
                .iter()
                .find(|child| contains_stage(child))
                .map(horizontal_stage_depth)
                .unwrap_or(0);
            child_depth + usize::from(*axis == SplitAxis::Horizontal)
        }
        LayoutNode::Tabs { .. } => 0,
    }
}

fn stage_width_fraction(node: &LayoutNode) -> f64 {
    match node {
        LayoutNode::Pane(role) => f64::from(*role == PanelRole::Stage),
        LayoutNode::Split {
            axis,
            children,
            shares,
            ..
        } => {
            let Some(index) = children.iter().position(contains_stage) else {
                return 0.0;
            };
            let child_fraction = stage_width_fraction(&children[index]);
            if *axis == SplitAxis::Vertical {
                child_fraction
            } else {
                let total: u64 = shares.iter().map(|share| u64::from(*share)).sum();
                f64::from(shares[index]) / total as f64 * child_fraction
            }
        }
        LayoutNode::Tabs { .. } => 0.0,
    }
}

fn clamp_stage_path(node: &mut LayoutNode, minimum_ratio: f64) -> Result<(), LayoutError> {
    let LayoutNode::Split {
        axis,
        children,
        shares,
        ..
    } = node
    else {
        return Ok(());
    };
    let Some(stage_index) = children.iter().position(contains_stage) else {
        return Ok(());
    };
    if *axis == SplitAxis::Horizontal {
        let current_total: u64 = shares.iter().map(|share| u64::from(*share)).sum();
        let current_ratio = f64::from(shares[stage_index]) / current_total as f64;
        if current_ratio < minimum_ratio {
            let mut normalized: Vec<f32> = shares.iter().map(|share| *share as f32).collect();
            let other_total: f64 = shares
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != stage_index)
                .map(|(_, share)| f64::from(*share))
                .sum();
            for (index, share) in normalized.iter_mut().enumerate() {
                *share = if index == stage_index {
                    minimum_ratio as f32
                } else {
                    ((1.0 - minimum_ratio) * f64::from(shares[index]) / other_total) as f32
                };
            }
            *shares = normalize_runtime_shares(&normalized)?;
        }
    }
    clamp_stage_path(&mut children[stage_index], minimum_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn constraints() -> LayoutConstraints {
        LayoutConstraints {
            viewport_width: 1_000.0,
            stage_min_width: 400.0,
        }
    }

    #[test]
    fn built_in_signature_is_deterministic_and_has_status_chrome() {
        let first = PanelLayout::built_in();
        let second = PanelLayout::built_in();
        assert_eq!(first.canonical_signature(), second.canonical_signature());
        assert!(first.canonical_signature().ends_with("|status=Status"));
    }

    #[test]
    fn fixed_auxiliary_operation_sequence_roundtrips() {
        let mut layout = PanelLayout::built_in();
        layout
            .move_tab_for_test(PanelRole::Browser, PanelRole::Inspector, constraints())
            .unwrap();
        layout.select_tab_for_test(PanelRole::Browser).unwrap();
        layout
            .move_split_for_test(
                PanelRole::Timeline,
                PanelRole::Stage,
                SplitAxis::Vertical,
                false,
                constraints(),
            )
            .unwrap();
        layout
            .apply(LayoutAction::Hide(PanelRole::Inspector), constraints())
            .unwrap();
        layout
            .apply(LayoutAction::Restore(PanelRole::Inspector), constraints())
            .unwrap();
        layout.validate().unwrap();
        assert!(layout.canonical_signature().contains("T(Browser)"));
        assert!(layout.is_visible(PanelRole::Inspector));
    }

    #[test]
    fn stage_cannot_be_hidden_or_tab_subject() {
        let mut layout = PanelLayout::built_in();
        let before = layout.clone();
        assert_eq!(
            layout.apply(LayoutAction::Hide(PanelRole::Stage), constraints()),
            Err(LayoutError::NonAuxiliarySubject)
        );
        assert_eq!(layout, before);
        assert_eq!(
            layout.move_tab_for_test(PanelRole::Stage, PanelRole::Browser, constraints(),),
            Err(LayoutError::NonAuxiliarySubject)
        );
        assert_eq!(layout, before);
    }

    #[test]
    fn invalid_proposal_does_not_partially_replace_authority() {
        let mut layout = PanelLayout::built_in();
        let before = layout.clone();
        let invalid = PanelLayout {
            root: LayoutNode::Pane(PanelRole::Stage),
            hidden: BTreeSet::new(),
        };
        assert!(layout
            .accept_runtime_proposal(invalid, constraints())
            .is_err());
        assert_eq!(layout, before);
    }

    #[test]
    fn runtime_share_normalization_is_fixed_order_and_reduced() {
        assert_eq!(
            normalize_runtime_shares(&[1.0, 3.0, 1.0]).unwrap(),
            vec![1, 3, 1]
        );
        let first = normalize_runtime_shares(&[0.1, 0.2, 0.7]).unwrap();
        let second = normalize_runtime_shares(&[0.1, 0.2, 0.7]).unwrap();
        assert_eq!(first, second);
        assert_eq!(first, vec![1, 2, 7]);
        assert_eq!(
            normalize_runtime_shares(&[f32::NAN, 1.0]),
            Err(LayoutError::InvalidRuntimeShare)
        );
    }

    #[test]
    fn stage_minimum_clamps_horizontal_share() {
        let mut layout = PanelLayout::built_in();
        layout
            .clamp_stage_minimum(LayoutConstraints {
                viewport_width: 800.0,
                stage_min_width: 500.0,
            })
            .unwrap();
        assert!(stage_width_fraction(layout.root()) >= 0.625);
    }

    #[test]
    fn separator_reset_and_full_reset_are_deterministic() {
        let mut layout = PanelLayout::built_in();
        layout
            .apply(
                LayoutAction::Separator {
                    path: vec![0],
                    boundary: 0,
                    action: SeparatorAction::IncreaseLeading,
                },
                constraints(),
            )
            .unwrap();
        let changed = layout.canonical_signature();
        layout
            .apply(
                LayoutAction::Separator {
                    path: vec![0],
                    boundary: 0,
                    action: SeparatorAction::Reset,
                },
                constraints(),
            )
            .unwrap();
        assert_ne!(layout.canonical_signature(), changed);
        layout
            .apply(LayoutAction::ResetPreset, constraints())
            .unwrap();
        assert_eq!(layout, PanelLayout::built_in());
    }
}
