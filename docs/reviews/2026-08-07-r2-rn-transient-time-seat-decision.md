# R2-RN-TIME-SEAT — RN routeのtransient評価時刻席の縮小採用

日付: 2026-08-07
状態: **縮小採用 / 実装未着手**

## 1. この決定が閉じる一問と、閉じない一問

閉じる:

> RN routeで「今どの時刻を見ているか」を、Document外のtransient stateとしてどこが持ち、
> どう一方向publishするか。

**閉じない（重要）**:

`R2-FOCUS-PLAYHEAD-AUTHORITY` は `TARGET_MISSING` のまま**維持する**。
実行地図は当該nodeのownerを「essential focus／playhead consumer、gesture epoch、ProjectSession／Transient」と
置いており、本決定はそれを特定も充足もしない。本決定は背骨に必要な**最小の時刻席**だけを置く`REDUCE`であり、
focus、gesture epoch、scrub、playback、Timeline navigation、ProjectSession由来のplayhead authorityは未決のままとする。

## 2. なぜ今必要か（背骨上の実在gap）

- `project_stage_geometry(document, eval: EvaluationTime, tracks)`（`R2-STAGE-GEOMETRY-READ`成果）は
  評価時刻を引数に取る。hit-testもgizmoもこれを消費する
- `SetPositionKeyValueRequest { target, key: KeyframeId, old, new }` は**既存KeyframeIdを要求**する。
  どのkeyかは時刻が決めるため、時刻なしでは背骨が閉じない
- しかし `RnProductHost` は時刻を一つも持たない。`RationalTime` の出現は `mod tests` 内の2箇所のみ
- Stageは `prepare_in_setup_worker` + `bootstrap_frame_desc()` で描いており、評価時刻の概念自体がない

時刻を持たないまま実装すると、実装担当が `RationalTime::ZERO` を暗黙の既定として焼き込む。
それは意味の発明であり、後から所在が分からなくなる。

## 3. 決定

### 3.1 RN Hostがtransient評価時刻を一つ持つ

`RnProductHost` が `RationalTime` のtransient現在時刻を**一つだけ**持つ。
Document、serde、journal、revisionへは入れない。

### 3.2 既存 `EditorPlayhead` を型として流用しない

`product_runtime.rs:222` の `EditorPlayhead { current, scrub: Option<PlayheadScrub> }` は
旧routeのlocal holderであり、実行地図が置き換えたい consumer-local owner の側に属する。
v1で必要なのは `current` 相当だけで、`scrub` は本決定が実装しないgesture概念である。

したがって**旧型をRN routeへ持ち込まず**、旧型の移動・公開・共通化も行わない
（`product_runtime.rs` を触らない）。将来 `R2-FOCUS-PLAYHEAD-AUTHORITY` が閉じた時点で、
双方を正式ownerへ寄せる。この一時的な二箇所併存は**本決定が明示的に受け入れる負債**であり、
`R2-FOCUS-PLAYHEAD-AUTHORITY` の未決状態がその返済期限である。

### 3.3 一方向publish

snapshotへ現在時刻を載せ、read-onlyで配る。RN側が正本を持たない。
時刻の変更はtyped intentだけが行い、accepted変更のときだけ `projection_generation` を進める。

### 3.4 v1の範囲

- できること: 現在時刻を明示intentで設定する。snapshotから読む
- しないこと: playback、transport、scrub gesture、Timelineからのseek、自動進行、frame stepping
- 境界外時刻（負、composition duration超過）はtyped拒否とし、暗黙clampしない

## 4. positive / negative oracle

positive:

1. 初期時刻が `RationalTime::ZERO` で、snapshotに載る
2. 有効時刻のintentが受理され、snapshotへ反映される
3. accepted変更で `projection_generation` が進む
4. 同一時刻の再設定がno-opで、generationが進まない

negative:

1. 時刻変更で **Document write 0、revision不変、journal追記0**
2. 負の時刻、duration超過、非有限値がtyped拒否される（暗黙clamp 0）
3. `primary_layer_id` が変化しない
4. `product_runtime.rs` に差分がない（旧`EditorPlayhead`を動かさない）
5. 自動進行・timer・playback loopが新設されていない

## 5. 非目標

- `R2-FOCUS-PLAYHEAD-AUTHORITY` の充足を主張すること
- focus、gesture epoch、selection、scrub、playback、transport
- Timeline投影、Timelineからのseek
- `EditorPlayhead` / `PlayheadScrub` の移動・公開・共通化
- Document schema、公開plugin契約、永続形式への時刻の追加
- Stage描画を時刻連動にすること（live renderは `R1-STAGE-BASE-B` の別境界）
