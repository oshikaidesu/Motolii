# M3 React Native + Rust/Skia UI runtime再基線決定

日付: 2026-08-07
状態: **決定 / 製品実装は未着手**

## 1. 決定

Motoliiの標準製品UIを次の責任分担へ再基線化する。

| 面 | 標準owner | 主な責任 |
|---|---|---|
| application shell、dock、toolbar、Browser、Inspector、設定、dialog、text/form | React Native | 通常密度UI、IME、focus、a11y、既存React concept、将来のproduct-owned custom panel |
| Timeline | Rust headless interaction + rust-skia canvas | ruler、track header、lane、clip、key、selection、scroll、zoom、drag、trim、snap、transient preview |
| Curve Editor | Rust headless interaction + rust-skia canvas | curve、key、tangent、marquee、zoom、pan、preset preview |
| Stage base preview | Rust core + wgpu | media/composite結果、GPU texture、present、resource lifecycle |
| Stage overlay | Rust headless interaction + rust-skia、wgpu composite | grid、safe area、bounds、path、gizmo、selection、snap補助 |
| editing/playback/save/media/GPU core | Rust | Document、D2 single writer、Undo/Redo、journal、projection、playback、render、resource |
| platform seam | macOS AppKit / Windows RNW Fabric Component View | native view、surface、pointer capture、focus、DPI、resize、lifecycle、bounded accessibility projection |

React Nativeを残すことは、全surfaceをReact Nativeで描くことを意味しない。逆にnative canvasを採ることは、shell、通常panel、IME、a11y、React component資産をRustへ再実装することを意味しない。

旧標準のopaque child WebView islands、1 top-level wgpu Surfaceへ全surfaceを集約する構成、direct wgpu/VelloをTimeline／Stage UIの既定rendererとする構成は、**新規製品実装の標準から外す**。成立済みcode、fixture、benchmark、visual oracleは削除せず、移行中の比較・回帰・意味検証へ使う。eguiも同じく新規製品UIには使わずbaselineとして保持する。

## 2. 採択理由

1. 既存React mockの情報設計、component分割、文言、interaction conceptをRN componentへ移しやすい。DOM/CSSのbyte-for-byte流用ではないが、製品conceptを捨てる全面再設計ではない。
2. Timeline、Curve Editor、Stage overlayは同じcanvas型問題であり、rust-skiaのpath、text、stroke、clip、transformを使うことでprimitive rendererやcurve tessellationのスクラッチを減らせる。
3. Stageの動画／composite結果は既存wgpu資産を維持し、Skia overlayをdirty時だけraster/uploadして同じStage native component内でcomposeできる。
4. pointerの高頻度moveをJS bridgeへ送らず、native component内のRust gesture stateへ渡せる。terminal semantic intentだけをD2へ一回commitできる。
5. WebView特有のnavigation、process、opaque island、HTML5 DnD、DOM/native pointer二重ownerを標準製品shellから外せる。
6. Rust core、D2、projection、render worker、wgpu preview、既存React conceptを再利用し、全面書き直しを避けられる。

## 3. 2026-08-06〜07の隔離probe

probeはMotolii製品code外で行った。したがって採択根拠ではあるが、製品routeの実装・受入を意味しない。

### 3.1 dense canvas

- Timeline 2560×1440 CPU raster: rich clip 50件 p95 4.791 ms、100件 4.071 ms、500件 7.280 ms、1000件 14.487 ms。
- Curve Editor、4 curves / 24 keys: p95 1.039 ms。
- 4K Stage overlay、100 full gizmos: raster p95 5.37 ms、upload call p95 4.68 ms。
- realistic stress、500 bounds + 1 active group gizmo、dirty 2048×1088: raster p95 1.38 ms、upload p95 5.86 ms。

これらは主開発Mac上のraw probe値であり、製品SLOではない。1000 rich clipや500 full gizmoを通常表示要件へ昇格せず、semantic zoom、visible culling、dirty regionを先に使う。

### 3.2 real native surface

RN macOS Fabric appのnative Stage componentで次を実行確認した。

- separate wgpu preview texture + rust-skia overlay + wgpu composite + CAMetalLayer present
- Retina 2560×1440 @2x、live resize、overlay dirty upload
- native child view内dragと、Inspector上でreleaseしたoutside-pointer terminal
- native Stage unmount/remount
- RN TextInputの値を保持したfocus transfer
- Rust release build、clippy、TypeScript、ESLint、Jest、arm64 macOS Release app build

