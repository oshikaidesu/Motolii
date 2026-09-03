# Inspector の今後と、机 — 2026-09-02 利用者との対話の裁定

状態: **決定**(1〜6)。7 は最初の一枚を実窓で見てから。

## 1. Inspector は複数選択を受ける

基本は Transform の共通行。選んだ層に共通する Vism があればそれも出す。全体適用なら Group で足りるが、別の意図があり得るので閉じない。

## 2. 関係の行は「選ぶ」でなく「手つき」にする

ラジオ的な選択肢は直観的でない。既に手慣れた作法へ認知負荷を預ける。

| 関係 | 借りる手つき |
|---|---|
| 親子 | AE のピックウィップ |
| マット | クリスタ / Photoshop のクリッピング(直上の層)。任意の層を選ぶのは advanced 項目として残し、そこでもウィップで指す |
| flatten | フリーズ後にだけ選べる。**不可逆**(Undo では戻る、値としては戻せない) |
| ブレンド | 文字で選ばない。机のサムネイル格子(§4) |

## 3. 机 — 常設の一枚、Document を覗くレンズ

Stage=絵、Timeline=時間、Inspector=値、Browser=素材。**長い物**を置く面が無かった。マーカーの本文、Undo の履歴、ease の preset、blend のサムネイル、参考画像。

- **常設なのは場所であって機能ではない。** 顔は書き置き・参考画像・カササギ。履歴や preset は引き出しで、開けるまで見えない。
- **机は誰にも呼ばれない。** 他の面が「机を開け」と言う口を持たない。机は Document と焦点を読むだけ。書くのは Intent 経由で構わない(Inspector と同じ)。
- **引き出しは焦点の型に一つ。** キー→curve と preset、列挙→サムネイル格子、文→全文。数の型は何も出さない。型に紐付かない例外は履歴の 1 つだけ。新しい Vism が来ても param が既存の型なら引き出しは増えない。**色は例外(2026-09-03 利用者裁定): 机に出すと視線がごちゃつくので Browser の Colors に輪を常設し、焦点の色に付いて行かせる。**
- **焦点**は Inspector が行を光らせて共有 read model に書く。Inspector は机を知らない。焦点の行と机の見出しが同じ物を指す 1 本の線を視覚で通す。
- **preset は人の物、参考画像と本文は作品の物。** 机は 2 層。作品層は `.rrd` に、人の層はユーザー設定に。
- **机は増やせる。** 一枚目は焦点に追従。二枚目以降は pin して型に留める(Blender の editor area)。panel 種は増えない。pin と複数 instance は最初の一枚を見てから。
- **引き出しは浮かせない。机の中に出る。** 開けた間だけ机の tile が上へ広がり(Inspector が縮む)、閉じたら畳んでいた割合へ戻る。実窓で右下に浮かせる形を試して却下(2026-09-02 夜)。Blitz では `position: fixed` が親基準なので浮かせる層は座標測りが要り、それも理由。
- **顔に欄を常設しない。** 書き置きは Text の引き出しの中。歌詞を置く前提なら欄でなく文書で、編集は Blitz の `textarea`(上流)へ委託する。これは欄の持ち主を 1 つにする手(下記)と一緒に。
- 書き置きはマーカーの本文。別 entity を作らず `Marker.body`。Timeline は名前だけ、机は全文。

先例: Ableton Clip View / Info View、Photoshop History、Figma Version History、Clip Studio Sub View、PureRef、Blender N パネルと editor area、Apple Motion HUD。

## 4. ブレンドはサムネイル、hover で Stage が preview

Photoshop / Figma / Affinity / Procreate は一覧を hover するだけで絵が変わる。サムネイルは Stage が隠れている時の補助。

## 5. カササギは机に居る

2026-08-08 の 3 条件(ペット無しでも全状態が読める、非表示で判別が劣化しない、UI 密度の予算を侵さない)は、机に載る物が全部ユーザーの物である事で満たす。鳥は何も示さない。

## 6. 面は三段

| 段 | 何 | 数 |
|---|---|---|
| 常に出る | Browser、Stage、Inspector、Timeline、Desk | 増やさない |
| 引き出し | 焦点の型に一つ + 履歴 | 型が増えた時だけ |
| ヘッダにしまう | Settings、Export の進捗 | 作品でなく窓の都合 |

Utility(ANCHOR)は Inspector の Transform 行へ。Output は File▸Export と status bar へ。Ease は机になる。Panel は 11 → 5 で、憲法の「家は 5 つ」と揃う。

## 6b. 欄は許可制(2026-09-02 夜、利用者裁定)

押すまで欄は無い。押したら出る。Enter・Escape・**外を押す**・窓を離れる、のどれでも欄ごと消える。欄が無ければ鍵は窓の物。
窓に `input` が 1 つでも在れば打鍵が全部そこへ行く規則(`host::aim_keystrokes`)の裏返しで、閉じ損ねた欄は鍵の全喪失になる。今夜の「Space が効かない」は全部これだった。
当座の橋: 欄の外を押したら host がその欄へ Enter を送る(`commit_field_outside`)。本丸: 欄の持ち主を `Session.field` 1 つにし、本文・数値・文字・名前の 4 実装を 1 つの component へ畳む。

