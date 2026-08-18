# セッション引き継ぎ — iced 4面分業 campaign(第3区切り)

日付: 2026-08-18(夜)
状態: **引き継ぎ**(観察。決定を含まない)

## セッションID

- 本セッション: `6447b487-d69e-49ad-8d13-0f6b144ccdc8`(単一・クラッシュなし。
  worktree `session-handoff-notes-516742` / branch `claude/ux-driver-iced-m1-landing-3de3e7`
  で開始したが、**このworktreeは古いbase `1e630dac` のまま** — 作業は全て
  本チェックアウト `/Users/member_ottoto/rust_ae/Motolii` の main 上で直接行った)
- 前区切り: `96b6d47f-7306-44c9-8853-b0c50ae4cd40`
  ([前引き継ぎ](2026-08-18-session-handoff-ux-driver-seat-and-iced-migration.md))

## 現在地(機械的事実)

- **main tip = `172dc76f`**。remote未push(方針不変)
- 本セッションで main に入った列(順):
  1. `1c76140e` まで **ff merge** — 前区切りの統合branch `claude/ux-cli-gui-integration-002b03`
     の57 commit(iced M-1まで)。前区切り終了時点で main 未反映だったのを一本化した
  2. `9a16c363` 話し合い記録merge — [Rerun埋め込み前例調査](2026-08-18-rerun-embedding-precedent-survey.md)・
     [iced実績外部調査](2026-08-18-iced-track-record-survey.md)・
     [Stage対話の概念地図](2026-08-18-stage-interaction-concept-map.md)・「先例ゼロ」行の再仕分け注記
  3. `0d794c7c` **発掘vism実装のmerge** — mainの作業ツリーに未commitで残っていた
     List(ElementType)実装(mtime 08-17 18時台)を `codex/vism-param-list-impl-rescue-20260817`
     (`334db56c`)へ退避→検証レーン(816 green・決定文書と完全一致)→merge
  4. `1465b0e4` **fix-forward** — vism mergeが壊した inspector の List match 5箇所
     (post-vism gate が検出。List=AssetRef同格の不支持、という parameter_control 慣行で修復)
  5. `93e9ee88` 引き継ぎ整合(iced殻実窓を利用者が目視した件)
  6. `288f2ba9` **M-4t theme merge** / `46282d50` **M-2 Stage島 merge**(conflict 1:
     view.rs=M-1プレースホルダ退役で解消)/ `172dc76f` **M-4w widgets merge**

## レーンの状態(6+2)

発注の型は全て capsule+red先行。**返却済み6本は全て検収合格**。各レーンの詳細返却は
各 worktree と commit message が正本。

| レーン | branch(worktree=/private/tmp/motolii-*) | 状態 |
|---|---|---|
| M-4t theme | `claude/m4t-theme-20260818` | **merge済み**。token正本(DTCG `motolii-dark.json`→生成CSS)の写し21 role・手書きhexゼロ・snapshot golden(iced_testに`Simulator::snapshot`実在)。Lightは正本欠如により不発明 |
| M-2 Stage島 | `claude/m2-stage-island-20260818` | **merge済み**。bind groups床の実効実測 2→4(fork台帳§4受入条件を閉塞)・調停3状態(`grab_probe`=ギズモの席)・pixel証拠 `docs/reviews/evidence/iced-m2-stage-island/`・egui非依存(`EmbeddedSpatialStage`経由)・新seam不要 |
| M-4w widgets | `claude/m4w-widgets-20260818` | **merge済み**。scrub_value/key_button/context_menu/drop_zone、18/18、契約逸脱なし、`widgets/palette.rs`は仮(→`theme::Tokens`対応表がtheme/mod.rs docsに) |
| M-4b Inspector | `claude/m4b-inspector-20260818` | **merge済み**(統合第1弾)。4 section(Audioは口が無いため不出=Q0)・逸脱受理: intent背骨へwave E変種追加(select/flags/param edit/key/fx。`project_mut`がpub(crate)で公開経路が無かったため=迂回よりwrapper) |
| M-4a Browser | `claude/m4a-browser-20260818` | **merge済み**(統合第1弾)。rail 3種のみ・double-click=`AdmitPaths`(OSドロップと同一レール)・motolii-ui可視性のみ変更(M-1前例)・選択intentの空白は統合第2弾で `SelectLayer` union へ解消(下記) |
| M-3 Timeline | `claude/m3-timeline-20260818` | **merge済み**(統合第2弾)。drive 14+unit 7・intent列replayでclip実位置まで一致・intent背骨へtimeline 9変種+`editor_mut(`禁止フェンス・release-only commit(egui のlive-commit不採用を記録)・zoom/panはintent外(Message列replayで対) |
| 統合第1弾 | `claude/m4-integration-20260818` → main `5c7a05c3` | **merge済み**。M-4b+M-4a merge・3面pane合成(左Browser/中Stage/右Inspector)・stub→本物widgets・theme統一 |
| **統合第2弾** | `claude/m4-integration2-20260818`(base `5c7a05c3`) | **merge済み**。M-3 merge(intent.rsのunion解消: `SelectLayer{layer:u64,additive:bool}`1本化)+4面合成(上段Browser/Stage/Inspector・下段Timeline、比率1:1=egui shell中央列の実測)+選択結線(Timeline→`UiIntent::SelectLayer`→Inspector、replay台本つき)+fence修理(GPU owner台帳・SCANNED走査表)+**full workspace gate**(下記) |

