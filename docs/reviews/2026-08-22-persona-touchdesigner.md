# ペルソナ調査: TouchDesigner 上がり・表現を求めるクリエイター

日付: 2026-08-22 / 状態: **調査**(read-only・cargo 不使用・第1手 `git merge main` 実施 = 差分ゼロで up to date) / 対象リポ: `next/`(2026-08-20 リセット後の正本)。旧 `crates/`・`docs/generative-user-boundary.md` 等の旧世界文書は**歴史証拠**として引用するが、`next/` に実装が無い限り「未実装」と判定する。

---

## 0. このペルソナが特別な理由

他の3ペルソナ(歌詞動画・Vlog・モーショングラフィックス)は「タイムラインに並べて書き出す」という**工程が既知**の利用者で、調査は「その工程のどこまでが実装済みか」の確認になる。このペルソナは逆に**製品の前提そのものを疑う**——「信号が流れ続ける系を組む」「規則を書く」「出力を入力へ戻す」という要求は、AE 型の「時刻→静的な絵」というこの製品の評価モデル(`Engine::render_frame(&StoreView, t, comp)` 1本、裁定18)と正面衝突する。ゆえにこのペルソナの判定は「拡張の哲学=自由の保護」(2026-08-20 裁定)と vism(プラグイン圏)が**構造として本当に効くか**の唯一の試験になる。fixture に「グリッチトランジション」層が実在すること、および記憶にある「datamosh proof が拡張の哲学を実証した」という一次資料(後述 A-6)からも、利用者自身がこのペルソナに最も近い当事者だと分かる。

---

## A. この人が最初に試すこと(そして詰まる所)——判定表

各行は「通る」「vism 圏で可能(継ぎ目はあるが未実装)」「原理的に不可能(現設計の公理と衝突)」の3値。

