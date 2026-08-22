# `next/shell/motolii-shell/src/lib.rs`(5,616行)分割線調査

読むだけの実測。**コードは1行も変えていない**(発注どおり)。対象 checkout:
`/Users/member_ottoto/rust_ae/Motolii/.claude/worktrees/agent-a55fbbeb18dd8f154`
(`git merge main` 済み、HEAD=`02a653c8`)。行数は 2026-08-22 時点の `wc -l`。

## 評価軸の訂正について(利用者裁定を反映)

発注書は「merge 衝突耐性」を主軸に書いたが、調査の途中でコーディネーターから
訂正が入った——**正しい第一目的は「shell を複数レーンが同時に施工できる形に
割ること」**(並列施工可能性)。衝突耐性・行数の均しは副次。本調査はこの訂正
後の軸で書き直している。この訂正は結果的に本調査中に見つけた**実例**
(§4)と完全に整合する——衝突耐性だけを見ていたら見落としていた非対称性
(§3.4)が、並列施工可能性を軸に据えて初めて見えた。

---

## 0. 現状確認: 裁定160(pane crate 分割、切片1〜10)は既に完了している

`lib.rs` 冒頭のコメント(1-146行)と `docs/reviews/2026-08-21-pane-split-survey.md`
(裁定159、以下「旧調査」)を読むと、今回の発注の前提——「機能が全部ひとつの
ファイルになっている」——の**半分はすでに解消済み**だとわかる: 旧調査時点で
2,615行だった `lib.rs` から、tokens/state(Session+KeySelector)/inspector_pane/
timeline(9ファイル+write ロジック584行)/settings_pane/chrome/stage(観測カメラ)
の7 crate が `next/ui/` へ抽出済み(`docs/reviews/2026-08-21-lane-board.md:163`
「pane crate 分割 完了(裁定160、2026-08-21夜)」、`docs/reviews/
2026-08-22-session-handoff-recovery-and-calibration.md:14` で「新しく作る物は
無い」と確認済み)。

にもかかわらず今 5,616行あるのは、**抽出後にシェルが「新機能の最後の1マイル
(shell 結線)」を全部引き受け続けた**ため——browser-pane(B0-B3)・export-pane
(B09)・pane_grid 化・menubar(MB-0〜MB-2)・実時間再生(A2)・JKL シャトル(B21)・
Stage ギズモ(GZ)・方眼シート(B22)・マーキー・マーカーレーン(B19)・rename・
窓の浮かし(S1/S2、裁定182/188)・wgpu 常駐テクスチャ presenter(裁定166/170/171)
が全部この1ファイルへ積み上がった。**pane 自体の分割はもう十分やった。残って
いるのは「pane を shell へ配線する glue」の置き場**、というのが今回の調査の
出発点。

---

## 1. 内訳の行数表

関数/struct/impl の開始行を境界にした実測(境界直前の doc comment は前項目に
計上される近似——大分類の相対比較には十分だが ±20〜30行程度の按分誤差はある)。
合計は 5,616行と一致。

