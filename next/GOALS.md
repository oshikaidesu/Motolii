# Motolii の目的と合否条件 — Hero creation

Motolii の製品定義は、一般的な動画ソフトの縮小版ではなく、制作者が自分や作品の hero を立ち上げ、
制作と発信へ進む動機を生むことにある。一般的な動画ソフトの条件は、その目的を成立させるための基礎床であって、
製品の同一性そのものではない。

**この表が「完成」の検収面**。既存台帳(`../docs/`)から組み立てたもので、ここで新しく発明していない。
出所欄は `../docs/` 配下。状態欄は **新 workspace(`next/`)** の実装状況。

旧 workspace で実装済みでも、新側に無ければ「未」と書く。旧側の実装は移植元であって成果ではない。

## 状態欄の語彙(裁定219)

**`済` は窓を開けて実機で確認した物だけ**に使う。台帳と正典が独立に同じ誤り
(「バック実装が在る」を「利用者に届く」と取り違える)をした実測が4件あったため、
語彙を分ける:

| 語 | 意味 |
|---|---|
| **済** | **実機で確認した** |
| **結線済(実機未確認)** | 静的には利用者の動線から到達する。窓での確認はまだ |
| **結線待ち** | バック実装は在るが到達路が無い(裁定192 の単位) |
| **未** | 実装が無い |

「部分」は**範囲を明記した上で**のみ使ってよい。

**2026-08-23 時点で `済` と書ける行はほぼ無い** — この日の実装は全て静的検査だけで着地しており、
**実機検分は一度も行われていない**。これは後退ではなく事実の表示である。

## 問題起点の優先順位(裁定244)

この台帳の1粒は「機能があるか」だけでなく、**何の失敗・停滞・不安を解決するか**を持つ。
出典に機能名しかない場合も、採用理由は利用者の問題と観測可能な結果へ翻訳してから決める。
「便利そう」は問題の名前にならない。主張が弱い粒をhero creationの根拠に昇格させない。

| 優先度 | 解決する問題 | 扱い |
|---|---|---|
| **P0 信頼・安全** | 作品の喪失、誤った出力、無反応、面ごとの不一致、復旧不能 | 最優先。hero以前に、触っても壊れないことを保証する |
| **P1 制作ループ** | 素材を入れ、編集し、結果を見て、再生・保存・書き出しまで進められない | 次点。制作を止めず、最初の結果へ到達させる |
| **P2 hero表現** | 結果が一般的な素材の並べ替えで終わり、作品の主役・動機・個性を立ち上げられない | Motoliiの主張。点群・3D・文字MV・音楽同期・時間変化はこちら |
| **P3 摩擦削減** | 同じ制作を何度も行う際の手数、探索、視線、移行コストが大きい | P0〜P2を壊さない範囲で進める。単独で製品の主張にしない |
| **P4 便利** | あれば快適だが、無くても作品の意味・安全・制作ループが成立する | 主張を抑え、後回し。先回り実装しない |

現行の基礎床20粒をこの問題レンズで読むと次の順になる。元の検収条件・出典・状態欄は変更せず、
ここでは優先順位と解決対象だけを補う。

