# M2終了前判定 — Param Pipelineと操作単純化の持ち越し境界

日付: 2026-07-14

状態: **M2終了判定。実装仕様ではない。PP-Gateの反対側レビューは別途必須**

歴史注記（2026-07-23）: cutoff版は[Unit 4N回収](2026-07-23-historical-param-element-constraint-lineage-recovery.md)で処分済み。現行`DocParam`は本判定どおり6 sourceのままで、PP-Gateは未発火・未実装。後続のElement Domain／Constraint Graph境界と合わせ、M2 task IDはglobal backlogの同名IDとの衝突を避けて`M2-GAP-15`と表記する。

関連: [操作単純化モデル](../interaction-simplicity-model.md)、[凍結ゲート宣言](2026-07-10-freeze-gate-declaration.md)、[4ツール監査](2026-07-14-motion-tools-praise-diy-gap-audit.md)

## 1. 結論

Autograph型の`Generator → Modifier[] → Result`は有力だが、**M2の完了ブロッカーにはしない**。Wave4へ割り込ませず、M2を現行`DocParam`意味で閉じる。

同時に、Param Pipelineを「v2候補」「いつか検討」へ落とさない。**M3の高度property UI、常設Relative Offset、汎用parameter pluginのいずれかへ着手する前に通す、M1/M2横断の解凍ゲート**として固定する。

当面の意味は次の通り。

| 項目 | M2終了時の判定 |
|---|---|
| Const / Keyframes / Data / Vec2Axes | 現行意味を維持 |
| LookAt / Follow | M2の型付き参照として維持。文字列expressionへ戻さない |
| Relative Move | 選択keyへ同じ差分を適用するD2 macro。常設後段offsetとは呼ばない |
| Generator / Modifier列 | 未実装。既存`DocParam`へ推測で追加しない |
| parameter plugin | 現行ParamDriver/DataTrackの範囲。任意Modifier APIはPP-Gate待ち |
| Advanced UI | 現行の値の由来を検査できる範囲から始める。未実装pipelineを先にUI化しない |

## 2. なぜ今すぐ焼かないか

### 2.1 現行M2は「出所」、Autographは「評価列」で意味の形が違う

現行`DocParam`は次の閉じた出所選択である。

```rust
Const
Keyframes
Data
Vec2Axes
LookAt
Follow
```

一方、候補pipelineは複数段を合成する。

```text
Base → Link/Driver → Modifier[] → Result
```

これはvariantを1個足すだけではない。順序、型変換、循環、Canvas逆変換、cache invalidation、未知Modifier保持まで同時に決める必要がある。

### 2.2 M1凍結面にも触る

Documentだけを変えても、評価層`ParamSource`は`Const / Keyframes / Data / Vec2Axes`の単一出所である。Modifier列を実動させるには、凍結ゲート項目4の`ParamEval`と、場合により項目2のparameter plugin境界を解凍する。

[凍結宣言](2026-07-10-freeze-gate-declaration.md)どおり、変更理由と実証、migration、影響goldenの3点なしにWave4へ混ぜられない。

### 2.3 Wave4の責務と独立している

