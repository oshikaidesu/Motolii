# 第二実コード監査の裏取りと台帳化: D1系スキーマ・評価・永続(2026-07-12)

ステータス: **裏取り済み所見+決定記録**(規律1/2/6適用済み — 監査主張を設計根拠にせず、本文書で全コード主張をfile:line現物確認した。判定語併記。**S1/S6/S16は2026-07-12ユーザー決定: 3点とも案1承認**(補正4点込み、下記決定節))

> **2026-07-23現行注記**: 下表のfile:lineと「現行」は対象SHA `9c8e274` 当時の事実であり、現在の未実装一覧ではない。全4版と現在のコードを再照合した処分は[Unit 4C-2回収](2026-07-23-historical-d1-code-audit-lineage-recovery.md)を参照する。S1〜S7、S9〜S16、S18、B④は実装済み。S8は後続VSM-B2でidentity責任ごと比較中、S17の中間写像/loss report審判は未実装である。

## 前提と手法

- 出所: 2026-07-12の第二LLM監査(第一監査=[2026-07-11-code-audit-pre-m2](2026-07-11-code-audit-pre-m2.md)の後続。前回はプラグイン境界中心、今回はD1a〜D1cマージ後のスキーマ・評価・永続が対象)
- 対象SHA: `9c8e274`(本ブランチ作業ツリー)。**採用時に最新mainで再確認すること**(第一監査の検証注記を継承)
- 裏取り方法: コード主張は全てfile:line現物を本セッションで確認した(下表「裏取り」列)。外部出典のうち**S1/S6/S16決定節の出典(OTIO・FCPXML・Lottie・Adobe・Blender)は一次・公式資料を2026-07-12に確認済み(FCP Cafeのみ二次解説として併記)**。その他(S7〜S18表中のSQLite/Krita/Godot等)は公開恒久文書のリンク保持のみで個別記述の一次確認は未了(規律3: 整合する事例に留める)
- 監査の数値例(30000/1001でpos=14.999…等)は**再計算していない**。f64乗算+floorの構造(S7)が確認できれば境界誤りの可能性自体は成立するため、数値の精査はチケット側の再現テストに委ねる
- ID: 本監査所見は**S1〜S18**。既存台帳のA1〜A8・B群([d1-spec-holes 追補](2026-07-12-d1-spec-holes-prior-art.md))と重複するものは統合先を明記

## 裏取り結果と判定(S1〜S18)

判定語は規律6(採用/縮小/延期/棄却)。「行き先」は発注書・台帳上の割当。

### P0(スキーマ修正候補 — D1a/D1b再開対象)

