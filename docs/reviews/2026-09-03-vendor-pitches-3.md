# 2026-09-03 売り込み 第 3 波(中小 11 社)+ 技術カンファレンス(委託の館)

利用者の依頼: 第 2 波([vendor-pitches-2](2026-09-03-vendor-pitches-2.md))の大手に続き、Cargo.toml の直接依存のうち未訪問の中小 11 社を同じ形で回す。続いて会場(技術カンファレンス)を「拡張は後、**委託**が先」の指標で歩き、自前 code の引き取り手になる出展社だけを呼ぶ(§委託の館)。全員読み取り専用、code も backlog も触っていない。宿題番号は [persona-backlog](2026-09-03-persona-backlog.md)。

## 要約(1 社 1 行)

| 社 | 一番の売り | 正直な限界 |
|---|---|---|
| winit | title bar の点(H7)と IME 位置(K7)は**既に弊社で賄っている**。⌥ 鍵の表 30 行は `set_option_as_alt` で消える(WN1)。⌘ の張り付きは `ModifiersChanged` を写せば消える(WN2)。Rotation / DoubleTap gesture は捨てられている(WN3) | proxy icon(`set_represented_file`)は無い。PanGesture は macOS 非対応 |
| rubato | `SincFixedIn` の可変比で A1「掴んで動かす速さで鳴る」の土台 ≒25 行(RB1)。convert.rs は既に chunk 化されていて stream 化は今の code のまま(RB3) | 逆再生は比で表せない。pitch 保持は無い。mixer の `lerp_stereo` の置き換え口は無い |
| rfd | ⌘S の Save panel が親無し(RF1)、閉じる時の Save が独立窓(RF2)、3 択の並びが TextEdit と逆(RF3)— 計 ≒8 行で H5・H6 が閉じる | `allowedContentTypes` 等の panel の細部は無い。`x.mp4.mp4` は弊社の口では直せない |
| tiny-skia | 縁取りの 2 度焼きは同じ Pixmap に stroke → fill で 1 回(TS1)。**CV2 の縁の黒ずみの根は sRGB のまま lerp する 0.11**、0.12 の `Paint.colorspace` 1 行で消える(TS5)。glyph 単位の Mask で S7 の不透明度(TS4) | blur / glow は無い。link しているのは 0.11.4 |
| kurbo | CSS bezier の x→t 53 行、split の de Casteljau 40 行、trim の弧長 70 行が消える(KU1・KU3・KU5)。F14 の等速(roving)は `arclen` / `inv_arclen`(KU2) | motolii-doc への依存 1 本が要る(裁定)。`solve_t_for_x` は `pub(crate)`。整列・分布は売る物無し |
| waveform-data | A8 ステレオは `split_channels: true`(WF1)。zoom は `slice` + `resample(Width)`(WF3) | dB scale・FFT・進捗は無い。`WaveformBuilder` という型は無い(依頼文の誤り) |
| rtrb | AU1 の「seek 世代」は consumer 側の `read_chunk(slots()) → commit_all()` で一括に捨てる(RT1)。4096 の根拠は `2n + 1024`(RT3) | device 死活は検知できない(RT2)。寸法の指針は README に無い |
| Apache Arrow | **判定: 今のまま JSON でよい**。SE3 で欠けているのは番号だけ — `motolii.SchemaVersion` を static component 1 つ(AR1)。AE5 の UndoLabel は Utf8 で即(AR5) | cast は field 数不変が条件。Union の evolution は不可。serde の default/alias 18 箇所の方が手数が少ない |
| Rive text 仕様 | `coverageAt(t)` の実装が cpp に在る(Lottie は文だけ)、GC4/LD6 の glyph 単位の重みは ≒20 行で写せる(RV1)。lottie-coverage.tsv:713 の「3 値」は誤記(4 値) | reference/rive-text-defs を参照する code は 0 件(空手形、RV7)。縦書き・ルビは Rive にも無い |
| proptest | 嵐は既に弊社で回っているが `max_shrink_iters: 128` で 78 手の seed が縮まない(PT1)。Document の不変条件(undo∘redo、循環無し、split の和)を Strategy 1 本で(PT3) | state machine / derive は別 crate。oracle は弊社に無い |
| wgpu | 窓の device は `Limits::default()`(8192)で **Stage と書き出しの limits が既に食い違う**(WG6、4K×3 で落ちる)。GPU 時間は `TIMESTAMP_QUERY` で pass 単位(WG1)。書き出しは GPU と ffmpeg を重ねられる(WG2) | Metal に GPU memory の残量問い合わせは無い。E13 の予算は自前の算術 |

## 横断で見えた物

- **「既に弊社で賄っている」が 3 件**: winit の title bar の点と IME 位置、proptest の嵐と shrink、wgpu の readback padding。backlog の H7・K7 は済んでいた可能性があり、Z13 は設定値 1 つの話だった。
- **CV2 の根が変わった**: 第 2 波までは「乗算済み sRGB を非乗算として linear 化」と読んでいたが、tiny-skia 営業は「0.11 が sRGB のまま AA の縁を lerp する」を根と指し、0.12 の `colorspace` 1 行で消えると言う。実窓で縁を見て決める。
- **依頼文の誤りを 2 件正した**: waveform-data に `WaveformBuilder` は無い、Rive の units は 3 値でなく 4 値。
- **裁定待ち**: kurbo を motolii-doc の直接依存にするか(≒160 行減)、tiny-skia を 0.12 へ上げるか、wgpu の limits を adapter 値にするか(WG6、数行だが前提)。

---

## winit(0.31.0-beta.2)

前提(裏を取った所): `$A` = winit-appkit-0.31.0-beta.2/src、`$C` = winit-core-0.31.0-beta.2/src、`$B` = blitz-shell/src。Motolii の `Windows::window_event`(host.rs:519-698)は **全 WindowEvent を `inner.window_event`(host.rs:693)より先に見る**ので、blitz-shell が no-op にしている物(`$B/window.rs:846-855`)も弊社の event は Motolii に届いている。既に使い切っている物: PinchGesture(host.rs:589-617)、DragEntered/Left/Dropped(host.rs:618-649)、Focused(false)(host.rs:687-692)、`set_document_edited`(host.rs:419-423 → `$A/window_delegate.rs:1991`)、`request_ime_update` + cursor_area(host.rs:198-235 → `$A/window_delegate.rs:1677-1715`)。**title bar の点(H7)と IME 候補窓の位置(K7)は弊社で既に賄っており、売る物は無い。** 「M8 の OS drop が死んでいる」は dioxus-dnd の FileDropZone の話で、弊社の経路(`$A/window_delegate.rs:349-375, 410-436`、`registerForDraggedTypes` :747)は host.rs:618 に生きている。M8 自体(Browser→Stage の窓内 drag)は pointer の話で弊社の範囲外。

