# M3タスク翻訳: Text Motion(Live Text)縦切り第1弾(2026-07-19)

日付: 2026-07-19
状態: **条件付き実装発注の正本候補**。発注解禁条件: (1) `codex/m3-entry`(M3=段階発注可)のmain統合、(2) 各枝番の依存到達、(3) ユーザーの発注指示。意味の正本は[リリック比較台帳](2026-07-19-lyric-motion-text-sequence-comparison.md)(**比較中のまま変更しない**)と[text-model.md](../text-model.md)(ドラフト)であり、本書はそのうち恒久スキーマへ触れない範囲だけを実装粒へ翻訳する。

## 1. 縦切り第1弾の範囲(ユーザー確定 2026-07-19)

```text
Text 1 Object
+ 標準Random Entrance
+ 展開可能なCharacter Score
+ 文字選択同期
+ timingはread-only投影
```

含めない: 個別override書き戻し、本文編集後の和解の製品化、Ghost Pose編集、Detach / Materialize、第三者Animator Vism、3D、複数animator合成UI、プリセット群、CONTROL/RESULT二段UIの確定。

**なぜ比較台帳が「比較中」のまま発注できるか**: 第1弾は台帳§3.2/§3.3の恒久スキーマ候補(override store・安定書記素ID・Needs Review状態)へ一切触れない。Documentへ保存されるのはText pluginの通常params(本文・style・Sequence規則)だけで、Character Scoreは評価結果のread-only投影、文字選択は評価時の一時identityで足りる。したがって**Identity/Reconcile gate(台帳§4.1 Gate 1)の通過を待たずに発注でき**、方式比較がどちらへ転んでも捨てるのはUI投影だけで済む。逆に、台帳の未決へ触れる書き戻し系(TM-5以降)は該当gate通過まで発注できない。

## 2. M3タスク表へ追加するTMレーン(codex/m3-entry統合後に転記する行)

| ID | 内容 | 依存 | 完了条件(概要) |
|---|---|---|---|
| TM-1 | first-party Text plugin第1号(本文+font+size+fill、横書き・単一style) | M5-P6(shape+fallback+cluster対応表+`draw_glyphs`)、U4a | 混在文字golden、自動panelから全params編集可能、Document Object数はText 1個のまま |
| TM-2 | Text Sequence評価: 標準Random Entrance 1本 | TM-1 | text-model §2の`{selector, properties}`形をplugin paramsとして保存し、L0閉形式純関数で評価。同一seed同一結果 |
| TM-3 | Character Score: 評価済み文字時刻のread-only投影+明示展開/折り畳み | TM-2, U3a | collapsed=Text 1行、展開は明示scope、Document項目増0、編集不可 |
| TM-4 | Stage文字選択同期 | TM-2, U1a, U2c | Stage / Score / Inspectorが同一の一時identityを投影。選択はTransientでDocument/Undo不変 |

後続レーン(**登録のみ・発注不可**。依存gateは[比較台帳](2026-07-19-lyric-motion-text-sequence-comparison.md) §4.1):

| ID | 内容 | 解禁条件 |
|---|---|---|
| TM-5 | 文字別timing/layout overrideの書き戻し(Sequence Timing Lane編集) | Gate 1(Identity/Reconcile)+Gate 2(Evaluation ownership)通過、override保存形式のtext-model/M2審判 |
| TM-6 | Detach(全体Detach先行、G4契約) | Gate 4通過。部分Detachは別実験 |
| TM-7 | 第三者Animator開放(Effector評価形への合流) | M5-P0I spike、[vism-package-concept §11](../vism-package-concept.md)停止線 |
| TM-8 | 3D(Transform3D delta / Spatial Score channels) | M5-P6契約とtext-model propertiesの正式拡張(台帳§9) |

## 3. 枝番(1 Issue = 1 commit)

| ID | 内容 | 依存 | 完了条件 | 非目標/拒否条件 |
|---|---|---|---|---|
| TM-1a | Text plugin第1号: 本文/font/size/fillのparams、P6経由shape→cluster対応表→`draw_glyphs`、正準座標配置 | M5-P6実装到達、U4a | (1)かな漢字英混在のshape→drawゴールデン (2)cluster対応表から「N文字目のグリフ範囲」取得 (3)全paramsが自動panelから編集可能で1 gesture=1 Undo (4)save/reload/preview/export同一 (5)Document Object数はText 1個 | style_spans、行組の一般化、縦書き、ルビ、animator、独自UI panelを入れない |
| TM-2a | Sequence評価: selector(矩形0/1+順序Forward/Random+明示seed)×Opacity/Position登場をL0閉形式で評価 | TM-1a | (1)同一seed・同一入力で再起動/preview/export一致 (2)時刻sampleゴールデン (3)行組はアニメーションの影響を受けない(text-model二層分離) (4)paramsは既存ParamSource/plugin paramsのみでDocument新field 0 | 新しいevaluator機構、複数animator合成UI、easing種追加、前状態依存、override |
| TM-3a | Character Score read-only投影: 折り畳み時Text 1行、明示展開で評価済み文字開始時刻をnode表示、本文/Interval/seed変更でライブ更新 | TM-2a, U3a | (1)collapsed=1行、展開行は明示したText scopeのみ (2)node数=表示中クラスタ数でDocument項目増0 (3)展開/折り畳み/packingでDocument snapshot不変 (4)nodeはdrag不可・選択のみ | 保存されるlane/Track、nodeドラッグや`Distribute`等の編集、CONTROL/RESULT二段の確定(台帳fixture 9の審判待ち)、1項目1横行の常設化(ui-score-model不変条件) |
| TM-4a | 文字選択同期: Stage hit-testで文字候補を選択し、同一の一時identityをStage/Score/Inspectorへ投影 | TM-2a, U1a-1, U2c | (1)3面が同一対象を強調 (2)選択はTransientでDocument/Undo/serialize不変 (3)同一本文内の再評価で選択が別文字へ移らない | marquee/複数選択の高度化、選択の永続化、介入、Ghost Pose |

DECIDE(発注前に要る小審判、実装粒と混ぜない):

- **TM-D1**: text-model §2の第1弾使用範囲(selector最小形+properties+クラスタ単位)をドラフトから「確定」へ上げる審判1枚。text-modelは凍結ゲート未通過の公約化禁止ドラフトのため、この範囲確定なしにTM-2aを発注しない
- **TM-D2**: M5-P6の発注状況確認。P6はM5レーンの管轄であり、TMから重複発注しない(依存として待つ)

## 4. 停止線(第1弾で変更しないもの)

- override store・安定書記素ID・Needs Review状態・Auto/Offset/Pinned等のDocument field追加(Gate 1通過前提の全て)
- 文字のObject化・per-char Timeline row・保存されるlane
- 公開`Element`/`EffectorPlugin`/capability trait、evaluated-domain公開面(P0I前)
- Timing Rail(R1一回性Tool)との名称・意味の混同(台帳§2.4)
- 台帳の比較中ステータス自体(本書は台帳の審判を代行しない)

## 5. 発注順序

```text
codex/m3-entry main統合
    ↓
U0b-1 / U0e-1 / U1a-1(egui基本面 — 既存の並列レーンどおり)
    ↓                          (並行: M5-P6発注、Gate 1残項目継続)
TM-1a(P6+U4a到達後)
    ↓
TM-2a
    ↓
TM-3a / TM-4a(並列可)
    ↓
TM-5以降はgate通過待ち(発注不可のまま)
```

M3仕様タスク表へのTM行転記と並列レーン文の更新は、`codex/m3-entry`統合後の別コミットで行う(本書§2がその文面の正本)。