| ID | 所見 | 裏取り | 判定 | 行き先 |
|---|---|---|---|---|
| S1 | `Clip.start`と`TimeMap.timeline_start`が二重正本。validateは関係を検査しない | **確認**: `schema.rs:301`(start)+`schema.rs:304`(time_map)+`time_map.rs:14`(timeline_start)。validate.rsにtime_map関係検査なし | **採用**(方式は**ユーザー決定S1**、下記) | D1aフォローアップ |
| S2 | TimeMap速度・Fpsが非正準(`2/2`≠identity、`30/1`≠`60/2`)。`is_identity()`が構造体比較 | **確認**: `time_map.rs:92-100`(正値検査のみ)・`time_map.rs:113-115`(構造体比較)・`time.rs:184-190`(Fps正値のみ、Eq/Hash derive) | **採用**。既約化はM2E-16(RationalTime/BPM)と同型の不変条件をFps/speedへ拡張。`is_identity`は意味比較へ。「正準アフィン写像として保持」まで行くかはS1の方式に従属 | D1aフォローアップ |
| S3 | AssetRefが`Value::AssetRef(String)`と`AssetId(u64)`に分裂し、validateがConstを素通し(存在・対応・ダングリング・cross-doc再写像を検査できない) | **確認**: `value.rs:12`・`asset.rs:16`・`validate.rs:252`(`Const(_)`無条件Ok)。F-10の「結線はM2 D1」予約が未消化のまま検査だけ先に閉じた状態 | **採用**。永続層はAssetId参照(推奨)、評価層へは解決済み値で渡す。方式詳細はD1bフォローアップ発注書で確定 | D1bフォローアップ+D3 |
| S4 | Document→eval経路の型検査がfail-open: 型不一致lerp→前値、空トラック→`F64(0.0)`、Vec2Axes不一致→`0.0` | **確認**: `value.rs:16-29`(コメント「検証層で弾く前提」と実態の矛盾)・`track.rs:97`(空→F64(0))・`eval lib.rs:85-107`(0.0縮退)・`validate.rs:250-261`(Const/Keyframes/Data素通し) | **採用**。第一監査P-2のDocument経路版。各DocParamに期待型を与え、トラック内全キー・fallback・DataTrack出力型・**空トラック**まで検査 | D1bフォローアップ(期待型表)+D3 |
| S5 | PathOpが名前+DocParamのみで意味論未定(単位・範囲・丸め・複数輪郭・適用順・乱数・退化パス)。Lottieは各modifierを明示しており「Lottie前例」は名前だけ | **確認**: `schema.rs:316-350`。validateもparam再帰のみ(`validate.rs:263-292`)。Lottie仕様の個別記述は未検証出典 | **採用**。各variantにつき「入力型/単位/範囲/丸め/複数輪郭/適用順/互換version/ゴールデン」の表を仕様化するまでD1a完了扱いにしない | **PathOp意味論仕様書**(D1aフォローアップ、D3前提) |
| S6 | `path_ops`が全Clipに存在し、動画Asset・ラスタPluginにも付く(パス→パス演算に入力パスが無い) | **確認**: `schema.rs:307`(Clip共通フィールド)・`schema.rs:282-296`(ClipSourceに種別制約なし) | **採用**(方式は**ユーザー決定S6**、下記) | D1aフォローアップ |

### P1(仕様化・完了条件の強化)

