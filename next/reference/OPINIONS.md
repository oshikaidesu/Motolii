# Motolii 自身の意見(借りていない物)一覧

**この文書の役割**: motolii は根幹を持たず、コンセプトすら Lottie(意味)・rerun(GPU合成の一部)・
iced(UI)・taffy(レイアウト)・cosmic-text(文字組み)・ffmpeg(復号/多重化)・Ableton(変調の作法)・
rerun(instance の数え方)からほぼ全てを借りているパッチワークである(利用者裁定)。だからこそ、
**借りていない物 = motolii 自身の意見**を1箇所へ集める必要がある。次にどの布を借りられるか
(= 意見が無い領域)を決める唯一の基準になる。

判定基準: **「これを外部の先例に置き換えたら、製品は壊れるか」**。壊れるなら意見、壊れないなら借り物。
新しい意見はここで作らない — `next/DECISIONS.md`(裁定1〜214)・`next/reference/CANON.md`・
`next/GOALS.md`・`docs/decision-index.md` に既にある物だけを集めた。

4列目(コスト)が本表の要である。意見はコストを伴い、そのコストが自前実装の正体だから
(例: 有理時間 → `motolii-core`/`motolii-eval`/`motolii-vector` 約19,000行)。

## 意見一覧(18件)

| # | 意見 | 出典(裁定番号・ファイル) | これを曲げると何が壊れるか | この意見のせいで自前になった物(コスト) |
|---|---|---|---|---|
| 1 | **有理時間(`RationalTime`)** — fps・keyframe の `t` を有理数で持つ。float フレーム番号に戻さない | 裁定10・32・64・78。`next/reference/CANON.md` A表(旧 `docs/pitfalls-and-roadmap.md`「VFR正規化」も系譜) | `30000/1001` 等の非整数 fps が表せなくなり、Lottie の `fr: f64` と同じ精度劣化に戻る。時刻⇄フレーム写像の正本が複数化する(TM-4 柵が守る不変量が消える) | rerun store を自前にした原因そのもの。`motolii-core`(有理数時刻)+`motolii-eval`(keyframe補間・bezier分割)+`motolii-vector` 約19,000行(移植・裁定10) |
| 2 | **Preview = Export の単一経路** | 裁定15・18・21・33・116(byte一致実測)・171(ゼロコピー化後も維持)。`next/GOALS.md` D1/M15、`docs/concept.md`「絶対規律」 | 「書き出し専用の速い道」ができ、プレビューと書き出しで違う絵が出るバグを構造的に防げなくなる(AE の痛点の再現) | `Engine::render_frame(view,t,comp)` 唯一入口(裁定18)。依存グラフで `motolii-export` が `motolii-compositor` を引けない型分離(裁定33)。**comp 解像度を落とす近道を自ら禁じた**ため素材 proxy が preview 高速化の唯一の手段になった(裁定21の実測: comp半分解像度でも16〜36msぶれるが素材1/4なら5.9ms)。ゼロコピー化後も export/screenshot だけ headless 読み戻しを残しbyte一致を機械実証し続ける二重経路維持コスト(裁定171) |
| 3 | **`Intent` が Document への唯一の書き口(背骨1)** | 裁定212(24枝の閉じた列挙)、裁定48・118(b)(`apply_all`) | store への直接書き込みの近道ができ、正本が2つになる・undo が壊れる | `Intent` enum の read-modify-write 規律(`SetTiming`/`SetSource`/`LayerAttrsPatch` 等)。「入口があるか」を機械判定する check(裁定212 — `SetCameraTrack`/`RemoveAsset` 等の入口ゼロを1コマンドで検出) |
| 4 | **決定性(実時間クロックを評価に入れない・ランダム化は seed 必須)** | `motolii-audio` 決定論的 mix・wall-clock 不使用の `PlaybackClock`(lib.rs 冒頭)、`motolii-compositor::render_is_deterministic`、裁定75・101・121(`TextRandomize{seed}`) | 同じ Document から毎回違う絵/音が出て、意見2(Preview=Export)の機械実証が原理的に不可能になる | `PlaybackClock`(`frames_supplied − device_wait`)という自前クロック抽象。`TextRandomize{seed}` — 裁定101 が明記する通り**先例が1つも無い**自前設計(Lottie の `rn` は seed 無し、Rive にはランダム化自体が無い) |
| 5 | **ゼロコピー合成(再生中は GPU 常駐、CPU readback 0)** | 裁定171 v2(`docs/reviews/2026-08-22-zero-copy-seam-decision.md`)。前身の裁定26→44→45→171 という往復の末に確定 | 通常の CPU readback へ戻り、大きい解像度で描画のちらつき・fps低下が再発する(裁定126 の DRAFT バッジ問題の再燃) | `Compositor::render_to_texture`/`Engine::with_device`/`render_resolved_to_texture`(additive API)。`StagePresenterPipeline`。readback 呼び出し回数の metrics カウンタ。市松 ON 時だけ CPU 合成へ落ちるフォールパック経路 |
| 6 | **可視性原理 S6(隠れていないから読める・複数入口)** | 裁定195・205・214。`ableton-visibility-principle` memory | 右クリックだけが唯一の入口になり、「触れそうで触れない」物が構造的に生まれる(Q0違反の温床) | 全機能に「第二の入口」(Layer メニュー等)を作る義務。S6監査を UI 発注の受入条件に機械的に組み込む仕組み(裁定195 — Lock/Hide/Solo・ラベル色・Split の単一入口3件を実際に検出) |
| 7 | **意図優先の原則**(UI の動詞は意図を語り、機構を語らない) | 裁定174(`docs/reviews/2026-08-22-intent-first-grouping-decision.md`) | parent ポインタ編集 UI(AE の Parent 列・pick-whip)のような機構露出が復活する | 親ドロップダウンの代わりに、グループ化専用動詞(⌘G/⌘⇧G)をゼロから作る必要 — `LayerSource::Group` 生成+1 undo+ Ungroup 時の world 位置保存という専用機構一式 |
| 8 | **1意図 = 1つの家**(統合できる操作は統合・視線の一点性) | 裁定177(`docs/reviews/2026-08-22-intent-bundling-decision.md`)、適用例= 裁定184(Inspector集約)・205(Browser集約) | AE のように同じ意図の設定が複数窓・複数パネルに散らばり、視線があちこちに行く(利用者が名指しした AE の悪い部分そのもの) | `normal-map.tsv` の bundle 列 + check による強制。Inspector が B46/B42/B04/B16/B38/B02/B03/B07 を型別 section として集約(別窓化しない、裁定184)。「追加する」動詞を Browser 1箇所へ統一(裁定205) |
| 9 | **接続子は加算・型付き port を作らない** | 裁定213(利用者裁定「値は加算で解決」)、`docs/reviews/2026-08-23-vism-decision-recovery.md` | 旧世界の Kit が型付き port + provider→consumer で肥大化し、Host integration 層が立つ前に世界が一度終わった実績の再演 | `EffectInstance::enabled` を bool から track へ(Lottie `en` 対応)。`Value` 型の Hold区分(Bool/Enum/LayerId=加算不可)という型による境界規定。負値許容に伴う「0横断・override無し・上限無し・負の time_offset」の5判断項目を実装レーンへ個別発注する運転コスト |
| 10 | **Inspector に映る物は全て時間軸に乗る**(AE のスイッチ非キーフレームを不採用、Ableton側を採る) | 裁定214(利用者裁定「AE はエフェクトのオンオフしかなくて、変だと思った」) | AE のようにレイヤースイッチだけ時間に乗らない例外だらけの製品になる | 裁定92(テキストstyleをv1でキーフレーム化しない)を失効させ実装範囲を拡張。「出力に現れるか」を境界とする再分類(テキスト内容=property、ラベル色=乗らない等)。`PositionZ` のような「track はあるが入口が無い」項目を機械監査で継続的に洗い出す義務 |
| 11 | **「無い」と「壊れている」を同義にしない** | 裁定37、裁定118(d)(`track_json_components` の Err区分) | 壊れた Document が静かに既定値へフォールバックし、利用者には「値が勝手に戻った」としか見えない事故が起きる(M13/Q3 違反) | 読み口全体を `Result<Option<T>>` にする規律。「値が無い」(`None`)と「型が合わず読めない」(`Err`)を区別する追加実装(裁定118(d)で実際に踏んだ取りこぼしの修正) |
| 12 | **Undo は `edit` timeline の時間移動 — 自前 undo 機構を作らない** | 裁定2・12(tombstone)・42(`revision()`)・47(`mark_undo_floor`)・48/118(b)(`apply_all`) | AE のような undo スタック深度制限問題が戻る。`drop_entity_path` 相当を使うと undo で戻せなくなる | tombstone 方式の削除(ハード削除をしない)。`revision()`(store世代+edit位置、`generation()` 単体では undo/redo を捉えられない)。`apply_all` のバッチ内 `Err` を1 edit 刻みへ畳んで undo 履歴に残さない処理(裁定118(b)) |
| 13 | **選択・時刻・幾何の正本は1つ** | `next/GOALS.md` M14、裁定41(`LayerPlacement` 共有)・42(`revision()`)・46/107(`Session`) | front の各 pane が独自の写しを持ち、ズレた真実を映す UI になる(旧 `inspector_model.rs` が3世代化した構造の再演) | front が持ってよい状態を `Session`(選択・再生位置のみ)1箇所に限定。Document の写しを front に持たせない設計(裁定46/107 — rerun 上流も選択/playhead をどちらの store にも置かず plain struct で持つ先例と一致させた) |
| 14 | **順序非依存の明示参照(並べ替えが合成結果を黙って変えない)** | 裁定66(matte)・73(shape の `group.it` 不採用)・85〜87・89(text style spans) | AE の matte `tt`/`td`/`tp`(`tp` 省略時に「1つ上のレイヤー」を暗黙参照)や Lottie の `group.it`(兄弟順スコープ)のように、レイヤーの並べ替えが合成結果を静かに変える事故が起きる — 編集ソフトとしては致命的 | matte を `LayerId` 参照1フィールドへ畳む(裁定66)。shape を「1つのパス源」で `group.it` 不採用(裁定73)。text style spans を flat run-list + 安定 `TextStyleId` 参照で持つ(絶対オフセットでなく長さ・隣接同値ラン併合が正しさの契約、裁定85/87/89) |
| 15 | **デザイン値は token 1箇所経由・raw 直書き禁止** | 裁定117(外出し+watch)・142(トンマナ共通要項)・178(徹底・browser系0件の欠陥実例) | パネルごとに値がバラつき、トンマナ変更(色・寸法)が全パネルへ伝播しない製品になる(裁定142 が防ごうとした事故そのもの) | `motolii-tokens-rs`(parse・watch・`ui_scale`)一式。`dimensions.json` 正本(Ableton実測値)。raw色/px直書きを機械的に禁じる柵テスト |
| 16 | **空間は1つの世界・1つのカメラ(2D/3Dスイッチを作らない)** | 裁定113(利用者再確認「これが Motolii の中核コンセプト」)・115(透視 preview)・116(camera 実装・byte一致実測) | AE の「2Dレイヤーはカメラを無視する」という空間分割セマンティクスが必要になり、パララックス前提(将来の3D撮影・rerun 選定理由の一つ)が崩れる | `motolii-core::camera` の投影計算1箇所実装。全層 z=0・既定カメラで旧正射影と **byte一致(max_diff=0)** の機械実証(裁定116)。`pinned` 属性によるカメラ逆行列の打ち消し実装 |
| 17 | **音声は Preview=Export のため自前 mix を持つ**(ffmpeg `amix` で混ぜない) | 裁定125(調達調査 DONE) | scrub 時の即応性(対話性)が失われ、preview で `amix` が使えない=評価経路が2本になり意見2(Preview=Export)が崩れる | owns 約950行(`mix`/`program`/`MixProducer`)を意図的自作。全候補(rodio/kira/Tracktion/GES/MLT/Ardour/WebAudio 等)を比較した上での選択(裁定125 に探索範囲を明記済み) |
| 18 | **拡張の口は trait 1本・first/third-party が同じ口** | 裁定6・70(意見9の前提となった先行裁定)、`next/GOALS.md` D6 | first-party 専用の特権型(例: `GaussianBlur` 型)が生まれた瞬間、D6「同じ口」の主張が嘘になる | `EffectInstance`/`PropertyLink` は共に `plugin_id: String` + 平坦 params のみ(型を持たない)。`owns:` が effect の種類数に比例して増えない設計。意見9(接続子は加算)の連鎖もこの1本の口の上に乗る |

