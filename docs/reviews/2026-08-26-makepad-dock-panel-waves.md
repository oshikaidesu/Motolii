# Makepad Dock に面を載せる波

作成日: 2026-08-26

状態: **決定**

対象: 製品 front `next/probes/r7-makepad-panel`。既存 Makepad `Dock` / `Splitter` / `Tab`。自前ドックは発明しない。

関連: 裁定251–257 / 258。ゼロコピー正本=[Stage ゼロコピー](2026-08-26-stage-zero-copy-makepad-fork-seam.md)。パネル視線の比較候補=[Icebook 草案](2026-08-25-icebook-panel-drafts/README.md)（B/I/ST/T/E/S の6領域。180案は story 候補のまま。製品へ落とさない）。

## 1. 大目標

既存 `Dock` に面を載せる。いま r7 は `panel.splash` の固定 `SolidView` 格子（Browser | Stage | Inspector / Timeline）。それを Studio と同じ `DockSplitter` + `DockTabs` + `DockTab` に載せ替える。

ゼロコピー Stage は Dock 内の**表示枠**だけ。絵の経路（共有面・import・`render_into`・fork pin）は触らない。

## 2. 責任表

| 面 | 所有するファイル | 触ってよい | 触ってはいけない |
|---|---|---|---|
| **Dock 枠**（波0） | `panel.splash` の木。`main.rs` の widget 登録行だけ | `Dock` / `DockFlat` / `DockSplitter` / `DockTabs` / `DockTab`。既存中身をタブへ移す | 各パネルの中身。Stage 画素。`stage_import.rs`。compositor。fork pin |
| **Browser** | 新 `src/browser_surface.rs` | タブ中身。素材一覧の投影。`Document` / `Session` への読みと Intent 書き | iced `motolii-browser-pane`。共有面。`stage_surface.rs` |
| **Inspector** | 新 `src/inspector_surface.rs` | タブ中身。property / key の投影と書き | iced `motolii-inspector-pane`。共有面。Stage 画素 |
| **Timeline** | 既存 `src/timeline_surface.rs`。`gesture_input.rs` | タブ中身。scrub / restack。いまある Document 書き | iced `motolii-timeline-pane` の view。共有面 |
| **Stage 枠** | 新 `src/stage_chrome.rs` | 見出し・Camera/User View・letterbox の枠。`Image` を置く矩形 | `stage_surface.rs` / `stage_import.rs` / `presentable.rs` / compositor の書く口 / fork pin / `StagePresent` の意味 |
| **Stage 画素** | ゼロコピー担当のまま（この波の外） | — | Dock 波は一切触れない |
| **Export** | 新 `src/export_surface.rs` | タブ中身。範囲・形式・進捗の投影。`Engine::render_frame` と同じ入力 | Stage 共有面。Export 専用プレビュー経路。iced `motolii-export-pane` |
| **Settings** | 新 `src/settings_surface.rs` | タブ中身。project / tokens / session の投影 | 共有面。iced `motolii-settings-pane` / `motolii-menubar` の延長 |
| **Chrome** | 新 `src/chrome/`（部品）と `src/chrome/gallery.rs`（葉） | 共通面・文字・線・行/ボタンとギャラリー。Document を持たない | 各パネル中身。Stage 画素。iced |

Chrome / splash: `ScrollYView` 禁止（eval 白紙）。
未登録 `Chrome*` を splash に書くな（葉が落ちる）。
灰色は空ではない: HotPanel 動的ラッパーが `height: Fit` のまま中に `height: Fill`（高さ0）／`flow: Down` 無しで幅競合／差し替え後は `cx.redraw_all()`。
灰色無エラーは受理しない。差し替え失敗は面に文言。Fit ラッパー禁止。
Chrome 実験は `--hot`。未登録 kind を splash に足すな。
Dock 葉は動いている `View` / `SolidView` / `Label` / `ButtonFlat` だけ直載せ。

意味の正本は `motolii-store` / `motolii-shell-state` / `motolii-engine`。`motolii-shell` は引かない（裁定253/254）。

Icebook 草案の reuse 欄が iced widget を指していても、製品 write-set は上表。草案行（B01… / I01… / ST-01… / T… / E… / S…）は改変しない。

## 3. write-set（交わらない組）

`plan_waves.py` と同じ原理: write-set が交わらなければ同時にやってよい。`panel.splash` は1ファイルなので、中身レーンは splash を触らない。

### 波0（直列。ゼロコピー完了後すぐ）

枠だけ。各パネル中身は動かさない。

- `panel.splash`: 固定格子 → `Dock` 木。Studio 一次と同じ形（左 Browser / 中央 Stage / 右 Inspector / 下 Timeline）。Export / Settings は空タブで植える
- `main.rs`: 空ホストの `register_widget` だけ。Stage インストール・`StagePresent`・import は触らない
- 空ホスト: `BrowserSurface` / `InspectorSurface` / `StageChrome` / `ExportSurface` / `SettingsSurface`。`TimelineSurface` は既存をタブへ入れるだけ

### 波1（並列可）

波0が空ホストを植えたあと、次は同時に走れる。

| 組 | write-set |
|---|---|
| Browser | `src/browser_surface.rs` |
| Inspector | `src/inspector_surface.rs` |
| Timeline | `src/timeline_surface.rs`（と必要なときだけ `gesture_input.rs`） |
| Stage **枠** | `src/stage_chrome.rs` |
| Export | `src/export_surface.rs` |
| Settings | `src/settings_surface.rs` |

Stage **画素**はゼロコピー担当のまま。この波に入れない。

`main.rs` の action 結線が必要になったら WIRE 1本（直列）。意味レーンから `main.rs` へ散らさない。

## 4. 開始条件

ゼロコピー戦が次を満たすまで波0を始めるな。

**通常表示は `StagePresent::Shared`。失敗は Stage 上のエラー画面。`FallbackCpu` を通常表示に使わない。**

進行中の発注（裁定257）がこれを緑にする。緑になる前に Dock 木へ手を出さない。

## 5. 非目標

- iced pane（`next/ui/motolii-*-pane`）を延ばす
- `motolii-shell` をアセンブラに戻す
- 共有面・compositor・fork pin をパネルが触る
- 自前ドック / 自前 splitter
- Icebook 180案を製品へ実装する
- wasm

## 6. 出典

- Makepad `Dock` / `DockSplitter` / `DockTabs` / `DockTab`: `/tmp/motolii-forks/makepad/widgets/src/dock.rs`（`Dock::register_widget`、`DockFlat`）
- Studio 一次: `/tmp/motolii-forks/makepad/studio/desktop/src/app_ui.rs`（`StudioDock`、`root := DockSplitter`）
- 上流同型: https://github.com/makepad/makepad/blob/dev/widgets/src/dock.rs
- Icebook 6領域の閉集合: `docs/reviews/2026-08-25-icebook-panel-drafts/`（生成索引は `python3 scripts/derive_icebook_panel_stories.py`。手で編集しない）