## 7. 未決

- 数の型へ焦点が移った時、前の引き出しを開けたままにするか(実窓で見る)
- 色の引き出しの書き戻し口(text fill / shape fill は property でなく shape data)
- 参考画像の admit(素材と同じ口、Timeline に置かれない role)
- 焦点とキー選択の新旧(今は焦点が在れば焦点、無ければキー)

## 8. 宿題(2026-09-03 朝、利用者裁定: 細々した整備は後回し。責任は分かれているので後からで直る)

2026-09-03 の片付け(実窓では未確認 — 見た目と手触りの合否は利用者):

1. **済。** 原因は 2 つ重なっていた。(a) `asset!` の stylesheet は harness に配る net が無く当たらない → test では同じ文面を inline で当てる(`tokens::stylesheet`)。(b) 赤の一部は CSS が当たって初めて見えた製品の穴: 窓の外で放した pointerup は blitz が root へ落とし `#app` に届かない(host の `on_primary_pointer_release` は在ったが窓無しでは未登録) → `Host::HEADLESS` を決めて harness も同じ線を通す。View menu の「✓ Timeline」を選んでも menu が閉じず覆いが次の押しを食っていた → 選んだら閉じる(Reset Layout・macOS と同じ)。数を決め打ちした test は 11 面時代の残り → 面の数から測る。PointerLeft を release 扱いする routing は外した(縁で dock してしまう)
2. 未再現のまま。1 の「外で放すと届かない」が同族の可能性あり
3. **済。** 4 実装を `semantic_menu::Field` 1 component へ、持ち主は `Session.field`(`OpenField { at: FieldAt, draft }`)1 つ。書き置きは `textarea`、Enter は改行、確定は Cmd+Enter(外を押した時に host が送る鍵も同じ)
4. **済。** `Focus::Color(ColorSlot)`。Inspector の COLOR 行が焦点、机の引き出しは色相の輪(conic-gradient)と彩度・明度の面。書き戻し口は property でなく data(text の style の fill / stroke、shape の木の葉の fill)。掴んでいる間は下書きで放した時に 1 回だけ書く(Undo 1 手)。参考画像は `Asset.role`(Material / Reference)。窓へ落とした先が `#desk` なら Reference で admit し、Browser には出さず机の顔の帯に thumbnail で並ぶ。preset(人の層)は未
5. **未特定。** `vello_encoding 0.10 config.rs:185` は `bin_data(1<<18) - bin_data_start` の減算で、draw object が数万を超えた時に落ちる。custom widget の paint は面 0 で描かず、tick は `MIN_PPS`、波形は列が幅ぶんで有界。疑うのは DOM 側の文字(細い tile で折り返し)。実窓の再現手順が要る
6. **配置は済。** `~/Library/Application Support/Motolii/layout.json` へ Dock を serde で仕舞い、起動時に読む。別窓は仕舞わない。机の pin と複数 instance は §3「最初の一枚を見てから」のまま(利用者裁定待ち)
7. **半分済。** host の 4 つの規則(押す前の `commit_field_outside`、放しの `primary_pointer_released` と「窓の外か」、打鍵前の `aim_keystrokes`、開いた欄の全選択 `select_new_field`)を harness が同じ順で通す(`gui.rs` の press / click / release / key / settle / lose_focus / drop_files)。規則そのものは host に居る。本丸は上流: blitz の autofocus は要素の作成時にだけ属性を見る(`mutator.rs:961`)ので、Dioxus が属性を後から付ける `#app` には効かない。`set_attribute` でも見る 1 行の fork が要る
8. **上流に壁。** blitz-dom の `custom-widget` feature は既に `accessibility` を連れてくるが、`Widget::accessibility_tree` は pin した rev でも upstream main(2026-09-03 時点)でもコメントアウト。custom widget が AccessKit node を申告する口が無く、fork か上流 PR が先。kittest 0.4.0 は registry に在る

「普通こうなる」の通し(2026-09-03 午後): 開いた欄は全選択・同じ値は書かない・空の名前は名前でない・窓を離れたら確定・書き置きは再生に逃げない・参考画像は × で外せる・画でない物は棚へ・落とした物は押さなくても落とした先に出る・tab は窓の外でだけ別窓へ(余白では何もしない)・色は引き出しの外へ出ても確定・配置は放した時に 1 回だけ書く。全部 harness の test で赤→緑


