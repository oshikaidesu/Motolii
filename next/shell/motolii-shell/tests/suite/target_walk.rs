//! `Shell::view()` を全 widget について歩き尽くす共有 walker。
//!
//! 元は `q0_fence.rs` に private として実装されていた(タスク#15)。S 空間
//! スコアの atlas dump(`entrance_atlas_dump.rs`、`docs/ui-spatial-score.md`
//! 「器具(実装計画)」1)が同じ walker を要求したため、二重実装を避けて
//! ここへ抽出した — `q0_fence.rs` のテスト本体・判定ロジックは無改変で、
//! `collect_targets` の呼び出し元を `crate::target_walk::collect_targets` へ
//! 差し替えただけ。
//!
//! ## 仕組み
//!
//! iced の `widget::Operation` は widget 自身が `operate()` で明示的に登録した
//! 物しか見えない(`iced_core::widget::Operation` の `container`/`focusable`/
//! `scrollable`/`text_input`/`text`/`custom` の6種、upstream 0.14.0 実測)。
//! `iced_test::Selector` はその登録済み候補(`Candidate`)を辿るだけなので、
//! **`view()` にある widget が全部見えるわけではない**(`canvas`/`slider` は
//! `operate()` 未実装 — `q0_fence.rs` 冒頭 doc の「検出できない限界」参照)。
//!
//! `Simulator::find` は1件だけ返す(`Selector::find()` → `One` 戦略)。
//! `Simulator` に `find_all` は無い(iced_test 0.14.0 の公開APIを実際に読んで
//! 確認)。全件を得るため、[`collect_targets`] は「まだ拾っていない候補」を
//! 選ぶ stateful な closure selector で `find` を **`Err` が返るまで繰り返す**。

use iced_test::selector::{Candidate, Target};

use motolii_shell::Message;

/// `element` の中で `widget::Operation` に登録されている候補を重複なく全部集める。
/// 6種(Container/Focusable/Scrollable/TextInput/Text/Custom)を区別せず集める
/// — button/row/column/container はどれも自分を `Container` として登録するので
/// (upstream 実測)、種別を絞ると button を取りこぼす。
pub fn collect_targets(element: iced::Element<'_, Message>) -> Vec<Target> {
    let mut ui = iced_test::simulator(element);
    let mut found: Vec<Target> = Vec::new();

    loop {
        let already = found.clone();
        let selector = move |candidate: Candidate<'_>| -> Option<Target> {
            let target = Target::from(candidate);
            if already.contains(&target) {
                None
            } else {
                Some(target)
            }
        };
        match ui.find(selector) {
            Ok(target) => found.push(target),
            Err(_) => break,
        }
        // 暴走防止。現行 pane 構成なら数十件で尽きる。増えるのは pane が
        // 増えた時の自然な成長であって、上げてよい — ここは無限ループだけを防ぐ。
        assert!(
            found.len() <= 5_000,
            "widget candidate が5000件を超えた — dedup が効いていない疑い"
        );
    }

    found
}
