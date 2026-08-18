# 外部LLM診断 — UX死角の総ざらい(検収済み)

日付: 2026-08-18
状態: **観察**(外部診断。supervisor検収済み)

発注: Codex CLI `gpt-5.6-sol`(reasoning high)・read-only order・family分離(実装=Claude系/診断=GPT系)。
検収: command log監査(55実行、docs+全UI source+cargo 3本+PNG拒否再現を実走)、
全10 findingの引用をsupervisorが回し直し **10/10 CONFIRMED**。
原文orderと証跡: `/private/tmp/motolii-ux-diagnosis-20260818{,-logs}`。

**全10件着地(同日)**: wave C(`b3b7e57b` F-07〜F-10)・wave D(`11331403` F-01/F-04/F-05)・wave E(`d82b7b40` F-02/F-03/F-06)。各merge後のworkspace gate失敗ゼロ。F-03は枝A(document意味が実在し結線・局所bool削除)。

検収後の優先度(supervisor付記): 高=F-01(再起動で続きが開かない)・F-02(死んだ3面)・
F-05(台本乖離Export)・F-06(Stage選択の読み捨て)・F-03(M/S・FXが見た目だけ)。
黙殺系F-07〜F-10はフェンス拡張1レーンに束ねる。F-04(矢印コマ送り)は小。

---

# UX dead-interaction diagnosis

## DONE

- `ORDER.md` の read set 内だけを、`rg`、行番号付き source read、許可された3本の `cargo test`、既存 `target/debug/motolii-cli` で診断した。
- welcome / Browser / Timeline / Inspector / Stage / transport / Export / shortcuts / drop を、`docs/ux-check-first-ten-minutes.md` P1-P5 の操作列へ対応付けた。
- 新規 finding はすべて B / C / D / E に分類し、source quote、再現 command または GUI 不在時の operation→code route、指定 known-list への有無を付けた。
- code、既存 test、既存 docs は変更していない。

### script / surface coverage

| 対象 | 判定 |
|---|---|
| welcome / New / Open / Save / unsaved close | source route と headless test は成立。native dialog と実ウィンドウ close は `BLOCKED`。保存後の通常再起動は F-01。 |
| Browser | single-click selection は実装済み。double-click placement は既知。chrome texture 面は F-02、thumbnail failure は F-09。 |
| Timeline | move / trim / split / lock / rename / Group / duplicate / key / easing / Undo は source と test で成立。矢印キーは F-04、内部 status は F-07。 |
| Inspector | Position 数値編集、diamond、key update は integration test で成立。M/S と FX ON/OFF は F-03。 |
| Stage | playhead→evaluated frame の test は成立。実描画・スクラブ追従は `BLOCKED`、selection return は F-06、失敗伝播は F-08、Preview=Export は `UNVERIFIED`。 |
| transport | Space / loop / M と audio-seat test は成立。実機で「音が鳴る」体験は `BLOCKED`。 |
| Export | start / cancel / partial-file removal / completion の integration test は成立。P3 script drift は F-05、thread spawn failure は F-10。native save dialog と実行中UI応答は `BLOCKED`。 |
| shortcuts | Cmd+N/O/S、Space、M、Cmd+Z、Cmd+D、Cmd+G、Cmd+K は source route あり。矢印 frame-step は F-04。 |
| drop | project無し案内、audio/video admission、混在drop、probe不能理由は headless test で成立。PNGは既知。native Finder DnD と日本語・空白 path はそれぞれ `BLOCKED` / `UNVERIFIED`。 |

### 既知（FINDING へ重複計上しない）

- **既知:** Browser card double-click は `response.clicked()` の selection だけで placement handler が無い。
- **既知:** PNG は admission extension list に無く、CLI でも probe 前に拒否される。
- **既知:** template由来projectへの import は plugin contract error で落ちる。
- **既知:** Rerun camera の残り、orbit/zoom持続、reset は handoff の既知リストにある。
- **既知:** soundtrack差し替え/削除/offset/gain、clip mix/clip waveform は未実装。
- **既知:** Effect追加/削除、Anchor/Scale/Rotation行、Custom面、auto-key は未実装。
- **既知:** Browser project assets / Collections / SVG thumbnail は未実装または非対応。
- **既知:** autosave と layout persistence は未結線。
- **既知:** Export の frame割合 progress と詳細設定UIは未実装。
- **既知:** text input focus 中にも M-key が発火しうる。

