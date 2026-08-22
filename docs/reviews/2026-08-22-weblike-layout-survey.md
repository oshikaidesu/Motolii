# web のようにレイアウトする周辺技術の調査 — モック再現の構造化(2026-08-22)

状態: **調査**(製品コード変更なし。probe は scratchpad 内 check 級)
起点: 利用者発注「勝負はモック再現。iced と HTML でかなり出来に差がある — 手癖の問題か?
web みたいに iced でレイアウトできたら嬉しい」

---

## 結論(推奨1本+次点)

**推奨: 案A+C の複合 — taffy 駆動 flex/grid container widget(自作・in-repo)を転写の
実装層に、既存器具 `motolii-css-metrics`(Blitz=stylo+taffy)の next への移植を検証層にする。**

- **実装層(案A)**: `iced_core::Widget` を taffy 0.13(crates.io、2026-08-08 更新)で
  layout する container widget を1本書く。**check 級 probe で fork rev 73e686ee
  (0.15.0-dev)との型互換を実測済み**(本文§2)。これで CSS の
  `display:flex/grid`・`gap`・`grid-template-columns: minmax(132px,1fr) repeat(3,64px) 26px`・
  `justify-content` 系がモックの語彙**そのまま**で書ける — 転写が「CSS 宣言→taffy::Style の
  ほぼ字面写し」になり、手癖(翻訳工程)が構造的に消える。
- **検証層(案C)**: モック側も実装側も **同じ taffy が矩形を解く**構図になるため、
  `motolii-css-metrics` が吐く box(taffy `final_layout`)と実装側 layout の突き合わせが
  「taffy 対 taffy」の ±1px oracle になる。器具は旧 workspace に実在(466行、CPU only・
  GPU 不要)で、next へは器具として移植するだけ(wraps>移植の第2位。スクラッチゼロ)。
- **判定軸との照合**: 保守=＋taffy 1 dep(Dioxus 保守・Bevy/Blitz が使う実証済み crate。
  旧 workspace も 0.12.2 を SD-02G で既採用)+自作 widget 1本(probe 実測 160 行、
  製品化で 400〜700 行の桁)。機械検証=taffy 対 taffy で ±1px 柵に**構造ごと**載る。
  手癖不要化=CSS→Style の対応表だけ渡せば subagent が迷えない。
  ホットリロード=taffy::Style は数値データなので tokens watch(notify)と同じ構造に載る。

**次点: 案C 単独(検証層だけ機械化)。** 新 widget を入れず、既存 Row/Column 転写のまま
`css_metrics` の box 出力と実装定数の oracle だけ張る。実績データ(§1)が示す欠陥源
(転写ズレ)はこれだけでも塞がる — ただし「web みたいに書けたら嬉しい」の DX は満たさない。

**不採用**: 案B CSS interpreter widget(案A+C と同じ成果をより多い保守負債で買う —
スクラッチ禁止方針に反する)。案D Blitz/dioxus-native pane 描画(**先行裁定維持**、§5)。
案E morphorm(非 CSS モデルで目的に合わない、§6)。

### 発注可能な分界図

```
モック正本 next/reference/mocks/*.html
   │
   ├─(検証層・案C)motolii-css-metrics 移植(next の器具 bin、製品 dep にしない)
   │      → pane ごとに box/padding/computed JSON を抽出
   │      → oracle テスト: 実装側 layout と ±1px 突き合わせ(旧 inspector_pixel_fence と同型)
   │
   └─(実装層・案A)ui/ 配下に flex container widget crate 1本(iced_core+taffy のみ依存)
          → 各 pane の「箱の入れ子」を taffy::Style で宣言(値は motolii-tokens-rs 経由のまま)
          → 中身(text/button/canvas/shader)は既存 iced widget を子として差すだけ
```

- レーン1(器具): css_metrics 移植+oracle 雛形 — write-set は器具 crate と tests のみ
- レーン2(widget): FlexBox widget 製品化(operate/update/mouse_interaction/overlay 委譲+
  taffy Cache)— write-set は新 crate のみ
- レーン3以降(pane 転写): pane 1枚=1レーンで FlexBox へ載せ替え、受入=レーン1の oracle green
- 適用範囲の注意: **効くのは widget 合成の面**(Inspector・Browser・Settings・Shell chrome)。
  Timeline/Stage の canvas/shader 内部の手描き座標には layout エンジンは効かない(そこは
  裁定165〜168 の比率文法+案C の抽出値照合が引き続き正)

---

## 1. 「手癖の問題か?」への直接の答え

**はい — ただし「腕」ではなく「工程」の問題。** 実績データは全て転写**工程**由来で、
iced の描画・レイアウト能力由来の差は 0 件:

- [CSS計算値抽出の突き合わせ](2026-08-19-css-computed-metrics-extraction.md): Inspector 側
  寸法定数 **11/11 が mock と機械一致** — iced(素の Row/Column+固定幅)でもモックは
  正確に写せることの機械確認。つまり表現力の壁ではない。