| # | やりたいこと | 判定 | 根拠(実在する識別子・裁定) |
|---|---|---|---|
| A1 | パラメータを音に反応させる(音量/スペクトルで scale を動かす) | **原理的に不可能(現状)/ vism圏で将来可能** | `motolii-audio::meter::AudioMeter`(`next/engine/motolii-audio/src/meter.rs:29-`)は再生中の L/R peak を lock-free に持つが**Document へ流す口が無い**(モジュール doc 1行目「Documentへ永続化しない」)。`motolii-media::waveform_peaks(path, buckets)`(`next/engine/motolii-media/src/waveform.rs:71`)は**ファイル全体を先頭から末尾まで一括デコードしてバケツ化する事前解析**(タイムラインの波形表示専用、`next/ui/motolii-timeline-pane/src/waveform_view.rs`)であり、**再生時刻 t における値をプロパティへ渡す経路ではない**。値をプロパティへ渡す型自体(`ParamSource`/`DataTrack`)は 2026-08-20 リセットで**意図的に削除**されている——`next/core/motolii-eval/src/lib.rs:8-12` のモジュール doc が明言:「`ParamSource` は旧 workspace の `ParamDriverPlugin`(4口の1つ)の形そのものだった。**口の形だけが先に入っているのは、口を決めたことにならない**(裁定13)」。ただし`docs/generative-user-boundary.md` §4 経路B「Pure Live `f(t)`」に「解析反応」が明示的に例示されており(現在地欄「既存ParamDriver/LayerSource」——これは旧世界の実装を指す)、**設計思想としては閉じていない**。AE 相当機能「Convert Audio to Keyframes」は `next/reference/normal-map.tsv:480`(id 479)で **採用予定**・bundle=B15(キーフレーム束)——ただしこれは**一度だけ解析してベイクする操作**(裁定150 の「借用が既定」路線)であり、TD が求める**常時追従のライブ反応**とは意味が違う。 |
| A2 | パラメータ同士を繋ぐ(A の回転で B の不透明度を動かす) | **原理的に不可能(コア)/ vism圏で戸は残る** | Lottie 地図(意味の正本、裁定191)は式そのものを不採用にしている——`next/reference/lottie-coverage.tsv:325`「`x` Expression: 不採用。AE の JS 式。処理系を1本抱えることになり軸4に反する」。`next/GOALS.md` の「要らないもの」節も明記:「**JS 文字列式 / AE のグラフエディタ** — 型付きlink + 区間イージング + ParamDriver へ写像」——つまり式は禁止だが**「型付き link」という代替概念**は志向されている。ただし GOALS.md の差別化条件 D5「文字列式が要らない(wiggle/loopOut/ピックウィップを型付きの口で全数カバー)」は**「未・カバレッジ表に穴」**のまま(2026-08-20 時点の記述、`next/GOALS.md` D5 行)。normal-map.tsv 側は UI 動詞として「Add Expression」等5行を **`拡張`** verdict(裁定175、`next/reference/normal-map.tsv:472,476,493,494,1174,1210`)に置き、理由列に統一して「vism抽象化候補・拡張として戸を残す(expression勢力は実在)——コアには入れない」と書く。**結論**: Document の意味としては永久に式を持たない(裁定191 が Lottie を正本にする限り)が、**プラグイン(vism)としてパラメータリンクを実装する戸は明示的に残されている**。 |
| A3 | 出力を入力へ戻す(フィードバック) | **原理的に不可能(現状)/ vism第2号以降で構造的に予定** | `spikes/m5-known-implementation/M5-R0/src/feedback.rs`(README: `spikes/m5-known-implementation/M5-R0/README.md`)が**private probe として実証済み**——Host 所有の2枚 RGBA texture を ping-pong し、「plugin／shaderは履歴を所有しない」設計。ただし README 自身が明記:「これは…Preview／Export接続…を証明しない」。裁定153 の末尾が明示的に温存を決定:「**feedback 系はフレーム間状態を要するため vism 第2号以降へ温存**」(`docs/reviews/2026-08-21-effect-seam-survey.md:259-265` — halftone/feedback は今回の Glow 対象外)。`docs/generative-user-boundary.md` §4 経路D「Feedback / Simulation Bake」は「状態所有、checkpoint、無効化、再生、scrub、予算」を Host が引き受ける設計を既に描いており(現在地「v1.x SCR-4 / SIM群」)、**構造上は不可能ではなく、既に順番待ちの列に並んでいる**。 |
| A4 | 自分でエフェクトを書く(シェーダ/WGSL) | **原理的に不可能(第三者プラグインとしては)——現状は内製限定** | vism 第1号(Glow、裁定153)は**実装済み**——`next/engine/motolii-compositor/src/effects/mod.rs` の `EffectPass`(`Identity`/`Glow{threshold,intensity,radius}`)は「compositor ローカルの **closed enum**(裁定13: trait はまだ作らない)」とモジュール doc 自身が明言。store→compositor の変換は `next/engine/motolii-engine/src/lib.rs:1252` `fn translate_effect_passes` が **`match effect.plugin_id.as_str() { "motolii.glow" => ..., _ => None }`** という**ハードコードされた文字列一致**で行う——`next/engine/motolii-engine/src/lib.rs:1239` のコメントが自ら認める:「**『対応している』plugin_id は `"motolii.glow"` 1本だけ**」。`pub trait` は `next/engine`・`next/core` 全体に**1つも存在しない**(grep 実測ゼロ件)。第三者が WGSL を書いて動的にロードする API・plugin crate・manifest 機構は `next/` に**存在しない**(`find next -iname '*plugin*'` もゼロ件——旧 `crates/motolii-plugin` は歴史遺産)。さらに `next/reference/KNOWN.md:44` が既知の穴を明記:「**effect の複数 pass は連鎖しない(最後勝ち)**——`LayerWithPasses.passes` の各 pass は元 texture を独立に読み共有 scratch へ書く…複数 effect の stack を絵にする時(vism 第2号以降)に直列化が要る」——1個の内製エフェクトでさえスタックが未完成。**結論**: コードを読めば「何を実装すれば効果が1つ増えるか」(store 側 param 追加→ engine の match 腕1本→ compositor 側 shader)は明確に読み取れるが、それは**このリポジトリへの PR**であって、外部プラグイン crate の話ではない。 |
| A5 | 実時間で触りながら探す(スライダ+プレビュー応答性) | **通る(基盤は実装済み)** | ゼロコピー経路(裁定171 v2)は `next/shell/motolii-shell/tests/suite/zero_copy_presenter_fence.rs`・`next/engine/motolii-compositor/tests/with_device.rs`・`next/engine/motolii-engine/tests/zero_copy_matte_text.rs` 等の試験ファイルが実在し、`StagePresenterPipeline`/`Compositor::with_device`(裁定171 が指す型)がコードに存在する(grep 実測)。「再生中の Stage 経路で readback 呼び出し 0」が受入条件。ただし本調査は cargo 不使用のため**実測 fps は未確認**——「基盤コードは着地している」以上の主張はしない。数値ドラッグ+即時プレビューという操作感自体(UI 側)は `next/GOALS.md` M12(Q0)や裁定142(トンマナ)で規律はあるが、TD 的な「パラメータを掴んで揺らしながら画を見る」体験の UI 側実装状況はこの調査のスコープ外(Inspector 側の数値入力は別調査が必要)。 |
| A6 | 意図的に壊す(datamosh・コーデックの誤読) | **原理的に不可能(製品ランタイムとしては)——「戸」はコード実証済みだが接続禁止** | `docs/reviews/2026-08-10-m5-datamosh-codec-domain-private-proof.md` が一次資料。**状態欄が明記**:「M5-DATAMOSH-P0 `DONE / PRIVATE PROBE`、**製品runtime未接続**」。preflight 表の `BUILD` 列は **`FORBIDDEN`**(同文書14-25行)。手法は FFmpeg `noise` bitstream filter で固定 MP4 の2番目の key packet だけを drop し、後続 P-frame の参照欠落を実証(MPEG-4 Part 2、10fps・2GOP固定 fixture)。文書自身の「成立範囲と停止線」節が明言:「H.264 IDR除去は decoder が後続 frame を捨てる場合があり、本fixtureのMPEG-4 Part 2成功を**全codecへ外挿しない**」「製品化は profile と failure contract を別仕様で閉じ、FFmpeg version を recipe identity へ含める **exact target が現れるまで `OBSERVATION / BUILD FORBIDDEN` を維持**」。**「拡張の哲学=自由の保護」を実証した一次資料はこれ**(記憶が指す文書と一致)——「意図的に壊す」という表現がコードで**可能であることの証明**は済んでいるが、**現在は接続禁止**という二重状態。 |
| A7 | 外部入力(MIDI/OSC/カメラ) | **原理的に不可能(コアの公理と衝突)** | `docs/generative-user-boundary.md` §5 表(96-111行)が明示的に禁止:「`mouse/keyboard/camera/network/file` → 編集時入力または Asset import。**レンダ時の非決定入力にしない**。許可能力を明示しsandboxする」。`next/reference/normal-map.tsv` 側でも生中継系が全て不採用——id 127「Record Multi-Camera」(`next/reference/normal-map.tsv:128`)の理由列:「マルチカメラ同時収録はライブ制作(キャプチャ)機能であり、**Motoliiは取込済み素材を編集する合成ツール**(generative-user-boundary.md §1)。領域外」。MIDI は preferences 内の1行のみ言及され(id 1165、`next/reference/normal-map.tsv:1166`)、**プロ用ハードウェア制御パネルとして不採用**(拡張 verdict すら付いていない=戸として残されてすらいない)。OSC への言及は repo 全体に**ゼロ件**。 |

