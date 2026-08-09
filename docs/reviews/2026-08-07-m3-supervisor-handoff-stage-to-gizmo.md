# M3 supervisor handoff — Stage pixelsからgizmoへ

日付: 2026-08-07
状態: **引き継ぎ / 施工停止 / 次発注未選定**

## 1. この文書の扱い

この文書はrunner規則、設計決定、baseline acceptanceではない。2026-08-07のsupervisionで確認・統合したcurrent Git／code事実、未閉鎖境界、失敗runの処分を次のfresh supervisorへ渡すための作業メモである。

再開時は会話や本書をauthorityにせず、root working treeの`AGENTS.md`、`docs/README.md`、decision index、implementation ledger、M3 checkpoint／execution map／rebaselineとcurrent codeを再照合する。

## 2. Git安全境界

- authority checkout: `/Users/member_ottoto/rust_ae/Motolii`
- branch: `codex/supervision-authority-guard-20260804`
- HEAD: `1cb9236279cad5870e5915b051b2e9e82cbf755d`
- rootは多数の既存tracked／untracked docs差分を持つ。これらは利用者のcurrent authorityであり、HEAD版へ降格しない。
- rootでreset、checkout、cleanup、stage、commit、push、main統合を行っていない。本書の追加だけが今回の新規root差分である。
- local main integration worktree: `/private/tmp/motolii-r0-main-integration-20260807`
- local main HEAD: `9b2deac4aabe2c87775459ad5617a58bb7369aff`
- local mainは`origin/main`より231 commit ahead。pushしていない。

## 3. 状態を繰り上げない

- baseline accepted item = 0
- baseline mapping = `OPEN`
- R0 candidatesのbaseline上の表記は`READY-RECHECK / MAIN NOT REACHED`を勝手に書き換えない。
- checkpointをM3全体の製品施工許可や製品完成へ読み替えない。
- fixture、probe、test green、外部review、local main統合、通常製品route、実機／人間gateを分ける。
- execution-envelope候補は`TESTED / UNINTEGRATED`の別境界のまま扱う。

## 4. 今回local mainへ到達した実コード

次のlocal commit列を確認した。

- `97b37eba` `feat(m3): add React Native product runtime seat`
- `d515e7b1` `feat(m3): add React Native editor shell slots`
- `63cd47ae` `feat(m3): decode initial Inspector snapshot`
- `66cec7a9` `feat(m3): present initial Inspector snapshot`
- `aef6c363` `feat(m3): bind React Native Stage GPU surface`
- `9b2deac4` `feat(m3): present initial Stage preview pixels`

最新粒`R1-STAGE-BASE-A`は、Hostが開いたcurrent Documentの初回snapshotを既存`prepare_in_setup_worker → RenderWorker → RenderedPreview → DisplaySlot`で一度だけ描画し、同じ`Arc<GpuCtx>`の既存preview pipelineからReact Native macOSの`CAMetalLayer`へpresentする。

変更は次の2ファイルだけだった。

- `crates/motolii-ui/src/product_runtime.rs`
- `crates/motolii-ui/src/rn_product_host.rs`

CPU pixel readback、第二GPU device／queue／event loop、Document write、公開API、schema、依存追加はない。pipeline、texture、worker、bind groupはdraw loop外で生成する。

## 5. `R1-STAGE-BASE-A`の検証事実

候補worktreeとlocal mainの双方でcode diffを照合した。local main統合後の結果は次のとおり。

- `cargo check --locked -p motolii-ui`: PASS
- `cargo test --locked -p motolii-ui --lib rn_product_host::tests`: 18/18 PASS
- `cargo test --locked -p motolii-ui --test r0_rn_product_seat`: 5/5 PASS
- macOS arm64 Release `xcodebuild`: `BUILD SUCCEEDED`
- `git diff --check`: PASS
- fresh Fable read-only diff review: `ACCEPT / P0=0 / P1=0 / P2=2 / SCOPE PASS`

P2はBGRA8Unorm固定とclear色変更である。前者はaccept済みsurface contractと一致し、後者はfullscreen previewで覆われるため採用阻害にはしなかった。