## 検証の実態(バイアス抜きの核)

**機械検証済み**: 全レーンred先行→green(red logは各返却/evidenceに現物)。merge毎の
targeted検証は `cargo test -p motolii-shell-iced`(`172dc76f` 時点 **69 passed / 0 failed**)。
replay oracle(intent列・Message列)・pixel oracle(M-2)・snapshot golden(theme)・
fence群(gateway/dep-policy/editor_mut)全green。

**full workspace gate は `1c76140e` 以降未通過**: post-vism gate(`0d794c7c`)は
motolii-ui 5エラーで**失敗**→ `1465b0e4` で修復し `-p motolii-ui` green を確認したが、
**workspace全体のgateはそれ以降回していない**(6レーン並走でcargo枠を譲った)。
統合第2弾の受入条件に full gate を含めてある。**統合第2弾で通過**:
`cargo test --workspace --no-fail-fast -j 5` が237 test binary全部green、
**2131 passed / 0 failed**([移行地図](2026-08-18-iced-host-migration-decision.md)
のM-4統合節に詳細)。

**人間検証済み(本セッション・2回)**: ①利用者がiced殻(M-1)の実窓スタート画面を目視
(驚いて閉じた=閉じるボタン実動)。②egui shell の fixture screenshot と実窓比較で
「見た目の移植」問題を特定 → theme レーン発注の根拠。

**人間未検証**: 6レーン全ての実窓挙動(全部headless検収のみ)。themed画面はgolden PNG
でしか見ていない。Stage島の実マウスdrag・scrubの手触り・widgetsのhover段階・
実mp3波形・実Finderドロップ・トラックパッド係数(40px=1ノッチ)。
**統合完了後に利用者の「触る20分」を最初に置く**(台本はP1-P5短縮+新部品の触りどころ)。

## 要裁定(継続セッションが拾う)

1. **選択のreplay可否**: **統合第2弾で決着**。Stage/Timeline→Inspectorのlayer選択は
   `UiIntent::SelectLayer{layer:u64,additive:bool}`1本(journal・replay可能)に統一した
   (M-4bのIntentとM-3のIntentが同名別形で共存できなかったのをunionで解消)。
   Browserのcard選択は引き続きpane-local(Document外=正典準拠)のまま繋がない
2. audio gain の editor 操作API(commandはdoc層に有り、口が無い)→ Audio section 復活の前提
3. egui側フェンス禁止リストへM-3の editor 入口4本を足すか(M-3返却§5)
4. Recent railの意味(=admit順。閲覧順ではない)が台帳の読みと合うか

## 運転規約(本セッションの追加教訓)

- **レーン発注は必ず `model: "sonnet"` を明示**(利用者指示 2026-08-18。既定継承だと
  Fable枠を食う。メモリ `fable-for-design-work` に追記済み)
- 6レーン全開時はcargo同時2本規則を実質超過した(利用者の全開指示による意図的逸脱)。
  マシンは耐えたが、compile渋滞で各レーンの体感が伸びる — 平時は2本に戻す
- worktreeの古いbase事故が今回も2レーンで発生(M-2/M-3がbase `0d794c7c` の
  コンパイル不能を踏み、mainの修正commitをcherry-pick処置)— capsuleへのbase hash明記は
  必須のまま、**発注時点のmain tipが自己完結で建つことも確認してから切る**こと
- merge conflictが「pane合成」のような実装作業に化けたら、supervisorが手で解かず
  統合レーンとして発注する(本セッションでM-4b merge中断→統合レーン化)