- [PR #151](https://github.com/oshikaidesu/Motolii/pull/151): 固定比resample。永続schema非接触。
- [PR #150](https://github.com/oshikaidesu/Motolii/pull/150): soundtrack muxと未知plugin時のexport拒否。
- [PR #149](https://github.com/oshikaidesu/Motolii/pull/149): clipping maskのdoc→graph→GPU意味論。

いずれもParam Pipelineの形を決める証拠ではない。ここへ混ぜると1 ticket=1 commit/PRを破り、docs header競合にschema判断まで重ねる。

## 3. 「M2後へ送る」と「先送り」の違い

本判定は無期限延期ではない。次の**発火条件**を置く。

### PP-Gate発火条件

次のどれか1つを起票する前に、PP-Gateを開始する。

1. key済みparameterへ常設offsetを保存する。
2. 1 parameterへDataTrackと手動補正を同時適用する。
3. Generator/Modifierをuser plugin種別として公開する。
4. Advanced property UIへ評価列の並べ替えを出す。
5. Add/Multiply/Clamp/Remap等を複数parameter共通の後段処理として標準化する。

M3の通常parameter panelは、現行`DocParam`の出所表示と編集だけならPP-Gate前に着手できる。

### PP-Gateの完了条件

[操作単純化モデルのPP-1〜PP-6](../interaction-simplicity-model.md#4-param-pipeline-gatepp-gate)に加え、次を要求する。

- **小さい代替との比較**: D2 key差分、Transform専用offset、presetだけでは不足する実例を最低3件。
- **正準形**: nested Modified、空Modifier列、同型identity opを保存上どう扱うか。
- **型表**: scalar/Vec2/Color/Bool/Asset等、各operationの受理型と拒否型。
- **逆操作**: Modifier有効中にCanvas handleをdragした時、BaseとResultのどちらを編集するか。
- **参照順**: LookAt/Follow/DataTrackとModifierの前後関係、循環診断。
- **可搬性**: 未知plugin Modifierを保持して開けることと、再現不能export拒否を分離。
- **追加的移行**: 既存`DocParam` JSONを無変換で読める追加variant、または明示migration。
- **意味論golden**: 既存projectの画が不変、新pipeline fixtureのpreview/export一致。
- **cache変異**: Modifier追加・削除・並べ替え・version変更でK1/K2が正しく無効化。
- **反対側レビュー**: 任意node graph化、順序例外、UI認知負荷、plugin ABI拡大を再判定。

## 4. M2終了時に確定するHost責務

Param Pipelineの有無に関係なく、次はM2で確定してよい。

1. **D2 command**: 1 gesture=1 history、Cancel=変更ゼロ、決定済みdomain値を記録。
2. **型付き参照**: LookAt/Follow/DataTrackをIDと期待型で検査。
3. **単一writer**: plugin/UIはDocumentを直接変更しない。
4. **欠落plugin**: 開く時は未知データ保持+警告、再現不能exportはtyped error。
5. **評価順**: UI入口に依存せず、同じDocumentは同じrender graphになる。
6. **Relative Moveの正直な表示**: keyを書き換えるone-shot操作と、常設offsetを混同しない。

この6点があればM3はDirect/Tool UIを進められ、後からPP-Gateを通しても操作入口を捨てずに済む。

## 5. Wave4マージとM2終了の順序

推奨順:

1. PR #151 D4-FU
2. PR #150 D6
3. PR #149 D7
4. main上でdocs task headerを一度だけ統合
5. M2残件と全workspace testを再監査
6. 本判定をM2終了記録から参照

順序はコード依存ではなく、各PRが自タスク完了を同じdocs headerへ書く競合を小さくするためである。別順でmergeしても、最後のPRで他タスクの完了表記を消さないこと。

## 6. M2終了判定

M2は次を理由に開け続けない。

- 将来Modifierが欲しくなる可能性。
- Advanced UIの最終形が未完成。
- 汎用Element DomainやConstraint Graphが未決。
- plugin市場で新しい表現が生まれる可能性。

これらはM2の恒久面を広げる根拠ではない。

一方、次が残っていればM2は閉じない。

- 仕様表の既存M2 taskが未完了。
- D1fの未知plugin保持とD6のexport拒否が結合していない。
- D2 command/Undo、D3評価順、D1i意味論goldenが完了していない。
- PR mergeでtask statusが巻き戻った。
- workspace testまたは保護golden policyが赤い。

## 7. 最終判定

> M2は「将来の全parameter表現を焼き切るフェーズ」ではない。現在証明されたDocument意味、Undo、参照、可搬性、評価順を閉じるフェーズである。

Param Pipelineは価値が高いからこそ、Wave4のついでに入れない。M2終了後、M3で最初の常設補正または高度property UIが必要になる直前に、独立した解凍PRとして決着させる。