| ID | 所見 | 裏取り | 判定 | 行き先 |
|---|---|---|---|---|
| S7 | DataTrackの添字がf64乗算+floorで正確なサンプル境界を外し得る。非補間型(AssetRef等)では1サンプル前を返す | **確認**(構造): `track.rs:161-170`。数値例は未再計算(前提と手法) | **採用**。整数添字は有理数演算で求め、補間率のみf64へ。境界ケースの再現テストを完了条件に | D1/D3チケット(DataTrack正確サンプリング) |
| S8 | `DataTrackId(pub String)`は名前変更で参照が切れる。正準ID(producer+version+output+source)が必要 | **確認**: `eval lib.rs:23` | **採用**(縮小: v1は正準ID**形式の宣言**まで。強制はDataTrack生産経路が繋がるD3/D8で) | D3/D8発注書 |
| S9 | 非有限値・値域の検査が不完全(NaN/Inf・0–1外Color・負radius・非整数copies)。validateはValueの中身を見ない | **確認**: `validate.rs:252`(素通し)。NaN注入経路(UIコマンド/内部API)は将来のD2で現実化 | **採用**。既存**B⑥(パラメータ範囲方針)へ統合**し、「非有限値は全パラメータで拒否+パラメータごとにクランプ/拒否を宣言」へ具体化 | D1bフォローアップ(B⑥統合) |
| S10 | プロジェクト入力に資源上限がない(`fs::read`全読み・再帰Group・任意extra・巨大キー列) | **確認**: `persist.rs:162-179` | **採用**。上限表(ファイルサイズ/Group深度/Track・Layer・Key数/文字列長/extra/コマンドpayload/journal総量/サンプル数)+fuzz corpusを完了条件へ | D1cフォローアップ+D1d発注書 |
| S11 | 現行abort注入は正常`Err`返しであり、FS故障(並べ替え・部分書き込み・rename未永続・ENOSPC・再クラッシュ)を模擬しない | **確認**(構造): D1c完了条件はabort注入のみ(M2仕様)。SQLiteの手法は未検証出典 | **採用**。D1d完了条件へ「擬似FS trait/fault-injection相当。`SaveAbortAfter`のみでD1d完了にしない」を明記 | D1d発注書 |
| S12 | migrationの件数一致では意味保存を証明できない(単位取り違え・回転反転・effect順逆転・参照付け替えが件数不変で可能) | 論理指摘(D1e未着手のため対象コードなし) | **採用**。D1e完了条件へ「代表時刻Param評価値・解決済み依存辺・TimeMap結果・レンダグラフ正準digest(可能なら低解像度ゴールデン)の前後比較」を追加 | D1e発注書 |
| S13 | 既知pluginの「未来版」がdowngrade errorになり、D1fの「未知pluginでも開ける」と非対称 | **確認**: `plugin lib.rs:450-455` | **採用**。D1fへ「既知だが未来版=未知と同じdegraded object(開く・無変更保持・pass-through評価・書き出し拒否・再保存で喪失なし)」の層別を明記 | D1f発注書 |
| S14 | read互換とwrite互換が分かれていない(`min_reader_version`のみ。新しい版のファイルを読めてしまえば再保存で破壊し得る) | **確認**: `persist.rs:167-179` | **採用**。ロードAPIが`OpenMode::ReadWrite / ReadOnlyNewer / Reject`相当の三状態を返す形へ。SQLiteのread/write version形式は整合する先例(未検証出典) | D1cフォローアップ+D1e発注書 |
| S15 | 「不正テール切捨て」が原本を直接truncateすると回復可能だったデータを失う。回復は原本保持+別成果物(`*.corrupt-日時`/`*.recovered-日時`)へ | 計画段階の指摘(D1d未着手)。Krita/Blenderは整合する先例(未検証出典) | **採用**。D1dへ「recovery非破壊原則: 原本・journalを上書きせず、回復結果は別ファイル+ユーザー提示」を明記 | D1d発注書 |
| S16 | PathOp・LookAt・Follow・BlendMode・Bezier solverに意味論versionがなく、アルゴリズム改善で旧プロジェクトの絵が変わる(「純関数」は同一バイナリ内の決定性のみ) | **確認**(構造): 該当型にversionフィールドなし(`schema.rs`) | **採用**(方式は**ユーザー決定S16**、下記) | D1aフォローアップ(方針宣言) |
| S17 | 「OTIO-shaped」(F-5)は変換テストなしでは自己申告。代表Documentを中間OTIO構造へ写す**loss reportテスト**をM2中に置く | 論理指摘。OTIOのschema区別(source_range/available_range/Gap/Transition/Stack)は未検証出典 | **採用**(縮小: 書き出し実装はしない。写像表+loss reportの試験1本) | ゲート台帳「M2期間中に消化する」へ追加 |
| S18 | Undo coalescingは同名Command結合では不足。結合キーは`gesture_id + command_kind + target_stable_id + property_id`(別レイヤーの連続opacity編集が同名だけで潰れる)。Godot UndoRedoのMERGE挙動が先例 | D2未着手のため対象コードなし。Godot出典は未検証 | **採用**。D2発注書のcoalescing仕様へ結合キーを明記 | D2発注書 |

### レイヤーフラグ(監査15)の扱い

hidden/solo/lockの「visible ≠ evaluable」(hiddenでもparent/mask/LookAt対象としては評価される — Lottie先例)は、既存**B④**の1行宣言では不足であることが確定。B④を「**フラグごとに描画・評価・書き出しの3軸で挙動を宣言する表**」へ書き換える(行き先: D1フォローアップ+D3発注書)。

## ユーザー決定3点【2026-07-12決定: 3点とも案1承認(補正4点込み)】

いずれもユーザーデータへ不可逆に焼かれるスキーマ形状の決定。A1〜A3と同じ手続きでユーザーへ提示し、**2026-07-12に3点とも案1で承認された**。承認時の補正(式の固定・S6説明の訂正・S16執行機構・出典厳密化)は各節へ反映済み。

