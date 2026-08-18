# アーカイブ索引

ここには**世代交代が確定した設計資産**を置く。削除はしない(このリポの規律「棄却物も歴史証拠として残す」)。移動と、この索引への一行登録だけで解決する。

現在の正本一覧は[docs/CANON.md](../CANON.md)を見よ。ここは「何が置き換わったか」の記録であり、「今どれが正本か」の索引ではない。

## 移動の判断基準

1. 後継が別の場所に存在し、リポ内のどこからも(コード・スクリプト・decision台帳の固定ID参照として)当該パスへ依存していないこと
2. 単なる「新しいものが出た」ではなく、当該資産自身が答えていた問いへの回答が後継へ移っていること
3. 判断が付かない場合は動かさない。特に、decision台帳が特定の generation ID や path を「不変(削除・移動しない)」と明記している資産(例: `docs/mocks-ui/reference-output/generations/u0e2-08f96cbd7754-85c0fc529ab1`)は対象外とする

## m3-main-ui-early-mocks/ (2026-08-19移動)

M3当初(2026-07-16〜18)に作られた、高密度メインUIの単発HTML比較モック群。後継は`docs/mocks-ui/`(React/Vite製、Playwright回帰付き)、視覚正本は`docs/mocks-ui/public/{inspector,browser,timeline}-library.html`(最終更新2026-08-16)。移動元は`docs/mocks/`。

| ファイル | これは何だったか | 何に置き換わったか | 残す理由 |
|---|---|---|---|
| `m3-main-ui-v1.html` + `-dark.png` / `-light.png` | M3視覚構成の最初の基準モック(密度・light/dark切替の実装見本) | `docs/mocks-ui/public/inspector-library.html` / `browser-library.html` / `timeline-library.html`(面ごとに分割・最終更新2026-08-16) | `docs/mocks-ui/component-map.json`が`status: comparison-only`(`historical-baseline`)として明示的に保持指定。密度比較の一次資料として有効 |
| `m3-main-ui-v2.html` + `-dark.png` / `-light.png` | v1と同一fixtureで「余白を分離の手段にしない」規約を検証したグリッド基調の比較案 | 同上(v1同様、面はmocks-ui側へ分割済み) | `component-map.json`が`status: comparison-only`(`historical-comparison`)。比較実験の記録として保持 |
| `m3-timeline-v0.svg` / `.png` | 最初期のTimeline静止画モック(製品UIでもgoldenでもない) | `docs/mocks-ui/public/timeline-library.html`、実装は`crates/motolii-ui/src/timeline_editor/` | `component-map.json`が`status: comparison-only`。当時の意図の記録 |
| `m3-interaction-v0.html` | 状態遷移をstep送りで見せた最初期のinteractionモック | 後継の操作モデルは`docs/ui-interaction-language.md` | `component-map.json`が`status: comparison-only`。比較記録 |
| `m3-ui-dynamics-v1.html` | 力学検証モック(状態機械をシナリオ列として前面に出す構成) | なし(下記の通り不採用) | `docs/mocks/README.md`が制作当時に**既に「以後は不採用比較案」と明記**していた(2026-07-16再判定)。以後どこからも参照されておらず、不採用の一次記録として保持 |

## 動かさなかったもの(検討したが対象外と判断)

調査の過程で以下も「古そう」に見えたが、下記の理由でこのアーカイブには**移動しなかった**。同じ誤解をしないための記録。

- **`docs/mocks/m3-vism-host-boundary.html`とその golden PNG 2枚**: `docs/mocks-ui/src/legacy/legacySource.js`が`?raw`でビルド時importする**現役の依存**。`docs/decision-index.md`と複数のreview文書からも参照されており、`component-map.json`は`status: archived`とラベルしているが、これは「新しいUI判断を持ち込まない」という役割凍結の意味であり、物理的な移動は禁止(移動するとmocks-ui のビルドが壊れる)
- **`docs/mocks/m3-main-ui-v3-monochrome.html`・`v4-generative.html`・`v5-color-dynamics.html`・`m3-plugin-boundary-learning.html`とそれぞれのgolden PNG**: `docs/mocks-ui/README.md`のプローズは「専門的な履歴比較候補」と表現するが、`docs/mocks-ui/component-map.json`(機械可読の権威マニフェスト)は同じファイルを`status: "active"`または`"candidate"`と明記しており、2つの文書の間で評価が食い違っている。判断が割れている資産を一存で動かすのは危険なので**保留**。利用者に判断を仰ぐ価値がある(詳細は本アーカイブ作業のセッション報告を参照)
- **`docs/mocks-ui/reference-output/generations/u0e2-08f96cbd7754-85c0fc529ab1`**: `docs/reviews/2026-07-28-g0-6h-s-human-judgment-input-route-decision.md`が「**削除・再生成・移動・期待値変更をしない**」と明記した不変の回帰証拠。CURRENTではないが対象外
- **`docs/mocks-ui/current-route-output/generations/`の非CURRENT 6世代**: `docs/implementation-ledger.md`および複数のCU決定文書(CU-203, CU-0a08btp, CU-0a08itp, G0-6H human acceptance, CU-0b02c)がgeneration IDを指定してacceptance証拠として引用している。移動すると引用元の追跡が壊れるため対象外
