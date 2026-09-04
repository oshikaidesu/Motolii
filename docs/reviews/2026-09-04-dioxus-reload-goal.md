# 開発目標 — 普段の実装をホットリロードで完結させる

- 状態: **決定**(2026-09-04 利用者)。通常の機能追加・修正を、作品と実行状態を保ったホットリロードで完結させる。全体ビルドは、原因と改善責任が残る例外にする。
- [関数群契約](2026-09-04-function-group-contract.md)の開発ループ目標と、Dioxus内部の調査結果を補う。機械可読の正本は [function-contracts.json](../../motolii/reference/function-contracts.json) の `development_goal`。
- 実装は接続済み。代表経路の検収を通過し、最後の2窓確認とsource復元のみをrootが収束中。2026-09-04の追加指示「あなたを縛っているものを解放して、最適で最速」により、強制時間切れと過剰な段階分割を廃止する。必要な初回buildを同じfeature/profileとcacheで完走させ、以後warm processで進める。

## 1. 完了の定義

次の変更が、既存の型と意味の契約内なら、再起動なしで実際の挙動へ届くこと。

| 通常の変更 | 到達させる経路 |
|---|---|
| RSXの配置・条件・部品、CSS、asset | RSX/asset reload、必要なRust部分はhotpatch |
| メニュー・shortcut・control descriptor・素材/effectのデータ追加 | 型付きの表・既存部品への接続 |
| 数値計算・編集意味・gesture・描画準備の関数変更 | 明示した差し替え入口から群関数へ |
| 既存型で新しい関数・組み合わせ・moduleを足す | 関数ブロックと配線のhotpatch。毎回Shellや保持stateを改造しない |
| doc/renderを含む既存workspaceの関数本体変更 | 対応するworkspace replay経路を検収。所属crateだけで再起動の例外にしない |

保持対象はDocumentとUndo/Redo、選択、再生状態、Device/Queue、無関係なGPU資源。差し替え前に未確定の操作を取消し、永続変更を発生させない。主窓と別窓で同じコード世代の挙動へ届く。

**Rustのthin patchにもコンパイルはある。** これは通常の反復に含める。計測は `RSX_ASSET / THIN_PATCH / FULL_BASELINE / RELINK / RESTART / BUNDLE` を分け、ログ中の単語「build」だけで分類しない。通常変更の全体ビルド・再起動はゼロを目標とする。時間は[budget](../../motolii/reference/build-loop-budget.tsv)の改善目標(hotpatch 5秒、初回60秒)として測り、到達時にcompilerを殺さない。必要なbuild/testは進捗を観測しながら完走させる。

## 2. 「ビルドに痛みがある」の具体的な規則

全体ビルドが起きた事実、失った状態、原因となった境界、通常路へ戻す担当を見えるまま残す。普通の変更がhotにならなかった場合は `HOT_ROUTE_GAP` とし、全体ビルド後の成功で帳消しにしない。

- 全体ビルド/restartへ進む前に、既存の `BUILD_NEED`・`EXTERNAL_RULER`・`HOTPATCH_OR_CHECK_GAP` を具体化する。
- 変更ファイルとdiff、変更の種類、実行の種類、実コマンドとtoolchain/dx/lock/profile/features、開始前後のPID/build ID/patch世代、保持／再生成するowner、時間・終了値・fallback、通常路へ戻す修正と証拠を記録する。
- 「念のため」「最後だから」「workspaceのファイルだから」「dxが要求したから」だけを例外理由にしない。
- 例外候補は、初期baselineが無い、toolchain/固定依存/feature graph変更、保持型や永続schemaの非互換変更と移行、codegen/link固有の観測、要求された配布物。分類名だけで通さず、必要範囲を根拠で絞る。
- 根拠と既存の利用者指示で実行可能な例外はその範囲で進める。毎回の再承認や人工的な待ち時間を規則にしない。
- 計測不能・適用先不一致・無反映・失敗は成功にしない。時間目標超過はslowとして記録し、途中停止と混同しない。変更→生成→受理→挙動反映まで一件として追う。

