# 裁定ログ(追記のみ)

1裁定1行。取り消し線を引かず、覆した時は新しい行を足して「(YYYY-MM-DD の N を覆す)」と書く。
理由が長い物だけ `../docs/reviews/` に置き、ここには結論を書く。

| # | 日付 | 裁定 |
|---|---|---|
| 1 | 2026-08-20 | ドリフトの累積を1度リセットし、軸を1本にする。旧 workspace は歴史証拠として残す |
| 2 | 2026-08-20 | Document の実体は rerun store。undo は `edit` timeline の時間移動で、自前の undo 機構を作らない |
| 3 | 2026-08-20 | rerun は `crates/store/*` と `re_renderer` だけ引く。viewer 層(egui)は引かない |
| 4 | 2026-08-20 | AE の意味(Layer/Transform/Keyframe/Effect)は `re_types_core` の custom component として Motolii 側に建てる。`re_types` を fork しない |
| 5 | 2026-08-20 | front は iced のみ。pane は store への query の投影で、独自の状態を持たない |
| 6 | 2026-08-20 | 拡張の口は trait 1本。「component を読んで値か画を書く」以外の口を足さない |
| 7 | 2026-08-20 | 規律は `wraps:` / `owns:` marker 1つだけ。リンク台帳・索引・リンク検査を新 workspace に持ち込まない |
| 8 | 2026-08-20 | Document は `comp` 軸に載らない。property track を `edit` 軸へまるごと1行で置き、comp 時間の値は Motolii の評価器が出す(R0-A 実測: `LatestAtQuery` が単一 timeline のみで2次元 query が書けない) |
| 9 | 2026-08-20 | R0 は常設試験として残す。rerun fork の rev を上げた時はこれを回す |
| 10 | 2026-08-20 | 移植は再実装より優先する。`motolii-core`(有理数時刻)と `motolii-eval`(keyframe 補間・bezier 分割)は旧 workspace からそのまま持ってきた。新しく書き直さない |
| 11 | 2026-08-20 | track は `KeyframeTrack` の serde 表現を **1つの component** に入れる。arrow schema へ割らない — 同じ意味の正本が2つになるため。代償は実測で 5.4倍(1000編集×300打点で 3.5MB → 18.8MB)、予算 64MB 内 |
| 12 | 2026-08-20 | 削除は tombstone(`present = false` の append)。`drop_entity_path` を使わない — undo で戻らなくなる |
| 13 | 2026-08-20 | 拡張の trait は**まだ作らない**。2つ目の利用者(compositor)が現れるまで待つ。口を先に決めると、決めた口に合わせて中身が歪む |
| 14 | 2026-08-20 | 合成は `re_renderer` 直叩き。layer = `TexturedRect` の板、重ね順 = `depth_offset`、不透明度 = `multiplicative_tint`、カメラは正射影 `TopLeftCornerAndExtendZ`(world 単位 = ピクセル・原点左上 = AE の comp 座標)。2026-08-11「direct `re_renderer` 禁止」の撤回を実装で確定した |
| 15 | 2026-08-20 | preview も export も `Compositor::render` 1本。同じ入力から byte 一致することを常設試験で縛る(第二経路が生まれたらここが落ちる) |
| 16 | 2026-08-20 | **alpha 付き書き出しは現経路では出せない**。`ScreenshotProcessor` は compositing 後を撮るので alpha が 255 へ潰れる。直し方は `ViewBuilder` の main target を composite 前に読む経路の追加(fork seam は要らない見込み)。限界は試験で固定済みなので、直った日に試験が落ちて気付ける |
| 17 | 2026-08-20 | headless GPU の instance / adapter 選択 / device limits は `re_renderer::device_caps` の物をそのまま使う。自前で limits を書かない(rerun の shader が要求する床とずれた時に原因が追えなくなる) |
| 18 | 2026-08-20 | 1フレームを出す経路は `Engine::render_frame(&StoreView, t, comp)` 1本。preview も export もこれを呼ぶ。「書き出し専用の速い道」を足さない |
| 19 | 2026-08-20 | 素材の種類は `LayerSource` の variant を足すことだけで増やす。動画・静止画・生成物が別経路を持たないようにする |
| 20 | 2026-08-20 | キーを打っていない property は静止値(位置0・不透明度1・大きさは素材のまま)。AE と同じ扱いで、track の有無が意味の有無と一致する |
| 21 | 2026-08-20 | **preview の速さは comp 解像度では買えない。素材側の proxy でしか買えない**。R1 実測(1080p・40枚): 等倍 40ms / comp だけ 1/2 にしても 16〜36ms(ぶれる)/ 素材を 1/4 にすると 5.9ms。律速は fragment の量ではなく **40枚ぶんの素材(332MB)を毎フレーム舐めるバンド幅**で、MSAA を切っても 9% しか変わらなかった。よって proxy(または mipmap)が preview の前提条件であり、「preview 品質」の口は comp ではなく**素材側**に付ける |
| 22 | 2026-08-20 | `motolii-gpu` は新 workspace に建てない。`ctx.rs` は裁定17で上流置換済み、`yuv.rs`(414行)と `transfer.rs`(192行)は `re_renderer` の YUV/readback 経路で置換できる。`resource_ledger.rs` は別の関心事なので必要になった時に単独で `owns:` を主張する |
| 23 | 2026-08-20 | YUV→RGB は **`re_renderer` の `SourceImageDataFormat::Yuv` に載せる**。ffmpeg `-pix_fmt yuv420p` の生バイトが上流の `Y_U_V420` レイアウトとそのまま一致する。自前 WGSL を書かない(色事故の動機ごと上流へ移る) |
| 24 | 2026-08-20 | decode/encode は **ffmpeg サイドカーを維持**する。`re_video` は既にグラフに居るが (a) MP4 のみ (b) 再生指向でフレーム正確ランダムアクセスではない (c) 全バイトをメモリに載せる (d) encode/mux を持たない — 書き出しの契約と噛み合わない。この4点が変わったら再裁定する |
| 25 | 2026-08-20 | 合否の定義は `GOALS.md` 1枚。ここに無い物を「足りない」と数えない(除外リストを含む) |
