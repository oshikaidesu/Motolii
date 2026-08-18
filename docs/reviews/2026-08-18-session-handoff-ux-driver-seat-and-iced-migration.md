# セッション引き継ぎ — 運転席・診断修正・iced移行 M-0

日付: 2026-08-18
状態: **引き継ぎ**(観察。決定を含まない)

## セッションID

- 本セッション: `96b6d47f-7306-44c9-8853-b0c50ae4cd40`(クラッシュなし・単一)
- 前区切り(campaign第1区切り): `3632460e-2dad-454a-8eec-5e7ef46b8e9a` /
  `adeabfb2-adca-4488-a381-0cab097b3d01`
  ([前引き継ぎ](2026-08-18-session-handoff-normal-editor-campaign.md))

## 現在地(機械的事実)

- 統合branch: `claude/ux-cli-gui-integration-002b03`
  (worktree `.claude/worktrees/gallant-blackwell-f4d72c`)
- tip: `984233cf`(M-1 merge)。campaign tip `026630f6` を ff で取り込み、その上に **57 commit**
- **Motolii repo は remote へ一切 push していない。全成果 local のみ**
- **fork は 2 本とも GitHub へ push 済み**(cargo の rev pin に必要なため):
  - `oshikaidesu/rerun` branch `motolii/stage-camera-seat` = `483b8559`
    (camera seat。乖離台帳=[rerun fork seam ledger](2026-08-18-rerun-fork-seam-ledger.md))
  - `oshikaidesu/iced` **新設 fork** branch `motolii/host-seams` = `73e686ee`
    (web-sys 釘打ち解除+bind groups floor。台帳=[iced fork seam ledger](2026-08-18-iced-fork-seam-ledger.md))
- 最終 gate: `cargo test --workspace --no-fail-fast -j 5` **失敗ゼロ**(tip `984233cf` 時点)
- **M-1 は着地済み**(`984233cf`、引き継ぎ作成後に返却→検収→merge→gate失敗ゼロ):
  iced 殻が Save/未保存3択/OSドロップ/Export開始・中止/replay oracle/--status-log
  まで持ち、iced_test 27テスト。注入可否の実測表と手動確認事項(実窓での
  経過秒・dialog 4種・実Finderドロップ)はレーン報告どおり migration 決定文書に追記済み。
  **次は M-2(Stage島)から**

## この区切りで入ったもの(merge順・全て gate 通過)

1. 運転席 red 先行(`00408b81`)→設計([CLI→GUI運転席](2026-08-18-cli-gui-driver-seat.md))→実装merge(`ec8415eb`): ShellTranscript/ScriptedPrompts/DrivenShell(kittest)/--status-log
2. 実走観察([初通し](2026-08-18-first-real-run-observations.md))→place/export修正(`ff1a69a3`): 尺=min(source, comp残り)・報告=現物
3. [Rerun合成基盤裁定](2026-08-18-rerun-as-composition-foundation.md)→E0 probe 3点成立(`c5bcd67a`)→camera seat(`7cf42a90`+fork push+pin `2a129e72`)→Stage正対化(`bf9f0b44`)
4. d1m lock flake根治(`076f2cce`)
5. 利用者一撃([初回タッチ](2026-08-18-user-first-touch-observations.md))→画像入口(`4bde7e86`)+Browserダブルクリック配置(`7d852ec3`)
6. [外部診断](2026-08-18-external-ux-diagnosis.md)(Codex gpt-5.6-sol・10/10 CONFIRMED)→wave C黙殺根絶(`b3b7e57b`)/wave D続き・矢印・台本(`11331403`)/wave E結線(`d82b7b40`) — **診断10件全着地**
7. [ログと構造の強制裁定](2026-08-18-log-and-structure-enforcement.md)→UiIntent背骨(`7b12d50e`): journal+gateway+replay oracle+フェンス
8. iced評価3probe(埋め込み`dfcf0b9f`・入力ブリッジ`eda55ef6`・[仮タイムライン](2026-08-18-iced-reentry-survey.md)`ebd5bc0e`)→**[ホスト移行裁定](2026-08-18-iced-host-migration-decision.md)**(利用者)→M-0(`5efa49d7`+pin `6545c039`)

## 検証の実態(バイアス抜きの核)

**機械検証済み:**
- 全レーン red 先行→green→workspace gate(no-fail-fast)。gate は各 merge 後に実行し全て失敗ゼロ
- E2E: CLI鎖(new→import→place→set-soundtrack→export)を実素材で実走し ffprobe+画素目視。画像も赤PNG→出力画素(250,0,0)まで
- replay oracle: 駆動セッション記録→世界を消して headless replay→座席/revision/帯一致(egui 版・iced 版の両方が常設)
- E0/camera/遮蔽/iced埋め込み/入力ブリッジ: 全て pixel 証拠つき probe(evidence/ 配下)
- フェンス群: eprintln(6ファイル)・gateway迂回・toolkit dep policy(egui/iced 両建て)・meta-fence

