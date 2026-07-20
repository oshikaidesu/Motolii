# M3 U1a-2 panel layout投影契約

作成日: 2026-07-21
状態: **決定 / 実装待ち**

## 1. 目的

U1a-2は、U1a-1の中央Stage shellを次の組み込み配置へ広げる。

```text
+---------+----------------------+-----------+
| Browser | Stage                | Inspector |
+---------+----------------------+-----------+
| Timeline                                   |
+--------------------------------------------+
| status                                     |
+--------------------------------------------+
```

正本はMotolii所有のtoolkit非依存layout intentであり、`egui_tiles`は編集可能なruntime
投影である。panel配置はDocumentの作品意味ではない。U1a-2ではin-memoryの意味と投影だけを
閉じ、Workspace profileへの保存形式、version、未知field保全、破損fallbackはU1a-3へ送る。

## 2. 役割と組み込みpresetの閉集合

U1a-2が持てるsurface roleは次の5つだけである。

| role | 組み込み位置 | U1a-2での内容 |
|---|---|---|
| Browser | Stageの左 | roleを識別できる空surface |
| Stage | 中央の残余領域 | U1a-1の同じnative texture |
| Inspector | Stageの右 | roleを識別できる空surface |
| Timeline | 上3面の下 | roleを識別できる空surface |
| Status | 最下端 | 診断を持たない空の固定chrome |

Browser / Inspector / Timelineの3補助paneはresize、hide、restoreできる。Stageは残余領域の
必須paneでありhideできない。Statusはdock tileではなく最下端の固定chromeであり、
hide、tab、split、drag対象にしない。StatusへBrief、Context、diagnostic、activity、
transport、再生状態を仮実装しない。

Browser内部のSources / Collections / Packs階層rail、検索、Results、Browser分類は
Browser surfaceの内側を実装する後続ticketへ送る。U1a-2のBrowserは単一roleであり、P48の
rail幅・開閉をlayout modelへ混ぜない。

## 3. Motolii layout intent

layout intentは`motolii-ui`内のprivate、非serde型とする。少なくとも次を表せる。

- 固定5 role。runtime treeへ入るpane roleはBrowser / Stage / Inspector / Timelineの4つ
- 水平または垂直splitと、複数paneを同じ領域へ置くtab group
- split childの正の整数相対share
- tab groupのactive role
- 3補助paneのvisible / hidden
- 組み込みpresetへ戻すreset

roleとnodeの識別子は同一process内でruntime提案を照合するための意味IDであり、公開ID、
保存ID、plugin ID、Document IDではない。文字列から生成せず、5 roleの閉じたenumと
private node keyを使う。各roleはlayout全体にちょうど一度だけ存在し、duplicate、欠落、
未知role、循環、空container、非有限または0以下のshare、Stageを含まない可視treeを
型付き拒否する。

組み込みpresetの初期shareはprivate constructorが決定的に作る。shareは比であり、
window px、DPI、monitor IDをmodelへ保存しない。具体的な製品spacing、separator幅、
panel最小値、色、radiusはU0e-3のvisual tokenではないため、本契約から決めない。

各splitのmodel shareは正の`u32`比とし、最大公約数で約分する。runtimeの有限な正の
`f32` share proposalは、各値を合計で割った後にprivate固定解像度1,000,000単位へ
largest-remainder法で量子化する。同率の余りは固定preorderで先のchildを優先し、全childへ
1単位以上を割り当て、最後に最大公約数で約分する。非有限、0以下、child数が解像度を
超えるproposalは拒否する。この整数比はruntime正規化と自動oracleだけのprivate表現であり、
U1a-3のwire形式を決めない。

## 4. runtime投影と権限ループ

1. frame開始時に、検証済みMotolii layout intentから固定preorderで
   `egui_tiles::Tree`を構築する
2. `TileId`はその投影中だけの対応表に閉じ、model、公開API、log、保存候補へ書かない
3. pointerによるresize、tab化、split移動、closeは、runtime treeの直接確定ではなく
   **runtime edit proposal**として受け取る
4. proposalをrole、container位相、正規化share、active role、visibilityから成る
   toolkit非依存候補へ変換し、§3の全不変条件と§6のStage制約を全体preflightする
5. 合格時だけMotolii intentを一回で置換する。不合格時は元intentを保持し、次frameで
   元intentから再投影する。部分反映しない
6. runtime treeを保持cacheとして再利用してもよいが、同じintentから破棄・再構築して
   同じcanonical signatureを得られなければならない

