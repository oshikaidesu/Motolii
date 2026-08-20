# 検証済み事実(レーンは再検証しない)

発注書はこのファイルを必読に入れる。**ここにある事実の再検証にツール呼び出しを使わないこと**。
疑わしい場合は黙って再検証せず、終了報告に「KNOWN.md の X を疑う、理由」と書く(supervisor が裁定)。
1行1事実、日付と出典を必ず添える。古くなった行は消さず「(失効 YYYY-MM-DD: 理由)」を付ける。

## 上流・依存(2026-08-20 一次ソース確認済み)
- rerun(pin 483b855・上流とも)に**操作ギズモは無い**。"gizmo" は軸表示の doc comment のみ。別語彙(manipulate/dragger)でも無し
- rerun の**音声対応は未実装**(issue #2852/#5181 とも open)。音声・再生時計は自前
- rerun の再生時計(TimeControl)は wall-clock 駆動で**音声同期に構造的に効かない**
- re_renderer に **mipmap 自動生成は無い**(TODO のまま)→ preview 高速化は素材 proxy 一本(裁定21)
- `re_video::load_mp4_from_reader` は **moov だけ先読みのストリーミング対応**(裁定24 の理由(c)は誤りだった)。encode/mux は無し(理由(d)は真)
- `re_chunk_store::gc` / `EntityDb::drop_time_range` は**既存 API**(store 層、即呼べる)
- (失効 2026-08-20: 実測で覆った)~~alpha には fork 改造2箇所が必要~~ → **`blend_with_background: Premultiplied` の1行(fork 改造ゼロ)で readback に実 alpha が乗る**(実測: 空comp=[0,0,0,0]、半透明層 alpha=128 素通し。試験 `alpha_survives_the_composite_step`)。「2箇所」判定は非公開 main target 直読み経路の話で、公開 API 経路を見落としていた(探索の壁の3例目)。**残り: export が alpha 付きファイル(ProRes/PNG連番)を吐く経路と、shell プレビューの alpha 合成(市松)は未実装** — 「合成器が出せる」と「書き出しが吐く」は別問題
- fork = 上流ほぼ素+seam 13個(全部旧 egui 埋め込み向け=next/ には死蔵)。pin 後の上流250コミットに関連変更なし
- **iced の Theme/Style は色・境界・影のみ。寸法は持てない**(API 実測)。iced 0.14 の公式ホットリロードは実験的すぎて前提にしない(裁定117)
- iced_test 0.14 は動く。ただし **canvas と slider は Simulator から構造的に不可視**
- `transform-gizmo` crate は 3D 前提で skew 無し。旧 gizmo は Motolii 追加(コミット fd6f54ba で依存追加)であって rerun 由来ではない
- rerun viewer の selection panel には**型別 component editor registry**(re_component_ui)がある — Inspector の型の先例(コードは egui 層、引かない)
- pucker/bloat の極性ラベル「正で bloat」は**正しい**(lottie-web 実コード+AE/Illustrator 3系統一致、裁定110)
- tiny-skia の gradient は stops 空/1本/半径0 で panic しない。**非昇順 stops は自動ソートされない**(構築時ソート実装済み、裁定109)
- Ravel の ShellManipulator は 2D・toolkit 非依存だが**利用者裁定でギズモはスクラッチ**(2026-08-20、Ravel はダサいため。plan B=transform-gizmo+自前カメラ行列)

## レーン運用(実測済み)
- **worktree の base はほぼ必ず stale**。作業前に `git reset --hard claude/motolii-reset-handoff-bda7f3` を無条件で行う(確認に時間を使わない)
- stash 禁止(worktree 間で共有)/ Edit 直後の stale fingerprint は touch / CARGO_TARGET_DIR 共有禁止(後勝ち事故の実測あり)
- 時間予算試験2本(`edit_storm_with_the_real_track_type`・r2 `timeline_projection_fits_a_frame`)は**負荷で落ちるのが既知**。単独実行で緑なら自分の変更と無関係。予算は緩めない
- 一次ソースの取得結果は終了報告に URL/rev を書く(次のレーンが KNOWN 経由で再利用できるように)

## store の流儀(読む前にこれで足りる場合が多い)
- Intent の型は3種: **丸ごと置換**(`SetShapes`/`SetEffects`/`SetTextDocument` — 検証を write arm で)/ **read-modify-write**(`SetTiming`/`SetSource`/`SetOrder`)/ **Patch**(`LayerAttrsPatch` — 全フィールド Option、None=不変。丸ごと置換の黙戻り事故を型で禁止)
- 動く値は struct field にしない — **平坦 PropertyId の KeyframeTrack**(`mask.{id}.shape` / `text_range.{id}.selector.start` の形、裁定92)。トラックの有無=意味の有無(裁定20)
- id は専用 newtype(LayerId/MaskId/TextRangeId/EffectId/TextStyleId)。**採番は store 正本 `StoreView::next_layer_id()`(墓石込み)** — 現存最大+1 を自前計算しない
- `apply_all` は原子的(失敗でロールバック、裁定118)。削除=tombstone(`present=false`)。`RESERVED` 名と `Document::mark_undo_floor()` に注意
- 保存は `flattened()` が store に聞く(手列挙禁止、裁定108/118)。component 追加時は `flatten_fence.rs` が守る
- ファイル地図: mask.rs / marker.rs / attrs.rs / text.rs / effect.rs(store)、frame.rs+camera.rs(core)、timeline_pane.rs+tokens.rs(shell)

## 既知の穴(発見報告不要。直すのも別途裁定してから)
- bm / matte / ao は store にあるが**合成器が未消費**(地図 note に明記済み)
- `parent` の変換合成は未実装(循環検査のみ)
- near-plane より手前の層は透視でクリップ(裁定116)
- 負の Speed の source_frame は未クランプ(doc 明記済み)
- テキストのアニメータ transform に skew が無い(Lottie/Rive とも語彙なし、text.rs doc 明記)
- alpha 付き書き出し不可(裁定16。直す口は fork 2箇所と特定済み)
- eval の未使用 import 警告等の fmt ドリフトが数ファイルに既存

## 採番・報告プロトコル(統一)
- **レーンは DECISIONS.md に書かない**(番号衝突が実際に起きた)。設計判断は終了報告に列挙し、supervisor が採番する
- 地図(tsv)の自分の束の行は書き換えてよい。束の外の行は報告のみ
- 既知の穴・KNOWN 記載事実は報告に書かない(新発見だけを書く)

## 音声(2026-08-20 解析済み)
- 旧 `motolii-audio` 4,286行の内訳: wraps 743(symphonia/rubato/cpal — 維持)/ 必然scratch 1,451(mix/producer/program — Document型と不可分、**移植価値の本体**)/ 再発明 367(`ring.rs` — 上流 `rtrb` が既にある)/ テスト 1,194
- **rodio / kira は採らない** — cpal だけが生のコールバックタイムスタンプ(`OutputCallbackInfo::timestamp()`)を露出。高レベル crate はクロック所有権の契約(D4/D5)を守れない
- **M8 の音声クロック→playhead は旧 `motolii-transport` に設計済み・実働済み**(audio-clock-master: `frames_supplied` − `device_wait`、wall-clock 不使用、無音補填で論理位置を進めない)。移植元として名指し可
- **export の音声 mux は現 motolii-media で解決済み**(`mux_soundtrack` / `mux_mixed_pcm`)。音声束は PCM を作るだけでよい
- **iced×リアルタイム音声は nih-plug エコシステムで実証済み**(`nih_plug_iced` — VST3/CLAP プラグイン GUI adapter)。ただし nih-plug は「プラグインを作る側」でありホスト側には使えない。VST ホスティング自体は GOALS の除外のまま(利用者確認 2026-08-20)
- (失効 2026-08-20: 探索範囲がRust界隈に閉じていた — 利用者指摘)~~mix/program/MixProducer は上流に無い~~ → **同型エンジンは他界隈に実在する**: Tracktion Engine(C++/JUCE、GPL+商用、まさに time-based sequenced audio の高水準データモデル)/ GES(GStreamer、LGPL、Rust bindings有、音声+動画timelineでMotoliiのドメインに最も近い)/ MLT / Ardour。現時点の順位: 代替性 Tracktion★5・GES★4・MLT★4 / 現実性はいずれも★2-3(言語・巨大依存・ライセンスの侵入コスト)。**「owns ~950行」の結論は生き残る見込みだが、根拠は「無いから」ではなく「成熟実装と比較した上で侵入コストが自前を上回るから」へ書き換え**。最終判定はゲームcinematic/WebAudio/放送playoutの掃討後(旧判定: : Firewheel(BillyDM現行、最有力)が設計文書で timeline/シーケンサを非目標と明記 / dropseed は 0.0.0 placeholder / creek は「本番未使用の非mix経路」にしか当たらず条件付き(現構成は decode-then-RAM でディスクストリーミング自体をしていない — PcmCache は5分ステレオ≈110MBを全展開、将来の穴)/ fundsp・dasp は誤差レベル。**削減0行で確定**
- 音声の規律 crate 候補(owns削減でなく柵): `assert_no_alloc`(コールバック内 alloc 禁止の機械化)・`audio-thread-priority`(スレッド優先度 — 現状未設定という穴あり)
- **音声の調達調査 DONE(2026-08-20、全界隈掃討済み)**: owns ~950行(mix/program/MixProducer)の最終根拠 = 「無いから」ではなく「**産業内の同型実装(Tracktion/GES/MLT/Ardour/FMOD/Wwise/Unity/Unreal/CasparCG)を比較した上で、いずれも (a)ソース非公開 (b)ホスト不可分結合で移植コスト超過 (c)別問題領域、のどれかに落ちるため意図的に自作**」。定量の決め手: 最有力 GES は Rust バインディング層だけで 94k LOC(依存込み281k = owns の100〜300倍)+ GObject 伝播モデルが store/Intent と異質。再訪条件: Motolii が別目的で GStreamer に厚く依存した場合のみ。意味の先例として Unreal `-deterministicaudio` と Tone.js Transport は設計追認の傍証
- **KNOWN 運用規則の追加**: 「探したが無い」と書く時は**探索範囲を必ず併記**する(今夜2回の教訓: gizmo=語彙の壁、音声=界隈の壁。範囲の宣言なき「無い」は嘘になる)
- **iced 0.14 の image 同期アップロード予算は 2MiB**(実測 2026-08-20): それ以上の RGBA は背景スレッド行きになり、完了まで**何も描かれない**(= チラつきの真因。1080p フレーム 8.3MB は4倍超過)。対策: Handle は 1.5MB 以下へ縮小して同期経路に収める(柵テスト `render_pipeline_fence.rs` あり)。preview 縮小は裁定21(preview 1/2 既定)と整合。恒久解の候補は iced 上流修正 or GPU 埋め込み(裁定26)
- **timeline_projection probe の慢性超過の真因(2026-08-21 計測)**: (a) `track()` コストの97%が `serde_json` の KeyframeTrack 解析(~88µs/call — 裁定11「track まるごと1 component」の代償が投影側に出た形)。恒久解候補= revision 鍵の解析済み track キャッシュ(裁定は未起草) (b) **このマシンに旧 MotoliiRn プロセスが火曜から常駐**(累計2184 CPU分・70%持続)し load を汚染 — 利用者に終了を推奨(勝手に kill しない)
