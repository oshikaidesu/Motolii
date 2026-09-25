# UI rebaseline — Phase A report(監査・コード変更なし)

2026-09-25。正本の指示は [brief.md](brief.md)。原則:

> **Current UI is the capability oracle, not the layout oracle.**

この文書は Phase A の 6 項目(写真・目録・持ち主の地図・直書き監査・移行分類・Classic を残す境目)の到達点。
Phase B 以降はここを入口にする。

| 置き場所 | 中身 |
|---|---|
| [concept/](concept/) | Concept Art 3 枚(dark が第一案)。visual grammar の参照で、pixel target ではない |
| [classic/](classic/) | 現行窓の写真 18 枚(下の索引) |
| [inventory/](inventory/) | Capability Inventory 本体 5 本、計 **約 640 行**。各行に trigger・条件・file:line・送る命令・disposition |
| [audit-visual.md](audit-visual.md) | theme 構造・直書きの数・共通部品・色の意味・font・token 案 |

目録はコードを全部読んで作り、実窓で照合した(メニュー・popover・全 panel を開いて撮影)。
右クリックメニューは窓を前面に出さないと開けないため、コードからの列挙だけで実窓では未照合。
Stage は既存 build で画が出ておらず(Rust の native build が古い)、
**Stage の画そのものは今回の監査対象外**とした(利用者判断: 「ただのビュー」)。

## 1. 写真(Classic)

| file | 状態 |
|---|---|
| `00-default.png` | 起動直後。Files・Camera・Inspector・Timeline・Desk(Depth) |
| `01-menu-view.png` | View メニュー: 17 panel + Reset layout |
| `02-menu-composition.png` | Composition popover: 比率 preset・幅高さ・尺・背景・fps |
| `03-menu-export.png` | Export popover: All / Marker to marker・MP4 H.264+AAC |
| `04-menu-settings.png` | Settings: tile size・Key the start too・新規 layer の空間・panel 配置表・color theme JSON・Outside dim・Scale |
| `05-menu-file.png` / `06-menu-edit.png` | File(Open〜Rerun Script)/ Edit(Undo〜Ungroup) |
| `10-create.png` 〜 `14-colors.png` | Browser の棚: Create / Media / Effects / Fonts / Colors |
| `15-desk-ease.png` 〜 `18-desk-history.png` | Desk: Ease / Blend / Notes / History |
| `19-desk-depth-notes.png` / `20-web.png` | Depth と Notes 同時・Web |

## 2. Capability Inventory — 集計

| 本 | 行 | 主な disposition |
|---|---|---|
| [browser.md](inventory/browser.md) | 134 | PRESERVE 134・MOVE 11・MERGE 7・CONTEXTUALIZE 7・VISUALIZE 5 |
| [stage.md](inventory/stage.md) | 100 | PRESERVE 96・VISUALIZE 10・MOVE 6・CONTEXTUALIZE 5・MERGE 5 |
| [inspector.md](inventory/inspector.md) | 142 | PRESERVE 135・**VISUALIZE 30**・CONTEXTUALIZE 9・MOVE 8・MERGE 4 |
| [timeline.md](inventory/timeline.md) | ~150 | PRESERVE 171・MERGE 6(Timeline・Desk・Export) |
| [shell.md](inventory/shell.md) | ~90 | PRESERVE 89・MOVE 9・CONTEXTUALIZE 6・MERGE 2(menu・shortcut・workspace・settings) |

disposition は組み合わせ可(`PRESERVE + MOVE` 等)なので合計は行数を超える。**DELETE は 0**。

### 目録で分かった、移行で落としやすい物

