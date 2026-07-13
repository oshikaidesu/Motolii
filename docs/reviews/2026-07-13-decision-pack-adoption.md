# 決定パック採択(2026-07-13ユーザー承認)

ステータス: **【決定】**。発明ではなく既存規約の採択。以後の追加広範調査は不要。例外は実装中に既存仕様同士の矛盾が実発見された場合のみ。

出典の方針: AE/Lottie・OTIO・DAW・Qtの公開規約を正とする。Motolii独自最適解の探索はしない。

関連: [#103](https://github.com/oshikaidesu/Motolii/issues/103) / [#100](https://github.com/oshikaidesu/Motolii/issues/100) / [M2仕様](../specs/M2-document-model.md) / [確認メモ](2026-07-13-undecided-critical-path-confirm.md)

## #103 9項目

| ID | 決定 | 採択元 |
|---|---|---|
| A4 | 同一Track内のクリップ重なりは**原則禁止**(validate)。重なりは別Track、または将来の明示的Transitionで表現 | [OTIO timeline](https://opentimelineio.readthedocs.io/en/latest/)(同一Track=順次、重なり=Transition) |
| A6 | 音源開始とは**独立**のTempo/Meter map。`beat_origin=0`、先頭拍子は4/4。`Soundtrack.start_offset`とは分離 | Ardour Tempo Map / Cubase Signature Track |
| A8 | Layerに加え Effect・Keyframe 等の操作対象すべてに、不変・非再利用の document-local `u64` ID。並べ替えで維持、複製時は新規採番+サブツリー内再写像 | M2E-15の拡張(D2前提) |
| B① | 同一プロパティ・同一時刻のKeyframeは**1個だけ**。追加は既存の更新・置換 | Adobe keyframes |
| B④ | 下表(3軸) | Adobe layers |
| B⑤ | 未知BlendModeは**Deserializeエラー**。閉集合コアenum。`Normal`代替も`Unknown(String)`も採らない。追加時はschema/`min_reader_version`上げ | Lottie schema / F-9はプラグイン拡張点の話であり本enumは対象外と明文化 |
| B⑦ | `Composition.fps`=編集表示・スナップ・標準出力fps。内部時刻は`RationalTime`。`ExportJob`のみ明示fps上書き可 | Lottie composition / Adobe rendering |
| B⑧ | bool / enum / AssetRef 等の離散型は**Holdのみ**。線形・Bezier補間禁止 | Lottie keyframe schema |
| ⑨ | モデルはプロパティ単位のatomic command。ユーザー1 gesture=1 macro。ドラッグ中の同一対象・同一プロパティはmerge。選択・hover・IME中間はUI状態のまま | Qt Undo Framework |

### B④ 3軸表(visible / solo / lock)

| フラグ | 描画 | 評価(依存先として) | 書き出し | 編集 |
|---|---|---|---|---|
| `visible=false` | 自身は描画除外 | **評価可能**(parent / mask / LookAt 対象になり得る) | 自身は含めない(依存経由の評価結果は別) | 可 |
| `solo` | 描画対象フィルタ(ソロ集合のみ描画) | 評価は通常どおり | 描画フィルタに従う | 可 |
| `lock=true` | 影響なし | 影響なし | 影響なし | **禁止のみ** |

## PathOp(#100 / D1i-2)【決定】

正本はLottie/AE準拠。Cavalryの厚い角はv1に焼かない。詳細表は[M2仕様 PathOp意味論表](../specs/M2-document-model.md)。要約:

| op | 固定 |
|---|---|
| `pucker_bloat` | Lottie百分率意味を`[-1,1]`へ正規化 |
| `zig_zag` | `amount` / `ridges` / `point_type=corner\|smooth`。Wave派生は入れない |
| `offset` | `distance` + `line_join` + `miter_limit`。v1は**閉路限定**、開路は型付き unsupported |
| `round_corners` | 通常fillet、`radius >= 0` |
| `trim` | **幾何**modifier。`start`/`end`、循環`offset`、`parallel`/`sequential` |
| `twist` | `angle` + **必須** `center: Vec2` |
| `repeater` | 整数`copies`、fractional `offset`、完全`Transform2D`、composite順、開始・終了opacity |
| 退化 | 空→空、1頂点以下→identity、NaN/Inf拒否、自己交差を勝手に修復しない |
| `wiggle` | 相互運用乱数は無い → 再現性のための実装定数として **PCG32-based value noise + u64 seed** を仕様名付きで固定([PCG](https://www.pcg-random.org/paper.html)) |

## 残小項目【決定】

| 項目 | 決定 | 採択元 |
|---|---|---|
| Undo深さ | live と再起動後を**別limit**。既定は Qt と同じ **0 = unlimited**。数値100等を仕様上の真理にしない。snapshot/compactionは保存実装の問題 | Qt QUndoStack / Ardour preferences |
| ExportJob | Document外。snapshot/ref・出力先・範囲・fps override・解像度・codec/container・音声mux | 一般的な Render Queue |
| Group時間 | Group=folder/transform container の限り **retimeを持たせない**。必要なら Group を肥大化せず明示的 `CompositionClip`/precomp 型を追加 | OTIO / AE |
| audio gain/offset | 現行 `Soundtrack.start_offset` + master gain で足りる。追加調査不要 | 既存スキーマ追認 |

## 以後の進め方

- M1: 維持作業
- M2: 実装依頼と受入テスト中心(`#100` D1i-2、`#109` D2は `#99`+本決定、`#110` D3は `#100`+第二凍結点+本決定)
- 追加の広範な先行事例調査は**不要**
- 例外: 実装中に既存仕様同士の矛盾が実発見された場合のみ仕様改訂PR
