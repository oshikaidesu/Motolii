# 構造解決台帳 — Motolii が構造で解決している物(2026-08-22)

調査レーン(読み取り専用)の成果物。目的= map 精査(6レンズ)の前提となる一次資料。

**読んだ物**: `docs/decision-index.md`(全裁定、旧workspace分〜173含む)・`next/DECISIONS.md`(裁定1〜160)・
`docs/reviews/2026-08-21-lane-board.md`(完了表)・`next/reference/KNOWN.md`・`next/GOALS.md`・
`next/README.md`・`next/reference/normal-map.tsv`(1,551行、該当語彙 grep)・関連 review 文書(iced/rerun/blend/mask/camera/text 等の裁定原文)。

**判定の型**: 既存ソフト(AE/Premiere/Resolve/CapCut)の機能には「そのソフト自身の構造的欠陥を補償する物」が
混ざる。Motolii が根を構造で解いているなら、対応する補償機能は不要になる。以下は3節構成:

1. **構造解決済み**(N=14) — 根治が実測・merge 済み
2. **構造で解決すると宣言済みだが未完**(M=9) — 裁定はあるが実装が追いついていない(先に補償機能を消すと危険な柵)
3. **委託技術の解決範囲**(supervisor 追加指示、2026-08-22) — Motolii 自身でなく依存技術が解いている範囲・その正直な境界・今後の見込み

---

## 1. 構造解決済み(14件)

