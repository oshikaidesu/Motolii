# セッション引き継ぎ — Timeline 移植・器具・台帳(第4区切り)

日付: 2026-08-19
状態: **引き継ぎ**(観察。決定を含まない)

## セッションID

- 本セッション: `6447b487-d69e-49ad-8d13-0f6b144ccdc8`(第3区切りと同一セッションの続き。
  途中で利用者がモデルを Opus 5 へ切替。**区切り後に Fable へバトンタッチ予定**)
- 前区切り: [第3区切り引き継ぎ](2026-08-18-session-handoff-iced-four-pane-campaign.md)

## 現在地(機械的事実)

**main tip = `3dc664b8`**。remote 未 push(方針不変)。

本区切りで main に入った列:

| commit | 中身 |
|---|---|
| `2ea6ae5e` | 近道キー: 表を `shortcuts.rs` に一本化、Cmd+A 追加、**効くキーだけ**を legend に |
| `3091eefe` | [普通のタイムライン先例調査](2026-08-19-normal-timeline-prior-art.md)(7製品44操作) |
| `d24acfe4` | **[配置土台の裁定](2026-08-19-timeline-packing-model-decision.md)**(利用者): 自由配置で確定 |
| `78d31569` | [egui Timeline 能力台帳](2026-08-19-egui-timeline-capability-ledger.md)(48能力) |
| `e06b7995` | リポ整理: `docs/CANON.md` 新設、旧モック10点を `docs/archive/` へ、存在しないパス参照を訂正 |
| `ac7e5ace` | **CSS 計算値抽出器具** `motolii-css-metrics`(Blitz で HTML/CSS → 計算済み値 → oracle) |
| `2aba6d99` | CANON: **面ごとに手本が違う**ことを明記(下記) |
| `055d621b` | Timeline キー編集移植(菱形・選択/移動/削除・補間メニュー) |
| `d56c3296` | **再生機構**(Space / L / ▶ / ⏮、audio seat 共用) |
| `3dc664b8` | Inspector 改定(**HTML から意図を解析**、X/Y 同一行、oracle 両方向化) |

## 利用者裁定(本区切り)

1. **Timeline の配置土台 = AE 型の自由な絶対時間配置**。gapless packing 前提の
   trim family(ripple/roll/slip/slide/insert/overwrite/lift/extract/sync lock)は
   **設計上の除外**であって漏れではない。「以降を押し出す」は便利機能として先送り
2. **Timeline は egui 実装が正本**(「egui 版が最も機能を詰めれていて優れている、UI も」)。
   `timeline-library.html` は副参照へ降格
3. **Browser / Inspector は逆** — 正本は HTML/CSS **そのもの**で、
   **egui 実装は手本にしない**(「egui 変換が上手くできなかった部分」)。
   iced 側は HTML から**意図**(section 階層・class の意味・行の内部構造・状態の表現)を
   解析して作る。egui から拾ってよいのは**振る舞いの結線と意味関数**だけ
4. **Blitz は器具として使う**(製品に組み込まない)。`iced_webview_v2` は不採用・観測点(メモリ `iced-webview-v2-observation`)
5. **レーン発注は `model: "sonnet"` 明示**(Fable 枠の保護。第3区切りからの継続)

この 2 と 3 の**非対称**は `docs/CANON.md` に恒久記録済み。取り違えると事故になる。

## 走行中(**継続セッションが最初に検収する**)

| レーン | branch | 状態 |
|---|---|---|
| Timeline 構造操作移植 | `claude/tl-structure-20260819` | 畳み開閉 / Group 子帯 / rename / lock / Cmd+G。**台帳が見つけた「ロック中 clip が動いて見える嘘」の修復も追加指示済み**。最終検証中 |
| Browser 改定 | `claude/browser-revise-20260819` | HTML から意図を解析。`view.rs` から `browser_pane.rs` へ切り出し、検索/フィルタを**機能ごと**移植 |

## 検証の実態

- **機械検証**: 各レーン red 先行 → green。merge ごとに `cargo test -p motolii-shell-iced` を実行
  (`3dc664b8` 時点の集計は継続セッションが確認すること — 実行はしたが本文書作成時点で未回収)
- **full workspace gate は `abf59aa0` 以降未実行**。次の区切りで回すこと
- **人間検証済み**: 利用者が実窓を2回確認(スタート画面 / 4面+Stage の空表示)。
  **その後の全変更(キー編集・再生・Inspector 改定)は人間未検証**
- **再生の音・実時間追従は誰も聞いていない**(headless テストのみ。手動確認事項として各 evidence に明記)

## 器具(本区切りの最大の資産)

- `motolii-css-metrics`: `cargo run -p motolii-ui --bin motolii-css-metrics -- all out/`。
  HTML/CSS の**計算済み値**(box/padding/border/色/gap)を JSON で吐く。
  `motolii_ui::css_metrics::extract()` として公開、oracle テストが直接呼ぶ。
  **罠**: `<link>` は解決されない(inline 要)/ 帯・アクセントバーは `::before`/`::after` =
  `AnonymousBlock`(Element だけ歩くと消える)/ JS 依存の初期状態は再現されない
- `--screenshot <out> <frames>`: iced shell の撮影。**frames は 120 以上**
  (25 では非同期の合成が間に合わず「Stage が空」と誤診する)