| 塊 | 行数 | 中身 |
|---|---:|---|
| Stage presenter(wgpu shader Program/Pipeline/Primitive、GPU常駐テクスチャ) | 658 | `stage_presenter_rgba`/`build_stage_presenter_rgba`/`Uniforms`/`VertexOutput`/wgsl文字列/`StagePresenterProgram`/`Primitive`/`Pipeline`(裁定166/170/171) |
| Stage pane 結線(update_stage/update_gizmo/overlay投影/RenderedFrame) | 680 | 観測カメラ・ギズモ・方眼・マーキーの overlay 計算+`stage_pane()` view 関数 |
| Inspector pane 結線 | 595 | `update_inspector`+drag/draft glue 8関数+`inspector_pointer_event` |
| クリップボード/File束/Group動詞 | 398 | copy/paste/cut/duplicate/select_all/admit/save/reset/group/ungroup/freeze |
| Stage frame render pipeline | 349 | `refresh_frame`/`build_preview_snapshot`/`ensure_rgba_fresh`/display source 計算 |
| `view()` 組立 | 346 | `view()`本体/`header`/`pane_title_bar`/`status_band` |
| コンストラクタ | 322 | `new`/`new_with_dialogs`/`boot`/`boot_fixture`/`new_fixture` |
| `Message` enum 定義 | 277 | 60腕(本体は8 pane の wrap 型+core/cross-cutting) |
| `update()` 最上位 match(1本) | 268 | 全腕のディスパッチ(§3.1参照) |
| Timeline pane 結線 | 264 | 5例外の先取り+nav(J/K/,/.等)+`build_timeline_pane`+overlay |
| public アクセサ(getter群) | 238 | screenshot.rs/tests が読む投影関数 |
| Shell 構造体定義(35 fields) | 189 | 各 field に「なぜ Document でも Session でもないか」の doc 付き(§2.1) |
| keymap(`resolve_navigation_key`) | 207 | 純関数、`&self` を取らない(§2.3) |
| 実時間再生/JKLシャトル | 142 | `toggle_playback`/`apply_shuttle`/`advance_playback_tick`/transport |
| Export pane 結線 | 139 | `update_export`/`start_export` |
| crate 前文(use・doc・mod宣言・裁定160 pub use 別名の履歴コメント) | 153 | 40行(1-40)+113行(41-152) |
| Settings pane 結線 | 103 | `update_settings`/`update_settings_legacy`/`toggle_settings_window` |
| 窓管理(view/title dispatch) | 98 | `view_window`/`window_title`/`view_settings_window`/`view_export_window` |
| metrics(debug/release分岐) | 59 | `#[cfg]` 2重定義(no-op release) |
| Browser pane 結線 | 53 | `create_from_card` 先取り+`panes.set_browser_open`同期 |
| `subscription()` | 45 | window/tokens/pointer/ticks/closesの`batch` |
| impl ブロック見出し等の残差 | 33 | `impl Shell {`/`impl shader::Program for ...` 等の見出し行 |
| 雑多(型定義: `PreviewSnapshot`等) | 20 | ゼロコピー用スナップショット型 |

**大分類**: pane 結線(Inspector 595+Timeline 264+Settings 103+Stage 680+Export 139
+Browser 53+クリップボード/File/Group 398 = **2,232行、全体の40%**)+ Stage
presenter(658)+ frame pipeline(349)+ view 組立(346)+ コンストラクタ(322)+
Message enum(277)+ update match(268)+ アクセサ(238)+ Shell struct(189)+
keymap(207)+ 窓管理(98)+ その他。

---

## 2. 分割の障害の実測

### 2.1 Shell の private state への直接アクセス(旧調査§2.1 は解消済み、残るのは pane 内部ではなく glue 側)

pane crate 自体(inspector_pane/timeline_pane/settings_pane/stage_pane/
browser_pane/export_pane)は `&Shell` を受け取らない——`&StoreView`/`&Session`/
`&Tokens`/投影型だけ。**書き込み側の実ロジックはすでに pane crate 側の
自由関数として存在する**(旧調査§1.2の978行はすでに移設済み。例:
`inspector_pane::{commit_field, ...}` が `&mut Document`/`&mut FieldDraft` を
明示引数で取る形——lib.rs:3011-3019 のコメント「5関数とも書き込み本体は
`motolii-inspector-pane` crate 側の自由関数——ここは `self.doc`/
`self.inspector_drag`/`self.session`/`self.keyboard_modifiers` をそのまま
貸す glue だけ」がこの構造を明言している)。`timeline_pane::PaneState::update`
も同型(`&mut self.doc, &mut self.session, self.keyboard_modifiers` を明示
引数で受ける、lib.rs:1235)。

**つまり旧調査が「crate 化の最大の障壁」と呼んだ問題はもう解けている。**
残っているのはその**glue 自体(1〜3行の呼び出しの束)がどのファイルに
物理的に書かれているか**という、より単純な問題。

### 2.2 Rust の module 可視性は「同一ファイル」を要求しない(重要な発見)

