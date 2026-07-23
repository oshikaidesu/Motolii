# M3 detachable panel / multi-window契約

作成日: 2026-07-22

状態: **決定／isolated fixture実装可**。永続workspace形式、公開plugin API、製品WebView/native統合は変更しない。

2026-07-22追補: Timeline / Graphだけでなく、**全ての通常製品panel**をdetach / re-dock / tab化 / split resize /
top-level resize可能にする。Stage、Browser、Inspectorを例外にしない。

## 1. 決定

Timeline ViewとGraph Editorは同じdock位置で切替できるだけでなく、別top-level windowへdetachし、別monitorへ
移動できる製品panelとする。この能力は二面の特例にせず、Stage、Browser、Inspector等の製品panelへ同じ
placement modelを適用できるようにする。modal、popover、toastはpanelではなく対象外である。

```text
Host coordinator: revision付きsnapshot / selection / focus intentの唯一のowner
  ├─ Window A: 1 wgpu Surface + native viewport群 + opaque WebView dock stack群
  ├─ Window B: 1 wgpu Surface + native viewport群 + opaque WebView dock stack群
  └─ Window C: 同じ構成（必要なpanelだけを投影）
```

通常window内は従来どおり1 top-level Surfaceである。detachしたtop-levelは独立Surfaceを持つが、Document、Undo、
selection、playhead、Graph/Timeline channelを複製しない。全windowはHostの同じrevision付きsnapshotをread-only投影する。

## 2. placement model

panel identityとsurface実装を分ける。

- `PanelId`: panel instanceの安定識別。role名やwindow番号から状態を逆算しない
- `PanelRole`: Timeline、Graph、Stage、Browser、Inspector等の表示能力
- `PanelPlacement`: dock stack内、detached top-level、hiddenのTransient/Workspace候補
- `WindowId` / surface generation / logical bounds / DPI / monitor / fullscreen: OS session状態
- Document revision / selection / playhead: Host coordinator状態

panel geometryはheadless layout treeで解く。Flexbox/Gridをrendererやwindow systemから独立して提供する
`taffy 0.12.2`をisolated fixtureへ限定して採用し、logical size、min size、split比からrectangleを導出する。
`taffy`へpanel identity、dock操作、selection、snapshot、window lifecycleを所有させない。

初回fixtureでは永続codecを作らない。window位置やDPIをDocument、D2、plugin契約へ入れず、reopenは既定boundsと最新Host
snapshotから再投影する。将来workspace保存を採択する場合も別version付きUser/Workspace形式として審判する。

## 3. detach / re-dock規則

1. detachはpanelのrendererやsemantic stateを移す操作ではなく、placementとprojection targetを切り替える
2. native panelはtarget top-levelのSurface内viewportへ再投影し、surface間texture共有を状態同期に使わない
3. React panelはtarget windowの同一version bundleをrole付きで起動し、DOM stateを正本にしない
4. close、crash、surface lostは対象windowだけを破棄し、Host snapshotと他windowのpresentを維持する
5. re-dockは最新revisionを投影してから旧targetをretireし、同じpanelを二つのwriterとして残さない
6. focusとshortcutはactive window/panelからHostへintentを送り、surface別keymapを作らない
7. panel間drag中のcapture loss、window外release、Escは同じgesture tokenをCancelし、Document変更ゼロ
8. 全通常panelは同じ`dock / detach / tab / hide / resize`能力を持つ。role別にresize可否をhard-codeしない
9. split dividerはwindow logical sizeに対する比としてTransient previewし、releaseでWorkspace候補へ1回反映するがDocument/Undoは0
10. top-level resize、minimize、最大化、DPI変更は各windowのlayout epochを進め、全panel rectangleを同epochで再計算する
11. min sizeを下回るdragはclampし、負値、NaN、Infinity、0×0 windowをlayout treeへ通さない
12. StageはReact header / transportとnative Preview viewportを一つのpanel identityで移動する。内部の非重複rectはresizeに
    追従するが、個別のdock対象や状態ownerへ分裂させない

GraphとTimelineを同時表示する場合も、Graph selectionとTimeline selectionを同期し合わない。両方が同じHost selectionを読む。

## 4. isolated合格条件

- TimelineとGraphを2 top-level / 2 Surfaceへ投影し、共有device/queue 1組、GPU readback 0
- 両windowが同じsnapshot revision、stable selection、playheadを読む
- 片側close/reopen、resize、fullscreen、疑似surface lostで他方のpresentとHost stateが不変
- detach/re-dockでsemantic commit 0、Undo 0、projection owner 1
- 異なるwindowから同じgesture tokenを二重commitしない
- panel roleをenum分岐だけで閉じず、新しいfirst-party panelをplacement modelへ追加できる
- Stage / Timeline / Graph / Browser / Inspectorの全roleでdetach→resize→re-dock→tab化が同じ結果になる
- horizontal / vertical split、nested split、tab stack、window resizeでrectangleが重ならずbounds内に収まる

既存G0-10のEditor / detached Preview実機fixtureは2 Surface lifecycleの証拠として再利用する。ただしPreview専用roleを
一般panel契約へそのまま昇格せず、今回のheadless placement fixtureでTimeline / Graphと任意panelの同型性を固定する。

## 5. 未証明

- 異DPIの第二monitor、HDR/SDR、Windows WebView2、実device lost
- React panelを含むwindow間Tab/focus/IME/AX graft
- 永続workspace layout、community panelの別realm detach
- 製品D2、正式dock library、製品renderer採択

## 6. isolated実施結果

`spikes/g0-9-timeline-visual-parity`へ一般dock treeを追加し、Stage / Timeline / Graph / Browser /
Inspectorの5 roleを同一操作へ通した。全roleでdetach、detached top-level resize、tab再ドック、split再ドックが成立し、
Host snapshot revision 17、selection `pulse-rings`、semantic commit 0を維持した。

矩形計算だけを`taffy 0.12.2`へ委ね、nested horizontal / vertical split、tab stack、window resize、logical min size、
split ratio clamp、非finite入力拒否を自動試験で固定した。OS側はG0-10の2 top-level / 2 Surface fixtureを再利用する。
したがってheadless配置契約とOS multi-Surface lifecycleは個別に成立したが、製品panel renderer、WebView、実マウスdock
chromeを同時に接続した結合試験は未証明のままである。
