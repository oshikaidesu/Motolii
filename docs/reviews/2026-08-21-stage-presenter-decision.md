# 裁定166 — Stage 提示の構造転換: image widget → shader widget 永続テクスチャ+再生 tick の vsync 整列

日付: 2026-08-21(深夜) / 状態: **決定** / 起点: 利用者実窓較正第1回「イージングがガタついてる。image widget は無理があったんじゃないか」

## 1. 事実(実測・一次資料)

1. **release ビルドでもガタつきは再現**(利用者確認 — debug ビルド疑いは除去済み)。構造の問題と裁定。
2. **image widget 経路には前科がある**: 2026-08-20 チラつき調査(`shell/motolii-shell/src/metrics.rs` ヘッダが一次資料)
   - `image::Handle::from_rgba` は呼ぶたび `Id::unique()` — 毎フレームが「新しいテクスチャ」
   - iced_wgpu は `MAX_SYNC_SIZE = 2MB` 超を**非同期アップロードし、その間 `draw_image` は何も描かない**(iced 自身の doc が「アニメで望ましくないチラつき」と明記)
   - 当時の柵 = `STAGE_HANDLE_SYNC_BUDGET_BYTES = 1_500_000`(`lib.rs:154`)への **auto 縮小 — 1080p comp は約 816×459 に落として引き伸ばし表示**(解像度が代償)
3. **再生 tick は OS スレッドの `sleep(16ms)`**(`transport.rs:129`)。30fps(33.33ms)と噛み合わず、フレーム表示時間が 32/48ms で揺れる 2-3-2 エイリアシング+sleep ドリフト。イージング(滑らかな動き)で最も目立つジャダーの型。
4. **device 共有は型レベルで不可能**(Cargo.lock 実測): iced_wgpu 0.14 → wgpu **27.0.1** / re_renderer fork → wgpu **29.0.4**。エンジンのテクスチャを iced に直接渡すゼロコピーは今は取れない。
5. **iced 0.14 に必要な口は両方ある**(registry ソース実測):
   - `iced_widget-0.14.2/src/shader.rs` — shader widget(workspace は `wgpu`+`advanced` feature 有効済み。wgpu 型は iced の re-export を使えば新規 dep 不要)
   - `iced_runtime-0.14.0/src/window.rs:208` `pub fn frames() -> Subscription<Instant>` — RedrawRequested(vsync)由来の tick

## 2. 決定

**(a) Stage の絵は `iced::widget::shader` の自前 Program で提示する**(`lib.rs:2413` の `image(frame.handle.clone())` を置換):

- **永続 `wgpu::Texture`**(comp 寸法変化時のみ再作成)+フレーム変化時のみ `queue.write_texture`(世代カウンタ比較 — 同一フレームの再描画で再アップロードしない)
- **フル解像度復帰**: 1.5MB 柵・`stage_auto_scale`・nearest 縮小は Stage 提示から撤去(非同期アップロードの「描かないフレーム」穴が存在しない経路なので柵の存在理由が消える)。resolution cap ½/¼(状態帯 μ)は「明示的な縮小」として意味が残る — Auto は 1.0 固定になる
- letterbox は既存の Contain 数学(`motolii-stage-pane` — screenshot.rs と共有の単一源)を使う。2箇所目を作らない
- 市松合成・observation/checkerboard の意味・export/screenshot 経路は**不変**

**(b) 再生 tick は `iced::window::frames()` へ**: `transport::tick_subscription`(sleep 16ms スレッド)を置換。「再生中のみ購読」分岐(`lib.rs:600`)と PlaybackTick の意味(位置は wall-clock 由来、tick は通知)は不変。表示リズムが vsync に整列し 2-3-2 ビートが構造で消える。

## 3. 却下案

- **fork の wgpu を 27 へ下げて device 共有(ゼロコピー)**: fork 差分の膨張・rev pin 全崩し。readback+upload は Apple Silicon UMA では 30fps で余裕(~500MB/s 級のメモリ交通)であり、必要が実測されたら再訪
- **image widget のままフル解像度**: 非同期アップロードの draw しない穴が残る(前科の再演)
- **tick の絶対スケジューリング補正(sleep_until)**: `frames()` が実在するので不要。fallback として記録のみ
- 本裁定は wrapper-over-hack(2026-08-18)の適用第3例: 迂回(縮小柵)を重ねるのをやめ、境界に素直な口(iced 描画界面への GPU テクスチャの口)を1本作る

## 4. 受入条件(oracle — red 先行)

- (a) 再生 tick N 回で `metrics::handle_creations()` が増えない(現状 red: 毎フレーム +1)
- (b) presenter へ渡る寸法 == comp 寸法(fixture で 1920×1080。現状 red: 816×459)
- (c) `--fixture --screenshot` PNG バイト不変(器具経路は `frame_rgba()` でこの裁定の対象外 — 変わったら混線の証拠)
- (d) shell suite 全緑
- **最終審判 = 利用者の実窓合否**(イージングの滑らかさ — S 較正データを兼ねる)

## 5. 既知の残り

- readback(engine→CPU)は残る(device 分離の帰結)。~~将来 iced が wgpu 29 系へ追随したら再訪~~ → **訂正(2026-08-21 深夜・利用者指摘→一次資料確認)**: iced master(0.15.0-dev)は既に `wgpu = "29"` を宣言(raw Cargo.toml 実測)— re_renderer fork の 29.0.4 と同一線で解決可能で、**型障壁は master では既に無い**。残る障壁は wgpu でなく (1) next/ の pin が crates.io 0.14 安定線(0.15-dev への破壊的 API 追随が全 pane に及ぶ) (2) 検分器具 iced_test 0.14 線の tester 0.15-dev への揃え。手元に 0.15.0-dev 線の fork(tester/test rev)が既存のため、rerun と同型の rev pin+seam 台帳で載る余地あり — ゼロコピー spike は「待ち」でなく「発注可能な玉」へ格上げ
- `frames()` は window が occluded の時に止まり得る(winit の RedrawRequested 依存)— 再生位置は wall-clock 由来なので絵が止まっても時間は正しく進む(復帰時に追いつく)。KNOWN へ記載
