# native surface renderer伸長レビュー（Fable回答・2026-07-21）

状態: **比較中**。[反対側レビュー](2026-07-21-native-surface-renderer-counter-review.md)（潰す方向）と
対になる、**伸ばす方向**のFableレビュー。[拡張サーチ](2026-07-21-native-surface-renderer-extended-search.md)
で得た知見を使い、第一候補（direct wgpu primitive batch + Vello局所 + React複合）を「守る」だけでなく
「何をどこまで伸ばせるか」を機会として整理する。各機会に有効化条件と受益先を付け、条件が揃うまで
依存・契約へ焼かない。本回答は助言であり、採否はfixtureと実機証拠で行う。

## 1. すぐ伸ばせるもの（windowed spikeと同時に有効化可能）

1. **GPU計測基盤**: wgpu-profilerはinterleaved command bufferを公式対応しており、Stage/Timeline
   2 surface + 1 deviceの構成をそのまま計測できる。spike初日から入れれば、headless benchで測れなかった
   「present込み・2 surface同時」のGPU時間が最初から機械可読evidenceになる。既存のevidence JSON慣行
   （g0-9）の自然な拡張。有効化条件: `Features::TIMESTAMP_QUERY`が対象GPUで立つこと。受益先: spike
   合格判定の自動化、以後の性能回帰検知。
2. **CI上のdeterministic render検証**: vello_cpuはGPU無しでpeniko/kurbo語彙のsceneをraster化できる。
   今回のCI再現でも判明した通り、GPU oracle系testはGPU無しcontainerでskipになる——vello_cpuで
   補助capture比較をCPU実行できれば、**GPU無しCIでも構造的な描画回帰を検知**できるようになる。ただし
   GPU Velloとは別rendererなので画素正本やGPU goldenの代替にしない。有効化条件:
   vello_cpu出力とVello(GPU)出力の画素差が許容内であることの実測、および0.0.x APIの変動を
   spike/testツリー内へ隔離すること。受益先: CI、Japanese/CJK固定captureの自動比較。
3. **coordinator契約の先行ドラフト**: anyrender `CustomPaintSource`のlifecycle 3操作（device受領/
   texture登録/suspend）と GraphiteのFrontendMessage一方向投影は、spike前に**紙の契約**として書ける。
   spikeで測るのは「この契約で足りるか」であり、契約を後追いで発見するより検証が速い。有効化条件:
   なし（文書作業）。受益先: spike設計、将来のU3a。

## 2. spike合格後に伸ばせるもの（第一候補の能力拡張）

1. **Timeline labelの国際化能力**: parleyはfontique + harfrustの公式上位層で、Bevy採用が示す通り
   単体libraryとして成立している。Timeline labelが省略記号・fallback・bidiを要した時点でparleyを
   足せば、**独自text layout実装ゼロ**でCJK混在・長い名前・RTLへ届く。有効化条件: 単純single-line
   labelでharfrust直が不足する実例がfixtureで出ること（先回り導入はしない）。受益先: Timeline、
   将来のBrowser native化があれば同様。
2. **roto/feather系の視覚品質目標**: Rive Vector Featheringはpath縁の柔らかさをblurなしで実現する
   GPU vector技法として、Motoliiのroto境界表示・soft mask previewの**品質ベンチマーク**に使える。
   採るのは実装でなく到達水準——「Vello局所passでfeather相当の見た目に何ms掛かるか」をspike後の
   比較fixtureにする。有効化条件: roto UIの要件確定。受益先: M5系roto/mask編集。
3. **Lottie/コミュニティ資産の取込線**: ThorVG（MIT、C API、v1.0系で活発）はrendererとしては不採用
   だが、**Lottie→Motolii Document変換の入口**として別論点で比較する価値がある。rendererを増やさず
   importerだけ足す構成なら停止線と両立する。有効化条件: community資産のLottie需要の確認、変換先を
   Documentの既存語彙に限定できること。受益先: asset Browser、community配布。
