# 再生中の編集は再生を止めない(Ableton型)

日付: 2026-08-17
状態: **決定**

## 決定

**編集操作は再生を止めない。** DAW(Ableton Live)の慣行に従う。AE型(編集で停止)は採らない。

| 再生中の操作 | 挙動 |
|---|---|
| clip移動/トリム/分割/キー編集/ロケータ追加 | そのまま適用。再生継続。次フレームから新snapshotが映る |
| playheadの手動移動(スクラブ/ロケータ跳び) | audio sessionを新originで開き直す(実装済み: `audio_seat.rs`のreseek) |
| soundtrackを変える操作(`SetSoundtrack`) | audio sessionを開き直す(GUIにsoundtrack操作が載る時に接続) |
| Undo/Redo | 通常編集と同じ。再生継続 |

## 根拠

1. **構造がすでにこちら側にある。** 音は`AudioProgram::from_document`がsoundtrackだけから組む。clip/キーの編集はaudio programに影響しないため、再生を止める技術的理由が無い。絵は毎フレームsnapshotを読み直す設計(editor→Stageのsnapshot配布)なので、編集の反映に停止は不要
2. **P1(DTMer)のnudgeループが前提にする。** ループ再生しながら1フレームずつ調整する手の動きは、編集ごとに停止すると成立しない([UX迂回策分析 2026-08-17](../decision-index.md))
3. **先例**: Ableton Liveは再生中の編集が標準。映像側でもResolve/Premiereはカット編集で再生を止めない。AEが止まるのはRAMプレビューがbakeであるためで、Motoliiのlive評価には当たらない

## oracle

- ロケータ粒(2026-08-17発注)のテストが「再生状態での追加が再生を止めない」を固定する(第一号)
- 以後、再生中に適用できる操作familyを増やす粒は、同型の「再生継続」テストを1本伴う

## 非目標

- 再生中のsoundtrack差し替えのgapless化(開き直しの瞬断は許容。gaplessは将来粒)
- clip audioのlive mix(AudioProgramの現行範囲のまま)
- 編集とaudio callbackのlock共有(writerはUI thread、audioは自分のring bufferを読む。既存分離を変えない)