## FILES

- `ORDER.md`
- `docs/ux-check-first-ten-minutes.md`
- `docs/reviews/2026-08-18-*.md`（特に session handoff、driver seat、first real run、user first touch、Rerun E0/foundation/seam）
- `crates/motolii-ui/src/blitz_shell/`
- `crates/motolii-ui/src/browser_panel/`
- `crates/motolii-ui/src/inspector_panel/`
- `crates/motolii-ui/src/timeline_editor/`
- `crates/motolii-ui/src/export_seat.rs`
- `crates/motolii-ui/src/rerun_stage/`
- `crates/motolii-ui/src/stage_frame_seat.rs`
- `crates/motolii-cli/src/`
- `crates/motolii-media/src/admission.rs`
- `crates/motolii-media/src/probe.rs`

## ORACLE_RESULT

**Overall: PARTIAL / BLOCKED.** Source-route diagnosis and allowed headless lanes completed; native GUI behavior is not passed. The prescribed exact negative-oracle status is blocked by the pre-existing untracked `ORDER.md`.

### positive oracle

- PASS: F-01 through F-10 each contain source quote + operation route/repro command + known-list decision.
- PASS: `cargo test -p motolii-ui -j 5` — exit 0。lib 311/311 と表示された integration/doc test suites は全て green。
- PASS: `cargo test -p motolii-media -j 5` — exit 0。通常 test は green、manual hardware benchmark 1本は `ignored` のまま。
- PASS: `cargo test -p motolii-cli -j 5` — exit 0。表示された unit/integration/doc test suites は全て green。
- PASS (expected rejection reproduced): `target/debug/motolii-cli import --project /tmp/motolii-ux-diagnosis-missing-project.json --media /tmp/motolii-ux-diagnosis-still.png` — exit 1、`probe failed: unsupported media file extension for admission: ...still.png`。

### negative oracle

- PASS: code / test / existing docs の変更は 0。
- BLOCKED: order が要求する `git status --short` の「`?? DIAGNOSIS.md` のみ」は達成不能。開始前から利用者所有の `?? ORDER.md` があり、削除・追跡・変更は scope 外。最終実測は `?? DIAGNOSIS.md` と `?? ORDER.md` の2行。
- PASS: 既知項目は上の一行一覧だけに抑え、新規 finding として水増ししていない。

## NOT_DONE / UNVERIFIED / BLOCKED

### UNVERIFIED

- P4 Preview=Export pixel identity。`docs/reviews/2026-08-18-rerun-e0-composition-probe.md:245-246` 自身が offscreen と window の比較未実施、同じ `SpatialStage::show` は構造上の推測に過ぎないと記録している。`cargo test` green を代用しない。
- P5 日本語・スペース入り素材 path。該当する実media fixtureを read set内で確認できず、作成・変換は runtime allowlist 外。
- Stage entity click 後の Rerun 内部 highlight の有無。F-06 が確定するのは Motolii 側が返却値を捨てる点まで。
- 実時間の滑らかさ、M打鍵遅延、再生中編集の手触り、Export中のUI応答、視覚品質。性能・見た目評価は non-goal でもあり未判定。

### BLOCKED

- native GUI window が無いので、Finder drag/drop、card double-click、native dialogs、window close、actual audio output、Stage/Timeline の実操作、Export中の対話を実走できない。
- cargo lane は BLOCKED なし。3 command とも exit 0。
- exact negative oracle は、開始前からの `?? ORDER.md` を保存するため BLOCKED。

## EVIDENCE_GAP

- windowあり driver / per-step status-log / screenshot が無いため、headless route と user-visible result の間は閉じていない。
- GPU texture copy failure、Stage mesh ingestion failure、export thread spawn failureを安全に強制する既存oracleは read set内に無い。F-08/F-10 は到達可能な error branch と caller behavior の code-route 診断であり、runtime injection 実測ではない。
- Preview と Export を同一時刻・同一入力で比較したpixel pairが無い。
- 日本語・スペース入りの実media fixtureと実drop logが無い。

## FINDING

### F-01 — C（副分類 E）: 保存済みprojectを通常再起動時に自動復帰する entry が無い

