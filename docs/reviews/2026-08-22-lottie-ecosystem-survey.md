# Lottie エコシステム調査 — 第三者資産と実装の棚卸し(2026-08-22)

調査レーン(読み取り専用)の成果物。発注書: 利用者の気づき「rerun × Lottie の世界を考えている。そういえば Lottie のサードパーティ資産を考えていなかった。あくまで Lottie は製品名」。

**読んだ物**: `next/reference/lottie-coverage.tsv`(738行・source=lottie 656項目)、`next/reference/lottie.schema.json`、
`next/DECISIONS.md`(裁定54/55/56/57/58/62/65/67/68/70/77/81/82/85/94/111/112)、`docs/decision-index.md`、
`docs/reviews/2026-08-22-structural-solutions-ledger.md`、`docs/reviews/2026-07-21-native-surface-renderer-extended-search.md`、
`docs/reviews/2026-07-21-native-surface-renderer-growth-review.md`、`docs/reviews/2026-07-23-first-party-vism-expression-demand-survey.md`、
`docs/vism-package-concept.md`、`next/core/motolii-store/tests/lottie_coverage.rs`、`next/core/motolii-core/src/frame.rs`、
`next/engine/motolii-compositor/src/lib.rs`、`next/engine/motolii-vector/Cargo.toml`、`next/core/motolii-eval/Cargo.toml`。
crates.io / GitHub API / Codeberg API / docs.rs を実在確認に使った(下記「実在確認した crate」節に生の値)。

**先に言っておく既知の事実**(調査前から repo にあった): Lottie(Bodymovin JSON)は**Airbnb**の製品名で、実質「OSS 化された AE のデータ模型解析」として一次資料に使ってきた(裁定54)。だが**保存形式そのものは採らない**(裁定55、上流 `.rrd` — Rerun の recording format)。これは「Lottie を製品として調べていない」という利用者の指摘そのままで、**意味論の参照元と配布フォーマットは最初から別物として扱われている**。今回の調査はこの区別を保ったまま、サードパーティ実装・資産・相互運用を実在確認する。

---

## 0. 現状(owns の実測)

| crate | 行数(src) | 依存 | 役割 |
|---|---|---|---|
| `next/engine/motolii-vector` | 2,618行 | `tiny-skia`(既存 iced 経由で lock 増なし)、`serde` | Shape 語彙・ブール/パス演算・ラスタライズ |
| `next/core/motolii-eval` | 1,173行 | `motolii-core`、`serde` | KeyframeTrack 評価(補間) |
| `next/core/motolii-store` | 14,181行 | — | Document/Intent/Component(Lottie 意味論の器) |

(発注書に書かれた「motolii-vector 3,471行・motolii-eval 1,238行」は実測とややズレる — 現在の実測値は上表。過去の別集計かもしれないが本調査は実測を正とした。)

`next/reference/lottie-coverage.tsv` は**スキーマの語彙を Document の型にどう写すかの判断表**であって、**JSON を実際に読み書きするコードは repo のどこにも無い**(`grep -r` で `import`/`lottie` ヒットは地図とその照合試験だけ)。つまり「地図が557〜656項目埋まっている」は**意味の設計が完了している**ことを示すのであって、**パーサ/インポータが1行でも存在する**ことは意味しない。この区別は判定全体の前提になる。

---

## 1. Rust 実装の棚卸し(実在確認済み)