`egui_tiles::Tree`のin-place mutation、`Behavior::on_edit`、share、visibilityを
「すでに保存された正本」と扱わない。runtime proposalをMotolii intentへ正規化しないまま
次frameへ持ち越す実装は禁止する。

canonical signatureは`TileId`とmap iteration順を除き、固定preorderのcontainer種別、
axis、role、前段の約分済み整数share、active role、visibility、および固定Status chrome
roleから成る。
同じintentを異なる空のruntimeへ2回投影したsignatureは一致しなければならない。

## 5. split / tabのU1a-2範囲

U1a-2は`egui_tiles`との権限往復を証明するため、Browser / Inspector / Timelineの
固定3補助paneを操作subjectとして次の最小操作を扱う。

- 補助paneをStageまたは別の補助paneの左右または上下へ移す1段以上のsplit proposal。
  Stageはsplitのanchorにはできるが、Stage自身を移動しない
- 補助paneを別の補助paneと同じ領域へ置くtab proposalとactive tab変更。
  Stageをtab groupへ入れない
- separator resize
- Browser / Inspector / Timelineのhideとrestore
- separator単位または全体の組み込みpreset reset

新しいpaneの生成、同一roleの複製、任意plugin pane、別window、別monitor、floating surface、
無制限の製品ドッキング体験はU1a-2の完成条件ではない。実window上で
split/tab/resize/hide/restoreを製品として受け入れるspikeはM3実装ガード9どおりU1eで行う。
U1a-2は固定roleに対するmodel操作列とruntime proposal往復を自動試験し、自由panel機構の
完成を主張しない。

## 6. resize、Stage残余、単位

- Browser / Inspector / Timelineのpointer resizeはruntime proposalを§4の同じ経路へ通す
- focus中separatorのArrow操作はprivate layout actionへ正規化し、Homeとdouble clickは
  そのseparatorの組み込みshareへ戻す
- 全体resetは固定View構造入口から組み込みpresetへ戻す
- hideされた補助paneのrestoreも同じView構造入口から行い、Status文言、Browser内部、
  diagnosticを復帰入口にしない
- separatorの局所keyboard操作はfocus中widgetのaccessibility操作であり、
  global shortcutやDocument intentではない。`DomainIntent`、D2 command、Undoへ追加しない
- pointer capture喪失、window focus喪失、またはIME非active時のEscapeで継続resizeを
  cancelし、gesture開始前のintentへ戻す

Motolii intentが持つ寸法は相対shareだけである。Stage最小幅の判定に必要なviewport幅と
最小幅は、egui logical point単位のprivate adapter入力として注入し、model、Document、
Workspace profile wire候補へ保存しない。U1a-2の自動試験は複数の注入値で、補助paneを
拡大してもStageへ指定最小幅が残るようshareをclampすることを確認する。製品用の具体値は
U0e-3以後にtokenまたはadapter policyとして決め、Reactの`440px`等を転記しない。

## 7. raw inputの限定adapter

U0d-3時点のraw toolkit input許可fileゼロは、U1a-2の局所separator操作に限って次のように
狭く改訂する。

- 許可するproduct sourceは`motolii-ui`内の単一private layout runtime adapterだけ
- 読めるraw keyはfocus中separatorに対するArrow / Home / Escapeだけ。pointer resizeと
  double clickは標準`egui::Response`から読む。安全中断に限り`egui::Event::PointerGone`と
  `egui::Event::WindowFocused(false)`を読み、それ以外のraw pointer/window eventを
  走査しない
- 出力はprivate layout action、または既存のtoolkit非依存
  `SafetyInterrupt::{PointerCaptureLost, WindowFocusLost}`であり、`DomainIntent`を直接
  発行しない
- 入力には`ImeGateState`を必須で渡す。`PreeditActive`中はArrow / Home / Escapeを
  layout actionへ変換せずIMEを優先する
- raw安全eventは一度だけ対応する`SafetyInterrupt`へ正規化し、同じ値を
  `InputRouter::route(NormalizedInput::SafetyInterrupt(..))`とprivate layout reducerへ
  配送する。routerは既存のglobal in-flight gestureだけを、layout reducerは継続中の
  resizeだけをcancelする。layout reducerはrouterが返す`DomainIntent`を受け取らない
- PointerCaptureLost / WindowFocusLostはIME状態によらず継続resizeをcancelする。
  EscapeはIME非active時だけprivate cancel actionへ変換し、どのIME状態でも
  `SafetyInterrupt`へ読み替えない