初回overlayは約1.1〜1.3 msで、通常frameは再uploadせず、drag／resize時だけ更新した。完全なIME composition、VoiceOver操作、device lostは未審判である。

### 3.3 Windows compile preflight

`x86_64-pc-windows-msvc`でrust-skia 0.99系とwgpu 29を含む共通renderer coreの`cargo check --release`を通した。これはSkia／wgpu／Rust共通層に根本的なWindows compile blockerがないことを示す。

ただしWindows RNW Fabric Component View、Microsoft.UI.Composition／DX12 present、pointer capture、focus、DPI、remount、device lostはWindows実機で未実行である。`WINDOWS_EXTERNAL_GATE_PENDING`を維持する。

## 4. runtime不変条件

- Document mutationはD2 single writerだけが所有する。
- RN、Timeline、Curve Editor、Stageは同じrevision／generation付きread-only snapshotを消費する。
- drag中はRust transient previewだけを更新し、release時だけ既存commandを高々一回prepareする。
- native componentとRNの境界をper-object／per-frame同期にしない。snapshot、viewport、typed intent、bounded diagnosticsだけを流す。
- 一つのnative component内でGPU device／queue／surface ownerを一意にし、Stage base previewとoverlayのために第二event loopや第二Document ownerを作らない。
- SkiaのsceneをDocumentへ保存しない。renderer型、platform handle、CSS/RN layout型をdomain/public plugin契約へ漏らさない。
- logical coordinate、physical pixel、scale factor、viewport transformを型または明示fieldで区別する。
- native canvasのa11yは全clip/key/objectを無制限にnode化せず、visible／selected／focusedを中心にbounded semantic projectionを出す。
- plugin custom UI公開契約はproduct-owned RN componentと別gateにする。RN採択だけでuntrusted third-party codeを同processへ許可しない。

## 5. 移行方針

全面置換を一度に行わない。旧routeを動くoracleとして残し、利用者outcome単位でRN product routeへcutoverする。

1. RN shellとRust Hostの最小lifecycle、snapshot、typed intent、diagnostic契約を製品routeへ置く。
2. 既存VS-1を最初の移行oracleにし、Browser Rectangle → Stage / Timeline / Inspector同revision → Undo/RedoをRN shellで再閉鎖する。
3. Stageをwgpu preview + rust-skia overlayへ移し、selection／gizmo／grid／snapを接続する。
4. Timelineをrust-skiaへ移し、selection、scrub、move、trim、snap、lane moveを既存headless projection／D2 commandへ接続する。
5. Curve Editorを同じcanvas/input語彙へ接続する。
6. outcomeが新routeで合格した後だけ対応する旧presentation routeを`FROZEN → RETIRE`する。意味oracle、fixture、履歴decisionは削除しない。

Windows adapterはmacOS操作体系の設計を止める前提ではない。ただしMetal/AppKit固有型を共通契約へ漏らした変更は受け入れず、最初の製品vertical slice後にWindows実機gateを早期実行する。

## 6. 製品実装開始条件と受入

本決定により、macOSでのRN shell、rust-skia Timeline／Curve、wgpu + rust-skia Stageの製品実装を開始できる。採択probeを繰り返して選定を再開しない。

製品受入は次を別々に記録する。

- automated: deterministic gesture、zero-write cancel/stale、one terminal commit、revision一致、resize/DPI unit、bounded projection
- macOS product route: real surface、outside release、focus transfer、live resize、remount、GPU recovery
- human: IME composition、keyboard-only editing、VoiceOver、drag feel、visual density
- Windows product route: RNW native component、Composition/DX12 present、DPI、capture、focus、remount、device lost、NVDA
- distribution: arm64/x64 artifact、license notice、offline bundle、crash recovery

rust-skiaはMIT、SkiaはBSD 3-clause系である。配布物へ両者のcopyright、license、disclaimerを再掲し、third-party notice生成をDistribution gateへ含める。

## 7. 非目標

- RNだけ、Rustだけ、Skiaだけへの全面統一
- DOM tree相当のclip/key componentを大量生成するTimeline
- native側で汎用form、dock、theme、text editor、widget frameworkを再発明すること
- CPU readbackを伴う毎frame Stage合成
- 1000 rich clipまたは500 full gizmoを常時フル情報表示すること
- probe成功をWindows、IME、a11y、distributionの完了へ一般化すること
- 旧direct-wgpu/Vello資産を検証なしに削除すること