4. **低スペックGPU・将来のWeb展開への保険**: vello_hybridはcompute shader無しのrender passのみで
   動き、WebGL2 backendも持つ。classic Velloが要求するcompute対応GPUを持たない環境（古いGPU、
   一部のWeb実行）へ将来届かせる時、**scene語彙を変えずにrendererだけ差し替える**経路が既に上流に
   存在する。有効化条件: sparse stripsのtop-level昇格声明（watchlist済み）。受益先: 配布対象の拡大。

## 3. アーキテクチャとして伸ばせるもの

1. **Linux desktopの現実的経路**: wryのchild WebViewはLinuxでX11のみ・Wayland非対応だが、Graphiteの
   CEF + Vulkan `accelerated_paint`構成はLinuxを含む。つまりCEF枝は「macOS/Windows失敗時の予備」で
   あると同時に、**Linux対応を後日足す時の主経路候補**でもある。撤退順序（反対側レビュー§1-6）を
   変えずに、Linux計画だけはCEF前提で見積もれる。有効化条件: macOS/Windows spike完了後、Linux需要の
   確認。受益先: 配布platform拡大。
2. **a11yの段階的拡張**: AccessKit 0.24のmulti-tree graftで、bounded proxyは「Timeline 1 tree」から
   始めて**panelごと・plugin panelごとの追加tree**へ同じ機構で伸ばせる。将来communityのnative拡張が
   仮に生まれても、graft単位で隔離されたa11y treeという同型で扱える。有効化条件: spikeでWebView側
   treeとの縫合が成立すること。受益先: a11y全域、plugin UI。
3. **投影bridgeのcodegen化**: GraphiteはRust型からwasm-bindgenでTS APIを自動生成し、手書き二重定義を
   避けている。MotoliiのWebView bridgeも、typed bounds/intent/semantic projectionのRust型定義から
   TS型とserializerを生成すれば、**React側とnative側の型乖離をCIで検知**できる。有効化条件: bridge
   語彙がspikeで安定すること（早すぎるcodegenは語彙を凍結してしまう）。受益先: U1a/U1b系contract、
   community UI kitのversioning。

## 4. やらないことで伸ばすもの

- **wgpu 30への追随を急がない**: 29固定の間はVello 0.9系と完全整合で、spike測定の再現性が守られる。
  イベント駆動期限（反対側レビュー§2-5）までは「上げない」こと自体が資産である。
- **watchlistを能動化しない**: vello_hybrid/rive-rs/ThorVG/parleyは、各有効化条件が立つまで調査
  文書上の存在に留める。先回りのspikeは第一候補の検証時間を食う。
- **第二scene graphを作らない**: §2-§3の全機会は「Vello scene語彙 + domain projection」の範囲内で
  成立するよう選んである。この範囲を出る提案が現れたら、それは機会でなく再発明の兆候として
  反対側レビュー§1-1の検出条件へ回す。

## 5. 優先順位（Fable勧告）

| 順 | 機会 | 時期 | 条件 |
|---|---|---|---|
| 1 | wgpu-profiler + evidence拡張（§1-1） | spike初日 | TIMESTAMP_QUERY |
| 2 | coordinator契約ドラフト（§1-3） | spike前 | なし |
| 3 | vello_cpu CI検証（§1-2） | spike並行 | 画素差実測 |
| 4 | parley導入判定（§2-1） | label不足の実例発生時 | fixture証拠 |
| 5 | Lottie取込比較（§2-3） | community計画着手時 | 需要確認 |
| 6 | bridge codegen（§3-3） | bridge語彙安定後 | spike完了 |
| 7 | Linux/CEF見積（§3-1） | macOS/Windows合格後 | 需要確認 |

採否の正本は本回答でなく、[再選定](2026-07-21-native-surface-renderer-reselection.md)の合格条件を
満たすspike証拠とする。本書の機会は全て、条件成立まで依存・公開API・Document・plugin契約へ焼かない。