ただし通常起動artifactでproject pixelsを人間が視認するgate、live update、selection、bounds、gizmoは未実行／未実装である。`9b2deac4`を「触れる動画編集ソフト」やR1完了へ繰り上げない。

## 6. gizmo直行を止めたcurrent-code gap

read-only再compileでは`R2-STAGE-GIZMO`の直接発注を`TARGET_MISSING`とした。current RN routeに次の三identityが揃っていない。

1. displayed revision／cameraから導出された`LayerId + geometric bounds`
2. native Stage pointer hitからHost transient primary selectionへ入る単一producer
3. accepted selection publish後、Stage／Inspector／Timelineへ同じprojection generationを再配送するwake consumer

補足:

- `WireStageBound`は`layer_id/display_name`だけで幾何boundsではない。
- `RnProductHost.primary`は初期`None`で、RN routeにproducerがない。
- native Stage componentのmouseDownはfocus取得だけでpointer position／hit／gesture sequenceをHostへ送らない。
- `ProductStageProjection`はlatest render generation gateであり、selection/bounds/gizmo projectionではない。
- `R1-STAGE-BASE-B`のpersistent live renderも、RN componentへcompletionを通知する既存wake identityがないため未閉鎖。
- current dependenciesに製品rust-skia Stage overlay backend targetはまだ実在しない。即席wgpu overlayへ迂回しない。

preview pixelsからidentityを推測する、先頭layerを暗黙選択する、旧`ProductApp` input ownerをcopyする、callback registry／pollingを新設する、gizmo releaseから直接Position commandを呼ぶ案は不採用である。

## 7. Browser経由の検討と停止位置

gizmoの前に正規primaryを作るR1 spineとしてBrowser Rectangle identityを調べた。

current fact:

- RN `App.tsx`のBrowser／Timelineはplaceholder。
- RN Host intentは`read_snapshot`とStage lifecycleだけ。
- RN root propsは`hostHandle/projectPath/snapshotJSON/diagnostic`だけ。
- 旧product routeの`BrowserHostSession`はinstance epoch、Rectangle source、source equality、sequence、bounded inbox、snapshot projectionを既に所有する。
- `BrowserHostRuntime::fresh_instance_epoch()`と`built_in_rectangle_source(epoch)`はstaticだが、`BrowserHostRuntime`本体はWebView lifecycle ownerである。

最初の広いFable high相談runは300秒timeoutし、最終結論を返さなかった。途中stream、sample、推論はすべて不採用。

その後の縮小fresh Fable medium相談は、WebView非依存`BrowserHostSession`をsource projection／stale comparison ownerとして`REUSE`可能と返した。さらにread-only compilerは`R1-BROWSER-RN-SOURCE-READ-A`案を作り、create responseの別private fieldからRN Browserへidentityだけを表示する境界を提示した。

しかし利用者から「仮のもので通っても回り道」と指摘があり、ここで停止した。このcapsuleは**未発注・未実装・未review・未採用**である。次のsupervisorは古い`DO`として実行してはならない。

## 8. 次のfresh supervisorが最初に行うこと

1. 本書ではなくcurrent authority／code／Git factを再読する。
2. local main `9b2deac4`とroot dirty authorityの関係を確認する。
3. 利用者の直接outcomeを「表示中objectを選び、Stage gizmo releaseで同じPosition keyを一回更新し、TimelineとEasingへつながる」と置く。
4. Browser identity read-only表示がそのoutcomeのload-bearing edgeか、単なる見栄え／plumbing detourかを先に反証する。
5. load-bearingでなければ、Browser案を実装せず`REMAP / REDUCE`し、current codeに実在するselection geometry／wake ownerの成立箇所を一問ずつ探す。
6. exact owner、consumer、write route、positive／negative oracleが閉じた一契約だけを発注する。

再開まで外部LLM、Web probe、mutation、追加M3実装を起動しない。

## 9. 明示的非目標

- provider共通frameworkの新設
- failed research runのsample／threshold／feature row採用
- hardcoded Browser identity、先頭layer暗黙選択、fake geometry
- live update、selection、bounds、gizmo、Timeline、Position key、Easingの一括発注
- root dirty差分の整理、stage、commit、push
- local mainのremote push
