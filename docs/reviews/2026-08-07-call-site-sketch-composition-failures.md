# 仮コード合成テストの検出結果 — 決定間の合成失敗14件

日付: 2026-08-07
状態: **観察 / 修理許可ではない**

## 0. 扱い

`AGENTS.md`「findingは権限ではない」に従い、**報告と分類だけ**を行う。
本文書を根拠に発注・実装・決定改訂を行わない。

## 1. 方法

[仮コード器具](2026-08-07-provisional-call-site-sketch-instrument-decision.md)により、
[creator-developer連続体決定](2026-07-22-creator-developer-continuum-decision.md)が定める
`Use → Tune → Compose → Inspect → Fork → Author → Publish → Reuse`の呼び出し鎖を
4区間へ分けて起草し、**全決定が1本の鎖で同時に成立するか**を検査した。

`decision-index`は234の`決定`を持つが**主題からの逆引き**であり、
387本のreview文書は別々の時期・レビューで書かれている。
**同時成立の検査は本日が初回である。**

検出対象: 矛盾 / 断絶 / 孤児 / 順序不能。

## 2. 完成条件を塞ぐ 3件（Tune / Compose）

[成果駆動統合地図](../outcome-driven-integration-map.md)§5.9へ node として反映済み。本文書では再掲しない。

## 3. 作者・配布枝 11件

`concept.md`の完成条件（3〜5分MVを音楽同期で書き出す）を塞がない。
`concept.md`は解析駆動生成を「優先度=最終フェーズ、M1〜M5完成後」と定めており、
**後置は既決の踏襲である。**

### 3.1 Inspect / Fork（4件）

| 種別 | 内容 |
|---|---|
| 断絶 | A4Sのfork入口はCLI固定引数（`--from core.layer_source.radial_repeater`）でDocument selectionを受けない。「選択中Vismをforkする」経路が無い |
| 断絶 | 決定49は「作用先、typed input/output、space、temporal mode、diagnosticをHost契約から投影」を要求するが、実装`InspectorSnapshot`（`inspector_host_runtime.rs:1063-1073`）は`target{...}`止まり |
| 断絶 | 決定45（作者連続性）は「**任意の選択中Vism**へ一般的に適用される経路」だが、A4S §1は「first-party参照Vismを別crateへfork」に限定 |
| 順序不能 | Fork/atomic adoptionはInspect時点の開始revisionとライブselectionを前提にするが、A4S §4 step5-6のadoptionはcomposition root登録＋**rebuild/restart**を伴う。「変更カプセルは作品・identity・Previewを失わない」と同時に成立しない |

### 3.2 Author（3件）

| 種別 | 内容 |
|---|---|
| 断絶 | 明示capabilityが、片方はoperation粒度（fixtureの文字列のみ、Rust型なし）、片方はVism instance粒度（grant/revocation）。両者を繋ぐ決定が索引にも正本にも無い |
| 断絶（コード実証） | `PluginRuntime::try_new`のkind検査ループが`Simulation` / `ScriptWasm`を**素通りする**。Hostに要求されている契約検証を、予約された唯一の受け皿enumが実行時に受けていない |
| 順序不能 | 「Hostが型付き入力を渡して評価する」の呼び出し側は、`LANG-TS-F0 = WAIT`により**評価対象のruntime自体が実在しない** |

### 3.3 Publish / Reuse（4件）

| 種別 | 内容 |
|---|---|
| 断絶 | Vism packageは「安定した表現identityとversion」を要求するが、実在するidentity型は`NodeDesc.id: PluginId(&'static str)`で**compile-time静的登録専用** |
| 断絶 | community-distribution-modelは「catalog entryをruntime registryへ直接登録する」ことを停止線として禁じるが、実在するruntime registryは`PluginRegistry`のみで`register`はcompile-time`PluginContract`しか受理しない。**Reuseの最終段が着地する先が無い** |
| 断絶 | GAP-3停止線（`Asset.content_hash`にversion/algorithm/collision照合のauthorityが無い）と、Kit/Projectが「declared asset referencesで再現する」責任を持つことが噛み合わない |
| 順序不能 | Kit導入は「全体成功時だけ1 macro commit」を要求するが、`DocumentWriter::apply_command`は逐次適用のみでbatch rollbackを提供しない。**正本自身がこれを明記している** |

### 3.4 収束

Author 3件とPublish/Reuse 4件の**計7件は同一の欠落へ収束する**。

> **runtime identityとinstallation pathが存在せず、実在するのはcompile-time静的登録のみ。**

個々は`未決` / `停止線` / `WAIT`として散在して記録されていた。
1本の鎖に並べて初めて、**同じ欠落が7つの決定を同時に止めている**ことが可視化された。

## 4. 起草runの品質記録

4区間中2区間が主担当のharness設定により劣化した。

- **Author区間**: 最終フォーマットを出さず「3件で進めてよいか」と確認して停止 → `PARTIAL`
- **Fork区間**: `--permission-mode plan`で起動しながら`ExitPlanMode`/`Write`を禁じたため、
  成果物の保存先が失われた。内容は生streamから回収したが正規の返却ではない

plan modeとtool禁止の組合せが噛み合っていない。次回はplan modeを使わないか、
「最終テキストへ全て書く」を明示する。

## 5. 非目標

- 本文書を根拠に修理・発注・決定改訂を行うこと
- 作者・配布枝11件の先行着手
- 7件の収束先（runtime identity / installation path）をM3で新設すること
- 未検証の疑いを確定した欠陥として外向きに扱うこと
