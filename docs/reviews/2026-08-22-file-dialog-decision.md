# 裁定176 — File 束の入口: rfd(native file dialog)採用

日付: 2026-08-22(延長戦) / 状態: **決定**(供覧 — 保守最低限の自明適用)

New Project / Save As / Save a Copy / Open の path 選択は **`rfd` crate(native dialog)**で受ける。wraps>移植>スクラッチの自明適用(自前 file browser は作らない)。確認済み: rfd は winit/iced と併存可能な同期/非同期 API を持つ標準選択肢。персист経路は既存の汎用 persist(flattened)を使い、新しい保存形式は発明しない。Quit は未保存確認つき(dirty=revision 比較)。map: 1221(New Project, freq4)/1225(Save As)/1227(Save a Copy)/1223(Quit)。
