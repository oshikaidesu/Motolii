# New Shell — routing(inventory の各項目が New からどこで届くか)

正本は [inventory/](inventory/)。ここは「New UI ではどこから届くか」だけを書く。
**NOT YET ROUTED** は New から届かない項目。偽装しない。Classic(`MOTOLII_SHELL=classic`、既定)からは全部届く。

起動: `MOTOLII_SHELL=new scripts/motolii-ui.sh dev [document]`。1 回の起動で shell は 1 つ。同じ Document・session・Rust runtime。

## Workspace(2026-09-26)

New の面は固定 layout をやめ、既製の dock(`docking`)に載せた。各 panel は dock の tab で、`c.panelPlacementRequested` は tab を前へ出す(閉じていれば開き直す)。
下の表の「左の面の tab」「中央の面の tab」「右下の Desk」は、既定の並びがそうなっているという意味で、利用者が動かせる。

- 配置は `settings` の `newWorkspace`(version 1: `docking` の木と大きさ、各 strip で前に出ている tab)に、他の key を保ったまま書く。読めない state は既定の並びに戻す。Classic の `persist` は `layout.json` を丸ごと書き換えるので、Classic を起動すると `newWorkspace` は消える。
- Stage は dock の tab で「前にいるか」を `Visibility` で判断する。dock がそれを渡す(`DockWorkspace.view()`)。隠れた Stage は window を 0x0 に戻し、resize と移動では見せる範囲を出し直す。
- 閉じた panel は名前で呼べば開き直す(SH-18・WS-21)。Desk の引き出し(`deskDrawer`)は今までどおり。
- Console は独立の panel(`NewConsole`)。Graph はまだ存在しないので席も作らない。
- capability の突き合わせ結果は new-shell-audit.md (`git:912382f048:docs/product/ui-rebaseline/new-shell-audit.md`)。

## Phase B(scaffold, 2026-09-25)

New Shell は Classic の panel を四面に固定して載せただけ(写真: [new/phase-b-scaffold.png](new/phase-b-scaffold.png))。
配線の確認用で、visual projection としては不合格(利用者裁定)。Phase C でこの表を面ごとに書き換える。

| inventory | New での到達先 |
|---|---|
| BR-*(Browser) | 左の面の tab: OBJECTS(=Create)/ EFFECTS / MEDIA / COLORS / FONTS / FILES。棚の中身は同じ |
| BR-001・BR-013(dormant の tab 列・`browserTab` の reveal) | 左の面の tab 列が受ける(`c.browserTab` を聞く) |
| ST-*(Stage) | 中央の面の tab: STAGE / CAMERA |
| IN-*(Inspector) | 右上の面 |
| IN-073..079(Blend desk)・DK-*(Desk 各種) | 右下の Desk。選択への自動追従はそのまま |
| DK-050..067(Notes)・Web | 中央の面の tab: NOTES / WEB |
| TL-*・EX-*(Timeline・Export) | 下の面 / 上の帯の EXPORT |
| SH-01..17(File・Edit) | 上の帯の FILE / EDIT(`editor_actions.dart` を Classic と共有) |
| SH-18・WS-21(panel を名前で出す) | 上の帯の VIEW → その panel の tab か Desk の引き出しを前へ |
| SH-20・SH-21(Composition・Export) | 上の帯の COMPOSITION / EXPORT |
| SH-23・SH-24(書類名・状態行) | 上の帯の右端・最下段 |
| SH-26(保存確認)・SH-36(閉じる時の確認) | `confirmClose` を New も張る(共有関数) |
| SH-27(file の drop) | New も `filesDropped` を張る |
| ST8-01..03(tile size・Animate・新規 layer) | SETTINGS(同じ `storeDesk` の鍵) |
| KB-*(shortcut) | `EditorShortcuts` を共有。KB-25 の Inspector を出す操作は、常時出ているので何もしない |
| WS-19・WS-20(面をまたぐ移動) | `panelPlacementRequested` → tab / Desk の引き出し |

### NOT YET ROUTED(Phase B 時点)

| inventory | 理由 |
|---|---|
| SH-19 Reset layout・WS-03..09 dock の操作(tab の drag・分割・閉じる・resize・空の leaf) | New は四面固定。dock の概念を持たない。面の大きさの調整は Phase C 以降 |
| SH-29・SH-30・WS-13 panel の切り離し窓 | `placement == 'window'` は何もしない |
| WS-14 配置の保存・ST8-04 panel の配置表 | New は `layout.json` に書かない(Classic の file を壊さないため)。読むのは `deskWork` と `deskDefault` だけ |
| ST8-05 色の theme file | New は `ShellTokens` の固定の look。Classic の theme JSON は Classic のもの |
| ST8-06 Outside dim | Classic でも効いていない(README §7) |
| ST8-07 UI scale | 保存の口が Classic の persist にしか無い |
