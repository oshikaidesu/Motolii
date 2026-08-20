# 正本索引(CANON)

このリポジトリは長期間の設計探索で資産が世代交代を繰り返しており、**同じ主題を指すファイルが複数存在する**(旧HTMLモック、旧crateモジュール、旧実行地図)。それ自体は悪くないが、docsのどこにも「今どれが正本か」を1枚で集約した場所が無く、外部LLMレーンや新規セッションが旧版を正本と誤認して発注・実装する事故が実際に起きた(2026-08-18/19、[経緯](#誤読の実例と訂正)参照)。

**このファイルの役割は索引だけ**。各行の詳細・理由・経緯は正本自身、またはリンク先のreview/decision文書を読む。ここに新しい設計判断を書かない。

各行に**最終更新(git logの日付)**を入れる。古さが一目でわかることを目的とする。

## 視覚(見た目)の正本 — 面ごと

| 面 | 正本 | 最終更新 | 備考 |
|---|---|---|---|
| Inspector | `docs/mocks-ui/public/inspector-library.html` + 同名`.css` | 2026-08-16 | `crates/motolii-ui/src/inspector_panel/theme.rs`が冒頭で「全部`inspector-library.css`の写し」と自己宣言。fallback値は`docs/mocks-ui/src/tokens/mock-candidates.css`の実値と一致確認済み |
| Browser | `docs/mocks-ui/public/browser-library.html` + 同名`.css` | 2026-08-16 | `crates/motolii-ui/src/lib.rs`の`browser_panel`docコメントが同ファイルを正本と明記 |
| Timeline | `crates/motolii-ui/src/timeline_editor/`(egui実装そのもの) | 実装2026-08-18 / 裁定2026-08-19 | **他の面と逆方向**: 2026-08-19利用者裁定「タイムラインに関してはegui版が最も機能を詰めれていて優れている、UIも」により、Timelineの再現目標は`docs/mocks-ui/public/timeline-library.html`(2026-08-16、副参照へ降格)ではなく**egui実装自身**になった。iced側(`crates/motolii-shell-iced/src/timeline/`)がこの実装へ追いつく途上(下記) |
| chrome(titlebar / splitter / modal / extension panel) | `ui/motolii-rn/src/productStyles.ts` + `chrome.tsx` + `panels/{registry.tsx,AssetTaggingPanel.tsx}` | 2026-08-11〜13 | `crates/motolii-ui/src/chrome_blitz/theme.rs`が同ファイル群を「写し」と自己宣言。**`chrome-library.html`という名のファイルは`docs/mocks-ui/public/`に存在しない**(Inspector/Browserと違い、chromeの正本はReact Native側のTypeScriptソースであってHTML/CSSモックではない) |

RN由来のBrowserモックが別にもう1つある: `docs/mocks-ui/public/rn-browser-161c7ccd.html`(2026-08-16、`ui/motolii-rn/src/Browser.tsx`固定commit`161c7ccd`からの投影専用・Host/Document/intent/drag/persistence非接続)。上表の`browser-library.html`と役割が異なる(こちらは視覚回帰の投影専用、上表は製品egui実装が直接写す対象)。

**2026-08-19 利用者裁定 — 面ごとに「手本」が違う(取り違え注意)**:

- **Browser / Inspector**: 正本は上表の HTML/CSS **そのもの**。**egui 実装(`browser_panel` / `inspector_panel`)は手本にしない** — 利用者の判断で「egui 変換が上手くできなかった部分」。定数が css と一致していることと、構造・階層が設計の意図どおりであることは別問題。iced 側は **HTML から意図(section 階層・class の意味・行の内部構造・状態の表現)を解析して**作る。egui から拾ってよいのは**振る舞いの結線と意味関数**だけ
- **Timeline**: 逆。**egui 実装が正本**(2026-08-19 裁定「egui 版が最も機能を詰めれていて優れている、UI も」)。`timeline-library.html` は副参照

器具: `motolii-css-metrics`(`motolii_ui::css_metrics::extract()`)が HTML/CSS の計算済み値を吐く。写経せず器具の値を根拠にする。**罠**: `<link>` は解決されない(inline 要)/ 帯・アクセントバーは `::before`/`::after` = `AnonymousBlock` なので Element だけ歩くと消える / JS 依存の初期状態は再現されない。

## Timeline engineの実装

| 実体 | 最終更新 | 状態 |
|---|---|---|
| `crates/motolii-ui/src/timeline_editor/`(`mod.rs` 8,186行 + `audio_seat.rs` + `import_seat.rs` + `waveform_band.rs`、計**9,059行**) | 2026-08-18 | **正本**。`crates/motolii-ui/src/lib.rs`のdocコメント「egui Timelineエディタ(旧labの本体)」。`blitz_shell`のTimeline paneと`examples/timeline_egui_lab.rs`の薄殻が同じ実装を呼ぶ(`grep -rn timeline_editor crates/motolii-ui/src/blitz_shell/`で配線確認可) |
| `crates/motolii-ui/src/timeline_skia_raster.rs` | 2026-08-16 | **死蔵**。`lib.rs`に`#[cfg(target_os = "macos")] mod timeline_skia_raster;`とだけ宣言され、他のどこからも参照されていない(2026-08-16に一度は製品正本と裁定されたが、同日中にegui再選定で上書きされた経緯は[decision-index.md](decision-index.md)のTimeline行を参照) |
| `crates/motolii-shell-iced/src/timeline/`(semantics / pane / canvas / waveform) | 2026-08-19 | **現行製品route**。egui版を視覚・機能参照としてicedへ移植する。個別能力の現在値と残余は[egui Timeline能力台帳](reviews/2026-08-19-egui-timeline-capability-ledger.md)で確認し、日付付きhandoffの欠落一覧を現在値にしない |

**`crates/motolii-ui/src/timeline_egui.rs`という名のファイルは存在しない**(旧`timeline_egui/`961行は2026-08-16に削除。原文は`git show f209da9d^:crates/motolii-ui/src/timeline_egui/mod.rs`)。この名前で発注されたレーンが「移植元が無い」と誤報告した実例が[2026-08-18セッション引き継ぎ](reviews/2026-08-18-session-handoff-iced-four-pane-campaign.md#追記3--視覚第2ラウンド着地と次の本題2026-08-19-朝)にある。

## token

| ファイル | 最終更新 | 役割 |
|---|---|---|
| `ui/motolii-tokens/sources/motolii-dark.json` | 2026-07-29 | **手書きの正本**。DTCG形式(`$schema: designtokens.org/schemas/2025.10`) |
| `ui/motolii-tokens/generated/tokens.css` / `tokens.rs` | 2026-07-29 | `motolii-ui-token-gen`が`motolii-dark.json`から機械生成。ファイル冒頭に`DO NOT EDIT`。Rust/egui adapterと下記`accepted-route-product-tokens.css`が直接参照する対象 |
| `docs/mocks-ui/src/tokens/mock-candidates.css` | 2026-07-31 | `--mock-candidate-*`。HTMLモックから移した**比較用候補値であり製品tokenではない**。component/adapterから直接参照禁止(ファイル冒頭に明記)。現時点では生成tokenと数値が一致することを確認済みだが、それは検証結果であって正本の理由ではない |
| `docs/mocks-ui/src/tokens/accepted-route-product-tokens.css` | (mock-candidates.cssと同時期) | `generated/tokens.css`を`@import`し、mocks-ui側の`--bg`/`--panel`等の変数名へ再マップする橋。採択route(`#plugin-browser-candidate`)がこれを使う |

一行でまとめると: **`motolii-dark.json`(手書き)→ 生成 →`generated/tokens.css`(Rust/egui・採択routeが読む)**。`mock-candidates.css`は別系統(HTMLモック由来の比較値)で、混同しないこと。

## 製品shell

| shell | crate / bin | 最終更新 | 現在地 |
|---|---|---|---|
| iced | `crates/motolii-shell-iced`、bin `motolii-shell-iced` | 2026-08-19 | **現行製品host / 新規機能target**。M-0〜M-4で4 pane、Stage島、`UiIntent` gateway、drive/replay oracleが統合済み。未実装能力や視覚残余があるため製品完成を意味しない |
| egui | `crates/motolii-ui`(`blitz_shell`モジュール)、bin `motolii-blitz-shell` | 2026-08-19 | **legacy/reference**。Timelineの参照実装、Rerun Stage島の内部実装、比較・回帰器具として残る。明示依頼なしに製品機能を追加したりfallback先にしない |

2026-08-19にhost authorityをicedへ切り替えた。既定bin名やlauncherに機械的な残余があっても、それはauthorityをeguiへ戻さない。「Motolii Studioを起動」等の要求への応答は[ui-artifact-terminology.md](ui-artifact-terminology.md)の起動ルールと本表を併用する。

## 撮影器具 — それぞれ何を撮るか

| 器具 | 撮る対象 | 備考 |
|---|---|---|
| `motolii-blitz-dump`(bin, `motolii-ui`) | 個別Blitzパネル(`timeline`/`browser`/`dock`/`chrome-export`/`chrome-settings`/`chrome-panels`/`chrome-parts`)をHTML→PNGへ直描き | 組み立て済みのshell窓ではない。`cargo run -p motolii-ui --bin motolii-blitz-dump -- <対象> <出力先>` |
| `motolii-inspector-blitz-dump`(bin, `motolii-ui`) | `inspector_blitz`が出すHTMLだけをPNG化する小道具(C7判定材料) | Inspector単体、offscreen |
| `motolii-blitz-shell --screenshot`(bin, `motolii-ui`) | **egui製品shell窓の実撮り** | `cargo run -p motolii-ui --bin motolii-blitz-shell -- --screenshot out.png [frames]` |
| `motolii-shell-iced --screenshot`(bin, `motolii-shell-iced`) | **iced製品shell窓の実撮り** | 2026-08-18夜に追加(「以後の視覚検収の常設器具」)。`--screenshot <out> [frames]`。**frames=25では非同期評価が間に合わずStageが空に見える実測あり。120を使うこと**([出典](reviews/2026-08-18-session-handoff-iced-four-pane-campaign.md#追記2--修復の検証結果2026-08-19-未明supervisorが実窓で確認)) |
| Playwright(`docs/mocks-ui/scripts/*.mjs`: `reference-capture.mjs`、`current-route-capture.mjs`等) | **CSSモック自体**(`inspector/browser/timeline-library.html`等)のPNG化・回帰証跡生成 | Rust側の製品shellは撮らない。`reference-output/`・`current-route-output/`の`generations/`へ出力し`CURRENT`ファイルが最新世代を指す |

2026-08-18深夜に使われた`capture-design-reference.mjs`はscratchpad上のアドホックスクリプトで、**リポジトリにはコミットされていない**(`docs/mocks-ui/scripts/`には存在しない、恒久器具ではないので注意)。

## 誤読の実例と訂正(このレーンで対応した分)

| 誤読の実害 | 場所 | 訂正 |
|---|---|---|
| 「視覚構成の基準は高密度メインUIモック(m3-main-ui-v1)」が旧版を指したまま残っていたため、supervisorが旧版を再現目標として発注した | `docs/ui-visual-language.md:7` | 訂正注記+取り消し線を追加。現行正本(上表)を指すよう修正。該当html/pngは`docs/archive/m3-main-ui-early-mocks/`へ移動 |
| `timeline_egui.rs`が正本として台帳・メモリに書かれていたが、そのファイルは存在しない(2026-08-16削除)。この名前で発注されたレーンが「移植元が存在しない」と誤報告した | `docs/decision-index.md`、`docs/ui-friction-ledger.md`、`docs/blitz-port-order-capsules.md`(C1〜C3) | `blitz-port-order-capsules.md`のC1〜C3capsuleに失効注記(このcapsuleは発注不可)。`ui-friction-ledger.md`にパス訂正の脚注。`decision-index.md`は各エントリが既に自己訂正の連鎖(撤回→撤去済み)を持っていたため無変更 |
| `motolii_ui_shell`(2026-08-16撤去済み)を現在の起動先として書いていた | `docs/ui-reference-map.md`、`docs/ui-artifact-terminology.md` | 各ファイル冒頭に既知の陳腐化バナーを追加。現行shellは本ファイルの上表を見よと明記 |
| `docs/m3-rn-runtime-execution-map.md`・`docs/implementation-ledger.md`が個別ファイル名(`rn_product_host.rs`等、いずれも撤去済み)を現在の実装場所であるかのように列挙 | 同上2ファイル | 冒頭に現在地バナーを追加。本文の個別ファイル名は歴史記録として残す(全面書き換えはしない) |

上記以外にも、`docs/`配下の"アクティブな作業台帳"(`m3-parallel-implementation-map.md`、`m3-executable-dispatch-map.md`、`decision-index.md`本体、`docs/reviews/**`)には、実在しないファイルパスへの言及が多数残っている(スキャン結果は本レーンの作業報告を参照)。これらは日付付きの**履歴記録**であり(このリポジトリの規律「棄却物も歴史証拠として残す」に従う)、個別に全数訂正すると本文の大部分を書き換えることになるため、このレーンでは**現在も読まれる可能性が高い入口文書**(上記4件)だけを訂正した。個別の古いファイル名に迷ったら、まずこのCANON.mdで現在の実体を確認すること。

## アーカイブ

世代交代が確定した資産は`docs/archive/`へ移した(削除はしない)。内容と移動理由は[docs/archive/README.md](archive/README.md)を見よ。**移動を検討したが対象外と判断したもの**(ビルド時依存があるファイル、decision台帳が「移動しない」と明記した回帰証拠など)も同READMEに記録してある。

## 今後の再発防止(提案・強制しない)

- **CANON.mdを触る条件**: 上記いずれかの行が指す正本ファイルが移動・世代交代・撤去された時、またはこのレーンのようにgrepで「誤読の実害」が実測された時
- **正本が動いた時にすること**(推奨する3手順): (1) 移動先ファイルの先頭docコメントに出所を書く(このリポジトリの既存慣行そのもの。`inspector_panel/theme.rs`等が実例) (2) このファイルの該当行と最終更新日を更新する (3) `grep -rn <旧パス> docs/`で参照が残っていないか確認し、残っていれば「歴史記録として残す/訂正する」を個別判断する(全部消す必要は無い)
- **恒久化の判断は利用者に委ねる**: このCANON.mdの維持を`scripts/check-docs.sh`のような機械チェックへ昇格するかどうかは、このレーンでは提案しない(「Fableの役割は回収であって仕組み化ではない」)
