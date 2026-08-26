# 実行計画(2026-08-25)— 裁定237〜247 を発注へ落とす

日付: 2026-08-25 / 状態: **観察**(**計画であって裁定ではない**)。
本文書は波の順序・write-set・発注の型を並べるだけで、**設計判断を1件も下さない**。
判断が要る物は §1「発注前に閉じる穴」に**穴として置く**。

前提となる裁定: 237(mock CSS が値と状態の正本)/ 238(縦切りスライス)/
239(3Dカメラに向き)/ 240(組織ペルソナ = 動線)/ 241(property モデル)/
242(hero creation が製品目的)/ 243(横断問題の着地)/ 244(問題表の優先順位)/
245/246/247(最小コアの剪定)。
棚卸しの正本: [layer 属性 vs property](2026-08-25-layer-attrs-vs-property-audit.md)。

なお、一般的な動画ソフトの機能表はここでの目的ではなく基礎床である。実装の優先順位は、
一般 NLE の機能数ではなく、hero creation の動線を閉じるかで読む。

## 0. 問題起点の優先順位

各台帳粒は、機能名ではなく「何が止まっているか」を先に書く。P0(信頼・安全)とP1(制作ループ)を
閉じないままP3/P4の便利機能を足さない。P2(hero表現)はP0/P1の結果が観測できる状態で、
heroを立ち上げる表現手段として発注する。これは新しい台帳体系ではなく、既存の
`truth_safety` / `core_edit` / `render_export` / `fanout` / `frequency` / `convenience` の
重みを利用者問題へ翻訳する読み口である。各レーンの発注書に最低限、次の3行を追加する。

`next/reference/normal-map.tsv` は他製品の候補在庫であり、全行を製品バックログへ昇格させない。
採用する粒だけをM/Dまたはcomponentへ束ね、採用しない粒は「不足」ではなく候補のまま残す。

### 0.1 第一剪定の実績

第一剪定で507粒、第二剪定で410粒、計917粒の `採用予定` を `拡張` へ戻した。削除や証拠消去ではなく、
候補在庫と最小コアの境界を台帳へ反映したもの。現在の静的件数は `採用予定 37 / 拡張 976`。
残した37粒は、入口・素材差替え・基本編集・再生・安全、property時間変化、点群用3Dカメラに直接対応する
未着地候補。テキスト詳細、音声制作、パネル/ワークスペース、細かな3Dビュー/ギズモ、リタイム補助、
トラック表示、重複ショートカットは候補在庫として残した。これは静的台帳の整理であり、実機受入・
Cargo合格の主張ではない。

```text
PROBLEM: 利用者が何に詰まり、何を失い、何を表現できないのか
OUTCOME: 何が起きればその問題が解決したと言えるか
PRIORITY: P0 信頼・安全 / P1 制作ループ / P2 hero表現 / P3 摩擦削減 / P4 便利
```

## 1. 発注前に確認する境界(レーンに判断させない)

**この3件はレーンに判断させない。持ち主・延期条件・再審議条件を裁定243で固定した。**

| # | 穴 | 背景(実測) |
|---|---|---|
| H1 | **descriptor の持ち主** — ラベル・単位・範囲・刻み・感度・表示形式をどこが宣言するか | **解決**: provider/device catalog が宣言し、既存 `ParameterDescriptor` を host-side owner とする(裁定243)。Inspector はそれを投影する。現状の `GlowParam` と `TransformField::EffectParam(EffectId, GlowParam)` はこの境界へ移す実装対象 |
| H2 | **姿勢表現**(quaternion か Euler か)+ handedness | **延期**: property 移送と現在の基盤ゲート後に、回転補間を含む一つの pose property として裁定する(裁定239/243)。レーンに選択を委任しない |
| H3 | **`next/core/motolii-store` を割るか、割るならどこで** | **解決**: 具体的な write-set 衝突が測定されるまで分割しない。`motolii-store` は単一の Document owner として保つ(裁定243) |

W1 は裁定241とG0後に進める。W2 はH1の既定ownerに沿って実装する。W3はH2の姿勢裁定とG0後に
進める。どのレーンもdescriptor owner・pose表現・store分割を再発明しない。

## 2. ボトルネックの名指し

`next/core/motolii-store` は Document の意味を一つに持つ境界である。P群10件が同じ crate に
集まることだけでは分割理由にならない。裁定243により、まず単一 owner のまま write-set の
実測を取り、具体的な衝突と分割後の公開境界が同時に示せる場合だけ再審議する。目的を守るために
境界を増やすのであって、並列数を増やすためだけに store を割らない。

## 3. 波の順序と依存

```
G0(現在)   : Design Profile v0 → CORE-M0 実窓。FOUNDATION_SERIAL、並列LOCKED
W1         : property 移送 P群10件            ← 裁定241 + G0
W2         : Inspector = 窓                   ← W1 依存 + H1の既定owner(裁定243)
W3         : camera 向き                      ← W1 依存 + H2のpose裁定(裁定239/243)。G0後の拡張
並行独立   : 裁定237 の切片6本                ← parallel unlock 後だけ
```

- crate の憲章は**新規体系にしない**。各 crate の既存 `//! owns:` をそのまま境界宣言とし、
  足すのは「断ってよい/断るべき物」1〜2行だけ(裁定240)
- 裁定237 の切片6本は意味上独立でも、現在の基盤ゲートが解くまで開始しない。ただし §4 の衝突1件は残る

## 4. write-set 表(互いに素であることを示す)

| 束 | write-set |
|---|---|
| P群移送(W1) | `next/core/motolii-store`、`next/core/motolii-eval` |
| Inspector = 窓(W2) | `next/ui/motolii-inspector-pane` + plugin の param 宣言(engine 側) |
| camera 向き(W3) | `next/core/motolii-core/src/camera.rs`、`next/engine/motolii-compositor` |
| 裁定237 切片(並行) | `next/ui/motolii-taffy`、`next/ui/motolii-css-metrics`、`next/reference/mocks`、各 pane の view |

**既知の衝突1件**: 裁定237 の**切片4(呼び手を chrome へ広げる)**と **W2(Inspector = 窓)**が
`next/ui/motolii-inspector-pane` で衝突する。
**対処 = 切片4 から Inspector を外す**。browser / settings / timeline を先に行い、
**Inspector は W2 着地後**に回す(W2 で section の書き方自体が変わるため、先に転写しても
捨てることになる)。

## 5. レーン発注の型(全レーン共通)

1. **落ちるテストで渡す** — 仕様書ではなくテストを先に書く
2. **レーン検収線 = `cargo check --tests -p <crate>`**(型まで。テストは書くが回さない)、
   **波末に一括 `cargo test` を1回**(裁定189)。**適用条件は「意味が既決の消化フェーズのみ」**
   ——新しい意味論・store/engine 跨りの束は従来の即検収に戻す(裁定189 の適用条件そのまま)
3. **closed order / NON-GOALS / RETURN** — 判断が要ると感じたらレーンは書かずに返す
4. **値は決めるのではなく `file:line` から写す**
5. 各レーンに**視覚受入条件と検証器具**を必ず入れる(green 100% は発注書の完全性を保証しない)

## 6. 未解決として残す1件(本計画では扱わない)

`docs/decision-index.md:23` が「**リセット後(2026-08-20〜)の裁定は `next/DECISIONS.md` に
置く。この索引は旧 workspace の歴史台帳として凍結し、新しい行を足さない**」と宣言している
一方で、凍結後も同索引に行が足され続けている(冒頭行が裁定237 の下地を載せている)。
**失効させるか守るかは未裁定**。本計画では扱わず、開いた項目として記載するだけ。
