# 2026-09-10 未明: Codex 9 本並走の仕分けと、debug のまま滑らかにする

## 結論

- Codex が 9/9 夜に同じ作業ツリーで 9 本並走させた未仕分けの成果は `wip/codex-2026-09-09` に丸ごと退避し、そこから Claude の 7 レーン(Ease / Inspector / Browser / Blend / History / 3D ギズモ / ホバー→M3)を worktree 隔離で作り直して `claude/ui-lanes-integrate` に合流した。
- そのうえで「debug build のまま UI が滑らか」を基準に性能を 5 弾で直した。実窓(Cube 1 + Rectangle 2、同じ 4 操作)の結果:

| | 修正前 | 修正後 |
|---|---|---|
| フレーム中央値 | 8.4 ms | 7.1 ms |
| フレーム p90 | 178 ms | 11.2 ms |
| 16 ms 超のフレーム | 14 / 40 | 3 / 63 |
| 最大 | 227 ms | 50.7 ms(1 回) |
| 操作の切り替わりの 1 フレーム目 | 45〜134 ms | 4〜10 ms |
| Stage で Cube をドラッグ(定常) | — | 2〜3 ms |
| GPU raster 中央値 | 1.6 ms | 1.3 ms |

native(FFI 直叩き、`bench6.py`):

| 15 層の書類 | 修正前 | opt-level 0 のまま | opt-level 1 |
|---|---|---|---|
| status 全層再構築 | 10.8 ms | 6.8 | 2.8 |
| 1 層だけドラッグ | 13.4 ms | 7.3 | 3.0 |
| 80 層で 1 層ドラッグ | 69.6 ms | 23.3 | 9.1 |
| 書体見本 1 枚 | 7.4 ms | 6.2 | 4.0 |

Flutter(widget test の build 数、1 回の値更新 / 選択変更 1 回):

| | 前 | 後 |
|---|---|---|
| Window 全体(値更新) | 1793 | 197 |
| Inspector(値更新) | 1037 | 85 |
| Timeline(値更新) | 372 | 115 |
| Window 全体(選択変更) | 4861 | 1173 |
| Blend / Desk / Ease(選択変更) | 639 / 430 / 394 | 0 / 0 / 0 |
| Fonts: 書式 1 つ触ったときの見本再生成 | 661 枚 | 0 |

## 何が重かったか(実測で裏づけたもの)

1. native が毎操作、全層の status を組み直していた(層数に線形、debug 0.49 ms/層)。→ 層ごとに指紋で cache。時刻・観測者は鍵に入れず、枠だけ毎回載せ直す。
2. Flutter の全パネルが 1 本の `document` 通知にぶら下がり、返信 1 回で全部作り直していた。→ パネルごとの slice 購読(`DocumentSlice`)、Inspector は行ごと、Timeline は骨格を書類から外す。
3. Fonts の見本が直列 1 本の native worker を塞ぎ、書式 1 つで 661 枚が無効になっていた。→ 鍵を書体と本文だけに、可視行だけを止まってから数枚ずつ。Swift は見本の返信を「書類が変わった」として全窓に配るのをやめた。
4. 表示を切っている Tooltip が widget として残り、Semantics と GestureDetector を積んでいた。→ `EditorTooltip` は表示オフのとき child を返す。
5. Stage の上下 bar が絵の更新のたびに IntrinsicWidth を測り直していた。→ 別 slice。
6. 「LAYOUT が 100 ms」の正体は RenderObject の layout ではなく、LayoutBuilder の中で走る widget build(68 回 69 ms)。debug は 1 widget 0.02 ms でも 3000 個で 60 ms。→ 上の 2 と 4 で消えた。
7. 自前 crate だけ opt-level 0 だった。→ `[profile.dev] opt-level = 1`(debuginfo 保持、再 build 24 s)。これは上乗せで、1〜6 が主。

無罪だったもの: Codex の status 版管理(効いていた)、History の台帳、spatialGizmo、Material 3 化、`_FittedName` の TextPainter、GPU。

## 残り(朝に判断してほしい)

- 最大 50 ms のフレームが 1 回残る(選択の枠ドラッグを離した直後)。Inspector の選択変更 1 回はまだ 1061 build(骨格は const 化済みで実質は少ないが、初回の layout は残る)。
- port 契約の要望 2 つ: `visualSamples` の一括形(見本 1 枚 1 往復をやめる)、編集の返信に描画結果を同梱して 1 ドラッグ 1 往復に。どちらも Rust 側の変更。
- 書体見本は新しい書体の初回だけ 4 ms(cosmic-text の face 読み込み)。見た目を変えずに縮める余地は小さい。
- 実窓の見た目の合否は全部未検収。各レーンの branch は残してあるので、落ちたものは branch 単位で外せる。

## 使ったもの

- 計測: `scratchpad/perf/frames.py <ws> start|stop`(VM service の timeline からフレーム時間)、`bench2〜6.py`(FFI 直叩き)。結果は同じ場所の `before.txt` / `after-round*.txt` / `baseline-notes.md`。
- レーンの branch: `worktree-agent-*`(7 本)、`claude/perf-flutter`、`claude/perf-layout`、`claude/perf-inspector`、`claude/perf-panels`。統合は `claude/ui-lanes-integrate`。
- 教訓: worktree を跨ぐ git は `git -C`。本線の ff を worktree 内で打ったせいで、第 2〜4 弾の実窓計測は第 1 弾のコードのままだった(RenderObject の数がぴったり同じで発覚)。
