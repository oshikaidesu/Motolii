//! UI の既定表示方針だけを外部 JSON から読む leaf crate。
//!
//! `tokens.json`(色・寸法)と`.motolii-state.json`(利用者の現在状態)はここへ
//! 混ぜない。Document/Intent/StoreView/Renderにも依存しない。debug 実行時は
//! [`watch_subscription`]が変更を通知し、呼び手が[`LastGoodPresentation`]を
//! reload する。不正なファイルは現在の値を置き換えない。

use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const CURRENT_VERSION: u32 = 1;

/* motolii-component
id = "presentation.config"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["PresentationFileChanged", "parse", "reload_from_path"]
meaning = ["reload_from_json", "LoadError"]
evaluation = ["bundled_config_is_valid", "last_good_keeps_the_previous_value_after_a_bad_reload"]
render = ["PresentationConfig"]
observable = ["last_good_keeps_the_previous_value_after_a_bad_reload"]
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserTabId {
    Media,
    Effects,
    Create,
    Panels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageViewTabId {
    Camera,
    UserView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimelineDetailTabId {
    Timeline,
    Graph,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PresentationConfig {
    pub version: u32,
    /// 配置と可読性の出典を設定ファイル自身に残す。由来を隠さず、後から
    /// レイアウトを変更する人がどの作法を変更したのか辿れるようにする。
    pub sources: PresentationSources,
    pub layout_defaults: LayoutDefaults,
    pub browser: BrowserPresentation,
    pub stage: StagePresentation,
    pub timeline: TimelinePresentation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresentationSources {
    /// Browser/Timeline/Inspector/Stage の配置と概念の根拠。
    pub layout: String,
    /// 文字・コントラスト・状態表示の可読性の根拠。
    pub readability: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutDefaults {
    pub browser_open: bool,
    pub ratios: LayoutRatios,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutRatios {
    pub browser: f32,
    pub inspector: f32,
    pub content_timeline: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrowserPresentation {
    pub tabs: TabPresentation<BrowserTabId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StagePresentation {
    pub view_tabs: TabPresentation<StageViewTabId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelinePresentation {
    pub detail_tabs: DetailTabPresentation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabPresentation<T> {
    pub order: Vec<T>,
    pub visible: Vec<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetailTabPresentation {
    pub order: Vec<TimelineDetailTabId>,
    pub visible: Vec<TimelineDetailTabId>,
    pub default: TimelineDetailTabId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    InvalidVersion { found: u32 },
    InvalidRatio { name: &'static str, value: f32 },
    DuplicateId { group: &'static str },
    UnknownOrMissingId { group: &'static str },
    VisibleIdNotInOrder { group: &'static str },
    NoVisibleTabs { group: &'static str },
    DefaultTabNotVisible,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidVersion { found } => write!(f, "unsupported version: {found}"),
            Self::InvalidRatio { name, value } => write!(f, "invalid ratio {name}: {value}"),
            Self::DuplicateId { group } => write!(f, "duplicate tab id in {group}"),
            Self::UnknownOrMissingId { group } => write!(f, "unknown or missing tab id in {group}"),
            Self::VisibleIdNotInOrder { group } => {
                write!(f, "visible tab id is absent from order in {group}")
            }
            Self::NoVisibleTabs { group } => write!(f, "all tabs are hidden in {group}"),
            Self::DefaultTabNotVisible => write!(f, "default timeline tab is hidden"),
        }
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug)]
pub enum LoadError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Invalid(ValidationError),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "cannot read presentation config: {error}"),
            Self::Json(error) => write!(f, "invalid presentation JSON: {error}"),
            Self::Invalid(error) => write!(f, "invalid presentation config: {error}"),
        }
    }
}

impl std::error::Error for LoadError {}

impl From<std::io::Error> for LoadError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl PresentationConfig {
    pub fn parse(json: &str) -> Result<Self, LoadError> {
        let config: Self = serde_json::from_str(json).map_err(LoadError::Json)?;
        config.validate().map_err(LoadError::Invalid)?;
        Ok(config)
    }

    pub fn load_from_path(path: &Path) -> Result<Self, LoadError> {
        Self::parse(&std::fs::read_to_string(path)?)
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.version != CURRENT_VERSION {
            return Err(ValidationError::InvalidVersion { found: self.version });
        }
        for (name, value) in [
            ("browser", self.layout_defaults.ratios.browser),
            ("inspector", self.layout_defaults.ratios.inspector),
            ("content_timeline", self.layout_defaults.ratios.content_timeline),
        ] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) || value == 0.0 {
                return Err(ValidationError::InvalidRatio { name, value });
            }
        }
        validate_tabs("browser", &self.browser.tabs, &[BrowserTabId::Media, BrowserTabId::Effects, BrowserTabId::Create, BrowserTabId::Panels])?;
        validate_tabs("stage", &self.stage.view_tabs, &[StageViewTabId::Camera, StageViewTabId::UserView])?;
        validate_detail_tabs(
            "timeline",
            &self.timeline.detail_tabs,
            &[TimelineDetailTabId::Timeline, TimelineDetailTabId::Graph],
        )?;
        if !self.timeline.detail_tabs.visible.contains(&self.timeline.detail_tabs.default) {
            return Err(ValidationError::DefaultTabNotVisible);
        }
        Ok(())
    }

    /// debug 実行時に watch する既定ファイル。release では呼び手が通常使わない。
    pub fn debug_source_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("presentation/presentation.json")
    }
}

impl Default for PresentationConfig {
    fn default() -> Self {
        Self::parse(include_str!("../presentation/presentation.json"))
            .expect("bundled presentation config must be valid")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LastGoodPresentation {
    current: PresentationConfig,
}

impl Default for LastGoodPresentation {
    fn default() -> Self {
        Self { current: PresentationConfig::default() }
    }
}

impl LastGoodPresentation {
    pub fn load() -> Self {
        #[cfg(debug_assertions)]
        {
            let mut state = Self::default();
            let _ = state.reload();
            state
        }
        #[cfg(not(debug_assertions))]
        {
            Self::default()
        }
    }

    pub fn current(&self) -> &PresentationConfig {
        &self.current
    }

    pub fn reload(&mut self) -> Result<(), LoadError> {
        #[cfg(debug_assertions)]
        {
            self.reload_from_path(&PresentationConfig::debug_source_path())?;
        }
        #[cfg(not(debug_assertions))]
        {
            // release は埋め込みの既定値だけを使い、file I/O をしない。
        }
        Ok(())
    }

    /// 指定されたファイルを検証してから last-good を置き換える接続点。
    /// watcher 以外の検証器や隔離 fixture も同じ経路を使える。
    pub fn reload_from_path(&mut self, path: &Path) -> Result<(), LoadError> {
        self.reload_from_json(&std::fs::read_to_string(path)?)
    }

    /// 検証済みの JSON だけを last-good に反映する純粋な接続点。
    pub fn reload_from_json(&mut self, json: &str) -> Result<(), LoadError> {
        let next = PresentationConfig::parse(json)?;
        self.current = next;
        Ok(())
    }
}

fn validate_tabs<T: Copy + Eq + std::hash::Hash>(
    group: &'static str,
    tabs: &TabPresentation<T>,
    registry: &[T],
) -> Result<(), ValidationError> {
    validate_tab_lists(group, &tabs.order, &tabs.visible, registry)
}

fn validate_tab_lists<T: Copy + Eq + std::hash::Hash>(
    group: &'static str,
    order: &[T],
    visible: &[T],
    registry: &[T],
) -> Result<(), ValidationError> {
    let order_set: HashSet<T> = order.iter().copied().collect();
    let visible_set: HashSet<T> = visible.iter().copied().collect();
    let registry_set: HashSet<T> = registry.iter().copied().collect();
    if order_set.len() != order.len() || visible_set.len() != visible.len() {
        return Err(ValidationError::DuplicateId { group });
    }
    if !visible_set.is_subset(&order_set) {
        return Err(ValidationError::VisibleIdNotInOrder { group });
    }
    if visible.is_empty() {
        return Err(ValidationError::NoVisibleTabs { group });
    }
    if order_set != registry_set || order.len() != registry.len() {
        return Err(ValidationError::UnknownOrMissingId { group });
    }
    Ok(())
}

fn validate_detail_tabs(
    group: &'static str,
    tabs: &DetailTabPresentation,
    registry: &[TimelineDetailTabId],
) -> Result<(), ValidationError> {
    validate_tab_lists(group, &tabs.order, &tabs.visible, registry)
}

#[cfg(debug_assertions)]
pub fn watch_subscription() -> iced::Subscription<()> {
    iced::Subscription::run(watch_stream)
}

#[cfg(not(debug_assertions))]
pub fn watch_subscription() -> iced::Subscription<()> {
    iced::Subscription::none()
}

#[cfg(debug_assertions)]
fn watch_stream() -> impl iced::futures::Stream<Item = ()> {
    iced::stream::channel(8, |mut output: iced::futures::channel::mpsc::Sender<()>| async move {
        use notify::Watcher;
        let path = PresentationConfig::debug_source_path();
        std::thread::spawn(move || {
            let (tx, rx) =
                std::sync::mpsc::channel::<notify::Result<notify::Event>>();
            let Ok(mut watcher) = notify::recommended_watcher(tx) else { return };
            if watcher.watch(&path, notify::RecursiveMode::NonRecursive).is_err() {
                return;
            }
            loop {
                let Ok(event) = rx.recv() else { return };
                if event.is_err() {
                    continue;
                }
                while rx.recv_timeout(std::time::Duration::from_millis(80)).is_ok() {}
                if output.try_send(()).is_err() {
                    return;
                }
            }
        });
        std::future::pending::<()>().await;
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_json() -> String {
        include_str!("../presentation/presentation.json").to_owned()
    }

    #[test]
    fn bundled_config_is_valid() {
        PresentationConfig::parse(&valid_json()).expect("bundled config should validate");
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let json = valid_json().replace("\"media\", \"effects\", \"create\", \"panels\"", "\"media\", \"media\", \"create\", \"panels\"");
        assert!(matches!(PresentationConfig::parse(&json), Err(LoadError::Invalid(ValidationError::DuplicateId { group: "browser" }))));
    }

    #[test]
    fn hidden_all_tabs_are_rejected() {
        let json = valid_json().replace("\"visible\": [\"media\", \"effects\", \"create\", \"panels\"]", "\"visible\": []");
        assert!(matches!(PresentationConfig::parse(&json), Err(LoadError::Invalid(ValidationError::NoVisibleTabs { group: "browser" }))));
    }

    #[test]
    fn visible_must_be_a_subset_of_order() {
        let json = valid_json().replace("\"order\": [\"media\", \"effects\", \"create\", \"panels\"]", "\"order\": [\"media\", \"effects\", \"create\"]");
        assert!(matches!(PresentationConfig::parse(&json), Err(LoadError::Invalid(ValidationError::VisibleIdNotInOrder { group: "browser" }))));
    }

    #[test]
    fn last_good_keeps_the_previous_value_after_a_bad_reload() {
        let mut state = LastGoodPresentation::default();
        let before = state.current().clone();
        let result = state.reload_from_json("{\"version\": 999}");
        assert!(result.is_err());
        assert_eq!(state.current(), &before);
    }
}