| 粒 | 解決する問題 | 結果 | 優先度 |
|---|---|---|---|
| M1 | 起動時に制作の入口がなく、既存作品にも戻れない | 空 project または既存 project から制作を開始できる | P0 |
| M2 | 素材を持っているのに制作へ入れられず、拒否理由も分からない | 素材が理由つきでDocumentへ入る | P1 |
| M3 | 配置した結果が見えず、操作が正しいか判断できない | TimelineとStageが同じ結果を示す | P1 |
| M4 | 素材終端や尺の境界で、存在しない映像を出力してしまう | 時間境界が正しく、背景も正しく見える | P0 |
| M5 | 時間上の編集が意図どおりに確定せず、直し方が分からない | 移動・trim・snap・復元が予測可能に働く | P1 |
| M6 | 置いた素材を分ける・消す・複製するという基本修正ができない | 失敗を作り直さず修正できる | P1 |
| M7 | 既存素材や編集結果を再利用できず、同じ作業を繰り返す | Copy/Cut/Pasteが編集の継続を支える | P1 |
| M8 | 動きと音の結果を体験できず、heroの良し悪しを判断できない | 再生・scrub・音・playheadが同期する | P1 |
| M9 | 書き出し結果が再現されず、完成物を外へ持ち出せない | Previewと音声mux込みExportが一致する | P0 |
| M10 | 試すことが怖く、1回の操作で戻せない | 1 gesture = 1 Undoで安全に試せる | P0 |
| M11 | 作業を中断・再起動すると作品や状態を失う | 保存・復帰・終了確認が成立する | P0 |
| M12 | 見た目が操作を約束するのに、押すと何も起きない | 偽の入口を置かず、触れる物は反応する | P0 |
| M13 | 失敗が無言で、次に何を直せばよいか分からない | 拒否が理由つきで観測できる | P0 |
| M14 | Stage・Timeline・Inspectorが別々の真実を表示する | 選択・時刻・幾何が一つのDocumentを映す | P0 |
| M15 | Previewで良く見えた作品がExportで変わる | 同じ評価関数で結果を再現する | P0 |
| M16 | 入力やrender失敗がpanic・クラッシュ・喪失へつながる | 失敗しても作品と画面を失わない | P0 |
| M17 | 空の状態が制作の最初の一歩を拒む | 空でもplace・scrub・keymapが働く | P1 |
| M18 | 見たい時間・範囲へ移動するだけで制作が止まる | Zoom/Fitが視線移動の摩擦を下げる | P3 |
| M19 | 値を時間で変化させられず、heroの動きが作れない | property単位のkeyframe操作が働く | P2 |
| M20 | 面を跨いだ編集・IME・Undoが互いに干渉する | どの面からでも安全に修正できる | P0 |

この表でP3/P4に分類される機能は、実装されても「Motoliiの本質」とは宣伝しない。
P2はheroを生む直接の表現、P0/P1はheroへ到達するための条件であり、三つを同じ「機能数」として数えない。

なお、`reference/normal-map.tsv` は他製品に存在する語彙を並べた**カウンター／候補在庫**であり、
その全行がMotoliiのバックログになるわけではない。`採用予定`または`採用済`へ進める粒だけが、
この問題表のM/D粒またはcomponentへ接続し、問題・結果・優先度を持つ。未採用・未判定の粒を
「不足」と数えてP3/P4の実装を先回りしない。

### 最小コアの剪定(裁定245〜247)

`normal-map.tsv` は候補在庫として残したまま、現行の hero 縦切りと基礎床へ直接つながらない
純粋な拡張束を `採用予定` から `拡張` へ戻した。第一剪定で507粒、第二剪定で410粒、計917粒を
移し、静的スナップショットは全1,551粒のうち `採用予定 954 → 37`、`拡張 59 → 976` となった。
続く裁定248の因果判定で、既存の正本へ収まり独立実装を増やさない128粒を
`scope=absorbed / verdict=構造吸収`へ移し、現在の `拡張` は848粒となった。
最終的に残した37粒は、入口・素材差替え・基本編集・再生・安全、property時間変化、点群用3Dカメラに
直接対応する未着地候補である。各行の理由欄に `PROBLEM / OUTCOME / PRIORITY` を付け、機能名だけで
採用を正当化しない。マスク/シェイプ、カラー、トランジション、グラフ編集、テキスト詳細、
音声制作、パネル/ワークスペース、細かな3Dビュー/ギズモ、マーカーやnudge等は候補在庫として残す。
行や既存の `採用済` の証拠は削除していない。構造吸収128粒も候補在庫には残るが、独立した
実装・検収の粒量には数えない。この数字は静的台帳の現在地であり、実機受入やコンパイル成功を
意味しない。

### 因果で見る構造吸収(裁定248)

「普通のソフトに名前がある」ことは、Motoliiの独立した実装粒であることを意味しない。
まず問題と観測結果を固定し、既存の正本がその結果を一度だけ生むなら、個別の入口・モード・
enum・パネル・ショートカットは作らない。`normal-map.tsv` では行を消さず、`構造吸収` として
在庫に残す。これは意味を捨てる判定ではなく、独立した状態・owner・検収を増やさない判定である。

