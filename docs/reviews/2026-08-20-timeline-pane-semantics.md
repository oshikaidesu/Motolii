# Timeline パネルの意味論 — 第2波の仕様源

日付: 2026-08-20 / 状態: **決定**(回収レーンの調査を supervisor が裁定化。出典は各正典 — ui-score-model / 裁定46/107/119/120 / normal-timeline-prior-art / Ableton 実測)

## レイアウト不変量(全切片共通の柵)
1. 固定 chrome(ruler / transport / ループ帯)の合計高さは**層数に依存しない定数**
2. 行は独立の**スクロール viewport** に住む。`scroll_y` は `[0, content_h - viewport_h]` へクランプ
3. viewport からはみ出す行は描かない(クリップ)。**pane 総高は層数によらず一定**(実機で崩れた欠陥の反対仕様)
4. 横(時間)パン・zoom は**行ヘッダ列を動かさない**

## 置き場(3分類)
- **Document**: 層・timing・parent・name・hidden・グループ所属・isolate/frozen・solo/locked(m/s/l は LayerAttrs 追加、裁定119)
- **Session**(undo 対象外): 選択・playhead(既存)+ **scroll_y・zoom/view_span・fold 開閉**(本決定 — ui-score-model §6 の類推を明文化。裁定46/107/119 と同族)
- **tokens**: 行高・帯高・**行ヘッダ列幅(RAIL_W)** — Ableton 実測に無い値なので初期値は導出とし、fixture 窓で利用者が調整して確定する(裁定117 の運用)
- bar の縦位置 = packing 結果、所有者なし・毎フレーム再計算(ui-score-model §2)

## 時間軸
zoom はカーソル下の時刻を保つ(M18、先例 7/7 必須)。Fit あり。パン/zoom は frame 座標に触れない読み取り view state。マーカー実装済み、ループ区間は標準。

## 第2波の切片(判断領域別・重さ均等)
1. 領域モデル(scrollable 化+不変量の負例 fixture)/ 2. Session 拡張(scroll/zoom/fold)/ 3. 行ヘッダ列(RAIL+m/s/l — store の solo/locked 追加込み)/ 4. グループ/fold(store の Group entity、isolate UI は後日)/ 5. zoom/Fit / 6. ループ帯。
共通柵: **「層数を増やしても pane 総高が不変」テスト1本を全切片が共有**

## 未決のまま残す
m/s/l の視覚表現の細部(アイコン位置・グレーアウト)/ 複数選択への Session 型拡張(M6 側の判断)