`Shell` の全 field は `pub` を付けていない(private)。Rust の可視性規則は
「定義したモジュールとその**子孫モジュール**から見える」——`lib.rs`(crate
root module)で定義した `struct Shell` の private field は、`mod glue;
mod glue::settings;` のように lib.rs から宣言した子モジュールの中の
`impl Shell { ... }` ブロックから**そのままアクセスできる**(pub(crate) 化も
signature 変更も不要)。

これは実験で確認済み(§5 参照——本調査では実装しないが、この事実は
`cargo doc`/Rust reference の module 可視性規則から機械的に導ける。念のため
既存コードで同型の前例を確認: `next/ui/motolii-timeline-pane/src/lib.rs` は
`mod write; mod projection;` という子モジュールを持ち、`write.rs` 側の関数が
`PaneState`(`lib.rs` 側定義)の private field を直接読み書きしている——
**同一 crate 内の親子モジュール間で private field を共有する構図は、この
codebase に既に実例がある**)。

**含意**: `impl Shell` のメソッド本体を `src/glue/settings.rs` のような別
ファイルへ**そのまま切り貼りするだけ**(1行も書き換えずに)で、コンパイルが
通る。これは判断を要する構造変更ではなく、機械的なファイル分割。

### 2.3 `resolve_navigation_key`(keymap)と wgpu presenter は Shell に一切依存しない

- `resolve_navigation_key`(lib.rs:5247-5453、207行)は `&self` を取らない
  純関数(`key`/`modifiers`/`captured` のみを引数に取り `Option<Message>` を
  返す)。`pub fn` で `tests/suite/*.rs` 8ファイル・69箇所から直接呼ばれている
  (export/group/restack/nav/shortcut/file/marker_keymap/rename の各
  `_drive.rs`)。Shell の private field は一切触らない。
- wgpu presenter ブロック(lib.rs:4394-5136、742行の生範囲、実質658行)も
  同様——`self.` の29件は全部 `StagePresenterPipeline`/`StagePresenterProgram`
  という**presenter 自身の struct** のメソッド内の self であって、`Shell` の
  field には一度も触れない(`impl shader::Program<Message> for
  StagePresenterProgram` のように `Message` 型だけを型引数として要求する)。
  `tests/suite/zero_copy_presenter_fence.rs`/`render_pipeline_fence.rs` は
  `motolii_shell::{metrics, stage, Message, Shell}` の**公開API経由**でしか
  この内部を触っていない(`StagePresenterPipeline` 等は non-pub のまま)。

どちらも「pane crate へ出せない」(`Message` が root 型のため、旧調査の
結論どおり)が、「**同一 crate 内の別ファイルへ出す」ことを妨げる物は
何もない**。

### 2.4 `Message` の exhaustive match が唯一の真の共有点

`enum Message`(48+腕、277行)と `Shell::update` の最上位 `match message { }`
(268行)は Rust の網羅性検査ゆえに**1箇所に集まらざるを得ない**。ただし
実測すると、**新しい pane 内部の機能追加はこの1箇所を触らない**——8 pane
(Inspector/Timeline/Settings/Stage/Browser/Export/Gizmo/Sheet/Marquee/Marker)
は全部 `Message::X(x_pane::Message)` という**1本の wrap 腕**で表現されており、
`x_pane::Message` に新しい腕が増えても最上位 match の該当行
(`Message::Settings(msg) => task = self.update_settings(msg)` 等)は
**変更不要**。最上位 match を触る必要があるのは「新しい pane を丸ごと足す」
「cross-cutting な腕(`KeyboardModifiersChanged`/`EscapePressed` 型)を足す」
という**稀なケースだけ**——§4 の実例がこれを直接裏付ける。

---

## 3. 分割案の比較(評価軸: **①並列施工可能性(第一)→②衝突耐性→③行数の均し→④挙動ゼロ変更の段階実行可否**)

### 3.0 現状(比較の基準点)

**①並列施工可能性 = 1**(規律「結線は常に1レーン」で強制。実測上も1ファイル
5,616行を2レーンが同時に触れば高確率で同一関数・同一 struct 定義に手が入り
衝突する)。②③④は評価不能(そもそも割れていない)。