- [Timeline 転写ギャップ台帳](2026-08-21-timeline-transcription-gap-survey.md): 逸脱・余剰の
  全件が「別世代 mock からの借用(#6)」「比率の分母の自前宣言(#2)」「角丸の写し漏れ
  (#4,#16)」「素の iced widget 既定意匠の露出(#23 transport slider)」— どれも人手翻訳の
  取り違えであり、iced が表現できなかった物は 1 件も無い。文字品質は裁定168/169 で解決済み。
- 一方で iced 素の語彙は web より狭いのは事実: fork(0.15-dev)実査で Row/Column は
  spacing/padding/align/Fill・FillPortion のみ(`justify-content: space-between` 相当なし、
  wrap なし)、新設の stock `Grid` widget は**均一セルのギャラリー grid**
  (`columns(n)`/`fluid(max_width)`/アスペクト比)で、Inspector mock の
  `grid-template-columns: minmax(132px,1fr) repeat(3,64px) 26px` は書けない。
  今までは固定幅+Fill で**手作業エミュレート**しており、その翻訳こそが手癖工程だった。

つまり「出来の差」の正体 = HTML では宣言 1 行の物を iced では手で数値に翻訳しているという
**工程の段差**。段差を消す最短手は、翻訳先の語彙を web と同じにする(案A)+翻訳結果を
機械で照合する(案C)。

## 2. 案A: taffy 駆動 container widget — probe 実測

### 実在確認(先行例)

- **taffy**: crates.io 0.13.0(2026-08-08 更新 — 現役)。flexbox+CSS grid+block を実装。
  Bevy/Dioxus/Blitz が本番使用、Zed が fork を保持。保守が死ぬリスクは iced 本体と同程度に低い。
- **iced_taffy(nicoburns — taffy 保守者本人)**: 実在するが**最終 commit 2023-04-11**、
  iced は本人 fork の古 rev に pin、README 自ら "very rough proof of concept"。Grid 1個のみ。
  **そのまま使う物ではなく、「iced Widget::layout を taffy で解く」パターンの実証としてのみ有効**
  (child 測定を taffy measure 関数+taffy::Cache で行う構造は写せる)。
- egui 圏では `egui_taffy` が同パターンで成立済み(2026-07-20 spike で自験あり)。

### fork 互換の実測(check 級 probe、憶測でない根拠)

probe: `/private/tmp/claude-501/…/scratchpad/taffy-iced-probe/`(Cargo.toml+lib.rs 160 行)。
依存は `iced_core`(**fork rev 73e686ee そのもの** — 製品と同一 rev、wgpu 不要)+
`taffy = "0.13"`(crates.io)。**`cargo check` green。**

確認できた事実:

1. fork の trait は `fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits)
   -> layout::Node`(0.15-dev で `&mut self` 化)— taffy の measure 閉包から
   `child.as_widget_mut().layout(...)` を呼ぶ形が**そのまま**借用検査を通る:

   ```rust
   taffy.compute_layout_with_measure(root, avail,
       |known, space, _id, ctx: Option<&mut usize>, _style| {
           let i = *ctx?; // 子 index
           let child_limits = Limits::new(Size::ZERO, Size::new(w, h));
           let s = children[i].as_widget_mut()
               .layout(&mut trees[i], renderer, &child_limits).size();
           taffy::Size { width: s.width, height: s.height }
       })?;
   // taffy が解いた矩形で子を最終 layout し Node::with_children へ写す
   ```

2. `layout::Node::with_children(size, vec)` + `move_to(Point)` が taffy の
   `Layout{location,size}` を無変換で受ける(座標系・f32 とも一致)。
3. `Tree::diff_children(&mut self.children)`・`draw` の子委譲も既存 Row と同型で書ける
   (製品化時は operate/update/mouse_interaction/overlay の委譲を Row から写す)。
4. `&mut self` layout なので taffy ツリー/Cache を widget 自身に保持する最適化も型的に可能
   (probe は毎回構築 — 製品でも pane 規模(数十〜数百ノード)なら taffy は µs 桁で解く)。

### 工数の桁

probe 160 行 → 製品版(委譲一式+Cache+`taffy::Style` の builder 糖衣+テスト)で
**数百行・1 crate・日の桁**。fork への追加は**ゼロ**(iced 側は一切触らない —
rebase 複利を増やさない。裁定170 案A棄却と同じ理由で fork 非改変を維持)。

### リスク・限界

- 子の layout が measure+最終確定で複数回呼ばれる(flex の常。taffy::Cache で抑える。
  iced 自身の `layout::flex::resolve` も2パスであり新奇な負荷ではない)
- text の intrinsic 測定は limits 経由で正しく返るが、`AvailableSpace::MinContent/MaxContent`
  を Definite へ潰す近似を入れた(probe 同様)。折返しテキスト多用の面では要検分
- widget 合成の面にのみ効く(§ 分界図の注意)

## 3. 案C: モック→データのコンパイラ(転写の機械化)

**新設ではなく移植**。`motolii-css-metrics`(旧 workspace `crates/motolii-ui/src/css_metrics/`、
mod 335 行+bin 131 行)が既にモック HTML/CSS を blitz-dom+stylo で解き、全要素の
`box{x,y,w,h}`(taffy final_layout・祖先加算済み絶対座標)+padding/border+computed style
(background/font_size/gap/border_color)を JSON で吐く。`::before` 生成 box も取れる。
GPU 不要・`document.resolve` 2回だけ・`<link>` は in-memory 展開済み。

- 実績: Inspector 11 定数の機械照合(2026-08-19)+oracle テスト(inspector_pixel_fence)
  という**運用実績つき**。今回やることは (a) next の器具 bin として移植(製品 dep にしない —
  blitz-dom 0.3.0-beta.1 は器具側にだけ居る)、(b) 出力を pane 別 oracle テストへ配線、
  (c) 必要なら比率(裁定165/167/168 の分母)への正規化列を足す。工数=**日の桁**。
- 案A と組むと検証が「taffy 対 taffy」になる(モック側の矩形も実装側の矩形も同じエンジンが
  解く)ため、±1px どころか丸め誤差レベルで一致させられる。
- dimensions.json の節を自動生成する所まで進めるかは任意(まず oracle。生成は正本の所在が
  JSON⇄mock で二重になるので、裁定(値の正本は JSON 側)との整理が要る — 発注時に分ける)。

## 4. 案B: CSS サブセット interpreter widget — 不採用

実行時に mock の CSS を読んで layout する container。魅力はホットリロードの完全性
(mock 編集=即実画面)だが:

- パーサ+セレクタ束縛(どの iced widget がどの HTML 節に対応するかの対応表)+カスケード
  近似で **数千行のスクラッチ層**。maintenance-minimal(wraps>移植>スクラッチ)に正面衝突
- 得られる成果(web 語彙で書ける・±1px 照合)は案A+C が既により少ない保守で出す
- ホットリロードは tokens watch が寸法値で既に持っており、taffy::Style を data 駆動にすれば
  同じ notify 構造に載る(必要になった時に案A の上へ足せる — 今日決めない)

## 5. 案D: Blitz / dioxus-native を pane 描画に使う — 先行裁定維持(不採用)

先行裁定2本([timeline 実行時基盤の egui 再選定 2026-08-16](2026-08-16-timeline-runtime-reselection-to-egui.md)が Blitz runtime を実測4欠陥
— z-index 絶対配置の文書原点飛び・初回 hit の resolve 2回・`round_layout` の 0.5px clip 消失・
transition のドラッグ遅延+retina 単位境界 — で棄却、メモリ `iced-webview-v2-observation`
2026-08-19 が不採用)を覆す新実測は**無い**: blitz-dom は今も 0.3.0-beta.1
(2026-07-10 公開、それ以降 crates.io 更新なし)で当時測った版のまま。shell 自体の描画という
今回の文脈はむしろ当時より要求が重く(全 pane 常時+入力/IME/D&D が texture の向こうへ行く
構図は 2026-08-19 の利用者判断と同型)、再評価の材料が発生していない。**1行結論: 不採用維持。
ただし Blitz は「器具」としては現役採用**(案C の心臓部 — 製品に入れず値だけ貰う、
2026-08-16 の「ビルド時コンパイラ」役割変更どおり)。

## 6. 案E: その他の layout crate

- **morphorm 0.9.0**(2026-07-17 更新、Vizia の layout engine): 現役だが独自モデル
  (stacks/percentage、CSS flexbox/grid 非互換)。「モック CSS の字面をそのまま写す」という
  今回の勝ち筋に合わず、iced 統合の先行例もゼロ(スクラッチ)。不採用。
- 宣言マクロ系(iced_aw Grid 等): iced_aw は別文脈で probe 済みだが、その Grid も
  grid-template 語彙を持たない。taffy が上位互換。不採用。

## RETURN 要約

- 文書: `docs/reviews/2026-08-22-weblike-layout-survey.md`(本書)
- 結論: 推奨=**案A(taffy container widget)+案C(css_metrics 移植 oracle)**、
  次点=案C 単独。B/D/E 不採用(D は先行裁定維持)
- probe: `taffy-iced-probe`(scratchpad、160 行)— fork rev 73e686ee の `iced_core` +
  taffy 0.13 で `cargo check` green。fork trait は `layout(&mut self, …)` で taffy measure
  閉包と無理なく噛む
- 工数の桁: 案A=日(数百行・fork 非改変)、案C=日(既存 466 行の移植+配線)、
  pane 載せ替えは 1 pane=1 レーンで並列可
