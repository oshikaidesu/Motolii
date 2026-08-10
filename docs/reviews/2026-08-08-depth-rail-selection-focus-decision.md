# Depth Rail 選択フォーカス設計 — 灰色統合・個別化=逸脱・実声調査の返却

日付: 2026-08-08
状態: **決定(利用者裁定)＋観察 / 実装発注は未実施**

## 0. この文書の扱い

会話で生じた設計決定と調査結果を正本へ回収する。**本書を根拠に実装を発注しない。**
skia fixtureはリポジトリ外のprobe([所在](#5-fixtureの所在))にあり、`crates/`は無変更。

## 1. 経緯 — 7案の却下が何を教えたか

Codexの一枚目(v6)を台帳照合で批判した後、代理(Claude)が5案を重ねて全て却下された。

| 案 | 形 | 却下理由(利用者) |
|---|---|---|
| v6 (Codex) | 数直線+文字pill | 主題(27個の山)が文字で、印の量と重要度が反転 |
| v7/v8 | 件数の棒+扇+配布slot | 分厚い、直感的でない |
| v9 | 削った数直線 | ピンとこない |
| v10 | parallax係数列 | 解釈を増やした(zと率の二重表示) |
| v11 | ジオラマ(側面図) | 根本的に違う |
| v13 | 面の全廃+Spread 1量 | UIから逃げている |

共通の誤りは**「深度をどう見せるか」を問いにしたこと**。利用者の原則が問いを裏返した:

> zはユーザにとって避けたいもののはず。画面は平面なのに奥行きを求められるから。
> 情報量を無闇に増やすのではなく、いかに段差をなくせるかという思考が必要

## 2. 決定(利用者設計)

1. **z=0の既定シェイプ群は灰色1塊に統合し、個別に描かない。**
   `0 × N` stack(2026-07-22受入契約)の具体化であり、件数は塊の中の一語で足りる
2. **個別に描かれること自体が逸脱表示である(新規則)。**
   zを持ってしまったObjectだけが個別の灰色tickになる。規範「状態語は逸脱時のみ表示」の
   個別化への拡張。scope chip(`ROOT`)も同規則で沈黙し、Group内に居る時だけ現れる(改訂)
3. **選択だけがrail上のフォーカスを与える。** 選択されたObjectはTimeline行と同色の
   可動markerとして塊から覗く。識別・名前・値は行が持ち、railは複製しない
   (AEプラグインが表を複製したのはホストのレイヤー一覧に触れないためであり、
   Motoliiは両方を所有するから省ける — 利用者の指摘)
4. **視差はdragで初めて生まれる。** railは状態の地図ではなく、選択に対する操作の舞台。
   レーンは現在時刻に存在するシェイプのみ反映する(既決の追従の再確認)
5. **`Preserve Appearance`(keep look)は既定ON。**
   [specs/M5-3d-and-post.md](../specs/M5-3d-and-post.md) §「Preserve Appearance」の
   「明示的に選べる」を既定ONへ強化する。spec本文の改訂が必要(本書では改訂しない)

drag確定・Cancel・1 gesture=1 UndoはD2既決のまま。開閉は明示操作のまま(自動openしない)。

## 3. 観察 — 実声調査10方向の返却

設計が3周空転した後、利用者の指示で実際の声を検索した(2026-08-08、10方向並行、80件超)。
**規律6点に従い、本調査は設計根拠ではない。** 設計の正本は§2の利用者裁定であり、
調査は触媒と整合確認に留める。反例(深度パネルを快適に使う声)は未探索で、
不満フォーラム偏重・Reddit封鎖によるAdobe Community偏重のバイアスがある。

頻度順の痛みクラスタ(代表引用):

1. **黙って壊れる** — カメラ追加で全layerが消える・無反応・同一Zのz-fight原因が見えない。
   Harmony公式docsが「どのlayerがZ移動済みか把握し難い」と自認
   ([About Multiplane](https://docs.toonboom.com/help/harmony-22/premium/staging/about-multiplane.html))
2. **深度を触ると絵が壊れる** — 「maintain scale as you move it back in Z space?」が
   newsgroup時代から続く定番質問。対策はexpressionのみで書けない人に壁、カメラ追加で壊れる
3. **カメラがボス戦** — 「the camera often gets lost」。AviUtl圏では
   「カメラ制御を使わない遠近感」が記事ジャンルとして成立
4. **深度は帳簿仕事** — 100枚で数ヶ月見積り。AEのalign/distributeは3D layerに効かない
5. **深度が構造と絡まる** — Animate Layer Depthはrootのみ・preview≠export。
   Harmonyはhierarchyごと動く(Motoliiのscope分離既決が同じ穴を先回りで塞いでいる)
6. **ワンタップは技能の削除ゆえに愛され、失敗時の逃げ道が無いゆえに憎まれる**

**「深度関係を一覧・比較したい」という声は0件**(不在証明ではない)。§2の設計はこの分布と
整合する: 灰色統合と逸脱chipが1・4に、keep look既定が2に、選択フォーカスが4に対応する。

先例の対比: AE系プラグインは全て表+範囲+動詞(AnimateParallaxの
`Near/Far`+`Distribute`/`Apply`、識別は表)。2Dゲーム系はZを経由せず視差率
(scroll factor)を直接編集する系譜。Motoliiは表をTimeline行が既に担うため、
railには操作の舞台だけが残る — という分業が§2である。

## 4. 未決

- 選択が大きい時のタブ表現(40選択で塊上に40本は覗けない)
- 群dragの意味 — 一括平行移動と、既決Layer Order Distribute(奥端・手前端)への接続、両者の区別ジェスチャ
- camera markerと警告(camera順≠Layer Order等)の逸脱ゲートの具体
- 視差率(parallax rate)をDeveloper info/第二readoutへ出すか(正本はzのまま。第二のDepth fieldは作らない)
- orthographic時の型付き無効文言(視差が定義できない)
- marker px、AX、20〜100層fixtureでの密度検証

## 5. fixtureの所在

`~/Documents/Codex/2026-08-06/motolii-ui-hybrid-research-handoff/work/skia-timeline-probe/src/bin/`
(2026-08-10移管: 現在の正本は[`spikes/skia-timeline-probe/src/bin/`](../../spikes/skia-timeline-probe/README.md))

| bin | 内容 |
|---|---|
| `motolii_depth.rs` | Codex v6(oracle対照用・無変更) |
| `motolii_depth2.rs` | v7〜v10の変遷(棒/扇/削減/係数) |
| `motolii_depth3.rs` | v11 ジオラマ |
| `motolii_depth4.rs` | 縦案(未提示・放棄) |
| `motolii_depth5.rs` | v13 段差ゼロ3段 |
| `motolii_depth6.rs` | **v14 本決定の静止画** |
| `motolii_depth_interactive.rs` | **本決定の対話demo**(選択→フォーカス→drag→視差) |

出力PNG `motolii-depth-rail-v*.png` も同ディレクトリ。

## 6. 非目標

- 本書を根拠とする実装・発注
- Document schema・公開APIの変更(保存値は`position.z`のまま)
- `crates/`へのfixture持ち込み
- 調査結論の設計根拠化(規律6点)