| crate | 最終活動(2026-08-22 時点) | ライセンス | 実態 | `owns` を縮小できるか |
|---|---|---|---|---|
| **`velato`**(`linebender/velato`) | v0.11.0 / 2026-07-21 公開、GitHub push 2026-07-21。★活発 | Apache-2.0 OR MIT | **Vello(GPU ベクタレンダラ)専用の Lottie パーサ+ランタイム**。`src/schema/`(JSON 模型)と `src/runtime/`(Vello Scene への変換)が分離されている | **できない** — Motolii のレンダラは `tiny-skia`(motolii-vector)であって Vello ではない。依存すると Vello 一式(wgpu 経由でない独自 GPU パイプライン)を二重に抱える。**すでに設計参照としてのみ使用中**(裁定58 transform 順序・裁定111(a) skew 式は `velato::runtime::model::animated` の式をそのまま移植・裁定67 blend mode の `Add`/`HardMix` 欠落根拠)。この使い方(コード査読→式だけ移植)が正しい距離感 |
| **`lottie`**(`zimond/lottie-rs`) | crates.io 0.1.0 / 2024-05-05、GitHub push 2024-05-29。**2年以上停止** | MIT OR Apache-2.0 | JSON パーサ+汎用ツールキット(独自レンダラ含む)。事実上放棄 | 不可(停止・スキーマ更新に追随しない)。**既に設計参照としてのみ使用中**(裁定58 の transform 裏取りに velato と並べて引用) |
| **`dotlottie-rs`**(`LottieFiles/dotlottie-rs`) | crates.io 表記は 0.1.0-alpha.1 / 2024-09-18 で止まって見えるが、**GitHub は v0.1.58 / 2026-06-22 タグ、push 2026-08-18(4日前)— crates.io の停滞は誤誘導**。実体は活発 | MIT | 公式 dotLottie ランタイム。`Cargo.toml` に `links = "thorvg"` — **中身は ThorVG(C++)への FFI ラッパー**+独自の JSON/dotLottie コンテナ/state-machine/theming/audio(rodio)層。純 Rust 実装ではない | 不可として扱う — 採用すると C++ ビルド依存(ThorVG コンパイル、`cc`/`bindgen`)を丸ごと抱える。**保守最低限(wraps>移植>スクラッチ)の観点で「wraps」に見えるが、実際は「C++ ライブラリ全体を wrap した何かを wrap する」で保守面の入れ子が増える**。GitHub 実測を crates.io だけで判断すると「放棄」と誤読するので、この食い違い自体を記録として残す |
| **`thorvg`** / **`thorvg-sys`**(Rust 安全ラッパー) | 0.5.1 / 2026-08-17(5日前)。★活発 | MIT | ThorVG(C++)への safe FFI。`cc` crate で ThorVG を C++ ソースからビルド | 不可(C++ ビルド依存) — ただし独立した crate として存在することは確認(`docs.rs/thorvg`)。既存判定(下記2節)と結論は同じ |
| **`rlottie-rs`**(`msrd0`、本拠地 Codeberg) | Codeberg 更新 2026-03-08、GitHub は意図的にアーカイブ済み(「moved to codeberg」)+ mirror。stars 20、open issues 12 | MIT | Samsung `rlottie`(C++)への Rust バインディング+ファイル変換ツール | 不可(C++ 依存、かつ rlottie 本体は Samsung が2023年頃に事実上更新停止 — ThorVG が後継)。NeoUtl(memory: `neoutl-agpl-timeline-prior-art`)と同じ「本拠地 Codeberg・GitHub はミラー」の形なので混同注意 |

**保守最低限の結論**: 上記5個とも `motolii-vector`(2,618行)/`motolii-eval`(1,173行)を**縮小できない**。理由は一貫している — Motolii のレンダラは tiny-skia、これら5個は「Vello 専用」か「C++ FFI」のどちらかで、どちらも tiny-skia 経路に接がらない。**唯一の実効的な使い方は velato/lottie-rs をコード査読の一次資料にすること**で、これは既に裁定58/67/111 で実践済み。新しい依存は増やさない、という現状の判断は妥当。

---

## 2. 非 Rust 実装を FFI/参照仕様として使う価値

- **ThorVG(C++、MIT、1,780 stars、push 2026-08-22 = 今日)**: `docs/reviews/2026-07-21-native-surface-renderer-extended-search.md:63` と `-growth-review.md:38` で既に **"REJECT as renderer / WATCH as Lottie import"** という結論が出ている。今回の実在確認はこの結論を裏付けるだけで新事実はない。renderer 役は wgpu-native の非公開 handle 問題(公開 `wgpu` crate と bridge しない)で不可。import pipeline 候補としては「見るが着手なし」のまま — 変更なし。
- **rlottie(C++、MIT)**: ThorVG の前身格。Samsung が事実上開発を ThorVG へ移した経緯があり、新規参照先としての優先度は ThorVG 以下。
- **lottie-web(JS、MIT、Airbnb)**: 意味論の参照実装として `docs/references.md:60` で既に使用中(パス演算子の数学的根拠)。これは repo が最初から実践している「FFI で埋め込む」ではなく「コードを読んで式を移植する」パターンで、velato/lottie-rs と同じ距離感。追加の判断は不要。

**結論**: 非 Rust 実装は「FFI で埋め込む」価値はゼロ(全部 C++ ビルド依存を持ち込む)。「読んで式を移植する」参照資料としての価値は**既に実践中**で、今回の調査は追加の一次資料候補を増やさなかった。