- 基準画像: `/tmp/motolii-design-reference/{inspector,browser,timeline}-reference.png`、
  同一 document の egui 版 `/tmp/egui-same-doc.png`。撮影器具は scratchpad の
  `capture-design-reference.mjs`(`docs/mocks-ui/` で実行)

## 残作業(正本)

**Timeline 移植**: [能力台帳](2026-08-19-egui-timeline-capability-ledger.md)が正本。
48 能力中 **無 28 / 部分 4**(キー編集・再生の着地でこの数は減っているはずだが**再計測していない**)。
台帳が挙げた危険な見落とし上位:
1. ロック中 clip の嘘(構造レーンが修復中)
2. **拒否理由が iced のどこにも表示されない**(`take_rejections` の呼び出しが crate 全体で 0 件、
   `dispatch` の戻り値が `let _ =` で握り潰し)。2026-08-18 診断 F-07〜F-10 と同じ型の再発。
   **未着手 — 次の波で単独レーンにする**
3. fold 状態が常に既定(全閉じ)— 構造レーンが対応中
4. snap 候補にキーと loop 端が無い

**その他**: `Cmd+K`(分割)/ `M`(マーカー)は **D2 の口はあるが UiIntent の口が無い**。
`UiEditParam` に `Anchor` が無い(キー編集レーンが報告)。
Inspector の Q0 残差(Effect/Custom タブ・Fill 色編集・FX stack toolbar 等、
HTML にあるが intent が無い物)は round3 evidence の README に列挙済み。

## 運転規約(本区切りの追加)

- **worktree レーンは `CARGO_TARGET_DIR=/private/tmp/motolii-lane-target` を設定する**
  (AGENTS.md に規約化)。設定しないと各 worktree が依存ツリーを個別にビルドし、
  実測で**本体 target 179GB / 各レーン 33〜47GB**まで膨れ、ディスク飽和で `ls` すら返らなくなった。
  **supervisor は本チェックアウトの target を共有しない**(レーンの WIP バイナリが
  `target/debug/motolii-shell-iced` を上書きすると screenshot 検証が別物を撮る)
- 走行中レーンに後から共有を強制しない(温まった target を捨てることになる)
- 本区切りで完了レーンの target を一括削除済み(走行中2本と本体は残置)
- **並列上限の実感**: 5〜7レーン同時は cargo lock とディスクで詰まる。
  merge を捌けるのは supervisor 1 人なので、**返却の消化が律速**。
  レーンを増やすより返却を捌く方が速い局面がある

## 状態の正本

- 本引き継ぎ + `docs/CANON.md`(正本索引)+ [能力台帳](2026-08-19-egui-timeline-capability-ledger.md)
  (残作業)+ [配置土台裁定](2026-08-19-timeline-packing-model-decision.md) + decision-index の 2026-08-19 行群
- メモリ: `normal-editor-campaign-playbook` / `capsule-gaps-are-the-defect-source` /
  `structure-over-supervision` / `iced-webview-v2-observation` / `fable-for-design-work`(sonnet 明示)

## 追記 — 全レーン着地(2026-08-19 12:50)

**main tip = `d1fe8ea1`。`cargo test -p motolii-shell-iced` = 172 passed / 0 failed。**

引き継ぎ本文で「走行中」としていた2本は着地した:

- `e5928e8f` **Browser 改定** — `browser_pane.rs` へ切り出し、約50定数を `browser-library.css:行` つきで写し器具と照合。検索 / フィルタ chip / 表示モードを **egui の意味関数だけ**移植。**絵を見て2つのバグ発見**: `iced::Border` は4辺すべてを塗る(単辺のつもりが箱になる)/ `Shrink` の中の `Fill` サムネイルが高さ0に潰れる(今朝の「帯が高さ0.0」と同じ family)
- `d1fe8ea1` **構造操作** — **レーンのプロセスが死亡**(transcript 消失・再開不能)。worktree に 688行が未 commit で残っていたのを supervisor が commit(`ab4ff63e`)→ テスト green 確認 → merge。畳み開閉 / Group 子帯 / rename / lock / 構造系近道キー / **ロック中 clip の嘘の修復**

### merge で解いた**意味の衝突**(記録に値する)

キー編集レーンと構造操作レーンが `semantics.rs::hit_test` の Property 行で**逆の要求**を書いていた:
キー編集=「菱形を掴ませる」/ 構造操作=「Property 行では何も掴ませない」。
**菱形が掴めないとキー編集が機能しない**ので、Property 行はキー判定を先に返し、
構造側のガードは**それ以外の非 Object 行**に適用する形で解決した。

### 器具が仕事をした実例

構造レーンが `RAIL_W` を 210→234 に変更(L ボタン追加で名前欄が詰まるため)したのを、
`css_metrics_oracle::timeline_known_divergences_are_pinned` が**落として検出**した。
コード側に理由が書かれていたので正当と判断し、**根拠ごと固定値を更新**した。
「定数が黙って変わる」を器具が防いだ最初の実例。

### 掃除

完了レーンの target を一括削除し、**空き 161GB → 655GB**(+494GB)。
残置は走行中だった2レーンの worktree と本体 target のみ。ソース・commit・evidence は無傷。

### この時点の UI

`/tmp/ui-final.png`(`--screenshot ... 130`)。4面 + Stage に合成結果 + transport 帯 +
行の色丸/M/S/L + 効く近道キーだけの legend。**Inspector の Opacity 行が下端で切れている**
(スクロール要)、Browser のカードが縦に詰まる、等は残差として残っている。