**人間検証済み(本セッション中・2回だけ):**
- 利用者が egui shell を1回触り、2欠陥を発見(→当日修正)。**修正後の egui shell 全体は未再確認**
- 利用者が iced 仮タイムライン spike を触り「変な感じはしない」→移行裁定の根拠の一つ

**人間未検証:**
- 修正後 egui shell の通し(診断10件修正が「手触りとして」直ったかは誰も見ていない)
- 音同期の耳・実 dialog・日本語/空白パスの実ドロップ(診断でも UNVERIFIED)
- ~~iced 殻(M-0)は窓を開いた実走を人間が見ていない(iced_test headless のみ)~~
  → 解消(2026-08-18夜): 利用者が実窓のスタート画面を目視(驚いて閉じた=閉じるボタンも実動)。
  実窓 screenshot も取得。見た目は iced 既定 theme のままで、トーン移植は M-4t レーンが担当
- Preview=Export の pixel 同一性(E0 §でも未測のまま)

## 既知の欠陥・残タスク(修正順は未決。ここが残作業の正本)

**iced 移行(地図= [移行裁定](2026-08-18-iced-host-migration-decision.md)):**
- M-1 着地済み(上記)。次= M-2 Stage島(bind groups seam の実効確認が受入条件・fork台帳§4§6)、M-3 Timeline(spike `spikes/iced-rerun-embed-probe/timeline/` 342行が種・D2結線)、M-4 Browser/Inspector、M-5 切替(台本P1-P5+replay green→既定bin切替・egui shellは`--legacy`)
- iced 側の既知穴: WheelScrolled が modifiers を運ばない/canvas にフォーカス概念なし/AccessKit 未統合(後退として裁定に明記)/shader widget に repaint 経済・IME 経路なし(島がview専用なら踏まない)

**egui shell(切替まで現役):**
- `project_mut()` の穴: Timeline/Inspector 編集は intent journal 外(宣言済み・M-5 まで許容)
- `--fixture` の Inspector は writer 無しで M/S が no-op(開発動線のみの Q0 gap)
- `--screenshot`(--fixture 無し)も last-project 自動 open の対象(wave D 逸脱注記)
- 奇数寸法画像は拒否(ffmpeg chroma 実測による据え置き)。GIF/TIFF/EXR/SVG は意図的除外
- reference-document.json の `core.filter.opacity` param 名が第一者契約(`amount`)と乖離 —
  該当 effect を含む document で Inspector が read-model を作れない。**チップ発行済み
  (task_ea4cb091)だが再起動で消える** — 消えていたらこの行が正本
- blitz-dump(窓なし tool)は stderr が唯一の言い場所(carve-out 済み・フェンス注記)

**前区切りから未着手のまま:** テンプレ由来 project の import が plugin 契約エラー/
soundtrack 差し替え・offset/gain UI/clip 音声 mix/Effect 追加削除 UI/autosave 結線/
Export 進捗のフレーム割合/orbit 持続(S2)・orthographic(S3)

## 運転規約(今回の追加教訓込み。playbook メモリと同内容)

- レーン: capsule(OUTCOME/CURRENT STATE/EXACT TARGET/ALLOWLIST/NON-GOALS/RETURN)+
  red 先行。**worktree は古い base で切られる — capsule に「tip へ reset」を書き、
  返却時も base を確認**(今回ほぼ全レーンで発生)
- **同時 cargo 2本まで・全 cargo に `-j 5`**。gate の集計は1パイプで(2回書くと2周する)
- **API エラー(500)で落ちたレーンの SendMessage 再開は worktree 隔離が外れ、
  統合 branch へ直接 commit する** — 再開後は必ず `git log`/`git status` で着地先確認。
  障害中は再spawnせず背景 sleep で待ってから再開
- 検収: diff読み→自分で再実行→merge --no-ff→gate。外部LLM発注は motolii-dispatch skill
  (command log 監査・oracle 回し直し)
- 裁定は docs/reviews→README index→decision-index 同一 commit+`check-docs.sh`
- fork の rev 更新検収は常設 oracle を回すだけ(rerun=E0 probe、iced=drive_seat)

## 状態の正本

- 本引き継ぎ+[icedホスト移行裁定](2026-08-18-iced-host-migration-decision.md)+
  decision-index の 2026-08-18 行群+git log+メモリ
  (`iced-host-migration` / `wrapper-over-hack` / `toolkit-reentry-trigger` /
  `full-delegation-normal-video-editor` 追記3)
- TaskList はプロセス再起動で消える。**残作業の正本は本文書の「既知の欠陥・残タスク」節**
- 利用者の役割(UX合否のみ・他は推奨自走)と乗り換え裁定の経緯はメモリが正本
