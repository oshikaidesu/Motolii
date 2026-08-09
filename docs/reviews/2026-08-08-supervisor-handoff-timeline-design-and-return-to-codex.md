# supervisor handoff — Timeline設計と代理supervisorの返上

日付: 2026-08-08
状態: **引き継ぎ / supervisor席をCodexへ返す / 発注未実施**

## 0. この文書の扱い

runner規則でも設計決定でもacceptanceでもない。作業メモである。
再開時は本書をauthorityにせず、`AGENTS.md`、`docs/README.md`、`decision-index`、
current codeと再照合する。

**Codex利用枠逼迫のあいだ Claude（Opus 5）が代理supervisorを務めていたが、
Codex復帰により本日をもって席を返す。** 利用者判断による。
authority、次粒選定、owner、scope、oracle、finding処分、最終統合はsupervisor席が持つ。

## 1. Git安全境界

- authority checkout `/Users/member_ottoto/rust_ae/Motolii`、branch `codex/supervision-authority-guard-20260804`
- **セッション開始時 HEAD は `f800cb4f`（前日の引き継ぎ記載 `1cb92362` から進んでいた）。
  working tree は clean だった。** dirtyだったdocsは `f800cb4f` で commit 済み（本session外の作業）
- **root で reset / checkout / cleanup / stage / commit / push / main統合を行っていない**
- local main worktree `/private/tmp/motolii-r0-main-integration-20260807`、HEAD `9b2deac4`、**clean・不変**
- 背骨統合branch `/private/tmp/motolii-r2-spine-20260807`、HEAD `68546b8d`、**local main未統合・未push**
- skia依存branch `/private/tmp/motolii-n-overlay-20260808`、HEAD `ed9024fc`、**未統合**

### 未commitで残しているもの

```text
M  docs/decision-index.md
M  docs/reviews/README.md
?? docs/reviews/2026-08-08-completion-condition-call-site-sketch.md
?? docs/reviews/2026-08-08-timeline-design-decisions-and-skia-fixtures.md
?? docs/reviews/2026-08-08-supervisor-handoff-timeline-design-and-return-to-codex.md
```

`2026-08-08-mascot-and-pet-decision.md` と `decision-index` のマスコット行は**本sessionの成果ではない**
（並行して書かれたもの。触っていない）。

`./scripts/check-docs.sh` と `git diff --check` は通過している。

## 2. 本日の成果

### 2.1 完成条件の鎖を初めて書いた

[完成条件の鎖](2026-08-08-completion-condition-call-site-sketch.md)。
sort key（3〜5分・音楽同期・音声mux）そのものの鎖が一度も書かれていなかった。

**最大の発見: 音声mux は実装済みなのに、楽曲bedを作品へ据える編集操作が無い。**
値型・validation・永続化・評価・mix・mux はすべて実在し、欠けているのは Command 1系統だけ。
repo内外とも確認済みで `ABSENT`。`N-SOUNDTRACK-WRITE` として新規node候補に立てた。

**起草者自身が完成条件を読み替えていた**という訂正も記録した。「音楽同期」は
`concept.md` の含意列挙では**音声mux**であり、拍同期編集ではない。
BPM Rhythm Vism は完成条件の critical path に乗っていない。

**鎖のgateではこの誤りは出ない。** 検査4点に「その`???`は完成条件を塞ぐか」が無いため。

### 2.2 Timeline設計（本日の中心）

[Timeline設計決定とskia fixture](2026-08-08-timeline-design-decisions-and-skia-fixtures.md)。
**決定12件、すべて理由つき。** 観察4件、暫定設計値、未決6件、非目標。

進め方を途中で変えたことが効いた。HTMLでモックを重ねても解けなかったので、

1. **一次資料を測る** — インストール済み Live 12 のテーマXML（236色）、公式マニュアル、利用者の実機screenshot
2. **目標renderer（skia）で直接描く** — 小サイズ文字のラスタライズ、1px罫、grid密度の破綻は
   HTMLでは判定できなかった

**代理supervisorの推測は色・レイアウト・密度のすべてで外れていた。** 実測に切り替えてから収束した。

副産物として**CJKフォント同梱が必須**という製品要求が確定した
（skia既定フォントがCJKを解決せず日本語が豆腐になる。AbletonがNoto Sans CJKを同梱しているのと同じ理由）。

### 2.3 skia fixture 7本

`~/Documents/Codex/2026-08-06/motolii-ui-hybrid-research-handoff/work/skia-timeline-probe/src/bin/`

`motolii_tl` / `kf` / `fold` / `row` / `full` / `group` / `clip`。
**既存probeファイルとリポジトリのコードは無変更。** `cargo build --release --bin <name>` は数秒。

## 3. 次の候補（未発注。選定はsupervisor席）