- subagentの「Waiting for …」通知は中間停止で、背景処理完了時に自走再開する。
  再spawnしない(前区切りの500エラー教訓と別物)

## 状態の正本

- 本引き継ぎ+[移行裁定](2026-08-18-iced-host-migration-decision.md)+各レーンbranchの
  commit message+`docs/reviews/evidence/iced-m2-stage-island/`+メモリ
  (`normal-editor-campaign-playbook` / `structure-over-supervision` / `fable-for-design-work`追記)
- 残作業の順: 統合第1弾検収→統合第2弾(M-3+full gate)→利用者の触る20分→
  M-5切替判定(台本P1-P5+replay green→既定bin切替・egui shellは`--legacy`)。
  その先の残キューは[前引き継ぎ](2026-08-18-session-handoff-ux-driver-seat-and-iced-migration.md)
  「既知の欠陥・残タスク」節が引き続き正本

## 追記 — 利用者の実機裁定(2026-08-18 深夜)と修復レーン

4面着地・full gate 2131/0 の直後、利用者が `cargo run -p motolii-shell-iced` を実走。
**2件の不合格**(実機裁定は最上位の証拠)。どちらもレーンの失敗ではなく**発注書の穴**である。

1. **Stage に何も映らない** — `stage_island.rs::frame()` が
   `present_probe_frame()`(試験用の絵)を流しており、評価済み Document フレームを
   流していない。M-2 が「M-3 が置き換える」とコメントして席を空けたが、
   M-3 の capsule は Timeline しか書いておらず、**2つの capsule の隙間に落ちた**。
   島自体は生きている(空シーンの背景が描画されている)
2. **見た目が egui / html / Skia を全く再現していない** — M-4 系のどの capsule にも
   「視覚言語の再現」が受入条件として無かった(theme レーンは色 token を M-1 画面へ
   当てただけ)。実物は素のシステムフォント見出し+平文で、section 構造・帯色・
   行密度・chrome が無い

**修復レーン(sonnet・走行中、base `8b7cd640`)**:
- `claude/stage-frame-seat-20260818` — `stage_frame_seat`/`render_worker` を iced 側へ移植し
  probe を置き換え。playhead 追従・revision 監視・失敗報告・既知色 fixture の pixel oracle
- `claude/visual-fidelity-20260818` — 4面を css 正本(`docs/mocks-ui/`)から導出し直す。
  タイポ階層・Inspector の section/property 行・Browser の card・Timeline の ruler と bar・
  窓の既定サイズ。**`--screenshot` を iced shell へ追加**(以後の視覚検収の常設器具)し、
  egui の絵と並べて `docs/reviews/evidence/iced-visual-fidelity/` へ

**教訓(メモリ `capsule-gaps-are-the-defect-source` に記録)**: 型・oracle・fence は
「書いた物が正しいか」しか守らない。green 100% は発注書の完全性を何も保証しない。
UI 発注には視覚の受入条件と**検証器具**(撮って自分の目で見る)を必ず入れること。

### 視覚の正本と撮影器具(2026-08-18 深夜に確定)

利用者の指摘「それらは旧版、最新は散らかっている」を受けて mtime / git log で走査した結果:

- **視覚の正本 = `docs/mocks-ui/public/{inspector,browser,timeline}-library.html` + 同名 `.css`**
  (最終コミット 2026-08-16。リポ内の設計資産で最新)。egui の
  `inspector_panel/theme.rs` は冒頭で「**全部 `inspector-library.css` の写し**」と
  宣言しており、**egui 画面は写しにすぎない**。再現は写しでなく元を見る
- `docs/mocks/m3-main-ui-v1*`(`ui-visual-language.md` が基準と呼ぶもの)は旧版。
  密度の基準としては有効だが、面の詳細はライブラリ側が新しい
- 素の html はスタイルが当たらない(`<link>` が2本: token css
  `src/tokens/mock-candidates.css` と library css)。**両方を `<style>` へ流し込んだ
  `-inlined.html` を作ってから撮る**

**撮影器具**(Playwright。`docs/mocks-ui/` の node_modules を使うためその場所で実行):
scratchpad の `capture-design-reference.mjs`。出力した基準画像:
`/tmp/motolii-design-reference/{inspector,browser,timeline}-reference.png`。
**この3枚を Read で開いて目視するのが視覚検収の唯一の合否判定**であり、
visual-fidelity レーンへは正本の訂正とともに送付済み。
