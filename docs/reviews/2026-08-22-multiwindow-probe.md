# iced multiwindow(窓の浮かし)probe — 裁定188 のゲート判定

**verdict: 緑(実装可・切片割り付き)**

- 日付: 2026-08-22
- 発注: 裁定188「ドッキングだけでなく浮かしも欲しい。Settings はだいたいポップアップだから」— Settings を OS 窓として浮かす第1号のゲート
- probe: `next/probes/r5-multiwindow/`(merge 前提ではない検証コード。製品 crate 非接触 — 触ったのは workspace members への1行追加のみ)
- 実行: `cargo run -p r5-multiwindow -- --auto`(自走 — 人手なしで開閉・計測・判定を踏んで終了)

## 0. 実測ログ(自走1発目・全緑、exit=0)

```
PROBE main_opened id=Id(1)
PROBE settings_opened id=Id(2) counter=1
PROBE device_shared=true queue_shared=true main_frames=10 settings_frames=2 cross_write_buffer=survived
PROBE settings_closed
PROBE main_alive_after_settings_close=true frames_after_close=5
PROBE settings_opened id=Id(3) counter=2
PROBE state_persists=true counter=2
PROBE RESULT: OK
```

計測器の作り: 各窓の view に 4×4px の `shader` widget を置き、
`Primitive::prepare(device, queue, ..)` — その窓の実描画パスから呼ばれる — で
窓タグ別に device/queue を記録して Eq 比較(wgpu 29 の `Device`/`Queue` は
`impl_eq_ord_hash_proxy!` で inner dispatch handle を比較する実 Eq、
`wgpu-29.0.4/src/api/device.rs:26`)。さらに機能面の裏取りとして
「main 窓の prepare で作った `wgpu::Buffer` へ settings 窓の prepare から
`write_buffer`」— device が窓ごとに別なら wgpu validation error(既定 handler
は panic)で即落ちる。`cross_write_buffer=survived` はそれが通った証拠。

## 1. 問い別の答え

### Q1. fork の daemon/multiwindow API の実在と形 — ある・動く

fork rev 73e686ee に `iced::daemon(boot, update, view)` が実在
(`src/daemon.rs:24`、root 再輸出 `src/lib.rs:669`)。builder は
`.title/.subscription/.theme/.style/.scale_factor/.settings/.presets` を持ち、
view/title/theme/scale_factor は **`Fn(&State, window::Id)`** 形。
`window::open(Settings) -> (Id, Task<Id>)`(`runtime/src/window.rs:260`)で
第2窓が開き、`view(&self, window::Id)` の窓別分岐で別 UI が出た(実測ログ)。
fork 自身が `examples/multi_window` を持ち、probe と同型(daemon +
BTreeMap<window::Id, _>)— 先例コードとして写せる。

### Q2. 単一 device 複数 Surface — 共有される(実測+ソース両面で確定)

- **実測**: `device_shared=true queue_shared=true cross_write_buffer=survived`。
- **ソース**: `wgpu/src/window/compositor.rs` — `Compositor` が `Engine`
  (device+queue)を1つ抱え、`create_surface` は `self.instance` から surface を
  作り `configure_surface` は `&self.engine.device` を使う(313・321行)。
  winit shell(`winit/src/lib.rs:557-`)は **compositor を `Option<C>` 1個**で
  持ち、初窓で1回だけ作って以後の全窓の `window_manager.insert` に渡す。
  つまり構造が「1 instance / 1 adapter / 1 device / 1 queue、窓ごとに
  surface+renderer」— **Stage の zero-copy presenter(裁定170/171)の
  単一 device 前提はそのまま保たれる**。

注意点1つ(黒ではない): `winit/src/lib.rs` の `window::Action::Close` 処理は
**全窓が閉じると `*compositor = None`**(device 破棄、次の open で作り直し)。
Settings 窓の開閉では main が生きているので起きない。製品は「main 閉=アプリ終了」
なので実害なし。ただし将来 `Engine::with_device`(zero-copy seam 決定)で iced の
device を Stage が抱えた後に「窓ゼロで daemon が生き続ける」形を作るなら、
その device は死んでいる — **窓ゼロ状態を作らない(main 閉=exit)を不変量として
維持**すれば消える論点。

### Q3. 既存 shell への影響半径 — 小さい(2ファイル+柵1本が本丸)

- **`shell/motolii-shell/src/main.rs`**(104行): `iced::application(boot, update, view)`
  → `iced::daemon(...)` へ。boot が `window::open` で main 窓を開く形になり、
  `.theme(|shell| ..)` は `.theme(|shell, _window| ..)` へ。**±20行の桁**。
  `--screenshot` 一発ツール経路(窓を開かない)は無関係・不変。
- **`shell/motolii-shell/src/lib.rs`**(4,191行): `Shell::view(&self)` は
  **改名も改形もしない**。daemon へ渡すのは薄い dispatcher
  `view_window(&self, id: window::Id)` — main 窓なら既存 `view()`、settings 窓
  なら settings 用 view を返す。これで **`.view()` の既存呼び出し 78 箇所
  (tests 11 ファイル+`screenshot.rs`+`transport.rs` 等)は無傷**。
  追加は window 台帳(`main_window`/`settings_window: Option<window::Id>`)+
  boot の open Task + `window::close_events` 購読 + dispatcher で **50〜100行の桁**。
