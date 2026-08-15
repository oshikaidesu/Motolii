# Web窓とRN製品面の畳み込み

日付: 2026-08-16
状態: **決定**

## 決定

利用者裁定(2026-08-16): **Web窓(wry WebView)と React Native 製品面を畳む。** どちらも移行が途中で止まった残骸であり、生きた製品経路ではない。

[Web窓を含む製品projection正本化(2026-08-14)](2026-08-14-web-window-product-reflection-authority.md)を**撤回**する。本文は歴史として残す。撤回は既に[Blitz移行起案](2026-08-15-blitz-ui-runtime-adoption-proposal.md)の採択時に「明示的処分が必要な既決3件」の1つとして予告されていた。ここで実行した。

**残すもの**: `ui/motolii-rn/src/`(TypeScript 4,054行)。移植済みBlitzパネルが色・寸法を写した**唯一の出所**であり、`productStyles.ts` は現在も参照される。**Rust のビルドには元から一切入っていない**(TS なので)。畳んだのは Rust 側だけ。

## 何を落としたか

| 対象 | 行数 |
|---|---|
| `crates/motolii-ui/src/product_runtime/` (winit ネイティブ窓ランタイム) | 6,442 |
| `timeline_tools_host_runtime` / `stage_chrome_host_runtime` / `browser_host_runtime` / `inspector_host_runtime` (wry WebView host 4本) | 約 3,700 |
| `crates/motolii-ui/src/rn_product_host/` | 15,522 |
| `ui/motolii-rn/native-renderer/` (RN staticlib) | 14,993 |
| 合計 | **約 40,657** |

`wry` は `motolii-ui` の**直接依存**だった。**webview を置き換えるために Blitz を採択したのに、webview 本体をリンクしていた。**

## 根拠(現物の到達可能性。文書ではなくコードを数えた)

- `product_runtime::run` を呼んでいたのは `run_shell_with_project` だけで、それは同日に畳んだ旧 egui アプリの一部だった → **Web窓ランタイムは既に到達不能**
- `browser_host_runtime` / `inspector_host_runtime` は `mod` 宣言と、ソース文字列を検査する `ui_numeric_trace.rs` のテスト以外から参照されていなかった
- RN 面は利用者裁定による(到達可能ではあった。下記「これは死んだ枝ではない」を参照)

## 先に引き剥がしたもの(製品が使っていた部品)

畳む前に、内側にあって外から使われていた部品を出した。**この順序を守らないとビルドが壊れたまま止まる。**

| 部品 | 元の場所 | 移した先 | 使っていたのは |
|---|---|---|---|
| `canonical_drop_from_ndc` | `app/browser.rs` | `canonical_drop.rs` | `product_runtime/` 14ファイル |
| `create_preview_pipeline` | `product_runtime/surface.rs` | `preview_pipeline.rs` | `rn_product_host/gpu_surface.rs` |
| `app_stage_geometry` | `rn_product_host/stage_projection.rs` | `stage_app_geometry.rs` | **合体シェルの Stage** |
| `TimelineFrameBorrow` | `rn_product_host/surfaces.rs` | `timeline_skia_raster.rs` | 同ファイル |

`app_stage_geometry` は音声・書き出し・転送・GPU を抱えた444行のモジュールに同居していたが、投影自体は `Document` と `Affine2D` しか見ない。RN の wire 型を経由せず `AppStageGeometry` へ直接畳むように変えた(渡す相手が居なくなったため)。数値の扱いは変えていない。

## これは死んだ枝ではない — 製品を一度捨てる決定である

旧 egui アプリと Web窓は**誰からも呼ばれていなかった**が、**RN は違う**。畳むことで:

- **`timeline_skia_raster` を呼ぶ者が居なくなった。** 唯一の消費者が `rn_product_host/timeline_gpu.rs` だった
- **repo に「触れる製品」が無くなった。** 残るのは合体シェル(`motolii-blitz-shell`)だが、**入力が1つも通っていない**
- 移植元の `productStyles.ts` は残るが、**それを読んで動く RN アプリは無い**

同日の[製品Timelineの正本はSkia](2026-08-16-skia-timeline-authority-correction.md)は「製品 Timeline は Skia、`rn_product_host` から呼ばれる」と書いた。**それは RN 製品についての事実であり、その RN を数時間後に畳んだ**。Skia は意味・hit・oracle の源として残すが、**Timeline の座席は現在空席**である。唯一動くシェルで唯一描いている Timeline は `timeline_blitz/` である。

## 測った効果

`[profile.fast]` で `motolii-ui` を1行触ってから単一バイナリを建て直す時間:

| 時点 | 時間 |
|---|---|
| 今夜の開始時(release、profile 設定なし) | 23.4s |
| `[profile.fast]` 導入後 | 6.5s |
| Web窓撤去後 | 4.4s |
| RN撤去後 | **3.10s** |

`motolii-ui` は 57,784 → **32,289行**、警告は 1,136 → **306**。

**依存の数はほとんど動かない**(746 → 734)。`wry` 固有のcrateは消えたが、その木の大半(objc2 系など)は `winit` / `eframe` / `wgpu` と共有されていた。**効いたのは自分のコードの量であって、依存の本数ではない。** 当初「69クレート消える」と見積もったのは誤りだった。

撤去後の確認: Browser の dump は撤去前と**バイト一致**、合体シェルは**5ペインとも描画**(Stage の実幾何を含む)。

## 失われた検査(2026-08-16 追記)

撤去した経路を被写体にしていた `crates/motolii-ui/tests/` の**12本・1,185行を削除した**。

| 被写体 | 本数 |
|---|---|
| `product_runtime`(winit窓＋Web窓ランタイム) | 10 |
| `motolii_ui_shell`(旧eguiアプリ) | 2 |

**いずれも振る舞いではなくソース文字列の検査**(`include_str!` + `contains`)であり、
`include_str!("../src/product_runtime.rs")` のように**撤去以前から存在しないpath**を
読んでいたものもある(`product_runtime` はディレクトリだった)。つまり元々コンパイルできず、
引き継ぎに残っていた「テストが通らない」の一部だった。

**ただし検査の意図まで無効になったわけではない。** 例えば CU-111 は
「製品の Undo/Redo が `command_registry` の `motolii.edit.undo` / `.redo` へ
一意に配送される」ことを見ていた。**その意味は生きている**が、通す経路(製品ホスト)が
無くなったので今は確かめようがない。**入力(C2)が配線された時点で、
同じ意図を新しい経路で組み直すこと。** 削除した内容は `git show 259d62a9^:` 以降で読める。

## 残余

- **Timeline の座席が空席。** `timeline_blitz` を製品面と見なすのか、Skia を別の宿主から呼び直すのかは未決
- **入力が通っていない。** 製品と呼べるものにするには C2(入力ルーティング)が要る
- `timeline_viewport_state`(341行) / `timeline_intent_adapter`(166行) は現在参照ゼロだが**残してある**。Timeline 移植の配線先であり、消すと同じ作業を書き直すことになる
- `palette_settings` も参照ゼロだが残す。[ユーザーパレット契約(2026-08-14)](2026-08-14-user-palette-library-contract.md)の実装で、配線前の状態
