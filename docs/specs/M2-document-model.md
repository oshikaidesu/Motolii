# M2: ドキュメントモデルとタイムライン

ステータス: **段階発注可**(コア締結宣言は**撤回** — [記録](../reviews/2026-07-14-m2-core-closure.md)。P1修復 #153/#154 完了。D7(#145/#149)は維持。**次発注可: D5(#144)**。PP-GateはM3前追跡のまま。再宣言は追補レビュー後)。
着手条件: Wave1〜4の実装とP1修復(#153/#154)は main 到達済み。**D5(#144)発注可**。Transport方針【採択】正本=[先例調査](../reviews/2026-07-14-d5-transport-prior-art.md)。

## 目的(退治する落とし穴)

C-1(Undo後付け)、C-2(スキーマ進化)、B-1(音声・時間表現)、F-1(座標系)、F-2(所有権)、F-4(時間写像)。

## 方針

- プロジェクト状態は単一のserde可能な純データ構造(ドキュメントモデル)。エンジン(motolii-render)はこれを読むだけ
- ドキュメントモデルはAsset実体とプロジェクトBPM(手動入力。ビート検出はやらない)を持つ。**Assetは一般アセット(opaqueペイロード+type文字列+内容ハッシュ)として定義し、動画・SVGはその特殊ケース**(F-10、[plugin-resources.md](../plugin-resources.md)§3 D2/D3。将来のImporterプラグイン=点群等の口。`ValueType::AssetRef`の予約と対)。**F-10は凍結ゲートで確定済み**(PipelineCache実証+AssetRef予約)。以後のD1系並列はタスク表の依存レーンに従う(「F-10未確定だからD1禁止」は失効)
- **音声のMV既定導線はプロジェクト直下の`Soundtrack`(楽曲1本+開始オフセット+マスターゲイン)**。これは既定ワークフローであり恒久制約ではない。Clip内蔵audio componentとmixer一般化は[音声一般化設計](../reviews/2026-07-14-audio-generalization-design.md)の独立レーン(AG-1〜)で追加する。M2コアの実装範囲ではSoundtrackのみを扱い、動画クリップ内蔵音声はミュートのまま
- 全編集操作はコマンド(適用/逆適用可能な差分)として実装 → Undo/Redoはコマンド履歴で自動的に得られる
- ファイル形式はバージョンフィールド付きJSON + マイグレーション関数の枠組み
- **空間値は正準座標系で持つ(F-1)**: 単位なし・原点=コンポ中央・Y-up・高さ基準正規化(高さ=1.0、幅=アスペクト比)。位置・アンカー・サイズ・エフェクトの空間パラメータにpx値をスキーマに入れない。グループは変形コンテナ(変形+合成の両方)であると同時に、**クリップと同じ項目エンベロープ(順序付きエフェクトスタック・クリッピングマスク・ブレンド/不透明度)を持つ**(concept 2026-07-10「プリコンポは作らない」の帰結。グループのエフェクトは子を合成した1枚に適用=AM式、per-child適用はしない)。レイヤーの親参照(ペアレンティング)フィールドをスキーマに予約(v1のUIはグループ変形のみでも、スキーマは初日から)
- **シェイプ間リンク(レイヤー参照付きParamSource)をスキーマに含める(concept 2026-07-10)**: LookAt/Follow/ParentRef 等は**別レイヤーの変形を読む型付き参照**。AEエクスプレッションの代替。`LayerId`参照と依存グラフ(無効化伝播の入力)を D1/D3 で予約。評価は F-3 の順序で参照先を先に評価
- **シェイプ系レイヤーは順序付きパス演算子スタックを持つ(concept 2026-07-10、F-13)**: パンク・膨張/ジグザグ/パスのオフセット/角丸/トリムパス/ツイスト/パスのウィグル(+リピーター=F-7)を、標準シェイプ・SVG・テキスト由来パスに共通適用する`Vec<PathOp>`相当の席をD1で予約。全演算子は`(パス, パラメータ, t)→パス`の純関数で、パラメータは通常のParamSource(キーフレーム/リンク)駆動。シリアライズはLottie形式(`pb`/`zz`/`op`/`rd`/`tm`/`tw`/`rp`)を前例にする([references.md](../references.md))。v1はファーストパーティの閉集合(プラグイン契約には出さない=`PathOp`種別化はv2判断、解凍手続き対象)
- **クリップは時間写像(TimeMap)を持つ(F-4)**: `clip_local_time → source_time`の単調写像(D1g)。v1実装は恒等+定数速度(オフセット+speed)のみだが、**motolii-renderのソース時刻解決は必ずTimeMapを通す**。速度ランプ・逆再生はスキーマ互換のまま将来拡張。キーフレームはTimeMapを通さない
- **所有権は単一writer+不変スナップショット(F-2)**: ドキュメントを書き換えるのはコマンド適用の編集スレッドただ1箇所。レンダ・書き出し・解析・プロキシ生成は`Arc<Document>`スナップショットを受け取る読み手。バックグラウンド成果はメッセージでwriterへ返し、writerがコマンドとして適用する
- **スキーマの素性はOTIO互換寄りに保つ(F-5)**: トラック/クリップ/ソース区間を有理数時刻で表現し、「OTIOに写像できない構造を発明しない」。OTIO書き出し自体はv2候補

## スキーマ境界の宣言(M2E-11 / 監査SC-1)

D1着手前に固定する。エージェントが「もっともらしい継承」で恒久負債を作らないための境界宣言。

1. **D1は`ProjectV1`を継承も移行もしない**: `ProjectV1`(`motolii-cli`)はM1 CLI専用の使い捨て。Documentの`version`採番は独立。`export-project`はD3完了時にDocument読み込みへ置換する(ProjectV1増築禁止)
2. **Document ≠ ExportJob**: Documentはレシピのみ。出力パス・書き出し範囲・エンコード設定(qp等)は別構造`ExportJob`(仮称)。Asset参照は初日から多重キー(実装ガード10)
3. **クリップのin/out/durationは`RationalTime`**: フレーム添字(`start_frame`/`frame_count`形式)をスキーマに入れない。`ProjectV1`のフレーム添字は入力素材fps基準の暫定であり、Documentへ持ち込まない
4. **bpmは有理数**: `f64` bpm禁止。有理数で持ち、拍時刻(`60/bpm`秒)が`RationalTime`に畳めることをD1完了条件に含める。**小数bpm入力を許容する(ユーザー決定 2026-07-12: DAW慣行。例 120.35 → 12035/100を既約化)**。当初併記の「ミリbpm整数」案は棄却 — 小数桁数を3桁に固定する理由がなく、有理数が上位互換。`bpm > 0`をvalidate(M2E-16のFpsと同型)
5. **`ExportOverlayRequest`形式のジョブミラーを温存しない**: D3はDocument→render層リクエスト(`BackgroundTextureRequest`系)を直結する。ProjectV1→PreparedProject→ExportOverlayRequestの4層コピーをD3で廃止する

## D1-prelude(M2E-12 / 監査SC-2)

入場条件として先行実装済みの骨格。**トラック/クリップ/Asset/BPM/キーフレーム等のスキーマ本体は含めない**(本体はゲート全緑後の**D1a**)。

| 予約 | 役割 |
|---|---|
| `Document.version` + `min_reader_version` | 版番号と前方互換の拒否閾値(実装ガード7) |
| `#[serde(flatten)] Document.extra` | 未知キー保持→再保存で書き戻し(unknown-keys roundtrip) |
| `DocumentWriter` + `edit`/`apply`/`snapshot` | 単一writer・`Arc`スナップショット(F-2)。`edit`はD2で`apply(Command)`に置換、呼び出し追加禁止 |
| `DocumentWriter.revision: u64` | 編集世代(決定性テスト・無効化の席)。`edit`/`apply`で加算 |

注: prelude時点のプロジェクト直下`time_map`プレースホルダは**D1aで削除し、クリップの`TimeMap`へ移した**(F-4の本席)。

## ネスト未知フィールドの方針(D1a決定 / ガード7の緊張)

`#[serde(flatten)] extra` を持つのは **Documentトップ・EffectInstance・ClipSource::Plugin** のみ(プラグイン拡張点とトップレベル前方互換)。`Composition` / `Track` / `Clip` / `ItemEnvelope` / `Transform2D` / `Group` 等は **未知フィールドを黙って捨てる**(serde既定)。

これは「旧リーダーが新版を開ける」と「開いて再保存すると新フィールドが消える」の緊張を残す。**解消策はネスト構造へ一律`extra`を足すことではなく、次の規律**:

1. **ネストに永続フィールドを追加する変更は、必ず`min_reader_version`を上げる**(旧リーダーはファイルを拒否する。開いて壊す経路を作らない)
2. プラグイン作者が足しうる未知キーだけ Effect/Plugin の`extra`で保持する(F-9)。コア構造の進化は版番号で守る
3. `CompCamera`等を将来Compositionへ入れる判断も同じ — CQ-5完了後にフィールド追加するなら`min_reader_version`同時上げ

D1e(マイグレーション)はこの規律の運用口。D1a時点でネスト全面`extra`化は採らない(恒久スキーマの表面積を増やしすぎる)。

## 色契約の宣言(M2E-13 / 監査CQ-1・F-4)

キーフレーム済みカラー量産後の解釈変更はマイグレーション不能な破壊になるため、D1着手前に3層を分離して固定する。**配線(`precise_color`)はM2E-18で分岐点まで到達済み**(v1実装は恒等=sRGBブレンド)。

1. **永続スキーマ上の`Color`の意味**: **straight-alpha・非線形sRGB・各成分0–1**。`Value::Color` / Documentの色パラメータはこの意味。現実装(無変換で`ColorSpace::Srgb`ターゲットへ書く経路)と一致させる
2. **レンダ中間表現との区別**: 保存値とレンダ中間(現状premultiplied、将来はlinear)を混同しない。スキーマ値→レンダ入力の変換は**レンダ側の責務**(絶対規律「色変換の一元化」)。UI/JSONのstraightをレンダ直前またはComposite境界でpremulへ落とす既存方針([M1仕様](M1-vertical-slice.md)注)を維持
3. **合成空間はこの決定と別問題**: 「保存値がsRGB」から「sRGBでブレンドする」は帰結しない。v1の合成は**sRGB空間ブレンド**(暫定、ゴールデン焼き込み済み=監査C-1)。linear premultiplied合成への移行は`precise_color`配線(M2E-18)の先の将来判断。M2期間中にこの暫定出力へ依存するゴールデンを増やさない

## LayerId予約(M2E-15 / 監査SC-3・F-7)

ドキュメント内レイヤーの恒久ID。**所有はmotolii-doc**(core/eval/プラグイン契約には出さない)。LookAt/Follow/ParentRefの参照解決はD3(doc→グラフ変換)で行い、変換時に解決済みの具体参照へ落とす。将来`Value::LayerRef`等が要る場合は解凍手続きでcore移動を判断する。

1. **表示名はIDと別フィールド**: `LayerId`は不透明な採番値。ユーザー向け名前は`LayerIdTable`の値側(または将来のレイヤー構造の`name`欄)に置く。表示名の変更で参照が切れない
2. **ID再利用禁止**: 削除後も採番カウンタは戻さない。削除→再作成は新しいIDになる(ジャーナル/リンクの幽霊参照を構造的に避ける)
3. **重複挿入拒否**: 同一`LayerId`の二重登録は型付きエラー。ロード復元も同じ不変条件

## 時刻型のserde不変条件(M2E-16 / 監査TM-2・TM-5)

ジャーナル/JSONから不正有理数を注入できないようにする。`RationalTime`/`Fps`/`TimeMap`はDeserialize時に検証または正規化する。

1. **`RationalTime`**: (a)`den==0`拒否 (b)負の分母は符号を分子へ (c)既約化 (d)`0/x`→`0/1` (e)正規化後のi64溢れは`RationalTimeError::Overflow`(panicしない)。公開経路は`try_new`/`try_from_frame`/`try_to_frame_floor`/`try_to_frame_round`/`try_add`/`try_sub`/`try_mul`/`try_neg`のみ(中間演算はchecked)。**時刻→フレーム添字は `try_to_frame_floor` / `try_to_frame_round` のみ**(TM-4/#48)。ffmpeg `-ss` 秒文字列は `motolii_core::format_ffmpeg_seek_before_frame` のみ
2. **`Fps`**: 正の`num`/`den`のみ。フィールドは非公開(正値を型の不変条件として固定)。構築は`try_new`とDeserializeのみ
3. **`TimeMap.speed`**: M2では`speed_num > 0`かつ`speed_den > 0`のみ(ゼロ・負の速度は明示拒否。逆再生は将来拡張)。構築時・Deserialize時に**既約化**し、速度フィールドは**非公開**(正準コンストラクタ`try_new`限定 — Eq/Hashが意味同値と一致)。`try_map`は未検証入力でもpanicせず`TimeMapError`/`RationalTimeError`を返す。**`Fps`も既約化**。永続JSONキーは`overrun_mode`(短縮`overrun`は拒否)

## duration/区間規約(M2E-17 / 監査TM-3)

時刻区間の開閉を1流儀に固定する。M4区間キャッシュと音声終端の前提。

1. **`duration`は総尺**: 最終フレームのPTSではない。`MediaInfo.duration`も総尺(fpsグリッドスナップ済み)
2. **区間は半開`[start, start+duration)`**: 終端ちょうど(`t == start+duration`)は範囲外。等間隔サンプル添字は`0..n`で、**`n = ceil(duration × rate)`**(有理数で厳密。`i/rate < duration` を満たす非負整数iの個数)。`duration`がrateグリッドに整列しているときは整数`n = duration × rate`に一致する。**`MediaInfo.duration`はfpsグリッドへスナップ済み**のため、`export_frame_count`の`floor(duration × fps)`は整列前提で正しく、非整列の一般式(ceil)はParamDriver側に適用する
3. **互換性**: この規約により`ProjectV1`の書き出しフレーム数が1減る(例: 90フレーム素材で旧91→90)。`ProjectV1`は使い捨て(M2E-11①)でありユーザーデータの互換対象外。旧`from_frame(89)+1`系テストの更新は規約変更に伴う正当な期待値更新(M2E-2の「テスト更新」手続き)

## 補間・変形の意味論宣言(2026-07-12ユーザー決定 / D1a追認)

D1aが焼いた`Transform2D`/`Value::lerp`の意味論を仕様契約として固定する。キーフレーム量産後の解釈変更はM2E-13(色契約)と同格のマイグレーション不能であるため、D3がゴールデンを焼く前に宣言する。3点とも**宣言時点の実装と一致**(コード変更なしの文書化)。経緯は[D1スキーマ未決点メモ§7](../reviews/2026-07-12-d1-spec-holes-prior-art.md)。

1. **色キーフレームの補間空間**: `Value::Color`の補間は**保存空間(非線形sRGB・straight-alpha)における成分ごと線形**(アルファも同様に線形)。linear/Oklab等の知覚補間を導入する場合は将来版の**追加的**オプトイン(補間空間フィールドの追加)とし、この既定の意味は変えない
2. **回転の表現**: `Transform2D.rotation`は**単一スカラー・単位はラジアン**。多回転は値そのもので表す(0→4π=2回転)。AE式「回転数+度」の複合表現は採らない。補間はスカラー線形で**最短経路ラップをしない**(350°→10°は逆回りに340°戻る — 多回転をそのまま表現できることの裏面)。**度はUI側の表示変換のみ**(M3。例:「720°(2回転)」の括弧表示)。スキーマ・ジャーナル・プラグイン境界に度の値を流さない — UIが度で保存する実装はマイグレーション不能の破壊であり禁止
3. **変形の適用順**: 子ローカル→親空間の写像は **`M = T(position) · R(rotation) · S(scale) · T(−anchor)`**(アンカーを原点へ→スケール→回転→位置。F-3の文言をスキーマ契約へ昇格)。親参照(`Transform2D.parent`)は親の`M`を左から合成する。**継承は変形のみ**(不透明度・エフェクト・ブレンドは継承しない)。D3の`compose_transform`はこの式のゴールデンで固定する

## D2/D3着手前決定パック(#103 / **【決定】** 2026-07-13)

正本: [決定パック採択](../reviews/2026-07-13-decision-pack-adoption.md)。発明ではなく AE/Lottie・OTIO・DAW・Qt の採択。Composerは実装のみ(案の再発明禁止)。

| ID | 【決定】 | 実装割当 |
|---|---|---|
| A4 | 同一Track内クリップ重なりは**validate禁止**。重なりは別Track、または将来の明示Transition | D1bフォロー / D3 |
| A6 | Tempo/Meter mapを音源開始と**独立**に持つ。`beat_origin=0`、先頭拍子4/4。`Soundtrack.start_offset`と分離 | D1フォロー(小さな席)+M3スナップ |
| A8 | Effect/Keyframe等も不変・非再利用の document-local `u64`。並べ替え維持、複製時新規採番+サブツリー再写像 | **D2必須** |
| B① | 同一プロパティ・同一時刻Keyframeは1個。追加は更新・置換 | D1hフォロー / validate |
| B④ | `visible=false`=自身描画除外・依存先としては評価可。`solo`=描画フィルタ。`lock`=編集禁止のみ(評価・描画無影響)。3軸表は採択文書 | D3 |
| B⑤ | 未知BlendMode=**Deserializeエラー**。閉集合。Normal代替/`Unknown(String)`禁止。追加時は`min_reader_version`上げ。**F-9対象外**と明文化 | 現状serdeと一致。方針固定 |
| B⑦ | `Composition.fps`=編集表示・スナップ・標準出力fps。内部は`RationalTime`。fps上書きは`ExportJob`のみ | D3 / 書き出し |
| B⑧ | bool/enum/AssetRef等は**Holdのみ**。線形・Bezier禁止 | eval / validate |
| ⑨ | atomic command=プロパティ単位。1 gesture=1 macro。同一対象・同一プロパティのドラッグはmerge。選択/hover/IMEはUI状態 | **D2必須** |

### 残小項目【決定】(同日)

| 項目 | 【決定】 |
|---|---|
| Undo深さ | liveと再起動後で別limit。既定 **0=unlimited**(Qt)。100等を仕様真理にしない |
| ExportJob | Document外。snapshot/ref・出力先・範囲・fps override・解像度・codec/container・音声mux |
| Group時間 | Groupにretimeを持たせない。必要なら明示的CompositionClip/precompを追加(Group肥大化禁止) |
| audio | 現行`Soundtrack.start_offset`+master gainで足りる |

## PathOp意味論表(D1i-2 / **【決定】** 2026-07-13)

**ユーザー承認(2026-07-13)**: Lottie/AE準拠で採択。発明ではなく既存規約の固定。[決定パック採択](../reviews/2026-07-13-decision-pack-adoption.md)。比較材料(未採用): [pathop-ae-cavalry-comparison](../reviews/2026-07-12-pathop-ae-cavalry-comparison.md)。

予防GR-PV-1: **意味を先に焼き、実装・ゴールデンはその写し**。画素意味の変更は新variant(S16)。単位はMotolii正準空間(Lottieの%・度をそのままスキーマに入れない)。**パス意味の正本はLottie**。Cavalryの厚い角(Wave mode・両面Offset・描画Trim・Noiseスープ等)はv1に焼かない — 追加的取り込みかv2。Lattice/Pinch/PathfinderはPathOp閉集合外。

出典(一次):
- Lottie Animation Community [Shapes 1.0 / 1.0.1](https://lottie.github.io/lottie-spec/1.0.1/specs/shapes/)
- [lottie-docs Shapes](https://lottiefiles.github.io/lottie-docs/shapes/)(`pb`/`zz`/`op`/`rd`/`tm`/`tw`/`rp`)
- AE: [creating shapes and masks](https://helpx.adobe.com/after-effects/using/creating-shapes-masks.html)系譜
- Offset実装参照: [Clipper2](https://github.com/AngusJohnson/Clipper2) offset
- Wiggle乱数: [PCG](https://www.pcg-random.org/paper.html)(AE/Lottieに相互運用可能な乱数アルゴリズムは無い → 再現性のための実装定数)

### 共通契約(全variant)

| 項目 | 【決定】 |
|---|---|
| 型 | `(paths, params@t) → paths` の純関数。隠れた可変状態禁止。Wiggleのみ seed 付き決定論ノイズ |
| 適用順 | `modifiers[i]` は **index 0から順**(D1i-1)。Motoliiはroot stackなので順方向が正 |
| 複数輪郭 | 各輪郭に独立適用。Trim `sequential`のみ連結長で扱う |
| 座標 | 正準空間(原点中央・Y-up・高さ=1)。距離・振幅・半径は正準長さ |
| 非有限 | NaN/Infはvalidate拒否 |
| 退化 | 空入力→空出力。頂点1以下のパスは恒等。自己交差は許容し**勝手に修復しない** |
| 互換 | `algorithm_version`は焼かない(S16)。意味変更は新`op` tag / 新variant |
| ゴールデン | variantごと意味論ゴールデン(D1i-2)。更新禁止はD1i-4 |
| v1閉集合外 | Lattice / Pinch / Pathfinder / Wave派生 / Cavalry Noiseスープ / Stroke描画Trim / Travel別op |

### variant表【決定】

| `op` | Lottie `ty` | パラメータ(型・単位・範囲) | 意味 |
|---|---|---|---|
| `pucker_bloat` | `pb` | `amount: F64` ∈ **[-1, 1]**。0=恒等。+1=頂点が重心へ(Lottie +100%)。-1=重心から距離2倍(Lottie −100%)。接線は頂点と逆向きに補間 | AE Pucker & Bloat。Lottie百分率意味を正規化 |
| `zig_zag` | `zz` | `amount: F64` ≥ 0(正準・峰谷距離)。`ridges: F64` ≥ 0(セグメントあたりの峰数)。`point_type`: `corner`\|`smooth` | AE Zig Zag。**Wave派生は入れない** |
| `offset` | `op` | `distance: F64`(正=外、負=内)。`line_join` + `miter_limit` | AE Offset Paths。**v1は閉路限定**。開路は型付き unsupported error(Clipper2 offset) |
| `round_corners` | `rd` | `radius: F64` ≥ 0 | 通常filletのみ。Chamfer/点別半径は焼かない |
| `trim` | `tm` | `start`/`end`: F64 ∈ **[0, 1]**。`offset`: F64(**周回**)。`mode`: `parallel`\|`sequential` | **幾何**modifier(描画Trimではない)。Travelは別口にしない |
| `twist` | `tw` | `angle: F64`(**ラジアン**)。**`center: Vec2`必須** | AE Twist。中心は永続必須フィールド |
| `wiggle` | (AE。Lottieコア外) | `amp` / `freq` / `seed: u64`。アルゴリズム名: **`pcg32_value_noise`**(PCG32-based value noise) | 再現性のための実装定数として固定。相互運用乱数は存在しない |
| `repeater` | `rp` | 整数`copies`、fractional `offset`、完全`transform: Transform2D`(default恒等=追加的)、composite順、開始・終了opacity | AE/Lottie Repeater。Duplicator全体はPathOpに畳まない(F-7) |

### D1i-2実装への拘束(発効中)

1. 本表と食い違う「便利なデフォルト」は禁止(H-3)
2. validateは拒否項目を型付きエラーにする
3. 意味論ゴールデンは境界を最低限カバーする
4. スキーマ追加は追加的: `zig_zag.point_type` / `offset.line_join`+`miter_limit` / `twist.center` / `repeater.transform`+opacity / Wiggleのアルゴリズム名は仕様契約(フィールド増はseed型の明確化)
5. 現行コードに無い席はD1i-2で追加。意味をコードへ先焼きしない

## 音声トランスポート設計(音ズレ・途切れの構造的排除)

コンセプト決定(2026-07-06/07)を受けた根本設計。「音がズレる/途切れる/重い時に映像だけ遅れる」を後付けで直すのではなく、**音声クロック常時主**で不可能にする([2026-07-14採択](../reviews/2026-07-14-d5-transport-prior-art.md))。

1. **再生ヘッド(Transport)の所有者は常にちょうど1つ**。音声も映像も追従者(follower)であり、勝手に自分の時計で進むコンポーネントを作らない。ズレとは「時計が2つある」ことの症状であり、時計を1つにすれば定義上発生しない
2. **通常再生モード**: 音声デバイスクロックが主。再生位置の正本 = **デバイスへ供給済みのサンプル数**(cpalコールバックがデバイスへ書き込んだ累計。壁時計は使わない)。映像は現在位置に最も近いフレームを表示し、間に合わなければフレームドロップ(音は絶対に途切れない・ズレない)
3. **低速時(レンダが実時間に届かない時)**: クロック所有者は**交代しない**(音声クロック常時主)。映像はフレームドロップで追従し(常に最新の現在時刻のみレンダ)、実時間回復は**適応解像度降格(DRS)**(Draft 1/2→1/4、[performance-model.md](../performance-model.md)既定段階。二重閾値ヒステリシス+連続超過パニック降格+CPUバウンド判定)で行う。なお不足なら音声正速のままドロップ継続(Premiere/FCP/Resolveと同型 — 「60秒は60秒」)。**自動バリスピード(音声をピッチごと低速化)は持たない** — AEの悪評挙動と同型であり、mpv/ffplayの±0.125〜1%ジャダー除去とも別物([2026-07-14先例調査](../reviews/2026-07-14-d5-transport-prior-art.md)で**【採択】**)。明示スクラブ/シャトルのバリスピードは別機能。**DRS計測の正本はGPU timestamp query**だが、wgpuでは全アダプタ必須ではない(`motolii_gpu::required_features`はoptional併設方針 — `crates/motolii-gpu/src/ctx.rs`)。**非対応時の縮退**: 自動DRSを無効化し、フレームドロップのみ継続(手動Draft段階固定は運用パラメータ。スキーマに焼かない)
4. **音声コールバックは絶対にブロックしない**: コールバックはリングバッファから読むだけ。デコード・(D4-FUの)固定比リサンプルは専用プロデューサスレッド。アンダーラン時は無音を出力しカウンタに記録(クロックは進めない — 無音充填は供給済みサンプルに含めない)
5. **楽曲はインポート時にPCM全展開してRAM保持**: M2のSoundtrack経路では楽曲1本が音声データの全て(5分ステレオ48kHz f32 ≈ 110MBで許容範囲)。ミキシングが存在しないため「再生位置→サンプル」は単一バッファの添字計算に還元され、シーク・スクラブ・逆再生が同一コードパスになる(明示バリスピードも同じ読み出し口へ接続)。Clip audio / 複数source mixは[音声一般化設計 AG-2](../reviews/2026-07-14-audio-generalization-design.md)で一般化する。**ビット同一検査の境界**は変換前PCMキャッシュ上に限定する(デバイスレート変換後の出力ビット同一は要求しない — D4-FU)
6. **レイテンシ補償**: 表示するフレームは「いま聞こえている音」に対応させる。補償で引くのは **cpalのデバイス待ち分のみ**(`OutputCallbackInfo::timestamp()`の`playback - callback`相当)。時計の起点が「デバイスへ供給済み」なので、**リング充填量(未来の未供給音声)をさらに引いてはならない**(二重計上)。リング量を時計に使う構成は採らない。**固定比リサンプラのアルゴリズム遅延はD4-FUがproducer側で吸収**し、Transport境界へ持ち出さない(本項の引き算対象にしない)

## タスク分割(1PR粒度。旧粗案D1をD1a〜D1fへ再分割)

旧「D1」はスキーマ本体+永続化+ジャーナル+マイグレーションが1行に詰まっていたため、仕様ルール1(1タスク=1PR)に合わせて分割する。**D1完了= D1a〜D1i全緑**(D1g〜D1i-4は2026-07-12第二監査フォローアップ)。D8の依存はスキーマが存在する**D1a**。D2/D3の依存は第二監査によりスキーマが動くため**D1i系通過後**(各行の依存欄参照)。

| ID | 内容 | 依存 | 完了条件(概要) |
|---|---|---|---|
| D1a | **完了**: **スキーマ本体+serde**(I/Oなし): Composition(有理数アスペクト・duration・fps。**`CompCamera`非包含** — #55/監査C-7)/既約有理数BPM/Soundtrack(楽曲1本・gain∈[0,1])/Asset一般(**D1aはパス+type+hashのメタのみ**。opaqueペイロード本体はImporter/GpuAssetCache側 — ファイル実体は多重キーpathが指す。ガード10)/`AssetId`・`TrackId`台帳(再利用禁止・LayerIdと同型)/Track・TrackItem(Clip\|Group再帰)/項目エンベロープ/クリップTimeMap/DocParam(Const・Keyframes・Data・Vec2Axes・LookAt・Follow。LayerIdはdoc内のみ。serdeはsnake_case — ProjectV1のParamSourceとは別名空間)/PathOp閉集合/Effect・Pluginソースに`extra` flatten予約(F-9の席。警告はD1f)。prelude継承。**ProjectV1非継承**。プロジェクト直下仮`time_map`はクリップへ | 凍結ゲート, **[M2入場条件](../reviews/2026-07-11-M2-entry-gate.md) 全緑** | JSON roundtripでグループネスト・エフェクト付きグループ・TimeMap・親参照・PathOp・Asset多重キーが保存される。**拍時刻がRationalTimeに畳める**。Composition型にcamera欄が無い。未知トップレベルキーは`extra`で保持。Effect/Pluginの未知フィールドも`extra`で保持 |
| D1b | **完了**: 保存前不変条件検証(ガード1): 参照整合・ID一意・区間正当性。失敗時は既存スナップショットを壊さない口 | D1a | 壊れたDocumentが`validate`で型付きエラー。正常系は通る |
| D1c | **完了**: アトミック保存+読込(ガード2)+`min_reader_version`超過拒否(ガード7のI/O側) | D1a | temp→fsync→rename→dir fsync。各段abort注入で旧ファイルまたはジャーナルから復元可(ジャーナル本体はD1d) |
| D1c-FU | **完了(#101、S10/S14)**: 読込時の`ResourceLimits`と`OpenMode`をI/O境界へ追加する。上限はファイル・コンテナ要素・文字列/`extra`・コマンドpayload・ジャーナル・音声サンプルを覆い、呼出側から注入可能にする。`OpenMode`は`ReadWrite` / `ReadOnlyNewer` / `Reject`を型で分離し、新版を黙って書換可能にしない。production既定値は運用調整値であり永続JSON・migration・plugin契約に焼かない | D1c | 小さい上限を注入した境界テスト、超過時の観測値/上限を持つ型付きエラー、各`OpenMode`の読込/保存可否テストが緑 |
| D1d | **完了**(#105): ジャーナル(ガード3/4/6): レコードchecksum+世代salt・不正テール切捨て・UUID相互参照・リプレイ失敗フォールバック・ピン留め世代。#101のジャーナル総量/レコードpayload上限を使い、上限を別定義しない。実FS故障注入(S11)と非破壊recovery(S15) | D1c-FU(#101) | 壊れ方カタログの単体/注入テスト緑。破損テールは論理的に無視し、原本を切詰めない。プロセスロック/stale lock/read-only fallbackは未決のため本タスク外 |
| D1e | **完了**(#104): マイグレーション枠+量的不変条件+旧版コーパス(ガード8)。前方互換の版上げ経路。**第二監査分の個別migration(旧`timeline_start`形式・旧`path_ops`形式→現行schema)を含む**。意味保存比較(Param評価・依存辺・TimeMap)。新版判定は#101の`OpenMode`(ReadOnlyNewer/Rejectでmigration拒否)、入力境界は同じ`ResourceLimits`(迂回経路なし)。loadは旧形式拒否を維持し、変換は明示`migrate_*`のみ | **D1i-1かつD1c-FU(#101)** | in-place禁止・バックアップ(失敗時原本非破壊)・クリップ/トラック/キー数一致・goldenコーパス回帰(`timeline_start`/`path_ops`世代)。意味保存自動テスト。OpenMode拒否テスト |
| D1f | **完了**: 未知プラグインID: 開く=警告+パススルー・再保存で未知部分喪失なし(F-9、ガード9の「開く」側)。**第二監査S13採用**: 既知pluginの未来版(`effect_version`が現行`NodeDesc.version`超過)もdowngrade errorにせず未知idと同じdegraded契約(開く・無変更保持・pass-through評価)に揃える。plugin_idが既知だが構造上の種別(Filter/LayerSource/...)が違う場合はdegradeで救わず型付きエラー(`DocumentError::PluginKindMismatch`)。書き出し厳格化はD6接続(本タスクでは発明しない) | D1a | 未知`plugin_id`を含むJSONがロード成功+警告(`Document::plugin_open_warnings`)+roundtrip保持。既知pluginの未来版も同じ契約(downgrade errorにしないテスト)。plugin kind違いが型付きエラーになるテスト |
| D8 | **完了**: 所有権モデルの骨格(F-2): 編集スレッド=単一writer、`Arc<Document>`スナップショット配布、バックグラウンド成果のメッセージ適用経路 | D1a | 「編集中にレンダスレッドが古いスナップショットで完走する」並行テスト。writer以外からの書き込み経路が型レベルで存在しない |
| D7 | **完了**(#145): クリッピングマスク合成。「下のレイヤーにクリップ」(クリスタ方式)をコンポジット処理とdoc→グラフ変換に実装。マスクモード=アルファ/ルミナンス/反転。**sRGBブレンド依存ゴールデンはregenerateマーカー必須**(台帳C-1系 / `d7_clipping_mask.rs` provisional) | D3 | 各モードのゴールデンイメージテスト(シェイプレイヤーを下に置いたクリップ結果)。`d7_clipping_mask.rs` + `mask_node` GPU |
| D2 | **完了**(#109/#130): コマンドシステム(apply/revert)+ Undo/Redo履歴(ガード5/12)。**#103決定織り込み**: A8安定ID・⑨プロパティ単位atomic+gesture macro+merge・Undo深さ0=unlimited(live/再起動後別limit)・複製時ID再写像。LayerId台帳はAdd/RemoveTrackItemの`layer_names`+`restore`でundo/redo対称 | D1i-1 **かつ** [#103決定](../reviews/2026-07-13-decision-pack-adoption.md) | 全編集コマンドに対し「apply→revert→状態一致」のプロパティテスト。安定IDアドレッシング。gesture mergeテスト |
| D3 | **完了**(#110): ドキュメント→レンダグラフ変換(motolii-docとmotolii-renderの接続層)。**凍結ゲートで確定した単一評価モデル(F-3: ソース(TimeMap)→エフェクトスタック→変形→クリッピングマスク→グループ合成の決定的順序)をそのまま実装する**。順序の解釈をこのタスク内で発明しない。**M2E-11⑤: ExportOverlayRequest形式のジョブミラーを温存せず、Document→render層リクエストを直結**。Document≠ExportJob(書き出し設定は別構造 — 採択: snapshot/ref・出力先・範囲・fps override・解像度・codec/container・音声mux)。**評価時刻の意味論宣言(#55、監査T-9/LG-3)**: v1の評価時刻はソースPTS(タイムライン=ソースの縮退)であり、D3でtimeline_timeに再定義する — この再定義を暗黙に行わず、変換層の宣言として明示する。**D1h二段化の後段**: 実行時結線で実DataTrack出力型と期待型を照合。**#103織り込み**: A4重なり禁止前提・B④3軸表・B⑦fps役割 | D1i-2(PathOp意味論確定後) **かつ** [#103決定](../reviews/2026-07-13-decision-pack-adoption.md) **かつ** 第二凍結点 | M1のE2Eテストがドキュメントモデル経由で通る。DataTrack出力型照合のテスト。マスク付きグループ内のエフェクト付きレイヤーの評価順ゴールデンテスト。グループ自体にエフェクトスタックを積んだ場合(子合成→グループスタック適用)の評価順ゴールデンテスト。B④3軸の挙動テスト |
| D4 | **完了**(#123): motolii-audio: 楽曲1本のSymphoniaデコード→PCM全展開キャッシュ + cpal出力 + リングバッファ/プロデューサスレッド(ミキサーなし)。アンダーラン時は不足フレーム数を記録し、実音声の供給済みサンプル数を無音充填数と分離して公開する。**責務境界(旧PR#90の再発防止)**: producer/ring(decode/cache)とdevice出力を分離し、cpal device出力にTransportクロックの所有権を持たせない — 再生開始位置はproducer起動時の`start_frame`のみが決め、映像frame drop・適応解像度はD5に委ねる。デバイスが素材レートを直接サポートしない場合は型付きエラーで拒否(固定比フォールバックは**D4-FU**) | 凍結ゲート | 任意位置からのサンプル読み出しテスト、アンダーランなしの連続再生。アンダーランの無音充填で論理サンプル位置が進まないテスト |
| D4-FU | **完了**(#147 / #153): デバイス≠素材サンプルレート時の**固定比リサンプル**フォールバック: rubato。プロデューサ側・リング書き込み前。レート一致時は恒等パスを維持。Transport時計・レイテンシ補償の意味は変えない(時計は引き続きデバイス供給済み)。**リサンプラのアルゴリズム遅延はproducer側のpre-roll／先頭trimで吸収し、Transport境界へ持ち出さない**(D5が引くのはcpalデバイス待ちのみを維持)。**ビット同一検査は変換前PCMキャッシュ境界に限定**(デバイス出力ビット同一は非要求)。永続スキーマに焼かない | D4 | 素材レート非対応デバイスでも再生開始できる。変換前PCMの読み出しがビット同一。変換後は正速・無欠落(アンダーラン分離カウンタ)・時刻連続。レート一致パスでリサンプラが挿入されない。**インパルス等で、変換後の時刻付きサンプルが期待出力フレームへ対応すること**(開始・シークを含む)を自動テスト |
| D5 | Transport(音声クロック常時主)(#144): 再生位置=**デバイスへ供給済み**サンプル数。映像はフレームドロップで追従。低速時はDRS(Draft 1/2→1/4、二重閾値+パニック+CPUバウンド)。timestamp query非対応時は**自動DRS無効+ドロップ継続**。なお不足なら音声正速のままドロップ継続。クロックオーナー交代・自動バリスピードは持たない。レイテンシ補償=**cpalデバイス待ちのみ**(リング充填は引かない)。[先例調査【採択】](../reviews/2026-07-14-d5-transport-prior-art.md) | D3, D4, **D4-FU** | **ドリフト**: 10分再生で\|表示フレームPTS−聴感時刻(供給済み−デバイス待ち)\| ≤ **1フレーム長**を自動判定。**0.5x律速**: 変換前PCMが正速・無欠落・**ビット同一**、映像はドロップ(+DRS可時は降格)で追従。**パンピング**: 閾値近傍の人工負荷で、最小滞留時間内に同一段階へ再復帰する振動が0回。**グリッチ**: 解像度切替前後でアンダーラン増加0かつコールバック供給サンプル列に不連続挿入が無い(自動)。`cargo test --workspace`全緑 |
| D6 | **完了**(#133 / #154): 書き出しへの楽曲mux: 元の音声ファイルをオフセット付きでffmpegに渡し映像とmux。ミキシングバウンス不要のため、コーデック互換ならストリームコピー(無劣化)を優先。D1f接続: `plugin_open_warnings`非空なら書き出し拒否(`ExportError::DegradedPlugins`)。未来版`doc.layer_source.rect`の除外迂回を#154で塞ぐ | D4 | 書き出したmp4の音声が元素材とサンプル一致し、映像とズレない(コンセプト: MVの最終書き出しに必須)。`cargo test -p motolii-export --test d6_audio_mux` / `cargo test -p motolii-media mux` 緑 |
| D1g | **完了**(#94): **第二監査フォローアップ①(S1/S2、[決定2026-07-12](../reviews/2026-07-12-code-audit-2nd-d1.md))**: TimeMapを「クリップローカル時刻→ソース時刻」へ変更し`timeline_start`を**削除**。固定式(このまま焼く): `clip_local_time = timeline_time - clip.start` / `source_time = time_map.map(clip_local_time)`。キーフレーム評価にはTimeMapを通さない(spec-holes §1と同一領域)。**正準化**: speed既約化+`Fps`既約化(M2E-16と同型の構築時不変条件。速度フィールド非公開でEq/Hashが意味同値と一致)、`is_identity()`は意味比較へ。**尺の正本宣言(2026-07-12ユーザー承認)**: v1は`Clip.duration`が表示区間の正本、source使用終端は **`source_end = time_map.map(clip.duration)`** で導出(=`time_map.map((clip.start + clip.duration) - clip.start)`の簡約。TimeMapはクリップローカル契約なので`start+duration`のタイムライン値を直接通さない。(b)ノット列導出はv1の定数速度TimeMapに終端ノットが無いため不可能 — §1c位置ノット列導入時に**再判定**(自動移行しない)。**OverrunMode席の確保(§7b消化、契約をここで固定)**: `enum OverrunMode { Freeze, Black, Loop }`をTimeMapのフィールドとして予約、永続キー`overrun_mode`、serde default=**Freeze**(§1bユーザー決定の候補3つをそのまま列挙)。**両端鏡像規約(B②消化)**: 解決済みsource時刻が素材available範囲(メディア総尺)の始端より前・終端以降の**両側に同一モードを適用**(Freeze=近い側の端フレームへクランプ / Black=非描画 / Loop=available範囲でwrap)。**責任境界**: TimeMapは素材尺を知らない純写像であり、モードを**保持するだけで適用しない** — 適用は素材尺を知るD3。v1のD3実装は**Freezeのみ**で、Black/Loop指定は型付き「未実装モード」エラー(黙ってFreezeへ縮退しない) | D1a | `timeline_start`がスキーマ・serdeに無い。OverrunModeのserde roundtrip+default=Freezeテスト。**D3以前のモード無視ガード**: ①`is_identity()`は正準アフィン写像が恒等**かつ`overrun_mode == Freeze`**の場合のみtrue ②D3完成前の全評価入口(現行の`try_map`直呼び経路)でBlack/Loopを型付き`UnsupportedOverrunMode`エラーとして拒否 ③Black/LoopがFreeze相当として描画されない回帰テスト(「保存はできる/未実装は黙って縮退しない」をD1g直後から成立させる。端フレームの厳密PTS・空available範囲の扱いはD3発注時に固定)。**旧`timeline_start`形式のJSONはD1gでは型付きエラーで拒否**し、新旧変換は**D1e(migration枠)の担当**(実行順はD1g→D1e。D1eの旧版コーパスへ`timeline_start`世代を追加)。`2/2`速度・`60/2`fpsが構築時に既約化される+意味的`is_identity`テスト。クリップ移動の不変条件テスト: `resolve(moved_clip, timeline_time + delta) == resolve(original_clip, timeline_time)`。既存の縮退挙動テストを仕様違反拒否テストへ反転。`cargo test --workspace`全緑 |
| D1h | **完了**(#98): **第二監査フォローアップ②(S3/S4/S9、同決定)**: 各DocParam受け口の**期待型表**(例: position=Vec2、opacity=F64、color=Color)をスキーマ側正本として宣言し、validateが Const / Keyframesの**全キー+同一variant** / Dataの**fallback型** / Vec2Axes内部 / **空トラック**=スキーマ拒否(期待型default注入は採らない)まで検査。**検査責任の二段化**: DataTrack本体はDocument外のキャッシュであり`DataTrackId`から実出力型を判定できないため、**D1hはDocument内で完結する型整合まで**(参照側が期待型を保持)。**実際のDataTrack出力型と期待型の照合はD3(実行時結線)の完了条件**とする。**AssetId結線**: 永続層のasset参照は`AssetId`型(`DocAssetRef`)へ — 存在検査・ダングリング拒否・cross-document copy時の再写像規約。評価層へは解決済み値で渡す。**非有限値・値域**: NaN/Infは全パラメータで拒否、範囲はパラメータごとにクランプ/拒否を宣言(B⑥消化) | D1g | `Transform.position = Color`等の型不一致・空トラック・`Value::AssetRef`ダングリングが**validateで型付きエラー**になる。評価側縮退(空→F64(0)、型不一致→前値/0.0)へ到達する入力がvalidateを通らないことをテストで固定(縮退テストの反転) |
| D1i-1 | **完了**(#99): **第二監査フォローアップ③a(S6、同決定)— VectorRecipe構造移動**: `Clip.path_ops`を削除し、Vector系ソースを`VectorRecipe { content: StandardShape\|SvgAsset\|TextPath\|Group, modifiers: Vec<PathOp> }`へ。固定規約: ①modifiersはrootの全パス集合に作用 ②index 0から順に適用 ③Trimのparallel/sequentialはPathOp自身のフィールド ④raster sourceには構造上存在しない(Lottie型入れ子スコープ=`Vec<VectorItem>`混在は**v2の別設計**であり本タスクで発明しない)。**永続JSON**: Asset/Vectorは未知フィールド拒否(Assetへ`recipe`/`modifiers`混入でserdeエラー。Pluginの`extra` flattenは維持)。**asset種別**: `SvgAsset`=`image/svg+xml`、`TextPath.font_asset`=`font/ttf`\|`font/otf`\|`font/woff`\|`font/woff2`(ここで正本化) | D1h | raster+modifiersが**型レベルで構文不能**かつ永続JSONでも拒否。VectorRecipeのJSON roundtrip。旧`path_ops`形式の扱いはD1g同様(D1i-1では拒否・変換はD1e)。動画AssetをSvgAsset/TextPathへ入れてもvalidateが`WrongAssetType` |
| D1i-2 | **完了**(#100): **第二監査フォローアップ③b(S5)— PathOp意味論表と実装**: 仕様の「PathOp意味論表」は**【決定】済み(2026-07-13)**。validate+幾何実装+variantゴールデンを表に一致させる。追加的席: `point_type` / `line_join`+`miter_limit` / `twist.center` / `repeater.transform`+opacity。Wiggle=`pcg32_value_noise`+u64 seed。開路Offsetは型付きunsupported。幾何実装は`motolii_doc::pathgeom`(解決済みスカラー受け取り。DocParamのt評価結線はD3) | D1i-1 | 意味論表が仕様に**【決定】**として存在する(済)。validateが表の拒否項目で型付きエラー(`d1i2_pathop_validate.rs`)。variantごとの保護された意味論ゴールデン(`d1i2_pathop_geometry.rs`)。`Repeater`に`transform`があり旧JSONはdefault恒等で読める。`Twist.center`は必須のため旧JSON(center無し)は型付き拒否(変換はD1e) |
| D1i-3 | **完了**(#108 / #139): **第二監査フォローアップ③c(S16)— 非PathOp演算の保護ゴールデン**: BlendMode / LookAt・Follow / Bezier / Transform合成の意味論ゴールデン。LookAtはconcept正本(`rotation=look_at`、PlusX/PlusY、`eval_look_at_rotation`/`eval_rotation`)。台帳=`classification.tsv`の semantic 行(本体更新禁止) | D3 | 各演算のゴールデンが存在し、意味を固定している。LookAtはconcept正本(`rotation=look_at`)と一致 |
| D1i-4 | **完了**(#107): **第二監査フォローアップ③d(S16執行)— ゴールデン更新禁止のCI検査**: ゴールデンを**2分類**に分ける — (a)**意味論ゴールデン**(S16。既存variantの意味を永久固定する審判。**更新禁止・例外なし** — regenerateマーカーでも迂回不可。変更したければ新variant+新ゴールデン) / (b)**暫定ゴールデン**(C-1系のsRGBブレンド依存等。regenerateマーカーで更新可)。CIは(a)への変更を機械的に拒否する。台帳=`crates/motolii-testkit/golden_policy/classification.tsv`、施行=`scripts/check-golden-update-policy.sh`+`golden_update_policy` | D1i-2 | 意味論ゴールデンを書き換えるPRが**マーカー有無に関わらず**CIで落ちる施行テスト。暫定ゴールデンはマーカー付きでのみ更新できる分類テスト |

## 将来境界の持越し決定（実装なし）

| ID | 状態 | 内容 | 依存 | 完了条件(概要) |
|---|---|---|---|---|
| GAP-15 | **決定（実装なし）**: Param Pipeline / generic Element Domain / Constraint GraphはM2恒久面へ追加せず、現行`DocParam`/typed ID/LookAt・Follow意味を維持。persistent offset/複数source/parameter modifier→PP-Gate、cross-kind永続ID→ED-Gate、generic constraint/order/plugin→CG-Gateを先に通す。本行は再締結中のM3停止を解除しない | [持越し境界](../reviews/2026-07-16-m2-param-element-constraint-disposition.md), GR-PV | (1)既存field/variant解釈不変 (2)各発火条件前にdecision/spec PR必須 (3)再締結解除後のM3通常property UIは現行single sourceだけ (4)将来変更は追加的またはD1e明示migration |

## Shared Effectレーン(コア外・独立PR)

正本: [2026-07-15-relative-scope-duplicator-decision.md](../reviews/2026-07-15-relative-scope-duplicator-decision.md)、lifecycle=[GAP-14決定](../reviews/2026-07-15-shared-effect-lifecycle-decision.md)、[journal/Undo追補](../reviews/2026-07-15-d1l-journal-revert-boundary-decision.md)、[新規Document v4生成](../reviews/2026-07-16-d1l-current-document-constructor-decision.md)。Definition/Use分離、lifecycle、journal/Undo/Writer境界は決定済み。

| ID | 状態 | 内容 | 依存 | 完了条件(概要) |
|---|---|---|---|---|
| GAP-14 | **完了**(#166): 参照中Delete=Reject、Unlink=RemoveUse、Copy Local=Materialize、orphan=Keep、未知plugin同一規則。Cascade/Purgeは延期 | — | [lifecycle決定](../reviews/2026-07-15-shared-effect-lifecycle-decision.md)の表・不変条件・試験列挙 |
| D1l | **READY**(実装PR #173を再開): Effect Definition/Use共有schema+inline migration。Effect Definition台帳+Use参照、非隣接共有、Group同stack、Composite Set/Backdropなし。v2 lifecycle Commandをv1 wireへ必須field追加せず新variant化。Writer prepareがCreate/Link/Copy Localを内部採番し、Copy Localは自己完結payload+nested Keyframe全再採番。製品の新規Documentは`new_current()`でv4とし、`new_v1()`はlegacy専用 | D1e, D1f, D1i-2, **GAP-14**, [journal/Undo追補](../reviews/2026-07-15-d1l-journal-revert-boundary-decision.md), [新規Document v4生成](../reviews/2026-07-16-d1l-current-document-constructor-decision.md), GR-PV | (1)inline migrationでpixel/order/extra/要素数保持、旧reader拒否+再migration冪等 (2)ID一意・参照整合・欠落拒否 (3)共有/同layer複数Use roundtrip・rename/reorder参照不変 (4)target集合/timeline隣接非保存、Composite Set/Backdrop構文不能 (5)GAP-14 §4–§5全条件 (6)nested Keyframe Copy Localの固定順再採番・payload完全性・Undo干渉Reject (7)v1全Command corpusをD1e共用plannerでlossless replay、v1/v2混在 (8)Create/Link/Copy Local準備はDocument不変・raw caller採番なし (9)reservation閉集合6 variantだけcounter以外全文一致+非巻戻し、version/min不変、Redo全文一致 (10)Unlinkに偽reservationなし、全失敗時Document全文不変 (11)reader/writer/min=4、`new_current()` roundtrip、`new_v1()`非test利用拒否、v1〜v3と版偽装inline/hybrid入力の型付き拒否を機械判定 |
| D1m | **READY（仕様決定、実装未着手）**: project-scoped sidecar identity + process間read-write session lock。`<parent>/<file-name>.motolii/`とsibling `.motolii.lock`へnative `OsString` suffixで一意化し、canonical path aliasを同一identityへ畳み、OSの非blocking exclusive lockを保持する`ProjectSession`経由だけでproject/journalを変更する。legacy親共有layoutは自動帰属せずstaging検証後のatomic installのみ。read-only fallback/lock steal/Save Asは作らない | D1c-FU, D1d, [sidecar/session決定](../reviews/2026-07-16-m2-project-sidecar-session-decision.md), GR-PV | (1)同一directoryの2 projectでsidecar全要素が非衝突、相互byte不変 (2)subprocess保持中の同一path/symlink alias openが即時`ProjectAlreadyOpen`、Windows CI+macOS証跡 (3)正常Drop/強制終了後に再取得+復旧可 (4)root公開open/mutationは`ProjectSession`だけ、raw-path/WalSession bypassなし (5)family存在述語/状態表どおりlegacy通常open・partial・invalidをtyped reject、明示移行はsource保持・media除外・verify・atomic install・再実行冪等 (6)D1d意味/保護golden不変、path literalのみhelper化可 (7)`cargo test --workspace`全緑 |
| D3e | **WAIT**: Shared Effect Use評価接続。各Use位置で個別評価、同definition params共有。Groupは子合成後1回。source非消費 | D1l, D3 | (1)非隣接3 layerが同definitionを各stack位置で使いinline複製とpixel同一 (2)definition変更で全useの画が変わる (3)use並べ替えは対象layerだけ (4)Groupは子合成後1回 (5)timeline順/renameでpixel不変 (6)欠落typed error (7)preview/export同一 (8)Composite Set/Backdropへ縮退しない |

## CompCameraレーン（M2再締結対象・直列）

正本: [planar v1 camera決定](../reviews/2026-07-16-m2-comp-camera-decision.md)。単一camera/単一worldを採択するが、M2恒久schemaは`PlanarOrthographic`だけ。D1lのv4と競合させず、Spatial/Perspectiveを推測で足さない。

| ID | 状態 | 内容 | 依存 | 完了条件(概要) |
|---|---|---|---|---|
| CAM-G0 | **WAIT（camera決定main待ち）**: camera接続前の既存`ViewportTransform` 2D出力を固定fixture `crates/motolii-render/tests/cam_g0_planar_identity.rs`で採取しsemantic分類。matching aspectの単純solid形状でGPU出力byteを固定 | camera決定, GR-PV | fixture/classification追加だけの独立PR。golden policyが期待値変更を拒否し、`cargo test --workspace`全緑 |
| D1j | **WAIT（CAM-G0+D1l main待ち）**: `LATEST/READER/WRITER==4`を前置確認後、その直後のversion(v5)へinternally-tagged `Composition.camera: CompCameraDoc::PlanarOrthographic { center: DocParam<Vec2>, roll_radians: DocParam<F64>, height: DocParam<F64> }`を追加。v1〜v4へ既定cameraをD1e migrationし、版偽装をtyped reject | CAM-G0, D1l, D1e, camera決定, GR-PV | versionが4以外なら停止。migration冪等、counts/ID/extra不変、wire roundtrip/未知kind拒否、非有限/height<=0拒否、reader/writer/min=5、旧reader拒否、v1〜v4版偽装拒否 |
| D1k | **WAIT**: radian+planar orthographic runtime camera、camera-bearing render入力、CQ-5解凍。aspect=Composition正本、FrameDesc mismatch typed reject、Draft整数除算がaspectを壊す時は入力desc不変、resolutionを重複保持せずSlint非依存 | D1j, CQ-5 review | 解凍記録に旧/新API・`LayerSourceContext`/`RenderGraphInputs.camera`・degree→radian・migration非影響・goldenを列挙。center/corner/roll/NDC→Y-down px fixture、typed invalid/aspect mismatch、`1920x1080/2=960x540`・`16x9/2=16x9`、preview/export同一関数、旧position/target/FOV shim禁止 |
| D3f | **WAIT**: Document cameraを`t`で評価し2D graph/renderへ接続。旧Viewportとの二重変換、layer順/Z意味変更を禁止 | D1k, D3 | CAM-G0 byte不変、既存transform/eval期待値不変、center/roll/height animation方程式一致、preview/export同一、depth/order不変 |

並列レーン(2026-07-16改訂): **D1a**→(D1b/D1f と D8)→D1c→D1c-FU(#101)→D1d→**D1m**。第二監査フォローアップは**D1g→D1h→D1i-1→D1i-2(直列)**。**D2はD1i-1後、D3はD1i-2後**、D3→D7。D1eはD1i-1かつD1c-FU後。D1i-3はD3後、D1i-4はD1i-2後。D4は独立。D5が合流点。D6はD4後。旧形式JSONの変換は全てD1e担当(D1g/D1i-1は拒否のみ)。**Shared Effect**: GAP-14→D1l→D3e。**保存所有権**: D1d→D1m。**camera**: CAM-G0→D1j→D1k→D3f(上のCompCameraレーン)。**予防**: この直列を飛ばす並列発注はGR-PV-4違反([permanence-prevention](../reviews/2026-07-12-m2-permanence-prevention.md)、実装ガード16)。

## 音声一般化レーン(M2コア外・独立PR)

正本: [2026-07-14-audio-generalization-design.md](../reviews/2026-07-14-audio-generalization-design.md)。M2コア締結後に着手。依存は `AG-1 → AG-2 → AG-3 / AG-4`。

| ID | 状態 | 内容 | 依存 | 完了条件(概要) |
|---|---|---|---|---|
| AG-0 | **完了**(本PR): 「楽曲1本=恒久制約」を既定導線へ縮退。設計決定をdocsへ固定 | — | コード/serde変更なし(本PRではAG-1と同居) |
| AG-1 | **完了**(本PR / #156): 全stream probe、`StreamSelector`+Asset Clip video/audio component、旧欠落default=video only、`min_reader_version=3`規律 | AG-0 | 旧project意味不変、roundtrip、欠落stream拒否、video/audio/audio-only fixture、`cargo test --workspace` |
| AG-2 | **完了**(#157): per-stream PCM cacheと決定論的mixer | AG-1 | 設計§6/Issue完了条件 |
| AG-3 | **進行中**(Part of #158 / PR#164): commands/分離/waveform peaks。Slint UIは別Issue・M3 | AG-1, AG-2 | 設計§7のdomain受け入れ+拒否テスト。#158はUI完了まで閉じない |
| AG-4 | **完了**(#159): stream-copy fast pathとmixed PCM export | AG-1, AG-2 | 設計§6.3/Issue完了条件 |
| AG-5 | **追跡のみ**(#160): fade/pan/role/bus/effect/pitch preserve。**一括実装禁止**。候補はすべて保留(需要・意味論表待ち) | AG-1〜4後に個別判断 | 子Issue分割前に意味論表+需要記録。本台帳へ直実装しない |

**2026-07-12 第二監査によるフォローアップ**: D1a/D1b/D1cは「完了」だが、[第二実コード監査](../reviews/2026-07-12-code-audit-2nd-d1.md)がスキーマ・検証の穴S1〜S18を確認した(D1a: S1/S2/S5/S6/S16、D1b: S3/S4/S9、D1c: S10/S14)。**S1/S6/S16は2026-07-12ユーザー決定(3点とも案1、補正4点込み — 同文書決定節)**。スキーマ側フォローアップは**D1g/D1h/D1i-1〜D1i-4**(上表)として発注済み。D1c分(S10/S14)の責任は**D1c-FU(#101)**へ集約し、D1d(#105)・D1e(#104)はその型を利用する。D1f(#106、S13織り込み済み)は完了。未着手のD2・D3は同文書の該当行を各発注書へ織り込むこと。

## 実装ガード(先行ツールの失敗・ユーザー不満クロスチェック 2026-07-11)

出荷済みエディタのプロジェクト破損・互換性・Undo苦情と、ジャーナリングの参照実装(SQLite WAL)・イベントソーシング実務を調査し、既存方針(C-1/C-2/F-9、上記「方針」)に無いガードを抽出した。割当は上記D1a〜D1f / D2へ織り込み済み。

1. **保存前に不変条件を検証する** → **D1b**
2. **アトミック保存の完全形+クラッシュ注入テスト** → **D1c**
3. **ジャーナルはSQLite WALの壊れ方カタログを直輸入する** → **D1d**
4. **リプレイ失敗のフォールバックを一次機能として設計する** → **D1d**(ジャーナル形式の版はD1eと接続)
5. **コマンドは「意図」でなく「決定済みの値」を記録する** → **D2**
6. **オートセーブ世代は件数ローテーションだけにしない** → **D1d**
7. **前方互換の枠組み** → 骨格はM2E-12済み。I/O拒否は**D1c**、版上げ経路は**D1e**
8. **マイグレーションの量的不変条件+旧版コーパス** → **D1e**
9. **欠落プラグインは「開く」と「書き出す」で厳格さを変える** → 開く=**D1f**、書き出す=**D6**
10. **素材参照は多重キーのフォールバックチェーン** → **D1a**のAsset定義(解決器の実装もD1aに含めてよいが、最低限フィールドのserde)
11. **クラウド同期フォルダの検出と外部変更の監視** → 口は**D1c**(オープン経路)、UIはM3
12. **Undoの退行防止** → **D2**

出典: community.adobe.com(Premiere破損/オートセーブ全滅/undo履歴バグ/missing footage) / discussions.apple.com(FCPX+Dropbox) / support.alightmotion.com(再インストールで全プロジェクト消滅) / KDE Bug 353125(Kdenliveマイグレーション欠落) / sqlite.org/atomiccommit.html・wal.html・howtocorrupt.html / forum.ableton.com(復元ループ) / developer.blender.org T74072(undoフリーズ) / manual.ardour.org(履歴永続化と深さ設定)

### 恒久焼き込みの予防(2026-07-12)

正本: [reviews/2026-07-12-m2-permanence-prevention.md](../reviews/2026-07-12-m2-permanence-prevention.md)。落とし穴H-4。エージェント入口は[AGENTS.md](../../AGENTS.md)「恒久焼き込みの予防」。

13. **意味を先に仕様へ書く**(GR-PV-1) — PathOp意味論表は**【決定】済み(2026-07-13)**。D1i-2は表の写しとして実装
14. **恒久面を狭く保つ**(GR-PV-2) — 未決・未証明・UI都合だけのフィールドをスキーマに足さない
15. **追加的変更のみ**(GR-PV-3) — 新フィールド/新variant。既存フィールドの解釈変更は禁止。破れたときだけ形状→D1e / 画素→新variant(D1i-4)
16. **依存直列を守る**(GR-PV-4) — **D1i-2完了前にD3しない**。レーン表の独断変更禁止
17. **完了=意味の審判**(GR-PV-5) — テスト緑だけでタスク表を「完了」にしない。migration/フォローアップPRにはnon-goals(Git型、副次)

## 未決事項

- ~~コマンド粒度/UI編集状態~~ → **【決定】#103⑨**(2026-07-13採択文書)
- 波形表示用のピークデータ生成をどこで持つか — **低優先化済み(2026-07-12)**: 持ち場はM3/U3発注時にUI都合で決めてよい。**M2ブロッカーではない**
- ~~CompCameraのv1採否/恒久意味~~ → **【決定】D1j〜D3f**(2026-07-16): 常在単一cameraを採択し、v1は`PlanarOrthographic`のみ。Spatial/PerspectiveはM5の新variant決定へ延期
- ~~Param Pipeline / generic Element Domain / Constraint GraphのM2持越し境界~~ → **【決定】GAP-15**(2026-07-16): 現行意味不変で延期し、各発火条件前にPP/ED/CG-Gateを必須化
- ~~process間lock / stale lock / read-only fallback~~ → **【決定】D1m**(2026-07-16): `<file-name>.motolii` sidecar + OS exclusive `ProjectSession`をM2再締結前に実装。legacy自動帰属・stale推測・lock steal・read-only fallbackは非目標

D2/D3着手前の決定パック(#103)・PathOp(#100)・残小項目は[採択文書](../reviews/2026-07-13-decision-pack-adoption.md)で**【決定】済み**。追加の広範調査は不要。例外は実装中の既存仕様同士の矛盾発見時のみ。
