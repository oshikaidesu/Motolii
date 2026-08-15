//! 絵を出すための**固定サンプル**。`Document` にも `rn_product_host` にも触らない。
//!
//! C8 は「`chrome.tsx` / `panels/` の見た目が Blitz で出るか」だけを見る回なので、
//! ここの値は**意味を持たない置き**である。色・寸法・間隔は1つも含まない
//! (それらは `theme.rs` = `productStyles.ts` と各ローカル StyleSheet の写しだけが持つ)。
//!
//! `SettingsScreen`(`chrome.tsx:159-172`)と2つのパネル(`registry.tsx` / `AssetTaggingPanel.tsx`)は
//! props を取らず文言も原文に直書きなので、サンプルは持たない。サンプルが要るのは
//! props を取る `ExportScreen`(`chrome.tsx:109-121`)だけ。

/// `chrome.tsx:77` の `ExportPhase` の写し。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExportPhase {
    Idle,
    Exporting,
    Complete,
    Failed,
}

/// `chrome.tsx:109-121` の `ExportScreen` の props のうち、**表示に出るものだけ**。
/// `onOutputPathChange` / `onExport` は意味なので写さない。
pub struct ExportSample {
    /// `chrome.tsx:135` の `value={outputPath}`。空なら `chrome.tsx:131` の placeholder が出る。
    pub output_path: &'static str,
    /// `chrome.tsx:118` の `phase`。`titleActionDisabled` が付くかどうかに効く。
    pub phase: ExportPhase,
    /// `chrome.tsx:152-154` の `chromeModalStatus` に出る文字列。
    pub status_text: &'static str,
}

impl ExportSample {
    /// `chrome.tsx:122` — `const busy = phase === 'exporting';`
    pub fn busy(&self) -> bool {
        matches!(self.phase, ExportPhase::Exporting)
    }

    /// `chrome.tsx:123` — `const canRun = !busy && outputPath.trim().length > 0;`
    pub fn can_run(&self) -> bool {
        !self.busy() && !self.output_path.trim().is_empty()
    }
}

/// 既定の置き。`canRun` が真になるので `titleActionDisabled` は付かない。
pub const EXPORT_SAMPLE: ExportSample = ExportSample {
    output_path: "/Users/sample/out.mp4",
    phase: ExportPhase::Idle,
    status_text: "Ready",
};

/// `chrome.tsx:147` の `!canRun && styles.titleActionDisabled` の枝を絵で見るための置き。
pub const EXPORT_SAMPLE_BUSY: ExportSample = ExportSample {
    output_path: "/Users/sample/out.mp4",
    phase: ExportPhase::Exporting,
    status_text: "Exporting…",
};