---

## 3. dotLottie(`.lottie` コンテナ)

**仕様は実在する**: `dotlottie.io/spec/2.0/`(LottieFiles 提供)。ZIP コンテナで `manifest.json`(必須・アニメーション一覧/初期状態)+ `a/`(Lottie JSON 本体、必須)+ `i/`(画像アセット、任意)+ `s/`(state machine、任意)+ `t/`(テーマ、任意)+ `f/`(フォント、v2 で追加、任意)。v1.0 spec も現存(`dotlottie.io/spec/1.0/`)。

**Rust 実装**: `dotlottie-rs`(上記1節)がコンテナの読み書き(`src/dotlottie/archive.rs`, `manifest.rs`)を持つが、**ThorVG への FFI を伴う一枚岩**で、コンテナ部分だけを取り出して使う形にはなっていない(切り出すなら zip 展開+`manifest.json` パースの数百行を自分で書く方が軸4「保守をしたくない」に合う — 元 crate 全体を依存に持ち込むのは C++ ビルドという別種の保守コストを増やす)。

**我々が読み書きすべきか**: **読み(import)は価値がある、書き(export)は現時点では価値が薄い**。
- 読み: LottieFiles マーケットプレイスの配布は `.json` 単体より `.lottie`(サイズ最大10倍圧縮、複数アニメ+画像同梱)が主流化しつつある(下記4節)。コンテナ自体は「ZIP+JSON」で技術的難度は低い(`miniz_oxide`/`zip` crate で十分、C++ 依存不要)。
- 書き: Motolii は Document を**内部的に**Lottie 語彙へ写せる(裁定54以来の設計)が、**配布物として `.lottie` を書き出す動機**(v1 は音声mux込み映像 export のみに限定 — 裁定・decision-index 244/245)が今のところ無い。5節で詳述。

---

## 4. 第三者資産としての Lottie(この調査の主眼)— 取り込み可否

`next/reference/lottie-coverage.tsv` の判定を集計すると(source=lottie の656項目、`@extends`/継承元の「該当なし」を除く実質判断480項目強):

| グループ | 採用済 | 不採用 | 該当なし | 備考 |
|---|---|---|---|---|
| shapes(パス・塗り・線・グループ・演算子) | 59 | 25 | 50 | **主要形状語彙はほぼ被覆** |
| layers | 25 | 27 | 21 | precomp 系がまるごと不採用(下記) |
| text | 39 | 43 | 1 | style_spans 等は別途裁定77/82/85で凍結済み(採用済に計上) |
| effects / effect-values | 3+7 | 33+12 | 34+18 | **多くの AE エフェクトが未対応**(param 型は用意されているが個々のエフェクト実装は effect 発注単位10行の仕事) |
| styles(Photoshop layer style) | 0 | 78 | 19 | 意図的に effect へ統合(裁定59)。実害は小さい(同等効果を effect 経路で表現) |
| composition/assets | 7+1 | 22+9 | 2+12 | **precomposition が構造的に不採用**(最重要、下記) |

**最大の障壁 = precomposition(ネストされた comp)の不採用**。`assets/precomposition` と `layers/precomposition-layer` は「プリコンポは設計上の除外(GOALS)。グループ化+fold+ベイクへ置換済み」として明示的に不採用。**LottieFiles マーケットプレイスで配布される実物のアニメーションの相当数は AE の precomp 書き出しに由来し、ネストされた `assets` 参照を多用する**(After Effects の実務上、複雑なアニメーションほど precomp を使う)。よって:
- **単純なアイコン/マイクロインタラクション系アセット**(1階層・precomp なし)は、地図の判定を見る限り**ほぼ写像で取り込める**可能性が高い(shapes 59/84 該当行が採用済、layers の基本 transform/timing 系も採用済)。
- **precomp を含む中〜複雑なアセット**は、**インポート時に precomp をベイク(1つの comp へフラット化)する変換パスが要る** — これは「読むだけ」では済まず、変換ロジックの新規実装(precomp の transform 合成・タイムリマップ・マスク継承をベイクして単一階層へ畳む)が必要。地図の判定表はこの変換パスの必要性を明記していない(不採用の note は「グループ化+fold+ベイクへ置換済み」と言うのみで、**「JSON の precomp をベイクして取り込む importer」自体は未実装**)。
- **effect 依存のアセット**(グロー・ドロップシャドウ以外の AE エフェクト)は個別 effect 発注単位が埋まるまで欠落したまま取り込まれる(見た目が変わる)。
- **式(expression)を使うアセット**は明示的に不採用(裁定65 の系譜、`properties/property/x` = Expression が不採用行)なので、**アニメーション自体が壊れる**(値が動かない)。LottieFiles 上の資産で expression 依存はデザインツール(AE)由来のものに限られ、After Effects 直書き出しの資産で稀ではない。