| 因果 | 問題 | Motoliiの解決粒(証拠) | 観測可能な結果 | 構造へ吸収する粒の型 |
|---|---|---|---|---|
| C1 | 1操作が複数履歴・専用undoへ分裂する | `Document::apply/apply_all` が唯一の書き口で同じedit刻みに原子化する (`next/core/motolii-store/src/document.rs:502-545`) | 1操作を1回のundoで戻せる | History/Revival Undo/専用履歴パネル |
| C2 | 時間編集ごとに別モード・別intentが増える | `LayerTiming` と `SetTiming` に move/trim/split/speed を収める (`next/core/motolii-store/src/document.rs:95-147`, `next/core/motolii-store/src/lib.rs:311`) | 時間上の変更が同じ正本で評価される | Add Edit/Razor/Blade/Trimの別入口・別alias |
| C3 | 追加・置換の入口が面ごとに増え、書き方が分岐する | Browser cardの意図を一つのdispatchへ畳み、`Document::apply/apply_all`へ送る (`next/shell/motolii-shell/src/create.rs:272-305`) | 追加/置換が同じDocumentへ届く | File/Media Browser/Effects panel等の入口重複 |
| C4 | propertyごとに値・補間・keyframe UIが別機構になる | `PropertyId`/`KeyframeTrack` と providerの `ParameterDescriptor` (`next/core/motolii-eval/src/track.rs:234-303`, `next/ui/motolii-inspector-pane/src/device.rs:63-71,283-300`) | 一つのproperty系で値と時間変化を扱える | Keyframe mode/reveal/editor/move/reverse/holdの専用粒 |
| C5 | 3D view・gizmoのプリセットがカメラ状態を増殖させる | `Composition.camera` と既存property/Stage gizmo (`next/DECISIONS.md:123-125`) | 同じカメラ/transform結果を操作できる | Front/Back/Custom View/個別gizmo起動 |
| C6 | PreviewとExportが別の画を計算する | `Engine::render_frame` を共有する (`next/shell/motolii-shell/src/render.rs:232-236`) | 同じDocumentから同じframe truthが出る | 別preview/export render経路や表示だけのPreview panel |
| C7 | transformの便利操作ごとに専用状態・書き口が増える | Property/LayerPlacement/SetOrderへ書く (`next/core/motolii-store/src/document.rs:95-109`, `next/DECISIONS.md:61,112`) | 位置・拡縮・回転・順序の結果だけが変わる | reset/center/flip/increment/fit/stackのalias・tool |
| C8 | workspace/panelのプリセットが製品固有の第二状態になる | Browser/Stage/Inspector/Timelineの固定pane構造 (`next/shell/motolii-shell/src/pane_layout.rs:109-124`) | 同じ4役割で制作を続けられる | workspace preset/page/maximize/dockの専用粒 |

この分類で独立に残すのは「問題の結果」が増える粒である。heroの主役を立ち上げるカメラ操作、
property別イージング、素材復旧、音声同期、保存/書き出しの安全は、汎用構造に吸収せず残す。
逆に、同じ結果へ到達する別名・別パネル・別ショートカットは、最小コアの粒量を増やさない。

### 技術委託とスクラッチ抑制(裁定250)

ここでいう委託は、人やレーンへの作業委託ではない。各問題を解くときに、意味・評価・I/O・
描画のどこを **既存構造、上流、先例、移植元へ預けられるか**、どこだけがMotolii固有の
継ぎ目として自前に残るかを決めることである。`verdict` の意味判定と混ぜず、
`reference/generated/technical-delegation.tsv` を `map_id` でjoinして見る。

| 問題束 | 技術の委託先 | スクラッチ境界 |
|---|---|---|
| project / save / recovery | `Document` / `Composition` / `DocumentIO` / `.rrd` / filesystem | **抑制**。lifecycleのadapterだけ。第二のproject正本は作らない |
| asset import / replace | `Document` / `Asset` / filesystem / FFmpeg | **抑制**。decoder・cache・replace stateを二重化しない |
| timeline edit / split / trim | `apply_all` / `LayerTiming` / Timeline grammar / AE・Premiere先例 | **抑制**。専用edit modeを増やさない |
| keyframe / retime | `PropertyId` / `KeyframeTrack` / provider catalog / Lottie | **抑制**。値・補間・keyframeの別機構を作らない |
| playback / audio sync | decode・resample・deviceは上流/移植元、mix・program・`PlaybackClock`は既存のMotolii音声正本 | **許容**。同期を成立させるmix/clockだけ自前。音声全体を無条件に外注しない |
| hero camera | `Composition.camera` / `re_renderer` / `glam` / AE・Blender先例 | **抑制**が既定。向きのpose propertyという不足した継ぎ目だけ**許容** |
| WIRE / marker / navigation | 既存のpane message → `Shell::update` → `Intent` → `Document` | **抑制**。結線を意味の新実装にしない |

