# UXチェック台本 — ペルソナ別「最初の10分」

最終更新: 2026-08-18
対象build: `motolii-blitz-shell`(`cargo run -p motolii-ui --bin motolii-blitz-shell`)

これは**採点表ではない**。各ペルソナの手の動きを演じるための脚本であり、
引っかかった場所を「どこが変か」で指してもらうための道具である。
台本どおり動いても違和感があればそれが最優先の発見で、台本の方を捨てる。

## 起動

```bash
cargo run -p motolii-ui --bin motolii-blitz-shell
```

projectなしで起動すると**スタート画面**(New Project / Open)が出る。**Cmd+N**で新規
project(既定1920x1080)、**Cmd+O**で既存を開く。以降の全編集はCmd+Sで保存(未保存は
帯に「● unsaved」)。開発用のfixture展示は`--fixture`でのみ出る。

## P1: 曲が先にあるDTMer

1. Cmd+N で新規project
2. 曲(m4a/mp3等)をFinderから窓へドロップ — **soundtrack未設定ならそのまま曲として貼られる**
   (2本目以降の音声はclip配置。差し替えUIは未実装=入れ間違いはCLIで)
3. soundtrackが付くとruler直下に**波形帯**が出る
4. Space で再生 — 音が鳴り、playheadが音に同期して進む
5. 聴きながら **M** でセクション頭にマーカーを打つ(再生は止まらない)
6. 動画/画像をドロップしてマーカー間に配置、端をドラッグしてマーカーへ
7. ループ的に詰める: 再生中でも移動/トリムできる(止まらないのが正)
8. **Export** ボタン → 保存先を選ぶ → 進捗と経過秒 → 完了

見るべき手触り: 波形と目盛のズレ / M の打鍵遅延 / 再生中編集の安定 / 書き出し中のUI応答。

## P2: AviUtl移住者

1. Cmd+N → 動画をドロップ
2. clipを右クリック — 分割/ロック/名前変更等がメニューに揃っているか
3. clipを選択 → Inspector で **Position X/Y を数値直打ち**(タイプ→Enter)
4. 矢印キー等でのコマ確認、トリム端の精度(ズームして1フレーム単位が狙えるか)
5. Cmd+Z 連打でどこまでも戻れるか、Undo/Redoボタンでも同じ225か
6. Cmd+S → 帯の「● unsaved」が消えるか。閉じようとすると確認が出るか(未保存時)

見るべき手触り: 数値直打ちの確定感 / 右クリックの網羅 / Undoの深さと正しさ。

## P3: スマホ世代の初制作

1. 起動 → 何も読まずに**縦動画(スマホ撮影9:16)をドロップ**
   - projectが無ければ「先に作る」案内が帯に出る → Cmd+N
2. ドロップしたclipがtimelineに立ち、**Stageに絵が出る**(縦動画は左右letterbox)
3. Space → 再生。playheadドラッグでスクラブ、絵が追従
4. clip端をドラッグしてトリム、真ん中を掴んで移動
5. 間違えたら**Undoボタン**(画面上にある)
6. **Export** → 保存先を選ぶ(既定ファイル名は入っている)

見るべき手触り: ドロップから絵までの時間 / 触れそうな物が全部触れるか(Q0) / 無反応ゼロ。

## P4: AE難民

1. project を開き、clipを選択
2. Inspector の Transform に **◇** — 押すとplayhead位置にキーが打たれ ◆ になる
3. playheadを動かして値を変える → 2つ目のキー → 再生で補間を確認
4. timeline上のキー(菱形)をドラッグ/右クリックでイージング
5. 複数clipを選択(Cmd/Shift)→ Cmd+G でGroup化 → Groupにも操作が効くか
6. Cmd+D で複製
7. **プレビューが嘘をつかないか**: 書き出した結果とStageの絵を見比べる(Preview=Export同一評価が売り)

見るべき手触り: キー打鍵の即時性 / イージングpopupの操作感 / Group越しの選択の自然さ。

## P5: 無言の床(全員)

- 編集→Cmd+S無しで閉じる→**確認が出る**(Save/Discard/Cancel)
- 保存→再起動→**続きがそのまま開く**
- 書き出し中にCancel→途中fileが残らない
- 日本語・スペース入りファイル名の素材を入れてみる
- わざとprobe不能なfile(.txt等)をドロップ→**理由つきでskip**される(黙って消えない)

## 既知の「まだ無い」(違和感ではなく未実装)

- soundtrackの差し替え/削除UI、offset/gainのUI
- clip音声のmix再生(soundtrackのみ鳴る)/ 波形はsoundtrackのみ
- Effectの追加/削除UI、Anchor/Scale/RotationのInspector行(経路は実在、行が未表示)
- Browserパネルにproject assetが出ない(folder閲覧のみ)
- 中間bake/proxy(K7まで無し)・autosave(明示保存のみ)・レイアウト永続化
- 書き出しの設定詳細UI(既定値のみ)・フレーム割合の進捗バー
