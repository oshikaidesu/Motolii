# ホストを iced へ — 乗り換え裁定と移行地図

日付: 2026-08-18
状態: **決定**(利用者裁定: 「変な感じはしない、なによりコード数が決定的すぎる、乗り換えるか」)

## 裁定と根拠(全て同日の実測)

シェルのホスト toolkit を egui から **iced(master 系)** へ移行する。根拠:

1. **DX 実測**([仮タイムライン spike](2026-08-18-iced-reentry-survey.md)): 同4ジェスチャが
   342行 vs 約1,415行、draw パス内の永続状態書き換え 0(型が禁じる) vs 97文。
   利用者が spike を実際に触り「変な感じはしない」と確認
2. **段差ゼロが公式不変量**: iced_test(headless)+ time-travel が「操作=Message列」
   を一級機能として保証 — 本日自前で建てた「ログと構造の強制」をフレームワークが
   肩代わりする
3. **Rerun はホスト非拘束**(3 probe 実測): offscreen+入力ブリッジで iced 内に
   絵・入力・カメラまで成立済み。egui は Stage 島の内側の実装詳細に縮む

## 引き受けるコスト(実測済み・目をつぶらない)

- **fork 2本体制**: rerun(camera seat 済み)+ **iced fork 新設**(web-sys 釘打ち
  1行解除・`max_bind_groups` 定数 — どちらも上流 PR 候補)。seam 台帳方式は
  [rerun fork seam ledger](2026-08-18-rerun-fork-seam-ledger.md)を踏襲
- master 追随(1.0 前・minor ごと破壊的変更)。rev pin+常設 oracle での検収は
  rerun fork と同じ型
- **AccessKit 後退**: egui は統合済み・iced upstream は未統合。運転席の
  AccessKit query は iced_test の Selector へ置換。アクセシビリティ自体の後退は
  iced 上流(System76 が作業中)の再観測点として残す
- WheelScrolled が modifiers を運ばない等の小穴(spike で回避策実証済み)

## 移行地図(絞め殺し方式 — campaign を止めない)

**不変の資産(移行対象外)**: motolii-doc / render / export / media / audio /
plugin(D2・journal・評価器)。**UiIntent 背骨は設計どおり持ち越す**(intent ≒ Message)。
`motolii-input`(2,413行)は toolkit-free で無傷。

- **M-0 土台**: iced fork 作成(pin rev+seam 2件+seam 台帳)、`motolii-shell-iced`
  crate 新設(dep policy へ iced 系 allowlist を追加)。egui shell は**並走のまま**
  — **着手済み**(2026-08-18)。fork の乖離と再適用手順は
  [iced fork seam 台帳](2026-08-18-iced-fork-seam-ledger.md)。柵は2本(iced を持てるのは
  新殻だけ / 新殻は egui を持てない)で、後者は egui 側 allowlist へ新殻を**入れないこと**で
  成立している
- **M-1 殻**: スタート画面・New/Open/Save・status 帯・prompts 台本・
  `--intent-log`/replay・iced_test の新運転席(kittest 相当の駆動+replay oracle)
  — **完了**(2026-08-18)。`Cmd+S`+未保存 guard(3択は egui と同じ `decide_unsaved`)・
  OS ドロップ→`AdmitPaths`・Export 開始/キャンセル(`export_seat` 共用)・
  `--status-log`・replay oracle の iced 版が green(運転席 27本)。
  dialog は**共用**になった: `ShellPrompts`/`NativePrompts`/`ScriptedPrompts` を
  `motolii_ui::blitz_shell` から `pub` にして差すので、`rfd` を呼ぶ場所は repo に
  1箇所のまま(新殻から `rfd` 依存を落とした)。
  **実測できた注入の境目**: `iced_test::Simulator` は生の `iced::Event` を流せるので、
  近道キー(修飾つき `KeyPressed`)・OS ドロップ(`window::Event::FileDropped`)・
  閉じる要求(`CloseRequested`)は**全部 headless で注入できる** — ただし
  それらを widget 木で受けている限りである(購読 `keyboard::listen` /
  `window::events()` に置くと Simulator の外に出る)。木で受ける薄い widget
  (`window_input`)を1枚置いたのはこのため。購読に残るのは書き出しの刻み
  (`window::frames()` → `ExportPolled`)だけで、そこは同じ Message を直に流して審判する
- **M-2 Stage 島**: 入力ブリッジ probe を製品 adapter 化(camera seat・正対既定・
  transcript 相当の失敗報告) — **完了**(2026-08-18)。bind group 床の実効実測を
  2→4(fork seam 台帳§4の受入条件を閉塞)、ギズモの掴みは3状態の調停
  (`grab_probe`)、pixel 証拠は
  `docs/reviews/evidence/iced-m2-stage-island/`。Rerun の合成絵・入力は
  `EmbeddedSpatialStage` 経由のままで egui へ依存しない(新しい入力 seam を
  作らない)
- **M-3 Timeline**: spike を種に Document へ結線(prepare_*/D2 は既存。egui 版の
  意味関数・oracle は移植元)。波形帯・audio seat の載せ替え — **完了**(2026-08-18)。
  `motolii-shell-iced/src/timeline/` に semantics(移植した意味関数)/ pane
  (ジェスチャ状態機械)/ canvas(絵と翻訳)/ waveform(座席の写し。縮約は
  `WaveformPeaks` を共用)。**編集は全部 `UiIntent` 経由**: Timeline の編集 intent
  (`SelectLayer` 〜 `StepPlayhead`)を共有ゲートウェイ(`blitz_shell::intent`)へ
  実装し、egui pane の `project_mut()` の穴は新殻に持ち込まなかった
  (柵: `tests/intent_gateway_fence.rs` が `editor_mut(` を禁止)。
  ドラッグは **release の1件だけが intent**(preview は pane、Document は release
  まで無傷 — Esc=復元が「何も起きていない」の同義になる。Skia 側 transient
  lifecycle と同じ判断で、egui の live-commit とは違う)。運転席は
  `tests/drive_timeline.rs` の 14本(Q1 文法・複数選択の塊クランプ・scrub・
  コマ送り・zoom/pan・intent 列 replay・Message 列 replay(表示状態)・波形帯)。
  zoom / pan / scroll は view 状態なので intent にせず、再現は Message 列 replay が持つ
