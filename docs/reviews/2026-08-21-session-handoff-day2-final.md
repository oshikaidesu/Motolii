# セッション引き継ぎ — 2日目最終(地図クローズ〜UI手触り戦役)

日付: 2026-08-21 / 状態: **引き継ぎ**
セッション ID: `1ac8f720-602a-48be-a910-7ba7c703d850`(day2 引き継ぎと同一セッションの最終版)
ブランチ: `claude/motolii-reset-handoff-bda7f3` / 裁定138本 / 全レーン合流済み(例外1件、下記)

## 1. まず疑うべきこと

| # | 事実 | なぜ疑うべきか |
|---|---|---|
| 1 | **市松レーン(worktree-agent-af46a5fdd4449e25b)が未回収** | 2度起こしても最終報告なし。branch に途中コミットが残っている可能性 — **待たずに打ち切り、新レーンで引き継ぐ**こと(真因はほぼ確定済み: 既定背景が不透明黒なので市松が1画素も見えない=M13違反の連動不足。修正方針は同 branch の diff と発注文を見よ) |
| 2 | **transient overlay は store に入ったが shell の drag はまだ旧ハック** | 書き換え手引き: continue_drag = `set_transient` 1発 / 確定 = Intent 1発+`clear_transient` / Esc = `clear_transient` のみ / 再描画判定は `display_revision()` へ。無害化ワークアラウンド(cancel_inspector_interaction 内)は丸ごと削除 |
| 3 | **裁定138 の常設 worktree 運用はまだ初適用前** | scratchpad の wt-plain(warm)は**セッション死で消える**。次セッションは安定パス(例 `.claude/worktrees/lane-{shell,store,engine}`)に常設 worktree を作り、Agent の isolation を使わず cd 指定で発注する。`-p` 集合固定を忘れると28.6sの再ビルドを踏む |
| 4 | UI の合否はまだ人間未確定 | 候補B palette・Inspector v2・drag は実装済みだが、利用者の「まだ違和感」(面→線、文字weight)への**線化 pass は未実施**(裁定137が仕様) |
| 5 | probe 環境ノイズ | MotoliiRn は終了済みだが Cursor fileWatcher 等の常駐が第二の汚染源(裁定138) |

## 2. day2 引き継ぎ以降にやったこと(1行ずつ)

- **Lottie 地図 557 完全クローズ**(採用230/不採用327/未判定0、slot は untagged serde で保存 bit 互換)
- **alpha 解禁**: fork 改造ゼロの1行(裁定132)。市松・alpha export への道が開通(export 側消費は未実装)
- **m/s/l**(solo は engine 消費まで・locked は RemoveLayer 含む拒否)+ **Composition.background**(Document・書き出しに乗る)
- **Inspector v2**: mock 列グリッド+**±1px 照合柵8本**(実欠陥2件を捕獲)+ **数値ドラッグ**(Shift 微調整・Esc 復元)+ **設定パネル**(背景色/市松トグル/ui_scale)
- **色正本を候補Bへ差し替え**(watch で即反映を実機確認)・**ui_scale**(1%刻み)実装
- **transient overlay API**(store、履歴無傷、11テスト)
- **R2 回帰を probe が捕獲→根治**(any_solo O(N²)→O(N)、7701→680µs)— 横断柵の検収規則を裁定136に
- **ビルド速度の決着**(裁定138): warm+`-p`固定で16〜33s合格。sccache 等は理由つき全却下。**次の構造レバー= shell の test バイナリ10本統合**(リンクがコストの9割)
- 器具: KNOWN.md 大幅増補(音声調達 DONE・iced API欠け・レーン運用)・裁定123〜138

## 3. 次の人の最初の手(順序つき)

1. 市松レーンの打ち切り回収(§1-1)→ 常設 worktree 運用の初適用として再発注
2. **shell test バイナリ統合**(裁定138 — ビルド税の本丸)
3. transient への drag 書き換え(§1-2 の手引き)
4. **線化 pass**(裁定137: 区切りは線・面は2段まで・文字は weight/ink 3段。mock ui-scale-and-z が実装形)
5. タスク#17(preview 忠実度)・#16(dock/multiwindow spike)・#19(track解析キャッシュ)・#14(Browser 意味起草)
6. 音声②(cpal+rtrb 結線 — KNOWN の調達地図どおり)/ 字形描画(harfrust 依存裁定から)

## 4. supervisor の誤り(このセッション後半)

- **cwd 事故を計4回**(リポ直下で next/ のコマンド空振り・迷子 DECISIONS.md 生成2回)— コマンドは常に絶対 cd から
- 発注書の規律を末尾に置き、**cargo 背景実行の自停止を3レーンで再発させた**(冒頭太字化で対処)
- Inspector 発注で「判断領域の同載」を自分の新規則制定直後に再犯
- sccache を測定前に有望視(界隈の定番=このリポの律速、ではなかった。「理論値を先に・検索を先に」の利用者の方法論が正した)
