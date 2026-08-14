# User Palette Library契約

日付: 2026-08-14  
状態: 決定／実装中

## 利用者成果

利用者は名前付きPaletteをプロジェクト横断で登録・整理し、Inspectorで選択中のColor parameterへ色を適用できる。適用は既存Document commandとしてStageへ即時反映され、通常Undoで戻る。

## 所有と寿命

| 意味 | owner | lifetime / write |
|---|---|---|
| Palette名、順序、登録Color | User Settings | user単位・project横断。Document journal／Undo不変 |
| 選択Palette、Browser階層、適用target | Transient | session内。target消失／selection変更でclear |
| 適用後の色 | Document `DocParam::Color` | 既存single writer、journal、Undo |
| Stage表示 | Rerun Spatial Viewer | 同じDocument評価textureのread-only投影 |

Paletteをfile/hash主体のDocument `AssetTable`、Workspace Profile、Project Session、Rerun storeへ保存しない。

## 永続形式 v1

- rootは`version = 1`と順序付き`palettes`を持つ。
- PaletteとColorはuser library内で再利用しないstable string IDを持つ。
- Palette名は空文字を拒否し、Palette／Color件数、文字列長、総byte数へ固定上限を持つ。
- Colorは`{r,g,b,a}`。非線形sRGB、straight alpha、各成分finiteかつ`0.0..=1.0`。
- 未知version、重複ID、空のPalette名、非有限／範囲外Color、上限超過はtyped rejectする。Palette未登録とColor未登録のPaletteは有効とする。
- missing fileは空library。malformed fileは既存bytesを上書きせずerrorを返す。
- saveは同一directoryのtempへwrite／flush／sync後にatomic replaceし、可能なplatformでは親directoryもsyncする。

同期、cloud共有、共同編集、自動palette生成、Documentへのpalette埋込みはv1非目標。

## 適用route

`Palette Color RGBA + current target identity`
→ 既存`preview_source_param` / `preview_effect_param`
→ 既存`set_source_param` / `set_effect_param`
→ `Command::SetProperty`
→ journal-first Document writer / 1 Undo
→ Preview／Export共通graph
→ GPU texture
→ Rerun Spatial Viewer。

target identityはSourceなら`LayerId + param_id`、Effectなら`LayerId + EffectUseId + param_id`。Color型でないtarget、stale identity、targetなしはwrite 0で拒否する。keyframe／Data駆動ColorをConstへ上書きしない。現在時刻keyへの適用は別の既存D2 key commandへ解決できる時だけ許可する。

## Oracle

- 正例: save→restart→別projectでもPalette ID、順序、RGBAが同一。
- 正例: 実Color targetへpreviewするとStage textureが変わり、commit後のsnapshotと再openが同値、Undoで元色へ戻る。
- 負例: Paletteの閲覧、選択、rename、並べ替えでDocument revision／journal／Undoが変わらない。
- 負例: targetなし、型不一致、stale EffectUse、非有限Color、破損settingsはDocument／Stageを変更しない。

## 採択

- `DocValue::Color`、`DocParam`評価、既存Color intent、`SetProperty`、render graph、Rerun wrapperを`REUSE`する。
- 新設はUser SettingsのPalette codec/storeと、現在targetへRGBAを渡す薄いprojectionだけ。
- mock local Palette、hard-coded `Color Wash › Tint`、local color ownerは退役する。
- `BUILD JUSTIFICATION: NONE`。一般asset store、第二writer、第二Stage rendererは作らない。
