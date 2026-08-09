# supervisor handoff — 統合地図の実測再構築、仮コード器具、Stage×M5判定

日付: 2026-08-08
状態: **引き継ぎ / 施工停止 / 次発注未選定**

**本書は前日の[handoff](2026-08-07-supervisor-handoff-map-rebuild-and-spine.md)を置き換えるものではなく、
その後に確定した事実を追加する。** 両方を読むこと。

## 1. この文書の扱い

runner規則でも設計決定でもacceptanceでもない。作業メモである。
再開時は本書をauthorityにせず、`AGENTS.md`、`docs/README.md`、decision index、
[成果駆動統合地図](../outcome-driven-integration-map.md)とcurrent codeを再照合する。

Codex利用枠逼迫のため、利用者判断でClaude（Opus 5）が代理supervisorを務めた。

## 2. Git安全境界（変更なし）

- authority checkout `/Users/member_ottoto/rust_ae/Motolii`、branch `codex/supervision-authority-guard-20260804`
- **rootでreset / checkout / cleanup / stage / commit / push / main統合を行っていない**
- local main worktree `/private/tmp/motolii-r0-main-integration-20260807`、HEAD `9b2deac4`、**clean・不変**
- 背骨統合branch `codex/r2-spine-integration-20260807`（worktree `/private/tmp/motolii-r2-spine-20260807`）
  — **local main未統合・未push**。`7851e3d0` / `0eb2a3c0` / `11c8d012` / `68546b8d` の4粒、`278 passed`

## 3. 前日handoff以降に確定したこと

### 3.1 地図を実測で再構築した

[成果駆動統合地図](../outcome-driven-integration-map.md)を新設。旧3地図（M3実行地図、M4/M5採択地図）の
**dispatch authorityを移し、M4/M5の技術採択は正本のまま維持**して実装順だけ需要側へ従属させた。

62項目のnode surveyにより、状態語彙を
`WIRED / BUILT_UNWIRED / PARTIAL / ABSENT / UNDECIDED / EXTERNAL` へ分けた。
**旧`TARGET_MISSING`が「本当に無い」と「旧routeにあるが未接続」を1語へ潰していたことが、
系統的な悲観と誤った見積もりの原因である。**

### 3.2 M3の価値観を更新した

[M3価値観更新](2026-08-07-m3-integration-zone-value-update.md)。
M3は「UIを作る工程」ではなく「**先に作った資産を接続し製品として成立させる統合ゾーン**」。
既定の推定を`BUILT_UNWIRED`とし、優先順位のsort keyを`concept.md`の完成条件
（3〜5分MVを音楽同期で書き出す）とした。

### 3.3 仮コードという器具を導入した

[器具境界決定](2026-08-07-provisional-call-site-sketch-instrument-decision.md)。
呼び出し側を先に書き、実名で埋まらない箇所を`???`として露出させる**非compile・非authority**の器具。

3段階で運用した。

| 段階 | 目的 | falsifier | 成果 |
|---|---|---|---|
| 背骨・M3 | 接続されているか | `???` × survey `ABSENT` | `N-OVERLAY`発見 |
| 製品全体（7区間） | 決定が噛み合うか | `???` × decision-index | [区間内側の合成失敗14件](2026-08-07-call-site-sketch-composition-failures.md) |
| 継ぎ目 | 区間Aの出力がBの入力になるか | 統合して照合 | [継ぎ目9件＋Stage×M5判定](2026-08-08-call-site-sketch-seams-and-stage-m5-verdict.md) |

**照合の不一致が毎回最大の発見だった。** survey単独でも仮コード単独でも出ない。

### 3.4 Stage×M5 判定

**絶対規律2（色変換一元化）と6（Preview/Export同一評価）は鎖の上で成立**。
点群Provider席は`LayerSourcePlugin`→`RenderStep::Plugin`→`render_graph_cached`として**配線ごと実在**。

