# UX chaos — 無茶な操作列の受入

日付: 2026-09-02

利用者前提: 実際の使用者は、操作を終えてから次へ進まない。掴んだまま別面へ行き、連打し、
menu/file-drop/focus lossを割り込ませ、Undo/Redoを境界より多く押す。

## 着手時の網

- proptestは24 case × 最大39手のPress/Motion/Release/素キーを生成
- 事後条件は「描画panelが残る」「tabが残る」「最後のtab clickが順序を変えない」の3つ
- focus loss、file-drop、wheel、修飾shortcut、chorded buttons、animation clock、double clickは生成しない
- 嵐後のmenu/dock復旧、Document書込、Save/Open、overlay/gesture終端は確認しない
- deterministic testはmenu、dock、splitter、file-drop、focus lossを個別には持つが、相互割込みを持たない

## 検索receipt

| field | 内容 |
|---|---|
| `NEED` | 正常な一操作は通るが、異なるgestureやoverlayが途中へ割り込む操作列でghost、capture、menu、dirty stateが残るか分からない |
| `SEARCHED` | 現行`gui.rs`/GestureSurface/Host、品質バーQ3/Q6/Q9、`gesture-tests.md`、W3C Pointer Events 3、WPT pointer capture、UIKit gesture state、Android touch slop |
| `DISPOSITION` | **REUSE_WRAP**。別fuzzerや独自input runtimeを作らず、pin済み`blitz-test-harness`のcontrolled inputと既存proptest shrinkingを拡張 |
| `OWNER/ROUTE` | raw event列→Blitz/Dioxus→既存GestureSurface/Dock/Document。試験は観測し、製品修正が必要な時だけ既存cancel ownerへ接続 |
| `RULER` | [W3C Pointer Events 3](https://www.w3.org/TR/pointerevents3/)はmenu/modal等で`pointercancel`、up/cancel後のcapture解放、chorded buttonの`buttons`変化を規定。[UIKit recognizer](https://developer.apple.com/documentation/uikit/uigesturerecognizer)はcontinuous gestureがEnded/Cancelledへ必ず到達。[Android touch slop](https://developer.android.com/develop/ui/views/touch-and-input/gestures/viewgroup)はtapとdragの誤認防止をplatform値から得る |
| `ORACLE` | 生成列の後にactive gesture/ghost/dropmap/menu/file overlayが0、Reset LayoutとCreate Rectangleが通り、Document Save/Openが成立。割込みmatrixは変更ゼロまたは一Commitだけ |

## 今回増やす操作族

1. primary + secondaryのchorded move
2. focus loss
3. file drag enter/leave
4. wheel up/down
5. Cmd+A/D/G/Z/Shift+Cmd+Z、Alt+[、Escape
6. animation clock tick
7. window外座標を含むpress/move/release

## deterministic割込みmatrix

| 開始中 | 割込み | 合格 |
|---|---|---|
| dock tab drag | File menu | drag Cancel、ghost/dropmap 0、menuだけ開く |
| splitter drag | File drag enter | splitter Cancel、layout不変、file overlayだけ開く |
| layer rename | Cmd+D / Delete | Document shortcut 0、text editorがowner |
| idle selection | Escape連打 | selectionだけ解け、Document revision不変 |
| Undo/Redo境界 | 100往復 | panic 0、最終Document一致 |
| 任意storm | recovery | overlay/gestureを閉じ、Reset Layout→Create Rectangle→Save/Open |

## 実施結果

### 自動試験

- proptestを **64 case × 最大95手**へ拡張。primary/secondary重ね押し、窓外座標、
  wheel、focus loss、file enter/leave、animation tick、修飾shortcutを同じ列へ混ぜた
- 嵐の終端でpointer release→focus loss→Escape×2を行い、active gesture、dock ghost、
  drop map、menu、file overlayがすべて0であることを確認した
- 復旧oracleはReset Layout→Create Rectangle→Save→Open。同じlayer数へ戻るまでを確認した
- `cargo test --locked --lib ui::gui --no-fail-fast`: **28/28 PASS**
- `cargo test --locked --no-fail-fast`: **109/109 PASS**

### 割込みで発見し閉じた穴

| 穴 | 修正 | oracle |
|---|---|---|
| menuを開いてもdock/splitter/shared gestureが生き残る | menubar pointer downをcancel境界にした | drag中menu→ghost/dropmap 0、menuだけ開く |
| Finderのfile enterがlocal gestureを生かしたままoverlayを出す | `DragEntered`でHost focus-loss cancelを先に通す | splitter不変、file overlayだけ開く |
| rename inputからCmd+D/Delete/Escapeがroot keymapへ漏れる | rename中はroot shortcutを止め、inputが全keyを所有 | 文書増減0、Escapeはrenameだけ取消 |
| 窓外Pressの反復後、focus lossでもmenuが残る | focus-loss終端でmenuも閉じる | 4手へ縮小したseedと決定的test |

### 実窓

通常製品windowへ、Create連打、rename中のCmd+D/Delete/Escape、12 panel/menu切替、
Cmd+A/D/Z/Shift+Cmd+Z/G/Shift+Cmd+G/Alt+[、Stage drag、wheel往復、Escape連打、
tab drag、splitter急移動を一続きで投入した。panic 0、ghost/dropmap/overlay残留0。
最後にView→Reset Layout→Create→Rectangleが通り、layer数が17→18へ増えた。

実窓のStage dragは選択が無く編集不成立だったため、直後のUndoは直前の作成を戻した。
これはログの`history moved=true`と、回復時のRectangle作成で因果を区別した。

## 残る外部gate

- native window外で物理releaseした時のcapture終端
- 複数pointerのidentity / primary競合
- coalesced / predicted eventの順序と取り込み
- Finderからの実ファイルdrag enter中に別gestureを保持する実機試験

現在判定: **PASS**。上の4項目は合格に含めず、machine ledgerで`PENDING`を維持する。