| # | 構造解決 | 解いた根 | 不要になる補償機能の型(map 行id) | 正直な境界 |
|---|---|---|---|---|
| 1 | **ゼロコピー GPU 再生**(裁定166/170/171、M4 merge `61d5189e`) | プレビューの遅さ — iced 2MiB 同期アップロード予算超過による「絵が描かれない」チラつき(実測)、CPU readback 1.7ms/1080p | プリレンダー・RAM プレビュー・ディスク/メモリキャッシュのパージ操作系(map 696 Pre-render・697/705 RAM Preview・1148/1194/1207/1236 Memory&Disk Cache・Purge) | 巨大 comp・重エフェクト構成での実測はまだ(M4 検収記録)。市松 ON は CPU 合成フォールバック(readback 発生)。**該当 map 行はまだ全部「採用予定」のまま** — 構造解決が map 判定に未反映(L2 対象) |
| 2 | **Intent/KeyframeTrack 意味論 store + `apply_all` 単一ゲートウェイ**(裁定2/47/48/56/118、R0 実測) | 破壊的編集・undo 深度制限・「1操作が複数編集ステップに分裂して Undo が壊れる」問題 | flatten/焼き込み/ラスタ化系、undo 深度制限に由来する「セーフティコピー」運用の一部 | 書き出しの顔(flatten して保存)は必要。**undo 履歴の GC 方針は空席**(KNOWN 明記 — 無限成長を許容する設計判断がまだ無い) |
| 3 | **RationalTime(有理数フレーム時刻)+ TM-4 柵**(裁定10/32/64、`tm4_no_float_frame_math.rs`) | フレームずれ — f64/fps 混在の丸め誤差の累積(NTSC 30000/1001 を表せない Lottie `fr:f64` が反面教師) | 時刻計算のアドホックな特殊ケース処理(浮動小数点の再合わせ) | ffmpeg サイドカー側のタイムスタンプ精度は境界外(委託技術節参照)。「Conform Lock」「Match Frame」等の編集ワークフロー系 map 行(188/189/232-234/612/919/920 等)は別問題で構造未解決のまま採用予定 |
| 4 | **素材台帳 + fingerprint(Document 所有)**(裁定162、η着地 `d5370a0e`) | メディア迷子 — bin-first ワークフローで「どこにも登録されない素材」が生まれる | 重複整理・再リンク系の一部 | 再リンク UI 未実装(browser-seam-survey EVIDENCE_GAP)。multi-GB 素材の fingerprint(全読み hash)は B7(4ms)予算超過リスク未計測(KNOWN)。map 行id: 未特定(relink/offline media/media manager 語彙 grep 一致なし) |
| 5 | **実時間 audio クロック**(A2 merge `f026e7cc`、cpal コールバックタイムスタンプ直読) | AV ずれ — wall-clock 駆動では音声同期が構造的に効かない(rerun TimeControl が反面教師、KNOWN) | 別体の drift 補正 UI・resync 操作系 | seek 時リング容量 4,096 フレーム(~85ms)分だけ古い音が流れきる既知の制約。map 行id: 未特定(resync/drift 語彙 grep 一致なし) |
| 6 | **Preview=Export 単一評価経路**(裁定15/18/26/44、D1/M15 済) | プレビューと書き出しの評価経路が2本に分岐しズレる問題(旧「direct re_renderer 禁止」が反面教師) | 「Draft/Full quality」のような**別経路**の品質切替(解像度を落として速くする系) | byte 一致ではなく Y≤8(h264 の YUV420 変換のため)。alpha 付き書き出しは未解決(項目9参照)。**裁定21 の発見が重要**: 「解像度を落としても速くならない(帯域律速)」ため、map 844-846「Fast Previews > Draft/Fast Draft/Off」の根拠自体が Motolii では成立しない可能性が高い(未反映・L2 対象) |
| 7 | **解像度 cap(Stage 下縁状態帯、μ 裁定163・merge `3737d5c4`)** | プレビュー速度と解像度のトレードオフを UI から選べる形にする必要性(裁定21 の proxy 方針を UI で受ける口) | map **1450「Fast Previews > Adaptive Resolution」に相当する実装が既に着地**(Auto/½/¼ cap、既存 sqrt 予算と min 合成) | **map 1450 はまだ「採用予定」のまま — 実装済みの verdict 更新漏れの可能性(L2 の具体的な指摘対象)**。他の Fast Previews 系(844-846,1451)は未対応 |
| 8 | **`Document::revision()` による変化検出**(裁定42) | `generation()` が undo/redo で変わらず、front が独自の dirty 判定を持たざるを得ない問題 | front 側の手製差分検出・dirty flag 手動管理系のワークアラウンド | 特筆すべき境界なし(内部設計、UI に直接露出しない) |
| 9 | **`LayerPlacement` 単一合成点(裁定41/69)+ camera/z 統合(裁定113/115/116、D12)** | 空間分割(2D/3Dモード切替・レイヤー別カメラ)という AE の構造的複雑さ | 「enable 3D」トグル、レイヤー別カメラ、ショット切替機構 | near-plane 手前でクリップ(裁定116 残穴a)。shell にカメラを触る口(gizmo)はまだ乏しい(裁定124 でスクラッチ方針のみ決定、実装は静止描画の視覚受入止まり) |
| 10 | **mask 被覆代数(MK1/MK2、mode 別単位元設計)**(裁定108/133、merge `8fa92130`) | AE 実機の「先頭 mask の mode で絵が変わる」という非自明な意味論のアドホック対応 | mask 合成の特殊ケース処理 | r2 負荷依存 flake 観測(KNOWN)。mask 座標系(comp 絶対 vs layer local)は Canvas を呼び手が組む設計で MK3 へ先送り(EVIDENCE_GAP) |
| 11 | **blend 逐次合成 fork accessor(main_target read アクセサ)**(裁定161、BL1b merge `cbbe4f2b`) | CPU 側 layer 毎の gamma 往復(unmultiply→srgb→premultiply)によるバイト不一致(真因特定済み) | 独自 composite シェーダの複製実装(スクラッチ再発明の回避、wrapper-over-hack 裁定の直接適用) | 分離可能11モード(BL3)・非分離4モード(BL4)はまだ発注段階(項目5参照、未完節へ) |
| 12 | **track_cache(revision 鍵付き KeyframeTrack キャッシュ)**(裁定140) | Timeline 投影コストの97%が serde_json 解析という慢性予算超過(裁定11「track まるごと1 component」の代償) | front 側の独自キャッシュ層(段差の温床になるはずだった構造上の穴) | 負荷依存の flaky 予算テスト2本(`edit_storm_with_the_real_track_type`・r2)は既知(緩めない) |
| 13 | **保存 = 上流 `.rrd` + 履歴畳み込み**(裁定55/56) | 保存形式発明のコスト・store 全 edit 刻みをそのまま書くと編集回数に比例して肥大化(実測18.8MB→30KB) | 独自ファイルフォーマットのバージョニング/マイグレーション機構 | fork rev を上げると旧 project が読めなくなるリスク(裁定55 明記の「危険」)。Cmd+S・未保存●・閉じる確認の UI 結線はまだ(GOALS M11 部分) |
| 14 | **Timeline `tick_steps` 純関数(目盛り梯子)**(ν裁定、merge 済み) | 全尺等分による半端な目盛り値(7.5s 級)という表示の不整合 | 特殊ケースの目盛り値テーブル手打ち | 特筆すべき境界なし |