**S1: クリップ時間原点の一本化方式 — 決定: 案1承認**
- **案1(採用)**: TimeMapを「クリップローカル時刻→ソース時刻」の写像にし、`timeline_start`を削除。タイムライン位置の正本は`Clip.start`のみ
- 案2(不採用): 二重保持+`clip.start == time_map.timeline_start`のvalidate不変条件(規約違反が新たな恒久バグ族になる)
- **固定式(ユーザー指定。仕様へこのまま焼く)**:
  ```
  clip_local_time = timeline_time - clip.start
  source_time     = time_map.map(clip_local_time)
  ```
  帰結: タイムライン位置の正本=`Clip.start` / キーフレーム時刻=クリップ起点のローカル時間(spec-holes §1と同一領域) / TimeMap=ローカル時間→ソース時間 / クリップ移動=`Clip.start`のみ変更 / **キーフレーム評価にはTimeMapを通さない**
- **残る宣言義務(発注書で必須)**: **尺の正本**。(a) v1は`Clip.duration`が表示区間の正本でsource使用終端をTimeMapから導出する、または (b) TimeMapの終端ノットから`duration`を導出し独立保存しない — のどちらかを宣言する。宣言しないと原点の二重正本を消して**尺の二重正本**を残す(§1b「尺は導出値」との整合確認込み)
- **先例(2026-07-12調査、一次確認済み)**: OTIOはTrack内の順序とGapから親時間上の位置を**導出**するためMotolii(明示start)と完全同型ではないが、「source領域と親timeline領域を別座標系にする」根拠になる — [OTIO Timeline Structure (v0.18.1)](https://opentimelineio.readthedocs.io/en/v0.18.1/tutorials/otio-timeline-structure.html)/[Time Ranges (v0.18.1)](https://opentimelineio.readthedocs.io/en/v0.18.1/tutorials/time-ranges.html)。FCPXMLは`offset`(親タイムライン上の位置)+`start`(ソースのイン点)+`duration`で位置と写像の領域が分離 — 一次: [Apple FCPXML Reference](https://developer.apple.com/documentation/professional-video-applications/fcpxml-reference)、解説(二次): [FCP Cafe](https://fcp.cafe/developers/fcpxml/)。二重保持(案2)の先例は**調査範囲では未発見**(不在証明ではない)

**S6: PathOpの適用可能性の型表現 — 決定: 案1承認(VectorRecipe形。root modifier stackに限定)**
- **案1(採用)**: PathOpをVector系ソース内へ移動。案2(validate拒否のみ)は不採用 — 「どのsourceがパス出力を持つか」の判定表が別正本として必要になり、構造で防げるものを検査で防ぐことになる
- **v1構造(ユーザー指定)**:
  ```
  VectorRecipe
  ├── content: StandardShape | SvgAsset | TextPath | Group
  └── modifiers: Vec<PathOp>
  ```
  固定規約: ①modifiersは**rootの全パス集合**に作用 ②**index 0から順に適用** ③Trimのparallel/sequentialは**PathOp自身のフィールド** ④raster sourceには**構造上存在しない**
- **説明の訂正(承認時補正)**: 当初の「Vector系ソース内へ移せばLottieのスコープ・適用順も手に入る」は**不成立**。LottieはShape・Group・Style・Modifierが同一の内容リストに混在し、Modifierが同一スコープ内の**先行シェイプ兄弟**に作用する構造であり、`content+modifiers`の分離形では兄弟スコープは再現されない(誤適用防止のみが得られる)。Lottie同等の入れ子スコープが必要になったら`Vec<VectorItem>`(Shape/Group/Style/Modifier混在)の**別設計**が必要 — 本フォローアップよりかなり大きく、v1のroot modifier stackと混同しない(v2判断)
- **先例(2026-07-12調査、一次確認済み)**: Lottie/AEともmodifierはシェイプレイヤーの内容リスト内にのみ存在し、フッテージレイヤーには構造上付けられない。複数modifierの合成順・Trimのparallel/sequentialも同仕様が定義 — [Lottie Shapes仕様 (1.0)](https://lottie.github.io/lottie-spec/1.0/specs/shapes/)。層レベルvalidate(旧案2)の先例は**調査範囲では未発見**

**S16: コア演算の意味論version — 決定: 案1承認(意味論ゴールデンで機械執行)**
- **案1(採用)**: per-opの`algorithm_version`フィールドは**焼かない**。方針: **データ形状変更→migration / 画素意味の変更→新variant / 既存variantの意味→固定 / legacy variantは新規UIから隠しても読込・評価は維持**。案2(フィールド予約)は不採用
- **執行機構(承認時補正: 1文宣言だけでは執行できない)**: 最低限、永続演算ごとに**保護された意味論ゴールデン**を置く — PathOp各variant / BlendMode / LookAt・Follow / Bezier solver / WiggleのPRNG・seed / Transform合成。加えてCI規則: **画素意味を変えるPRは既存ゴールデンを更新できない。新variant+新ゴールデンを追加する**。PathOp分のゴールデンはS5の意味論仕様とセットで整備(D1発注書)、評価系演算(BlendMode/LookAt/Follow/Bezier/Transform合成)分はD3完了条件へ
- **先例(2026-07-12調査、一次確認済み)**: AEはブラーのアルゴリズム変更時に旧エフェクトを「Gaussian Blur (Legacy)」等へ改名してObsoleteカテゴリへ移し、新アルゴリズムは別エフェクトとして追加した。Adobe公式はLegacyが現行と**異なる結果を出す**こと、旧エフェクト(例: 旧Lightning)を旧プロジェクトで**継続利用できる**ことを明記 — [Adobe公式: Obsolete effects](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/obsolete-effects.html)。ただし恒久保証までは出典から言えないため「**長期互換の実例**」に留める。Blenderの`do_versions`はロード時migration方式の先例だが、公式に「as best as possible」変換と明言しており画素一致は保証しない — [Blender Developer Handbook: Blend File Compatibility](https://developer.blender.org/docs/handbook/guidelines/compatibility_handling_for_blend_files/)。読み: **データモデル変更はmigration(Blender型)、画素に効く変更は新variant(AE型)**という分業が先例と整合(規律3: 整合する事例)

## 審判(テスト)への含意

`cargo test -p motolii-core -p motolii-eval -p motolii-doc`は140件全緑だが、S1〜S4・S9の挙動は検出されないどころか一部は**現行挙動としてテスト済み**(例: `track.rs`の空→F64(0)テスト、`value.rs`のlerpフォールバックテスト)。つまり該当箇所の審判は「現実装の固定」であり「仕様違反の拒否」ではない。各フォローアップの完了条件に「**既存の縮退挙動テストを、仕様違反を拒否するテストへ書き換える**」ことを含める(削除ではなく反転 — 挙動変更の意図をテスト差分で可視化する)。

## 修正順(受理)

監査提案の順序を受理する: ①TimeMap正準化+原点一本化(S1/S2) → ②DocParam期待型+AssetId型付け(S3/S4/S9) → ③PathOp意味論+適用可能性(S5/S6) → ④read/write互換(S13/S14) → ⑤入力上限・fuzz・fault injection(S10/S11) → ⑥recovery非破壊(S15) → ⑦DataTrack(S7/S8) → ⑧migration意味保存(S12) → ⑨OTIO loss report(S17)。S18(D2)とB④改訂は各発注書起草時。

①〜③のユーザー決定(S1/S6/S16)は2026-07-12に案1で完了。発注書化はM2仕様のD1g/D1h/D1i-1〜D1i-4行として実施済み(→[M2-document-model.md](../specs/M2-document-model.md)。2026-07-12承認時補正: source終端式=`time_map.map(clip.duration)`、依存の直列明記、DataTrack検査の二段化、D1iの子チケット分割)。

## 既存台帳との重複整理

- S4 = 第一監査**P-2**のDocument経路版(P-2自体はM2E-8で消化済み=プラグイン側のみ)
- S9 → **B⑥**へ統合(本文書が上書き)。B④ → 上記のとおり書き換え
- S1 = spec-holes **§1b決定**の原点版(§7bのOverrunModeと同じ「決定済み・スキーマ未反映」族に加える)
- S17 = 凍結ゲート**F-5**の審判化。ゲート台帳の並走チケット表へ追加済み
- 第一監査の「その他所見はf020ec8基準」注記と同様、本監査所見も**チケット採用時に最新mainで再確認**
