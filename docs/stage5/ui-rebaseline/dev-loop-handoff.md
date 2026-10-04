# UI 開発ループ引き継ぎ(2026-10-01)

別の AI がユーザーと Flutter UI を実時間で詰めるための手順書。作業場所は `/Users/member_ottoto/rust_ae/Motolii`、branch は `claude/ui-rebaseline`。
`Motolii-canonical` とリポジトリ整理には触れない。

## 1. entry point と起動コマンド(コードから確認済み)

| 窓 | entry | 起動 |
|---|---|---|
| editor(Classic / New) | `motolii/ui/lib/main.dart` → `app/editor_app.dart` の `EditorApp` | `scripts/motolii-ui.sh dev [document.rrd\|script.js]` |
| hf GUI(Motolii Live の client) | `motolii/ui/lib/live_hf/main.dart` | `scripts/motolii-ui.sh live [document]` |

- **ユーザーが見たい現行 UI は `live`(hf GUI)。** `dev` は古い Classic 窓なので使わない(2026-10-01 にユーザーが確認)。
- `dev` は `MOTOLII_SHELL=new` を付けても古い配置で開いた。
- script の中身は `flutter run -d macos --pid-file ~/.local/state/motolii-stage5/flutter.pid --dart-define=MOTOLII_SHELL=...`。flutter は `.tools/flutter/bin/flutter`(確認済み、存在する)。
- `motolii/target/debug/libmotolii_ui.dylib` は存在する(確認済み)。無い場合だけ `scripts/motolii-ui.sh native` を 1 度、途中で切らず完走させる。

## 2. 初回

```bash
cd /Users/member_ottoto/rust_ae/Motolii/motolii/ui && ../../.tools/flutter/bin/flutter pub get
cd /Users/member_ottoto/rust_ae/Motolii && scripts/motolii-ui.sh live
```

`live` は前景で居座る。`nohup scripts/motolii-ui.sh live > /tmp/motolii-live.log 2>&1 < /dev/null & disown` で起動する(Mac に `setsid` は無い)。起動に使うシェルを `... & sleep; tail` の形で終わらせると子ごと落ちたので、起動と待機を別の呼び出しにする。

## 3. hot reload

```bash
scripts/motolii-ui.sh reload       # SIGUSR1 → hot reload(Document・GPU 資源を保持)
scripts/motolii-ui.sh restart-ui   # SIGUSR2 → hot restart(初期化で UI 状態を失う)
```

- 通常の Dart 変更は `reload`。restart は const・main・初期化子を変えた時だけ。
- Rust を変えた時だけ `native` で build → アプリを再起動。
- 反映確認は実窓(スクショ)。見た目を test で assert しない。

## 4. 毎ターンの形

ユーザーの視覚指示 → 関係する Widget だけ特定 → 最小 Dart 変更 → `reload` → 実画面確認 → 停止。
周辺 UI を再設計しない。architecture 整理を始めない。reload で済む変更で再起動しない。

## 5. 守る柵(memory より)

- 意味・機能・データを捏造しない。数値は Instrument の 4 番目。
- 動き: 入力直後に動く ease-out、hover で geometry を動かさない。
- 意図は UI、意味(delta・clamp・対象決め・undo 区切り)は owner。UI で決めない。
- Done は要求の実物達成。slice・green・push は Done ではない。

## 6. 現在の作業ツリー(未 commit)

```
M  motolii/ui/lib/app/new/inspector/new_inspector_panel.dart
M  motolii/ui/lib/panels/timeline/paint.dart
?? motolii/ui/lib/app/new/inspector/{new_fill,new_matte,new_text,session_layer_rows}.dart
?? motolii/ui/test/relations_capture_test.dart
?? work/
```

これは前の作業者の未レビュー作業。勝手に revert しない。

## 7. 読むべき既存文書

- [brief.md](brief.md) — UI rebaseline の正本。「Current UI is the capability oracle, not the layout oracle」
- [README.md](README.md) — Phase A の到達点と索引
- [../product-home.png](../product-home.png) — 階層・IA・操作約束の正本
- [../panel-placement.md](../panel-placement.md) / [../inspector.md](../inspector.md) / [../relations-v0.md](../relations-v0.md)
- [../workspace.json](../workspace.json) — Stage 5 の現在地
- `motolii/AGENTS.md` の開発入口の段

## 8. 実測(2026-10-01)

`live` で窓が表示され(Create・Stage・Inspector・Timeline)、`reload` が成功した(100ms)。見た目の変更を反映する reload はまだ試していない。
