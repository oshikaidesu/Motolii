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
- **引き出しは焦点の型に一つ。** 色→サークルと preset、キー→curve と preset、列挙→サムネイル格子、文→全文。数の型は何も出さない。型に紐付かない例外は履歴の 1 つだけ。新しい Vism が来ても param が既存の型なら引き出しは増えない。
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

1. **headless の窓 test 16 本が赤。** HEAD 時点で赤(stylesheet が harness に当たらず全 zone が縦積み)。実窓には出ない。**宿題の先頭** — これが赤いままだと以降の直しで緑を頼れない
2. 右上の panel(Inspector / Desk)が他の panel と同じ手で dock できない。未再現
3. 欄の持ち主を `Session.field` 1 つにし、本文・数値・文字・名前の 4 実装を 1 component へ。歌詞は Blitz の `textarea` へ委託(§6b)
4. 色型の引き出し(サークル、書き戻し口は shape/text の data)、参考画像の貼り場(素材と同じ口、Timeline に置かれない role)
5. 仕切りを引きすぎると vello が溢れて落ちる(`vello_encoding config.rs:185`、8/30 から既知)。tile に最小寸法の柵は立てた。根は未特定
6. 配置の永続化、机の pin と複数 instance
