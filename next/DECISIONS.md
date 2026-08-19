# 裁定ログ(追記のみ)

1裁定1行。取り消し線を引かず、覆した時は新しい行を足して「(YYYY-MM-DD の N を覆す)」と書く。
理由が長い物だけ `../docs/reviews/` に置き、ここには結論を書く。

| # | 日付 | 裁定 |
|---|---|---|
| 1 | 2026-08-20 | ドリフトの累積を1度リセットし、軸を1本にする。旧 workspace は歴史証拠として残す |
| 2 | 2026-08-20 | Document の実体は rerun store。undo は `edit` timeline の時間移動で、自前の undo 機構を作らない |
| 3 | 2026-08-20 | rerun は `crates/store/*` と `re_renderer` だけ引く。viewer 層(egui)は引かない |
| 4 | 2026-08-20 | AE の意味(Layer/Transform/Keyframe/Effect)は `re_types_core` の custom component として Motolii 側に建てる。`re_types` を fork しない |
| 5 | 2026-08-20 | front は iced のみ。pane は store への query の投影で、独自の状態を持たない |
| 6 | 2026-08-20 | 拡張の口は trait 1本。「component を読んで値か画を書く」以外の口を足さない |
| 7 | 2026-08-20 | 規律は `wraps:` / `owns:` marker 1つだけ。リンク台帳・索引・リンク検査を新 workspace に持ち込まない |
| 8 | 2026-08-20 | Document は `comp` 軸に載らない。property track を `edit` 軸へまるごと1行で置き、comp 時間の値は Motolii の評価器が出す(R0-A 実測: `LatestAtQuery` が単一 timeline のみで2次元 query が書けない) |
| 9 | 2026-08-20 | R0 は常設試験として残す。rerun fork の rev を上げた時はこれを回す |
