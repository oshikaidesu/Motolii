# 裁定171 — ゼロコピーの合流設計(M4 の縫い目): engine を iced の device へ遅延合流

日付: 2026-08-22 / 状態: **決定** / 起点: 利用者裁定「もうゼロコピーまで行こう」。前提= 裁定170(wgpu 29 統一・M01/M2 済み・M3 走行中)

## 1. 問題の形

- iced の `wgpu::Device`/`Queue` は **shader widget の `Pipeline::new` が初めて呼ばれる時**まで外から手に入らない(compositor のフィールドは private、ω 実測)
- 一方 Shell は起動時に engine を持つ必要がある(`--screenshot`・テスト・export は**窓なし**で動く)
- つまり「起動時に1回だけ headless で建てる」現行と「device は窓の初回描画で届く」が衝突する

## 2. 決定 — 遅延合流+headless 恒久フォールバック

1. **wgpu 29 の `Device`/`Queue` は clone 可能なハンドル** — `StagePresenterPipeline::new(device, queue, format)` で clone を取り、shell へ渡す。`Pipeline::new` は `Message` を出せないので、**受け渡しの口を1本**(`OnceLock` 系の静的チャネル。wrapper-over-hack の型 — iced の内側へは触らない)。次の update tick で Shell が読む
2. Shell は受領時に **`Engine::with_device(device, queue)`(新設、`Compositor::with_device` を使う)で engine を作り直す**。切替は起動直後の1回だけ・Document 側キャッシュは revision 機構で自然再構築。切替前の数フレームは現行 readback 経路がそのまま描く(視覚断絶なし)
3. **headless 経路は撤去しない — 恒久フォールバック**: `--screenshot`・テスト・窓なし export は従来どおり `Engine::new()`(headless)。裁定166 の「器具・export 経路不変」を構造で保つ。export の CPU 読み戻しは共有 device 上でも従来どおり動く(export は CPU バイトが要るので readback が本務)
4. **共有 device 化後の Stage 経路**: engine の描画結果は **BL1b で fork に開けた `main_target` accessor**(裁定161)から GPU texture として取り、`RenderedFrame` に Arc で保持 → presenter の `Primitive::render(encoder, target_view)` で**直接 blit**。CPU readback・`write_texture`・`presenter_rgba` は Stage 表示経路から消える(export/screenshot 用の `rgba` 読み戻しは「要求された時だけ」へ遅延化 — 毎フレームやらない)
5. **順序保証**: engine の submit と iced の描画は**同一 queue** — submission 順で「書いてから読む」が構造的に成立(fence 不要)。市松合成は当面 CPU 合成のまま(表示用 RGBA が無くなるため、市松 ON の時だけ readback 経路へ落ちる — 市松は静的検分モードなので性能非目標、doc 明記)

## 3. 受入条件(M4 レーンへ)

- (a) 再生中の Stage 経路で **readback 呼び出し回数 0**(metrics に presenter_blit カウンタ追加・red 先行)
- (b) `--screenshot` PNG バイト不変(headless フォールバックの証明)
- (c) export 経路のバイト不変(golden)
- (d) 実窓: 起動→数フレームで合流が起き、絵の断絶・ちらつきがない(supervisor 実画面検分)+ fps 実測(readback 撤去の効果測定)
- (e) 市松 ON で従来どおりの絵(フォールバック経路の生存証明)
- 最終審判 = 利用者実窓

## 4. 却下・保留

- 起動時から iced device を待つ(engine 遅延構築のみ・headless 廃止)— 器具が壊れるため却下
- 2 engine 並走(headless+shared 常時)— VRAM/デコードキャッシュ二重化のため却下
- 市松の GPU 合成化 — 別玉(このレーンの NON-GOAL)
