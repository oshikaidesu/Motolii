# キーボード到達性の穴(A02) — 裁定215 に照らした施工設計

日付: 2026-08-23 / 状態: **調査+設計案**(実装なし・裁定なし)

前提: 裁定216(`next/reference/GESTURES.md`)が主要ジェスチャ34件を実測し、**25件がマウスのみ**と
確定した。原因はレーンの実測によれば iced の `button`/`pick_list`/`pane_grid` がキーボード到達性
(フォーカス移動・Tab 巡回・Enter/矢印での活性化)を持たないこと。本文書は裁定215
(「持つ」には意見を名指しできることが条件、既定は「借りる」)の型で、この穴をどう扱うかを設計する。

## 0. 読んだ範囲

- `next/reference/GESTURES.md`(202行、全文)— §2 が既に fork ソースの実測を1回行っている
- `next/reference/OPINIONS.md`(59行、全文、意見18件)
- `next/DECISIONS.md` 裁定215(`owns:` 32件の仕分け、(a)意見が強制14/(b)上流不在8/(c)測定器具8/(d)立証不足2)・裁定216(全文)
- `docs/ui-quality-bar.md` Q9(68-70行)・`next/GOALS.md` M20(31行)
- `next/reference/KNOWN.md` iced 節(全件、focus/Tab に触れた行なし — 既存 KNOWN に本件の記載は無い)
- `next/Cargo.toml`(88行、iced pin: fork `oshikaidesu/iced` rev `73e686ee05efd7d1b61cfea2647186b336d9ab9c`)

## 1. 上流に本当に無いのかの再検証(独立確認)

`GESTURES.md` §2 の実測を鵜呑みにせず、fork チェックアウトを自分で読み直した。

```
~/.cargo/git/checkouts/iced-1bbb4ed9d90ae4f8/73e686e/
```

(このディレクトリ名が `next/Cargo.toml` の rev `73e686ee05efd7d1b61cfea2647186b336d9ab9c` と一致することを
`ls ~/.cargo/git/checkouts/iced-1bbb4ed9d90ae4f8/` で確認済み — 別ワークスペースの古いチェックアウトを
誤って読んでいない)

- `widget/src/button.rs` — `grep -n "Key::Named\|is_focused\|focus\|Tab"` 空振り(0件)。`Focusable`
  トレイトの実装が存在しない(`grep -rn "impl.*Focusable" widget/src/*.rs` で `text_input.rs:496` の
  1件のみヒット、`button.rs`/`pick_list.rs`/`pane_grid.rs` はヒットしない)
- `widget/src/pick_list.rs` — `grep -n "Key::Named\|is_focused\|focus\|Tab\|Enter"` 空振り。実装内容を
  読むと(472-517行)Cmd+wheel での循環選択はあるが、キー入力での開閉・選択は無い。GESTURES.md の
  行番号(472-517)は実測と一致
- `widget/src/pane_grid.rs` — `grep -rn "Key\|keyboard\|focus"` 空振り。resize もパネル入替も
  キーボード経路が無い
- `widget/src/text_input.rs` — `text_input.rs:496` で `impl<R: text::Renderer> operation::Focusable for
  State<R>` を確認(`is_focused`/`focus`/`unfocus` を実装)。**入力自体は Focusable だが、Tab で
  そこへ辿り着く配線はどこにも無い**(`grep -n "Tab" next/shell next/ui/motolii-keymap` 空振り、
  GESTURES.md と同じ結果を独立に再現)
- `core/src/widget/operation/focusable.rs`(1-180行) — `focus_next`/`focus_previous`/`focus`/`unfocus`/
  `count` という一般 operation が**存在する**。ただし `focusable()` コールバックは
  `Operation::operate` の実装が明示的に呼び出さない限り叩かれない。**この trait を実装しているのは
  fork 全体で `text_input` だけ**(上記の1件のみ)なので、たとえ Tab を配線しても
  **`focus_next()`/`focus_previous()` が巡回するのは text_input 系フィールドだけであり、button /
  pick_list / pane_grid はそもそも巡回対象に入らない**。これは GESTURES.md §2 に明記が無かった
  新しい確認事項 — 「Tab 配線だけでは button/pick_list は救えない」という追加の切り分けが要る