### 今日、既存 API で賄える物

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| WN1 | ⌥ 付きの鍵は logical が変わる(⌥[ → "“")ので keymap.rs:495-502 で physical code に戻し、`code_to_char` の表(keymap.rs:553〜)で救っている | `WindowExtMacOS::set_option_as_alt(OptionAsAlt::Both)`(`$A/lib.rs:171-176, 598-611`、`window_delegate.rs:1995`)— 生の字 + Alt で届く。欄が開いている間は `None` に戻す(place_ime が既に FIELD の有無を見ている host.rs:204) | 表 ≒30 行減。欄の外だけに限れば diacritics は壊れない |
| WN2 | 修飾の保持は keymap.rs:402-430 の `HELD` を打鍵から自前で積む。⌘Tab で離れると up が来ない | 弊社は `windowDidResignKey` で ModifiersChanged を合成して零に戻す(`$A/window_delegate.rs:193-205`、`view.rs:901-905`)。host.rs の window_event で `WindowEvent::ModifiersChanged` を HELD に写す(blitz も同じ物を `$B/window.rs:642-644` で保持) | ⌘ が張り付く事故の芽を消す。窓で未確認、事故の再現も未 |
| WN3 | RotationGesture・DoubleTapGesture は捨てている(host.rs は Pinch だけ) | `rotateWithEvent:` → `RotationGesture { delta: 度, phase }`(`$A/view.rs:741-758`)、`smartMagnifyWithEvent:` → `DoubleTapGesture`(`view.rs:732-739`)。gesture の前に `mouse_motion` を出すので host.cursor(host.rs:566-569)は新鮮(`view.rs:570-577`) | Stage の回転(層/orbit)と 2 本指ダブルタップで Fit に戻す。Pinch と同じ翻訳 ≒25 行。`phase`(Started/Ended)を捨てているのは Pinch も同じ |
| WN4 | 覆い(app.rs:211-244)は「Onto the Desk adds images…」と静的。位置は落とした瞬間だけ `drop_role_at`(host.rs:634-636) | `DragMoved { position }`(`$A/window_delegate.rs:388-397`、`$C/event.rs:88-94`)を host.rs で拾い、`drop_role_at`(keys.rs:82)を hover 中にも回す | 覆いが今の指の下の役目を言う。≒10 行 |
| WN5 | dark 固定(H18)。`set_theme` の呼びは無く、styles.css に `prefers-color-scheme` も無い | 初期値は blitz が `window.theme()` で読む(`$B/window.rs:181`、`$A/window_delegate.rs:1799-1812`)、変化は `effectiveAppearance` の KVO → ThemeChanged(`window_delegate.rs:475-508` → `$B/window.rs:633-637`)。「Appearance: System / Light / Dark」の設定は `Window::set_theme(None/Some)`(`window_delegate.rs:1814-1816` = `setAppearance`)か `WindowAttributes::with_theme`(`$C/window.rs:300`)。Some を入れると ThemeChanged は黙る(:497-499、仕様) | 弊社側 0 行、設定 1 読み + 窓ごと 1 呼び(host.rs:295-331 の window() に)。見た目は B10 の @media |
| WN6 | H22 scale_factor | `ScaleFactorChanged { surface_size_writer }`(`$A/window_delegate.rs:898-915`)は blitz が `set_hidpi_scale`(`$B/window.rs:629-632`)。window_frame.rs:21-28 は手で割っている | 弊社側 0 行。要実窓は変わらず |
| WN7 | Export finished(output.rs:304-305)は窓に文字だけ | `Window::request_user_attention(Some(Informational))`(`$C/window.rs:1258`、`$A/window_delegate.rs:1735`)— 別 app に居る時だけ Dock が跳ねる | 1 行 |
| WN8 | 別窓(host.rs:163, 472-490)は独立の NSWindow | `with_tabbing_identifier`(`$A/lib.rs:412`)/ `select_next_tab`(`window_delegate.rs:1948-1960`)で同じ tab 群に | 窓の tab 化。H14 の menu は解決しない |
| WN9 | fullscreen(H4) | native は `set_fullscreen(Some(Borderless(None)))`(`$C/window.rs:1007`、`$A/window_delegate.rs:1416`)、space を作らない pre-Lion 式は `set_simple_fullscreen`(`$A/lib.rs:104-116`、`window_delegate.rs:1856-1900`、native 中は false を返す) | 再生の全画面下見に simple 側。menu の「Enter Full Screen」は AP1 の Window menu |

検討のみ(合否は利用者): `with_titlebar_transparent` / `with_fullsize_content_view` / `with_movable_by_window_background`(`$A/lib.rs:349-384`、適用 `window_delegate.rs:594-601, 654-674`)。blitz は `safe_area` で内容を寄せる(`$B/window.rs:608, 622-625`)ので、fullsize にした時の inset は窓で確認が要る。

### 弊社に無い物(正直に)

- **proxy icon / `set_represented_file`**: 無い。`window_delegate.rs:1666-1674` に「いずれ `with_represented_file` で」と注釈があるだけ。今すぐ要るなら raw-window-handle(rfd の set_parent が使う host.rs:664 と同じ口)から NSView → `NSWindow::setRepresentedURL` を objc2 で直に。
- **PanGesture は macOS 非対応**(`$C/event.rs:284-288`、iOS / Wayland のみ)。2 本指の pan は MouseWheel の PixelDelta で来る。
- NSMainMenu・app delegate・NSDocument(AP1〜AP3 の領分)、窓内 drag(M8)、`set_window_icon` は mac で no-op(`window_delegate.rs:1666`)。

### 上流(blitz)PR 待ち — 弊社の event は届いているが blitz-dom が意味にしていない物

| # | 内容 | Motolii 側 |
|---|---|---|
| WU1 | `ImeCapabilities::new()` に `with_cursor_area` が無い(`$B/lib.rs:119`)— 弊社側は `$C/window.rs:1886` に在る | place_ime 38 行(B4) |
| WU2 | Pinch/Rotation/DoubleTap に対応する `UiEvent` が blitz_traits に無い(`$B/window.rs:848-851`) | host.rs の Wheel 翻訳が残る(WN3 も同型) |
| WU3 | Focused(false) を blur にしない(`$B/window.rs:846`)— 弊社は `windowDidResignKey` で出している | host.rs:687-692(BU7) |
| WU4 | Drag* を DOM の drag event にしない(`$B/window.rs:852-855`) | dioxus-dnd の FileDropZone を使う時だけ要る。host.rs:618 の直配りで今は困っていない |

確認点(売りではない): S14 の「composing 中の Enter」は、IME が有効な間は弊社が KeyboardInput を出さない(`$C/window.rs:1112-1116` の契約、`$A/view.rs:316, 396`)。事故が出るなら place_ime が欄の切替の狭間で Disable している瞬間(host.rs:207-209, 231)で、弊社の event 順の問題ではない。

見込み: WN1〜WN4・WN7 で ≒40 行減・新規 ≒40 行、WN5 は設定 1 本、WN8・WN9 は利用者の合否次第。

## rubato(0.16.2)

`$RB` = rubato-0.16.2(registry src)。Motolii 側は `crates/motolii-render/src/audio/`(`$A`)と `src/ui/playback.rs`。今の使い方は `FftFixedIn` 1 本(resample.rs:2, 38-44)— 素材→48k(convert.rs:59-87)と 48k→device(producer.rs:146-160)の 2 箇所、どちらも固定比。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| RB1 | A1: ruler の drag は pointer move 毎に `clock.seek`(timeline_widget.rs:1037-1043)→ `state.session = None` で device を開き直す(playback.rs:288-299)。「掴んで動かす速さで鳴る」経路が無い。`FftFixedIn` は `set_resample_ratio` が常に `SyncNotAdjustable`($RB/src/synchro.rs:662-664) | producer.rs:147 の resampler を `SincFixedIn`(または `FastFixedIn`)に替える。比は `device/48000` を原点に `max_resample_ratio_relative`(構築時、asynchro_sinc.rs:252)で幅を取り、drag の速さ s に対し `set_resample_ratio_relative(1/s, ramp=true)`(lib.rs:264-275、>1 で遅く低く)。ramp は次 chunk 内で線形補間(asynchro_sinc.rs:360-365, 381-382 の `t_ratio_increment`)— 1024 frame ≒ 21 ms 単位で滑らか。遅延は `output_delay` = `sinc_len*ratio/2`(asynchro_sinc.rs:509-511、256 なら ≒2.7 ms)、Fast は 8*ratio/2 = 4 frame(asynchro_fast.rs:454-456)。AU1 の「stream を保つ」と組で、seek 毎の開き直しが消える | producer.rs へ ≒25 行 + 速さを渡す atomic 1 本。A1 の「鳴る」半分 |
| RB2 | 同上、逆向き(左へ掴む) | **比は正のみ**(`validate_ratios` asynchro_sinc.rs:229-235)。逆再生は mixer が frame を逆順に渡す形(producer.rs:251 は `playhead += need` の一方向、TimeMap も逆速度を拒む time_map.rs:8-9, 65-67)。sinc は対称なので逆順入力でも品質は同じ | Motolii 側 ≒15 行 |
| RB3 | A17: 生成中が無表示。波形の peak は既に別 thread(waveform.rs:85-113)だが、decode(decode.rs:75-104、丸ごと Vec)→ `to_canonical`(program.rs:267-268)が `from_view` の中で同期、しかも `sync_document` の state lock 下(playback.rs:184-200) | convert.rs:68-80 は既に 1024 frame の chunk loop + `process_partial`(resample.rs:106-113)— **stream 化は今日の code のまま**。symphonia の packet(AU4)をそのまま流せば「丸ごと 2 本目の Vec」は要らない。進捗は `cursor/total`(convert.rs:68)で 1 行 | 弊社側は 0 行。「生成中」を消すのは thread 化 + `PcmCache` を追記可能にする Motolii 側(cache.rs:18 は `from_interleaved` 一発) |
| RB4 | 素材毎に `FftFixedIn::new` → 直後に `reset()`(convert.rs:62-63) | README「Creating a new resampler is an expensive task… call reset()」($RB/README.md 「Resampling a given audio clip」末尾)。同じ (src_rate→48k) の resampler を cache に持ち `reset()` で回す。今の `reset()` は new 直後で空振り | ≒8 行、44.1k 素材が多い作品で取り込みが軽くなる |
| RB5 | F15 speed: 層の速さは `TimeMap::constant_speed`(program.rs:207)→ mixer が小数位置を `lerp_stereo` で読む(mix.rs:276-286, 289-302)。2x は低域通過無しで折り返し、0.5x は線形補間の高域減衰 | 層ごとに `SincFixedIn::new(den/num, …)` を持ち、素材 stream → 比 den/num → 48k で mix へ。固定比なら sinc は構築時の比で切られる(`make_interpolator` に ratio を渡す asynchro_sinc.rs:268-274)ので 2x でも折り返し無し。AE 同様 pitch は変わる(pitch 保持は弊社に無い)。speed の keyframe track(resolve.rs:340-343、音側は未対応)は `set_resample_ratio(…, ramp=true)` を chunk 毎に呼べば 21 ms 段で追える。注意: **動的に比を下げる(速くする)時は構築時の filter のままなので数 % 超で artefact**(asynchro_sinc.rs:86-90)— 固定 2x は構築時に比を与えれば無問題、scrub の可変は Fast で割り切る | mix.rs の読みを「source 毎の stream」へ組み替える Motolii 側 ≒80 行。弊社は resampler だけ |
| RB6 | feature: Cargo.toml:106 `rubato = "0.16"`(既定 = `fft_resampler` on、`log` off) | `log` off は正しい(README「should be disabled for realtime」)。`fft_resampler` は `FftFixedIn` を使う限り必要。RB1/RB5 で全部 Sinc/Fast に寄せるなら `default-features = false` で realfft + rustfft の 2 crate が落ちる(Cargo.lock:6611-6616, 6788-6796、他に使い手無し)。ただし FFT 版は固定比では「considerably faster」(README「Synchronous resampling」)— convert.rs の丸ごと変換は FFT のまま残すのが妥当 | 0 語、または 2 crate 減(条件付き) |

### 弊社に無い物(正直に)

- **mixer の乱択読み(`lerp_stereo` mix.rs:289-302)の置き換え口は無い。** 弊社は chunk を流す設計(README 冒頭「process audio in chunks」)で、「位置 pos の値を返す」API は無い。`interp_cubic` 等は非公開(asynchro_fast.rs:163, 177 は `fn`、`interpolation` module は `pub` でない lib.rs:48-60)。lerp を良くしたいなら RB5 の組み替えが唯一の道で、`lerp_stereo` に cubic を足すのは Motolii の 10 行。
- **scrub の「どの区間を何 ms 鳴らすか」の判断**、seek 直後の無音詰め・fade は弊社の外(producer.rs 側)。
- **pitch 保持のタイムストレッチ**(phase vocoder / WSOLA)は無い。速さ = 音程が動く varispeed だけ。
- **逆再生**(RB2)は比では表せない。
- `process_partial` は入出力とも alloc する(lib.rs:140-141, 172-173)。producer.rs:217 の flush が毎回 alloc — audio callback ではなく producer thread なので実害は薄いが、realtime 厳密なら `process_partial_into_buffer` + 前確保。
- 比の上限は構築時固定(`max_resample_ratio_relative`)、大きいほど `output_frames_max`(asynchro_sinc.rs:499-503)の buffer が増える。scrub 0.1x〜10x なら 10 で chunk 1024 → 出力 ≒10k frame/ch、許容範囲。
- aarch64 の Neon は runtime 判定で使う(README「SIMD acceleration」、asynchro_sinc.rs:176-179)— 計測はしていない。

見込み: producer.rs へ ≒25 行(RB1)+ RB4 の 8 行で A1 の「鳴る」土台、feature 変更は 0 語。A17 と F15 の音は弊社が resampler を出すだけで、消えるのは Motolii 側の thread 化と source 毎 stream 化(≒100 行)が要る。

## rfd(0.17.2)

前提(裏を取った所): `rfd = "0.17"`(motolii/Cargo.toml:84、default features)。macOS backend は NSSavePanel / NSOpenPanel / NSAlert を objc2-app-kit 0.3 で直叩き(rfd Cargo.toml、src/backend/macos/)。`set_parent` は `HasWindowHandle + HasDisplayHandle`(file_dialog.rs:96)で、winit-core 0.31 の `dyn Window` が両方を実装(winit-core window.rs:1456, 1462)— blitz の窓は app.rs:254 / :488 の `try_consume_context::<Arc<dyn Window>>` で既に手元にある。sheet の実体は `beginSheetModalForWindow_completionHandler`(panel_ffi.rs:39・:72、message_dialog.rs:97・:147)。

### 今日、既存 API で賄える物

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| RF1 | ⌘S / ⇧⌘S の Save panel が `AsyncFileDialog::new()` に親無し(project.rs:38-42)。H6 の「Save は put_away が窓を持たず未」。実は非同期の親無しは弊社が `app.mainWindow()` か最初の窓に勝手に貼る(modal_future.rs:71-75)ので今も大抵 sheet だが、別窓(host.rs:340 `detached`)が main の時はそちらに貼り付く | `put_away` に `window: Option<Arc<dyn Window>>` を 1 つ足し、:38 を既存の `project::sheet(window.as_deref())`(project.rs:9-15、Import・Export・Open は既にこれ)に置き換える。呼び手 app.rs:1023・:1380・:1399 は `window.clone()` が同じ scope に在る(:1027 が渡している) | ≒5 行、H6 の残り |
| RF2 | 閉じる時の Save(host.rs:386-405)は同期の `rfd::FileDialog` に親無し → 同期路は親が無いと `runModal()` の独立窓(file_dialog.rs:255-266、panel_ffi.rs:66-77 は parent 有りの時だけ sheet)。直前の 3 択(host.rs:664)は sheet なので、Save を押した瞬間だけ窓から外れる | :391 に `.set_parent(&*view.window)`(:664 と同じ手、`self.inner.windows.get(&window_id)` を通す)。同期でも sheet + 入れ子 runModal になる(panel_ffi.rs:67-76) | 3 行 |
| RF3 | Save / Don't Save / Cancel を `YesNoCancelCustom("Save","Don't Save","Cancel")`(project.rs:90-94、host.rs:658-662)。NSAlert は button を**右から左**に置く(Apple doc: addButton(withTitle:))ので今の並びは [Cancel] … [Don't Save] [Save]。TextEdit は [Don't Save] … [Cancel] [Save] | 順を `("Save","Cancel","Don't Save")` に入れ替えるだけ。結果は label で照合している(project.rs:100-105、host.rs:667-672)ので他は変えない。既定 Return = 1 番目、"Cancel" = Escape、"Don't Save" = ⌘D は **AppKit が題名から自動で付ける**(同 doc、objc2 NSAlert.rs:135 の注釈)— 弊社は `keyEquivalent` を出していないが、この 3 語なら要らない | 0 行差、H5 の仕上げ |
| RF4 | D3 relink(素材が見つからない時に探す口)は未実装。取り込みは `fixture::admit_paths`(fixture.rs:1064)で生 path | `project::sheet(w).set_directory(old.parent()).set_title(format!("Locate {name}")).add_filter(kind, &[old_ext]).pick_file()`。`set_directory` → `setDirectoryURL`(panel_ffi.rs:123-138)、`set_title` → `setMessage`(:149-151、panel の説明行で窓題名ではない)、filter → `setAllowedFileTypes`(:107-121)。`FileHandle::path()`(file_handle/native.rs:136)で PathBuf | dialog 側 ≒6 行。「見つからない」の検出と Asset の差し替えは Motolii の Intent の仕事 |
| RF5 | 非同期と event loop の相性(既に Import・Export・Open が使用) | `ModalFuture` は AppKit の completion block で waker を起こす(modal_future.rs:54-66・:87-103)、`app.isRunning()` の間は sheet、外なら同期に落ちる(:79・:104-118)。app.rs:1423-1425 の「事象処理の外で開ける」注釈どおり、`dioxus_core::spawn` からなら窓は止まらない。同期 `show()` / `save_file()` は `runModal()` で入れ子の run loop(message_dialog.rs:101)— CloseRequested の中(host.rs:650)は async が使えないので同期で正しい | 変更なし |
| RF6 | default features(`xdg-portal`・`wayland`) | 両方 Linux 系の cfg 内(rfd Cargo.toml `target.'cfg(any(target_os = "linux" …))'`)、`gtk3` も同じ、`common-controls-v6` は windows-sys。macOS の build には 1 byte も入らない。`default-features = false` にしても本機は変わらない(Linux 配布を考える日の話) | 0 |

### 弊社に無い物(正直に)

- **NSAlert の button の並び・鍵の指定**: `keyEquivalent` / `tag` / 4 つ目以降 / suppression checkbox を出していない(message_dialog.rs:62-65 は `addButtonWithTitle` のみ)。RF3 の 3 語以外の題名にしたら鍵は消える。
- **Save panel の細部**: `setAllowedFileTypes` は deprecated で UTType(`allowedContentTypes`)・`allowsOtherFileTypes`・`extensionHidden`・`nameFieldLabel`・`prompt`(button の文言)・accessory view は無い。export_sheet.rs:135 の「filter を付けると x.mp4.mp4」はこの層の話で、弊社側の口では直せない — 拡張子を自分で足す今の形(project.rs:45-47、export_sheet.rs:146-148)が正道。
- **H10 auto_save・H9 Revert・NSDocument の版**: dialog の外(autosave.rs:29 が Motolii 自身で呼ぶ、AP 節の裁定どおり)。
- **副作用**: 毎回 `FocusManager`(utils/focus_manager.rs:18-24)が閉じた後に元の key window へ `makeKeyAndOrderFront`、`PolicyManager` は Prohibited の時だけ触る(policy_manager.rs:15-18)。`activate_cocoa_multithreading` が呼ぶたび NSThread を 1 本起こす(modal_future.rs:46)— 無害だが弊社の癖。

見込み: RF1〜RF3 で ≒8 行、H5・H6 が閉じる。RF4 は D3 の設計が決まってから 6 行。

## tiny-skia(0.12.0)

前置き(正直に): Motolii が link しているのは **0.11.4** です(`Cargo.toml:103` が `"0.11"`、`cargo tree -i tiny-skia@0.11.4` → motolii-doc のみ)。0.12.0 は `Cargo.lock:8070` に在りますが、`--target all` で見ると winit-wayland → sctk-adwaita 経由の Linux 専用で、mac の build には入りません。二重ではあるが実害は無し。ただし下の TS5 は 0.12 の機能なので、売るなら `"0.12"` へ上げる(差分は `RadialGradient::new` に start radius が 1 引数増えるだけ、raster.rs:101 に `0.0` を足す)。以下 `$T` = tiny-skia-0.11.4/src、`$T12` = 0.12.0/src。

使われ方の今: 弊社を呼ぶのは `crates/motolii-doc/src/vector/raster.rs` 1 file(fill_path:170、stroke_path:186、`s.join.into()`:178 で LineJoin は既に届く)。`group.rs:86 render_tree` は 1 Pixmap に leaf を順に描くので、複数の Shape を重ねる器も在る。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| TS1 | 縁取りの 2 度焼き: `engine/text.rs:115-131` が under(stroke×2)と over(fill)を **別 Pixmap** に焼き、`composite_over`(143-154)で手合成。join は `Stroke::default()` の Miter・miter_limit 4(vector.rs:229-230)— 「A」「V」の頂点で幅の 4 倍まで尖る | 同じ Pixmap に `stroke_path` → `fill_path` の順(SourceOver、`$T/painter.rs:327, 214`)。`render_tree(&[Leaf(under), Leaf(over)])` で今日の器に載る。`join: LineJoin::Round`(`$T`-path/stroker.rs:118)を text.rs:75 に 1 行。輪郭は `Command::Close` で `closed: true`(vector/text.rs:216-230)なので cap の欠けは無い | `composite_over` 12 行と Raster 1 枚が消える。GC3・LD6 の stroke_over_fill は同じ関数の順序の入れ替えだけ |
| TS2 | S3「幅 0 なら級数の 5%」は UI(`src/ui/color.rs:186-187`)で埋め、描画側は `text.rs:66` と `raster.rs:172` が `width > 0` で捨てる | 弊社は幅 0 を **hairline(1px 相当)** と解釈する(`$T/painter.rs:553 treat_as_hairline`)。0 を通すと細線が出て意図と違うので、今の「捨てる」は正しい。5% は UI ではなく `to_fill_stroke` で埋めると Lottie 書き出しと窓が一致する | 0 行(裁定の確認のみ) |
| TS3 | S8 runs / CT6 glyph 単位: `vector/text.rs:159-164` が全 glyph を 1 本の `contours` に畳む | 1 glyph = 1 `Path`(`PathBuilder`、raster.rs:14)。`draw()` は path 単位に fill/stroke を取るので、CT5 の `metadata` で style を引き、glyph ごとに `draw(pixmap, path, origin, fill_i, stroke_i, w)` を回すだけ。fill と stroke の色・幅・opacity は path ごとに独立 | raster.rs は無変更、engine/text.rs の loop ≒15 行。S8・S15 |
| TS4 | S7 Range Selector: model 在り(store/text.rs:171-176、`TextShape` 148-155)、描画無し | `Mask`(`$T/mask.rs:50 new`、`253 fill_path(path, rule, aa, transform)`)を glyph ごとに白で焼き、`data_mut()`(133)で selector の重み(0..255)を掛けて `fill_path(.., Some(&mask))` に渡す(painter.rs:126)。描画側は `mask_u8` 段(highp.rs:320)で coverage に乗る。glyph 単位の**不透明度**は弊社で済む | ≒25 行。位置・回転・級数の animate は Mask では無理 — TS3 の path に `Transform`(fill_path 第 4 引数)を glyph ごとに渡す方が本筋 |
| TS5 | CV2 の暗さ: `raster.rs:60 color_of` は sRGB の値をそのまま渡し、弊社 0.11 は色空間を知らない。fill(白)を stroke(黒)の上に AA で乗せる縁は **符号化 sRGB のまま lerp**(0.5 coverage → 128 ≒ 線形 21%)。これが縁の黒ずみの根。`texture.rs:39 unpremultiply` → shader の decode は正しく、根は弊社側 | 0.12.0 の `Paint.colorspace`(`$T12/painter.rs:56`、`ColorSpace::{Gamma2, SimpleSRGB, FullSRGBGamma}` color.rs:455-476)。`blitter.rs:93-111` が dst を expand → blend → compress するので、fill/stroke の縁が線形で混ざる。`paint_for`(raster.rs:76)に 1 行 | 版上げ + 2 行。実窓で縁を見る(CV2 の残り) |
| TS6 | premultiplied の一貫性: `Pixmap::take()`(raster.rs:194)= 乗算済み RGBA8、`Raster.premultiplied_rgba8` と一致。texture.rs は非乗算へ戻して `Rgba8Unorm`(device.rs:97)に上げる | 一致している。0.12 の `Pixmap::take_demultiplied`(`$T12/pixmap.rs:269`、`demultiply` は `/a + 0.5` の丸め color.rs:156)で `unpremultiply` の除算 8 行は消せるが、`bleed_edges`(texture.rs:53)は弊社に無いので残る | ≒8 行減 |
| TS7 | blend_preview.rs(129 行)の CPU 式 | `BlendMode` 全 29 種(`$T/blend_mode.rs:5-63`、W3C の 15 種 + Porter-Duff)。`fill_rect(.., Paint { blend_mode })`で 1×1 の Pixmap に焼けば札の色は出る。**ただし** 弊社 0.11 は sRGB のまま混ぜ、実物(blend.wgsl → vello の式、線形光)と CV4 の往復は Motolii 側に残る。依存は増えない(既に在る)が、行数は 129 → ≒60 で、VL8 と同じく「画素一致させたい時だけ」 | 売りは弱い。式を持たない裁定に合わせるなら可 |

**弊社に無い物(正直に)**: blur・drop shadow・glow(GC5)— `$T` 全体に blur / gaussian / shadow の語が無い。filter は無く、`Mask` を自分でぼかして `fill_rect` に渡す手はあるが、畳み込みは Motolii 側の自作になる。`vism/blur.wgsl` が正解のまま。色空間の変換(XYZ・広色域)も無し(`ColorSpace` doc に「gamma だけ」と明記)。縦書き・ルビは文字組みの話で弊社の外。

**見込み合計**: TS1・TS3・TS4 で engine/text.rs ≒40 行差し替え(削除 ≒15)、S7 の不透明度・S8・S15 が弊社の既存 API に乗る。TS5 は版上げ 1 語 + 2 行で CV2 の根を消す。

## kurbo(0.13.1、`$K` = registry の kurbo-0.13.1/src)

前提: 窓側(root `motolii`)は `peniko::kurbo` を既に使う(timeline_widget.rs:18、stage_widget.rs:15、ease_widget.rs:9)。契約側 `crates/motolii-doc` は glam + tiny-skia だけ(Cargo.toml:11・23)で kurbo は無い — 下の KU1〜KU3・KU7 は **motolii-doc への直接依存 1 本**(Cargo.lock:3771 に既に在るので compile は増えない)を要する。裁定は利用者。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| KU1 | eval/bezier.rs:1-53 が CSS 型 cubic-bezier の x→s を Newton 8 回 + 二分(EPS 1e-7)で自作、`sample`/`sample_derivative` も自前。track.rs:69・155 が呼ぶ | **`solve_t_for_x` 相当の公開 API は弊社に無い**(cubicbez.rs:466 `solve_monotonic_for_y` は `pub(crate)`)。代替は 2 つ: (a) `common::solve_cubic(-x, 3x1, 3x2−6x1, 3x1−3x2+1)`(common.rs:190、閉形式)で [0,1] の根を取る — 466-490 と同じ手順; (b) `solve_itp`(common.rs:676、二分と同等の頑健さで収束が速い)に `eval(t).x − x` を渡す。値は `CubicBez::eval`(cubicbez.rs:550) | 53 → ≒15 行。上流 PR「`solve_monotonic_for_y` を pub に」なら 1 行 |
| KU2 | F14: `SpatialTangent{out,in}`(track.rs:444)を track.rs:607-620 が `cubic_bezier_point(p0,p1,p2,p3,u)` で評価、**u は時間の ease 値** — AE の既定(空間キーは弧長で等速、roving)と違い、取っ手の長さで速さが変わる | `CubicBez::new(p0,p1,p2,p3)`(cubicbez.rs:59)+ `arclen(acc)`(:668、Gauss-Legendre 適応)+ `inv_arclen(s, acc)`(param_curve.rs:98-125、ITP)で「ease(u)×全長 → t」。Stage の線は `BezPath::curve_to` を vello に渡すだけ(`flatten` bezpath.rs:622 は当たり判定用の折線に)、掴みは `nearest(p, acc)`(cubicbez.rs:694、5 次の根)→ `Nearest{t, distance_sq}` | 評価 ≒10 行、UI の線と掴み ≒30 行。roving の数学は弊社 |
| KU3 | ⌘⇧D の切り口: track.rs:101-250 `split_at` が de Casteljau を手書き(l1/l2/l3/l12/l23、226-237)し CSS 箱へ再正規化、エラー分岐が 6 箇所。timeline_edit.rs:178 が呼ぶ | `ParamCurve::subsegment(0..s)` / `(s..1)`(cubicbez.rs:559、導関数で端点接線)、または `subdivide`(:571、半分固定)。CSS 箱への割り戻し(進捗・値で割る)は弊社に無いので Motolii に残る | ≒40 行減。0 除算判定(190-205)は残る |
| KU4 | F11 値グラフ: 縦軸の範囲を出す物が無い。Bezier の y1/y2 は [0,1] 外を許す(ease_model.rs:59・70、overshoot) | (時間, 値)の `CubicBez` に `ParamCurveExtrema::extrema`(cubicbez.rs:743、座標ごとの 2 次)/ `bounding_box`(param_curve.rs:214)。行き過ぎ込みの正確な上下端 | ≒8 行。Bounce/Elastic は式なので標本(ease_widget.rs:275 の 240 点)のまま |
| KU5 | Shape の trim: geom.rs:267-300 `segment_sample_lengths` / `t_at_length` が 24 分割の折線で弧長(ARC_SAMPLES、geom.rs:265)、ops.rs:662-678 `flatten_segments` と zigzag(ops.rs:130-140)が使う | `arclen` / `inv_arclen`(精度引数付き)。`bezier_point` / `bezier_tangent`(geom.rs:197-225)は `eval` / `deriv().eval`(cubicbez.rs:603) | geom.rs ≒70 行 → 0、精度が固定分割から許容誤差に |
| KU6 | Offset Paths: ops.rs:211-262 `offset_contour` が曲線を折線に落としてから辺を平行移動、開いた path は拒否(:220)、角は `join_corner`(:300) | `offset_cubic(c, d, tol, &mut BezPath)`(offset.rs:108)は **1 本の cubic** の平行曲線を曲線のまま返す(共線退化は呼ぶ前に線分へ、:104-106)。区間ごとに呼び、角の結合は今の `join_corner` を残す。両側に膨らませる用途なら `stroke(path, &Stroke, &opts, tol)`(stroke.rs:260、Join/Cap 込み)だが、それは線の輪郭で AE の Offset とは別物 | 曲線が曲線で残る(今は折線化で頂点が 24 倍)。行数は ±0、精度の話 |
| KU7 | ST16 整列・分布 | 弊社は `Rect::union`(rect.rs:198)/ `center`(:173)/ `width`(:107)だけ。分布(等間隔)は sort と割り算で弊社に無い。窓側は既に `peniko::kurbo::Rect` を持つ | 新規依存 0、売る物も殆ど無い(正直に) |

### 弊社に無い物(正直に)

- **CSS cubic-bezier の x→t 解法そのもの**(KU1 の通り。closed-form の `solve_cubic` か ITP で組む)。
- **ease_widget の取っ手の当たり**: ease_widget.rs:188 は取っ手を `Point::distance ≤ GRAB` で拾う — 取っ手は点なので `nearest` の出番は無い。曲線本体を掴む要望(F11 で出るかも)が来た時だけ `nearest`。ease_widget.rs:273-283 の 240 点折線は Bounce/Elastic も同じ経路で描くので、Bezier だけ `curve_to` 1 本に替える価値は薄い。
- **stage_widget の破線と timeline の菱形**: 既に弊社の `Stroke::with_dashes`(stroke.rs:150)と `Rect::from_center_size`(timeline_widget.rs:947)で正しい(pitches-2 vello 節と同じ結論)。
- **Shape の bbox**: raster.rs:54 が tiny-skia `compute_tight_bounds` に測らせている。`Shape::bounding_box`(bezpath.rs:1448)で置けるが定規が 2 つになるだけ。
- **`fit_to_bezpath` / `simplify_bezpath`**(fit.rs:165、simplify.rs:298): 今の Motolii に折線→曲線の当てはめを要る入口が無い(zigzag の出力は `build_point_type_vertices` ops.rs:167 で頂点列のまま)。F14 で手描きモーションパスを入れる時だけ。
- roving keyframe の **UI**(どのキーを roving にするかの状態)、値グラフの描画、AE の `KeyframeEase{speed,influence}` ↔ CSS 箱の変換 — 数学は上の `arclen` / `deriv` で組めるが、形は弊社に無い。

見込み合計: motolii-doc へ依存 1 本で ≒160 行減(KU1・KU3・KU5)+ F14 の等速と掴みの数学(KU2)+ F11 の縦軸(KU4)。KU6 は精度、KU7 は無し。

## waveform-data(0.1.1)

前置き(正直に): 弊社の口は `generate_waveform_data(&GenerateOptions)`(generator.rs:19-33, 57)と `WaveformData`(lib.rs:94)だけで、`WaveformBuilder` という型は在りません。Motolii は既に workspace 依存(Cargo.toml:109)で、waveform.rs:269-291 が生成と 2 倍刻みの `resample` を使っています。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| WF1 | A8: waveform.rs:254-268 が L/R を平均して mono 1 本にし `split_channels: false`(:273)、`columns()` は `channel(0)` 固定(:168) | `split_channels: true` で出力 ch 数 = 入力 ch 数(generator.rs:64-68)、ch ごとに (min,max) を積む(:113-124)。`resample` も ch を保つ(lib.rs:528-538)。読み出しは `channel(i)`(lib.rs:223-230)。Motolii 側は interleaved→planar の分離(mono 畳みの代わり)と `PeakColumn` に ch を持たせる形、描画は timeline_widget.rs:1434-1440 を 2 段にする | 弊社側 0 行、Motolii ≒25 行。resident は ch 倍(`resident_upper_bound` :67-83 も ch 掛け) |
| WF2 | A10: `normalize_peak`(waveform.rs:319-325)が線形。振幅 ±1 を行の 0.45 に写す(timeline_widget.rs:1427) | **弊社に無い**。dB / 対数の scale は無く、`amplitude_scale`(generator.rs:25)は線形 gain。`bits: 16`(:271)なので -90 dB まで階調は残る。`normalize_peak` の後に `20·log10(|v|)` を [-60,0] dB → [0,1] へ写す 1 関数(≒6 行)で済み、min/max の符号だけ保てば済む | Motolii ≒6 行、弊社 0 |
| WF3 | zoom: build_levels(:282-293)が 64→2 倍刻みを全段前計算し、`columns()`(:153-160)は「desired 以下の最大段」を選ぶ — 段の間の倍率では列が px より多く出る | `Resample::Width(f64)`(lib.rs:63)で「表示幅ちょうど」の列に落とせる。`slice(Slice::Time{start,end})`(lib.rs:70-83, 431)で見える範囲だけ切ってから `resample(Width(w))`。ただし下限は元の scale(`ZoomTooLow`、lib.rs:316, 350-353)なので BASE_SCALE=64 より寄れないのは今と同じ | 段 BTreeMap を捨てるなら ≒40 行減、paint ごとに resample が走る(範囲は slice 済み)。今の前計算で合っているなら据え置き |
| WF4 | IS5 audioFFT の texture | **弊社に無い**。弊社は (min,max) の peak だけで FFT は持たない(src に fft 無し)。ISF `audio` 入力(1 行 = 1 ch の生 sample)にも peak は合わない。`realfft 3.5.0` は Cargo.lock:6611 に rubato 経由で在るが直接依存ではない | `columns()` の使い回しは不可。A7/A8 は WF1 の列、FFT は別口 |
| WF5 | A17 生成中が無表示 | `generate_waveform_data` は一括(戻りまで進捗の口が無い、generator.rs:57)。`concat`(lib.rs:370)で区間ごとに生成→継ぎ足しなら進捗が出せる(要件: channels/rate/bits/scale 一致 :376-379) | Motolii 側 ≒20 行、弊社 0 |

弊社に無い物(正直に): dB scale(WF2)、FFT / spectrogram(WF4)、進捗 callback(WF5)、streaming 入力(全 sample を `&[f32]` で渡す前提、generator.rs:33)。

## rtrb(0.4.0)

前置き: 既に workspace 依存(Cargo.toml:108)。README は性能計測と開発手順で、**ring の寸法の指針は書いていません**。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| RT1 | AU1 seek 世代: playback.rs:272, 292-293 が「consumer 側に残った sample を消せない」理由で session を捨てる。ring.rs:18-22 は `pop()` を 1 sample ずつ | 消せるのは consumer だけ(SPSC)。callback で世代 atomic を読み、変わっていたら `read_chunk(consumer.slots())` → `commit_all()`(chunks.rs:342, 971)で一括に捨てる。producer 側の順は「pending を捨てる(producer.rs:171-182)→ 世代を進める → `producer.slots() == capacity`(lib.rs:364, 169)になるまで待つ → 新しい sample を push」。この待ちが無いと新 sample まで捨てる | ring.rs +6 行、producer.rs +8 行。残る古い音は device block 1 つ分(cpal の `BufferSize::Fixed` は AU1 のとおり) |
| RT2 | AU2 VisualOnly の検知 | `Producer::is_abandoned()`(lib.rs:464)は **consumer が drop された時**だけ true(lib.rs:286-289)。consumer は callback closure が持つ(device.rs:78, 90)ので、stream が drop されれば producer loop が `running` を待たずに抜けられる(producer.rs:193-195 の 1 ms spin を止める)。**device が死んで callback が呼ばれなくなっただけでは false のまま** — AU2 は cpal の error callback(device.rs:92-94)が正本。弊社は「糸が閉じた」検知まで | +3 行、AU2 の代わりにはならない |
| RT3 | 4096 frame(session.rs:14)の根拠が無い。device 側は `BufferSize::Default`(device.rs:85) | 弊社の性質は lib.rs:7-19 — 満杯で `Err`、待つ手段は無い。寸法の下限は「device block n + producer が寝ている間に消える分」。producer は 1 ms poll(producer.rs:13)で 1024 frame(:15)を作るので、`Fixed(n)` なら `capacity ≥ 2n + 1024`(n=512 で 2048、今の半分)。上限が seek 時に残る古い音(RT1 で消えるなら 4096 のままでも害は無い) | 定数 1 つ、根拠は Motolii の数字 |
| RT4 | producer.rs:110-121 が `push()` を sample ごと、ring.rs:18-22 が `pop()` を sample ごと。mix.rs:97 が chunk ごとに `Vec` を確保して producer が `pending` へ `extend`(producer.rs:255, 261) | `T: Copy` なので `push_partial_slice`(chunks.rs:281)/ `pop_partial_slice`(:408)で loop が消える。`write_chunk_uninit(n)`(:217)+ `as_mut_slices`(:705)へ mix が直接書けば pending への copy が 1 回減るが、`mix_audio` が `&mut [f32]` を受ける形に変わり、ring の折返しで 2 片に割れる。resampling 経路(:254)は rubato の出力を経るので減らない | slice API で ≒10 行減。`write_chunk_uninit` は非 resampling 経路のみ、unsafe な `commit`(:724)が要る |

弊社に無い物(正直に): 世代・時刻の付箋(item に metadata は付かない、RT1 は atomic を Motolii が持つ)、producer からの flush、device 死活(RT2)、寸法の計算式(RT3 は Motolii の数字から)。

## Apache Arrow(arrow-rs 58.4.0、rerun の store 経由)

前提: Motolii の文書 store は EntityDb(document.rs:229)で、component は 2 種だけ — `TrackJson`(`DataType::Utf8`、components.rs:57-59)と `LayerPresent`(`Boolean`、:92-94)。track・meta・attrs・masks・effects・shapes・text・composition・markers・slots・assets の 11 descriptor が全部 `TrackJson`(components.rs:144-238)。書きは `serde_json::to_string` → `TrackJson` が 15 箇所(document/apply.rs:28-272、validate.rs:148)、読みは `latest_at` → `component_batch::<TrackJson>` → `serde_json::from_str` が 12 箇所(view.rs:181-538)。弊社の型は「Utf8 の箱」としてしか使われておらず、中身の schema は arrow から見えない。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| AR1 | **SE3** 版番号が無い。`.rrd` の header version は rerun の `CrateVersion`(persist.rs:40、frames.rs:114-139)で、上流は「file > local なら warn のみ、0.23 未満は拒否」。chunk の `sorbet:version`(sorbet_schema.rs:41,48)は re_sorbet の私有 `trait Migration`(migrations/mod.rs:31-40)が回すもので Motolii は差し込めない。`StoreInfo` に version 欄は意図的に無い(re_log_types lib.rs:632 注記)、Motolii は `SetStoreInfo` も書かない(document.rs:229) | 3 層目を Motolii が持つ: `motolii.SchemaVersion`(`UInt32` か `Utf8`)を composition path に static で 1 component。`LayerPresent` の写し(components.rs:92-134)で足り、`load`(persist.rs:75-99)で `rebuild_head_from_store` の前に読んで分岐。弊社の `Field.metadata`(field.rs:376,383)に載せる手もあるが、rerun の `ComponentColumnDescriptor` が field metadata を自前で組み直す(component_column_descriptor.rs:243-249,357-359)ので未知 key の往復は**未検証** — component として持つ方が確実 | ≒25 行、SE3 の「番号」半分 |
| AR2 | migration の実体は既に serde: `#[serde(default)]`/`alias`/`untagged` が 18 箇所(store.rs:132,161、attrs.rs:129-134、asset.rs:84-100、track.rs:454、slot.rs:28) | 正直に: field 追加・rename は JSON+serde が今日すでに賄っている。弊社の `cast` の Struct→Struct は**field 数が同じ時だけ**(arrow-cast mod.rs:199-201、名前一致か位置一致 :216-235)で、「欄を足して既定値」は cast ではなく `new_null_array`(arrow-array mod.rs:1020)+ `StructArray::try_new`(struct_array.rs:106)の組み直し。serde の `default` より手数が多い | JSON 維持。SE3 の残り半分は AR1 の番号で `from_str` 前に分岐する 1 関数 |
| AR3 | **P1** の parse: RecordCache(document.rs:156-172、449b2191)と TrackCache(:176-200)で revision ごと 1 回に減っている。残るのは revision 1 回 × 層 × component の `from_str` と、`flattened`(persist.rs:8-32)が全 JSON を文字列のまま運ぶ分 | 型付きにすれば `Chunk::iter_slices_from_struct_field`(re_chunk iter.rs:259-262、「column 全体を 1 回 downcast、batch は slice 参照」)で f64/[f64;N]/String/bool(`ChunkComponentSlicer` impl iter.rs:339-979)を parse 無しで読める。ただし Motolii の読み口は `latest_at` の 1 値(view.rs:181)で、列走査で得をするのは「全層の全キー」を 1 度に組む rows 系だけ | 計測前は売らない。P1 が cache 後も残るなら track 1 種(AR4)から |
| AR4 | `Keyframe { t: RationalTime, value: Value(7 変種、track の Path 含む), interp: Interp(9 変種・可変引数), spatial: Option }`(value.rs:17-25、track.rs:37-63,450-456)、`PropertyBase` は `untagged`(slot.rs:28-37) | `List<Struct{t, value, interp}>` にする場合、enum は (a) `Union` — 上流でも使用は tensor_buffer.rs:70 の 1 箇所(dense)、弊社の `cast` は Union **へ**は不可(mod.rs:115)、**から**は型抽出のみ(:788)で evolution が止まる。(b) 疎 Struct(変種ごとに nullable 欄、`Path` は `List<Struct>`)— cast は効くが Loggable は手書き(derive 無し、上流は fbs codegen の re_types_builder; Motolii の components.rs:57-90 と同じ手書き)で 1 型 ≒80-120 行。既存 .rrd は AR1 の番号で「JSON → 型付き」を load 時に 1 回書き直す | 正直に: 得るのは parse 消滅、失うのは serde の evolution と ≒300 行。今は JSON |
| AR5 | **AE5** UndoLabel(vendor-pitches-2.md の AE 節) | `Utf8` 1 component は妥当 — `TrackJson` と同じ型(components.rs:58)、上流の `Text` も Utf8(re_sdk_types components/text.rs)。`apply_all` の `at`(document.rs:316-338)に composition path で書けば `discard_batch_at` の `drop_time_range`(:341-345)で失敗時も一緒に消える | ≒20 行、新規型無し |

**判定**: 今のまま JSON でよい。弊社の schema evolution(Field.metadata・cast)は「型付き Struct を弊社の型で持って初めて」効くが、Motolii の evolution は serde の `default`/`alias` が既に 18 箇所で賄っており、弊社の cast は field 数不変が条件(AR2)なので置き換えても手数は減らない。SE3 で本当に欠けているのは**番号**だけ(AR1)。P1 は cache 後の実測待ち(AR3)。AE5 は Utf8 で即(AR5)。

弊社に無い物(正直に): Rust の struct/enum から Loggable を出す derive(上流は fbs codegen、Motolii は手書き)。Union の schema evolution(cast 不可)。「欄を足して既定値」の 1 発 API(cast は等数のみ、組み直しが要る)。serde_arrow 級の橋(弊社の外)。rerun の migration 連鎖への差し込み口(`trait Migration` は私有)。

## Rive text 仕様(reference/rive-text-defs)

前提: `PINNED_REVISION` = 1f04919a。JSON は dev/defs の型定義で、`unitsValue` 等の uint が何を指すかは JSON に無い — 同 rev の `include/rive/text_engine.hpp`・`include/rive/text/text_modifier_range.hpp`・`src/text/text_modifier_range.cpp`・`text_modifier_group.cpp` と rive.app/docs(text-overview)で裏を取った。`T` = crates/motolii-doc/src/store/text.rs、`I` = store/document/ids.rs、`E` = motolii-render/src/engine/text.rs、`tsv` = reference/lottie-coverage.tsv(rive 行 666-738)。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| RV1 | `TextRangeSelector{based_on, range_units, shape, randomize}`(T:171-176)、property は start/end/offset/max_amount(I:66-79)。1 range = selector 1 個、合成 mode 無し | text_modifier_range.json: `modifyFrom`/`modifyTo`/`offset`/`strength` ↔ start/end/offset/max_amount(tsv:709-711,718 採用済)。`unitsValue` = characters / charactersExcludingSpaces / words / lines ↔ `TextBasedOn` 4 値、`typeValue` = percentage / unitIndex ↔ `TextRangeUnits`。**tsv:713 の「Character/Word/Line(3 値)」は誤記 — hpp は Lottie `b` と同じ 4 値**。Lottie と違う所: (a) `shape` 列挙は無く `falloffFrom`/`falloffTo` + interpolator(linear/cubic)+ `clamp` で重み曲線(tsv:715-717 不採用)、(b) `randomize` は無い、(c) `modeValue` add/subtract/multiply/min/max/difference は **group が複数 range を 1 本の coverage 配列に順に畳む**形(text_modifier_group.cpp `computeCoverage` → 各 `range->computeCoverage(m_coverage)`;tsv:714 不採用なので押さない)、(d) `runId` で 1 run に限る(tsv:719「後日」) | tsv:713 の訂正 1 行。**`coverageAt(t)`(範囲外 0、falloff 内 1、両端 ramp)は cpp に実装が在る**(Lottie は文だけ、LT2)。MIT なので GC4/LD6 の glyph 単位の重みは ≒20 行で写せる |
| RV2 | glyph 変形は `text_range_origin/opacity/position/rotation/scale`(I:118-135、tsv:701-708 採用済)。描く側に読者無し | text_modifier_group.json の originX/Y・opacity・x/y・rotation・scaleX/Y と同型。違いは `modifierFlags`(tsv:700 不採用)と opacity の合成式: 既定 `current * opacity * t`、`invertOpacity` で `current*(1-t) + opacity*t`(text_modifier_group.cpp `computeOpacity`) | 新しい語は無い。描く時の式を 1 つ借りるだけ |
| RV3 | Text Follow Path: model に path を指す口が無い(`TextRange` T:184-189 に target 無し) | text_follow_path_modifier.json `radial`/`orient`/`start`/`end`/`strength`/`offset`、親 text_target_modifier.json `targetId`。**Lottie 側(tsv:620-625)も Rive 側(tsv:683-688,733)も不採用済み**。「後から animator として足せる」と書かれている | 既決。押さない |
| RV4 | `wrap_size: Option<[f32;2]>`(T:233)。E:95 は幅だけ渡し、高さは読まない(texture.rs:567 は素通し)。wrap = `is_some()`。縦中央は E:98-108 の手計算 | text.json `sizingValue` autoWidth/autoHeight/fixed、`overflowValue` visible/hidden/clipped/ellipsis/fit/fitFontSize、`wrapValue` wrap/noWrap、`verticalAlignValue` top/bottom/middle(text_engine.hpp)。docs: overflow と vertical align は Fixed でだけ効く。tsv:667/668/676/677 全て不採用(「`sz` の有無が同じ分岐」「fit だけ後日」) | S10 の語彙として: Motolii の今は autoWidth(None)/autoHeight(Some、高さ未読)の 2 状態で fixed が無い。高さを読む時に `fit` の名前だけ借りる(tsv:668 の留保内)。GC1 の中央寄せは `middle` と同義 |
| RV5 | 箱の原点は枠の左上(LD2 済)、層の anchor が正本 | text.json `originX`/`originY`(正規化)・`originValue`・`fitFromBaseline`、`TextOrigin` = top / baseline。tsv:672/673/675/678 不採用(層の anchor と二重帳簿)。E:100 の「1 行目 baseline − 1 級」は origin=top の近似 | 既決。押さない |
| RV6 | `TextRun{len, style}`(T:224-227、`validate_runs` T:282-304)。描く側は E:34/95 が `styles[0]` だけ、`.runs`/`.ranges`/`alignment` は render 配下に読者 0(grep、export/lottie を除く) | text_value_run.json `styleId`(animates)+ `text`。Motolii は content 1 本 + 長さ分割(tsv:734 採用済 / 735 不採用)。`TextStyle` は container で子に `TextStylePaint`(fill/stroke は子の並び順)・`TextStyleAxis.axisValue`(animates)・`TextStyleFeature`・`TextStyleBackground.cornerRadius`;Motolii は 1 行に平坦(T:110-122) | model は同型で揃っている。売れる新語は無く、描く側は CT5 の道 |
| RV7 | **空手形**: repo 内で `rive-text-defs` を参照する rs/md/tsv は 0 件(grep)。tsv の rive 行 73 本は手書き、tsv 頭書 7 行目「機械照合は source=lottie の行だけ」、8 行目の `lottie_coverage.rs` は tree に無い(`git ls-files` の coverage は mask_coverage.rs と vector/coverage.rs のみ) | JSON 20 本の `properties.<key>` は tsv 第 3 列と同名。lottie 側と同じ枠で `rive-text-defs/*.json` を読み「表に在って JSON に無い / JSON に在って表に無い」を落とす試験 | ≒30 行。RV1 の誤記(4 値→3 値)の類を機械が拾う |

弊社に無い物(正直に): **縦書き** — `TextDirection` は ltr / rtl のみ(text_engine.hpp)、docs にも無し。S4・LD9 は Rive でも救えない。**ルビ**(S9)無し。`shape` 6 種と `randomize`(Lottie 由来)無し。fill/stroke の animator(I:90-99 `text_range_fill_color` 等)は Rive の group に無く Lottie が正本。text_input*.json 6 本はフォーム widget(tsv:689-697 不採用)で本文編集と無関係。

見込み合計: 訂正 1 行 + 照合試験 ≒30 行 + coverage の写し ≒20 行。Follow Path・Overflow・Origin は既決を覆さない。

## proptest(1.11.0)

前提の訂正: 嵐は既に弊社で回っている。gui.rs:973-1003 が `prop_oneof` の Poke Strategy と `proptest!`(cases 64、**max_shrink_iters 128**)、`collection::vec(poke(), 1..96)` が 95 手。shrink も効いていて `proptest-regressions/ui/gui.txt` に 4 seed が在る(1〜4 手に縮んだ物 3 本)。ただし 4 本目 e6b284f は **78 手のまま**—128 回で打ち切られた跡。弊社の既定は `u32::MAX`(config.rs:177)、時間の天井は `max_shrink_time`(:176、0 = 無制限)。ux-chaos.tsv の `proptest_seed_48ad517d` は弊社の `failure_persistence`(既定 `SourceParallel("proptest-regressions")`、file.rs:50-59)が残した物で、repo に既に入っている。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| PT1 | gui.rs:1002 `max_shrink_iters: 128` — 78 手の seed が縮まない。`prop_oneof` は前の腕へ縮む(unions.rs)ので今の並び(Press が先頭)は縮んだ列が Press/Motion だらけになる | `max_shrink_iters` を外し `max_shrink_time: 60_000`(1 分の天井、AGENTS.md と同じ)に。腕を「壊れにくい順」(Tick・FocusLoss を先頭)へ並べ替え、縮んだ列が割り込みの本質だけ残る | 2 行。Z13「storm が新しい状態を見ない」の再現列が短くなる |
| PT2 | Z13: 嵐の事後条件は DOM の個数と gesture だけ(gui.rs:1049-1054)。Q1「Undo/Redo が窓側の id を掃除しない」類は Session の中身を見ないと捕まらない | 同じ嵐に **reference model** を足す: 各手の後で `session.selection ⊆ doc.view().layers()`、`typing` の欄が DOM に居る、`writable`(session.rs:376)が錠と一致。弊社に state machine crate は無い(下記)が、`vec(op)` + model の型は今の嵐そのもの — 事後条件を毎手に移すだけ | ≒20 行。Q15・Q16・Z3 の「storm の assert は未」が閉じる |
| PT3 | Document の不変条件は決定的 test のみ(tests/project_round_trip.rs・group_command.rs)。doc crate 内に `#[test]` 0 | Intent の Strategy(`prop_oneof` で AddLayer/SetOrder/SetAttrs{parent}/SetMarkers/Freeze…、document.rs:28-134)を 1 本書き、`proptest!` で (a) `apply_all` → `undo` → `redo` で `view()` が一致(undo/redo は head の移動だけ document.rs:298-314 なので `revision()` は store generation を含み(:451-456)literal には戻らない — head で比べる)、(b) `validate_no_parent_cycle`(validate.rs:10)を通った後の親鎖に循環無し、(c) `split_layer` / `split_track_at`(timeline_edit.rs:77,167)の前後で尺の和とキー数が同じ、(d) save→load で view 一致(persist.rs:34,75) | Strategy ≒40 行 + 各 10 行。P9「履歴が伸びっぱなし」の再現にも同じ Strategy が使える |
| PT4 | tests/preview_equals_export.rs:11 は手組み 1 本(ffmpeg 素材、温まり待ち 150×20ms) | ffmpeg 無しの層(矩形・文字)で comp(層数・キー数・時刻)を乱択し、冷えた Engine 1 回 vs 別時刻を何枚か描いた後の同時刻、を比べる。GPU 1 case ≒数十 ms なので `cases: 32` が現実的。動画は今の 1 本を残す | ≒50 行。P1 の cache を入れた時の回帰検知 |
| PT5 | 64 case が固定 | `PROPTEST_CASES` 環境変数(config.rs:26)で code を触らず夜間だけ 512 に。`PROPTEST_MAX_SHRINK_TIME` も同様 | 0 行 |

**速度・天井(正直に)**: 嵐の 1 case は fixture + VirtualDom + blitz DOM を建て直す(gui.rs:47-63)。時間は測っていない(他 session が作業中で走らせていない)。cases は線形に効くので、blitz harness を回す試験は 64〜128 が天井、Document 単体(PT3)は 256 既定のままで良い。

**弊社に無い物(正直に)**: state machine testing は別 crate `proptest-state-machine`(registry に無し、dev-dep 1 本増える。前提条件付き shrink をやるだけで、PT2 は無くても書ける)。`#[derive(Arbitrary)]` は `proptest-derive`(同じく無し)— Intent は手書き Strategy(今の Poke と同じ)。oracle は弊社に無い: 弊社が出すのは生成・縮小・seed の台帳だけで、ux-chaos.tsv の `ruler` 列(W3C pointercancel・UIKit)は Motolii が借り続ける。AGENTS.md「自作 test は借りた定規の写像だけ」には、定規 = 不変条件(undo∘redo = id、循環無し、和の保存)である限り弊社は定規ではなく**定規を当てる回数と失敗の縮め方**を売っている、と読んでほしい。GUI の合否は依然として利用者の窓。

## wgpu(29.0.4)

前提: Motolii の lock 上の wgpu は 29.0.4 の 1 本(`motolii/Cargo.lock:9143-9308`、wgpu_context 0.9.0 `:9321`)。窓の経路は blitz の `DeviceHandle` を Engine が借り(`src/ui/stage_widget.rs:874-878` → `compositor/device.rs:69-81` → `RenderContext::new_from_device` `device.rs:25`)、**同じ device・同じ adapter**で re_renderer と vello が動く。書き出しは別スレッドで `Engine::new()` → `headless.rs:22-30` が 2 本目の Instance/adapter/device を作る(`src/ui/output.rs:181-185`)。二重 adapter ではなく、用途が分かれた 2 device — 問題は後述 WG6 の「要求が揃っていない」事だけ。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| WG1 | GPU 時間を測っていない。`RenderTiming`(`compositor.rs:235`)は submit→`poll` の壁時計(`render_basic.rs:119-129`)で、`render_with_timing` は呼び手なし。re_renderer の pass は全部 `timestamp_writes: None`(`view_builder.rs:1090`、`screenshot.rs:97`) | `Features::TIMESTAMP_QUERY` + `TIMESTAMP_QUERY_INSIDE_ENCODERS`(`wgpu-types features.rs:1620`、`:718`)。Metal は stage boundary で対応(`wgpu-hal metal/adapter.rs:623-628`、`:1056-1063`)。Motolii 自前の encoder(`sequential.rs:516-541` finalize_into、`render_effects.rs`、`vism.rs:356`)の前後に `CommandEncoder::write_timestamp`(`api/command_encoder.rs:313`)→ `resolve_query_set`(`:233`)→ 既存の readback 経路で読み、`Queue::get_timestamp_period`(`api/queue.rs:283`)で ns に。要求は `DioxusNativeWindowRenderer::with_options(RendererOptions{features: Some(..)})`(blitz `dioxus_renderer.rs:41-46`、`:80`;今は `default()` `src/ui/host.rs:311`)。wgpu_context は `adapter.features()` と AND するので無い GPU でも落ちない(`wgpu_context lib.rs:45-47`) | ≒40 行。P 波の「層 200 でどの pass が重いか」が数字で出る。ViewBuilder 内部の pass は fork 側の 1 行(`timestamp_writes`) |
| WG2 | 書き出しは 1 コマずつ直列: `render_frame`(submit → `poll(wait_indefinitely)` `sequential.rs:441-446` → map)→ `write_frame`(ffmpeg stdin へ同期 write `media/encode.rs:103-114`)`export.rs:151-160`。GPU が描く間 CPU は待ち、ffmpeg に書く間 GPU は空 | `PollType::Wait{submission_index, timeout}`(`wgpu-types lib.rs:229-259`、`api/device.rs:95`)で「N の submit だけ待つ」形にし、N+1 を先に submit。ScreenshotProcessor は identifier 付きで複数 in-flight を前提に作られている(`gpu_readback_belt.rs:327-346` が chunk を frame 越しに保持)。`timeout: Some` にすれば固まる代わりに `PollError::Timeout`(`lib.rs:296-298`)が文言になる。`map_buffer_on_submit`(`api/command_buffer_actions.rs:105`)は belt 側の TODO(`belt.rs:330`)で、上流待ち | ≒60 行。D 波の速度は「GPU と ffmpeg の重なり」が上限を決める。ffmpeg 書き込みを別糸にするのは弊社の外 |
| WG3 | CV1 の panic / 無限ループは CPU 側(`sequential.rs:60-66` の idx が進まない件)で弊社の範囲外。GPU 側は re_renderer が共有 device に `on_uncaptured_error` を据え(`context.rs:360-369`)、`ErrorTracker` が `re_log::error!` に流すだけ(`error_tracker.rs:61-68`、`:120`)→ `re_log::setup_logging()`(`src/main.rs:3`)で console 止まり、窓に出ない。handler は device に 1 つなので vello の error も同じ所へ落ちる | `push_error_scope(ErrorFilter::Validation)` / `pop()`(`api/device.rs:454-480`、thread-local の stack)を vism の pipeline 生成と `flush_pending`(`sequential.rs:16-26`)の周りに置き、`Option<wgpu::Error>`(`device.rs:793`)を `CompositorError` に載せて status へ(文言は英語、D11)。`set_device_lost_callback`(`device.rs:591`)で「GPU reset」を黒画面でなく報せに | ≒30 行。誇張なし: Metal で validation error が出るのは submit 時で、事前検証は NG5 |
| WG4 | 読み戻しの padding は re_renderer が `align_to(.., COPY_BYTES_PER_ROW_ALIGNMENT)`(`texture_info.rs:36`、定数 256 `wgpu-types lib.rs:169`)で足し `remove_padding` で剥がす(`gpu_readback_belt.rs:100-111`、`screenshot.rs:137`)。Motolii 自前の `copy_texture_to_buffer` は無い | 正しい。売る物なし(確認済みの報告) | 0 行 |
| WG5 | E13 RAM preview の予算は decision 無し。文字 texture は **comp 寸法**の Rgba8Unorm を層ごとに 256 枚(`engine/texture.rs:185-212`、`:34`)— 4K なら 33 MB × 256 ≒ 8.5 GB | 予算の口: `device.limits().max_texture_dimension_2d`(`api/device.rs:115`)は寸法の天井、`counters` feature の `HalCounters`(`wgpu Cargo.toml:50`、`wgpu-types counters.rs:108-132`)は個数。バイト数は w×h×4 の自前計算 | 上限の式は Motolii 側 ≒10 行。実効は P2 の「実バウンディング」が先 |
| WG6 | 窓の device は blitz が `Limits::default()`(`max_texture_dimension_2d: 8192` `wgpu-types limits.rs:376`、`wgpu_context lib.rs:51-55`)、features は `CLEAR_TEXTURE \| PIPELINE_CACHE` のみ(`anyrender_vello window_renderer.rs:226-227`)。書き出しは adapter の limits(`device_caps.rs:211` from_adapter)。**同じ作品が Stage では 8192 超で落ち、書き出しでは通る**差が既にある(4K×3 = 幅 11520) | `RendererOptions.limits`(`dioxus_renderer.rs:44-46`)に本機で確かめた `adapter.limits()` の値を渡す。WG1 の features と同じ 1 箇所 | 数行。P 波の 4K×3 の前提 |

弊社に無い物(正直に):
- **GPU memory の残量・予算の問い合わせ**。`MemoryBudgetThresholds`(`wgpu-types instance.rs:325-336`)は dx12 / vulkan だけ(`wgpu-hal dx12/mod.rs:528`、`vulkan/mod.rs:170`、metal に無し)。`generate_allocator_report` は Metal で `None`(`api/device.rs:534-545`)。`HalCounters.texture_memory` も Metal は数えない(`metal/device.rs:428` は buffers の個数のみ)。E13 の予算は Motolii の算術で決めるしかない。
- **`SHADER_F16` は shader の演算幅**(`features.rs:1657`)で texture のバイト数は減らない。**圧縮 texture(BC7 / ASTC)は弊社が encode しない** — texture_manager は受ける(`texture_manager.rs:589-595`)が CPU 側 encoder crate が要り、文字の縁で 4:1 の非可逆は勧めない。文字 texture の予算は「comp 寸法をやめる」(P2)が本筋。
- **`TIMESTAMP_QUERY_INSIDE_PASSES` は Apple GPU で無い**(`features.rs:726-731`)。draw 単位ではなく pass 単位まで。
- `PUSH_CONSTANTS` / `MULTI_DRAW_INDIRECT` は Motolii に使う場所が無い(re_renderer は `Features::empty()` `device_caps.rs:105-107`)。要求しているのに未使用の feature は無し。
- CV1 系(Rust の走査の誤り)を GPU の error scope は捕まえない。

見込み合計: 弊社側 0 行、Motolii ≒140 行(WG1 40・WG2 60・WG3 30・WG5-6 10)で「pass ごとの GPU ns、書き出しの GPU/ffmpeg 重ね、GPU error が窓の文言、Stage と書き出しの limits 一致」が既に払った代金の中で出る。順は WG6(数行、前提)→ WG1 → WG3 → WG2。

---

# 委託の館(技術カンファレンス、拡張ではなく自前 code の引き取り手)

指標(利用者): 拡張は後、自前で書いている物の引き取り手だけ。素通りした拡張の館: LRC/TTML・SRT、Ableton Link・aubio、OpenTimelineIO・FCPXML、OpenColorIO、Syphon/NDI・OSC、resvg、OpenUSD、cargo-deny/packager(法務・配布)。

## 委託の要約

| 社 | 引き取る自前 code | 代償(正直に) |
|---|---|---|
| muda | File/Edit/View の DOM menubar ≒500 行 → ≒110 行。K16 の手書き加速鍵 6 本と keymap.rs の重複が 1 本に。H3・H4・H14・BU6 の半分が 0 行 | Composition/Export/Settings の popover は NSMenu に載らず殻が残る。⌘Q の delegate は別(AP2)。DOM の menu を押す試験 4 本が消える — 消すなら利用者裁定 |
| color(Linebender) | 丸めの式 7 箇所 → `to_rgba8` 1 箇所、hex の parse/出力、HSV 31 行 → HWB 写像 6 行。UI 側は依存 0 増(peniko 経由) | W3C の blend 式は無い(blend_preview.rs 129 行は残る)。Rec.709 / YUV の語彙は無い。u8 の premultiply 直行路は無く、文字 raster 全画素には今の整数 loop が速い |
| re_log / re_tracing / re_error | `println!("PROBE")` 107 箇所を target 付き `re_log::debug!` に付け替え、`RUST_LOG=warn,motolii::write=debug` で部屋ごとに絞る(RL1)。warn を status bar へ(RL5)。re_renderer の 94 scope は `new_frame()` を誰も呼ばず**死んでいる**、7 行で生き返る(RL2)。file に落とす capture(RL3) | `ChannelLogger` という型は無い(訂正)。puffin の server(port 開放 + viewer spawn)は売らない。エラーの連鎖を String に潰した 5 箇所は `#[source]` に戻さないと re_error は無力 |
| directories / NSUserDefaults | `HOME` 手書きの path 3 行 → 1 行、空 HOME の相対 path 事故が消える(DR1)。**Settings の面の値(Scale・Outside dim)はどこにも仕舞われず窓を閉じると消える** — NSUserDefaults 2 key で残る(UD1) | file I/O は無い。layout.json / window.json は JSON のまま。3 対の JSON 読み書きが同じ形で 3 回、原子的 rename 無し — 畳むのは Motolii 側の整理 |

---

## muda(native menu、委託)

前提(裏を取った所): 本機 registry に muda は無く、crates.io から 0.19.3 の source を scratchpad に展開して読んだ(build はしていない)。muda の macOS 実装は `src/platform_impl/macos/mod.rs` 1283 行、依存は objc2 ^0.6.1 / objc2-app-kit ^0.3.0 / objc2-foundation ^0.3.0 / objc2-core-foundation ^0.3.0(Cargo.toml:101-140)。winit には依存しない(`NSApplication::sharedApplication` を直に取る、mod.rs:211-214)。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| MD1 | app.rs:1274-1610 の `#menubar`(File/Edit/View ≒300 行が DOM の button と closure)、semantic_menu.rs:1-192(MenuId・MenuDismiss・MenuItems の roving・SemanticMenu・SemanticControl)、styles.css の menu 系 ≒190 行(131-170、552-690、1055) | `Menu::new()` + `Submenu::with_items` + `MenuItem::with_id(id, text, enabled, Some(Accelerator))`(items/normal.rs:56)→ `menu.init_for_nsapp()`(menu.rs:414)。定義は 1 項目 1 行、≒60-70 行 | DOM 側 ≒500 行減、新規 ≒110 行。**ただし** MenuId::Composition / Export / Settings(app.rs:1590-1608)は menu でなく form の popover — NSMenu には載らない。3 つを sheet か別窓へ出さない限り SemanticMenu の殻(semantic_menu.rs:52-142)と CSS の半分は残る。消えるのが確実なのは SemanticControl(145-192)と File/Edit/View の DOM だけ |
| MD2 | enable / checked を Document の状態で変える(Export の `disabled`: app.rs:1446-1450、View の `checked`: 1543・1561) | `MenuItem::set_enabled`(normal.rs:97 → mod.rs:492-496 `setEnabled`、`setAutoenablesItems(false)` mod.rs:134 で手動制御)、`CheckMenuItem::set_checked`(check.rs:137 → mod.rs:540-550 `setState`)。項目は `Rc<RefCell>`(menu.rs:13-14、normal.rs:15-16)で main thread 限定 — 呼ぶ場所は host.rs:418 `reflect_document`(window_event の後、711 行目で毎回通る) | 追える。Export の「押せない理由」文(app.rs:551 の `export_hint`)は NSMenu に置き場が無い — `set_text` で末尾に足すか status へ |
| MD3 | K16 の加速鍵表記は手書き文字列(app.rs:1293 "⌘N"、1364 "⌘S"、1383 "⇧⌘S"、1460 "⌘Q"、1488 undo/redo) | `Accelerator` → `KeyAccelerator::key_equivalent` / `modifier_mask`(macos/accelerator.rs:9-75)で AppKit が ⌘S を右端に描く。文字列 6 本が消える | **副作用(正直に)**: NSMenu が key equivalent を持つと `performKeyEquivalent:` が view の `keyDown:` より先に食う。winit-appkit の `sendEvent:` 上書きは Cmd+KeyUp だけ(app.rs:33-39)。keymap.rs:340-346 の ⌘N / ⌘O / ⌘Q は menu に移した瞬間 keymap 側では二度と発火しない → 表は 1 本(menu)にする。AGENTS.md の「同じ意味を 2 箇所」がそのまま解ける |
| MD4 | BU6 roving focus、K3 / KB9 の ↑↓(semantic_menu.rs:85-117 の 33 行) | NSMenu は ↑↓ Home End・文字打ちの選択・Escape を AppKit が持つ。File/Edit/View 分の上流 PR は不要 | Composition / Settings の popover(KB9 の当事者)が DOM に残る間は MenuItems の roving も残る。BU6 は「半分不要」 |
| MD5 | MenuEvent の配線 | lib.rs:164-178 の作法どおり `MenuEvent::set_event_handler(Some(move \|e\| proxy.send_event(BlitzShellEvent::embedder_event(e))))`。`BlitzShellProxy` は `Arc<Inner{Sender}>`(blitz-shell event.rs:71-74)、rustc 1.96.1 で Sender は Sync なので handler の `Send + Sync` を満たす。受けは host.rs:719-723 の `Embedder` arm に `downcast_ref::<muda::MenuEvent>()` を 1 本足す | launch() 3 行 + proxy_wake_up 5 行 + MenuId→Intent 表 ≒25 行。**残る宿題は Motolii 側**: Intent の match が DOM の onkeydown closure の中(app.rs:768-778)に在り、外から呼べない。keys.rs:247 `act` と同じく DioxusDocument を取る関数へ持ち上げる必要が在る。MenuEvent は窓 id を持たないので key window(host.rs:701 の Focused 追跡)を使う |
| MD6 | H14 別窓に menu が無い | main menu は app に 1 本(`setMainMenu`、mod.rs:214)。`Submenu::set_as_windows_menu_for_nsapp`(submenu.rs:197 → mod.rs:718-725)で Minimize / Zoom / 窓一覧を AppKit が埋める。`PredefinedMenuItem::{minimize, maximize, fullscreen, close_window}`(predefined.rs:84-147) | H4・H14 は 0 行。View の「{panel} を出す / Window へ」の ✓ は key window の dock を反映する必要が在る(MD2 の reflect で窓が変わる度) |
| MD7 | H3 About・⌘,、H2 Services | `PredefinedMenuItem::about(text, AboutMetadata)`(orderFrontStandardAboutPanel、mod.rs:1060-1124)、`services`(`setServicesMenu`、mod.rs:883-886)、`hide` / `hide_others` / `show_all` | H3 の About と H2 の Services は 4 行。⌘, は普通の MenuItem |

弊社に無い物(正直に):
- **⌘Q は素通りのまま**: `PredefinedMenuItem::quit` は `terminate:`(mod.rs:994)で AP1 と同じ欠陥。`MenuItem::with_id("quit", …, ⌘Q)` で Intent::Quit(app.rs:1191)へ回せば menu からは直るが、Dock の Quit と ⌘Q 以外の terminate は `applicationShouldTerminate` delegate が要り、muda は持たない(AP1 の後半は残る)。
- **Edit の標準項目は responder chain 前提**: undo/redo/cut/copy/paste/select_all は `undo:` `copy:` の selector を投げるだけ(mod.rs:981-986)。winit の view はどれも実装しないので Undo/Redo は普通の MenuItem(⌘Z)で Motolii の history へ。Cut/Copy/Paste を predefined で載せると B2 の blitz-shell clipboard(keyDown で ⌘C を拾う)より先に menu が ⌘C を取る恐れ — **未検証**。Emoji(`orderFrontCharacterPalette:`)は predefined に無い(mod.rs:981-998)。
- **dynamic**: Open Recent は `Submenu::append` / `remove_at`(submenu.rs:95, 144)で毎回組み直せる。menu bar の非表示 API は無い。
- **版衝突**: Cargo.lock は objc2 0.6.4 / app-kit 0.3.2 / foundation 0.3.2 / core-foundation 0.3.2 を既に持ち、muda の ^0.3 / ^0.6 に収まる — 新しい二重は生まない。accesskit_macos 由来の 0.2.2 / 0.5.2 の二重(AP 節)はそのまま。足す依存: crossbeam-channel・dpi・keyboard-types・once_cell・png 0.18・thiserror。`default-features = false` を勧める(既定の libxdo / gtk は linux cfg だが)。
- **試験の道が消える**: gui.rs:583-690 と 731 の 5 本(menus_are_one_semantic_family、losing_window_focus_dismisses_an_open_menu、menu_motion_has_a_real_intermediate_frame…、outside_menu_click_is_consumed_before_the_stage、dead menu の検査)は `#menu-file` `.vmenu` `.menu-dismiss` を DOM で押す。NSMenu は harness から押せず、`MenuEvent::send` は pub(crate)(lib.rs:523)。残せるのは「MenuId → Intent の表」の単体試験だけで、開閉・動き・外側 click の 4 本は AppKit の物として捨てる判断が要る。DOM menubar を残す理由はこれ 1 つ — 消すなら利用者裁定。

要約: File/Edit/View を丸ごと NSMenu へ委託すると DOM ≒500 行が ≒110 行に、K16 の手書き鍵と keymap.rs の重複が 1 本になり、H3・H4・H14・BU6 の半分が 0 行。残るのは Composition/Export/Settings の popover、⌘Q の delegate、Intent match の持ち上げ、試験 4 本の喪失。

## color(Linebender 0.3.3、委託)

前提の確認: root の `motolii` は peniko 0.6 を直接引いており(motolii/Cargo.toml:33、workspace 76)、peniko は `pub use color;`(peniko-0.6.1/src/lib.rs:37)で `peniko::Color = AlphaColor<Srgb>`(同 57)。UI 側は**依存 0 増**で弊社の型が使える。Cargo.lock:1286 に color 0.3.3(依存は bytemuck のみ)。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| CL1 | color.rs `hex_of`:91-94(round)、`parse_hex`:107-116、fixture.rs `hex_of`:438-440 と `(x*255).round() as u8` が 4 箇所(468-470・508-511・519-522・816-819)、`default_palette` の hex 読み 489-491、blend_preview.rs `css`:109-112、compositor.rs `clear_color`:165-172 — 99c41deb で round に揃えたが**丸めの式は今も 7 箇所** | 丸めは `AlphaColor::to_rgba8`(color.rs:595、`fast_round_to_u8` 631 = `(x*255+0.5) as u8`、飽和は cast)1 箇所。hex 出力は `Rgba8` の `LowerHex`(serialize.rs:123-126、α=255 なら `#rrggbb`、それ以外 8 桁)。hex 入力は `parse_color("#f80")`(parse.rs:568、3 桁は 633 で展開)→ `to_alpha_color::<Srgb>()`(dynamic.rs:87) | ≒30 行減、CV3 の再発を式の数で止める。**注意**: `parse_color` は CSS 文法なので `#` 無しの `ff8800`(color.rs:106 が読む)は通らない — `#` を補う 1 行は残る |
| CL2 | color.rs `hsv_to_rgb`:58-72・`rgb_to_hsv`:74-89(31 行)、輪の色 `hue_hex`:223、gui.rs:523 の試験 | **HSV は弊社に無い**。在るのは `Hsl`(colorspace.rs:1513)と `Hwb`(1635)。輪は色相 + 正方形(彩度×明度)= HSV 前提なので、HWB へ写す: w=(1−s)·v、b=1−v、逆は v=1−b、s=1−w/(1−b)。成分は**百分率**(hwb_to_rgb が ×0.01)。灰色の色相は 0 を返す(rgb_to_hsl 1539、Motolii の 78 と同じ挙動)ので輪の印は壊れない | 31 行 → 写像 6 行 + `AlphaColor<Hwb>::new([h,w,b,1]).to_rgba8()`。f32 化で color.rs:242 の 1e-9 比較は 1e-6 へ緩める要 |
| CL3 | texture.rs `unpremultiply`:39-50(u8 の整数割り)、frame.rs `premultiply_rgba_u8`:191-198 | `PremulRgba8 → PremulColor<Srgb>`(rgba8.rs:131)→ `un_premultiply`(color.rs:661)→ `to_rgba8`(718)。逆は `Rgba8 → AlphaColor<Srgb>`(rgba8.rs:67)→ `premultiply`(458) | 式は消えるが **u8 の直行路は弊社に無い**(f32 往復、画素毎 4 乗算)。文字 raster 全画素に回すなら今の整数 loop の方が速い。`bleed_edges`(52-90)は弊社の外 |
| CL4 | CV2: 乗算済み sRGB を非乗算として decode(compositor.rs:148-160 の `decode_srgb` / `texture_alpha` の対) | `AlphaColor<Srgb>` / `PremulColor<Srgb>` / `PremulColor<LinearSrgb>` は別型。`PremulColor::convert` は非線形なら**必ず un_premultiply → 変換 → premultiply**(color.rs:647-657)なので、色 1 個の単位では取り違えが型で消える | ただし CV2 の現場は GPU の flag(ColormappedTexture)で、弊社の型は画素 buffer には届かない。buffer の札は `FrameDesc.premultiplied`(frame.rs:49)が既に在り、そこは Motolii の領分 |
| CL5 | CV4: desk.rs `blend_tint`:8-21 が sRGB の塗りをそのまま blend_preview.rs `blend`(「線形光」を要求、:6)へ渡す | `AlphaColor<Srgb>::convert::<LinearSrgb>()`(colorspace.rs:374-400 の srgb_to_lin / lin_to_srgb)で札の中だけ往復 | 入口・出口で各 1 行。**blend の式自体は弊社に無い**(下記) |
| CL6 | X5 の色タグ、frame.rs `ColorSpace`:34-40(LinearRgb / Srgb / Rec709Limited / Rec709Full / Rec601Limited) | `ColorSpaceTag`(tag.rs:26-58)は Srgb・LinearSrgb・DisplayP3・Rec2020・Oklab… で、Srgb↔DisplayP3 の matrix は在る(colorspace.rs:456 `LINEAR_DISPLAYP3_TO_SRGB`)。**Rec709 は無い** — trait 文書の「自分で impl する例」(colorspace.rs:54-73、BT.1886 の transfer のみ)。limited/full range・YUV の係数(bt601/709)は概念ごと無い | sRGB / P3 の語彙は借りられる。bt709 タグは re_renderer の `YuvMatrixCoefficients`(device.rs:160-166)と ffmpeg(encode.rs:77-85)のままが正しい |
| CL7 | 依存: motolii-doc(Cargo.toml:11-27)・motolii-render(11-37)に peniko も color も無い。`Value::Color([f64;4])`(value.rs:20)、`Rgb{f64}`(vector.rs:146) | doc に入れるなら `color = "0.3"` 1 行(lock 済み、default feature `std`、serde は任意)。型は f32 | Document は f64 のまま、弊社は縁(hex・premul・線形化)だけで使うのが安い。`Value::Color` を `AlphaColor` に替えると lerp(value.rs:34)も `lerp_rect`(color.rs:475)へ移るが、f64→f32 で保存値が動く |

### 弊社に無い物(正直に)
- **W3C の blend 式**(blend_preview.rs:7-104、separable + Hue/Saturation/Color/Luminosity)。弊社の `lerp` / `lerp_rect`(color.rs:263・244)と `gradient`(gradient.rs:114)は**補間**で、合成ではない。129 行はそのまま。
- **HSV**(CL2 の通り Hwb への写像で代替)。
- **u8 の premultiply 直行路**、edge bleed(texture.rs:52-90)。
- **Rec.709 / Rec.601、limited range、YUV 係数**。PNG が premultiplied のまま(X5 後半)は書き出し側の `FrameDesc` 指定(export.rs:133-139 の `true`)で、色の型の話ではない。
- **スポイト(C10)・パレットの保存(C9)**。`color::palette`(lib.rs 参照)は CSS 名前色の定数で、利用者パレットの器ではない。
- 依頼文で名指しされた TS5 / TS7 は同じ第 3 波の tiny-skia 節(本 file 内)を指す。

### 見込みの合計
UI 側(root crate)は依存 0 で CL1・CL2・CL5 ≒60 行減、丸めの式 7→1。doc / render に入れるなら各 1 行の依存増と f32 化の判断が要る(CL3・CL7)。CL4・CL6 は「型で語れる範囲」が色 1 個までで、buffer と YUV は Motolii / rerun の領分。

## re_log / re_tracing / re_error(rerun の身内、委託)

前提(file:line で確認): `re_log` は motolii/Cargo.toml:78 で `features=["setup"]`、呼ぶのは main.rs:3 の `re_log::setup_logging()` の 1 箇所だけ。motolii-render / motolii-doc も `re_log` を依存に持つ(crates/motolii-render/Cargo.toml:37、motolii-doc:27)が `re_log::` の呼び出しは 0。`re_tracing` は Motolii の依存に無いが、re_renderer(Cargo.toml:59、`profile_function!` 94 箇所)・re_chunk_store・re_query 経由で puffin 0.20.0 が既に Cargo.lock:5540 に居る。puffin_http は lock にも registry にも無い(取得が要る)。anyhow は不使用、エラーは全部 thiserror(motolii-render 14 enum、motolii-doc 12 enum)。

**訂正(正直に)**: fork rev 346a0b3 の re_log は tracing 基盤で、`ChannelLogger` / `MultiLogger` という型は無い。在るのは `ChannelLayer` + `add_log_msg_receiver(LevelFilter) -> Receiver<LogMsg>`(channel_logger.rs:42)で、`LogMsg { level, target, message, fields }`(:16-29)。`re_log::debug!` は `tracing::debug!` の再輸出(lib.rs:44)。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| RL1 | `println!("PROBE room=… verdict=…")` 107 箇所(全 println が PROBE、うち `room=write` 75・`stage` 14・`input` 9・`project` 3・`browser` 3・`reload`/`playback`/`export` 各 1)。出力制御は無く常時 stdout、絞りは grep だけ(memory「`RUST_LOG=warn` + grep」) | `re_log::debug!(target: "motolii::write", verdict = "apply-error", %e)` — `room=` を `target`、`verdict=…` を tracing の field に。`setup_logging` は `RUST_LOG` を EnvFilter に渡す(setup.rs:46、lib.rs:203-205)ので `RUST_LOG=warn,motolii::write=debug` で 1 部屋だけ出る。写す時に `session.rs:534 noted()` が 20 箇所分を 1 口に畳んであるので、そこ 1 行で `room=write` の大半が移る | 行数 ±0、`PROBE` の語は message に残せる。注意: stderr 出力(setup.rs:43)— dx の log が stderr を拾うかは本機で未確認(dioxus-cli のソースが手元に無い)。debug build は既定 filter が `debug`(lib.rs:175-180)なので、これが memory の「数 GB」の根でもある |
| RL2 | 計測は `Instant::now` 手書きが render_basic.rs:27,119,131(`RenderTiming` build/gpu/readback)— だが `render()`:16 が `_` で捨て、`render_with_timing` の呼び手は無い。playback.rs:152-303 の `Instant` は再生時計で計測ではない。stage_widget.rs:1172 / timeline_widget.rs:1220 の `paint`、sequential.rs:600 `render_sequential` / :646 `render_layer_to_canvas` に計測無し | `re_tracing::profile_function!()` を上記 5 関数に置く(1 行ずつ)。re_renderer 側の 94 scope(view_builder.rs:455,732,1024「main target pass」…)が**同じ木に無料で乗る**。条件: `puffin::set_scopes_on(true)` と毎フレーム `GlobalProfiler::lock().new_frame()` を Motolii が呼ぶこと — 今は誰も呼ばないので re_renderer の scope は死んでいる。RenderTiming 3 欄は消せる | 新規 ≒7 行、P1/P11 の「どこが遅いか」が層別に出る |
| RL3 | 見る口。`re_tracing/server` feature = puffin_http + rfd + `0.0.0.0:8585` bind(server.rs:29)+ `puffin_viewer` 子プロセス spawn(:48)+ 失敗時 rfd dialog | **売らない方**: server は全アドレスに port を開き viewer を勝手に起こす — 憲法(隠れた状態を持たない、外部プロセス最小)と合わない。**売る方**: `re_tracing::ProfileCapture::start(n)`(profile_capture.rs:20)— メモリに n フレーム溜めて `finish() -> puffin::FrameView`、`FrameData::write_into`(frame_data.rs:569、feature `serialization`)で `.puffin` file に落とし、viewer は後で読む。port を開かず `MOTOLII_PROFILE=200` 等の起動時 env 1 つで済む(host.rs:765 の `MOTOLII_FIXTURE` と同じ作法) | feature 無し、新規 ≒15 行。「1 分の天井」の内側で層 200・キー 2000 の 200 フレームを取り切れる |
| RL4 | エラー文は `{e}`(Display)1 段のみ。`CompositorError::Draw(e.to_string())`(render_basic.rs:116)、`IsfError::GlslParse{detail: errors.to_string()}`(isf/mod.rs:308-317、NG2)、`StoreError::Chunk(String)` 等(store.rs:80-90)が**連鎖を String に潰した時点で source が消える**。X2/X3 で `ExportError::Desc/Write(String)`(export.rs:24-27)も同型 | `re_error::format_ref(&e)`(lib.rs:50)が `source()` を `: ` で辿って 1 行に、`format_with_details(summary, details)` / `split_details`(:64-88)が「人の言葉 \n Details: 生の連鎖」の 2 段を規約化。ただし効くのは `#[source]`/`#[from]` で連鎖を残した型だけ — `String` 化した 5 箇所は `#[source]` に戻す前提 | 関数 3 本は今日呼べる。効かせるには enum 側 ≒10 行の型直し |
| RL5 | 窓への口: app.rs:1629 `#status role=status aria-live=polite` の `status_line` は `project_notice`(session.rs:172)と export の文だけ。`noted()` の apply-error、autosave.rs:38 の auto-save-error、stage_widget.rs:889 engine-error は**窓に出ない**(NG2 と同根) | `re_log::add_log_msg_receiver(LevelFilter::WARN)` を host.rs:743 `launch` で 1 回、`Receiver<LogMsg>` を poke で読んで `project_notice` へ(`LogMsg.target` で部屋、`fields` で verdict)。RL1 で `warn!` にした apply-error / auto-save-error が自動で status bar に届く | 新規 ≒20 行、X 波の live region と 1 本に |
| RL6 | 起動: main.rs:3 で `setup_logging()` 済み。`ResultExt::warn_on_err_once` / `ok_or_log_error`(result_extensions.rs:9,3)は未使用 | 既存のまま。`RUST_BACKTRACE=1` を debug で勝手に立てる(setup.rs:25-38)ことだけ承知 | 0 行 |

**弊社に無い物(正直に)**: (a) target 名の規約 — tracing は `crate::module` が既定で、`room=` の 8 語は Motolii が決める(提案: `motolii::{write,stage,input,project,browser,export}` の 6 つ、`reload`/`playback` は `stage`/`input` に吸収)。弊社に台帳は無い。(b) `re_tracing::reexports::puffin` は使えるが、Puffin の egui viewer は別 crate(`puffin_egui` / `puffin_viewer`)で Motolii の blitz 窓には出せない — file に落として外で見るのが限界。(c) `re_error::format` は anyhow 前提の書き方だが `&dyn Error` を取るので thiserror でも動く。連鎖を String で潰した型には無力。(d) log を窓に流す `ChannelLayer` は unbounded channel(channel_logger.rs:45)— 読み手が止まると溜まる。poke 毎に drain する前提。(e) puffin_http は取得から要る — RL3 の file 方式なら不要。

**まとめ**: 型追加ゼロで済む順は RL1(付け替え)→ RL5(warn を status bar へ)→ RL2+RL3(scope 7 行と capture 15 行)。RL4 は enum の `String` を `#[source]` に戻す判断が先。

## directories / NSUserDefaults(設定の置き場、委託)

方針どおり新機能無し。Motolii が自前で書いている path 解決と JSON 読み書きのうち、弊社 2 社に引き取れる物と、Motolii 側の整理で済む物を分ける。AP3(NSDocumentController の recents)と重複しない。

### 現状(file:line)

- path の手書き: project.rs:217-219 `settings_dir()` が `HOME` を読んで `Library/Application Support/Motolii` を文字列連結。7be00b0b で他 OS 分岐を消し macOS 限定(host.rs:266 の再 export)。`HOME` が空文字なら `Path::new("")/Library/...` の相対 path になる(`var_os` は空を弾かない)
- 利用箇所: recents.json(project.rs:168・185-190)、window.json(window_frame.rs:13-46)、layout.json(app.rs:460 → dock.rs:144-163)、autosave の Untitled(autosave.rs:45-47)
- JSON の読み書きが **同じ形で 3 回**: `read_to_string → from_str` / `create_dir_all → to_string → fs::write`。3 つとも `fs::write` 直書きで、persist.rs:56-72 `save_atomic`(`.<name>.tmp` → `rename`)を使っていない
- Settings の面の値(settings.rs の Outside dim / Scale)は **どこにも仕舞っていない**: session.rs:302 `UiScale::new(100)`、:319 `frame_dim 75`、app.rs:119 `scale_pct 100` の固定値から起動。窓を閉じると消える
- サムネイルは thumbnail.rs:10-17 のメモリ内 HashMap のみ。disk の cache は今は無い

### directories 6.0.0

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| DR1 | project.rs:217-219 `HOME` 手読み+連結 | `ProjectDirs::from("", "", "Motolii")?.data_dir()`(mac.rs:96 で qualifier/org が空なら bundle_id = "Motolii" → path は今と **同一**、移行不要)。dirs-sys lib.rs:33-37 が空 `HOME` を弾き `getpwuid` へ落ちる | 3 行→1 行、空 HOME の相対 path 事故が消える |
| DR2 | autosave.rs:46 Untitled の逃がし先 | 同じ `data_dir().join("autosave")`。DR1 に付随 | 0 行 |
| DR3 | disk の cache 無し(将来サムネイルを書くなら Application Support に混ざる) | `cache_dir()` = `~/Library/Caches/Motolii`(mac.rs:11)。Time Machine 除外・「ストレージ管理」で消されて良い場所 | 今は用途無し、置き場だけ確保 |
| DR4 | 依存 | motolii/Cargo.lock には未収録。root の Cargo.lock には re_analytics / re_auth 経由で既に 6.0.0 が居る(dirs-sys + option-ext のみ) | 新規 crate 1、重い物無し |

弊社に無い物(正直に): **file I/O は無い**(path を返すだけ)。sandbox 化(App Store)では `HOME` 自体が container path になるので、手書きでも弊社でも結果は同じ — 「sandbox 対応」は売り文句にならない。原子的書き込みも無い。

### NSUserDefaults(objc2-foundation 0.3.2、既に lock に在る、feature `NSUserDefaults` を 1 語)

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| UD1 | Scale ±(tokens.rs:55)・Outside dim(session.rs:319)が仕舞われず毎回 100 / 75 | `standardUserDefaults().setInteger_forKey` / `integerForKey`(NSUserDefaults.rs:170・200)。plist は `~/Library/Preferences/<bundle id>.plist`、`defaults read` で見える | 「設定が消える」が 2 key で直る、file 無し |
| UD2 | 既定値が session.rs:302・319、app.rs:119 の 3 箇所 | `registerDefaults`(:236)で起動時に 1 辞書。値が無い時の既定が 1 箇所に | 帳簿 1 つ |

弊社に無い物(正直に): 値は plist の scalar / 配列 / 辞書。Dock(dock.rs の入れ子 enum `LayoutNode`)や WindowFrame の struct を入れるなら serde → Data 化が要り、JSON file より読みにくくなる。**layout.json / window.json は JSON のまま**が正直な線。`setObject_forKey` / `registerDefaults` は `unsafe`。macOS 限定だが、7be00b0b の裁定「置き場は macOS だけ(v1)」と揃う。

### Motolii 側の整理(弊社ではない、path は弊社)

- (3) recents / window / layout の 3 対を `Store<T: Serialize + DeserializeOwned>`(`load(name) -> Option<T>`、`save(name, &T)`)1 つに畳む。recents は AP3 で消える見込みなので実質 2 対。≒25 行減
- (4) その `save` に persist.rs:56-72 の tmp → rename を流用。layout.json が途中で切れると dock.rs:146 が黙って既定へ戻る(配置が飛ぶ)— 今は起き得る
- H22(scale_factor)は窓の DPI の話で置き場と無関係。H10 は DR2 の path だけ

見込み: DR1+DR2 ≒2 行減、UD1+UD2 で設定の面が初めて残る(≒20 行、feature 1 語)、Store 化で ≒25 行減と原子性。