- **Browser でしか出来ない編集がある。** `applyEffect`・`replaceAsset`・`relinkAsset`・`removeAsset`・`setFillMode`・`styleText`(字の大きさと範囲)・`text_justify`・`setColor`/`previewColor`(色の輪)。Inspector は font・色の欄を「Fonts / Colors 棚を開く」釦としてしか持たない。New UI でこの経路を切らないこと。
- **押している間だけの操作。** P/R/S を押しながら 3D gizmo を絞る(3D layer の拡縮は S を押す以外に道が無い)、⌘ を押しながら移動で吸着、Space+drag で pan。
- **Timeline の隠れた結合。** 見えているコマ数が新規 layer の既定の長さになる(`visibleFrames`)。timing drag と key の drop は native の返事の前に絵を先に出す(preview/commit)。これが無いと bar が一瞬戻る。
- **Desk の自動追従。** 選択に応じて Ease / Depth が開く(`desk.dart:48-56`)。Desk の中で作業中は追従を止める。
- **dormant の Browser tab strip。** 棚は今それぞれ独立した dock panel で、Browser 自身の tab 列(BR-001)は本番では届かない。New UI で tab 付き Browser に戻すなら、この道が生き返る。

### 現行にあると思われがちだが無い物(New UI で「在る」と描かない)

Front/Top などの名前つき視点(Front は 3D view の reset だけ)、Stage 上の作成 drag・文字の直接編集・anchor の drag・guide/grid/ruler・Stage の右クリック menu・trackpad pinch。
Timeline の loop/work area・layer の rename・利用者の色ラベル・snapping・marker の移動と削除(Rust にはあるが UI から呼ばれない)。
Export の形式・codec・解像度の選択。Inspector の式入力・単位つき入力・anchor の数値入力。Recent files。

Concept Art の PERS/TOP/FRONT/RIGHT・Relations・EDIT/PLAY/EXPORT もこの側。**実装しない**(brief §3)。

## 3. 持ち主の地図(要点)

詳細は [shell.md §2](inventory/shell.md)。

| 状態 | 持ち主 |
|---|---|
| Document・undo/redo・history 台帳・**選択**・**再生位置**・再生中・Animate・User Stage の camera・clipboard・export job | Rust `EditorRuntime`(1 process 1 個) |
| 再生の刻み・IOSurface・texture・`layout.json` の読み書き | Swift host |
| Rust status の写し・panel 間の焦点(`editingFocus`・`focusProperty`・`textStyleTarget` 等)・利用者の設定(`deskWork`・`panePlaces`) | Dart `EditorSession`(1 engine 1 個) |
| dock の木・分割比・tab | `workspace/*` |
| 入力中の下書き・Stage 2D の zoom/pan・Timeline の lane 展開・Browser の選択と検索 | 各 panel の widget |

**Domain logic は大半が Rust にあり、二重化の危険は小さい。** 例外は widget の中に住む次の 7 つ。
New UI が同じ事をするなら、**写さず、振る舞いを変えずに widget の外へ出して共有する**。

1. menu → 命令の対応と、保存確認の順序(`app/editor_window.dart:347-432`)
2. panel 配置の状態機械 tab/window/drawer/hidden(`editor_window.dart:273-328`)
3. trim・slip・move の時間計算(`panels/timeline/grip.dart:400-435`)
4. marker-to-marker の書き出し範囲と進捗の poll(`panels/export_controls.dart:32-100`)
5. 選択 → Desk の引き出しの規則(`panels/desk.dart:48-56`)
6. composition の寸法・fps の preset(`panels/composition_controls.dart`)
7. freeze 通知の文言と進捗(`editor_window.dart:23-38`)

`input/editor_shortcuts.dart`(nudge・前後の layer・全選択・F9)と session の既定 ease は、既に widget の外の共有 class にあり、そのまま使える。

## 4. Classic を残す最小の境目

詳細は [shell.md §3](inventory/shell.md)。今は `EditorWindow` が **session を作る役**と **Classic の配置**を兼ねていて、配置を選ぶ点が無い。

1. `SessionHost` を `app/editor_app.dart:65` の `home:` に置く。`EditorSession` の生成・initialize・dispose を `EditorWindow` から移し、shell に依らない host callback(file drop 等)を張る。
2. `ClassicShell` = 今の `EditorWindow`。session を引数で受ける以外は **byte 単位でそのまま**。
3. shell の選択は `--dart-define=MOTOLII_SHELL=new|classic`(既存の `MOTOLII_DOCUMENT` と同じ仕組み)。既定は `classic`。後で View menu の切替も可。**同時に mount する shell は常に 1 つ**。切り替えても session は生き残る。
4. New shell は既存 panel を `panels/registry.dart:20` の `buildPanel` 経由で使い、GlobalKey の map は自前で持つ。
5. 切り離した panel 窓(`main:false`)は shell によらず Classic の leaf のまま。