- **柵**: `tests/suite/theme_wiring_fence.rs` が `main.rs` を
  `iced::application(` で grep している(30-31行)— daemon 化で**必ず赤になる**
  ので同時に `iced::daemon(` へ更新(±10行)。これは柵が正しく働く証拠であって
  障害ではない。
- **subscription**: daemon でも `Fn(&State) -> Subscription` のまま(窓別形なし)。
  現行 `Shell::subscription`(lib.rs:850)は無改変で通り、`close_events` の
  1 本足しだけ。
- **theme/scale**: 窓別引数が増えるだけで、現行「全窓同一 theme」はそのまま
  表現できる(probe では settings=Light/main=Dark の窓別も通った)。
- **閉じた窓の状態**: daemon の State は窓と独立(実測 `state_persists=true` —
  counter が窓の閉→再開を跨いで保持)。Settings の状態は今も `Shell` 側
  (`settings_panel_open`・`background_draft` 等)にあるので、**窓を閉じても
  何も失われない**。`ToggleSettingsPanel` の意味が「レイアウト分岐」→
  「窓 open/close」に変わるだけ。
- **器具の注意(S2 で払う)**: `screenshot.rs` は単窓オフスクリーン合成なので、
  Settings が別窓へ出ると `--settings-open` の絵から Settings が消える。
  当面は screenshot 経路だけ旧・帯型描画を残す(表示分岐を screenshot 専用に
  維持)か、`window::screenshot`(在庫調査 §重い順10)で窓別撮りへ — S2 の
  発注に視覚受入条件として明記すること(capsule の穴対策)。

### Q4. Settings 窓の最小形 — 動いた

probe がそのもの: main 窓+設定風第2窓(共有 counter の表示・増分・Close)、
閉じても main 生存(`main_alive_after_settings_close=true frames_after_close=5`)、
開き直しで状態保持。製品の `settings_pane::view` は投影(composition/draft/
dims/colors)だけ受けて `Element` を返す純関数なので(lib.rs:2405-2412 の
呼び出し形)、第2窓の view へ**そのまま移せる**。

### Q5. 黒条件 — どれも成立しなかった

| 黒条件 | 実測 |
|---|---|
| fork に daemon が無い / compile 不能 | ある・probe は初回 build 41.7s で緑 |
| device が窓ごとに分かれる(zero-copy 前提崩壊) | 共有(Eq 実測+cross write_buffer 生存+compositor ソース) |
| 窓を閉じると daemon State が失われる | 保持(counter=2 が再開後も見えた) |
| view の窓別化が既存 78 呼び出しを壊す | dispatcher 追加で無傷(signature 変更不要) |

第1段モーダルへの後退は不要。

## 2. 実装切片案(S1→S3、絞め殺し方式)

- **S1 daemon 骨格(挙動不変)**: main.rs を `iced::daemon` 化・boot で main 窓
  open・`Shell` に window 台帳+`view_window` dispatcher+`close_events` 購読
  (main 閉=exit の現挙動を明示的に再現)・`theme_wiring_fence` を daemon 形へ。
  受入 = 全 suite 緑+実窓で従前と同一の見た目。
- **S2 Settings 移住**: `ToggleSettingsPanel` → 窓 open/close へ配線・
  `view_window(settings)` = 題帯+`settings_pane::view`・閉→開で下書き保持の
  drive テスト・screenshot 器具の扱い(旧経路温存 or `window::screenshot`)を
  発注書に視覚受入条件込みで明記。
- **S3 他パネル解禁**: 浮かし対象の一般化(Browser 等)— pane_grid からの
  脱着として設計(pane_grid レーンの「開閉で drag 並べ替えが失われる」既知制限
  と同じ Configuration 再構築論点をここで一緒に精算)。窓別 theme/scale の
  製品方針もここ。

## 3. Stage への影響(単一 surface 前提)

保てる。compositor/device/queue は窓数と無関係に1つ(§Q2)。Stage の
presenter は main 窓の描画パスに住み続け、第2窓は別 surface に描くだけ。
zero-copy seam 決定(`2026-08-22-zero-copy-seam-decision.md`)の
`Engine::with_device` 構想とも矛盾しない — device は「最初の窓」で生まれ全窓で
同一なので、受け渡し口は1本のまま。唯一の派生論点は §Q2 の「窓ゼロで
compositor 破棄」— main 閉=exit を保つ限り到達しない。

## 4. 証跡

- probe コード: `next/probes/r5-multiwindow/`(Cargo.toml+src/main.rs、
  workspace members に1行追加)
- 実行ログ: §0(自走・exit=0)。build: dev profile 41.69s(初回、fork 込み)
- ソース根拠: fork checkout
  `~/.asdf/installs/rust/stable/git/checkouts/iced-1bbb4ed9d90ae4f8/73e686e/`
  — `src/daemon.rs`・`wgpu/src/window/compositor.rs`・`winit/src/lib.rs`
  (compositor 単一保持 557-・全窓閉で None 化 1355-)・`examples/multi_window/`