```text
変更 → 種類と経路を分類 → RSX/asset または thin patch
                               ↓
                    受理 + 挙動 + owner保持を確認 → HOT_COMPLETE
                               ↓ 失敗/無反映
                          HOT_ROUTE_GAP → 接続を修正

真の例外 → 理由/影響の記録 → 最小のbuild/restart → 新baseline
                                                  ↓
                                        hot変更と保持を再確認
```

## 3. 使用するDioxusと内部の接続

現行入口は `scripts/motolii-dx.sh serve` → `reload-runtime.py` → **`motolii-dx-0.7.10-guarded`**。実行物とSHAの正本は [runtime descriptor](../../motolii/reference/dioxus-cli-runtime.json)。upstreamは **0.7.10 / 57d6794** で、[project patch](../../motolii/reference/dioxus-cli-0.7.10-reload.patch)が分類・fallback guard・TLS修正を持つ。旧 `-fixed` は出発点の実行物であり、現在の起動先として案内しない。以下のsource調査と§5の実窓証拠は区別する。

### 変更検知からビルド方式の選択

- [CLI runner](https://github.com/DioxusLabs/dioxus/blob/57d6794ad60b949e5bd8aa282f6f8c3dc97a365e/packages/cli/src/serve/runner.rs)の `initialize` はhotpatch有効時にFat baselineを作る。`handle_file_change` はRSX差分とassetを先に処理する。
- 同 `needs_full_rebuild` という変数は、hotpatch有効時には `order_changed_crates → patch_rebuild` へ進む。変数名だけで全体ビルドとは判定できない。
- [build request](https://github.com/DioxusLabs/dioxus/blob/57d6794ad60b949e5bd8aa282f6f8c3dc97a365e/packages/cli/src/build/request.rs)はThin時に `compile_workspace_hotpatch` を呼ぶ。捕捉済みrustc invocationを再利用し、依存順に変更crateを再生成する。
- Cargo/config変更はwatchされても、必ず設定を再読込して正しく再構築するとは限らない。graph更新、未知のfile/new crate、include由来の変更は専用の判定を要する。

### 説明文と実装の相違を修正する

[Subsecondの本文](https://github.com/DioxusLabs/dioxus/blob/57d6794ad60b949e5bd8aa282f6f8c3dc97a365e/packages/subsecond/subsecond/src/lib.rs)はtipのみと説明しているが、同版CLIにはworkspace replayが実装されている。**前の関数群契約の「下位crateの関数変更はすべて対象外」は撤回する。** 実際の対象範囲はcompiler invocationとlink可能なartifactで決まる。

一方、[CLI link実装](https://github.com/DioxusLabs/dioxus/blob/57d6794ad60b949e5bd8aa282f6f8c3dc97a365e/packages/cli/src/build/link.rs)と[compiler wrapper](https://github.com/DioxusLabs/dioxus/blob/57d6794ad60b949e5bd8aa282f6f8c3dc97a365e/packages/cli/src/rustcwrapper.rs)から、Motoliiの二つの具体的な接続問題が見える。

| 境界 | コードで確認した条件 | Motoliiへの影響 |
|---|---|---|
| 同packageのmain→lib | replay対象からtip packageを除外し、thinのcargo側はbinary targetを選ぶ | **UIはmain.rsから直接compileする形へ修正済み。** 代表的なUI関数変更を同じappへ受理した |
| `motolii-road` のdylib-only | wrapperのinvocation分類とworkspace linkはlibraryのrlib経路を前提とする | **roadはrlib+dylibへ変更し、unhashed rlibの取得も修正済み。** doc/render本体を含むworkspace replayは最終native runで到達を確認 |

これらは具体的な修正と実窓検収で閉じた。全workspaceに一律のbuild許可を与える理由にはしない。型layoutと保持stateの互換性はworkspace replayがあっても別の条件として残る。Fatも全依存のclean compileとは限らず、既存artifactを再利用する。

### 自動fallbackと状態の寿命

- [serve dispatcher](https://github.com/DioxusLabs/dioxus/blob/57d6794ad60b949e5bd8aa282f6f8c3dc97a365e/packages/cli/src/serve/mod.rs)は特定の `PatchError` でfull rebuildへfallbackする。一般のcompile/link失敗、情報不足によるignoredとは異なる。
- `automatic_rebuilds=false` はthin patch側も止めるため、これだけで「hotのみ許す」にはならない。**Fat/restartの直前を区別して制御し、thinは通常路として通す**。制御点は起動wrapperの外側だけでは足りず、固定CLIの既存分岐を使う。
- [Dioxus core component diff](https://github.com/DioxusLabs/dioxus/blob/57d6794ad60b949e5bd8aa282f6f8c3dc97a365e/packages/core/src/diff/component.rs)はrender関数の変更によりcomponentを置換し得る。hook/tasks/contextはscope破棄に従う。関数がpatchされたこととstateの保持は別々に検収する。
- [core properties](https://github.com/DioxusLabs/dioxus/blob/57d6794ad60b949e5bd8aa282f6f8c3dc97a365e/packages/core/src/properties.rs)のcomponent呼出しはHotFn経由。handlerは親の再実行から新closureへ更新する経路もあり、保持された古いcallbackを自動更新済みとは数えない。
- MotoliiのDocument/SessionはHostが保持する。Stage/Timelineも **MountStoreの状態ownerと、mount-localな接続へ分離済み**。Engine/実GPU textureを保持し、DOM nodeやrenderer-local ResourceId/Scene/Signal出口は再接続する。復旧後の代表的なThinとRSXで保持・再登録を確認した。単なる型配置から全変更の保持を推定しない。

### 実際のTLS失敗と復旧

最初のThinは2,686msで生成・受理されたが、Dioxusの `UNSYNC_OWNER` でborrow衝突、その後のHost callbackで破棄済みSignal参照が起き、appはexit 101になった。CLIがMach-Oの **`__thread_bss` をゼロ埋めせず、無関係な `__thread_data` のbytesをfallbackとしてコピー**し、RefCellのborrow flagを0から2にしたことをbinary/source双方で確認した。section別の初期値抽出と未知形式の拒否、Host callbackのepoch/登録IDによる更新・解除を実装した。

この失敗は[product receipt](../../motolii/reference/reload-product-receipt.jsonl)、修正とnative回帰は[driver receipt](../../motolii/reference/reload-driver-receipt.jsonl)に残した。終了したappを成功扱いせず、理由を記録した復旧baselineから§5の実窓確認をやり直した。

## 4. 運転入口と受理

guarded CLIとwrapperへ、一本のwarm process、実行物照合、原因を記録したbaseline/fallbackの入口を実装した。記録の無い初期baselineをnative dispatcherが実行前に拒否する試験は通過済み。CSSを先に通知しRust変更も失わないbatch処理を実装し、最終runのepoch1でCSS・UI Rust・docの混在変更を再生継続とともに確認した。

1. 既存warm serveの利用と新規起動を分ける。doctorで一本を確認してから二本目を起動しない。
2. stock入口・manual Cargo・restartを現在の正規路への無記録な代替にしない。診断不能をcount=0のPASSにしない。
3. [devtoolsの適用処理](https://github.com/DioxusLabs/dioxus/blob/57d6794ad60b949e5bd8aa282f6f8c3dc97a365e/packages/devtools/src/lib.rs)はbuild/PID不一致でもOkを返し得る。`verdict=applied`の一行だけで受理を断定しない。
4. patchはprocessで一度受理し、全窓の更新と保持を別に確認。hot5秒とbuild60秒は宣言だけでなく実際の経過と終了状態で判定する。

## 5. 検収と現在の状態

基準となる作品を開き、通常変更表の各族を変更して戻す。毎回、実際の動作変更、同じ作品/履歴/選択/再生、Device/Queueのidentity、全窓の反映、full/restartがゼロであることを記録する。少なくとも一つは初期の関数群を設計する時に使わなかった組み合わせで行う。

例外経路は、正当なbaseline更新と、普通の関数変更を誤ってfullへ送る反例の両方を扱う。後者は `HOT_ROUTE_GAP` のままであり、build後に絵が出ても通常路の合格にはしない。

**実装・検収完了 (COMPLETE、2026-09-05)。** 最終runは `/tmp/motolii-live-reload-final.log`、CLI **42238** / app **43349**。理由を記録したbaselineはbuild30.39秒、first paint38.25秒。Document **5712755216**、main Engine **5706051584**、Device/Queue **6961116553929376407**、catalog runtime **5685643920**を保持して通常変更を受理した。epoch1はCSS・UI Rust・docの混在変更を再生中に反映し、epochs15/16は新consumerの最初の読み取りからcanonical catalogとlast-goodを確認した。

新しい `gain.wgsl` はRust登録やcompileを伴わず追加され、cardとgain欄から1.0→0.25へ編集すると白から暗灰色になり、Undo一回で戻った。tabを移動しても保持した。不正blurはmagenta/radius12をRust patch越しに維持し、原文復元で白/radius8とerror notice消去へ到達。Blur16bit→Gradient8bit連鎖とUndoも実窓で確認した。元の復旧runでは、新placement moduleの追加、Anchor/Positionの補償、一Undo、妥当なgrid構造のRSX変更を確認済みである。

Doc **7**、関連UI **50**、Render/catalog実GPU **5**、最終Host **4**の回帰は各exit0。Hostには先のUI試験との重複がある。件数・所要時間・receiptは[採用計画](2026-09-04-pr-479-adoption.md#状態)を参照する。これらは各試験時点のsourceと代表操作を検査した結果であり、全関数を個別にhotpatchした証拠ではない。Thinには5秒を超えた回もあり、一般的な5秒以内は保証しない。

最後のsource復元と2窓の確認も完了した。epoch27後に開いた同じInspectorがLive→Hot→2.5Dを追従し、Gain既定値も0.5→1へ戻った。epoch30は両VDOMへ届き、catalog世代6・同じ作品・履歴・GPUを保持した。Inspectorのみを閉じて主窓へ戻り、白いStageを確認した。一時blur/CSS/table表示も復元済み。

### native接続で解決した反例

- workspace replayでcrateのstaticが複製され、Hostが保持するwatcherのArcと新consumerのcatalogが分裂した。root VDOM contextにcanonical Arcを置き、app/detachedが最初の読み取りより前に直接bindする。Hostでbindしたというログだけでは合格にしない。
- `call(|| function_pointer)` はclosureの差し替えであり、返されたcalleeの更新を保証しない。native WidgetとHostはcallee自体を `HotFn::current(typed_fn).call(args)` へ渡す。捕捉済みDioxus panicは、Rust受理後にroot ErrorContextをclearしてretryし、同じprocessで復帰した。任意のpanicを握り潰す規則にはしない。
- Vello **0.10.0**の `Renderer::mark_override_image_dirty` は、変更済みoverride textureを次のatlas描画へ再コピーするための規則を明記する。AnyRenderの口では画像を書いた後に新しいResourceIdを登録し、旧登録を遅延解除する。Stageは表示revision・catalog世代・時刻・viewport等をkeyにして、変化のないpaintで描画・登録を繰り返さない。[Velloの一次実装](https://github.com/linebender/vello/blob/v0.10.0/vello/src/lib.rs)
- roadのunhashed rlib、CSS+Rustの混在通知、catalog更新時の余分なbegin_frame、8bit/floatの前段入力は修正済み。TLS破損と強制時間切れの過去の失敗を消さず、修正後の結果から分離する。

別窓の終了操作はfullscreen状態を確認してから検査した。Escapeで退出し、AX close buttonでInspectorだけを閉じ、主窓のDocument/GPUを保持して再びInspectorを開いた。ghost windowを直した証拠とは扱わない。後のpatch imageから作った関数参照には次世代へのalias引き継ぎが必要で、修正の2回帰と後続2世代の別窓実測が通過した。

これらのnative接続は、現在のABIと有効なlayoutに対する契約である。保持型の非互換変更、rendererを跨ぐResourceIdの流用、通常dispose後の古いSignal呼出しは保証しない。主窓・別窓は同じ編集ownerを読み、各窓が自分のVDOM/rendererの接続を持つ。

新しい純関数やmoduleは既存入口から呼べる。baselineにない関数を新たな長寿命の入口として保持する場合は、識別契約または理由を記録したbaselineが必要であり、既存symbolのalias継承だけで対応済みとはしない。最終証跡は [reload-product-receipt.jsonl](../../motolii/reference/reload-product-receipt.jsonl)。