判定は`NO`だが理由は「未着手」ではなく「**休止契約が追加を禁じている3点**」。
不足は**C0-Schema（projective observationの公開型）1つ**に絞られた。
意味論は決定済み・private fixture 5/5通過で、未成立は公開型のみ。

確定した順序: `M3背骨を閉じる → 休止契約が意味境界で開く → C0-Schema → 点群/spatial Provider`

### 3.5 リポジトリ外資産で判定を訂正した

[リポジトリ外資産の棚卸し](2026-08-08-out-of-repository-asset-inventory.md)。

**node surveyも仮コードもリポジトリの中しか見ない。**
`~/Documents/Codex/`配下に`MotoliiRnProbe`（RN製品UI再現、App.tsx 660行）、
**`skia-safe 0.99.0`+`wgpu 29`実動の`skia-timeline-probe`**、Windows target check、
Qt/QSG/QSkinny/Avalonia比較群、`StagePresentProbe.app`が実在する。

- `N-OVERLAY`: `ABSENT` → **`PROBE_ONLY`**。次手は既知実装調査ではなく**移管・接続**
- `R1-BROWSER`: 判定は正しいが含意が誤り（構築でなく**移管**。移管契約は既決かつ停止線）
- **`ABSENT` 11件中、外部確認済みは2件のみ。残り9件は未確認**

## 4. 未閉鎖・未着手（次のsupervisorが判断するもの）

| 項目 | 状態 |
|---|---|
| gizmo既知実装調査のpreflight | 材料は揃ったが**未記録**。`transform-gizmo`(MIT/Apache、view/projection行列を外部から受け、頂点データのみ返す)、`bevy_transform_gizmo`(「always renders on top」明記)。**bus factorと製品仕様7件は判定不能** |
| Rerun `re_renderer` の席 | `references.md`は`PATTERN`のみと定める。**点群Provider席への丸ごと採用はM5開放後の裁定事項**。`crate依存 or fork`は未裁定 |
| egui復帰 | 利用者から提起されたが**`AUTHORITY_CONFLICT`として保留**。2回撤回済み（2026-07-24、2026-08-07再確認） |
| 継ぎ目#1（Document書き込み二重） | 絶対規律4に触れる可能性。**違反と断定していない**。M2 ownerへの確認事項 |
| 継ぎ目#3（playhead二重） | 時刻席決定が明示的に受け入れた負債。**返済期限は`R2-FOCUS-PLAYHEAD-AUTHORITY`閉鎖時。前倒ししない** |
| 統合版仮コード16,464字 | evidence log内。器具境界上いずれ退役するため正本化していない |
| 製品全体を仮コードで描く案 | 7区間まで実施済み。**残りの範囲と粒度は未決** |

## 4.5 セッション後半の追加（2026-08-08 後半）

| 項目 | 状態 |
|---|---|
| **Skia `REJECT`→`ADOPT` 裁定** | [完了](2026-08-08-skia-reject-to-adopt-authority-reconciliation.md)。2026-07-21/27の`REJECT`を撤回。**alpha・色の懸念は撤回せず維持**。旧`REJECT`理由「Velloと重複」はVello退役で前提消滅 |
| **rust-skia 依存追加** | **commit済み** `ed9024fc`（branch `codex/n-overlay-skia-dep-20260808`）。`cargo check` PASS 1m22s / 278 passed。**local main未統合** |
| `references.md` へ skia 登録 | 完了（撤回経緯つき） |
| **鎖のgate / capsuleのgate** | [器具境界決定](2026-08-07-provisional-call-site-sketch-instrument-decision.md)§6.45 に規約化。**頻度は施工駆動の直前に1回** |
| **gate実効性の実測** | [測定](2026-08-08-gate-effectiveness-measurement.md)。鎖のgateは回収、capsuleのgateは1周で非収束 |
| 背骨の鎖 | 鎖のgateの12件を反映して**修正済み**（scratchpad `sketch-spine.rs.md`） |
| `N-OVERLAY-MINIMAL` capsule | **v2まで作成したが施工不可**。oracleが空実装を通す。**発注しない** |
| GAP-3 調査 | 3方向完了（NLE再リンク5製品／OSS hash 10件／asset pipeline 4件）。**4/4でidentityと有効性判定は別機構**。分割案は未裁定 |
| 素材不足時の扱い | [決定](2026-08-08-source-shortfall-ask-before-remap-decision.md)。`OverrunMode::Freeze`既定維持、変更は利用者へ問う |
| Group Bake / GAP-3 の連鎖 | [観察](2026-08-08-group-bake-chain-and-gap3-root.md)。`M4_CALLED: 0`、根は GAP-3 |

