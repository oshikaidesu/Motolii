# ローカルworktree公開監査（2026-07-20）

状態: **観察／公開経路の整理**。本書は2026-07-20時点でローカルにだけ存在したcommit・未commit差分を、外部環境から再開できる資産と、現行mainへ吸収済みまたは旧契約の残骸へ分類した記録である。各branchの内容を採択する決定、mainへmergeする許可、未完成prototypeを正典へ昇格する決定ではない。

## 1. 公開した現行文書経路

| branch | 固定commit | 状態 | 外部環境での用途 |
|---|---|---|---|
| `codex/m3-text-motion-task-translation` | `5837808` | Draft PR #223 | Text Motionタスク翻訳、Rerun先例・139 package inventory、知覚表現の翻訳、Rerun発注動線のレビュー入口 |

このbranchだけが今回の正典更新候補である。Rerunのcrate依存、vendoring、移植、Document/plugin/public API変更は含まない。

## 2. 公開したM3比較・実装branch

次のbranchはmainへ混ぜず、分岐構造を保ったままGitHubへ公開した。外部環境では目的に合うbranchから再開し、複数branchを無条件に合成しない。

| branch | 固定commit | 関係 |
|---|---|---|
| `codex/m3-entry` | `cdb469e` | egui入場の共通土台 |
| `cursor/m3-u0b-1-night` | `201b34a` | `m3-entry` + 状態所有分類 |
| `cursor/m3-u0e-1-night` | `a74b93a` | `m3-entry` + 決定的token生成 |
| `cursor/m3-u1a-1-night` | `bd44a04` | `m3-entry` + visible egui shell |
| `codex/m3-browser-panel-spike` | `2661106` | `m3-u1a-1-night` + Browser fixtureのegui/taffy spike |

これらは同じ入場commitからの並行案を含む。branch名やcommitの存在を採択根拠にせず、対象spec、実装ガード、fixture、負例、現行mainとの差分を再確認する。

## 3. WIP保全したReact prototype

| branch | 固定commit | 状態 |
|---|---|---|
| `codex/m3-mock-components` | `7572376` | **WIP保全。PRなし、正典候補ではない** |

Timeline、Graph View、Easing、Browser、panel layout、操作台帳等の固有資産をローカル消失から守るため公開した。検証結果は次のとおり。

- `npm run build`: 成功
- `npm run test:visual`: **42 passed / 1 failed**
- 失敗: `browser-candidate.spec.js`のEasing旧selector期待（`.inline-key[data-key-context="current"]`）が現行DOMで0件
- `git diff --check`: 末尾空白2件を後続commitで修正済み

このbranchは古いmainを基底に持ち、legacy HTMLを含む移行途中の履歴も保持する。現行の`docs/mocks/`新規変更禁止を緩める根拠にせず、採用時はReact所有境界へ必要部分だけ再配置し、正典文書と現行mainへ再baseしてから別レビューする。失敗testを期待値変更で緑にしない。

## 4. 公開しなかったローカルdirty差分

以下は消去・resetせず保存したが、比較の結果、旧契約や現行mainへの吸収済み実装を再び正しそうに見せる危険が大きいため、新しい公開commitを作らなかった。

| worktree / branch | ローカル差分 | 比較結果 |
|---|---|---|
| `Motolii-ag-2` / `cursor/ag-2-pcm-mixer` | `encode.rs`の不要な`mut`除去1行 | 現行mainに同じ行が存在。吸収済み |
| `Motolii-m2-wave1-contracts` / `codex/m2-wave1-contracts` | plugin kind、future version、open warningの旧D1f案 | 現行mainは`EffectDefinition/Use`、prepared catalog diagnostics、Vism runtime境界へ進化済み。旧doc側ミラーを再導入しない |
| Cursor `d1l-contract-impl` | `new_current`、v4 readiness、inline/hybrid検出の旧案 | 現行mainはwriter v5、camera reader floor、v2 lifecycle、legacy migration、allowlist試験まで実装済み。旧v4前提は矛盾 |
| Claude `spike-57-timeline-bench` | Slint UI付きtimeline bench断片 | 現行mainの同名spikeはREADME、決定的data、headless evidence、culling/upload計測を持つ。旧Slint経路はegui決定とも不整合 |
| Claude `spike-56-ime-acceptance` | Slint IME画面断片 | 現行mainはchecklist manifest、static test、harness、READMEを持つ完成度の高い同名spike。旧断片を重ねない |

これらを将来参照する場合も、ローカル差分を直接cherry-pickせず、現行mainに欠ける意味が本当にあるかを先にfixtureで示す。

## 5. 外部環境での再開規則

1. 正典レビューはDraft PR #223から始める。
2. M3実装は`codex/m3-entry`を共通祖先として、対象ticketに対応するbranchを一つ選ぶ。
3. React prototypeは`codex/m3-mock-components`を比較資料として読むが、そのbranchのdocs、legacy HTML、状態所有を現行仕様とみなさない。
4. 未公開dirty worktreeは「GitHubに無い新機能」ではなく、原則として吸収済みまたは旧契約の比較証跡と扱う。
5. mainへ統合する前に、現行mainへのrebase/cherry-pick後の差分、対象spec、負例、必須testを改めて審査する。

## 6. 非目標

- すべてのローカルbranchをremoteへ複製してGitHubを履歴倉庫にすること
- 古いworktreeを削除、reset、cleanすること
- WIP branchを正典またはmerge可能と宣言すること
- 同じ成果が別hashでmainに入ったbranchを一律に再公開すること
- branch間の競合をこの監査だけで解消すること
