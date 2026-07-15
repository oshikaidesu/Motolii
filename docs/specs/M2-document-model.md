# M2: ドキュメントモデルとタイムライン

ステータス: **ドラフト**(凍結ゲートで確定。**2026-07-15**: Shared Effect Definition/UseをD1l schema/migration→D3e評価へ分割。**2026-07-14**: [統一カメラ設計](../reviews/2026-07-14-unified-stage-camera-design.md)を受け、共有`CompCamera`をD1j schema→D1k runtime→D3接続へ前倒し。一般音声設計を決定。**2026-07-13**: [#103決定パック](../reviews/2026-07-13-decision-pack-adoption.md)・PathOp意味論表採択)
着手条件: **Documentスキーマに触るタスク(D1/D2/D3/D7/D8)は[M2入場条件](../reviews/2026-07-11-M2-entry-gate.md)の全緑後に発注する**(D4/D6はDocumentスキーマから独立のため対象外 — 凍結ゲートのみで着手可。理由: 恒久性×並列化初陣×検証の弱さが重なる最初のフェーズであるため、審判の穴・プラグイン境界の乗算穴・D1が継承する罠を先に塞ぐ)

## 目的(退治する落とし穴)

C-1(Undo後付け)、C-2(スキーマ進化)、B-1(音声・時間表現)、F-1(座標系)、F-2(所有権)、F-4(時間写像)。

## 方針

- プロジェクト状態は単一のserde可能な純データ構造(ドキュメントモデル)。エンジン(motolii-render)はこれを読むだけ
- ドキュメントモデルはAsset実体とプロジェクトBPM(手動入力。ビート検出はやらない)を持つ。**Assetは一般アセット(opaqueペイロード+type文字列+内容ハッシュ)として定義し、動画・SVGはその特殊ケース**(F-10、[plugin-resources.md](../plugin-resources.md)§3 D2/D3。将来のImporterプラグイン=点群等の口。`ValueType::AssetRef`の予約と対)。この定義は凍結ゲートのF-10確定に依存する — 確定前にD1を並列レーンへ出さない
- **M2実装の音声はプロジェクト直下の単一フィールド(楽曲1本+開始オフセット+マスターゲイン)**。M2中は音声トラック・ミキサー・クリップ音声を実装せず、動画クリップ内蔵音声はmute。ただしこれはMVの最短出口であり恒久的なDocument制約ではない。将来はAsset Clipがvideo/audio componentを共有時刻で持つ[一般音声設計](../reviews/2026-07-14-audio-generalization-design.md)へ追加的に広げる。現行`Soundtrack`と旧Asset Clipの意味は変更しない
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
3. `CompCamera`の追加は[統一カメラ設計](../reviews/2026-07-14-unified-stage-camera-design.md)で【決定】済み。D1aの旧readerがnested `camera`を黙殺するため、D1jで`min_reader_version`を同時に上げ、D1e経路で旧projectへ既定cameraを追加する。schemaだけを先に足さない

## 統一カメラの永続意味(D1j / **【決定】** 2026-07-14)

2D/3Dで別の制作世界を持たない。全Compositionは共有`CompCamera`をちょうど1つ持ち、既存2D objectは同じ正準XYZ世界の`z=0`平面へ投影する。`Output Frame`はcamera projection apertureそのものであり、別のcrop transformを保存しない。

```text
CompCameraDoc {
  position: [DocParam<F64>; 3],       // 正準XYZ
  target: [DocParam<F64>; 3],         // 正準XYZ
  roll_radians: DocParam<F64>,
  projection: Orthographic {
    height: DocParam<F64>,             // 正準高さ
  } | Perspective {
    fov_y_radians: DocParam<F64>,
  }
}
```

- 既定は`position=[0,0,1]`、`target=[0,0,0]`、`roll=0`、`Orthographic{height=1}`。既存2D projectのOutput Frame内の画を変えないmigration値とする
- aspectは既存`Composition`の有理アスペクトが正本。cameraへ重複保存しない
- UIのdegree表示、logical/physical px、DPI、Stage View pan/zoom/fitはDocumentへ入れない
- 全camera parameterは既存DocParamと同じ時刻`t`で評価し、D2 command/Undo/cache invalidationへ載せる
- 非有限値、`position==target`、`height<=0`、`fov_y_radians`が`(0, π)`外は型付きvalidate error
- 現行`motolii-core::CompCamera`のdegree/Perspective固定形をそのままserdeしない。D1jは追加schema+default migration、D1kはCQ-5 runtime契約+解凍へ分け、既存意味論goldenを更新で通さない

D1aの「Compositionにcamera欄が無い」はD1a完了時の歴史的受け入れ条件であり、D1j後の最終schema条件ではない。D1jで当該拒否testを理由付きで新契約のmigration/roundtrip testへ置換する。

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

1. **`RationalTime`**: (a)`den==0`拒否 (b)負の分母は符号を分子へ (c)既約化 (d)`0/x`→`0/1` (e)正規化後のi64溢れは`RationalTimeError::Overflow`(panicしない)。公開経路は`try_new`/`try_from_frame`/`try_to_frame_floor`/`try_add`/`try_sub`/`try_mul`/`try_neg`のみ(中間演算はchecked)
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

## 音声一般化の境界(2026-07-14決定)

正本は[音声一般化設計](../reviews/2026-07-14-audio-generalization-design.md)。M2は楽曲1本で閉じるが、次を恒久境界として確認する。

1. **MV特化はUI既定でありファイル形式のproject modeではない**: `Soundtrack` music bedを最短導線として維持し、一般編集ではAsset Clipがvideo/audio componentを持てる。同じDocument内で共存可能にする
2. **1 ClipがA/V時刻を所有する**: 音付き動画をvideo/audioの2項目へ暗黙分割しない。全componentがClipの`start`/`duration`/`TimeMap`を共有し、move/trim/retimeで同期を壊さない
3. **旧意味を変えない**: stream選択の無い現行`ClipSource::Asset`はvideo ordinal 0/audioなし。既存projectを開いただけで音を出さない。新field実装時は`min_reader_version`を上げる
4. **分離はcommand**: 音声の独立編集はaudio-only Clipをmaterializeする1 macro。隠れlink/controllerを永続化しない
5. **Transportは維持**: 一般化後は単一PCMの代わりにdeterministic mixerの`AudioProgram`を入力する。audio主clock・非blocking callback・preview/export同一意味は変えない
6. **実装はM2へ割り込ませない**: AG-1(schema/media)→AG-2(mixer)→AG-3(UI)/AG-4(export)の独立レーン。schema着手時はGR-PV解凍手続きと旧project意味不変テストを先に行う

## 音声トランスポート設計(音ズレ・途切れの構造的排除)

コンセプト決定(2026-07-06/07)を受けた根本設計。「音がズレる/途切れる/重い時に映像だけ遅れる」を後付けで直すのではなく、クロックの所有構造で不可能にする。

1. **再生ヘッド(Transport)の所有者は常にちょうど1つ**。音声も映像も追従者(follower)であり、勝手に自分の時計で進むコンポーネントを作らない。ズレとは「時計が2つある」ことの症状であり、時計を1つにすれば定義上発生しない
2. **通常再生モード**: 音声デバイスクロックが主。再生位置 = 「デバイスに供給済みのサンプル数」から導出(壁時計は使わない)。映像は現在位置に最も近いフレームを表示し、間に合わなければフレームドロップ(音は絶対に途切れない・ズレない)
3. **バリスピードモード(レンダが実時間に届かない時)**: レンダ進捗がクロックの所有者に交代し、音声が適応リサンプリング(rubato等)で追従する。再生速度が0.6xしか出ないなら音も0.6x(ピッチ低下)で連続再生される。「映像だけ遅れて音が先行する」状態が構造的に存在しない。動画プレイヤーの adaptive resampling(mpv/ffplayの映像同期モード)と同じ確立された手法
4. **音声コールバックは絶対にブロックしない**: コールバックはリングバッファから読むだけ。デコード・リサンプリングは専用プロデューサスレッド。アンダーラン時は無音を出力しカウンタに記録(クロックは進めない)
5. **M2では楽曲をインポート時にPCM全展開してRAM保持**: M2は音声=楽曲1本なので「再生位置→サンプル」を単一バッファの添字計算へ還元する(5分ステレオ48kHz f32 ≈ 110MB)。一般音声AG-2ではこのTransport契約を保ったまま、入力をper-stream PCM cache+deterministic mixerへ置換する
6. **レイテンシ補償**: 表示するフレームは「いま聞こえている音」に対応させる(出力レイテンシ分を引いた位置)

## タスク分割(1PR粒度。旧粗案D1をD1a〜D1fへ再分割)

旧「D1」はスキーマ本体+永続化+ジャーナル+マイグレーションが1行に詰まっていたため、仕様ルール1(1タスク=1PR)に合わせて分割する。**D1完了= D1a〜D1k全緑**(D1g〜D1i-4は第二監査、D1j/D1kは統一カメラ決定)。D1j=doc schema/migration、D1k=core/render runtime契約に分け、D3が接続する。

### 操作単純化モデルへの割当

M2は[操作単純化モデル](../interaction-simplicity-model.md)の**意味・Undo・可搬性**を担当する。UI都合のfieldやイベント列をDocumentへ追加しない。

- **D2**: Direct/Tool/Advancedの入口が違っても同じDomain Intentは同じ決定済み値のcommandへ正規化する。代表操作コーパスで1 gesture/tool確定=1 macro、Cancel=変更ゼロ、隠れhelper生成なしをproperty testする。
- **D1a/D1h/D3**: target、DataTrack、scope等は型付きIDと期待型で検査し、layer名・property pathの文字列参照を作らない。
- **D1f/D6**: 未知pluginを保持してProjectを開けることと、再現不能な書き出しを拒否することを分ける。使用箇所と理由をtyped diagnosticにする。
- **D3/D1i-3**: UI入口に依存せず、意味同値なDocumentが同じrender graphと画になることをgoldenで固定する。
- **停止条件**: Autograph型Modifier列、汎用Element Domain、永続Constraint Graphは未決であり、本書の[PP-Gate](../interaction-simplicity-model.md#4-param-pipeline-gatepp-gate)等の独立仕様改訂前に既存`DocParam`へ焼かない。Relative MoveはM3-U2fのmodifier+drag one-shotで完結し、常設offsetへ拡張しない。Bounds/ROIはderived runtime契約でDocumentへ保存しない。Explicit Effectは[2026-07-15決定](../reviews/2026-07-15-relative-scope-duplicator-decision.md)に従いD1l/D3eだけで追加し、Composite SetやBackdrop評価地点を同時に発明しない。**D1lのschema実装はGAP-14で参照中Definition削除・unlink/copy-local・orphan処理を決めるまで開始しない**。instance/element一覧はM5-P0I/P7a前に保存しない。**M2終了判定**: Param PipelineはM2 blockerにせず、M3で常設補正/高度property UIへ着手する前の解凍ゲートとする（[判定記録](../reviews/2026-07-14-m2-exit-param-pipeline-disposition.md)）。

| ID | 内容 | 依存 | 完了条件(概要) |
|---|---|---|---|
| D1a | **完了**: **スキーマ本体+serde**(I/Oなし): Composition(有理数アスペクト・duration・fps。**`CompCamera`非包含** — #55/監査C-7)/既約有理数BPM/Soundtrack(楽曲1本・gain∈[0,1])/Asset一般(**D1aはパス+type+hashのメタのみ**。opaqueペイロード本体はImporter/GpuAssetCache側 — ファイル実体は多重キーpathが指す。ガード10)/`AssetId`・`TrackId`台帳(再利用禁止・LayerIdと同型)/Track・TrackItem(Clip\|Group再帰)/項目エンベロープ/クリップTimeMap/DocParam(Const・Keyframes・Data・Vec2Axes・LookAt・Follow。LayerIdはdoc内のみ。serdeはsnake_case — ProjectV1のParamSourceとは別名空間)/PathOp閉集合/Effect・Pluginソースに`extra` flatten予約(F-9の席。警告はD1f)。prelude継承。**ProjectV1非継承**。プロジェクト直下仮`time_map`はクリップへ | 凍結ゲート, **[M2入場条件](../reviews/2026-07-11-M2-entry-gate.md) 全緑** | JSON roundtripでグループネスト・エフェクト付きグループ・TimeMap・親参照・PathOp・Asset多重キーが保存される。**拍時刻がRationalTimeに畳める**。Composition型にcamera欄が無い。未知トップレベルキーは`extra`で保持。Effect/Pluginの未知フィールドも`extra`で保持 |
| D1b | **完了**: 保存前不変条件検証(ガード1): 参照整合・ID一意・区間正当性。失敗時は既存スナップショットを壊さない口 | D1a | 壊れたDocumentが`validate`で型付きエラー。正常系は通る |
| D1c | **完了**: アトミック保存+読込(ガード2)+`min_reader_version`超過拒否(ガード7のI/O側) | D1a | temp→fsync→rename→dir fsync。各段abort注入で旧ファイルまたはジャーナルから復元可(ジャーナル本体はD1d) |
| D1d | ジャーナル(ガード3/4/6): レコードchecksum+世代salt・不正テール切捨て・UUID相互参照・リプレイ失敗フォールバック・ピン留め世代 | D1c | 壊れ方カタログの単体/注入テスト緑 |
| D1e | マイグレーション枠+量的不変条件+旧版コーパス(ガード8)。前方互換の版上げ経路。**第二監査分の個別migration(旧`timeline_start`形式・旧`path_ops`形式→新形式)を含む** | **D1i-1**(migration枠の骨格設計のみD1a後に先行可。変換実装は移行先スキーマが確定するD1i-1後) | in-place禁止・バックアップ・クリップ/トラック/キー数一致・goldenコーパス回帰。旧版コーパスに`timeline_start`世代・旧`path_ops`世代を含む |
| D1j | **統一`CompCamera` document schema+default migration**: 上記`CompCameraDoc`をCompositionへ追加し、既定Orthographic cameraを旧projectへmigration。`min_reader_version`を上げる。core/render/Stage Viewは触らない | D1e, D1i-2, [統一カメラ設計](../reviews/2026-07-14-unified-stage-camera-design.md) | (1)旧projectのDocument要素数不変+既定camera追加 (2)camera全variant/animation roundtrip (3)非有限・退化・範囲外拒否 (4)旧reader拒否 (5)レイヤー別cameraが構文不能 (6)degree/px/DPI/Stage View field不在 |
| D1k | **CQ-5 runtime camera+Render入力の解凍**: `motolii-core::CompCamera`をradian+Orthographic/PerspectiveでD1j意味へ整合し、`RenderGraphInputs.camera`を必須化。dispatchの`DEFAULT`直書きを拒否する。Document schemaは触らない | D1j, 凍結ゲート解凍手続き | (1)runtime validateがD1jと同じ受理範囲 (2)orthographic height=1で正準`z=0`高がframe高へ一致 (3)camera入力欠落/非対応projectionを型付き拒否 (4)LayerSourceへ同じcameraが届く (5)既存2D pixelはD3接続前の専用fixtureで保護 |
| D1l | **Effect Definition/Use共有schema+inline migration**: plugin/version/enabled/params/extraを持つ`EffectDefinition`をDocument台帳へ置き、各ItemEnvelopeの順序付きstackは`EffectUseId + EffectDefinitionId`参照を持つ。同じdefinitionを非隣接layerの複数useから共有できる。Group effectは同じstack形でOwned合成後へ作用。Composite Set/Backdropは追加しない | D1e, D1f, D1i-2, **GAP-14決定**, [2026-07-15決定](../reviews/2026-07-15-relative-scope-duplicator-decision.md), GR-PV | (1)既存inline EffectInstanceをdefinition 1件+use 1件へ一対一migrationしpixel/order/plugin未知field/要素数を保持 (2)definition/use ID一意・参照整合・欠落拒否 (3)同definitionの複数use roundtrip (4)同layer内の同definition複数useはuse IDで識別 (5)rename/reorderで参照不変 (6)旧reader拒否+再migration冪等 (7)target集合やtimeline隣接をdefinitionへ保存しない (8)Composite Set/Backdrop variantが構文不能 (9)GAP-14のdelete/unlink/copy-local/orphan各操作が1 Undoかつ未知pluginを失わずroundtrip |
| D1f | 未知プラグインID: 開く=警告+パススルー・再保存で未知部分喪失なし(F-9、ガード9の「開く」側)。書き出し厳格化はD6接続 | D1a | 未知`plugin_id`を含むJSONがロード成功+警告+roundtrip保持 |
| D8 | **完了**: 所有権モデルの骨格(F-2): 編集スレッド=単一writer、`Arc<Document>`スナップショット配布、バックグラウンド成果のメッセージ適用経路 | D1a | 「編集中にレンダスレッドが古いスナップショットで完走する」並行テスト。writer以外からの書き込み経路が型レベルで存在しない |
| D7 | クリッピングマスク合成: 「下のレイヤーにクリップ」(クリスタ方式)をコンポジット処理とdoc→グラフ変換に実装。マスクモード=アルファ/ルミナンス/反転。**sRGBブレンド依存ゴールデンはregenerateマーカー必須**(台帳C-1系) | D3 | 各モードのゴールデンイメージテスト(シェイプレイヤーを下に置いたクリップ結果) |
| D2 | コマンドシステム(apply/revert)+ Undo/Redo履歴(ガード5/12)。**#103決定織り込み**: A8安定ID・⑨プロパティ単位atomic+gesture macro+merge・Undo深さ0=unlimited(live/再起動後別limit)・複製時ID再写像 | D1i-1 **かつ** [#103決定](../reviews/2026-07-13-decision-pack-adoption.md) | 全編集コマンドに対し「apply→revert→状態一致」のプロパティテスト。安定IDアドレッシング。gesture mergeテスト |
| D3 | ドキュメント→レンダグラフ変換(motolii-docとmotolii-renderの接続層)。**凍結済み単一評価モデルをそのまま実装**し、順序を発明しない。Document≠ExportJob。評価時刻はtimeline_timeへ明示再定義する。**統一camera**: D1jを時刻`t`で評価してD1kのRender入力へ渡し、既存2D objectを`z=0`へ投影。**型付き参照**: LookAt/Follow/Parentを`LayerId`依存グラフで参照先から評価し、循環/欠落をtyped error。D1h後段/#103も実装 | D1i-2, D1k, [#103決定](../reviews/2026-07-13-decision-pack-adoption.md), 第二凍結点 | M1 E2E/DataTrack型照合。camera animationで全layer投影が変わりレイヤー別cameraなし。既定cameraの2D pixel同一。LookAt/Follow/Parentの参照順・rename不変・cycle拒否golden。mask/group stack/B④3軸の評価順golden |
| D3e | **Shared Effect Use評価接続**: D1l definitionを各ItemEnvelopeのUse位置で個別評価し、同definitionのparamsを共有する。Groupは子を合成後に同じstackを一度評価する。definition→全use依存をrender graphへ列挙。source layerを複製/消費/再合成しない | D1l, D3 | (1)非隣接3 layerが同definitionを各stack位置で使い、個別inline複製fixtureとpixel同一 (2)definition param変更で全useの画が変わる (3)use並べ替えは対象layerだけ変化 (4)Group fixtureは子合成後1回でper-child適用と区別 (5)target timeline順/renameでpixel不変 (6)欠落definition/useをtyped error (7)preview/export同一 (8)Composite Set/Backdropへ黙って縮退しない |
| D4 | motolii-audio: 楽曲1本のSymphoniaデコード→PCM全展開キャッシュ + cpal出力 + リングバッファ/プロデューサスレッド(ミキサーなし) | 凍結ゲート | 任意位置からのサンプル読み出しテスト、アンダーランなしの連続再生 |
| D5 | Transport(クロックオーナー交代式): 通常=音声クロック主+映像フレームドロップ、低速時=レンダ進捗主+適応リサンプリング追従 | D3, D4 | 10分素材でA/Vドリフトなし(実測)。レンダを人工的に0.5xに律速した状態で音声が途切れずピッチ同調して追従するテスト。**可聴アーティファクト(クリック/ポップ/ジッタ)が無いことを人間の耳で確認**(レビュー指摘#8: 適応リサンプリングは実装難度が高く、数値テストだけでは品質を保証できない。レート変化はスムージングし、急変させない) |
| D6 | 書き出しへの楽曲mux: 元の音声ファイルをオフセット付きでffmpegに渡し映像とmux。ミキシングバウンス不要のため、コーデック互換ならストリームコピー(無劣化)を優先 | D4 | 書き出したmp4の音声が元素材とサンプル一致し、映像とズレない(コンセプト: MVの最終書き出しに必須) |
| D1g | **完了**(#94): **第二監査フォローアップ①(S1/S2、[決定2026-07-12](../reviews/2026-07-12-code-audit-2nd-d1.md))**: TimeMapを「クリップローカル時刻→ソース時刻」へ変更し`timeline_start`を**削除**。固定式(このまま焼く): `clip_local_time = timeline_time - clip.start` / `source_time = time_map.map(clip_local_time)`。キーフレーム評価にはTimeMapを通さない(spec-holes §1と同一領域)。**正準化**: speed既約化+`Fps`既約化(M2E-16と同型の構築時不変条件。速度フィールド非公開でEq/Hashが意味同値と一致)、`is_identity()`は意味比較へ。**尺の正本宣言(2026-07-12ユーザー承認)**: v1は`Clip.duration`が表示区間の正本、source使用終端は **`source_end = time_map.map(clip.duration)`** で導出(=`time_map.map((clip.start + clip.duration) - clip.start)`の簡約。TimeMapはクリップローカル契約なので`start+duration`のタイムライン値を直接通さない。(b)ノット列導出はv1の定数速度TimeMapに終端ノットが無いため不可能 — §1c位置ノット列導入時に**再判定**(自動移行しない)。**OverrunMode席の確保(§7b消化、契約をここで固定)**: `enum OverrunMode { Freeze, Black, Loop }`をTimeMapのフィールドとして予約、永続キー`overrun_mode`、serde default=**Freeze**(§1bユーザー決定の候補3つをそのまま列挙)。**両端鏡像規約(B②消化)**: 解決済みsource時刻が素材available範囲(メディア総尺)の始端より前・終端以降の**両側に同一モードを適用**(Freeze=近い側の端フレームへクランプ / Black=非描画 / Loop=available範囲でwrap)。**責任境界**: TimeMapは素材尺を知らない純写像であり、モードを**保持するだけで適用しない** — 適用は素材尺を知るD3。v1のD3実装は**Freezeのみ**で、Black/Loop指定は型付き「未実装モード」エラー(黙ってFreezeへ縮退しない) | D1a | `timeline_start`がスキーマ・serdeに無い。OverrunModeのserde roundtrip+default=Freezeテスト。**D3以前のモード無視ガード**: ①`is_identity()`は正準アフィン写像が恒等**かつ`overrun_mode == Freeze`**の場合のみtrue ②D3完成前の全評価入口(現行の`try_map`直呼び経路)でBlack/Loopを型付き`UnsupportedOverrunMode`エラーとして拒否 ③Black/LoopがFreeze相当として描画されない回帰テスト(「保存はできる/未実装は黙って縮退しない」をD1g直後から成立させる。端フレームの厳密PTS・空available範囲の扱いはD3発注時に固定)。**旧`timeline_start`形式のJSONはD1gでは型付きエラーで拒否**し、新旧変換は**D1e(migration枠)の担当**(実行順はD1g→D1e。D1eの旧版コーパスへ`timeline_start`世代を追加)。`2/2`速度・`60/2`fpsが構築時に既約化される+意味的`is_identity`テスト。クリップ移動の不変条件テスト: `resolve(moved_clip, timeline_time + delta) == resolve(original_clip, timeline_time)`。既存の縮退挙動テストを仕様違反拒否テストへ反転。`cargo test --workspace`全緑 |
| D1h | **完了**(#98): **第二監査フォローアップ②(S3/S4/S9、同決定)**: 各DocParam受け口の**期待型表**(例: position=Vec2、opacity=F64、color=Color)をスキーマ側正本として宣言し、validateが Const / Keyframesの**全キー+同一variant** / Dataの**fallback型** / Vec2Axes内部 / **空トラック**=スキーマ拒否(期待型default注入は採らない)まで検査。**検査責任の二段化**: DataTrack本体はDocument外のキャッシュであり`DataTrackId`から実出力型を判定できないため、**D1hはDocument内で完結する型整合まで**(参照側が期待型を保持)。**実際のDataTrack出力型と期待型の照合はD3(実行時結線)の完了条件**とする。**AssetId結線**: 永続層のasset参照は`AssetId`型(`DocAssetRef`)へ — 存在検査・ダングリング拒否・cross-document copy時の再写像規約。評価層へは解決済み値で渡す。**非有限値・値域**: NaN/Infは全パラメータで拒否、範囲はパラメータごとにクランプ/拒否を宣言(B⑥消化) | D1g | `Transform.position = Color`等の型不一致・空トラック・`Value::AssetRef`ダングリングが**validateで型付きエラー**になる。評価側縮退(空→F64(0)、型不一致→前値/0.0)へ到達する入力がvalidateを通らないことをテストで固定(縮退テストの反転) |
| D1i-1 | **完了**(#99): **第二監査フォローアップ③a(S6、同決定)— VectorRecipe構造移動**: `Clip.path_ops`を削除し、Vector系ソースを`VectorRecipe { content: StandardShape\|SvgAsset\|TextPath\|Group, modifiers: Vec<PathOp> }`へ。固定規約: ①modifiersはrootの全パス集合に作用 ②index 0から順に適用 ③Trimのparallel/sequentialはPathOp自身のフィールド ④raster sourceには構造上存在しない(Lottie型入れ子スコープ=`Vec<VectorItem>`混在は**v2の別設計**であり本タスクで発明しない)。**永続JSON**: Asset/Vectorは未知フィールド拒否(Assetへ`recipe`/`modifiers`混入でserdeエラー。Pluginの`extra` flattenは維持)。**asset種別**: `SvgAsset`=`image/svg+xml`、`TextPath.font_asset`=`font/ttf`\|`font/otf`\|`font/woff`\|`font/woff2`(ここで正本化) | D1h | raster+modifiersが**型レベルで構文不能**かつ永続JSONでも拒否。VectorRecipeのJSON roundtrip。旧`path_ops`形式の扱いはD1g同様(D1i-1では拒否・変換はD1e)。動画AssetをSvgAsset/TextPathへ入れてもvalidateが`WrongAssetType` |
| D1i-2 | **第二監査フォローアップ③b(S5)— PathOp意味論表と実装**: 仕様の「PathOp意味論表」は**【決定】済み(2026-07-13)**。validate+幾何実装+variantゴールデンを表に一致させる。追加的席: `point_type` / `line_join`+`miter_limit` / `twist.center` / `repeater.transform`+opacity。Wiggle=`pcg32_value_noise`+u64 seed。開路Offsetは型付きunsupported | D1i-1 | 意味論表が仕様に**【決定】**として存在する(済)。validateが表の拒否項目で型付きエラー。variantごとの保護された意味論ゴールデン。`Repeater`に`transform`があり旧JSONはdefault恒等で読める |
| D1i-3 | **第二監査フォローアップ③c(S16)— 非PathOp演算の保護ゴールデン**: BlendMode / LookAt・Follow / Bezier solver / Transform合成の意味論ゴールデン | D3 | 各演算のゴールデンが存在し、意味を固定している |
| D1i-4 | **第二監査フォローアップ③d(S16執行)— ゴールデン更新禁止のCI検査**: ゴールデンを**2分類**に分ける — (a)**意味論ゴールデン**(S16。既存variantの意味を永久固定する審判。**更新禁止・例外なし** — regenerateマーカーでも迂回不可。変更したければ新variant+新ゴールデン) / (b)**暫定ゴールデン**(C-1系のsRGBブレンド依存等。regenerateマーカーで更新可)。CIは(a)への変更を機械的に拒否する | D1i-2 | 意味論ゴールデンを書き換えるPRが**マーカー有無に関わらず**CIで落ちる施行テスト。暫定ゴールデンはマーカー付きでのみ更新できる分類テスト |

並列レーン(2026-07-14改訂): **D1a**→(D1b/D1f と D8)→D1c→D1d。第二監査は**D1g→D1h→D1i-1→D1i-2**。D1eはD1i-1後、**D1jはD1e+D1i-2後、D1kはD1j後、D3はD1k後**。D2はD1i-1後、D3→D7。D1i-3はD3後、D1i-4はD1i-2後。D4独立、D5合流、D6はD4後。**予防**: schema/runtime/接続の3PRを混ぜる、またはD1k前にD3へcamera defaultを焼く発注はGR-PV-4違反。

**2026-07-12 第二監査によるフォローアップ**: D1a/D1b/D1cは「完了」だが、[第二実コード監査](../reviews/2026-07-12-code-audit-2nd-d1.md)がスキーマ・検証の穴S1〜S18を確認した(D1a: S1/S2/S5/S6/S16、D1b: S3/S4/S9、D1c: S10/S14)。**S1/S6/S16は2026-07-12ユーザー決定(3点とも案1、補正4点込み — 同文書決定節)**。スキーマ側フォローアップは**D1g/D1h/D1i-1〜D1i-4**(上表)として発注済み。D1c分(S10/S14)と未着手のD1d〜D1f・D2・D3は同文書の該当行を各発注書へ織り込むこと。

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

D2/D3着手前の決定パック(#103)・PathOp(#100)・残小項目は[採択文書](../reviews/2026-07-13-decision-pack-adoption.md)で**【決定】済み**。追加の広範調査は不要。例外は実装中の既存仕様同士の矛盾発見時のみ。