1. **headless の窓 test 16 本が赤。** HEAD 時点で赤(stylesheet が harness に当たらず全 zone が縦積み)。実窓には出ない。**宿題の先頭** — これが赤いままだと以降の直しで緑を頼れない
2. 右上の panel(Inspector / Desk)が他の panel と同じ手で dock できない。未再現
3. 欄の持ち主を `Session.field` 1 つにし、本文・数値・文字・名前の 4 実装を 1 component へ。歌詞は Blitz の `textarea` へ委託(§6b)
4. 色型の引き出し(サークル、書き戻し口は shape/text の data)、参考画像の貼り場(素材と同じ口、Timeline に置かれない role)
5. 仕切りを引きすぎると vello が溢れて落ちる(`vello_encoding config.rs:185`、8/30 から既知)。tile に最小寸法の柵は立てた。根は未特定
6. 配置の永続化、机の pin と複数 instance
7. **UX の振る舞いが harness で捕まらない(病巣、2026-09-03 利用者)。** 打鍵の規則(`aim_keystrokes`・外クリック確定・focus 喪失)が winit の host に居て harness が通らない。上流 Blitz の焦点 model(autofocus が属性の付く前に可否を見る bug)を直して host の規則を消す、または規則を Blitz の event driver 層へ移し、窓と harness を同じ道にする
8. 試験の駆動を kittest(rerun 製、AccessKit 駆動)へ委託。Blitz の `accessibility` feature を有効にし、custom widget(Stage・Timeline・Ease)は Motolii が AccessKit node を申告する。画は vello_cpu の golden、振る舞いは kittest

実窓での確認(2026-09-03 午後、computer-use で background 操作): 名前のダブルクリック → 全選択 → 打てば置き換わる、Escape で戻る、外を押して確定、数字の欄に 42 → 42.0、Colors の輪と面で Fill が変わる(Undo 1 手)、`*` でマーカー → 机の Text → 書き置きに改行入りで打って外を押して確定、まで通った。実窓でだけ出た穴 2 つを直した: 全選択が editor の生成前に送られて caret が先頭に残る(host)、Colors の見出しが細く折り返す・書き置きの改行が潰れる(CSS)。未確認: 参考画像の drop(Finder からの drag は background 操作では出来ない)、日本語 IME(raw input では IME を通らない)、tab を窓の外へ出す。気になる観察: Stage の枠が左上に小さく描かれ、ホイールで拡大すると見えなくなる(私の変更の外、要確認)。

## 9. persona の通し(2026-09-03 夕、利用者の進め方: 並列に出して最大公約数を一撃で)

AE 10 年・Premiere/Resolve の編集者・Figma 育ち・初めてのリリックビデオ、の 4 人に code と test を歩かせた。4 人が重なった所を 1 batch で入れた(harness test 6 本、96/96 緑):

- **数値スクラブの終わり方**: Escape で掴む前へ戻る(Undo に残らない、選択も消えない)。面の外・窓の外で放してもそこで確定(host の release 経由)。擦りの持ち主は `Session.scrub`
- **文字の入口**(初めての人の離脱点): 本文は複数行(Enter は改行、Cmd+Enter で確定)。時間を開けていない限り(キー 1 つ以下)打った文字は差し替えで、勝手にキーが増えない
- **色の到達と下見**: COLOR 行を押すと Colors が前に出る(`Session.panel_ask`)。文字の色は掴んでいる間 Stage に出る(property の transient)。shape の塗りは data だけなので放した時
- **指の記憶**: Cmd+S、Enter で名前、M で印。印を打ったら本文を書く場所(Text)が開いている。Edit menu に Undo / Redo
- 打鍵ごとの `PROBE` 出力を消した

- **複数選択の Inspector**(§1): Transform の共通行。違う欄は「—」(Figma の Mixed)で掴めるまま。擦れば全部が同じ差分、打てば全部が同じ値。Cmd+click の多重選択は macOS では Cmd が SUPER なので META しか見ていなかった code を両方受ける形に(実窓で効いていなかった)

- **級数**: 文字の Inspector に Size の数の行(style の size を property が上書きする、resolve と同じ順)。擦れる
- **選んでそのまま動かす**(Stage): 押した層を選び、その手で Move を始める(押し直し無し)。実窓で確認済(2026-09-03 夕: 未選択の層を 1 回の drag で選んで動かせた)。あわせて、朝から気になっていた「Stage の左上の小さな枠」は bug でなく fixture の層(メインボーカル映像)だった

4 人が挙げて**入れていない物**(次の batch): Blend の hover preview と本物のサムネイル(§4、attrs に transient が無く、blitz に mix-blend-mode も無い)、Stage の文字を直接ダブルクリックで打つ、書体と級数の欄、J/K/L と I/O、印を掴んで動かす、履歴の行に操作名(Document に操作名が無い)、J/K/L(Clock に速度が無い)。

補記(2026-09-03): §8-5 の「vello の overflow」は起動時の「Tried to draw an invalid empty image」だった。renderer が 2 回作られ、前の renderer の id を持つ retained scene が出る競合。Stage は engine-up で描き直し、画像 id は毎 paint 登録 → 次の paint で外す形にした(decision-index 参照)。
