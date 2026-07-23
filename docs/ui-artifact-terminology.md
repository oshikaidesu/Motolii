# UI成果物・実装状態の用語

更新日: 2026-07-23

この文書は、MotoliiのUIについて「何を起動したのか」「何が成立したのか」を同じ名前で報告するための
用語正本である。見た目が似ていること、同じReact assetを使うこと、個別surfaceが動くことだけでは、
成果物の種別や結合状態を繰り上げない。

UIを呼ぶ時は、次の3軸を混ぜない。

1. **実行場所**: 外部browser / native desktop process / embedded WebView / native wgpu surface
2. **役割**: 製品 / 開発モック / baseline / spike / headless kernel / source asset
3. **結合段階**: isolated / product-owned / product-connected / product-integrated / preview-runnable

## 固有名と成果物名

| 用語 | 正しい意味 | 現在の実体 | 含意しないこと |
|---|---|---|---|
| **Motolii Studio** | 利用者へ届けるnative desktop製品の固有名 | 製品目標 | 現在すでに完成・配布可能であること |
| **Motolii Studio Mock** | 視覚・操作比較、React source asset確認、回帰試験のため外部browserで動かす開発モック | `docs/mocks-ui/` | native window、製品build、製品Preview |
| **Motolii Studio Preview** | 定義済みの製品surfaceを通常の製品経路で一つのnative desktop実行ファイルへ結合したpreview build | **未実装** | `motolii_ui_shell`、個別spike、外部browserのMock |
| **Native Shell Baseline** | 初期のnative製品外殻と既存接続を確認するbaseline | `motolii_ui_shell` | 最新UIの完全再現、全surface結合 |
| **Native Surface Spike** | 一つのnative描画面、操作kernel、window lifecycle等を隔離条件で検証する実験成果 | `g0-9-*`、`g0-10-*`の各spike | 製品接続、通常起動経路、Preview完成 |
| **G0-9 Native Product Mock** | 発注、粒、証跡を追跡する内部task ID | G0-9関連文書と作業記録 | 利用者向け画面名、製品build名 |

`Motolii Studio Preview`は説明用の愛称ではなく、結合状態を表す固有名として予約する。
該当実行物が無い時は「表示できない」または「未実装」と報告し、最も近い別成果物をPreviewへ改名しない。

## `native`、`React`、`WebView`、`hybrid`

- **native desktop製品**は、OSのdesktop applicationとして起動する製品形態を指す。全surfaceをRust widgetだけで
  描くという意味ではない。
- **React surface**はproduct-owned React componentが所有する表示・操作面を指す。製品ではembedded WebView island
  内で動き得るが、外部browserで動くMotolii Studio Mockとは別の実行物である。
- **native surface**はRust / wgpu側が所有するStage、Timeline time surface、Graph等の描画・高頻度操作面を指す。
- **hybrid**はReact/WebView chromeとnative surfaceを組み合わせるruntime architectureの説明であり、
  build名や完成状態ではない。
- React assetをそのまま再利用できても、product packageへの所有移管、Host projection / typed intent接続、
  WebView/native結合が済むまでは製品結合済みと呼ばない。

## `Preview`の二つの用法

| 表記 | 意味 |
|---|---|
| **Motolii Studio Preview** / **Preview build** | 結合済みnative desktop実行物の固有名 |
| **Preview viewport** / **映像プレビュー** / **preview frame** | Stage内で作品を表示・再生する製品機能 |

単独の「プレビューを表示」は会話文脈で曖昧になり得る。実行物を示す時は固有名を、Stage機能を示す時は
`Preview viewport`または`映像プレビュー`を使う。

## 実装状態

状態は上から順に自動昇格しない。各面について証拠がある段階だけを記す。

| 状態 | 合格条件 | 正しい報告例 |
|---|---|---|
| **source available** | 固定source、依存、CSS/model/test closureを特定できる | 「BrowserのReact source assetがある」 |
| **isolated verified** | spikeまたはheadless fixtureが対象oracleへ合格した | 「Timeline native surfaceは隔離検証済み」 |
| **product-owned** | sourceが製品packageの正本となり、mockがそのconsumerになった | 「Browser componentは製品所有へ移管済み」 |
| **product-connected** | Host projection / typed intent / lifecycleへ接続し、状態所有と負例が試験済み | 「Inspectorは製品snapshotへ接続済み」 |
| **product-integrated** | 同一native process、通常window、同一snapshotの下で対象surfaceが共存する | 「StageとTimelineを製品windowへ結合済み」 |
| **preview-runnable** | 下記Preview表示条件をすべて満たす | 「Motolii Studio Previewを起動できる」 |

「粒を実装済み」は、その粒のcompletion evidenceだけを指す。複数のisolated spikeやheadless kernelが合格しても、
それだけで製品UI全体、製品接続、またはPreview buildを「実装済み」と報告しない。

## Motolii Studio Previewを表示できる条件

次をすべて満たした時だけ、実行物を`Motolii Studio Preview`と呼び「プレビューを表示できる」と報告する。

1. 通常の製品起動経路を持つ、一つのnative desktop実行ファイルである。
2. Preview対象として宣言したBrowser、Stage、Inspector、Timelineとpanel/window orchestrationが同じ製品window群にある。
3. React所有面はproduct-owned sourceからembedded WebViewへ供給され、外部browserのMockを製品の代用にしていない。
4. native surfaceとReact surfaceが同一revision付きsnapshotを読み、永続編集は既存のtyped intent / single writerを通る。
5. 個別spike、diagnostic route、fixture-only stateを通常製品経路へ混ぜていない。
6. 対象specのplatform gateとSTOP条件を満たし、起動コマンドと自動試験の証跡が文書化されている。

条件未達の間は、利用可能な成果物を固有名で個別に案内する。

- browserでUI全体を比較する: **Motolii Studio Mock**
- 旧native外殻を確認する: **Native Shell Baseline**
- 個別のnative面を確認する: **Native Surface Spike**
- headless挙動だけを確認する: **Headless Kernel Fixture**

## 命名ガード

- 外部browserで開いた画面を`Motolii Studio`または`Motolii Studio Preview`と呼ばない。
- `motolii_ui_shell`を、最新UI、最高のUI、完成UI、Preview buildと呼ばない。
- spikeの実装完了を製品結合完了へ言い換えない。
- `mock`、`prototype`、`spike`、`fixture`、`baseline`を「未完成」の同義語として交換しない。それぞれ役割が違う。
- 内部task IDをユーザー向け製品名へ流用しない。
- 「UI完成」「全部実装済み」と報告する時は、対象surface、結合段階、通過したspec/task IDを併記する。
- 新しい実行物を追加する時は、コードより先にこの表の既存分類へ配置する。配置不能なら用語を先に決定する。

