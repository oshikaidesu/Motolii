# G0-6H-E 現行候補5画面承認の限定観察

- 日付: 2026-07-28
- 状態: **観察**

## 1. 対象と非対象

- 対象は、2026-07-28にユーザー本人が承認した現行 `#plugin-browser-candidate` の 5 画面。
- 非対象は、旧 `#reference/*`、old `30 PNG` 派生、画像への具体的採否判定、`#plugin-browser-candidate` 外部の状態判定、旧決定の再開。

## 2. 事実

- 日付は 2026-07-28 で、承認者はユーザー本人。
- 対象は現行 `#plugin-browser-candidate` の 5 画面 capture。
- 撮影条件は 1440×900、dark、normal 色である。
- 5 状態は mixed Timeline / Browser検索0件 / Interval Easing / Hand / Relative Move。
- 全5画面に Browser、Stage、Inspector、Timeline が含まれる。
- 固定 React source authority は `56c318edcddab7cf95d263cc2f7dd2b4e6791134`。
- 画像は閲覧済みであるが、リポジトリへは未取込。
- 表示環境（OS / display / scale / ambient）は未提供で、今回未取得。
- `npm run check-reference` は現行 tree で `reference generation OK: u0e2-08f96cbd7754-85c0fc529ab1 (30 PNGs)` を返したが、これは read-only の再現証拠である。
- ユーザーは旧 `#reference/*` と `u0e2-08f96cbd7754-85c0fc529ab1` の派生 25 枚を本粒で承認していない。

## 3. この観察が確定しないこと

- この観察は、5 画面の人間視覚的合否そのものを確定するものではない。
- 表示環境由来の評価（OS、display、scale、ambient）と派生判定（lightness / grayscale / CVD）は未取得のため確定しない。
- 画像の採択・棄却、具体 token 値、具体的な design token の採否、UI threshold の変更は未確定として扱う。
- `reference-handoff` の Decision template / checklist は未充足のまま維持し、観察はこれに代替しない。

## 4. 関連

- [G0-6H-E0 選定決定](2026-07-28-g0-6h-e-candidate-approval-evidence-selection.md)
- [reference handoff](../mocks-ui/reference-handoff.md)
- [この粒の証拠README](evidence/g0-6h-candidate-approval/README.md)