**必要になるコード量の見立て**(実装ゼロからの見積り、地図とは別に本調査独自の判断): (a) `.json`/`.lottie` デシリアライザ(serde、数百行〜)、(b) 地図の「採用済」写像に沿った Intent 列生成(store 側の `Intent::Set*` を組み立てるビルダー)、(c) precomp ベイクパス(変換ロジック、規模不明・地図に無い新規設計)、(d) 未対応語彙(effect/expression/style_spans の一部)の**劣化時の扱い**(黙って捨てるか、警告して部分インポートか — 未決)。(a)(b) は「ほぼ写像」だが (c)(d) は新規設計が要る。

---

## 5. 書き出し側(Motolii Document → Lottie)

裁定244/245(decision-index.md)が既に明言: **v1 の製品出力は既存 `ExportJob` による音声mux込み完成映像に限定**し、Lottie/animated SVG/OTIO/別Host project/Web runtime/外部service publishは**完成条件外**。将来のために **Delivery Adapter capability の席だけを残し、特定フォーマットの field を恒久面へ予約しない**方針(裁定と同時に vism-package-concept.md §5.3 が同じ内容を持つ)。

**技術的な可否だけを見れば**: 採用済230語彙(現状の地図の「採用済」総数)がそのまま Lottie JSON の対応フィールドへ写せるなら書き出しは原理上可能 — ただし
- precomp を使わない(Motolii は fold/bake 済みの単一階層構造を持つ)ので、**書き出す Lottie JSON は precomp を使わない「フラットな」JSON になる**(仕様上は合法、Lottie は precomp 必須ではない)。
- 逆方向の劣化は起きにくい(Motolii → Lottie は「持っている物を出すだけ」で、Lottie → Motolii のような「持っていない物を諦める」問題が起きにくい)。
- ただし、**expression や style_spans のような Motolii が持たない Lottie 語彙は最初から出さない**ので、他エディタ(AE/Lottie系ツール)で開いたときに機能劣化はしない代わりに「単純化された」ファイルになる。

**現状の判定を覆す新事実は見つからなかった**: 書き出しは技術的に着手可能な範囲に見えるが、**v1 完成条件の外というのは製品判断であって技術的難易度の判断ではない**ため、本調査はこれを覆す材料を提示しない。Delivery Adapter capability の席を将来使うなら、この調査の4節・5節がその時の一次資料になる。

---

## 6.「製品名」であることのリスク

- **仕様の所有者は分裂している**: 元祖 JSON スキーマ(Bodymovin/lottie-web)は **Airbnb**。dotLottie コンテナ・現在のエコシステム牽引(lottie-docs、コミュニティ、"Lottie Power Stack 2026" 提唱)は **LottieFiles**(別会社)。両者は協調しているが、**単一の標準化団体は無い** — W3C 等の中立機関の管理下ではない。
- **更新頻度**: `next/reference/lottie.schema.json` は上流そのまま vendoring している(裁定68)。dotLottie は v1→v2(2026年内)で state machine・音声・テーマを統合する拡張が進行中(`structural-solutions-ledger.md` 既述の "Lottie Power Stack 2026")。**スキーマは今も動いている** — 656項目の地図は「ある時点のスナップショット」であり、上流が語彙を追加すれば地図の再照合(`lottie_coverage.rs` 試験)が必要になる。
- **後方互換の実績**: JSON スキーマ自体は長期にわたり追加のみ(deprecated フィールド `e` を残す等 — 地図で不採用と明記済み)で破壊的変更は稀という評判はあるが、**本調査では一次資料(CHANGELOG等)による実証はできていない**(EVIDENCE_GAP として明記)。
- **fork/標準化の動き**: 本調査で新規に見つかった fork や競合標準化提案は無し。ThorVG は「Lottie を読める代替エンジン」であって仕様の fork ではない。
- **リスクの実務上の帰結**: Motolii が Lottie の**保存フォーマットを採用していない**(裁定55)ため、上流の破壊的変更やベンダーの意思決定変化から**保存資産が人質に取られるリスクは無い**。リスクが顕在化しうるのは「輸入元フォーマットとしての Lottie/dotLottie」を今後実装した場合に限られ、その時点でも「読めなくなったら地図と importer を更新するだけ」で済む(自前の意味論を正本にしているため)。**製品名であることのリスクは、保存形式として採用していない現状の設計判断によって、構造的に低く抑えられている**。