したがって、`採用予定` は「スクラッチで作る」という意味ではない。技術台帳の
`scratch_policy=抑制` を既定にし、`許容`を付ける場合だけ `scratch_boundary` と
`evidence` を必須にする。現在は構造吸収128粒と最小コア/結線待ち51粒の179粒を監査済み、
残り1,372粒は未監査として残している。候補を増やすために未監査を勝手に自前実装へ倒さない。

## 基礎床 — 一般的な動画ソフトとして壊してはいけない条件

| # | 条件(観測可能な形) | 出所 | `next/` |
|---|---|---|---|
| M1 | 起動して project を新規/既存で開ける。無ければスタート画面 | ux-check-first-ten-minutes | 未 |
| M2 | Finder からドロップで素材が入る。開けない物は**理由つきで skip**(黙って消えない) | ux-check P3/P5、first-touch観察 | **済**。3本まとめて落として1操作(winit は1ファイル1事象なので描画要求を区切りにする)。拒否はファイル名つきで status 帯へ |
| M3 | 置いた clip が Timeline に立ち、**Stage に絵が出る**。待たされない | ui-inherited-grammar-gap | **部分→ほぼ済**。Stage に絵が出て、Timeline pane 第1波(行・bar・ruler・playhead・scrub・選択)が実機で稼働(裁定120)。残りはドロップ→行が立つ一周の実機確認 |
| M4 | clip の尺は min(source, comp残り)。source 終端の先はフリーズせず背景 | first-real-run 欠陥(1) | **済**。`LayerTiming::place` が尺を決め、配置の外は描かない(フレーム全体は落とさない) |
| M5 | drag=移動 / 端drag=trim / release=確定 / Esc=復元。snap は clip端・key・playhead・loop端・0・終端 | normal-timeline-prior-art | 未 |
| M6 | split(Cmd+K)・Delete・複製(Cmd+D)・複数選択(Shift/Cmd/marquee/Cmd+A) | 同上 | **部分(split のみ未)**。Delete/複製/複数選択は C-2 で結線済(実機未確認)。**split(Cmd+K)だけ未統合** — `context.rs:55` が「`SplitAtPlayhead` は宣言のみで shell へ未統合」と明記 |
| M7 | **Copy / Cut / Paste が効く** — 旧 egui は menu に項目があるのに何も起きない(Q0違反の現物) | 同上、egui能力台帳§2 | **結線済(実機未確認)**。MB-0 で Edit メニューへ Copy/Paste/Cut/Undo/Redo/SelectAll/DeselectAll/Duplicate を配線、既存 shortcut と併存(S6) |
| M8 | Space で再生。**音が鳴り**、playhead が音に同期。scrub で Stage 追従 | ui-inherited-grammar-gap Tier0 | 未 |
| M9 | Export → mp4。**音声mux込み**。報告フレーム数=現物、cancel で残骸なし | concept、first-real-run 欠陥(2) | **結線済(実機未確認)**。C-3 が `mux_mixed_pcm` を `export_ops.rs:508` から呼び、出力 mp4 に音声ストリームが在ることを機械で検査。export は専用スレッドへ逃がして UI が止まらず cancel が届く。残骸は `out_path` を一時 path 経由にして残さない。**既知の限度**: mux 直前の cancel 再チェックは `Cancel::is_cancelled` が非 pub のため不可(わずかに遅れて効く) |
| M10 | Document を変える操作は**1回の Undo で戻る**。1 gesture = 1 Undo | ui-quality-bar Q2 | **済**。`Document::apply_all` が複数 intent を1つの edit 刻みへ書く。運転席が「layer 追加」「3本ドロップ」の両方で Undo 1回を確認。ドラッグは途中経過を pane が持ち確定の1件だけが intent なので元から1 undo |
| M11 | Cmd+S・未保存●・閉じる確認・**再起動で続きが開く** | ux-check P2/P5、外部診断F-01 | **結線済(実機未確認)**。C-1 が全4項目を結線 — Cmd+S(既知パスは無言で上書き・`input.rs:321`)/ 未保存●(`• name — Motolii`)/ ×ボタンの確認(`exit_on_close_request:false` + `close_requests()`)/ 再起動で前回を開く(User Settings 相当の sidecar に直近1件)。autosave からの復帰確認も追加。**MRU 一覧は未** |
| M12 | **触れそうな物は全部機能する**。未実装の chrome を置かない(disabled も不可=撤去) | ui-quality-bar **Q0**(利用者裁定) | 未 |
| M13 | **無反応ゼロ**。拒否は理由がその場で分かる。旧 iced は拒否を `let _ =` で捨てていた | ui-quality-bar Q3、能力台帳§5-2 | **部分**。読み口が「無い」と「読めない」を区別し、shell の拒否は status 帯へ出る。運転席が「戻せない」「開けない素材」の2件を確認。全操作を通した確認は Timeline 後 |
| M14 | 選択・時刻・幾何の正本は1つ。全面が同じ真実を映す | ui-quality-bar Q5 | **済**。幾何=store、fps/解像度/尺=Document の `Composition`(裁定40)、選択/playhead=`Session` 1箇所(裁定46/107)。Timeline と Stage が同じ正本を映すことを利用者が実機確認(裁定120) |
| M15 | **Preview = Export**。同じ評価関数を通る | concept 絶対規律、DECISIONS #15 | **済**。(1) 経路の一本性は依存グラフが守る(export は compositor を引かない) (2) 入力の一本性は `Composition` が Document にあることで守る(裁定40) (3) 現物での照合は可逆書き出しを decode し直して Y の最大差 ≤ 8(h264 が YUV420 を通るため byte 一致にはならない) |
| M16 | どの入力でも panic/クラッシュ/喪失なし。render 失敗でも画面を空にしない | ui-quality-bar Q6 | 未 |
| M17 | 空 project は空として表示。空でも place/scrub/keymap が効く | ui-quality-bar Q7 | 未 |
| M18 | Zoom(カーソル下の時刻を保つ)と Fit | prior-art 必須12件 | **部分**。Zoom In/Out/Fit は C-4 が `Cmd+=`/`Cmd+-`/`Cmd+9` へ結線(実機未確認)。**Zoom to 100% は未** — viewport bounds が `Program::draw(&self)` でしか手に入らず `Shell` へ書き戻せない(D-3/C-4 が独立に同じ結論)。**「カーソル下の時刻を保つ」は未確認** |
| M19 | keyframe の追加/削除/移動が property 単位で効く | 同上 | **部分**(store/eval/書き出しまで済、UI が無い) |
| M20 | undo/redo/delete がどの面からでも。TextInput 中はテキスト優先。IME を壊さない | ui-quality-bar Q9 | 未 |