利用者は「実装粒はUI設計が終わるまで混乱の元」と判断済み。read-onlyの2件は混ざらない。

| | 種別 | UI設計と混ざるか |
|---|---|---|
| `N-SOUNDTRACK-WRITE` 既知実装preflight | read-only、記録が出る | 混ざらない |
| b1 の鎖のgate | read-only監査 | 混ざらない |
| 背骨の次の1粒（RN host gesture席） | コードが動く | **混ざる** |
| `N-OVERLAY` 移管 | コードが動く | **混ざる** |

### 背骨の次の1粒について（本日判明した事実）

**drag → release → key は `N-OVERLAY` を必要としない。**

- `draw_stage_preview` は render済みtextureのfullscreen blit（`rn_product_host.rs`）
- transient preview の既存機構は overlay ではなく**実Commandの適用**
  （`resolve_position_gesture_command`、`product_runtime.rs:3396`。Inspector scrubが今これで動く）
- 1 Undo は `DocumentWriter::begin_gesture`、取消は `DomainIntent::CancelInFlightGesture`（登録済み7 intentの1つ）
- `rn_product_host.rs` に `PositionGestureBaseline` / `position_gesture` は **0件**＝席が無いだけ

**`N-OVERLAY` が塞いでいるのは handle の絵であって動きではない。**
地図の `gizmo handle描画 ABSENT → N-OVERLAY` を drag 全体が止まると読んでいたのが誤り。

## 4. 起動準備だけ済ませて**起動していない**もの

**`run-observed-cli.py` を一度も呼んでいない。** 発注はsupervisor席の裁量である。

準備済みの事実（再利用可。破棄してよい）:

- Codex CLI `/Applications/ChatGPT.app/Contents/Resources/codex` / `codex-cli 0.146.0-alpha.3.1` 実在確認
- **exact model ID を CLI自身の `~/.codex/models_cache.json`（本日06:10取得）から確認**
  — `gpt-5.6-sol` / `gpt-5.6-terra` / `gpt-5.6-luna` / `gpt-5.3-codex-spark` / `gpt-5.4` / `gpt-5.5`。
  runbook は `gpt-5.3-codex-spark` しか記載しておらず「無いIDを推測しない」と定めるため、実体から取った。
  **runbook §7 の表を更新する余地がある**
- **base worktree を local main `9b2deac4` に確定**。b1が引く `product_runtime.rs:3326` は
  main と背骨branch で同一行であり、b1の対象（media / doc / transport / cli）は背骨4粒が触っていない
- prompt 2本を scratchpad へ用意（session固有なので失われる。必要なら書き直す）
  - b1 鎖のgate: 検査4点、出力6欄（`ERRORS` / `SEAM_BLOCKED` / `OVER_UNKNOWN` /
    `FORBIDDEN_BUT_PRESENT` / `DRAFT_VERDICT` / `EVIDENCE_GAP`）、鎖本文を prompt へ内包
  - `N-SOUNDTRACK-WRITE` preflight: `AGENTS.md` の preflight 欄すべて＋`CURRENT FACTS`＋`OPEN QUESTIONS`

### 独立性について

b1/b2 を含む7区間の起草は、保全テキストに `ExitPlanMode` / `Write` / 「プランモード」が
繰り返し現れることから **Anthropic系である可能性が高い**（launch記録は失われている）。
器具境界決定 §6.45 は起草側と検査側を同一familyにしないと定めるため、**gateは非Anthropicが要る**。

## 5. 出しているチップ

`Inspectorを設計する — Timelineが送った責務の受け皿`（task_3034ef4d、未着手）。

本日「タイムラインは時間の操作へ集約する」と決めた結果、値・M/S・エフェクト・ブレンド・
クリッピングが**すべてInspectorへ送られた**。載るか、縦に長くなりすぎないかは未検証で、
破綻するとTimeline側の決定も連鎖で崩れる。**不要ならdismissしてよい。**

## 6. 未決

- 合成順（`Vec<TrackItem>` の入れ子）を見せる面はどこか。**Depth Rail は既決だが未描画**
- 非表示だが依存先として評価される参照元のミュート表現（`visible` の意味論）
- `lock` は本日の設計で一度も扱っていない
- marker 10px / 13px、音声1行だけ高さを許すか
- 前日から継続: `ABSENT` 11件中9件が外部未確認、継ぎ目9件、区間内側14件、休止契約、C0-Schema

## 7. 非目標（本sessionで守った線）

- root の stage / commit / push / main統合
- 背骨branch・skia branch の local main 統合、remote push
- 外部LLMの起動（準備のみ）
- `crates/` への skia fixture 持ち込み
- 未確認の `ABSENT` を推測で訂正すること
- 設計決定を実装許可として扱うこと