---

## 2. 構造で解決すると宣言済みだが未完(9件)

**重要**: この節は map 精査で補償機能を早消しする**柵**である。裁定はあるが実装(またはカバレッジ)が
追いついていない物を先に「不要」と判定すると、まだ塞がっていない穴が露出する。

| # | 宣言 | 現状 | なぜ危険か |
|---|---|---|---|
| 1 | **Group 層 + 単一再帰変換木**(裁定173、2026-08-22決定) — プリコンポの「まとめるためだけの用法」を不要にする | **H1(store への親変換合成挿入)が「走行中(未返却)」**(lane-board)。H2(Timeline ツリー行)のみ着地(`016b29e3`)、H3(親選択UI)・H4(シェイプ入れ子)は未着手 | **map 909「Pre-compose…」/910「Precompose selected layers」は既に不採用の verdict がついている**(裁定119/GOALS 根拠)。しかし親 transform の実合成(H1)がまだ resolve に届いていない — verdict が実装を先回りしている教科書的な例。**L2 の最重要チェック対象** |
| 2 | **拡張の口 = trait 1本**(裁定6/13、D6) | GOALS.md 明記「**意図的に未着手**(DECISIONS #13)」— 2人目の利用者(compositor)が現れるまで待つ方針 | 「独自 SDK・プラグイン契約の複雑さ」という補償機能不要化の宣言はあるが、口自体が無いので比較のしようがない。map の plugin/SDK 系行を不採用にする根拠としてまだ使えない |
| 3 | **alpha 付き書き出し + shell 市松合成**(裁定16→132 で部分的に覆る) | 合成器は実 alpha を readback できる(1行改修で実証済み)が、**export のファイル出力と shell 側の透明合成は未実装**(裁定132 明記) | 「Transparent プリセットで書き出せる」を前提にした補償機能(市松以外の透明可視化ワークアラウンド)を消すのは早い |
| 4 | **effect 複数 pass の直列合成**(裁定153、vism 第1号 Glow のみ) | KNOWN 明記: 「**effect の複数 pass は連鎖しない(最後勝ち)**」。現在は単一 pass のみなので実害なし、vism 第2号以降で直列化が要る | 「effect スタックが正しく合成される」を前提にした map 判定(複数エフェクト重ねがけ系)はまだ根拠にできない |
| 5 | **BL3(分離可能11 blend mode)/ BL4(非分離4 blend mode)**(裁定161 完了後に「発注可能になった」と記載のみ) | 未発注(lane-board 記載止まり)。実装済みは Normal/Add のみ(α レーン) | Lottie blend mode 0..15 語彙採用宣言(裁定67)があるが、実装は2種のみ。「blend mode が全部使える」という補償機能不要化はまだ言えない |
| 6 | **isolate / freeze(プリコンポの残り2用法)**(裁定119 起草、裁定173 で「対象外・別裁定」と明記) | 未実装・未裁定 | GOALS D4「プリコンポ地獄が無い」は Group+fold だけでは半分(まとめる用法のみ)。隔離(isolate)・性能キャッシュ(freeze)用途の補償機能はまだ生きている可能性 |
| 7 | **D5「文字列式が要らない」(型付き link で全数カバー)** | GOALS.md 明記「**未・カバレッジ表に穴**」。一方で **normal-map.tsv は既に Expression 系6行(471/475/492/493/1173/1209)を D5 根拠で不採用にしている** | **自己矛盾(c類)**: 宣言の実装未完のまま、map 側は先に判定を確定させている。型付き link のカバレッジが AE Expression の全用途(特に pick whip の任意プロパティ参照)を覆っているかは未検証 |
| 8 | **S 空間スコアの器具(裁定163/164)** | 正典(`docs/ui-spatial-score.md`)と台帳のみで、**S3(視覚動線)・S5b(画素判定)の器具が未建設**(session-handoff 明記)。S0/S1/S2 の一部(atlas dump・s-score.py)のみ υ レーンで着地 | 「UI 入口の普通度を機械判定できる」という前提で map の UI 系行を裁くと、まだ機械化できていない S3/S5b の判定を人力で代用している状態が見えなくなる |
| 9 | **ギズモ**(裁定114→124 で上書き、スクラッチ方針に確定) | 「意味=AE手本・見た目=3D-DCC文法」の方針決定のみ。**実装は第1波の静止描画視覚受入止まり**(裁定124) | カメラ/レイヤーの直接操作(ハンドルドラッグ)を前提にした map 判定はまだ早い(Inspector 数値欄経由の操作で当面代替) |

---

## 3. 委託技術の解決範囲(supervisor 追加指示、2026-08-22)

各技術につき: **今解決している範囲**(実測/KNOWN 引用) / **正直な境界** / **今後解決しそうな物**(一次資料URL)。
探索範囲: WebSearch(2026年時点の公開情報)+ GitHub issue 直接確認2件(iced #3281、rerun #2852)。新規 web 調査は
「今後の見込み」列に限定し、「今解決している範囲」は既存 KNOWN/裁定からの引用を優先した(効率化)。

| 技術 | 今解決している範囲 | 正直な境界 | 今後解決しそうな物 |
|---|---|---|---|
| **iced(0.15-dev、fork `host-seams`)** | Elm型宣言的UI・段差ゼロ(操作→Message列→update が iced_test で一級不変量、裁定 iced再評価調査)。AccessKit 統合済み(役割/ラベル/値等)・Tab フォーカス循環・IME 可変サイズ preedit window 対応(公式サイト記載)。wgpu 29.0.4 への統一を fork 2箇所(seam)だけで達成(裁定170、api差分は使用面338箇所で実質ゼロ・ω調査実測) | canvas/slider は `iced_test::Simulator` から構造的に不可視(KNOWN)。`iced::time::every` はこの workspace では使えない(tokio/smol feature 不足、KNOWN実測)。Theme/Style は色・境界・影のみで寸法を運べない(API実測、裁定117)。バス係数=事実上1(hecrj、iced実績調査 2026-08-18)。0.14時点でWindows IME変換ポップアップ位置bug(#3189)が放置との調査記録あり | 1.0 リリースは作者本人が「まだ遠い」と明言(SE Radio 713、2026-08-18時点の調査)。AccessKit は外部コントリビュートPR(#3281)を作者が「自分でやる」とクローズ済み(2026-08-22 WebFetch確認、closed・not merged)— 現在は本体に統合されているため Motolii は追随のみでよい。iced.rs 公式サイト記載のIME(preedit可変サイズ)・focus管理(on_focus_gained/lost)は0.14〜0.15系で拡充中 |
| **rerun store(`re_chunk_store`/`re_entity_db`/`re_query`)** | Document の実体・undo=`edit` timeline の時間移動として R0 実測 6/6 通過(query 9µs・保存3.5MB/1000編集、裁定8/9)。custom component(`re_types_core`)で AE の意味を fork 無改造で建てられる(裁定4) | `LatestAtQuery` が単一timelineのみで2次元query不可(裁定8の訂正)。undo履歴のGC方針は空席(KNOWN/項目2) | fork(`oshikaidesu/rerun`)は上流master と概ね同一(ドリフト最小、ω調査「fork は upstream master と sha 同一」)。rev bump の都度 R0 常設試験で往復検証する運用が既定路線(裁定9) |
| **re_renderer** | 合成・GPU の実体(裁定3/14)。層=`TexturedRect`、depth_offset で重ね順、`multiplicative_tint` で不透明度。YUV→RGB は `SourceImageDataFormat::Yuv` に委託(裁定23、ffmpeg `yuv420p` バイト列がそのまま一致)。headless GPU の instance/adapter/device limits は `device_caps` をそのまま使用(裁定17)。alpha は `blend_with_background: Premultiplied` 1行で readback に実 alpha が乗る(fork改造ゼロ、裁定132) | **mipmap 自動生成は無い(TODO のまま)**(KNOWN、2026-08-20確認)→ preview高速化は素材proxy側でのみ買える(裁定21実測: 素材1/4縮小で40ms→5.9ms、MSAA off でも9%程度)。effectの複数pass連鎖は上記未完項目4。ゼロコピー化は adapter 未公開という gap があったが `DeviceCaps::from_device`/`RenderContext::new_from_device` の fork 追加(裁定170 M3)で解決済み | WebSearch では re_renderer のmipmap issue番号を特定できず(検索範囲: GitHub issue検索・rerun releasesページ、2026年時点でヒットなし)。EVIDENCE_GAP として明記。rerun 公式リリースノート(0.24、2026)は動画ストリーミング対応の強化に言及しており、re_video 経由の周辺機能拡充が続いている傾向は確認できた(https://rerun.io/blog/release-0.24) |
| **re_video** | MP4 の `moov` だけ先読みするストリーミング対応(裁定24 理由(c)は誤りだったと訂正済み、KNOWN) | コンテナが MP4 のみ・API が再生指向(フレーム正確ランダムアクセス契約と不一致)・全バイトをメモリに載せる・**encode/mux を持たない**(decode専用、裁定24理由(a)(b)(c)(d)のうち(d)は真のまま)。この4点により Motolii は re_video を採らずffmpeg サイドカーを維持(裁定24) | rerun 0.24 リリースノートは「H.264以外のコーデック対応は今後」と明言(WebSearch確認、https://github.com/rerun-io/rerun/releases/tag/0.24.0)。現状は H.264 のみで、Motolii が要求する mov/mkv/webm/静止画の多様なコンテナ対応が re_video 単体で揃う見込みは低い(この4点が変わったら再裁定、裁定24の既定路線どおり) |
| **wgpu(iced/re_renderer 経由、直接依存ではない)** | Motolii は wgpu を直接持たず iced fork と re_renderer fork の両方から間接的に使う。**バージョン統一(29.0.4への単一化)が裁定170の中核成果**(旧: iced=27.0.1 / re_renderer=29.0.4 で型不一致・ゼロコピー不可 → fork2箇所の追加コンストラクタで解消) | wgpu 自体のロードマップは Motolii の意思決定に直接関与しない(バージョン追随のみ)。GL backend は `Limited` tier固定という保守的判断が残る(裁定170、device_caps 由来) | 深追いしていない(EVIDENCE_GAP — 探索範囲: このレーンでは wgpu 単体のリリースノートは検索せず、iced/re_renderer 経由の統合状況のみ確認。理由: Motolii の意思決定は常に iced/re_renderer 経由の間接依存として扱われており、wgpu 単体のロードマップが直接の意思決定材料になった裁定は無い) |
| **Lottie圏(仕様+dotLottie)** | 保存形式・transform・text・effect param・blend mode 等の**AE意味論の一次資料**として採用(裁定54/58/65/67等、"実質OSSのAE解析")。`reference/lottie.schema.json` を機械可読の地図として使用(裁定68、656項目) | Motolii は Lottie の**保存フォーマットそのもの**は採らない(裁定55、上流 `.rrd` を使う)。style_spans(範囲スタイル)は Lottie から凍結できず Rive を別途参照(裁定77/82)。パーセント単位・`animated`フラグ・`e`(deprecated)等は不採用(裁定65)。Lottie は編集器ではなく**交換/再生フォーマット**なので、Motolii の書き出し(export)がLottie/dotLottie形式を吐く経路は無い(意味論の参照元であって配布形式ではない) | LottieFiles は2026年に dotLottie v2 で状態機械(interactivity)・音声・テーマを1ファイルに統合する方向("Lottie Power Stack 2026"、https://lottiefiles.com/blog/inside-lottiefiles/the-lottie-power-stack-2026)。Motolii が将来 Lottie/dotLottie **書き出し**(他エディタとの相互運用)を持つなら、この state machine 語彙が参照先になりうるが、現時点で裁定・着手なし(EVIDENCE_GAP) |
| **ffmpeg サイドカー(`motolii-media` 経由)** | フレーム正確 seek 付き decode・probe・**encode・mux**(`mux_soundtrack`/`mux_mixed_pcm`、裁定30「音声はPCMを作るだけでよい」)。YUV→RGBAの正規化をffmpeg側に寄せる(色空間解釈の一元化)。プロセス境界隔離でデコーダクラッシュを防ぐ設計(リンクせずサイドカー、B-2対策) | mov/mkv/webm/静止画を含む広いコンテナ対応と引き換えに、**リンクでなくプロセス起動のオーバーヘッド**を払う(意図的トレードオフ、裁定24)。alpha付き書き出し(ProRes/PNG連番)は未実装(項目9、上記未完節) | ffmpeg自体は成熟した外部プロジェクトでMotolii側の裁定に現れる将来課題は無い。今回は「今解決している範囲」の確認に留め、ffmpeg本体のロードマップ検索は行っていない(EVIDENCE_GAP — 探索範囲: KNOWN/decision-index の既存記述のみ、外部ロードマップは未検索。理由: ffmpegはMotoliiの意思決定を左右する変数として裁定に現れたことがなく、優先度低と判断) |
| **音声束(cpal/symphonia/rubato/rtrb)** | cpal のみが生のコールバックタイムスタンプ(`OutputCallbackInfo::timestamp()`)を露出するため採用(KNOWN、rodio/kiraは高レベルすぎてクロック所有権の契約D4/D5を守れないため不採用)。rtrb でリングバッファ実装(裁定135、旧`ring.rs`の再発明367行を置換)。symphonia/rubato は `motolii-audio` 限定依存(裁定135、2人目の利用者まで workspace へ上げない) | 音声の規律crate候補(`assert_no_alloc`・`audio-thread-priority`)はまだ柵として導入していない — コールバック内alloc禁止の機械化・スレッド優先度設定の穴が残る(KNOWN明記) | 大きな変更は見込んでいない(音声の調達調査DONE、KNOWN — Tracktion/GES/MLT/Ardour等の全界隈比較の上で「意図的自作」が確定済み、再訪条件はMotoliiが別目的でGStreamerに厚く依存した場合のみ)。新規web調査は行っていない(EVIDENCE_GAP — 探索範囲: KNOWN記載の既存調査結論のみ。理由: 2026-08-20の調査で「全界隈掃討済み」と明記されており再調査の動機が無い) |
| **cosmic-text(iced 0.15/M01でシェイピングに使用)** | iced fork 経由でテキストシェイピングを委託。M01(裁定170)の一部として iced_test/font スタック更新が実測済み(利用者合否「見え方問題なし」、2026-08-22) | canvas::Text の Ellipsis はフィールドのみで描画側(`geometry/text.rs`)が無視する既知の穴(TL-arch調査、裁定165文脈)— widget text なら効くが canvas 経由では効かない、という区別がある | WebSearch で cosmic-text が rustybuzz から HarfRust へシェイピングバックエンドを置換したとの情報を確認(https://github.com/pop-os/cosmic-text)。2026年時点の詳細ロードマップ(バージョン計画等)はヒットせず、EVIDENCE_GAP(探索範囲: WebSearch1回のみ、GitHub releasesページの深追いは未実施) |

---

## 使い方の注記(map 精査 L2 向け)

この台帳は**三層フィルタ**として使う:

1. **構造解決済み**(節1)の根に対応する map 行(不採用/採用予定いずれも)は、**verdict が構造解決と整合しているか**を機械照合できる。特に **項目1(map 696/697/705/1148/1194/1207/1236)と項目7(map 1450)は「採用予定」のまま構造解決が反映されておらず、L2 の最優先確認対象**。
2. **未完節**(節2)に該当する根拠で map 行を不採用/採用予定にしている場合、**実装が追いつくまで verdict を確定させない**(項目1の「H1未着地なのに909/910が不採用確定」・項目7の「D5未完なのに471等6行が不採用確定」が現物の自己矛盾例)。
3. **委託技術節**(節3)の「正直な境界」に該当する map 行は、**Motolii 自身の裁定でも委託技術の実装でも解けていない**唯一の真の判断対象になる(例: alpha付き書き出し・mipmap不在による素材proxy依存・effect複数pass連鎖)。

## EVIDENCE_GAP

- **re_renderer mipmap TODO のissue番号**: WebSearch/GitHub検索で特定できず。KNOWN記載(2026-08-20確認)のソースコード上のTODOコメントが唯一の一次資料。
- **wgpu単体のロードマップ**: 未検索(Motoliiの意思決定はiced/re_renderer経由の間接依存としてのみ現れ、直接裁定になったことがないため優先度を下げた)。
- **ffmpeg本体のロードマップ**: 未検索(同上の理由)。
- **音声束(cpal等)の2026年時点の外部動向**: 未検索(KNOWN「全界隈掃討済み」の既存結論を優先)。
- **cosmic-textの詳細バージョン計画**: WebSearch1回のみ、releasesページ深追い未実施。
- **素材台帳(項目4)・実時間audioクロック(項目5)の対応map行id**: normal-map.tsvの grep(relink/offline media/media manager/resync/drift 等)で一致なし。該当行が存在しない、または別の語彙で表現されている可能性(要再検索)。
- **節2項目5(BL3/BL4)・項目9(ギズモ)**: lane-boardの記述のみに依拠。normal-map.tsvでの対応行の網羅的特定は未実施(時間制約)。
