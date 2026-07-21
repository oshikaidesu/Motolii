# G0-9 UI runtime部分スパイク（2026-07-21）

状態: **部分合格／runtime採否は比較中**。React/WebViewを製品へ組み込まず、G0-9で先に
確認できるBrowser virtualization、dense Timeline surface、browser WebGPU、Vite HMRと、
現行native wgpu基準を同じ規模のfixtureで実測した。本書はWebView採択、plugin UI公開、
Document/API変更を許可しない。

## 結論

1. **BrowserをReactにしても10,000 itemを全DOM化する必要はない**。固定高virtual listは
   10,000 itemに対して24 rowだけをDOMへ置き、最終itemのstable ID選択をscroll後も保持した。
2. **TimelineをReactにしても100,000 keyをDOMへ置く必要はない**。Reactはsurfaceを所有し、
   Canvas 2Dまたはbrowser WebGPUがvisible rangeを単一面へ描画できた。
3. **browser WebGPUの局所採択は技術的候補として残る**。ただし今回のadapterはheadless
   ChromiumのSwiftShaderであり、native Rust/wgpuのApple M4 Metal deviceとは別物だった。
4. **hot reloadの配線は成立した**。Vite virtual module更新をRust process再起動なしで
   acceptした。ただし製品component編集、状態保持、plugin更新の完全な審判ではない。
5. **G0-9の最終採否はまだ閉じない**。最大の未検証点はWebView内shellとnative wgpu Stageの
   CPU readbackなし合成、IME/a11y、community plugin sandbox、Windows実機である。

したがって現時点の方向は「React DOMかnative wgpuか」の二者択一ではなく、
**React DOM＝shell/Browser/form、Canvas/browser WebGPU＝Web runtime内の高密度面、
Rust/native wgpu＝映像Stage/render core**という責任分担をhybrid候補として次段へ送る。

## スパイク実装

[spikes/g0-9-web-ui](../../spikes/g0-9-web-ui/)は製品workspace外の隔離ハーネスである。

- React 19.0.0 / Vite 6.4.3 / Playwright 1.61.1
- `g0-9-dense-ui-v1`: Browser 10,000 item、Timeline 1,000 clip / 100,000 key / 32 track
- Browserはscroll位置からvisible rowだけをReact要素へ投影
- TimelineはCanvas 2Dとbrowser WebGPUを同じページ内の交換可能な局所rendererとして計測
- Vite dev serverのvirtual moduleをinvalidated/reloadし、HMR acceptまでを計測
- 製品WebView、native texture共有、Document/command、plugin sandboxは含めない

依存監査時にVite 6.0.11の既知脆弱性が検出されたため、既存Reactモックと同じVite 6系列の
6.4.3へ上げた。`npm audit --audit-level=moderate`は0件である。このversionはスパイク固定値で、
製品runtime契約ではない。

## 実測環境

- macOS Darwin 24.5.0 / arm64 / Apple M4
- native: wgpu 29.0.4 / Metal
- Web: Playwright Chromium 149 headless
- browser WebGPU adapter: Google SwiftShader
- Web viewport: Timeline 1200×512、Browser 420×480
- native timeline viewport: 1920×512

## 実測結果

| 対象 | 操作／同期 | 結果 | 判定 |
|---|---|---|---|
| React Browser | 10,000 itemを120段階scrollし各段階で次のanimation frameまで待つ | DOM 24 row、median 16.70 ms、p95 18.20 ms。`asset-09999`選択をscroll後も保持 | **構造合格** |
| Canvas 2D Timeline | 120 frame、48秒window、visible key 20,000。clip+keyを描画 | median 3.90 ms、p95 5.30 ms | **局所面成立** |
| browser WebGPU Timeline micro-probe | 120 frame、48秒window、visible key 20,000。submit後`onSubmittedWorkDone`待機 | median 3.70 ms、p95 4.00 ms | **利用可能性合格** |
| native wgpu Timeline | 600 frame、visible clip 210 + key 20,005。submit後`device.poll(Wait)` | median 4.17 ms、p95 6.26 ms | **native基準再合格** |
| Vite HMR | virtual module revision 0→1をaccept | 16 ms、Rust再起動なし | **配線合格** |
| React製品候補資産 | 固定比較worktreeでbuild + Playwright | build成功、43/43 test成功、npm audit 0 | **再利用可能性維持** |