### 後半で判明した重要な訂正

- **`BUILT_UNWIRED` = 「繋ぐだけ」は不正確。** 実routeに admission 層がある
  （`admit_easing_terminal` が generation・layout epoch・interval再導出・same-value を拒否）
- **`N-ABI-SPLIT` の効果を過大に書いていた。** kernel に残るのは `runtime` と
  `projection_generation` だけではなく、`current_time` / `primary` / `stages` / `destroyed` /
  GPU owner も保持する
- **器具は「あるが禁止されているもの」「あるが挿入できないもの」を素通りする**
  （`OverrunMode::Loop` はv1 typed拒否だが鎖に`???`が出ない）

## 4.6 次セッションの最初の一手（利用者裁定済み）

利用者判断により、**次セッションは「未通過の鎖へ gate を通して修正する」から始める。**

仮コードは[成果物保全](2026-08-08-call-site-sketch-artifacts.md)へ退避済み
（scratchpadは session固有で失われるため）。

| 鎖 | 鎖のgate |
|---|---|
| 背骨（outcome A） | **通過・修正済み** |
| outcome B / C、Tune、Fork、Author、Publish | **未通過（6区間）** |

**背骨で `ERRORS: 12 / SEAM_BLOCKED: 4` が出たため、他も同程度の誤りを含むと想定する。**

手順:

1. 未通過6区間へ**鎖のgate**を回す（別family。器具境界決定 §6.45 の4点を検査）
2. 検出された誤りを鎖へ反映する
3. `???` と塞がったseamを**発注候補リスト**として確定する

**「鎖が揃った＝発注できる」ではない。** capsuleは別途書き、capsuleのgateを回す必要がある
（[実効性測定](2026-08-08-gate-effectiveness-measurement.md)のとおり1周では収束しない）。

## 5. 次の一手候補（未発注）

`N-OVERLAY` を推す。`ABSENT`のうち最も多くの下流（`R2-STAGE-GIZMO` / `R2-TL-NAV` /
`R2-CURVE-READ` / `R2-STAGE-VIEW`）を解放し、**しかも既に`PROBE_ONLY`で移管作業である**。
着手前に[依存優先・責任最小化ゲート](2026-07-24-dependency-first-responsibility-gate.md)を通すこと。

## 6. 運用上の記録（次のsupervisorへ）

- **主担当capsule起因の欠陥は本日累計8件。施工側（Grok 4.5）のallowlist逸脱は0件。**
  品質のbottleneckは発注書である
- **Sol（OpenAI）のcapsule reviewが発注前に16件を検出した。** 実装capsuleではこのgateを外さない
- **外部runの最終出力が痩せる事例が3件**（Author区間、Fork区間、gizmo反例監査）。
  いずれもsub-agent spawnまたはplan mode設定が原因。
  次は「**sub-agentを使うな。最終テキストに全て書け**」を明示する
- `--permission-mode plan` と `ExitPlanMode`/`Write`禁止の組合せは**成果物の保存先を失わせる**

## 7. 明示的非目標

- root dirty差分の整理、stage、commit、push
- 背骨branchのlocal main統合、remote push
- 休止契約の解除、C0-Schemaの起草
- `BUILT_UNWIRED` 30件の再実装
- 未確認の`ABSENT` 9件を推測で訂正すること
- 継ぎ目9件・区間内側14件をM3の接続粒として修理すること
- egui復帰・Rerun `re_renderer` 採用を利用者裁定なしに進めること