## 参考基線 — 典型的な動画ソフトの慣習

この節の粒は、典型的であること自体を採用理由にしない。

| 参考群 | 解決する問題 | 優先度 |
|---|---|---|
| context menu、カーソル、矢印/Home/End | 操作の意味が画面ごとに違い、既存知識が移行できない | P3 |
| rename、label、lock、mute/solo、fold | 複数素材の見分け・整理・事故防止が難しい | P2〜P3 |
| group、marker、loop、playhead追従、waveform | 時間構造と音楽の中で、狙った瞬間を見失う | P2 |
| easing、Time Remap、parent、Effect UI | 値を置くだけでなく、時間変化と関係性を表現したい | P2 |
| Inspector行、Stageハンドル、Browser drag | 表現を探すだけでなく、直接触って形にしたい | P2 |
| Export設定・進捗、日本語・空白名、性能 | 制作結果を安全に外へ出せず、環境差で止まる | P0〜P1 |
| テキストレイヤー | 文字を作品の主役として動かせない | P2(製品の芯) |

したがって、この節の項目を一括して「標準だから必要」とは扱わない。各粒が上の問題を解決するか、
単なる快適化かを判定してから、基礎床・hero表現・後回しへ送る。

context menu / カーソル言語(trim端=resize、clip=grab、marquee=crosshair)/ 矢印1フレーム送り・Home/End /
行の rename・label色・lock・mute-solo / fold と**グループ化**(プリコンポの代替)/ M キーでマーカー /
soundtrack の波形帯 / ループ区間再生 / 再生中の playhead 追従 / **区間イージングの切替** /
Time Remap / 親子(型付き Follow/LookAt)/ Effect の追加削除 UI /
**Inspector に Anchor・Scale・Rotation 行** / Stage の bounding box と scale/rotate ハンドル /
Browser から **drag で配置** / Export 設定 UI と割合進捗 /
時間予算 B1〜B7(定常 p99 ≤ 8ms、gesture 中に 16.7ms 超を連続2枚出さない)/
トンマナ不変の機械検収 / 日本語・スペース入りファイル名 / soundtrack の差し替え・gain・clip音声mix /
**テキストレイヤー**(2026-08-20 利用者裁定で**製品の芯**へ格上げ。「文字MV」)