1. **source quote:** script は `docs/ux-check-first-ten-minutes.md:71-72` で「保存→再起動→**続きがそのまま開く**」と約束する。一方、`crates/motolii-ui/src/blitz_shell/main.rs:37-50` は起動時 `project` を `None` で始め、`--project` 指定時だけ path を入れる。`crates/motolii-ui/src/blitz_shell/app.rs:447-450` は座席なしなら welcome を出す。
2. **operation→code route:** Save → process終了 → 引数なし `motolii-blitz-shell` → `main.rs:39 project=None` → `runner.rs:51-56 seat=None` → `app.rs:447-450 welcome`。`rg -n "recent|last_project|reopen|restore.*project|project.*restore" <ORDER read set>` は 0 hit（exit 1）。
3. **known-list:** **なし。** autosave/layout persistence の既知項目は「明示保存済みprojectの最近使った座席を再起動で開く」entryとは別。

### F-02 — B: default shell に見える Export / Settings / Panels の内容面はクリックを受けない

1. **source quote:** `crates/motolii-ui/src/blitz_shell/app.rs:946-953` は3枚を default tree に挿す。`crates/motolii-ui/src/blitz_shell/pane.rs:266-270` は HTML pane について「マウスは受けない(`Sense::hover()`)。入力ルーティングは後続capsule」と明記する。実際の押せる Export は status帯へ別置きされ、`app.rs:720-724` も chrome fixture がマウスを受けないと明記する。
2. **operation→code route:** Export/Settings/Panels の tabを開く → 内容をclick → `BlitzPane::show` HTML texture route → `Sense::hover()` のみ → handler / result UI / status なし。tab切替自体ではなく内容面の判定。
3. **known-list:** **なし。** Export詳細設定不足は既知だが、default-visibleな3内容面全体が無入力なのは指定リストに無い。

### F-03 — B/E（ambiguity）: Inspector の M/S と Effect ON/OFF は見た目だけ変わり、製品意味へ届かない

1. **source quote:** `crates/motolii-ui/src/inspector_panel/mod.rs:101-105` は M/S と FX を「局所視覚状態」とし、書き込みrouteは後続。M/S click は `:500-505` で `self.solo/self.muted` のみ反転、FX click は `:965-980` で `disabled_effects` のみ更新する。呼び手へ返せる `InspectorAction` は `:43-67` の数値編集/終了/keyだけ。
2. **operation→code route:** layer選択 → Inspector の M/S または FX ON/OFF click → local bool/setだけ更新 → `InspectorAction` なし → `blitz_shell/pane.rs:206-230` の TimelineEditor適用口へ何も渡らない → Document/Stage/Export結果なし。局所のpressed表示は変わるため、完全無反応Bか「UIの意味と実装差」Eかは ambiguity として保持。
3. **known-list:** **なし。** Effect追加/削除や不足Inspector行は既知だが、既に見える ON/OFF と M/S のlocal-only挙動は別。

### F-04 — E: P2の矢印キー frame-step が実装shortcut集合に無い

1. **source quote:** script は `docs/ux-check-first-ten-minutes.md:36-40` で「矢印キー等でのコマ確認」を要求する。Timeline の shortcut tuple は `crates/motolii-ui/src/timeline_editor/mod.rs:4804-4815` の Z/Escape/D/Delete/G/K/A と、`:4850-4859` の M/Enterで、Arrow key branchが無い。
2. **repro command:** `rg -n "Key::Arrow|ArrowLeft|ArrowRight|ArrowUp|ArrowDown|key_pressed\\(egui::Key" crates/motolii-ui/src/timeline_editor/mod.rs` は Space/L/Escape/Enter/Z/D/Delete/G/K/A/M のみを列挙し、Arrow hit 0。
3. **known-list:** **なし。** M focus risk は既知だが frame-step 欠落ではない。

### F-05 — E: P3の「Export→1クリック既定書き出し」は常時保存先promptと一致しない

1. **source quote:** `docs/ux-check-first-ten-minutes.md:53` は「**Export** → 1クリックで既定書き出し」。`crates/motolii-ui/src/blitz_shell/app.rs:591-605` は Export のたび `prompts.export_path(seat.path())` を呼び、pathが返らなければ終了する。なお同script P1 `:30` は「保存先を選ぶ」と書き、script内部にもpersona間 drift がある。
2. **operation→code route:** status帯 Export click (`app.rs:741-764`) → `begin_export` → `prompts.export_path` → native保存先選択。1クリックだけでは export worker に到達しない。
3. **known-list:** **なし。** 既知は設定詳細UIとframe割合progressで、保存先promptの要否ではない。

