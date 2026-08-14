# Godot編集系のPORT採択（トンマナは現行維持）

日付: 2026-08-13
状態: **決定（利用者裁定）**

## 利用者成果

Timeline／Stage／Inspectorを、Godot editorが既に持っている普通のデスクトップ編集操作へ揃える。見た目は現行Motoliiのまま。編集操作を独自発明しない。**最低限ではない。Timeline／Keyframe／Inspectorの操作体系を全部載せる。**

原稿の席が現行Motoliiに無いなら落とさない。Godotの操作席を現行Skia／RNトンマナ（色・行高20px・token・KEY TOOLS・既存席）で持ってくる。Godot chrome／theme／Node型は持ち込まない。

## 既知実装

```text
MECHANISM CLASS: object/property/key 編集操作（選択・disclosure・clipboard・key 編集・Inspector席）
KNOWN IMPLEMENTATION SEARCH: docs/references.md、decision-index、Godot MIT editor source、Rerun re_ui、Theatre studio、OpenCut
CANDIDATES:
  - Godot MIT editor/animation + editor/inspector（一式の実装正本）
  - O3DE Track View Apache-2.0（第二候補、cinematic寄り）
  - Motolii現行: timeline_skia.rs / duplicate.rs / Inspector.tsx / keymap host kinds
ADOPTION ROUTE: PORT Godot操作系。足りない席はMotoliiトンマナで新設し、意味は既存Document/D2へREMAP
REJECTED CANDIDATES: Godot chrome/theme/Node型の持ち込み、Theatre studio(AGPL)、OpenCut React流用、re_ui製品chrome、foldのDocument永続化、選択で全property展開、意味ownerの無い死んだ席
THIN MOTOLII SEAM: Document/D2 single writer、duplicate_track_item、既存host kind、Skia/RN投影、SetProperty(param key)
THIN MOTOLII RESIDUAL: トンマナ、KEY TOOLS席、Stage camera、GPU lowering
RETIREMENT: `selected && property_rows` による自動全展開
BUILD JUSTIFICATION: NONE
```

正本コード（MIT）:

- [`editor/animation/animation_track_editor.cpp`](https://github.com/godotengine/godot/blob/master/editor/animation/animation_track_editor.cpp)
- [`editor/animation/animation_player_editor_plugin.cpp`](https://github.com/godotengine/godot/blob/master/editor/animation/animation_player_editor_plugin.cpp)
- [`editor/inspector/editor_inspector.cpp`](https://github.com/godotengine/godot/blob/master/editor/inspector/editor_inspector.cpp)
- [`editor/docks/inspector_dock.cpp`](https://github.com/godotengine/godot/blob/master/editor/docks/inspector_dock.cpp)

## 席の持ち込み規則

| 原稿の席 | 処分 |
|---|---|
| 現行Motoliiに同じ席がある | REUSE。トンマナは触らない |
| 席が無く、既存Document/D2/host kindへREMAPできる | **席を持ってくる**。見た目は現行token |
| 席がUI projectionだけ（fold、marquee、filter、複数選択） | **席を持ってくる**。Documentへ焼かない |
| 意味ownerが無い（RESET Animation、Bake、Pin、Favorite、Node path） | 死んだ席は作らない。カタログにNO-OWNERと残す |

施工順は実装順であり、残量スコープではない。全部がTARGET。

## 操作カタログ

### Timeline / key（Godot AnimationTrackEditor）

| Godot席 | Motolii |
|---|---|
| ノード選択だけでは全trackを開かない | layer選択だけでは Position/Scale/Rotation/Opacity を開かない |
| track / group disclosure | object行disclosure＝property track。group行disclosure＝子object。foldはUI projection |
| Shift範囲 / Cmdトグル / marquee / Cmd+A | 同じ操作。Document primaryはInspector用の1つ。複数選択はTransient |
| 空click＝解除、空drag＝marquee | 同じ |
| 全propertyのkey click/drag/delete | Position専用をやめ、開いている行へ同じ時間編集を載せる。paramはSetProperty |
| 空property track click＝insert key | クリック時刻へ `add_position_key` / `add_param_key` |
| Cmd+C/X/V keys | host kind。key値はDocumentからclipboardへ。成功前にCut削除しない |
| Cmd+C/X/V tracks | 既存`duplicate_track_item` / Remove |
| Cmd+D | 既存`duplicate` host kind（layer）。key選択中はkey duplicateを優先 |
| Delete | key選択中は当該propertyのremove、否則 delete_layer |
| Escape | 既存`motolii.gesture.cancel` |
| Cmd+Left/Right next/prev step | 既存set_timeへ1frame |
| Shift+Alt+D/A next/prev keyframe | 選択layerのkey時刻へset_time。KEY TOOLSにも同じ席 |
| [ ] move selected key to cursor | 選択keyの時刻をplayheadへ |
| LineEditがCmd+C/Vを持つ | `MotoliiResponderIsTextInput` を維持 |
| Copy/Paste Tracks dialog、Scale Selection dialog、Ease、Audio offset、Bake、Optimize、Clean-up、RESET | NO-OWNERまたは既存trim/interpへ既にREMAP済み。新Document意味は発明しない |
| Godot theme / dock chrome | **採らない** |

### Inspector（Godot EditorInspector）

| Godot席 | Motolii |
|---|---|
| section fold | Transform / Layer / Effects / Source を現行tokenのheaderで開閉 |
| filter | 現行search tokenのフィルタ。一致中はsectionを開く |
| keying button | 既存Rive tri-state。消さない |
| prev/next key | key button隣の席。set_time |
| revert | 既定値へ既存set intent（Position 0,0 / Scale 1,1 / Rotation 0 / Opacity 1） |
| copy/paste value | Inspector内部clipboard（Godot property clipboard）。TextInputのOS clipboardは奪わない |
| copy property path | 同じ内部clipboardへpath文字列 |
| pin / favorite / doc link / 複数object edit | NO-OWNER。死んだ席は作らない |

## Motolii mapping（要約）

| Godot | Motolii |
|---|---|
| Inspector section / track disclosure | 現行inbox／Inspector headerへdisclosure席を載せる。Documentへ保存しない |
| Cmd+C/X/V | host kind。clipboard成功前にCut削除しない |
| 全propertyのkey drag/delete | 開いているproperty行へ同じ時間編集 |
| Godot theme / dock chrome | **採らない。** 現行Skia色・行高20px・Inspector/KEY TOOLS席・Stageを維持 |

## 非目標

- トンマナ、Stage camera、GPU loweringの変更
- 既存席（KEY TOOLS、Mute/Solo、rail、Rive key button）の削除
- fold / 複数選択のDocument schema化
- Godot crate依存、Godot GUI埋め込み
- Inspector粒の未commit差分を巻き戻すこと
- NO-OWNER項目のDocument意味発明

## Oracle

Positive:

- layerをクリックしてもproperty行は増えない
- disclosureでそのobjectの必要なproperty行だけ開閉する
- foldはrevision再投影後も残る
- Cmd+C → Cmd+V で既存duplicateと同じID再発行のsubtreeがDocumentに載る
- TextInput focus中のCmd+C/Vは文字clipboardのまま
- Scale/Rotation/Opacity keyを掴んで時刻がDocumentに載る
- Inspector sectionを畳める。filterで該当sectionが開く

Negative:

- 選択だけで全property展開しない
- param keyをPosition intentへ流さない
- clipboard成功前にCut削除しない
- local selectionを第二Document authorityにしない
- 見た目token、行高、色相を変えない
- 意味ownerの無いPin/Favorite/RESET席を置かない
