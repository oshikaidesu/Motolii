# セッション引き継ぎ — リセット2日目(地図クローズの日)

日付: 2026-08-20 / 状態: **引き継ぎ**
セッション ID: `1ac8f720-602a-48be-a910-7ba7c703d850`
ブランチ: `claude/motolii-reset-handoff-bda7f3`(worktree 同名)

**バイアスを抜いて書く。** 進んだことより、信じてはいけないことを先に。

## 1. まず疑うべきこと

| # | 事実 | なぜ疑うべきか |
|---|---|---|
| 1 | **人が触ったのは Timeline 第1波とチラつき確認だけ** | camera の視差・blend Add・音声 mix・Inspector は機械検証(またはスクショ)止まり。「地図クローズ」は意味が store に載った宣言であって、体験の宣言ではない |
| 2 | **裁定の採番バックログ ~11件** | レーン報告に列挙済みだが DECISIONS.md 未記載(effect の Value 変種、音声の設計判断、blend/matte、R3、gizmo スクラッチ化=裁定114 の上書き、ui_scale、dock 配置=Workspace 等)。台帳だけ読むと今日の判断が少なく見える |
| 3 | **チラつき修正は緩和策** | 真因=iced 0.14 の 2MiB 同期アップロード予算。preview を 1.5MB へ縮小して回避(裁定21 と整合)が、恒久解(iced 上流修正 or GPU 埋め込み)は未裁定 |
| 4 | **並列の統合ずれが2回起きた**(`Keyframe.spatial` 追随漏れ×2) | write-set 分離でも「共有型の変更」はグローバルに波及する。supervisor が機械修正したが、型変更を含む束の直後は他レーンの再検証が要る |
| 5 | **音声はまだ音が出ない** | motolii-audio は意味核(mix/program/時計)のみ。cpal 結線・ring(rtrb)・再生は第2切片 |
| 6 | 時間予算試験2本は負荷で赤くなる(KNOWN 記載) | 並列レーン中の全体テストで赤が出ても単独緑なら既知 |

## 2. この日にやったこと(1行ずつ)

- **Lottie 地図 557項目 完全クローズ**(採用済230・不採用327・未判定0)。mask/shape-2+3/marker/split-position/transform-skew/layer-meta/camera/text75/effect/slot/motion-path
- **camera 束**: 透視 2.5D(z=0既定と画素一致 max_diff=0)・層z・pinned。空間モデルの正本を旧台帳から回収(裁定113、supervisor が一度誤読し利用者が訂正)
- **R3 probe**: 利用者実データの点群 196,133点を撮影、視差を画素差分で実証(D12)
- **Timeline 第1波**+fixture モード+**デザイン外出し**(tokens、保存即反映)+チラつき真因特定と柵
- **Inspector 第1波**(トンマナ主戦場、検収待ちで走行中)
- **blend/matte 消費①**: Normal 完動、Add は tint.a=0 で厳密に実装。**14 mode+matte は fork の shader 拡張が必要と特定し明示 Err**(黙って近似しない)
- **音声**: 調達調査 DONE(Rust→Tracktion/GES/MLT/Ardour→ゲーム/WebAudio/放送の全界隈。owns ~950行は「比較の上での意図的自作」)。新 crate motolii-audio(48 tests、決定論 mix+audio-clock-master)
- **器具**: KNOWN.md(検証済み事実台帳=レーンの重複検証を構造で禁止)、CANON.md(理想文書+デザイン正本の持ち込み索引)、Q0 機械柵(iced_test)、敵対的レビュー2回+柵塞ぎ
- **裁定文書**: グループ(parent/fold/隔離/フリーズ2段)、Timeline 意味論(不変量4・置き場3分類・切片6)
- 運用進化: 均等粒+判断領域別の発注、write-set 分離並列(最大5本)、採番は supervisor 専任

## 3. 次の人が最初にやること(順序つき)

1. **Inspector 第1波の検収**(走行中レーンの返りを待つ)→ fixture 窓で利用者に見せる
2. **採番バックログを DECISIONS へ落とす**(§1-2 の11件。次番号は 123 から)
3. **Timeline 第2波・切片1(領域モデル)** — 崩れの根治。仕様源=`2026-08-20-timeline-pane-semantics.md`
4. **dock/multi-window spike + ui_scale**(shell 波。pane_grid/daemon は上流実在の見込み — spike で実測してから KNOWN へ)
5. 音声②(cpal+rtrb 結線)/ 字形描画(harfrust/fontique の依存裁定が先)/ matte+14blend の fork seam 裁定
6. **利用者のトンマナ確定値を release の const へ畳む**(確定宣言を待つ)

## 4. この日に自分(supervisor)が間違えたこと

| 誤り | 露見 |
|---|---|
| 2.5D を「z 不変」と誤読しカメラを2D退化させる提案 | 利用者訂正。過去正本(2026-07-16)を検索してから語る、を CANON/メモリ化 |
| **「無い」を探索範囲の宣言なしに2回言った**(gizmo=語彙の壁 / 音声=界隈の壁) | 利用者の懐疑で発覚。KNOWN に「範囲併記」規則を追加 |
| 最終束に3判断領域を同載(責任過多) | 利用者指摘で走行中に分割。均等=行数×判断領域、をプレイブック化 |
| 実装レーンを opus で発注(sonnet 規則違反)/ cwd ミスでコマンド空振り数回 | 自覚訂正 |
| Ravel ギズモ推し | 利用者の実機感覚(ダサい)が却下 — スクラッチ裁定へ。調達の理屈は手触りに負けることがある |
