# 製品 Timeline の正本は Skia — egui Timeline の訂正と撤去

日付: 2026-08-16
状態: **決定**

> **同日追記(2026-08-16)**: 本文書の後に **RN 製品面を畳んだ**ため、`timeline_skia_raster` を
> 呼ぶ者が居なくなった(唯一の消費者が `rn_product_host/timeline_gpu.rs` だった)。
> 「製品 Timeline は Skia」は**RN 製品についての事実**であり、その RN が無い。
> Skia は意味・hit・oracle の源として残すが、**Timeline の座席は空席**。
> → [Web窓とRN製品面の畳み込み](2026-08-16-web-window-and-rn-product-fold.md)

## 決定

利用者裁定(2026-08-16): 製品 Timeline を描いているのは **Skia**([`crates/motolii-ui/src/timeline_skia_raster.rs`](../../crates/motolii-ui/src/timeline_skia_raster.rs))であり、これは以前から変わっていない。egui Timeline は**移行が途中で止まった残骸**であり、コードごと畳んだ。

[2026-08-15 egui Timeline engine 正本](2026-08-15-egui-timeline-engine-authority.md)のうち、**「Timeline engine／皮の正本は `timeline_egui.rs`、描画・hit・pointer を持つ」**を撤回する。同文書の本文は歴史として残す。

**維持するもの**: 同文書の「Rerun Time Panel へ engine 相乗りしない」という否定側は維持する。今回の訂正は engine がどちらの自前実装かという話であり、Rerun 相乗りを復活させるものではない。

| 座席 | 正本 | 備考 |
|---|---|---|
| Timeline engine／皮 | Skia `timeline_skia_raster.rs` | `rn_product_host/timeline_gpu.rs` から呼ばれる |
| Timeline の Blitz 移植面 | `timeline_blitz/` | **位置づけ未決**(下記 残余) |
| 編集意味 | Document／D2 | 変更なし |
| Stage spatial | Rerun Spatial Viewer | 変更なし |

## 根拠(現物の参照グラフ。記憶や文書ではなくコードを数えた)

| 実装 | 行数 | `::` 参照数 | 参照元 |
|---|---|---|---|
| `timeline_skia_raster.rs` | 314 | **1** | `rn_product_host/timeline_gpu.rs` ← **製品ホスト** |
| `timeline_egui/` | 961 | 14 | `app/`(旧egui アプリ)、自身のテスト、`timeline_clipboard_intents`(誰からも使われていない) |
| `timeline_blitz/` | 739 | 5 | 合体シェルのペイン |
| `ui/motolii-rn/src/Timeline.tsx` | 201 | — | RN 製品ソース |

**`timeline_egui` を製品経路から呼んでいる箇所は1つも無かった。** 参照元はすべて旧 egui アプリ側か、そこにしか使われていない死んだ module である。

利用者が `motolii_ui_shell`(旧 egui アプリ)を実機で起動して確認済み。出力は `U1A1_REGISTER` / `U1A2_LAYOUT` / `U1B2_JOIN` のスモーク行のみで、「全然ダメだったから放置していた」もの。

## 施工

commit `f209da9d`。**3,338行削除**。

- `app/` 7ファイル(旧 egui アプリ本体)
- `timeline_egui/` と `timeline_egui_interaction_tests.rs`
- `src/bin/motolii_ui_shell.rs`(旧 egui アプリのバイナリ)
- `shell.rs` の `run_shell` / `run_shell_with_project` / `run_shell_inner` とスモーク配管(218行 → 49行)
- `timeline_clipboard_intents.rs` — `mod` 宣言以外に参照が無かった

前段として commit で `canonical_drop_from_ndc` を `app/browser.rs` から `canonical_drop.rs` へ移した。カメラの逆射影だけでアプリ状態を見ない関数がそこに置かれていたため、`product_runtime/` の14ファイルが旧アプリへ依存していた。**この1本が `app/` を撤去できなくしていた唯一の実害**。

**残した**もの: `shell.rs` の `open_project_runtime` と `ShellError`(製品 `rn_product_host/registry.rs:113` が呼ぶ)、`toolkit_linked`。

撤去後の確認: Browser の dump は撤去前と**バイト一致**、合体シェルは**5ペインとも描画**。

## 残余(未決)

- **`timeline_blitz/` の位置づけ。** 合体シェルが描いているのは Blitz 版であり、正本(Skia)ではない。皮の実験として残すのか、Skia 側から取り直すのか、畳むのかは決めていない。[Blitz移植発注capsule](../blitz-port-order-capsules.md) の C1 は「Timeline描画をBlitzへ」のままである
- `timeline_blitz/rows.rs` と `html.rs` は doc コメントで `timeline_egui` を出所として参照している(コード依存は無い)。出所の記述は正本の変更に合わせて見直しが要る

## 上書きした既存の指示

[決定逆引き台帳](../decision-index.md)の該当行(2026-08-15)は、engine 指名を既に**撤回**扱いにした上で「`timeline_egui/` は `timeline_skia/` と同じく**意味・色・寸法の源として残す**(C1 の READ SET)。**削除しない**」と書いていた。

2026-08-16 の利用者裁定はこれを上書きする。理由は、源として参照していた `timeline_blitz/` が既に**写し終えている**(`rows.rs` / `html.rs` の doc コメントが出所を file:line で記録済み)ため、生きた source tree として置く必要が無いこと。

**源は失われていない。** 撤去前の全文は `f209da9d^` にある:

```
git show f209da9d^:crates/motolii-ui/src/timeline_egui/theme.rs
```

出所を追う必要が出たら、この ref を参照すること。

## この誤りが起きた機序(再発防止)

2026-08-15 の決定文書は「engine の正本は egui」と書いていたが、**現物では egui は製品のどこからも呼ばれていなかった**。文書と現物の食い違いを検出する手段が無かったため、後続(2026-08-16 のセッション)は文書を信じて「シェルの Timeline を egui に寄せるべき」と提案しかけた。

決定文書には**「現物のどこに配線されているか」を書く欄が無い**。座席表に正本の file を書いても、それが実際に呼ばれているかは別問題である。参照数を1行入れておけば、この誤読は起きなかった。

規律6点([reviews/README](README.md))は調査文書の結論を設計根拠にしないことを求めているが、**決定文書そのものが現物と乖離する経路**は塞いでいない。本文書では上の「根拠」節に参照グラフを置いた。同種の座席決定では同じ表を置くことを推奨する(規則化はしない — 提案に留める)。
