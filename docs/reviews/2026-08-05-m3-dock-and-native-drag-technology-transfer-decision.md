# M3 Dock Host / OS native drag技術移管決定

- 日付: 2026-08-05
- 状態: **決定**。Dock接続は分割し、content dragは置換probeを待つ。
- 対象: `U1e`、`CU-405`、`CU-407`、Browser Rectangle → native Stage

## 1. 利用者成果の背骨

1. 利用者はHost chromeのtab/titleを掴み、任意の通常panelをtab、split、別top-levelへ移せる。
2. 移動中はHostが一つのpreviewとdrop overlayを表示し、release一回でplacementだけを確定する。
3. native Stage / TimelineとReact Browser / Inspectorは同じpanel modelのGuestであり、panel種別ごとに別の移動機構を持たない。
4. BrowserのRectangleを掴むとOSがdrag session、capture、cursor、window境界を所有し、native Stageが最新layoutでdropを一回だけ受理する。
5. どちらの操作もDocument、journal、Undo、selection、playheadのownerを増やさず、cancel / stale / unsupported platformは変更0で戻る。

panel placementとcontent dragは同じpointer問題に見えるが、source、payload、commit先が異なるため一契約へ統合しない。

## 2. Dock Hostの採択

```text
MECHANISM CLASS: multi-window dock tree / native chrome drag / arbitrary guest projection
KNOWN IMPLEMENTATION SEARCH: detachable panel契約、surface topology、in-repo DockWorkspace、wry 0.55.1 reparent、KDDockWidgets architecture文書、Dockview文書
CANDIDATES: in-repo DockWorkspace + Taffy、wry reparent、KDDockWidgets Core/View/Guest pattern、Dockview、Tauri全面導入
ADOPTION ROUTE: REUSE(in-repo placement/oracle) + WRAP(wry reparent) + PATTERN(Core/View/Guest)
REJECTED CANDIDATES: KDDockWidgets/OBS source copy=GPL系、Dockview global owner=React内だけ、Tauri全面導入=既決境界と責任を増やす
THIN MOTOLII SEAM: existing LayoutAuthority / ProductApp / NativeHostLayout / BrowserHostRuntime / InspectorHostRuntime / ProductSurface
THIN MOTOLII RESIDUAL: PanelIdとGuest binding、layout epoch、single projection owner、Document write 0 oracle
IMPORTED RESPONSIBILITY: wry 0.55.1のplatform reparent、winit window lifecycle、既存Taffy projection
EXIT: reparent adapterとmulti-window projection registryだけを交換境界にする
RETIREMENT: fixed single-window LayoutAuthorityをparity後に一回cutoverし、isolated DockWorkspace duplicateを証拠fixtureへ戻す
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN
```

KDDockWidgetsは、dockの意味をrendererやGuest contentから分ける`Core / View / Guest`の比較先に限る。GPL sourceを読んで移植せず、公開architecture文書で確認できる責任分割だけを採る。Motoliiでは既存`LayoutAuthority`がCoreの起点、native tab/title/drop overlayがView、wgpu viewportとopaque WebViewがGuestになる。

`wry 0.55.1`のplatform extensionにある`reparent`は、macOSの`NSView`、Windowsの`HWND`、LinuxのGTK containerへ既存WebViewを付け替える原始機能として採る。DOM stateを正本にせず、reparent失敗時は旧parentを維持してplacementを確定しない。新しいWebView manager、別UI framework、Tauri全面導入は作らない。

## 3. Dock接続の分割

| ID | 一契約境界 | owner / target | primary oracle | 状態 |
|---|---|---|---|---|
| `U1e-DH1` | single-window layout authorityをstable `PanelId` / stack / split / window placementへ拡張する | `layout.rs` / `layout_authority.rs` | current 4面geometry byte-equivalent、全panel同型detach/re-dock、失敗時candidate不採用、Document write 0 | `DO` |
| `U1e-DH2` | `ProductApp`のtop-level window / shared GPU / per-window Surface projection registry | `product_runtime.rs` / existing G0-10 evidence | 2 window、device/queue 1、各window Surface 1、片側closeで他方とHost snapshot不変 | `WAIT U1e-DH1` |
| `U1e-DH3` | WebView Guestのparent切替 | Browser / Inspector Host runtime + wry `reparent` | latest epochの一parentだけ可視、old parent callback/write 0、失敗時旧parent維持 | `WAIT U1e-DH2` |
| `U1e-DH4` | native tab/title threshold、capture、preview、drop overlay、release/cancel一回 | Host chrome / latest layout | preview>=1、terminal=1、placement publish=1、Document/journal/Undo=0 | `WAIT U1e-DH3` |

`U1e-DH1`で第二layout ownerを追加しない。現行`LayoutAuthority`のcandidate検証とatomic adoptionを残し、その所有範囲をwindow/stackまで広げる。製品runtimeへ未接続のDock Coreを並置する案は棄却する。

## 4. Browser content dragの再採択

