# リポジトリ外資産の棚卸しと判定訂正 — surveyの適用範囲の限界

日付: 2026-08-08
状態: **観察 / 判定訂正を含む**

## 1. なぜ必要か

2026-08-07に62項目のnode surveyを実施し「**本当に存在しないのは11件**」と結論した。
これは正確には「**local main `9b2deac4` のtreeに存在しないのは11件**」であった。

**node surveyも仮コードも、リポジトリの中しか見ない。**
隔離probe、benchmark、別work directoryにある成果は原理的に見えない。

同日の対話で、リポジトリ外に**製品へ入りうる資産が複数実在する**ことが判明し、
survey判定のうち少なくとも1件が誤り、1件が含意の誤りであった。

## 2. 確認したリポジトリ外資産

いずれも `~/Documents/Codex/` 配下。**git history、全branch、worktree一覧のどこにも存在しない。**

| 資産 | 場所 | 内容 |
|---|---|---|
| **MotoliiRnProbe** | `2026-08-06/ui-rust-ui-c-react/work/` | RN製品UI再現。`App.tsx` 660行。Browser 3タブ(`MEDIA`/`EFFECTS`/`CREATE`)、Inspector/Extensions、Timeline 3モード、effect一覧、panel registry。native `MotoliiGpuComponentView.mm`、Fabric spec `MotoliiGpuView`/`MotoliiTimelineView` |
| **skia-timeline-probe** | `2026-08-06/motolii-ui-hybrid-research-handoff/work/` | **`skia-safe 0.99.0` + `wgpu 29` + `winit 0.30.9` が実動** |
| **windows-skia-target-check** | 同上 | Windows target でのrust-skia/wgpu確認 |
| **StagePresentProbe.app** | 同上 | ビルド済み `.app`（binary `stage_present_interactive`）。**sourceの所在不明** |
| renderer選定の比較群 | `2026-08-06/ui-rust-ui-c-react/work/` | `QtTimelineProbe` / `BareQSGTimelineProbe` / `QSkinnyTimelineProbe` / `qt-density-probe` / `qt-motolii-probe` / `qt-react-hybrid-probe` / `avalonia-density-probe` / `avalonia-actipro-compile-probe` / `cxx-qt-v0.9.1` / `qskinny-*` |
| 選定調査文書 | 同上 | `fable-*.md` 10本以上（hybrid UI research、zero-bias prompt、density probe review、RN+C++/Rust architecture 等） |
| rn-preview | `2026-08-06/motolii-ui-hybrid-research-handoff/work/` | Vite ベースのpreview |

`2026-08-07/skia-3d/` は `work/` `outputs/` とも**空**（着手前）。

## 3. 判定訂正

### 3.1 `N-OVERLAY` — `ABSENT` → `PROBE_ONLY`（**訂正**）

[成果駆動統合地図](../outcome-driven-integration-map.md)§4は
「rust-skiaは`Cargo.toml`に存在しない」ことから`N-OVERLAY`を`ABSENT`とし、
次手を「既知実装調査 → 採択 → 実装」と定めた。

**これは誤りである。** `skia-timeline-probe` で `skia-safe 0.99.0` + `wgpu 29` が**実動している**。
再基線決定が標準と定めた組み合わせそのものであり、Windows target checkも実施済みである。

正しい状態は `PROBE_ONLY`（隔離検証済み・製品未接続）で、
次手は**既知実装調査ではなく移管・接続**である。

### 3.2 `R1-BROWSER` — 判定は正しいが含意が誤り

survey判定`ABSENT`は**製品routeについて正しい**（`ui/motolii-rn/App.tsx`のBrowserは`<Text>` placeholder）。
しかし含意していた「だから構築が必要」は誤りで、**MotoliiRnProbeに660行の実体がある**。

移管routeは既決である。
[React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)は
`状態: 決定 / 発注停止線` で、「対象componentごとに表を実コードから作り、
**空欄または推測が一つでもあればHost接続を発注しない**」と定めている。
**移管は決まっていて、明示再開待ちで凍結されている。**

### 3.3 `N-GIZMO-SURVEY` — 変わらず

`StagePresentProbe.app` はビルド済みバイナリのみでsourceが確認できず、
gizmo実装の有無を判定できない。**未確認**のまま維持する。

## 4. 方法論の限界（記録）

`ABSENT` を「既知実装調査 → 採択 → 実装」の入口と定めた
[成果駆動統合地図](../outcome-driven-integration-map.md)§6の原則は、
**リポジトリ外に実在資産がある場合に二重開発を発注する。**

したがって次を運用へ加える。

> **`ABSENT` と判定する前に、リポジトリ外の隔離成果を確認する。**
> 確認範囲（探した場所）を判定へ併記する。確認していない場合は `ABSENT` と書かず
> `UNKNOWN_OUTSIDE_REPO` とする。

本日の`ABSENT` 11件のうち、外部確認を経たのは`N-OVERLAY`と`R1-BROWSER`の2件のみである。
**残り9件は未確認であり、同種の訂正が追加で生じうる。**

## 5. 経緯（発見方法として記録）

`MotoliiRnProbe`は利用者が**意図的にsurvey範囲から外していた**。
「Codexが既存モックUIを再現できるか」を測るbenchmarkとして隔離していたためである。
結果は「99%ほど再現できた」であり、
[再基線決定](2026-08-07-m3-react-native-rust-skia-runtime-rebaseline.md)§2-1の採択理由
（既存React mockをRN componentへ移しやすい）は**実証された**。

ただし目隠しが意図的でなくても同じ穴は開く。本文書§4はそのための手順である。

## 6. 非目標

- 本文書を根拠に移管を発注すること（React資産移管契約は停止線のまま）
- リポジトリ外資産を製品成果として数えること（`ui-artifact-terminology.md`のspike規定に従う）
- 未確認の`ABSENT` 9件を推測で訂正すること
- `StagePresentProbe`の中身を推測すること
