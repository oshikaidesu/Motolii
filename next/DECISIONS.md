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
| 26 | 2026-08-20 | Stage は **iced の `shader::Primitive` 経由で iced の device の上に re_renderer を建てる**(= 本当の埋め込み)。CPU 読み戻しは export と試験だけに使う。根拠: 一次資料(pin 済み fork `wgpu/src/primitive.rs:14-64`)で `prepare(.., device: &wgpu::Device, queue: &wgpu::Queue, ..)` と `render(.., encoder: &mut CommandEncoder, target: &TextureView, ..)` が生の wgpu を渡すことを確認 — iced の拡張点は **toolkit の層ではなく wgpu の層**にある。読み戻しは 1080p で 1.7ms かかる |
| 27 | 2026-08-20 | ~~別 device でも byte 一致するので preview(iced の device)と export(headless)が別 device でも背骨2は崩れない~~ → **主張が過大だった**(同日の敵対的レビュー)。実測したのは `Compositor::headless()` を2回呼んだだけで、**iced の device は1度も通していない**。言えるのは「同じ adapter・同じ limits・同じ RenderConfig の device 2個なら一致する」まで。iced の device を跨ぐ一致は**未検証**で、裁定26 を実装する時に測る |
| 28 | 2026-08-20 | `LayerSource::Media` は **動画も静止画も同じ variant**。素材種で経路を分けない(分けると片方だけ直る欠陥が生まれる — 初回タッチ観察の再発) |
| 29 | 2026-08-20 | 素材の大きさは Document が持たない。probe が決め、engine が「track も declared も無い軸」だけを実寸で埋める。AE の「キーを打っていない property は静止値」の延長 |
| 30 | 2026-08-20 | 旧 `motolii-export`(913行)は移植しない。大半が graph / plugin 機構で、評価経路が1本になった今は要らない。移すのは**機構ではなく意味**(報告=現物 / 中断で残骸なし / 音声は後段 mux)。新 export は `Engine::render_frame` を回すだけの薄い口にする |
| 31 | 2026-08-20 | engine が抱えるフレーム texture は**上限つき**(8枚)。3〜5分の MV は 5,400〜9,000フレームあり、溜め込むと約27GBでメモリが死ぬ。順次走査で要るのは直近の数枚だけ。試験 `long_export_does_not_accumulate_frames` が上限を守らせる |
| 32 | 2026-08-20 | 時刻⇄フレームの写像は `motolii-core` の正準口のみ。柵は**パターン列挙をやめ**、「fps と f64/f32 が同じ式に出てくること」自体を禁じる(`tm4_no_float_frame_math.rs`)。旧の5文字列 grep は engine の f64 換算を素通りさせた |
| 33 | 2026-08-20 | `CompSpec` は `motolii-core` に置く。`motolii-export` が `motolii-compositor` を引くと第二の合成器を建てられるので、**型を出して依存を切る**。柵は `motolii-export/tests/fence.rs` が依存グラフを見る |
| 34 | 2026-08-20 | `motolii-store` の marker は `owns:`。`wraps:` を名乗った crate の中に `owns:` の中身(`fingerprint.rs` / `resolve`)が入ると、crate の根しか見ない `check.sh` が空振りする |
| 35 | 2026-08-20 | `PropertyId` は layer 自身の component 名(`meta` / `present`)を予約語として弾く。弾かないと `PropertyId::new("meta")` が素材と重ね順を上書きする |
| 36 | 2026-08-20 | 重ね順の型は上流の `re_renderer::DepthOffset`(`i16`)に合わせる。`i32` のまま渡すと 32768 以上で符号が反転し、CPU の並べ替えと GPU の前後関係が食い違う |
| 37 | 2026-08-20 | 読み口は「無い」と「読めない」を区別する(`Result<Option<T>>`)。同義にすると壊れた Document が静かに既定値へ落ち、利用者には「値が勝手に戻った」としか見えない(M13) |
| 38 | 2026-08-20 | 素材の外の時刻はその layer だけ描かない(フレーム全体を落とさない)。`nb_frames` を見る。M4 と M16 の両方に効く |
| 39 | 2026-08-20 | **スクラッチは最低限・保守はしたくない**(利用者裁定)。移植したが `next/` から未参照のコードは落とす — 抱えると保守対象になり、`check.sh` が `owns:` の重さとして数えて自前実装の量を偽る。要る日に旧 workspace から持ってくればよい。第1弾: `motolii-core` の camera / canonical / time_map / quality = 1,256行 |
| 40 | 2026-08-20 | comp の設定(解像度・fps・尺)は **Document が持つ**(`Composition`)。以前は `render_frame(view, t, comp)` と `ExportJob { comp, fps }` が別々に持ち、**preview と export が違う入力を渡せた**。上流の `set_recording_property` は `TimePoint::STATIC` で undo が効かないので、layer と同じ `edit` timeline 上の普通の entity として置く(新しい機構を足さない) |
| 41 | 2026-08-20 | 置き方は `motolii-core::LayerPlacement` を store と合成器で**共有**する。並べ直すと property を1つ足すたびに6箇所を触ることになり、旧 `inspector_model.rs` が3世代になった構造の1世代目になる |
| 42 | 2026-08-20 | 変化検出は `Document::revision()`(store 世代 + edit 位置)。`EntityDb::generation` だけでは undo/redo を捉えられず、front が `last_edit_head` を自分で持つ入口になる |
| 43 | 2026-08-20 | **「保守をしたくない」を軸に格上げ**(利用者裁定)。自前で持つコードは資産ではなく負債で、薄いラッパーであることはその負債を最小にするための手段にすぎない。禁じるのは6つ: 上流にある物を書かない / 使われていない物を置かない / 使う分だけ移植する / 抽象を先に作らない / 器具と台帳を増やさない / 「一時的に」を作らない。`check.sh` が毎回 `owns:` の総行数を出し、**それは下がるべき数字**として扱う |
| 44 | 2026-08-20 | **裁定26 を撤回**。Stage は iced の device に re_renderer を建てるのではなく、**CPU 経路**(合成結果の RGBA を `image` widget へ渡す)にする。軸4「保守をしたくない」で計算が変わった: GPU 共有は (a) iced が `Primitive::prepare` に adapter を渡さないので instance を作り直して adapter を選び直す迂回が要る (b) `max_bind_groups` の床という fork seam を1本抱え続ける。CPU 経路の代償は読み戻し 1.7ms/1080p(preview 解像度ならその 1/4)で、**fork を1本減らせる**方を採る |
| 45 | 2026-08-20 | **iced は上流をそのまま引く**(fork しない)。裁定44 で seam 2 が要らなくなり、seam 1(web-sys の完全一致解除)も 0.14 では解決に出てこなかった。fork が1本減った |
| 46 | 2026-08-20 | front が持ってよい状態は `Session`(選択と再生位置)だけ。Document の写しは持たない。選択と再生位置は undo の対象ではない(rerun も選択は blueprint store の外)。**1箇所で持ち全 pane がそこを読む**ので M14 は満たされる |
| 47 | 2026-08-20 | `Document::mark_undo_floor()` — 起動直後や project を開いた直後の状態は編集ではないので戻せてはいけない。呼ばないと利用者が既定値を undo で消し、**Stage が理由もなく空になる**(実際に起きた) |
| 48 | 2026-08-20 | **1操作 = 1 undo** は `Document::apply_all`(複数 intent を1つの edit 刻みへ)で満たす。ドラッグは途中経過を pane が持ち確定の1件だけが intent なので元から1 undo — 畳む口が要るのは「本質的に複数 intent な1操作」だけ |
| 49 | 2026-08-20 | 落下は Message にして受ける(窓の event を直に処理しない)。運転席が窓を開けずに同じ道を通せる。winit は1ファイル1事象なので**描画要求を区切り**にして溜めた分をまとめて1操作にする |
| 50 | 2026-08-20 | **バックの完成を先に**(利用者裁定)。iced を採ったのは「バックができていれば UI は後から生えてくる」ため。よって UI の見栄えより、Document の意味の穴を先に塞ぐ |
| 51 | 2026-08-20 | layer は `LayerTiming { start, duration, source_in }` を持つ。**move / trim / split / 速度はすべてこの型の上に乗り、intent は `SetTiming` 1つ**。専用 intent を操作ごとに足さない |
| 52 | 2026-08-20 | 置いた時の尺 = **min(素材の尺, comp の残り)** は `LayerTiming::place` が持つ。**shell に書かせない** — 書かせると面ごとに違う置き方が生まれる(旧 workspace の import_seat と browser で起きた形) |
| 53 | 2026-08-20 | comp 時刻 → 素材フレームの写像は **Document が持つ**(`resolve` が `source_frame` まで解決して返す)。engine は時間の計算をしない — engine が別の写像を持つと時刻の正本が2本になる(2026-08-20 に一度やった失敗) |
