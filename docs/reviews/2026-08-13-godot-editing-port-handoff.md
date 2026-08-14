# Godot編集系PORT — 実装引き継ぎ

日付: 2026-08-13
状態: **観察**（dirty worktree。製品手感は人間確認中。本書はauthorityではない）

再開時は本書を正本にせず、[Godot編集系PORT決定](2026-08-13-godot-editing-system-adoption.md)、[決定逆引き台帳](../decision-index.md)、`AGENTS.md`、current dirty code を再照合する。

## 0. 今すぐ守ること

- **commit / push / PR しない**（利用者未発注）
- dirty `main` の既存差分を revert しない。Inspector key-add 粒と本PORTは同じ worktree に混在する
- 新Command variant、foldのDocument永続化、Godot chrome/theme、Pin/Favorite/RESET を発明しない
- 確認用アプリは **Release `MotoliiRn`**。`scripts/dev-macos-app.sh` の Debug は別物
- 今開いているプロセス（2026-08-13 20:51 実測、pid 34959）:

```text
/Users/member_ottoto/Library/Developer/Xcode/DerivedData/MotoliiRn-cxfazhwyescgcscaodvrwprtbtis/Build/Products/Release/MotoliiRn.app
```

## 1. Git

- repo: `/Users/member_ottoto/rust_ae/Motolii`
- branch: `main`（`origin/main` より ahead 54）
- HEAD at handoff: `5eaad708` *Connect native terminal results to RN state*
- 本PORTは **未commit**。clean worktree へ逃していない

混在している先行dirty（本PORT開始前から存在。巻き戻さない）:

- Inspector Rive key-add（[縮小採択](2026-08-13-inspector-key-add-ux-decision.md)）
- `README.md`、`ui/motolii-rn/macos/Podfile.lock`、`ui/motolii-rn/src/host.ts`、`productStyles.ts`

## 2. 利用者裁定（再掲・正本はadoption）

Timeline／Inspector／key の操作系は Godot MIT editor を **全部PORT**。トンマナは現行Motolii。席が無ければ現行tokenで持ってくる。見た目をGodotにしない。

## 3. 載ったもの（code）

| 席 | 接続 |
|---|---|
| 選択で全property展開しない | `TimelineScene::from_snapshot` は `expanded_layer_ids` 空。`toggle_property_disclosure` |
| object/group disclosure | object＝property tracks、group＝子fold。UI projection。revision再投影は `preserve_vertical_window_from` |
| Shift / Cmd / marquee / Cmd+A | MOD_SHIFT=1, MOD_META=8。複数選択はTransient。Document primaryは1つ |
| 空click解除 / 空drag marquee | EmptyBar Downはmarquee開始、未移動のUpで `ClearSelection` |
| 全property key drag/delete | Positionは既存intent。Scale/Rotation/Opacityは `SetParamKeyTime` / `RemoveParamKey` → `SetProperty` |
| 空property clickでinsert key | `AddKey` → `add_position_key` / `add_param_key` |
| Cmd+C/X/V | process-local `EditClipboard`。copyは非mutation。cutはcopy成功後だけ削除 |
| Cmd+D | Timeline focus時 result=7。key選択中は `duplicate_selected_key`、否則 layer duplicate |
| Delete | key選択中は当該property remove、否則 `delete_layer` |
| Cmd+Left/Right、Shift+Alt+D/A | `goto_*_step` / `goto_*_key`。`seek_to_time`。Document非mutation |
| Inspector fold / filter / revert / copy-paste / prev-next | RN local。内部clipboard。OS Clipboard APIなし |
| KEY TOOLS Prev/Next Key | 既存ボタンを消さず追加。`timeline-goto-prev-key` / `timeline-goto-next-key` |
| TextInput Cmd+C/V | `MotoliiResponderIsTextInput` 維持。keymap monitorは result 2..=7 をviewへ渡す |

REMAPの局所例外:

- Inspector Position の **off-key revert** は `commitHostStageTransform` kind0 を使わず、`move_layer_by` に現在値との差分を渡す
- param key 時刻は新Commandなし。`prepare_set_transform_param_key_time` → `push_param_set_property`