---

## 実在確認した crate 一覧(最終更新・ライセンス、2026-08-22 時点で確認)

| crate/repo | 最終更新(実測) | ライセンス | ソース |
|---|---|---|---|
| `velato`(linebender/velato) | v0.11.0 crates.io 2026-07-21、GitHub push 2026-07-21 | Apache-2.0 OR MIT | crates.io API、GitHub API |
| `lottie`(zimond/lottie-rs) | crates.io 2024-05-05、GitHub push 2024-05-29 | MIT OR Apache-2.0 | crates.io API、GitHub API |
| `dotlottie-rs`(LottieFiles/dotlottie-rs) | crates.io 表示 2024-09-18(alpha 止まり)/ GitHub タグ v0.1.58 2026-06-22、push 2026-08-18 | MIT | crates.io API、GitHub API(tags/releases) |
| `thorvg` / `thorvg-sys`(Rust safe binding) | 0.5.1、2026-08-17 | MIT | docs.rs |
| ThorVG 本体(thorvg/thorvg、C++) | push 2026-08-22(当日) | MIT | GitHub API |
| `rlottie-rs`(msrd0、本拠地 Codeberg) | Codeberg updated 2026-03-08。GitHub は意図的アーカイブ+mirror表記 | MIT | Codeberg API、GitHub API |
| lottie-web(airbnb/lottie-web、JS) | 既存参照(`docs/references.md`)、本調査で再確認せず | MIT | 既存記載を継承 |
| Glaxnimate(KDE/glaxnimate) | 既存参照(`docs/references.md`)、設計参考のみ | GPL-3.0(コード流用不可) | 既存記載を継承 |

---

## 推奨(判定軸: 保守最低限 > owns縮小可能性 > 第三者資産取り込み価値 > 相互運用)

1. **`owns`(motolii-vector/motolii-eval)は縮小しない**。velato/lottie-rs/dotlottie-rs/thorvg のいずれも tiny-skia 経路に接がらないか C++ FFI を要求し、置き換えると保守コストが増える方向に働く。現状の「読んで式だけ移植する」使い方(裁定58/67/111 で実践済み)が最適点。**新規依存は増やさない**。
2. **`.lottie`(dotLottie)コンテナの読み込みは、将来 import 機能に着手するなら価値がある** — ただし `dotlottie-rs` を依存に足すのではなく、コンテナ部分(ZIP+manifest.json)だけを自前で数百行書く(`miniz_oxide` は既に `motolii-vector`? いや store 系で使用歴なし要確認だが軽量)方が軸4に合う。**今は着手材料が無い**(v1 完成条件外)ので着手はしない。
3. **第三者資産の import は技術的に価値があるが、precomp ベイクという新規変換ロジックが必須**。「地図が採用済なら写像で済む」という前提は**precomp を含まない単純アセットに限って正しい**。この限定を伴わずに「ほぼ写像で済む」と言うと実態より楽観的になる — 発注時はこの限定を必ず明記すること。
4. **export(書き出し)は裁定244/245により v1 完成条件外のまま変更なし**。技術的な障害は薄い(Motolii → Lottie はフラット構造を出すだけ)が、製品判断が先にある。
5. **「製品名」リスクは現状の設計(保存形式に採用していない)により構造的に低い**。地図の再照合試験(`lottie_coverage.rs`)が上流スキーマ更新への追随を機械的に担保しているので、追加の防御策は不要。

## 逸脱・EVIDENCE_GAP

- 発注書記載の行数(motolii-vector 3,471行・motolii-eval 1,238行)と実測(2,618行・1,173行)が食い違う。実測を採用し、注記のみ残した。
- Lottie JSON スキーマの後方互換実績は、CHANGELOG 等の一次資料を直接確認できておらず、伝聞評判の域を出ない(EVIDENCE_GAP)。
- precomp ベイク変換パスの実装規模は本調査の範囲外(設計未着手のため見積り不能)。
