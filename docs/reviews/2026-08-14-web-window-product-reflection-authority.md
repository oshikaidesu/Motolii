# Web窓を含む製品projection正本化

日付: 2026-08-14  
状態: **撤回**

> **撤回(2026-08-16)**: Web窓(wry WebView)の経路はコードごと畳んだ。到達不能になっていた
> (呼び手が旧 egui アプリだった)上に、**webview を置き換えるために採択した Blitz の下で
> webview 本体をリンクしていた**。RN 製品面も同日に畳んだ。
> → [Web窓とRN製品面の畳み込み](2026-08-16-web-window-and-rn-product-fold.md)
> `ui/motolii-rn/src` の TypeScript は移植元として残す。本文は歴史として残す。

## 決定

Motoliiの画面構成、視覚、通常UI操作は、Webで直接実行できるReact Native共有componentを原本とする。
macOS／Windowsは同じcomponentを読み、OS surface、GPU、Rerun、IME、file dialog等だけを薄いplatform leafで接続する。
Web上で利用者成果を一周できることを、native接続へ製品意味を残さずplatform capabilityだけへ縮小できた証拠として扱う。

| 責任 | 正本 |
|---|---|
| shell、Browser、Inspector、Timeline chrome、panel配置、視覚、通常操作 | `ui/motolii-rn/`のWeb実行可能な共有component |
| Stage spatial viewer | Rerun Web Viewerを含むRerun既知実装。Document投影だけをMotoliiが薄く翻訳する |
| Timeline表示・入力UX | Webで即時確認できる共有Timeline component。nativeは同じ状態／操作契約をplatform surfaceへ接続する |
| Document、identity、single writer、Undo/Redo、journal、Preview/Export評価 | Rust Host／Documentの既存正本 |
| OS／GPU固有処理 | `.web`／macOS／Windowsの交換可能なplatform leaf |

Webは外観だけを写したmockではない。通常開発では共有componentを直接読み、Hot Reloadで同じsourceを確認する。
Web専用fixture、縮約shell、スクリーンショット、PNG、別のparameter/effect state ownerを製品正本として置かない。
反映種別、shortcut正規化、Rerun fork差分の処分は[Web窓への製品反映辞書](../web-window-reflection-dictionary.md)を正本とする。

## 既存決定の処分

2026-08-07の「Rust headless interaction + rust-skia Timeline／Curveを標準UI正本とする」部分と、Webを比較mockだけに限定する部分を**撤回**する。
既存Rust Timeline／Stage実装は、Document接続、操作意味、性能、native surfaceの実装資産として保持するが、以後の視覚・UX原本ではない。

React Native shell、Rust Document／Host、D2 single writer、VRAM常駐、色変換一元化、Preview／Export同一評価、Rerun Spatial Viewer採択は維持する。

## 移行順

1. `App.tsx`をWeb rendererから直接読む一つの開発入口を維持する。
2. 固定`ROWS`等のWeb fixtureは昇格させず、実`WireTimelineProjection`から生成する共有scene/modelとtyped hit／gesture contractへ置換して最新UXを閉じる。
3. Stageは独自placeholderを退役し、Rerun Web ViewerへDocument projectionを薄く接続する。
4. macOS／Windowsは共有componentとtyped Host contractを読み、固有surfaceだけをadapter化する。
5. 共有routeで置換されたRust presentationの重複部分は、操作・性能oracleを移管してから退役する。

## 非目標

- Document意味、永続形式、plugin契約をReact stateへ移すこと
- Preview／Export rendererをWeb UIへ分岐すること
- Rerun scene/view/query/camera/pickingをMotoliiで再実装すること
- Webとnativeで別々のUI仕様を維持すること

## 現在状態

`ui/motolii-rn/src/web-preview.tsx`は共有`App.tsx`を直接mount済み。
TimelineとStageの`.web` leafは移行中であり、製品正本へ昇格済みとはまだ数えない。