**守る物(破ると壊れる)**

- `NativeBridge` の listener は static で 1 つ。session を 2 つ作らない。
- `stageWindow` は Rust に 1 枠しか無い。Stage の dispose が `{0,0}` を送るので、User Stage を 2 つ同時に出さない。切替中の重なりにも注意。
- `_shownViews` は参照数を数えない集合なので、同じ view を 2 か所で出すと片方の detach で消える。
- `layout.json` は丸ごと上書きされる。New shell の配置は `deskWork` の中(`storeDesk` = 読んで・混ぜて・書く)に置くか、両方の persist を merge にする。
- `panePlaces` の語彙(tab/window/drawer/hidden)は Settings ▸ Panels・Desk・起動時の復元が読む。New shell も矛盾しない値を出す。

## 5. 見た目の監査(要点)

詳細は [audit-visual.md](audit-visual.md)。

- **直書きは少ない。** lint 2 本(`raw_color`・`raw_dimension`)が効いている。theme 外の `Color(...)` は 3 か所で、全部 data 由来。
  散らばっているのは「どの token を選ぶか」の方で、inline `TextStyle` が約 123、場当たりの alpha が 43、角丸が 31。
- **角丸 4〜5px** が floating menu・number field・color field・ease desk にあり、brief の 0〜3px を超える(New UI 側で直す。Classic は触らない)。
- **色の衝突は 09-20 より広がっている。** layer 色 = property family 6 色の並べ替え。同じ青が Shape・Image・Browser の選択枠を兼ねる。
  選択の表し方は 5 通り(橙・水色 tab・淡い水色・青・白)。橙 1 色が on 状態・key・再生位置・marker・marquee・drop 先・caret を兼ねる。
- **font は Inter(400/500/600)だけで、mono は無い。** tabular figures は 3 か所だけ。
- **注入の道は既にある。** `EditorTheme` は Material `Theme` の extension として配られ、各部品は `EditorTheme.of(ctx)` で読む。
  New shell の subtree だけを別の `EditorTheme` と新しい `ThemeExtension<ShellTokens>` で包めば、**Classic のコードは 1 行も変えずに**既存部品の色が変わる。
  ただし painter 12 個は `EditorTheme.chromatic` / `EditorInk.dark` を既定で直に読むので、New UI で使う時は theme を明示で渡す。
- `ShellTokens` の最小の名前は audit §6。accent 6 色は 1 色 1 役割。layer 識別色と property family は別の列にして、衝突を再生産しない。
- mono は、まず Inter + `tabularFigures` で readout を作る。asset の費用が 0 で、brief の「tabular numbers を優先」を満たす。mono の同梱は後で試す。

## 6. New UI での到達先(Phase B の下書き)

四面構成(Concept Art)を基準にした、能力の群ごとの行き先。行ごとの詳細は各 inventory の disposition 列。

| 面 | 入る物 | 主な disposition |
|---|---|---|
| **上の帯** | File / Edit / View の menu、Composition・Export・Settings の popover、書類名と dirty | PRESERVE(形だけ新しく) |
| **左 Browser** | Create・Effects・Media・Colors・Fonts・Files を **1 つの panel の tab** に | MERGE(dock 6 枚 → tab 付き 1 枚。dormant の tab 列を使う)。Files は tab の 1 つへ MOVE し、常時の大面積をやめる。全機能は残す |
| **中央 Stage** | Stage / Camera の view 切替、Fit・zoom・Extend・透明地、gizmo・吸着・eyedropper | PRESERVE。view 切替は Stage 上部へ MOVE。存在しない視点は出さない |
| **右 Inspector** | 見出し(layer 識別色・名前・ロック)→ Transform / World / 内容(Text・Shape・Fill・Layout・Camera…)/ Effects | CONTEXTUALIZE(選択の種類で内容の section が変わる)。Font・色は Browser への直通口として PRESERVE し、色の輪は Browser ▸ Colors に置いたまま(採用事項 l.16) |
| **下 Timeline** | transport・zoom・marker・layer 行(◇ M S L ↳)・key・lane | PRESERVE。行の色は layer 識別色の列へ(§5) |
| **下の Desk** | Ease・Depth・Blend・History・Notes・Web | Timeline と並ぶ tab 群へ MOVE。選択への自動追従は保つ |

