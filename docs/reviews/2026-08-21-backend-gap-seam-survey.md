# backend 消費ギャップ縫い目調査 — blend/mask/speed(RESEARCH_RETURN 保全)

日付: 2026-08-21 / 発注: 後任セッション(引き継ぎ疑うべきこと#3 の処置)/ レーン: read-only 調査(sonnet)
様式の前例: `2026-08-21-effect-seam-survey.md`。以下、レーンの最終 RETURN を無加工で保全する。
supervisor 注記: (1) speed の「型だけ」claim は部分的に誤りと確定(映像は消費済み・欠落は UI 書き口と音声配線)。引き継ぎ#3 の記述はこの台帳が上書きする。(2) FINDING 1 の stale doc コメント2箇所は supervisor 直の掃除候補。(3) FINDING 4 の「matte の方が mask より安い」は次の発注順序の判断材料。(4) SP2 の前提だった A2 は本調査の返却と同日に着地済み(`f026e7cc`)— 「A2 着地後」の柵は解消済みと読み替える。

---

DONE

台帳(3ギャップ)

**1. blend(14/16値が未消費)**
- store 型: `next/core/motolii-store/src/attrs.rs:18-35`(`BlendMode` 16値、Add無し=Normal〜Luminosityの標準Lottie語彙。裁定67「Add後回し」通り、**store enumにAddすら存在しない**)
- compositor 側の受け皿: `next/engine/motolii-compositor/src/lib.rs:70-78`(`motolii_compositor::BlendMode`はNormal/Addの2値のみ)。実消費点: `lib.rs:369-380`(`multiplicative_tint`のalpha分岐、Normal→opacity、Add→0)— **Add自体はcompositorが無改造で出せる状態だが、store側にAdd変種が無いため誰も選べない**(module doc `lib.rs:31-38`)
- engine 消費点: `next/engine/motolii-engine/src/lib.rs:484-490`(`translate_blend_mode`、Normal以外は全部`EngineError::UnsupportedBlendMode`で**fail-closed**)。呼び出し元 `lib.rs:310`、layer毎に他の処理(texture upload等)より先に判定(早期拒否、`lib.rs:304-310`のコメント)
- 種別: **fail-closed**(黙って近似しない、明示エラーでフレーム全体を止める)

**2. mask(合成器が一切消費しない)**
- store 型: `next/core/motolii-store/src/mask.rs` 全体(`MaskId`/`MaskMode`6値=Add/Subtract/Intersect/Lighten/Darken/Difference/`Mask`/`ResolvedMask`)
- resolve()内での組み立て: `next/core/motolii-store/src/view.rs:589-636`(`resolved_masks`、shape/opacityをproperty trackから読む)、`view.rs:812`で`ResolvedLayer.masks`へ格納。型定義側 `next/core/motolii-store/src/lib.rs:386-390`
- **消費不在点**: `next/engine/motolii-engine/src/lib.rs`の`for layer in resolved`ループ(303-347行)は`layer.matte`(307)・`layer.blend_mode`(310)・`layer.effects`(314)・`layer.source`/`source_frame`/`declared_size`/`placement`/`pinned`は読むが**`layer.masks`を一度も参照しない**。`next/engine`配下を`grep -rn "masks"`しても本体コードに1件もヒットしない(実測)。`motolii_compositor::Layer`(`compositor/src/lib.rs:114-129`)・`LayerWithPasses`(144-147)にも`masks`フィールドが存在しない
- 種別: **黙って無視**(silent)。`EngineError`に対応するvariantが無く、`UnsupportedMatte`のような明示拒否の対称形が存在しない — 非対称

**3. speed(「型だけ」claimは部分的に誤り — 実際は消費されているが、UIから書けない)**
- store 型: `next/core/motolii-store/src/lib.rs:320-359`(`Speed`構造体、`scale_frame_offset`)、`LayerTiming.speed`(260行)
- **映像側は実消費済み**: `LayerTiming::source_frame`(`lib.rs:312-317`)が`speed.scale_frame_offset(offset)`で素材フレームを実際にずらし、`view.rs:743`経由で`ResolvedLayer.source_frame`へ入り、`engine/src/lib.rs`の`texture_for`(431-457行)が**実際に異なる素材フレームを読む** — 絵は変わる。「型だけ」ではない
- **音声側も実消費コードが存在するが未結線**: `next/engine/motolii-audio/src/program.rs:135-150`が`timing.speed.num()/den()`から`TimeMap::constant_speed`を組み実際にmixへ使う。(supervisor 注: A2 着地で `motolii-shell`→`motolii-audio` の依存と再生経路は開通済み — この行の「未結線」は調査時点のスナップショット)
- **真の欠落はUI書き口**: `next/ui/motolii-timeline-pane/src/write.rs`の`Intent::SetTiming`唯一の発行元(342-348行、`finish_drag`)はmove/trimドラッグの`start`/`duration`しか操作せず、`speed`を書く`Message`腕が存在しない。next/ui, next/shell全体で`speed`出現は2件のみで、両方とも`Speed::NORMAL`を組むテスト用フィクスチャ。**ユーザーがSpeedをNORMAL以外にする経路が製品に一切無い**
- 種別: 映像=消費済み・音声=消費コードあり(A2で配線済み・speed付き素材の実機確認は未)・UI=書き口ゼロ(reachability gap)

---

経路比較(侵入コスト・保守面積順)

**speed(最安)**
- (a) EffectPass基盤: 不要 — レンダリング問題ではない。(b) rerun fork: 不要。(c) 旧crates移植元: 不要。(d) 上流: 不要
- 実態: UI動詞の新設(timeline-paneのMessage/write.rs) + 音声再生経路との整合確認。**GPU作業ゼロ**

**blend(中)**
- (a) EffectPass基盤: **乗らない** — `EffectPass`は単一texture入出力のlayer内チェーン。blendは「今まで描いた分(dst)」を読む必要があり、layer単体の情報だけでは閉じない。現行`render_with_timing`(`compositor/lib.rs:337-386`)は全layerを1回の`RectangleDrawData`にまとめて描く一括方式で、途中経過のdstテクスチャという概念自体が存在しない
- (b) rerun fork手術: **必須ではない可能性が高い**(要検証、EVIDENCE_GAP 1) — `effects::glow`前例(独自wgpuパイプラインを`motolii-compositor`内に新設、fork本体を触らない)と同じ手法で新規WGSLパイプラインを足せる見込み。ただし「1layerずつ順に描いてaccumulatorテクスチャへ焼き込む」逐次合成アーキテクチャへの転換が要る(裁定153 S2の「第二render pass禁止との整合」を再びクリアする大きい判断)
- (c) 旧crates移植元: `crates/motolii-nodes/src/composite_blend.wgsl`(53行)+`CompositeNode`。**Normal/Add/Multiplyの3値のみ**。実質Multiply 1個分の式+「2枚読んで1枚出す」構造先例。残り13モードの式は公開仕様から新規(Hue/Saturation/Color/Luminosityの非分離4種はHSL変換が要り数段複雑)
- (d) 上流既製: 無し
- ランキング: 逐次合成の枠(固定コスト)→Multiply(移植)→分離可能残り(均一)→非分離4(最重)

**mask(最重)**
- (a) EffectPass基盤: **乗らない** — mask適用は「layer自身のtexture」×「複数mask形状を畳んだ被覆率」の**2入力**合成。mask形状のラスタライズ自体もEffectPassの範囲外
- (b) rerun fork手術: ラスタライズは不要(下記(c))。最終合成だけ新規GPUパスが要るが、Glow前例に倣い`motolii-compositor`内で完結できる見込み
- (c) 移植元・**重要な訂正**(FINDING 2): `crates/motolii-nodes/src/mask_apply.wgsl`+`MaskNode`の4値は`MatteMode`と一致する語彙 — **mask本体の移植元ではなくmatteの移植元**。`MaskMode`6値のブーリアン被覆代数の移植元は旧crates/に無い。ラスタライズの実移植元は`next/engine/motolii-vector/src/lib.rs:514`の`render()`(tiny-skia、byte決定論、**現在engineから未配線** — engine側コメント`lib.rs:458-462`が自認)。`next/Cargo.toml`のworkspace.dependenciesには登録済み
- (d) 上流既製: 無し
- ランキング: 3ギャップ中最重(ラスタライズ配線・N項被覆代数の新規設計・2入力合成パスの3段が全部要る)

---

切片割り(調査時点では「A2施工中パスに触る切片はA2着地後」の柵つき — supervisor 注: A2 は着地済みにつき柵解消)

### speed(2切片、最軽量)

| # | 切片 | 推定行数 | 判断の重さ | 領域数 | write-set | 依存 | oracle案 |
|---|---|---|---|---|---|---|---|
| SP1 | timeline-paneにspeed編集UI動詞(retimeドラッグ or Inspector数値欄→`Intent::SetTiming`) | 80-120 | 中 | 1 | `next/ui/motolii-timeline-pane/src/write.rs`+`hit.rs` | なし | 「speed編集→`Intent::SetTiming{speed: 2/1}`発行」の赤テスト |
| SP2 | `motolii-audio`再生経路とspeedの整合(speed≠1.0素材の音) | 60-100 | 中 | 2 | `next/shell/motolii-shell/src/*` | A2(済) | speed≠1.0のcompで再生音声のピッチ/長さ回帰 |

### blend(5切片、中量級)

| # | 切片 | 推定行数 | 判断の重さ | 領域数 | write-set | 依存 |
|---|---|---|---|---|---|---|
| BL1 | compositor: 逐次合成(accumulator)経路の枠(no-opで通る形) | 100-140 | **高** | 1 | `compositor/src/lib.rs`+新設`src/blend/mod.rs` | なし |
| BL2 | store: `BlendMode::Add`追加+engine配線(裁定67解消、compositor無改造) | 20-30 | 低 | 2 | `store/src/attrs.rs`、`engine/src/lib.rs` | なし(最速の「まず1個」) |
| BL3 | 分離可能11モードのWGSL+変換表 | 120-160 | 中 | 2 | `compositor/src/blend/separable.wgsl`(新)、`engine/src/lib.rs` | BL1 |
| BL4 | 非分離4モード(HSL変換) | 100-140 | 高 | 1 | `compositor/src/blend/nonseparable.wgsl`(新、BL3と別ファイル) | BL1(BL3と並行可) |
| BL5 | golden回帰(モードごと) | 80-120 | 低 | 1 | `engine/tests/*.rs`(新) | BL2〜BL4 |

### mask(4切片、最重量級)

| # | 切片 | 推定行数 | 判断の重さ | 領域数 | write-set | 依存 |
|---|---|---|---|---|---|---|
| MK1 | engine: `motolii-vector::render`配線(shape→CPUラスタ→GPU texture) | 80-110 | 中 | 1 | `engine/src/lib.rs`、`engine/Cargo.toml` | なし |
| MK2 | `MaskMode`6値のN項被覆代数(移植元なし・CPU側Rasterバイト演算) | 90-130 | **高** | 1 | `engine/src/mask.rs`(新) | MK1 |
| MK3 | compositor: 被覆率×layer本体の2入力合成パス | 90-130 | 高(EVIDENCE_GAP 2) | 1 | `compositor/src/mask/mod.rs`(新) | MK1・MK2 |
| MK4 | golden回帰(mode×invert×opacity) | 80-110 | 低 | 1 | `engine/tests/*.rs`(新、BL5と別ファイル) | MK1〜MK3 |

---

EVIDENCE_GAP

1. **BL1「fork手術不要」は未検証の推測寄り** — 逐次合成が既存`begin_frame`/`ViewBuilder`/`RenderContext`ライフサイクルと衝突しないかは実装して初めて分かる。BL1切片自体がこの検証を含むべき
2. **MK3の実装形(EffectPass 2入力拡張 vs 完全新設パス)は未判断** — 裁定13(traitはまだ作らない)の再検討が要るかもしれず、調査でなく設計判断なので止めた
3. **`out_of_range`/`master_gain`欠落**(`program.rs` doc 21-23行)がSP2の範囲にどこまで含まれるかは未確定
4. **旧`composite_blend.wgsl`のAdd(Porter-Duff plus、「AE/AMの加算と異なる」注記)と新世界`multiplicative_tint.a=0`方式の数式一致は未検証**

FINDING(対象外だが観測した事実)

1. **`next/core/motolii-store/src/lib.rs`の`ResolvedLayer` docコメント2箇所がstale** — `effects`(391-396行)「まだ読んでいない」・`blend_mode`(398-400行)「まだ合成器は読んでいない」は、裁定153 S1〜S5着地済みの現在では両方誤り
2. **`crates/motolii-nodes/src/mask_apply.wgsl`+`MaskNode`は実質matteの移植元**(4値=MatteMode一致)。発注書がこのファイルを指す時は必ず「matte候補」と明記すべき
3. **`next/engine/motolii-vector`は「実装済みだが未配線」資産**(調査前のsupervisor CONTEXTに無かった新情報)— maskの調達コストを大きく下げる
4. **matte解消はmaskより実装コストが低い可能性**(旧`MaskNode`の4値式がそのまま移植元になるため)— 優先順位の再検討材料