## コストの4列目を特定しづらかった項目とその扱い

- **「保守をしたくない」(裁定39・43)= 自前コードは負債**: 明確なコスト指標(`owns:` 総行数を下げるべき数字として扱う、裁定43)は存在するが、これは**開発体制・実装方針の軸**であって、外部先例に置き換えても(=毎回スクラッチで書いても)*製品として利用者が触る挙動*が直接壊れるわけではない(コストと保守負債が増えるだけ)。よって表には採らず、ここに記録するに留めた。
- **裁定142(色は全部token)を除く「純関数」「vendor非依存」(`docs/concept.md:76` の絶対規律)**: 「単一writer」「正準座標」「Preview/Export同一関数」は意見3・13・16・2として個別に既に回収済み。「純関数」「vendor非依存」は具体的な自前実装コストの帰結を1文書内で特定できず(前者は決定性(意見4)への言い換え、後者は Lottie/.rrd 採用という「借りる」判断と表面的に矛盾するため意味が確定しない)、独立の意見としては採らなかった。

## § 意見が無い領域(= 借りてよい領域)

今日時点で「先例をそのまま採る」と明記されている、または実測で発明の余地が無いと確認された領域:

- **エフェクトの語彙**(発注前提・利用者実測): Lottie が効果の `ty` を持たず、Glow 1種しかなく、Lottie 書き出しが 0/10(裁定70 の前提調査)
- **色管理(YUV→RGB 変換)**: 裁定23「自前 WGSL を書かない(色事故の動機ごと上流へ移る)」— `re_renderer::SourceImageDataFormat::Yuv` に完全委任
- **blend mode の列挙**: 裁定67「AE / Photoshop / peniko / wgpu で共通の語彙で**発明の余地が無い**」— Lottie の 0..15 をそのまま採る
- **書き出しコーデック/コンテナ(decode/encode/mux)**: 裁定24「ffmpeg サイドカーを維持する」— `re_video` を含め自前コーデックを持たない
- **レイアウト計算(flex/grid)**: 裁定183「taffy 複合案(fork 非改変)」
- **文字シェイピング/ラスタライズ経路**: 裁定190「cosmic-text → swash outline → motolii-vector」、依存増分ゼロ
- **transform 適用順序の数学**: 裁定58(Lottie 仕様+velato+lottie-rs で裏取り)、裁定69・111(skew式は velato の実装をそのまま移植)
- **音声の decode/resample/device 出力**(mix 自体は意見17として自前): 裁定135「symphonia/rubato/cpal」
- **マルチウィンドウ機構**: 裁定128「iced 上流既製(pane_grid+daemon)」
- **メニュー UI 機構**: 裁定181「iced_aw の menu モジュールを vendoring 移植」

他に疑ってよい領域(未確認・今回は深掘りしていない): 素材ファイルフォーマット対応表(ffmpeg 任せ)、フォントファイル形式そのもの(cosmic-text/fontdb 任せ)。
