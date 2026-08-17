# CLI→GUI 運転席 — 連携エラーが構造的に隠れない駆動・観測座席

日付: 2026-08-18
状態: **決定**(2026-08-16 Timeline再選定で決めた開発動線 第2層=egui_kittest の、shell全体への実行)

## 問題(実測)

利用者の違和感「computer-useでのGUI確認はエラーメッセージも出ず分かりにくい」は
実装の構造と一致していた:

- shell の失敗のうち status 帯へ出るのは app.rs 経由のものだけ。**Stage構築失敗・
  composition失敗・document不読は `eprintln!` だけで消え、窓は黙って空白になる**
  (pane.rs:573-575, 586-588, 595-598, 655-657, 664-670, 772, 821。他に
  browser_panel/mod.rs:1108 等)。
- GUI を外から駆動する口が無い(IPC/stdinゼロ、`--screenshot`は撮って閉じるだけ)。
  New/Open/Export/未保存確認は rfd の native dialog で、外部駆動はそこで必ず止まる。
- 結果、窓の検証は「人が画素を見る」しかなく、campaign 引き継ぎの
  「実走は静止screenshotの合成でしか確認していない」ギャップが残った。

## 決定

**運転席(driver seat)を shell に常設する。** 3つの座席、いずれも製品UIを変えない:

1. **ShellTranscript = 言う場所は1つ**。窓の一言(status)は全て transcript を通す。
   帯は `latest()` を映し、全 report が順に残る。pane の stderr 専用失敗は全廃
   (フェンス: tests/shell_error_fence.rs)。runner に `--status-log <path>` を足し、
   JSONL(`{"seq":n,"text":"…"}`)で外へ流す — CLI 駆動の実行は必ず機械可読の
   失敗記録を持つ。
2. **ScriptedPrompts = dialog の台本化**。rfd 呼び出し4本(new/open/export/未保存)を
   `ShellPrompts` trait の後ろへ。窓は NativePrompts(現挙動そのまま)、テスト・CLI
   駆動は台本が答える。「dialog抜きの関数境界まで」だった機械検証の壁を落とす。
3. **DrivenShell = egui_kittest 0.35 の運転席**。`BlitzShellApp` を kittest Harness で
   回し、AccessKit ラベルでクリック・DroppedFile 注入・フレーム進行・transcript 照合
   まで headless で決定的に行う。構築の seam: `with_seat` の `CreationContext` 直結を
   ほどき、`egui::Context` + `egui_wgpu::RenderState` + prompts で組めるようにする
   (`ui()` の未使用 `eframe::Frame` 引数も落とす)。GPU 無し環境は
   `gpu_or_skip` と同じポリシーで skip。

合格条件は red 先行で `crates/motolii-ui/src/blitz_shell/drive_tests.rs` と
`crates/motolii-ui/tests/shell_error_fence.rs` に固定済み
(red証跡: evidence/2026-08-18-driver-seat-red.txt)。

## 理由

- kittest は 2026-08-16 決定の既定路線(公式・egui/egui-wgpu/wgpu 全一致、Rerun が
  同構成で実運用)。今回はその適用先を Timeline 単体から shell 全体へ広げるだけで、
  新しい toolkit 判断はしていない。
- 画素クリック(computer-use)を先に整備しない理由: エラーが構造的に見えない・
  非決定的・dialog で止まる。transcript+kittest が揃えば、同じ台本を「窓あり
  (--status-log で観測)」でも「headless(テストとして)」でも走らせられる。

## 残余(順不同)

- browser_panel ほか pane.rs 以外の stderr 専用失敗のフェンス拡張
- `--drive <script>`: 窓ありで台本(click/key/drop/shot)を注入し per-step JSONL+PNG を
  吐く口。kittest 側と同じ語彙にする
- typed diagnostic(diagnostic.rs / diagnostic_projection.rs、現在消費者ゼロ)を
  transcript の構造化面として接続する
- runner.rs `write_png` の `.expect` 2箇所(撮影失敗も transcript へ)
- timeline_editor 内部 status の transcript 合流