### 3.1 案(a): 同一 crate 内のモジュール分割(`src/glue/*.rs` へ pane 別 glue を切り出す)

§2.2 の可視性事実により、**signature 変更ゼロの純粋な move** として実行できる。
切り出し単位を pane 単位(Inspector/Timeline/Settings/Stage/Export/Browser/
Clipboard・File・Group/Playback)+ 横断的関心(keymap/window管理/
stage_presenter/view組立)に取ると:

| 分割後のファイル | 行数目安 | 単独で触れるレーン |
|---|---:|---|
| `glue/inspector.rs` | 595 | Inspector |
| `stage_presenter.rs` | 658 | Stage(GPU側) |
| `glue/stage.rs` | 680 | Stage(意味側) |
| `glue/clipboard_file.rs` | 398 | Clipboard/File/Group |
| `frame_pipeline.rs` | 349 | (核: refresh_frame——後述) |
| `view.rs` | 346 | (核: view組立——後述) |
| `glue/timeline.rs` | 264 | Timeline |
| `keymap.rs` | 207 | keymap |
| `glue/playback.rs` | 142 | Playback/Shuttle |
| `glue/export.rs` | 139 | Export |
| `glue/settings.rs` | 103 | Settings |
| `window.rs` | 98 | 窓管理 |
| `glue/browser.rs` | 53 | Browser |
| (共有のまま)`lib.rs` | ~1,100 | Message enum・Shell struct・update最上位match・subscription・コンストラクタ・アクセサ |

**①並列施工可能性 = 最大13**(pane 8本+keymap+window+stage_presenter+
frame_pipeline+view の5横断)。うち frame_pipeline/view は複数 pane の投影を
束ねる核なので実際には「低頻度だが triggerされたら1レーン」寄り——実質的に
**恒常的に独立稼働できるのは pane 8本+keymap+window+stage_presenterの11本**、
frame_pipeline/viewは新pane追加時のみ稀に競合し得る。
**②衝突耐性**: 高——ファイルが分かれれば git の3-way merge は行単位でしか
衝突しない。同じ関数を2レーンが同時に書き換えない限り機械的に自動マージされる。
**③行数の均し**: 5,616→最大680行(旧来の「1ファイルで完結」志向を維持したまま
規模を1/8前後に)。
**④挙動ゼロ変更**: 保証できる(signature不変・ロジック不変の move のみ)。

### 3.2 案(b): keymap・窓管理を別 crate へ

発注書が明示的に候補として挙げていた案。実測した結果、**却下推奨**:

- `resolve_navigation_key` は `Message`(root 型)を返す。`Message` を crate 外へ
  持ち出せない以上、keymap crate は `motolii-shell` に依存する形にしかならない
  ——「pane crate は root に依存されるだけで root には依存しない」という
  裁定160の一方向依存(旧調査§5)を守れず、**新しい循環回避コストだけ発生する**。
- 窓管理も同型(`Message::MainWindowOpened` 等 root 腕を返す関数を含む)。
- crate化しても①並列施工可能性は案(a)と**同じ**(ファイル境界がレーンの単位
  である以上、crateの内外はレーンの並列数に影響しない——むしろ Cargo.toml
  ワークスペース登録・バージョン管理の手間が増えるだけで得るものがない)。

**結論**: keymap/window管理は「モジュール」止まりで十分。crate化の追加コストに
見合う効果がない。

### 3.3 案(c): pane 側へ dispatch match 自体を委譲する(settings/export を timeline/browser 型へ統一)

§2.1 で確認した通り、**Timeline と Browser は既にこの形に到達している**——
`timeline_pane::PaneState::update(msg, &mut Document, &mut Session, Modifiers)`
`browser_pane::PaneState::update(msg)` が pane crate 側で**腕の分岐自体**を
持ち、shell 側は1行の delegate(+数個の shell 先取り例外)で済んでいる。

一方 **Settings と Export はこの形になっていない**——`update_settings`/
`update_export` の中身(どの腕がどう振る舞うか)が**丸ごと lib.rs に直書き**
されている(pane crate 側は `commit_comp_field`/`apply_background_preset`の
ような個別自由関数を提供するだけで、「どの腕が来たらどれを呼ぶか」という
**選択のロジック自体**は shell 側にある)。