## B. 構造の判定

### B1. vism の口は「読める」か

**部分的に読める、が「外部プラグインの口」としては存在しない。** コード上、Glow(vism 第1号)を追った限り、effect を1つ足すのに必要な変更点は明確に特定できる——(a) store 側は既に汎用(`PropertyId::effect_param`・`StoreView::value_at`、名前つき param map、裁定72)なので新機構ゼロ、(b) `next/engine/motolii-compositor/src/effects/mod.rs` の closed enum `EffectPass` へ variant を1つ足す、(c) `next/engine/motolii-engine/src/lib.rs::translate_effect_passes` の match へ `"motolii.xxx" => ...` を1行足す、(d) shader 本体を `effects/` 配下に書く。**この経路自体は読み取れる**——ただし読み取れるのは「**このリポジトリのソースを fork して PR する経路**」であって、TouchDesigner ユーザーが期待する「別プロセス/別 crate として書いた拡張を、製品を再ビルドせず動的にロードする」形の口は無い。`pub trait` が `next/` 全体にゼロ件であることがこれを裏付ける。裁定13(「拡張の trait はまだ作らない、2人目の利用者が現れるまで待つ」)は effect については裁定153 で「compositor が2人目の利用者」として**部分的に解消**されたが、それは「compositor 内部で closed enum を使う」形の解消であり、**外部利用者(TD 出身のプラグイン作者)を2人目として数えた解消ではない**。したがって「**このペルソナが2人目の利用者か**」という発注書の問いへの答えは——**まだ違う**。裁定13 が待っている「2人目の利用者」は今のところ社内(compositor)にとどまっており、外部作者が実際にプラグインを書こうとして詰まった実績が無いと、trait 設計の口は開かれない。