## 4. まだ無い / 未確認

カタログ残:

- **`[` `]`** 選択keyをplayheadへ（keymap未接続）
- keyの複数選択はGodotほど強くない（layer marqueeが主。`selected_real_key` は先頭1つ）
- Bake / Ease dialog / Pin / Favorite / RESET は NO-OWNER のまま

検証:

- Rust: native-renderer `timeline_` 87、`place_then_undo` band_count=1、motolii-ui copy/cut/param/goto 5、keymap 12
- RN: `tsc` / eslint / `App.test.tsx` 56
- 製品手感: 利用者が **Release MotoliiRn** で確認中。Debugで開いた事故あり（下記）

起動:

```text
今開いている Release app:
/Users/member_ottoto/Library/Developer/Xcode/DerivedData/MotoliiRn-cxfazhwyescgcscaodvrwprtbtis/Build/Products/Release/MotoliiRn.app

実行ファイル:
/Users/member_ottoto/Library/Developer/Xcode/DerivedData/MotoliiRn-cxfazhwyescgcscaodvrwprtbtis/Build/Products/Release/MotoliiRn.app/Contents/MacOS/MotoliiRn
```

`scripts/build-macos-app.sh` は `pod _1.15.2_ install --deployment` が **Podfile.lock の CocoaPods 1.17.0** と衝突して落ちる。本sessionのReleaseは:

```text
LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8
cargo build --manifest-path ui/motolii-rn/native-renderer/Cargo.toml --release --locked
xcodebuild -workspace ui/motolii-rn/macos/MotoliiRn.xcworkspace -scheme MotoliiRn-macOS \
  -configuration Release -destination 'platform=macOS,arch=arm64' \
  ARCHS=arm64 ONLY_ACTIVE_ARCH=NO CODE_SIGNING_ALLOWED=NO build
open /Users/member_ottoto/Library/Developer/Xcode/DerivedData/MotoliiRn-cxfazhwyescgcscaodvrwprtbtis/Build/Products/Release/MotoliiRn.app
```

`dev-macos-app.sh` は Debug + Metro。利用者の確認対象ではない。同じbundle idのため同時起動すると窓を取り違える。

## 5. 主要file

- `ui/motolii-rn/native-renderer/src/timeline_skia.rs` — disclosure / 選択 / key gesture
- `ui/motolii-rn/native-renderer/src/renderer_core.rs` — keymap copy/cut/paste/select_all/duplicate
- `ui/motolii-rn/native-renderer/src/host_bridge.rs` — result 2..=7、timeline keymap、band_count=1
- `ui/motolii-rn/native-renderer/src/lib.rs` / `platform/macos.rs`
- `ui/motolii-rn/macos/MotoliiRn-macOS/MotoliiGpuComponentView.mm`
- `crates/motolii-ui/src/rn_product_host.rs` — clipboard / goto / set_param_key_time / remove_param_key
- `crates/motolii-ui/src/keymap.rs` / `lib.rs` / `tests/keymap.rs`
- `crates/motolii-doc/src/position_key_prepare.rs` — `prepare_set_transform_param_key_time`
- `crates/motolii-ui/src/document_edit_runtime.rs` — `push_param_set_property`（OpacityをPosition intentへ流さない）
- `ui/motolii-rn/src/Inspector.tsx` / `Timeline.tsx` / `__tests__/App.test.tsx`

## 6. 次の一粒候補（未選定）

再開後に current code と人間確認結果から再選定する。候補であって発注ではない。

1. 利用者が指摘した手感ギャップの最小修正
2. `[` `]` を既存 `set_position_key_time` / `set_param_key_time` へ接続
3. 確認済みなら dirty を1契約としてcommit（利用者明示時のみ）

## 7. 負例（壊したらSTOP）

- 選択だけで全property展開
- param keyをPosition intentへ流す
- clipboard成功前にCut削除
- TextInputのCmd+C/Vを奪う
- KEY TOOLS / Rive key button / トンマナを黙って変える
- local selectionを第二Document authorityにする
- Inspector key-add の既存testIDを消す
