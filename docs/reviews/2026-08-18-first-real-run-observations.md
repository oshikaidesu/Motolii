# 実素材での初の通し実走 — インポート→出力の観察

日付: 2026-08-18
状態: **観察**(決定を含まない)

## 走らせたもの

- 素材: ffmpeg 生成(testsrc2 1280x720/30fps/4s の mp4、sine 440Hz/6s の m4a)
- CLI鎖: `new → import ×2 → place → set-soundtrack → export-document`(全 exit 0)
- 出力検証: ffprobe(h264 1920x1080 + aac)、フレーム抽出2枚の目視
- GUI: `motolii-blitz-shell --project … --screenshot`(exit 0、Stage は Rerun 経由で
  合成フレームを表示。Browser/Inspector/Timeline/transport/Export/Undo 帯まで描画)

## 見つかった欠陥(重要度順)

1. **place が clip 長を source 長でなく composition 長にする**。
   project.json 実測: `composition/duration = 10s`、`tracks[0]/items[0]/duration = 10s`
   (source は 4s)。結果、source 終端以降は最終フレームのフリーズフレームが続く
   (t=5s の抽出フレームに source の 3.967s が写っている)。普通のエディタの期待は
   「clip は source の長さで立ち、終端の先は背景」。
2. **export の報告と現物が食い違う**。CLI は「wrote 300 frames」(=comp 10s)と言うが、
   実出力は 178 フレーム(5.93s)。mux が音声(6s)側で黙って切っている。
   報告=現物の原則に反し、(1)と合わさって「10秒書いたはずが6秒しかない」体験になる。
3. **Stage の絵が斜めの3D視点のまま**(グリッド床+暗赤背景の上に遠近付きの板)。
   引き継ぎ既知の Rerun fork seam(`SpatialStage` が `AppendToStore` を落とす)の
   再確認。正対2Dプレビューが normal editor の期待。
4. **browser thumbnail の失敗が stderr 専用**(SVG 不読で「画像なしで描く」が帯に出ない)。
   [運転席決定](2026-08-18-cli-gui-driver-seat.md)の残余(フェンス拡張対象)と一致。
5. (小) `import` の標準出力が「imported asset 0」で、`--asset` は素の id を要求する。
   機械駆動には出力が id 単体である方がよい(運転席の CLI 側整合)。

## 素通りした部分(正しく動いた)

- 座標・時刻同期: 出力 t=2s のフレームに source の 2.000s が写る(ズレなし)
- 16:9 source の全面 fit、soundtrack の波形帯・Timeline 表示、lock file、screenshot 口

## 次の一手(推奨)

運転席レーン(進行中)の検収後、(1)+(2) を1レーンで修正発注する
(place の既定 duration=min(source, comp残り)、export の報告=現物、無音切り詰めの明示化)。
(3) は Rerun fork seam レーンとして別口(単独で重い)。