**VISUALIZE の候補**(将来の 2D gadget。今回は未来機能を偽装しない)は Inspector だけで 30 行ある。
Field Falloff → field、stagger / ghost → 空間×時間の坂、ease → curve、gradient → gradient、Depth → 既に俯瞰図。
Inspector の section は「図 + 精度の数値」を後から差し込める形(section の頭に gadget の枠)で作り、今は数値行だけを置く。

## 7. 既存の不具合・効いていない操作(直していない、記録のみ)

移行で「同じ動き」を保つ対象なので、ここで直すと Classic との差になる。直すなら別の作業として扱う。

- `setMatte` を Inspector が送るが、Rust に handler が無い。Matte の欄は常に無効。
- `U`(keyedOnly)と Settings ▸ Outside dim は効いていない(dim は 0.55 固定)。
- marker は足せるが、UI から動かせず消せない(Rust の `setMarker`・`deleteMarker` は呼ばれていない)。
- Space+drag の pan で再生も切り替わる。
- 右 drag の orbit 中に Esc を押すと、orbit が残るように見える(要実窓確認)。
- eyedropper が、orbit/zoom した Stage 上の点ではなく出力の座標を読む。
- Files 棚の「Cannot read this folder」が表示されない。
- Media で複数選択を collection へ drag すると、1 枚しか入らない。
- Inspector の Reset は effect の値にしか効かない。
- Camera の Distance の欄名が「Scale」になっている。
- `hanging_punctuation` が Text の欄に出ない(id が `text` で始まらないため)。
- [product-contract.md](../product-contract.md) l.55 が、今は使っていない Material component theme を記述している(古い)。

## 8. 利用者の判断が要る物

brief §13 の 3 条件(統合で挙動が変わる・Document/Intent の変更・Classic と New で意味が分かれる)に当たる候補。
**既定は「統合しない・今の挙動のまま移す」なので、判断が出るまで Phase B は止まらない。**

1. **同じ意図で刻みが違う物。** Stage の +/− は 1% 刻み、⌘=/− は ×1.2。Timeline の +/− は 1% 刻みで高倍率ではほぼ動かない。揃えると挙動が変わる。
2. **focus で意味が変わる鍵。** ←/→ は Timeline に focus がある時だけ key を動かし、他では再生位置を動かす。Timeline の Esc は全体の Esc(選択解除・Desk を閉じる)を飲む。採用事項の「見えない状態で同じ操作の意味を変えない」に反するが、揃えると片方の挙動が変わる。
3. **複数選択の書き先がばらばら。** Space は選択中の全部、Parent・Environment・Clip は表示中の 1 つ、Ghost は選べる全部に書く。
4. **選択の当たり判定。** クリックでは Group を選べないが、marquee では選べる。
5. **地の色。** Stage の透明地スイッチと Composition の背景 preset は同じ値を触り、preset を押すと透明が黙って切れる。

どれも New UI では**今の挙動のまま**移し、統合はしない。直したい物があれば番号で指示してほしい。

## 9. 後始末・注意

- 撮影で View menu から panel を開いたため、利用者の `~/Library/Application Support/MotoliiStage5/layout.json` が変わっている。
  撮影前の控えを取ってあり、窓を閉じた後に控えから戻す(開いている間に戻しても、窓が次の保存で上書きする)。
- Phase B の最初の一手は §4 の 1〜3(`SessionHost`・`ClassicShell`・`MOTOLII_SHELL`)。これは Classic の見た目と挙動を変えない切り出しで、Classic の widget test がそのまま通ることを合格の条件にする。