### F-06 — B/E（ambiguity）: Stageの選択entity返却値をshellが捨てる

1. **source quote:** `crates/motolii-ui/src/rerun_stage/adapter.rs:479-490` は `show_in` の戻り値を selected entity path と定義し、`:523-526` で `take_selected_entity_path()` を返す。`crates/motolii-ui/src/blitz_shell/pane.rs:713-723` は `Ok(_)` を読み捨てる。
2. **operation→code route:** Stage entity click → `SpatialStage::show` → `take_selected_entity_path()` → `StagePane::show` の `Ok(_)` → Timeline/Inspector selection/Document intent/statusへのdispatchなし。Rerun内部highlightの有無は `UNVERIFIED` なので、Motolii選択結果が無いBと期待意味の不明確さEを ambiguity とする。
3. **known-list:** **なし。** camera/orbit seam の既知項目とは別。

### F-07 — D: Timelineの操作失敗/statusが ShellTranscript を通らない

1. **source quote:** authority は `docs/reviews/2026-08-18-cli-gui-driver-seat.md:24-28` で全statusを `ShellTranscript` に通すと決める。Timeline は `crates/motolii-ui/src/timeline_editor/mod.rs:4738-4747` で inherited lock失敗を `self.status` へだけ書き、`:3316-3342` でTimeline内へ描く。shell側は `crates/motolii-ui/src/blitz_shell/app.rs:40-43` で `editor.show(ui)` の返りを持たず終了する。
2. **operation→code route:** 親lock中のchildで L click → `self.status="... is locked by a parent"` → Timeline header描画だけ → ShellTranscript / status-log なし。driver doc `:75` 自身も「timeline_editor 内部 status の transcript 合流」を残余としている。
3. **known-list:** **なし。** 指定された2つの known-list には無い（別のdriver decisionには残余として既出）。

### F-08 — D: Stage adapterのGPU image/geometry failureがShellTranscriptへ届かない

1. **source quote:** `crates/motolii-ui/src/rerun_stage/adapter.rs:672-683` は `copy_gpu_image(...).is_err()` で無言return。geometryは `:408-451` が失敗を `false` で返すが、`crates/motolii-ui/src/blitz_shell/pane.rs:699-703` は戻り値を無視して `applied_geometry` を成功時同様に更新する。adapter内の再適用も `adapter.rs:688-692` で `let _ =`。
2. **operation→code route:** play/scrub → `StageFrameSeat::frame` → `show_in` (`adapter.rs:504-508`) → GPU copy失敗なら無言return。または geometry revision/resize → `StagePane::show` → `apply_host_stage_geometry=false` → callerが無視 → transcriptなし。safe failure injection oracleは無いため code route 診断。
3. **known-list:** **なし。** handoffのdevice-handler ownershipは別問題。

### F-09 — D: Browser thumbnail read/decode failureはstderrだけで、再試行もtranscriptも無い

1. **source quote:** `crates/motolii-ui/src/browser_panel/mod.rs:1095-1115` は失敗時 `eprintln!` だけを出し、`None` をcacheして再試行しない。`:1243-1245` は `std::fs::read(path).ok()?` と `image::load_from_memory(...).ok()?` で原因を捨てる。
2. **operation→code route:** Browserに読めない/壊れたthumbnail対象を表示 → `draw_card` → `texture_for` → `load_color_image=None` → stderr + glyph card → ShellTranscript/status-logなし。`docs/reviews/2026-08-18-first-real-run-observations.md:31-32` に同じ実走観察もある。
3. **known-list:** **なし（指定リスト基準）。** handoffのSVG thumbnail非対応は既知だが、「失敗理由がstderr専用」は別のdriver/observation文書に既出。

### F-10 — D: Export worker threadを起こせない時はpanicし、ShellTranscriptへ失敗を返せない

1. **source quote:** `crates/motolii-ui/src/export_seat.rs:125-140` の `ExportRun::start` は `std::thread::Builder::spawn(...).expect("spawn export thread")`。通常のworker切断は同file `:177-183` で `ExportFinish::Failed` になるが、spawn自体の失敗はその前にpanicする。
2. **operation→code route:** Export click → `app.rs:595-616 begin_export` → `ExportRun::start` → OS thread spawn error → panic → `handle_export_finish` / `ShellTranscript`へ到達しない。安全なspawn-failure injection oracleは read set内に無いため code route 診断。
3. **known-list:** **なし。** Export progress/detail不足とは別。