### B2.「意味の正本は Lottie」(裁定191)がこの人を縛るか——最重要判定

**縛る。ただし縛るのは Document の意味だけで、vism 圏は縛らない。この二層構造がこのペルソナの生死を分ける。**

- Lottie に無い表現(生成的・実時間・フィードバック)は、**Document のデータとしては原理的に不可能**——裁定191 は「データ意味(パス・シェイプ・mask・変形・keyframe 等)の正本は `lottie-coverage.tsv`」であり「Lottie に無い語彙を足す時は、地図の行 id を示せない限り発明とみなす」と明言する。expression(`x` property)も `mn`(Match Name)も Lottie schema には存在するが**採用は不採用**(`lottie-coverage.tsv:211,295,325,575`)——理由は一貫して「処理系を1本抱えることになり軸4に反する」「JS 文字列式は要らないもの(GOALS)」。つまり**このペルソナが求める「規則を書く」「信号を繋ぐ」という行為を Document の一級市民にすることは、意味の正本を変えない限り絶対に起きない**。
- しかし裁定193(2026-08-22)が「パネル級の機能は『無い』ではなく vism(拡張)」を明言し、裁定175 が「拡張(vism)= 戸を残す、閉じない」という新 verdict を作った時点で、**Lottie 縛りは「コアの外に出す」ことと矛盾しなくなった**——`docs/reviews/2026-08-22-core-vs-vism-classification.md:17` が明言:「純化はコアの外形を絞ることであって、機能を消すことではない」。Expression 系5行・Convert Audio to Keyframes・ペイント族15行等が全て「拡張」verdict で**戸を残したまま**台帳に載っている(不採用ではない)ことが、この二層構造が単なる建前でなく実際に運用されていることの証拠。
- **結論**: Lottie はこの人を「Document の中では」縛るが、「vism 圏としては」縛らない、という**意図的な設計**が既に成立している。ただし vism 圏は**まだ空の戸**(trait 未着手・plugin crate 未着手)なので、**現時点でこの人を実際に迎え入れる実装は無い**——約束(裁定)はあるが器はまだ無い。

### B3. 保存できるか

**手続き的に作った物は`.rrd`に残らない。焼き付けるしかない、という以前に「手続き的に作る」機構自体がまだ無い。** 保存正本は上流 `.rrd` そのまま(裁定55)で、Document の実体は component の値——ParamDriver/式/リンクという「規則」を表現する型が `next/` に存在しない以上、保存する対象そのものが無い。唯一保存できるのは (a) 通常の keyframe track(A1 の「音声解析→ベイク」路線がここに落ちる)と (b) effect の named param(Glow の threshold/intensity/radius、これは animatable なので `.rrd` に残る、裁定72)。「規則」を保存する形が生まれるとしたら、それは vism 圏で plugin が持つ設定値(param map)としてであって、Document が「グラフ」や「式」を意味として持つ日は来ない(裁定191 が覆らない限り)。

## C. 最初に見限られる所

TouchDesigner 出身者が最も早く離脱する分岐点は **A7(外部入力)ではなく A2(パラメータリンク)** だと判定する。理由: A7 は「編集ソフトとライブ制作ツールは別物」という説明が一般的にも通りやすく、Resolve/Premiere も同じ境界を持つため驚きが小さい。しかし **A2(A の値で B を動かす)は TD だけでなく AE 経験者(pick whip)にも当たり前の操作**で、「効果を重ねる」「値を繋ぐ」という発想の入口に最初に立つ。ここで `x` Expression が不採用と分かった瞬間、「型付き link」という代替(GOALS.md が言及するのみで未実装、D5 が「カバレッジ表に穴」)も実体が無いため、**この人にとって最初の10分で「この製品には信号を流す場所が無い」と分かってしまう**。次に踏むのが A4(自分でエフェクトを書く)で、Glow の実装を見て「効果は増やせそうだ」と期待した直後に `pub trait` ゼロ件・plugin crate ゼロ件という事実に当たり、「結局ソースを fork するしかない」と気づいて離脱する。

## D. この人を繋ぎ止めるための最小実装(順序つき)