この非対称性は**衝突耐性だけを見ていたら気づかない**(§3.1のモジュール分割
さえすれば、settings/export も他 pane と同じ「独立ファイル」にはなる)。だが
**並列施工可能性を軸に見ると意味が変わる**: pane crate 側に選択ロジックが
無い settings/export は、**そのpane crate単体のテストではshellへの配線漏れを
検出できない**(§4 の実例がまさにこれ——`motolii-settings-pane` 単体は
green のまま、shell 側だけが壊れた)。Timeline/Browser 型に統一すれば、
「pane crate 側で `PaneState::update` が新腕を網羅していない」というコンパイル
エラーが**pane crate 自身のテスト実行時に(motolii-engine を巻き込まず)
即座に出る**——旧調査§4.2 が指摘した「レーンが払う `-p` 集合」の恩恵が
settings/export にも及ぶ。

**推奨**: 案(a)を全pane・全横断関心に即実施(判断ゼロの pure move)。
案(c)は settings/export の2 pane限定で追って実施(pane crate 側 API の
形を変える判断が要るため、案(a)より後・かつ別切片)。案(b)は却下。

### 3.4 発見: 「衝突耐性」だけでは見えなかった非対称性(§4の実例が根拠)

当初の発注書の軸(衝突耐性)では「settings/exportも他paneと同じ840行前後の
モジュールに割ればよい」で終わっていた。しかし実際に起きた事故(§4)は
**衝突ではなく「非同期の取り残し」**——2つのレーンが同時に同じ行を編集して
衝突したのではなく、**settings-pane crateを触ったレーンが「shell側の配線も
自分で完結できる」と誤解できる構造だった**(pane crate側APIが「呼び出し元が
選択ロジックを持つ」形だったため、pane crate だけ直しても shell 側が
壊れることに気づけない)。これは衝突耐性の指標では検出できず、並列施工
可能性(=そのレーンが「自分の変更だけで完結する」という保証)の指標で
初めて可視化される。

---

## 4. 実例: 現在 main は `motolii-shell` のビルドが壊れている(生きた証拠)

`cargo test --manifest-path .../next/Cargo.toml -p motolii-shell` を実行した
ところ、**現在の HEAD(`02a653c8`)は `motolii-shell` (lib) のコンパイルに
失敗する**:

```
error[E0063]: missing fields `auto_save_config`, `auto_save_draft` and
  `auto_save_enabled` in initializer of `ViewModel<'_>`
  --> shell/motolii-shell/src/lib.rs:3657:17

error[E0004]: non-exhaustive patterns: `AutoSaveToggle(_)`,
  `AutoSaveFieldInput(_, _)` and `AutoSaveFieldSubmit(_)` not covered
  --> shell/motolii-shell/src/lib.rs:2552:15