- `CommandId`、keymap resolver、Document command、作品意味へ接続しない
- adapter外のraw egui/winit inputは引き続きAST監査でゼロを要求する
- toolkit型をlayout intent、公開signature、他crateへ出さない

実装PRは許可fileを固定し、別module、alias、helper、macro経由のraw inputを拒否する
負例を追加する。製品global shortcutの例外として本節を一般化しない。

## 8. Statusと構造入口

Statusは5面presetの位置と境界だけを成立させる固定下端chromeである。U1a-2では内容を
空またはrole識別用の非診断placeholderに限定する。Statusの高さはlayout intentのshareや
保存候補に含めず、private adapterの一時値とする。

View構造入口はhide済み補助paneの一覧、各restore、全体resetだけを標準menu/buttonとして
提供し、eguiの既定keyboard focus/activationを使う。独自raw key分岐を追加しない。これは
Browser / Stage / Inspector / Timeline / Statusに続く第6のsurfaceではなく、layoutを
操作不能にしないためのshell chromeである。文言、icon、色、spacingの製品完成は主張せず、
U0e-3の共通componentへ後で置換できるprivate実装に留める。

## 9. Document不変と公開境界

layout操作列の前後で、同じ`Arc<Document>`について次をすべて不変とする。

- canonical serialize bytes
- revision
- journal
- Undo / Redo
- evaluation結果とU1a-1 display slot identity

U1a-2のin-memory layoutはG0-2の`UiStateOwner::WorkspaceProfile`分類に従うが、
Workspace profileのversion付きcodec、保存先、読込、未知field原本保全、破損fallbackを
実装しない。これらはU1a-3の単独境界である。

layout型、runtime proposal、projection signature、`egui_tiles`型を`motolii-ui`の公開面へ
出さない。egui familyの依存を他の製品crateへ追加せず、Document schema、plugin契約、
render/eval APIを変更しない。

## 10. 必須負例

- `egui_tiles::Tree`、`TileId`、crateのserde形、egui memoryを保存または公開する
- runtime treeの編集結果をpreflightせず次frameの正本にする
- duplicate、欠落、未知role、循環、空container、不正share、Stage非表示を受理する
- Browser内部railを別paneとして作る
- StageまたはStatusをhideまたは操作subjectにし、Stageを移動・tab化する、あるいはStatusを
  drag・tab・splitする。Stageを補助paneのsplit anchorにすることは許可する
- Statusへdiagnostic、activity、transport、再生状態を仮実装する
- hide後の復帰をStatus文言、Browser内部、test hookだけに置く
- Reactのpx値、CSS、色、spacing、文言を製品契約へ転記する
- resize、hide、tab、split、resetでDocument、journal、Undo、評価、display slotを変える
- panel操作用`DomainIntent`、D2 command、公開layout API、serde deriveを追加する
- 許可adapter外でraw egui/winit inputを読む
- 同一roleの複製、新規plugin pane、floating/別windowをU1a-2へ入れる

## 11. U1a-2完了条件

1. 組み込みpresetがBrowser左、Stage中央、Inspector右、Timeline下、Status最下端を持つ
2. 同じintentを空runtimeへ2回投影したcanonical signatureが`TileId`によらず一致する
3. 固定3補助paneに対するsplit、tab、active tab、resize、hide、restore、separator reset、
   全体resetの操作列がMotolii intentへ正規化され、再投影後も一致する。Stageはsplit
   anchor以外の操作対象にならない
4. 不正proposalは型付き拒否され、元intentへ部分変更がない
5. 複数viewport/minimum入力でStage最小幅を保ち、px/DPIをintentへ焼かない
6. hide済み補助paneを固定View入口からkeyboardでrestoreできる。Stage/Statusはhide不能
7. U1a-1 Stage textureを同じslot/TextureIdで投影し、layout操作でrender/copy/registerを
   増やさない
8. layout操作列の前後でDocument serialize、revision、journal、Undo/Redo、評価が不変
9. layoutはprivate・非serdeで、Workspace profile codec、toolkit型公開、他crateへの
   toolkit依存、Browser内部意味、Status診断、visual token、別windowを追加しない
10. raw input許可は単一private adapterだけで、範囲外をAST負例が拒否する
11. `cargo fmt --all -- --check`、`./scripts/check-docs.sh`、
    `./scripts/check-ui-toolkit-deps.sh`、`cargo clippy --workspace --all-targets -- -D warnings`、
    `cargo test --workspace`が通る

これを満たした時だけU1a-2を完了とし、U1a-3を混ぜず次の直列ticketへ進む。
