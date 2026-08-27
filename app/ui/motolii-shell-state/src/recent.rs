//! 最近使ったプロジェクトの application-level 状態。
//!
//! Document の内容や Session を複製せず、利用者が次に開く候補の path だけを
//! bounded list として持つ。保存先は shell の user-settings sidecar であり、
//! project 本体へ混ぜない。

use std::path::{Path, PathBuf};

const MAX_RECENT: usize = 8;

/* motolii-component
id = "shell.recent_projects"
kind = "semantic"
weight = "convenience"
maps = []
entry = ["remember", "remove_missing", "paths"]
meaning = ["OpenRecentRequested", "RecentFileSelected"]
evaluation = ["remember_moves_the_latest_project_to_the_front", "remember_deduplicates_paths"]
render = ["recent_projects_view", "OpenRecentRequested"]
observable = ["remember_moves_the_latest_project_to_the_front"]
*/

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecentFiles {
    paths: Vec<PathBuf>,
}

impl RecentFiles {
    pub fn remember(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        self.paths.retain(|known| known != &path);
        self.paths.insert(0, path);
        self.paths.truncate(MAX_RECENT);
    }

    pub fn remove_missing(&mut self) {
        self.paths.retain(|path| path.exists());
    }

    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    pub fn path(&self, index: usize) -> Option<&Path> {
        self.paths.get(index).map(PathBuf::as_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remember_moves_the_latest_project_to_the_front() {
        let mut recent = RecentFiles::default();
        recent.remember("/tmp/a.moto");
        recent.remember("/tmp/b.moto");
        assert_eq!(recent.paths(), &[PathBuf::from("/tmp/b.moto"), PathBuf::from("/tmp/a.moto")]);
    }

    #[test]
    fn remember_deduplicates_paths() {
        let mut recent = RecentFiles::default();
        recent.remember("/tmp/a.moto");
        recent.remember("/tmp/b.moto");
        recent.remember("/tmp/a.moto");
        assert_eq!(recent.paths(), &[PathBuf::from("/tmp/a.moto"), PathBuf::from("/tmp/b.moto")]);
    }
}