**結論**: GESTURES.md §2 の主張(iced/iced_aw のどちらも「ボタン・ドロップダウン・パネル境界を
キーボードだけで操作する経路を無償で提供しない」)を独立に再確認した。加えて、**text_input だけは
Focusable 機構が既に存在し、Tab 配線さえすれば無償で使える**という非対称を追加で確認した(§3 施工案へ反映)。

### 上流 issue/PR の探索

探索範囲: `gh search issues --repo iced-rs/iced` で `"keyboard focus button"`・`"tab focus"`・
`"accessibility"` の3クエリ(GitHub 検索 API 経由、2026-08-23 時点)。上流リポジトリは
`iced-rs/iced`(Motolii の fork元)。

見つかった主要 issue:

- [iced-rs/iced#489](https://github.com/iced-rs/iced/issues/489)「In native UI, cannot focus most
  widgets, control them via keyboard, or tab between widgets」— **2020-08-23 open、2026-08-23 現在も
  open**(ちょうど6年)。ラベル `feature, shell, accessibility`。本文が Motolii の実測と同じ症状を
  報告している(button に Enter/Space で活性化できない・text_input 内で Tab が効かない)。コメント
  (2021-01-10、`semtexzv`)が `focusable` 属性+pre-order traversal という設計案を出しているが、
  **実装は着手されていない**(直近コメントは2026-05-24「頻繁に text field を並べると煩わしい」という
  未解決報告のみ)
- [iced-rs/iced#552](https://github.com/iced-rs/iced/issues/552)「Implement accessibility support」—
  2021年台起源、open。プラットフォームアクセシビリティ API(ATK/MSAA/NSAccessibility)統合の
  上位 issue で、#489 はこの下位互換問題として関連付けられている。要件リストのみでチェックボックスは
  すべて未着手
- [iced-rs/iced#1130](https://github.com/iced-rs/iced/issues/1130)「no focus; arrow keys and tab does
  not work」(2021、closed as **duplicate** of #489)
- 個別ウィジェットの小さい先行例として `iced-rs/iced#366`「Tour's Slider: keyboard accessibility」
  (2020、closed)はあるが、button/pick_list/pane_grid を対象にした個別 PR は見つからなかった

**「PR で進行中の動きが無い」と書く時の探索範囲を明記する**: 上記3クエリ+#489/#552 の
コメント欄(2026-08-23時点の最新コメントまで)を読んだ限りでは、iced 本体に
button/pick_list/pane_grid のキーボード操作を実装する進行中 PR は見つからなかった。
`gh search prs --repo iced-rs/iced "focus"` は行ったが該当 PR は0件(GitHub 検索 API のクエリ結果、
2026-08-23)。ただし **iced リポジトリ全体の全 PR・全ブランチを網羅したわけではない**(検索 API の
キーワード一致に限定した探索であり、無関係な語彙で focus 実装が進んでいる可能性はゼロと断言しない)。

**これが持つ意味**: #489 が6年 open のまま停滞していることは、「上流 PR を出して数ヶ月待つ」が
現実的な解決速度ではないことの一次資料になる(§3 施工案の判断材料)。

## 2. 裁定215 の判定 — 意見を名指しできるか

`OPINIONS.md` 18件を1件ずつ当たった。

| # | 意見 | 「キーボード到達性を自前で持つ」を強制するか |
|---|---|---|
| 1 有理時間 | 無関係 |
| 2 Preview=Export | 無関係 |
| 3 Intent 単一書き口 | 無関係(Document への書き込み経路の話で、UI 入力手段の話ではない) |
| 4 決定性 | 無関係 |
| 5 ゼロコピー合成 | 無関係 |
| 6 可視性原理 S6 | **近いが違う** — S6 は「唯一の入口が右クリックだけにならない」(複数の**視覚的入口**)を要求するもので、**入力手段**(マウス/キーボード)の話ではない。GESTURES.md 自身も M(Hidden)トグルの2箇所入口を S6 の例に挙げているが、その2箇所は両方ともマウス専用ボタンであり、S6 はそれで満たされている扱いになっている(§7 参照) |
| 7 意図優先の原則 | **強制はしないが方向を縛る** — 下記で分離して論じる |
| 8 1意図=1つの家 | 無関係 |
| 9 接続子は加算 | 無関係 |
| 10 Inspector 全項目が時間軸 | 無関係 |
| 11 無いと壊れているを同義にしない | 無関係 |
| 12 Undo は edit timeline | 無関係 |
| 13 選択・時刻・幾何は単一正本 | 無関係 |
| 14 順序非依存の明示参照 | 無関係 |
| 15 デザイン値は token 経由 | 無関係 |
| 16 単一世界・単一カメラ | 無関係 |
| 17 自前音声 mix | 無関係 |
| 18 拡張の口は trait 1本 | 無関係 |

**結論: 「キーボード到達性を自前で持て」と直接強制する意見は18件の中に無い。正直に「意見なし」と書く。**

意見7(意図優先の原則: UI の動詞は意図を語り機構を語らない)は近接するが、**強制するのは
「持つか持たないか」ではなく「持つ場合にどの形で持つか」**である。意見7 を機構露出(汎用 Tab
フォーカス+各 widget の Enter/矢印キー実装)へ当てはめると、それは「フォーカスという機構」を
利用者に意識させる経路であり、意図優先の精神とは逆方向になる。一方、`motolii-keymap`/`VerbId` の
既存パターン(Space=再生トグル、Cmd+Z=Undo、`NudgeKeyframe`=Alt+←/→)は「ウィジェットのフォーカスを
経由せず意図に直接キーを割り当てる」形であり、意見7 と一致する。

**つまり**: 意見7 は「作るかどうか」を強制しないが「作るならこの形」を強制する、という間接的な効き方をする。
これは裁定215 が想定する「これを外部の先例に置き換えたら製品は壊れるか」という直接テストには
通らない(現状フォーカス機構自体が存在せず、「置き換える」対象が無い)。よって**裁定215 の意味では
(b) 上流不在・意見なし**が正しい分類であり、意見7 を根拠に「自前で持つべき」と結論づけるのは
コストの出所を偽ることになる(`OPINIONS.md` 冒頭「コストを特定できない項目は意見ではなく、ただの言葉」)。

## 3. 施工案3つ

### 案A: iced fork に汎用フォーカス機構を足す(button/pick_list/pane_grid へ `Focusable` 実装+Enter/矢印キー処理+視覚フォーカスリング)

- **(a) 行数**: 3 widget × (フォーカス状態フィールド+`Focusable` 実装+`on_event` 内のキー分岐+
  `draw` 内のフォーカスリング描画)。text_input の `Focusable` 実装(`is_focused`/`focus`/`unfocus`+
  周辺の状態管理、`text_input.rs` 496行以降)を1widgetぶんの目安にすると、button/pick_list/pane_grid
  の3つで概算 **200〜400行**(pane_grid は分割木の中でどのペインがフォーカスを持つかという追加の
  状態設計が要るため、単純な button より重い)
- **(b) どこが壊れ得るか**: fork は既に `motolii/host-seams` ブランチで upstream から分岐している
  (`next/Cargo.toml` コメント、裁定170)。`button.rs`/`pick_list.rs`/`pane_grid.rs` は上流が将来
  #489 に着手した時に最も触られる可能性が高いファイル群であり、**rebase のたびに衝突するリスクが
  最大化する場所に自ら手を入れることになる**。視覚フォーカスリングは全 pane の意匠(token 経由、
  意見15)と整合させる追加コストも伴う
- **(c) 上流が同じ物を入れた時に捨てられるか**: 条件付き。upstream の実装形が Motolii の fork 実装と
  API 形状(`Focusable` の粒度・イベント名)が一致すれば置き換えられるが、#489 のコメントが示す設計案
  (`focusable` 属性+pre-order traversal)は現状 iced のイベント処理モデルと必ずしも一致しない
  ラフ案であり、**最終的にどう実装されるか未確定**。一致しなければ「捨てて入れ替え」ではなく
  「両方保持して段階的移行」という余計な作業が発生する

### 案B: Motolii 側に「動詞→キー」の層を作る(`motolii-keymap`/`VerbId` の延長、ウィジェットのフォーカスを経由しない)

正典 §8.1 が「正へ採用」と書きながら未実装の動詞(`FocusRowPrev/Next`・`MoveClipInToPlayhead/Out`・
`TrimInToPlayhead/Out`・`SetWorkAreaIn`)はこの形。既存の `NudgeKeyframe`(Alt+←/→)・
`TogglePlayback`(Space)と同型の拡張。

- **(a) 行数**: `next/ui/motolii-keymap`(現状 1090行・`VerbId` 38件)への追加は1動詞あたり
  `VerbId` variant+既定キー割当+`shell/lib.rs` 側の match 腕、という3点セット。GESTURES.md §1 が
  「マウスのみ」判定した25行のうち、専用キーボード動詞で救える意図(数値巡回系のボタン・M/S/L・
  行選択・clip move/trim・Loop帯・pick_list 選択等)は概算15〜20件、1件あたり10〜30行として
  **概算 200〜500行**。テキスト巡回(pick_list の一覧選択・LINK/MATTE 参照選択)はキー単発では
  表現しづらく、「候補を順送りで巡回するキー」(Cmd+wheel の既存循環選択のキーボード等価)という
  設計が追加で要る
- **(b) どこが壊れ得るか**: fork には触れないため rebase リスクはゼロ。リスクは Motolii 側に限定される
  — 既存キー割当との衝突(`!modifiers.command()` ガード等、KNOWN.md が既に指摘する Cmd+O 衝突の
  再発パターン)。また「フォーカスされている物が何か」という視覚的手がかりが無いまま動詞キーだけ
  増えると、Q9「フォーカスの所在は視覚で分かる」を満たせない(選択ハイライトで代用できる意図と、
  できない意図(例: pick_list の一覧を開かずに巡回する)が混在する)
- **(c) 上流が同じ物を入れた時に捨てられるか**: **捨てられない・そもそも代替関係にない**。
  `NudgeKeyframe` は Tab フォーカスの代替ではなく「時刻ドラッグの designated キーボード等価」という
  独立した意図表現(正典 §3 が明記)であり、上流が汎用 Tab フォーカスを実装しても Motolii は
  Space=再生・Cmd+Z=Undo 等の動詞キーを引き続き必要とする。**恒久的な独自資産になる**という前提で
  コストを見積もる必要がある(これは欠点ではなく、意見7 と一致した設計の帰結)

### 案C: 上流へ PR を出し、それまでは穴を穴として台帳に残す

- **(a) 行数**: 実装コストは変動(PR の受理設計次第)。ただし fork 側での即時採用コストはゼロ
  (マージされるまで Motolii は何も書かない)
- **(b) どこが壊れ得るか**: 何も壊れない(現状維持)。ただし#489 が2020年から6年 open のまま
  停滞している実績(§1)から、**この案は近い将来の解決を保証しない**。全面委任裁定
  (2026-08-17「利用者の役割=UX合否のみ、他は推奨で自走」)と時間軸が合わない可能性が高い
- **(c) 捨てられるか**: 該当なし(Motolii 側に何も実装しないため「捨てる」対象自体が無い)

### 案D(§1 で見つかった副産物、案に満たない最小パッチ): text_input への Tab 配線だけ先に済ませる

§1 で確認した通り、`text_input` だけは iced 本体が `Focusable` を既に実装済み(fork 改変不要)。
`operation::focusable::focus_next()`/`focus_previous()` を Tab キー押下時に `Task` として発行する
配線だけで、**Inspector の click→type 数値入力欄のあいだを Tab で移動する**という限定的な効果は
無償で得られる。ただし GESTURES.md が指摘する通り、click→type 欄自体への**最初の到達**
(click 依存)は解決しない(フィールド群の中を移動できるだけで、最初の1つに入るには依然 click が要る)。
**行数概算: 20〜50行**(Tab キー捕捉+`Task::widget(focus_next())` 発行+フィールド `Id` の割当が
まだの箇所への `Id` 付与)。これは案B の一部として吸収できる先行スパイクであり、独立した意見判断を
要しない(borrowed 機構をそのまま呼ぶだけ、新規 owns がほぼ無い)。

## 4. 推奨

**案Bを推奨する。** 理由:

1. **意見なしと確定した以上、コスト大の方(案A=fork 改変)を選ぶ根拠が無い**。裁定215 の既定は
   「借りる」であり、「持つ」を選べるのは意見を名指しできる時だけ。案Aは「意見なし」のまま
   fork という最も upstream 追随コストが高い場所へ手を入れる選択であり、既定に反する
2. **利用者裁定「意図優先」(裁定174)と「wrapper > 迂回」(2026-08-18)の両方と最も一致する**。
   `NudgeKeyframe` 等の既存パターンは「ウィジェットの機構(フォーカス)を経由せず、意図に直接
   キーを割り当てる」という wrapper 型の解決であり、迂回(汎用フォーカスをこじ開けて使う)ではない。
   案Bはこの既存資産(`motolii-keymap`/`VerbId`)の単純な延長であり、新しい設計判断を要しない
3. **案Cは時間軸が合わない**(#489 の6年 open が一次資料)。全面委任裁定の「他は推奨で自走」に対し、
   上流 PR 待ちは自走の対義語になる
4. 案Dは案Bの一部として無償で先に着手できる(fork改変ゼロ・意見判断不要)ので、**案Bの最初の
   施工単位として案Dから着手する**のが妥当

ただし案Bには明記すべき限界がある: **視覚フォーカスの所在表示が別問題として残る**(Q9 後半)。
動詞キーで意図を実行できても「今どの行/どのフィールドが対象か」が画面上で分からなければ
利用者は迷う。GESTURES.md §1 の「Timeline 行選択」行がまさにこの形(`FocusRowPrev/Next` を実装しても、
選択ハイライト = フォーカス表示という前提が成立するかは別途確認が要る)。**この限界は案Bを選ばない
理由にはならないが、実装レーンへの発注時に個別に明記する必要がある**。

案Cについては、コストがほぼゼロ(PR提出そのものは Motolii のリソースを大きく食わない)なので、
**案Bと並行して低優先度で上流へ現象報告(#489への追記や独立issueでの実測共有)を行う余地はある**が、
それを本裁定のブロッカーにはしない。

## 5. 迷った判断

- **意見7(意図優先)の扱い**: 「強制する」と「方向を縛る」を混同しかけた。`OPINIONS.md` の
  判定基準(「外部の先例に置き換えたら壊れるか」)に厳密に照らすと、フォーカス機構は現状
  存在しないため「置き換える」対象が無く、意見7 だけを根拠に「自前で持つべき」と言うのは
  コストの出所を偽る(意見が無いのに意見があるかのように書く)。**「意見なし、ただし作る形は
  意見7 と整合させる」という書き方に倒した** — 誠実性(捏造しない)を優先
- **案Dを独立の4番目の案として立てるか、案Bに畳み込むか**: 独立させると「4案から1つ選ぶ」形式に
  ずれて発注書の「3つ」という指示と食い違う。**案Bの内部の最初の施工単位として位置づける形に
  倒した**(発注書の形式を守りつつ、fork改変ゼロで得られる部分を隠さない)
- **§1 の「Tab を配線しても button/pick_list は救えない」という追加確認をどこに置くか**:
  GESTURES.md 本体には書けない(NON-GOALS で書き換え禁止)ため、本文書の§1に新発見として明記し、
  RETURN でも独立して報告することにした

## 裁定案(採番は supervisor)

- A02(キーボード到達性)は **意見なし・(b)上流不在** として `OPINIONS.md` には追加しない
- 施工方針として**案B(動詞→キー層の拡張、`motolii-keymap`/`VerbId`)を採用**、案Dをその先行
  スパイクとして着手順の先頭に置く
- 案A(iced fork 改変)は不採用(コスト大・既定「借りる」に反する・意見の裏付けが無い)
- 案C(上流PR)はブロッカーにせず、低優先度の並行タスクとして扱ってよい