1. **「型付き link」の型を GOALS.md の約束どおり作る**(D5 の穴を塞ぐ)——`PropertyId` A の `value_at(t)` を `PropertyId` B の入力として宣言できる最小の型(裁定191 に触れない: Document の component としてではなく、**vism 圏の最初の実装**として)。スコープは「A の評価値をそのまま/線形変換して B へ渡す」だけに絞り、JS 処理系は持たない(軸4 を守る)。これが無いと A1(音声反応)も A2(パラメータリンク)も土台が無い。
2. **A1 の最小形を「型付き link」の初回消費者にする**——`motolii-audio::AudioMeter` の snapshot(peak_l/peak_r)を、明示的な "audio meter" ソースとして (1) の link の入力側に置けるようにする。裁定150(先例借用)に従い、AE の「Convert Audio to Keyframes」がベイク型で先に採用予定(B15)なので、**まずベイク経路を実装し**(既存 keyframe 機構だけで足りる、新機構ゼロ)、その後にライブ版を link 型の2人目の消費者として追加する二段構え。
3. **vism の口を trait として明文化する**(裁定13 の「2人目の利用者」を effect の内部消費(裁定153)から一段引き上げる)——(1)(2) が実装された時点で「値を生成して Document へ渡す」という形が Glow の「値を消費する」形と揃うため、`次に来る2人目の利用者は誰か`を裁定として一度確定させる。trait 自体は薄く(「時刻を受けて Value を返す」程度)。
4. **effect スタックの直列化**(`KNOWN.md:44` の穴)を先に直す——vism 第1号がまだ「複数 pass が連鎖しない」状態では、TD 的な「エフェクトを重ねて質感を作る」試行が最初の2個目で壊れる。これは新規機能ではなく**既存 vism 第1号の完成**であり、優先度は (1)(2) と並行できる。
5. **feedback の最小 vism(vism 第2号)**——`docs/generative-user-boundary.md` 経路D の設計(Host 所有 checkpoint・固定 step・明示 seed)に沿って、`spikes/m5-known-implementation/M5-R0/src/feedback.rs` を製品化する。裁定153 が既に「vism第2号以降」と名指ししているので、順序としては自然な次点。**readback/preview-export一致の受入条件が最も重い**(§9 全経路共通の5条件)ため、他の4項目より後に置く。
6. **datamosh の製品接続は最後**——BUILD FORBIDDEN の解除条件(exact target・FFmpeg version を recipe identity へ含める)がまだ無く、(1)〜(5) の型(link・vism trait)が固まってから「codec-domain な effect」という新しいクラスをどう vism へ載せるかを再設計する方が手戻りが少ない。

外部入力(MIDI/OSC/カメラ、A7)は最小実装キューに**含めない**——`docs/generative-user-boundary.md` の公理(レンダ時の非決定入力を禁じる)を覆す利用者裁定が無い限り、これは「まだ実装していない」ではなく「設計上ここには来ない」に分類されるため、繋ぎ止め策の対象外とする。

## 逸脱

- 発注書は「motolii-audio の waveform/peaks は実装済み」としていたが、実際には**2つの別物**が存在する——(a) `AudioMeter`(再生中のリアルタイム peak、Document 非接続)と (b) `waveform_peaks`(ファイル全体の事前一括解析、タイムライン波形**表示専用**)。どちらも「音に反応してプロパティを動かす」経路には使われておらず、A1 の判定は「実装済みの部品はあるが、繋ぐ経路が無い」という結線待ち未満(そもそも結線対象の型が無い)の状態と判定した。
- 裁定175 は `next/DECISIONS.md` の連番台帳に未転記(裁定178 の注記どおり 161〜177 は review 文書側にのみ実在)。本調査は `docs/reviews/2026-08-22-map-audit-rulings.md` と `normal-map.tsv` の実データを直接引用して裏を取った。
- `docs/generative-user-boundary.md` は 2026-07-15 作成、2026-08-20 の `next/` リセットより前の文書。5経路(Materialize/Pure Live/Temporal Window/Feedback-Bake/External)という枠組み自体を `next/DECISIONS.md` が明示的に継承・言及した箇所は見つからなかった(裁定153/193 は独立に同じ結論——vism への温存——へ到達している)。本調査ではこの文書を**現行の反証されていない設計哲学**として扱ったが、`next/` 側の裁定台帳に正式採録されているとまでは断定していない。
- A5(実時間応答性)は着地したコード(型・試験ファイル)の存在確認までで、実測 fps・UI 側のスライダ操作感は cargo 不使用の規律により未検証。