- **M-4 Browser / Inspector**: iced widget 化(標準 widget 領域 = iced の得意面)
  — **完了**(2026-08-18)。M-4a Browser は rail 3種(All media / Project /
  Recent)・double-click=`AdmitPaths`(OS ドロップと同一レール)・Browser の
  カード選択は pane-local のまま(外部候補の選択は Document 外 — 正典どおり
  Timeline/Stage の選択へは繋がない)。M-4b Inspector は 4 section 常設
  (Audio は口が無いため不出=Q0)・wave E の編集 intent
  (`ToggleItemFlag`/`BeginParamEdit`/`SetParamComponent`/`EndParamEdit`/
  `KeyParamAtPlayhead`/`SetEffectEnabled`)を実装
- **M-4 統合(1・2弾)**: M-4a+M-4b+M-4t(theme)+M-4w(widgets)+M-2+M-3 を1本の
  `motolii-shell-iced` へ合流 — **完了**(2026-08-18)。4面合成は
  `docs/ui-interaction-language.md` §3 の既定配置どおり、上段
  (Browser|Stage|Inspector、既存のまま)+下段 Timeline(canvas)。上段:下段の
  高さ比は egui shell(`blitz_shell/app.rs::build_initial_tree`)の中央列
  (Stage/Timeline を `insert_vertical_tile` へ明示 share なしで渡している =
  `egui_tiles` の既定等分割)を正典にして 1:1 を採った。**選択の合流**:
  M-4b が先に足していた `UiIntent::SelectLayer { layer: u64 }` と、M-3 が
  独立に足していた `SelectLayer { layer: LayerId, additive: bool }` は
  同名で共存できないため union で1本化した
  (`SelectLayer { layer: u64, additive: bool }`)。JSON 形は変わらない
  (`LayerId` が `#[serde(transparent)]` なので数値表現は同じ) — Stage/
  Inspector クリックは `additive: false` を渡し、Timeline のクリック
  (Cmd 併用可)は実値を渡す。Timeline の行選択 → Inspector 反映は
  `crates/motolii-shell-iced/tests/replay_oracle.rs`
  (`timeline_row_pick_selects_the_same_layer_the_inspector_reads`)が台本
  として持つ。fence 修理: `motolii-testkit` の
  `resource_owner_inventory::every_product_raw_gpu_allocation_callsite_has_an_owner_seat`
  が base 時点で赤だった(M-2 の `stage_island.rs` の
  `device.create_texture` 2箇所が GPU 割当台帳に未登録) — 台帳の既存流儀
  (owner seat = name/lifetime/peak_multiplicity)で登録して green にした。
  同時に `intent_gateway_fence.rs::every_product_source_is_scanned` も base
  時点で赤だった(M-4t/M-4w が足した `theme/` `widgets/` が走査表に無かった)
  ので、走査表へ足した。targeted 検証は `-p motolii-shell-iced` で 100 passed
  / 0 failed。**full workspace gate**(`cargo test --workspace --no-fail-fast
  -j 5`、統合第2弾の受入条件)は 237 test binary 全部 green、**2131 passed /
  0 failed**(motolii-shell-iced 100 本を含む。既知の profile 依存
  `ui_numeric_trace` fast-profile 問題はこの通常 profile 実行では出ない)
- **M-5 切替**: UX 台本 P1〜P5 が iced shell で通り、replay oracle・フェンス同等物が
  green になったら既定 bin を切替。egui shell は当面 `--legacy` で残し、
  勝負が付いたら撤去

各 M は red 先行+検収+gate の通常レーン運転。M 間は直列、M 内は並列可。

## 2026-08-19 authority cutover

利用者判断により、製品開発の現行hostと新規機能targetをicedへ切り替えた。egui shellはlegacy/referenceであり、Timelineの参照実装、Rerun Stage島の内部詳細、比較・回帰器具として残す。上記M-5の既定bin名やlauncher切替に残余があっても、それはhost authorityをeguiへ戻さない。

この切替は製品完成宣言ではない。個別能力、視覚忠実度、実機、performanceの残余とhost authorityの現在値は[CANON](../CANON.md)と能力台帳で追跡する。後続で全面退役したagent入口縮約は[歴史記録](../archive/agent-governance/2026-08-19-agent-entry-reset-and-iced-authority-cutover.md)として保存する。

## 既存決定との関係

- [2026-08-16 Timeline 実行時基盤=egui](2026-08-16-timeline-runtime-reselection-to-egui.md):
  **ホストについて supersede**(egui は Stage 島内へ)。当時の実測事実(DOM 天井・
  行モデル・P8)は引き続き有効で、iced canvas 実装の設計材料
- [運転席決定](2026-08-18-cli-gui-driver-seat.md)・[ログと構造の強制](2026-08-18-log-and-structure-enforcement.md):
  **意味は不変のまま iced へ移る**(transcript/intent/replay は host 非依存の契約)
- [Rerun 合成基盤裁定](2026-08-18-rerun-as-composition-foundation.md): 不変(島として続行)
- toolkit 再入場トリガー: 発火したのは「繋がっていない」でなく **DX 実測**だった —
  帳簿は「慢性コスト枠が決定打になった」と読む