証拠は[Web report](g0-9-web-ui-evidence/report.json)と
[native timeline report](g0-9-web-ui-evidence/native-timeline-report.json)に保存した。

## 数値を直接比較してはいけない理由

上表は同じ規模の入力を使うが、rendererの勝敗表ではない。

- Browserの時間はReact処理時間だけでなく60 Hzの`requestAnimationFrame`待ちを含む。
- Canvas 2DはAPI呼出し完了までで、GPU present完了を強制していない。
- browser WebGPU micro-probeはkey pointだけを描き、clip rectangleをまだ描いていない。
- browser WebGPUはSwiftShader、native wgpuはApple M4 Metalでadapterが異なる。
- Webは1200×512、nativeは1920×512でviewportが異なる。
- nativeはCPU cull、upload、submit、GPU完了待ちをすべて含む。

このため今回確定できるのは、virtual DOMと局所GPU surfaceが候補から脱落しないことだけである。
「browser WebGPUがnativeより速い」「Canvas 2Dで製品Timelineが完成する」とは結論しない。

## native egui基準の再検証

現行main相当のegui基準について次を再実行した。

- `cargo test -p motolii-ui --test public_boundary` — 3/3
- `cargo test -p motolii-ui --test u1a1_static_viewport` — 3/3
- `cargo test -p motolii-ui --test u1a1_window_smoke` — 3/3
- `cargo run --manifest-path spikes/timeline-bench/Cargo.toml --release -- --json` — PASS

これによりtoolkit型の公開境界流出拒否、event loop内render/join/readback拒否、実windowの
resize/minimize/restore、single-writer編集とlatest worker resultの製品preview到達を保持した。
React候補の存在はこの成立証拠を撤回しない。

## 未検証と次の停止線

G0-9を閉じる前に少なくとも次を別スパイクで検証する。

1. macOS/Windowsの製品候補WebViewで、native wgpu StageをCPU pixel readbackなしに合成できるか
2. resize、minimize/restore、DPI移動、overlay、alpha、色、device lost、古いgeneration破棄
3. 日本語IME、keyboard/focus、screen readerとCanvas上のaccessible proxy
4. Hostとcommunityが同じReact kitを使いつつ、file/network/GPU/Document mutationを権限分離できるか
5. 実ReactモックのBrowser/Timelineを今回のstress fixtureへ接続し、通常操作を保てるか
6. Windows WebView実機とoffline配布、dev serverを出荷物へ含めないproduction bundle

次のどれかを仮定した時点で停止する。

- browser WebGPUとnative wgpuを同じ`GPUDevice`または共有Textureとして扱う
- WebGPUが使えない環境で製品StageをCPU readbackへ黙ってfallbackする
- 24 DOM rowという結果から可変高tree/gridの完成を主張する
- HMR 16 msからplugin互換、sandbox、状態migrationまで解決済みと主張する
- SwiftShaderの数値をApple MetalまたはWindows GPUの製品性能値にする

## 再現

```sh
cd spikes/g0-9-web-ui
npm ci
npm audit --audit-level=moderate
npm run build
G0_9_EVIDENCE=../../docs/spikes/g0-9-web-ui-evidence/report.json npm test

cd ../..
cargo test -p motolii-ui --test public_boundary --test u1a1_static_viewport
cargo test -p motolii-ui --test u1a1_window_smoke
cargo run --manifest-path spikes/timeline-bench/Cargo.toml --release -- --json
```