```text
MECHANISM CLASS: Web content sourceからnative surface destinationまでのcross-window drag session
KNOWN IMPLEMENTATION SEARCH: CU-0B04S/P、CU-107PV/TC/AD/TD、Apple NSDraggingSession、Windows OLE DoDragDrop/RegisterDragDrop、GTK4 DragSource/DropTarget/GdkDrag
CANDIDATES: OS native drag session、現行AppKit local NSEvent monitor、HTML5 local lifecycle、汎用pointer framework
ADOPTION ROUTE: ADOPT(OS drag session) + REUSE(existing typed identity/admission/D2 chain)
REJECTED CANDIDATES: local NSEvent monitor=WebView境界後moveとpreview消失の実機反証、HTML5 local terminal=release二重/欠落、generic framework=三OS標準より責任増
THIN MOTOLII SEAM: BrowserPlaceIntent payload adapter、latest NativeHostLayout drop target、existing CU-107 admission、PendingStageDrop
THIN MOTOLII RESIDUAL: Rectangle identity codec、Stage canonical conversion、terminal at-most-once、negative trace oracle
IMPORTED RESPONSIBILITY: AppKit NSDraggingSession、Windows OLE DnD、GTK/GDK DnDのsession/capture/cursor/window crossing
EXIT: platform source/destination adapterだけを交換し、CU-107以降を不変にする
RETIREMENT: OS session product parity後にHostPointerCaptureのPlace用途とlegacy HTML5 terminalをFROZEN -> RETIRE
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN
```

AppKitは`NSView.beginDraggingSession(with:event:source:)`へmouse-down `NSEvent`、`NSDraggingItem`、`NSDraggingSource`を要求し、sessionがmoveとended/cancelを通知する。Windowsは`DoDragDrop`が`IDataObject` / `IDropSource`とdrop targetをmessage loop上で仲介し、GTK4は`GtkDragSource` / `GtkDropTarget`または`gdk_drag_begin`が同じ役割を持つ。三系統ともHostがraw moveをwindow外まで再構成する方式ではなく、OS drag sessionがglobal lifecycleを所有する。

ただし現行wry IPC callbackはtyped bodyだけを渡し、AppKitが要求するmouse-down eventまたは既に開始済みのnative session handleをHostへ渡さない。WKWebViewのHTML dragが生成するOS sessionをnative destinationへ接続できるかも通常製品routeで未証明である。ここを推測してplatform adapterを作らない。

したがって次の一粒`CU-0B04OS-P`はMac通常製品の置換probeに限定する。固定Rectangle identity一件について、(a) WKWebView sourceが開始したnative dragをHost destinationが受け取れる経路、または(b) supported APIで同じmouse-down eventからHostが`NSDraggingSession`を開始できる経路のどちらか一つを実証する。合格条件はbrowser-intent=1、Stage内move>=1、native preview/overlay>=1、terminal=1、command=1、publish=1、extra timeline hit=0、orphan generation=0。完成済みMoved/ReleasedのRust直接注入だけでは合格にしない。

probeが(a)/(b)の双方を否定した場合だけ、macOSではWebKit/AppKit間の追加supported seamを再調査する。Windows/GTK adapter、自由panel overlay、汎用drag frameworkをMac probeへ混ぜない。局所`WAIT_TARGET`でも`U1e-DH1`以降のDock laneは継続する。

## 5. 正本訂正と非目標

- `CU-0B04P`のunitと当時の実Mac receiptは歴史証拠として維持するが、現在の通常製品でWebView境界後moveとpreviewが消えた反証により、Placeの現行採用routeとはしない。
- `CU-107PV/TC/AD/TD`のtyped identity、分類、at-most-once admission、単一下流配送は置換後も再利用する。pointer取得方法だけをcutoverする。
- overlay sceneとpointer captureを同一ownerへ潰さない。OS sessionがcapture/lifecycle、Hostのlatest layout projectionがdrop overlayを所有する。
- workspace永続形式、community panel権限、plugin API、Document schema、D2 command意味、Windows/Linux製品完成、custom canvas frameworkは本決定の非目標である。

## 6. 一次資料

- [KDDockWidgets Architecture and Concepts](https://docs.kdab.com/kddockwidgets-manual/latest/architecture_and_concepts.html)
- [wry `WebView`](https://docs.rs/wry/0.55.1/wry/struct.WebView.html)
- [Apple `beginDraggingSession(with:event:source:)`](https://developer.apple.com/documentation/appkit/nsview/begindraggingsession%28with%3Aevent%3Asource%3A%29)
- [Microsoft OLE Drag and Drop](https://learn.microsoft.com/en-us/windows/win32/com/drag-and-drop)
- [Microsoft `DoDragDrop`](https://learn.microsoft.com/en-us/windows/win32/api/ole2/nf-ole2-dodragdrop)
- [GTK4 Drag and Drop](https://docs.gtk.org/gtk4/drag-and-drop.html)
- [GDK `gdk_drag_begin`](https://docs.gtk.org/gdk4/type_func.Drag.begin.html)