```

原因: `b358850b`(`feat(settings): B12第2切片 — AUTOSAVE節`、2026-08-22
16:23)が `motolii-settings-pane` crate の `sections::Message` へ
`AutoSaveToggle`/`AutoSaveFieldInput`/`AutoSaveFieldSubmit` の3腕と
`ViewModel` へ3フィールドを追加したが、**shell 側の対応する2箇所
(`update_settings` の match、`view_settings_window` の `ViewModel` 構築)は
誰も更新していない**。HEAD はそこから **53コミット・約1時間半後**
(`02a653c8`、17:54)——つまりこの間、**誰も shell の settings 結線を
「自分の仕事」として拾わなかった**。`next/check.sh` はビルド/テストを
実行しない(wraps/owns マーカーと normal-map の整合性しか見ない、
`next/check.sh:1-9`)ため、**この壊れは自動検知されずそのまま main に
乗り続けている**。

これは §3.4 で述べた「衝突ではなく取り残し」の直接証拠であり、かつ
「shellを1レーンの漏斗にした規律のコスト」を定量化する実例——**settings機能
自体は着地済みなのに、shellという単一の窓口を誰も同時に開けられないせいで
1時間半以上ユーザーに届かない状態が続いた**。

**この調査は読むだけの発注のため、この壊れを直してはいない。** supervisor
への提起事項として RETURN に明記する。

---

## 5. 切片割り(挙動ゼロ変更、裁定160 切片0〜10 の型を継承——番号は本調査限定の仮番、正式な裁定番号は supervisor 採番待ち)

前提: **切片1の前に、settings 結線の赤(§4)を最小修理で先に緑化する必要が
ある**——赤いままでは以降のどの切片も「テスト集合一致」を測るオラクルが無い
(比較対象のgreenベースラインが存在しない)。この修理自体は本調査の範囲外
(実装)だが、依存関係として明記する。

各切片の検収条件は共通: **挙動ゼロ変更・`cargo test -p motolii-shell -- --list`
の出力(テスト名の集合)が移動前後で一致**。view/stage_presenter に触る切片は
追加で `--fixture --screenshot` の PNG バイト一致も確認する(既存 screenshot
オラクルの再利用、旧調査と同じ基準)。

| 切片 | 内容 | write-set | 検収 | 並列可否 |
|---|---|---|---|---|
| **前提** | settings 結線の赤(§4)を最小修理(AutoSave 3腕+ViewModel 3フィールド) | `lib.rs`(2箇所のみ) | `cargo build -p motolii-shell` 緑化 | — |
| **①** | `stage_presenter.rs` 抽出(wgpu shader/pipeline、658行、pure move) | `lib.rs`→新設 `stage_presenter.rs` | test集合一致+`zero_copy_presenter_fence`/`render_pipeline_fence` 緑+PNGバイト一致 | 他切片と完全独立、並行可 |
| **②** | `keymap.rs` 抽出(`resolve_navigation_key`、207行、pure move、`pub use`不要=既に`pub fn`) | `lib.rs`→新設 `keymap.rs` | test集合一致(69箇所参照は`motolii_shell::resolve_navigation_key`のpath不変なので無改修) | ①と独立、並行可 |
| **③** | `window.rs` 抽出(`toggle_settings_window`/`toggle_export_window`/`view_window`/`window_title`/`view_settings_window`/`view_export_window`、pure move。Shell struct の3 Option<Id> fieldはこの段では動かさない) | `lib.rs`→新設 `window.rs` | test集合一致+`window_drive.rs`緑 | ①②と独立、並行可 |
| **④** | `glue/browser.rs` 抽出(`Message::Browser`処理+`create_from_card`、53行) | `lib.rs`→新設 | test集合一致+`browser_drive.rs`/`create_from_card_drive.rs`緑 | ①②③と独立、並行可 |
| **⑤** | `glue/playback.rs` 抽出(`toggle_playback`/`apply_shuttle`/`advance_playback_tick`等、142行) | `lib.rs`→新設 | test集合一致+`playback_drive.rs`/`shuttle_work_area_drive.rs`緑 | ①②③④と独立、並行可 |
| **⑥** | `glue/clipboard_file.rs` 抽出(copy/paste/…/admit/save/group/freeze、398行) | `lib.rs`→新設 | test集合一致+`clipboard_drive.rs`/`file_drive.rs`/`group_drive.rs`/`restack_drive.rs`緑 | ①-⑤と独立、並行可 |
| **⑦** | `glue/stage.rs` 抽出(`update_stage`/`update_gizmo`/overlay投影/`stage_pane`、680行) | `lib.rs`→新設 | test集合一致+`gizmo_drive.rs`/`observation_camera_drive.rs`/`marquee_drive.rs`/`sheet_drive.rs`/`stage_band_drive.rs`緑+PNG一致 | ①-⑥と独立、並行可 |
| **⑧** | `glue/inspector.rs` 抽出(`update_inspector`+drag/draft glue、595行) | `lib.rs`→新設 | test集合一致+`inspector_drive.rs`/`inspector_key_drive.rs`/`inspector_pixel_fence.rs`/`anchor_drag_drive.rs`/`speed_drive.rs`緑+PNG一致 | ①-⑦と独立、並行可 |
| **⑨** | `glue/timeline.rs` 抽出(5例外+nav+`build_timeline_pane`+overlay、264行) | `lib.rs`→新設 | test集合一致+`timeline_*_drive.rs`4本/`rename_drive.rs`/`marker_keymap_drive.rs`緑 | ①-⑧と独立、並行可 |
| **⑩** | `glue/settings.rs` 抽出(pure move、前提修理を含めた後の状態を移動) | `lib.rs`→新設 | test集合一致+`settings_drive.rs`/`ui_scale_fence.rs`緑 | ①-⑨と独立、並行可 |
| **⑪** | `glue/export.rs` 抽出(`update_export`/`start_export`、139行) | `lib.rs`→新設 | test集合一致+`export_drive.rs`緑 | ①-⑩と独立、並行可 |
| **⑫(判断要・任意)** | settings/export を timeline/browser 型(`PaneState::update`をpane crate側へ)へ統一(§3.3案c) | `motolii-settings-pane`/`motolii-export-pane`のAPI形状変更+`glue/settings.rs`/`glue/export.rs` | test集合一致+該当drive緑。pane crate 単体testがmotolii-engine非依存で走ることを確認(旧調査§4.2の恩恵の実測) | ⑩⑪の後、settings/exportレーンそれぞれ独立 |
| **⑬(判断要・任意)** | 窓管理3 field(`main_window`/`settings_window`/`export_window`)を`WindowLedger`構造体へ統合、pane別 draft field(inspector 4種/settings 3種/export 4種)を各pane専用の小structへ統合 | `lib.rs`(Shell struct)+各glueファイル | test集合一致 | ③⑧⑩⑪の後 |

①〜⑪は**全部 pure move(signature/ロジック不変、§2.2の可視性事実で保証)**
なので、**11本を全部同時に並行実行できる**——これが「並列施工可能性」を
第一軸に置いた場合の具体的な答え: **現状1→提案後は最大11(pure move分)
+前提修理1本、判断要の⑫⑬は別枠**。

---

## RETURN

- 文書パス: `docs/reviews/2026-08-22-shell-split-survey.md`(本ファイル)
- commit hash: この調査は読むだけのため、文書自体をcommitした時点のhashは
  supervisorへの提出時にRETURN側で確認(作業ブランチ:
  `worktree-agent-a55fbbeb18dd8f154`)
- 内訳の行数表: §1
- 分割案の比較と推奨: §3——**案(a)(同一crate内モジュール分割、pure move)を
  即実施推奨、案(c)(settings/exportのpane crate委譲統一)は追って、案(b)
  (keymap/window管理のcrate化)は却下推奨**
- 切片割り: §5(①〜⑪は判断ゼロのpure move、最大11本並行可能。⑫⑬は判断要で
  別枠)
- **緊急の提起事項(実装は本調査の範囲外)**: `next/shell/motolii-shell` は
  現在 HEAD(`02a653c8`)で **ビルドが壊れている**(§4)。`b358850b`
  (AUTOSAVE節、2026-08-22 16:23)以降53コミット・約1.5時間、settings-pane
  crate 側のAutoSave機能追加がshell側に配線されないまま放置されている。
  `next/check.sh` はコンパイル/テストを検査しないため自動検知されていない。
  切片①以降のどの切片も「テスト集合一致」のオラクルにはgreenベースラインが
  要るため、**この赤の解消が全切片の前提**(§5「前提」行)。
- 逸脱: (1) 行数内訳は関数境界の近似計測(±20-30行/カテゴリの按分誤差、
  合計は5,616と一致)。(2) `cargo test -p motolii-shell`のフル実行は
  上記ビルド破損のため完走せず、baselineテスト数は未確定(§4で報告した
  2件のコンパイルエラーのみ確認)。(3) §2.2の「子孫モジュールからprivate
  fieldへアクセス可能」という可視性事実は`motolii-timeline-pane`の
  `write.rs`/`projection.rs`が同型の構図を持つことで間接確認したが、
  本crate自体での実験(実際に`glue/`ディレクトリを作ってコンパイルを
  通す)はしていない(発注の「コードは1行も変えない」という制約のため)。