## Hero creation — Motolii が存在する理由

ここでは、基礎床の上で「作った結果を見て、さらに作りたくなる」ことへ寄与する意味を評価する。
一般的な NLE と同じ機能でも、hero を立ち上げる動線に効くなら採用する。逆に、一般的であっても
目的へ寄与しない機能を、数合わせだけで追加しない。

| 粒 | 解決する問題 | 優先度 |
|---|---|---|
| D1 | Previewと完成物の不一致で、作品への信頼を失う | P0 |
| D2 | 深い試行錯誤でUndoが破綻し、表現を試せない | P0 |
| D3 | 動きが直線的で、感情や勢いを表せない | P2 |
| D4 | 構造が深くなり、heroの調整箇所を見失う | P2 |
| D5 | 文字列式を覚えないと、表現の再利用や接続ができない | P2 |
| D6 | 拡張するたびにコアを壊し、表現の入口が狭くなる | P3 |
| D7 | 長い作品を完成物として書き出せず、発信へ進めない | P1 |
| D8 | 音楽の拍と映像の変化が結び付かず、MVの起点を失う | P2 |
| D9 | 最初の結果が遠く、制作意欲が立ち上がる前に離脱する | P1 |
| D10 | 音楽に反応する表現を手作業だけで作れない | P2 |
| D11 | 文字を作品の主役として細かく動かせない | P2 |
| D12 | 2Dの平面表現だけでは、空間的なheroの印象を作れない | P2 |

| # | 条件 | `next/` |
|---|---|---|
| D1 | **Preview = Export を機械で示す** | **済**(M15 と同じ3点。byte 一致ではなく Y ≤ 8 なのは codec の都合) |
| D2 | **Undo が壊れない・深さで落ちない**(AE の痛点Aの逆) | **済**(R0)。GC 方針は空席 |
| D3 | ネイティブな区間イージング(Bounce/Elastic/Steps、オーバーシュート可) | 部分(Bezier まで) |
| D4 | プリコンポ地獄が無い(グループ+fold+ベイク) | 未 |
| D5 | **文字列式が要らない**(wiggle/loopOut/ピックウィップを型付きの口で全数カバー) | 未・カバレッジ表に穴 |
| D6 | **拡張の口が trait 1本**。first/third-party が同じ口 | 意図的に未着手(DECISIONS #13) |
| D7 | 3〜5分(5,400〜9,000フレーム)を実用スループットで書き出せる | 未計測 |
| D8 | ビート検出・拍グリッド吸着(MV では編集の起点) | 空席 |
| D9 | 起動〜最初の結果が数秒 | 数値バーが空席 |
| D10 | first-party パーティクル(音楽同期) | 未 |
| D11 | **文字MV が作れる** — 1つのテキストレイヤー内で文字範囲ごとにフォント・サイズを変えられ(`style_spans`、裁定82)、文字ごとのアニメーションが AE のアニメータ模型で表せる(裁定75) | 未。Lottie から 39項目を採用予定、範囲スタイルだけ別の先例待ち |
| D12 | **rerun だからこそ: 3D**。世界は1つ(全員 z=0 既定の 2.5D、空間を分けない — 裁定113)で、**preview は透視カメラのインパクト重視**(層の z を動かした瞬間に視差が出る)。将来の点群・深度クラウド・3Dメッシュの撮影は `re_renderer` がネイティブに持つ(point_cloud / depth_cloud / mesh_renderer、rev 483b855 実確認) | **camera 束は済**(裁定116。`Composition.camera` の center/zoom/roll・`position.z`・pinned 属性・`Projection::Perspective` 化。全層 z=0・既定カメラで旧正射影と画素一致 = 済)。点群・深度クラウド・3Dメッシュの撮影は引き続き未(裁定113 の Spatial 変種と同じく将来の additive スコープ、向きの表現はまだ開けない) |

## 要らないもの — 欠落ではなく設計上の除外

**これらを「足りない」と数えない。**

- **trim family 一式**(ripple / roll / slip / slide / insert / overwrite / lift / extract / sync lock)
  — 自由配置土台の裁定(2026-08-19)。gapless packing 前提なので既存 gesture と機構的に衝突する
- 「以降を押し出す」修飾キー drag(便利機能として先送り)
- **プリコンポ / Nest / Compound clip** — グループ化+ベイクへ置換済み
- **ノードグラフ UI** — ユーザーに見せない
- **JS 文字列式 / AE のグラフエディタ** — 型付きlink + 区間イージング + ParamDriver へ写像
- IK / キャラリグ / 状態を積む本物のシミュレーション / 120Hz 最適化
- 動的配布 marketplace / 第三者 SDK / 独自 plugin UI / VST 互換
- **第二 runtime・第二評価経路**(背骨2)
- rerun の viewer 層 / egui shell / `ui/motolii-rn`

空席のまま(禁止ではない): 3点編集、A/V link-unlink、マルチカム、J/K/L shuttle。

## バック優先(2026-08-20 裁定50)

iced を採ったのは「**バックができていれば UI は後から生えてくる**」ため。
よって UI の見栄えより Document の意味の穴を先に塞ぐ。バック側に残っている穴:

| 穴 | 状態 |
|---|---|
| layer の時間(配置・trim・頭出し) | **塞いだ**(裁定51〜53) |
| 保存・読込(M11) | **結線済(実機未確認)**。形式は上流の `.rrd` そのまま、保存時に履歴を畳む。shell の Cmd+S は 2026-08-23 に結線 |
| Transform 全軸(anchor / scale / rotation) | **未**。今あるのは position / size / opacity |
| 音声(decode / mix / 再生 / export mux) | **部分**。export mux は結線済(実機未確認・C-3。`AudioProgram::mix_audio` が同期純関数として既に在り、新規 PCM 生成は不要だった)。**decode / 再生の時計は未** |
| 再生の時計(M8) | 未 |
| 拡張の口 | 意図的に未着手(裁定13) |

## 順序

1. ~~media を移植して compositor に**実素材**を流す~~ — **済**(2026-08-20)
2. ~~export で **鎖を閉じる**~~ — **済**(2026-08-20。音声 mux だけ残)。M15 をここで閉じた
3. iced shell の骨。**背骨1を型で作る**(`StoreView` と Intent の送り口しか渡らない)
4. **核の一周**を1本ずつ: ドロップ → clip が立つ → Stage に絵 → Space で音同期再生 → Export
5. 編集の必須12件(M5〜M7。**Copy-Paste の死に席を最初に潰す**)
6. 保存と復帰(M11)
7. **Q0 と Q3 を柵にする** — 「触れそうな物は全部機能する」「拒否は必ず理由が出る」を機械検査に。
   旧実装は同じ穴を2回開けているので、後から一括では潰せない
8. keyframe + 区間イージング + Inspector の Transform 全行(M19 / 標準)
9. 品質バー B1〜B7 を計測に乗せる
10. 実機で `ux-check-first-ten-minutes` の台本を通す —
    **旧実装の「良い」判定は全て機械検証止まりで、実機の手触りは人間未検証**

7 を 8 より前に置くのは、Q0/Q3 が「機能を足すたびに再発する型の穴」だから。

## いま空いている穴(2026-08-20 の敵対的レビューで判明)

順序3(shell)に入る前に塞ぐべきもの:

- ~~comp 設定(fps・解像度・尺)が Document に無い~~ — **塞いだ**(裁定40)。
  ~~選択・playhead はまだ無い~~ — **塞いだ**(裁定46/107。shell の `Session`、undo 対象外)
- ~~gesture を1 undo へ畳む口が無い~~ — **塞いだ**(裁定48。`Document::apply_all`)
- ~~`ResolvedLayer` と `Layer` が同じ形で2つある~~ — **塞いだ**(裁定41。共有の `LayerPlacement`)
- ~~`generation()` が undo/redo で変わらない~~ — **塞いだ**(裁定42。`revision()`)
