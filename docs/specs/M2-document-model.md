# M2: ドキュメントモデルとタイムライン

ステータス: **ドラフト**(骨格は[凍結ゲート](../reviews/2026-07-10-freeze-gate-declaration.md)済み。[M2入場条件](../reviews/2026-07-11-M2-entry-gate.md)全緑は Document に触る実装の**入場**であり、**本仕様の確定ゲートを代替しない**)。**2026-07-12**: [発注ゲート停止](../reviews/2026-07-12-M2-order-gate-halt.md) — 細目意味論・Command/journal/audio が閉じるまで **D1d/D2/D3追補/D4/D5/D6/M3 の新規エージェント発注を禁止**。D1a/D1b/D3 の「完了」表示は撤回(再開)。
着手条件: Documentスキーマに触るコードは入場条件達成済みでも、**未決事項(下記発注ブロッカー)に依存するタスクには着手しない**(ルール7)。ゲート再「確定」後に解禁。

## 目的(退治する落とし穴)

C-1(Undo後付け)、C-2(スキーマ進化)、B-1(音声・時間表現)、F-1(座標系)、F-2(所有権)、F-4(時間写像)。

## 方針

- プロジェクト状態は単一のserde可能な純データ構造(ドキュメントモデル)。エンジン(motolii-render)はこれを読むだけ
- ドキュメントモデルはAsset実体とプロジェクトBPM(手動入力。ビート検出はやらない)を持つ。**Assetは一般アセット(opaqueペイロード+type文字列+内容ハッシュ)として定義し、動画・SVGはその特殊ケース**(F-10、[plugin-resources.md](../plugin-resources.md)§3 D2/D3。将来のImporterプラグイン=点群等の口。`ValueType::AssetRef`の予約と対)。この定義は凍結ゲートのF-10確定に依存する — 確定前にD1を並列レーンへ出さない
- **音声はプロジェクト直下の単一フィールド(楽曲1本+開始オフセット+マスターゲイン)**。音声トラック・ミキサー・クリップ音声は存在しない(コンセプト決定: MVでは音声=完成済み楽曲。動画クリップ内蔵音声は常にミュート)
- 全編集操作はコマンド(適用/逆適用可能な差分)として実装 → Undo/Redoはコマンド履歴で自動的に得られる
- ファイル形式はバージョンフィールド付きJSON + マイグレーション関数の枠組み
- **空間値は正準座標系で持つ(F-1)**: 単位なし・原点=コンポ中央・Y-up・高さ基準正規化(高さ=1.0、幅=アスペクト比)。位置・アンカー・サイズ・エフェクトの空間パラメータにpx値をスキーマに入れない。グループは変形コンテナ(変形+合成の両方)であると同時に、**クリップと同じ項目エンベロープ(順序付きエフェクトスタック・クリッピングマスク・ブレンド/不透明度)を持つ**(concept 2026-07-10「プリコンポは作らない」の帰結。グループのエフェクトは子を合成した1枚に適用=AM式、per-child適用はしない)。レイヤーの親参照(ペアレンティング)フィールドをスキーマに予約(v1のUIはグループ変形のみでも、スキーマは初日から)
- **シェイプ間リンク(レイヤー参照付きParamSource)をスキーマに含める(concept 2026-07-10)**: LookAt/Follow/ParentRef 等は**別レイヤーの変形を読む型付き参照**。AEエクスプレッションの代替。`LayerId`参照と依存グラフ(無効化伝播の入力)を D1/D3 で予約。評価は F-3 の順序で参照先を先に評価
- **シェイプ系レイヤーは順序付きパス演算子スタックを持つ(concept 2026-07-10、F-13)**: パンク・膨張/ジグザグ/パスのオフセット/角丸/トリムパス/ツイスト/パスのウィグル(+リピーター=F-7)を、標準シェイプ・SVG・テキスト由来パスに共通適用する`Vec<PathOp>`相当の席をD1で予約。全演算子は`(パス, パラメータ, t)→パス`の純関数で、パラメータは通常のParamSource(キーフレーム/リンク)駆動。シリアライズはLottie形式(`pb`/`zz`/`op`/`rd`/`tm`/`tw`/`rp`)を前例にする([references.md](../references.md))。v1はファーストパーティの閉集合(プラグイン契約には出さない=`PathOp`種別化はv2判断、解凍手続き対象)
- **クリップは時間写像(TimeMap)を持つ(F-4)**: `clip_local_time → source_time`の単調写像(用語は下記)。v1実装は恒等+定数速度(オフセット+speed)のみだが、**motolii-renderのソース時刻解決は必ずTimeMapを通す**。速度ランプ・逆再生はスキーマ互換のまま将来拡張。キーフレームはTimeMapを通さない(下記意味論節)
- **所有権は単一writer+不変スナップショット(F-2)**: ドキュメントを書き換えるのはコマンド適用の編集スレッドただ1箇所。レンダ・書き出し・解析・プロキシ生成は`Arc<Document>`スナップショットを受け取る読み手。バックグラウンド成果はメッセージでwriterへ返し、writerがコマンドとして適用する
- **スキーマの素性はOTIO互換寄りに保つ(F-5)**: トラック/クリップ/ソース区間を有理数時刻で表現し、「OTIOに写像できない構造を発明しない」。OTIO書き出し自体はv2候補

## スキーマ境界の宣言(M2E-11 / 監査SC-1)

D1着手前に固定する。エージェントが「もっともらしい継承」で恒久負債を作らないための境界宣言。

1. **D1は`ProjectV1`を継承も移行もしない**: `ProjectV1`(`motolii-cli`)はM1 CLI専用の使い捨て。Documentの`version`採番は独立。`export-project`はD3完了時にDocument読み込みへ置換する(ProjectV1増築禁止)
2. **Document ≠ ExportJob**: Documentはレシピのみ。出力パス・書き出し範囲・エンコード設定(qp等)は別構造`ExportJob`(仮称)。Asset参照は初日から多重キー(実装ガード10)
3. **クリップのin/out/durationは`RationalTime`**: フレーム添字(`start_frame`/`frame_count`形式)をスキーマに入れない。`ProjectV1`のフレーム添字は入力素材fps基準の暫定であり、Documentへ持ち込まない
4. **bpmは有理数**: `f64` bpm禁止。有理数(またはミリbpm整数)で持ち、拍時刻(`60/bpm`秒)が`RationalTime`に畳めることをD1完了条件に含める
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

## Document version 2(D1e)

| フィールド | 型 | 既定 | 備考 |
|---|---|---|---|
| `color_interpretation` | enum(`straight_srgb` のみ、v2時点) | `straight_srgb` | M2E-13 層1(straight-alpha・非線形sRGB・0–1)の永続明示。トップレベル追加 |

- **版上げ**: v1→v2 マイグレーションで本フィールドを明示し、`version=2` かつ **`min_reader_version = max(既存, 2)`** を保証する(新規は`Document::new_v2()`)。旧リーダーが開いて再保存で意味を落とす経路を作らない
- **prelude `time_map`**: クリップがあれば全クリップ(ネスト Group 配下含む)へ写す。クリップが無く非恒等なら破棄し、`MigrationReport.warnings` に `prelude_time_map_dropped_no_clips` を記録する(undocumented `extra` キーは作らない)
- **バックアップ**: `*.motolii-pre-migrate.bak` が既にあれば上書きせず fail closed

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
3. **`TimeMap.speed`**: M2では`speed_num > 0`かつ`speed_den > 0`のみ(ゼロ・負の速度は明示拒否。逆再生は将来拡張)。`try_map`は未検証入力でもpanicせず`TimeMapError`/`RationalTimeError`を返す。**`overrun: OverrunMode`**は旧JSON欠落時`Freeze`(serde default)。採取は`resolve_source`(メディア半開区間付き)

## duration/区間規約(M2E-17 / 監査TM-3)

時刻区間の開閉を1流儀に固定する。M4区間キャッシュと音声終端の前提。

1. **`duration`は総尺**: 最終フレームのPTSではない。`MediaInfo.duration`も総尺(fpsグリッドスナップ済み)
2. **区間は半開`[start, start+duration)`**: 終端ちょうど(`t == start+duration`)は範囲外。等間隔サンプル添字は`0..n`で、**`n = ceil(duration × rate)`**(有理数で厳密。`i/rate < duration` を満たす非負整数iの個数)。`duration`がrateグリッドに整列しているときは整数`n = duration × rate`に一致する。**`MediaInfo.duration`はfpsグリッドへスナップ済み**のため、`export_frame_count`の`floor(duration × fps)`は整列前提で正しく、非整列の一般式(ceil)はParamDriver側に適用する
3. **互換性**: この規約により`ProjectV1`の書き出しフレーム数が1減る(例: 90フレーム素材で旧91→90)。`ProjectV1`は使い捨て(M2E-11①)でありユーザーデータの互換対象外。旧`from_frame(89)+1`系テストの更新は規約変更に伴う正当な期待値更新(M2E-2の「テスト更新」手続き)

## 音声トランスポート設計(音ズレ・途切れの構造的排除)

コンセプト決定(2026-07-06/07)を受けた根本設計。「音がズレる/途切れる/重い時に映像だけ遅れる」を後付けで直すのではなく、クロックの所有構造で不可能にする。

> **2026-07-12注(確定前・内部矛盾)**: 下記②「デバイスクロック主」と④「アンダーラン時は無音を出しつつクロックを進めない」は両立しない(デバイスは無音再生中も進む)。また「供給済みサンプル数」は聞こえている位置ではない([cpal `OutputStreamTimestamp`](https://docs.rs/cpal/latest/cpal/struct.OutputStreamTimestamp.html)は callback 時刻と DAC 到達予測を分離)。③の「レンダ進捗」もフレーム並列では単調一値にならない。**本節は方向性の骨格であり、発注ブロッカー(audio clock / D5 レンダ進捗)を状態機械で書き直すまで D4/D5 に着手しない** — [発注ゲート停止](../reviews/2026-07-12-M2-order-gate-halt.md)。

1. **再生ヘッド(Transport)の所有者は常にちょうど1つ**。音声も映像も追従者(follower)であり、勝手に自分の時計で進むコンポーネントを作らない。ズレとは「時計が2つある」ことの症状であり、時計を1つにすれば定義上発生しない
2. **通常再生モード**(要状態機械化): 音声デバイスクロックが主、という方向。再生位置の定義(供給済みサンプル vs DAC 到達予測)は未確定。映像は現在位置に最も近いフレームを表示し、間に合わなければフレームドロップ、という方向(「最も近い」は最大半フレームの映像先行を許す — 採否未確定)
3. **バリスピードモード**(要状態機械化): レンダが実時間に届かない時にクロック所有者を交代し、音声が適応リサンプリングで追従する、という方向。「レンダ進捗」の定義(例: 連続提示可能最大 PTS)・ヒステリシス・速度窓・変化率上限・resampler 位相連続は未確定
4. **音声コールバックは絶対にブロックしない**: コールバックはリングバッファから読むだけ。デコード・リサンプリングは専用プロデューサスレッド。アンダーラン時は無音を出力しカウンタに記録 — **「クロックを進めない」の所有権との関係は未確定**(上記注)
5. **楽曲はインポート時にPCM全展開してRAM保持**: 音声は楽曲1本のみなので、これが音声データの全て(5分ステレオ48kHz f32 ≈ 110MBで許容範囲)。ミキシングが存在しないため「再生位置→サンプル」は単一バッファの添字計算に還元され、シーク・スクラブ・逆再生・バリスピードが同一コードパスになる
6. **レイテンシ補償**: 表示するフレームは「いま聞こえている音」に対応させる(出力レイテンシ分を引いた位置)

## タスク分割(1PR粒度。旧粗案D1をD1a〜D1fへ再分割)

旧「D1」はスキーマ本体+永続化+ジャーナル+マイグレーションが1行に詰まっていたため、仕様ルール1(1タスク=1PR)に合わせて分割する。**D1完了= D1a〜D1f全緑**。**2026-07-12**: D1a/D1b/D3 の「完了」は撤回。下記 **着手禁止** タスクは [発注ゲート停止](../reviews/2026-07-12-M2-order-gate-halt.md) 解除まで発注しない。ゲート再確定後、D1d/D2/D4/D5 は複数PRへ再分割してから着手する。

| ID | 内容 | 依存 | 完了条件(概要) |
|---|---|---|---|
| D1a | **再開**(完了撤回): **スキーマ本体+serde**(I/Oなし)。席の大半はマージ済みだが、棚卸し未決(同一トラック重なり・グループ時間領域・拍0/拍子・Driver/Link評価時刻・Asset指紋の version 付き統一型・負スケール/レイヤーフラグ等 B群)が完了条件漏れ。現状 Asset 指紋は未検証 `String` 群([asset.rs](../../crates/motolii-doc/src/asset.rs)) | 凍結ゲート, **[M2入場条件](../reviews/2026-07-11-M2-entry-gate.md) 全緑**, **発注ブロッカーのスキーマ細目が仕様改訂で潰れていること** | 既存 roundtrip に加え、未決表の D1a 割当分が型/validate/1文宣言で閉じ、Asset 指紋が version 付き統一型(または明示的不採用の仕様改訂)になっていること |
| D1b | **再開**(完了撤回): 保存前不変条件検証(ガード1)。参照整合・ID一意・区間正当性の口は存在するが、「席はあるが一意でない意味論」(重なり規則・同時刻キー等)を覆えていない | D1a(細目閉じた後) | 壊れたDocumentが`validate`で型付きエラー。**意味論が仕様で一意になった項目**について正常系/拒否系が揃う。失敗時は既存スナップショットを壊さない |
| D1c | **スナップショット原子性に限定して達成**(注釈付き): アトミック保存+読込(ガード2)+`min_reader_version`超過拒否。**ジャーナル復元は完了条件に含めない**(本体はD1d)。並行保存テストは last-write-wins を許す暫定([d1c_persist.rs](../../crates/motolii-doc/tests/d1c_persist.rs)) — **プロセス単一writer(プロジェクトロック)は未達**で D1d ブロッカー | D1a | temp→fsync→rename→dir fsync。各段 abort 注入で**旧スナップショットファイル**が残る/読める。ジャーナル言及なし |
| D1d | **着手禁止**: ジャーナル(ガード3/4/6)。checksum だけでは WAL 相当にならない — commit record・単一writer・reader end mark・checkpoint 順序が一体([SQLite WAL](https://www.sqlite.org/wal.html))。耐久性契約・プロジェクトロックも発注ブロッカー | **D1c** + **D2(Command codec 最小)** + **D1e(版/マイグレーション接続点)** + journal ブロッカー確定 | 壊れ方カタログの単体/注入テスト緑。SIGKILL/別プロセス復元を含む耐久契約の自動判定。二重起動は lock で拒否または一方 read-only |
| D1e | マイグレーション枠+量的不変条件+旧版コーパス(ガード8)。前方互換の版上げ経路 | D1a(細目閉じた後) | in-place禁止・バックアップ・クリップ/トラック/キー数一致・goldenコーパス回帰 |
| D1f | 未知プラグインID: 開く=警告+パススルー・再保存で未知部分喪失なし(F-9、ガード9の「開く」側)。書き出し厳格化はD6接続 | D1a | 未知`plugin_id`を含むJSONがロード成功+警告+roundtrip保持 |
| D8 | 所有権モデルの骨格(F-2): 編集スレッド=単一writer、`Arc<Document>`スナップショット配布、バックグラウンド成果のメッセージ適用経路。**現状 `snapshot` は Document deep clone**([lib.rs](../../crates/motolii-doc/src/lib.rs)) | D1a | 並行テスト(古いスナップショット完走)+型レベル単一writer。**完了前に**「10万キー・毎秒N編集」負荷試験で deep clone コストを定量し、必要なら構造共有へ分割PR |
| D7 | クリッピングマスク合成 | D3(再確定後) | 各モードのゴールデンイメージテスト |
| D2 | **着手禁止**: コマンドシステム(apply/revert)+ Undo/Redo。完了条件は `apply→revert` だけでは不十分 | D1a(細目+stable ID アドレス確定) + Command ブロッカー確定 | 最低セットを自動判定: (1)apply失敗で部分変更ゼロ (2)compound 原子性 (3)undo→redo (4)undo後新規編集でredo枝破棄 (5)delete→undoでIDと内部リンク復元 (6)copy/pasteの内部参照再写像 (7)再起動後Undo (8)未知フィールド対象のUndo (9)coalescing後も journal replay 一致 (10)stale revision のバックグラウンド結果拒否。メモリUndo深さと永続Undo深さの分離(Ardour先例)を仕様に書く |
| D3-dep | **着手禁止**(D3先行): **統一依存グラフ** — 親変形 / LookAt·Follow / clipping mask / group包含 / effect param 参照 / 将来のキャッシュ無効化を同一グラフで辺種別・循環拒否・フォールバック | D1a(細目)+依存グラフ契約の仕様確定 | validate と評価順が同一グラフ契約を共有する単体テスト。片方では循環なし・全体では循環、が構造的に起きない |
| D3 | **部分実装・再開**(完了撤回): Document→レンダグラフ変換(F-3順序)。M1 E2E接続や評価順ゴールデンの一部は存在するが、統一依存グラフ・ParentRef配線は未達 | **D3-dep** 後 | M1 E2EがDocument経由で通る。マスク付きグループ/グループエフェクトの評価順ゴールデン。ParentRef継承がグラフ契約どおり |
| D4 | **着手禁止**: motolii-audio(PCM展開+cpal+リングバッファ) | 凍結ゲート + **audio clock ブロッカー確定** | 任意位置読み出し。アンダーラン時の所有権・再同期が仕様どおり自動判定される |
| D5 | **着手禁止**: Transport(クロックオーナー交代) | D3, D4 + **D5 レンダ進捗/遷移ブロッカー確定** | A/V drift を ms または sample で定量(ゼロ/±1 sample/±1 frame を仕様で選ぶ)。最大サンプル不連続・遷移区間スペクトル差・出力サンプル数と Transport 時刻の誤差上限・速度変化率上限・underrun/xrun カウンタを自動判定。**可聴確認はレビュー項目**(完了条件にしない) |
| D6 | **着手禁止**: 書き出しへの楽曲mux | **D4** + **D1a(Soundtrack.gain)** + **ExportJob** | 分岐: gain=1かつ互換codec→stream copy / gain≠1→PCM処理+encode / offsetがcodec境界非整列→trim\|reencode\|先頭padding。元素材との「サンプル一致」意味は分岐ごとに定義 |

並列レーン(ゲート再確定後の目標形。**現時点は発注停止**): D1a細目閉じ → D1b/D1e/D1f。Command ブロッカー確定 → D2(複数PR) → journal ブロッカー確定 → D1d(複数PR)。D3-dep → D3追補 → D7。audio ブロッカー確定 → D4(複数PR) → D5(複数PR) → D6。D8は D1a 後だが負荷試験完了前に「完了」としない。**「D4は独立」「D1a直後にD2/D3」は撤回**。

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

## 時刻の用語(固定)

| 用語 | 意味 |
|---|---|
| `timeline_time` | コンポ上の絶対時刻(半開総尺内) |
| `clip_local_time` | クリップ起点からの相対時刻=`timeline_time - clip.start`(キーフレームはこの領域) |
| `source_time` | メディア/ソース内時刻。**TimeMap の出力のみ**がこれを指す |

「クリップローカル」と「ソース時刻」を混用しない。TimeMapは`clip_local_time → source_time`の写像。

## キーフレーム・TimeMap・色/変形の意味論(2026-07-12 昇格)

出典: [D1調査メモ](../reviews/2026-07-12-d1-spec-holes-prior-art.md)のユーザー決定＋同日改訂。

1. **キーフレーム時刻**は`clip_local_time`。TimeMapはソース解決専用で、キーフレーム評価には通さない。速度変更でキーを伸ばすのは D2 コマンドが時刻を書き換える編集操作(評価意味論は不変)
2. **TimeMapとクリップ尺(2026-07-12改訂)**: `Clip.duration`はタイムライン上の**独立した表示区間**として維持する。`TimeMap { source_start, timeline_start, speed, overrun }`はその区間内でのソース採取だけを支配する。消費ソース範囲は`clip.duration × speed`から導出する(尺を素材から逆導出しない — 末尾超過は`OverrunMode`の管轄)
3. **`OverrunMode`**: `Freeze | Transparent | Loop`。旧JSON欠落時の既定は`Freeze`。`Black`は採らない(黒画素と非描画が曖昧なため。非描画は premultiplied 合成に合わせ`Transparent`)。Freezeはデコード側が最終フレームを保持する指示(`HoldLastFrame`)、Transparentは非描画、Loopはメディア尺での剰余
4. **色キーフレームの補間空間**: `Value::Color`の補間は**保存空間(非線形sRGB・straight-alpha)における成分ごと線形**(アルファも同様に線形)。linear/Oklab等の知覚補間を導入する場合は将来版の**追加的**オプトイン(補間空間フィールドの追加)とし、この既定の意味は変えない
5. **回転の表現**: `Transform2D.rotation`は**単一スカラー・単位はラジアン**。多回転は値そのもので表す(0→4π=2回転)。AE式「回転数+度」の複合表現は採らない。補間はスカラー線形で**最短経路ラップをしない**(350°→10°は逆回りに340°戻る — 多回転をそのまま表現できることの裏面)。**度はUI側の表示変換のみ**(M3。例:「720°(2回転)」の括弧表示)。スキーマ・ジャーナル・プラグイン境界に度の値を流さない — UIが度で保存する実装はマイグレーション不能の破壊であり禁止
6. **変形の適用順**: 子ローカル→親空間の写像は **`M = T(position) · R(rotation) · S(scale) · T(−anchor)`**(アンカーを原点へ→スケール→回転→位置。F-3の文言をスキーマ契約へ昇格)。親参照(`Transform2D.parent`)は親の`M`を左から合成する。**継承は変形のみ**(不透明度・エフェクト・ブレンドは継承しない)。正本実装は`motolii_core::Affine2::from_trs` / `mul`。**D3グラフのGroup再帰継承はこのAffine2で行う**(子へ`inherited.mul(local)`)。**`Transform2D.parent`(ParentRef)の解決は未配線** — 統一依存グラフ残件。Follow/LookAt対象位置は世界へ写したアンカー(契約上`position`)。`OverlayRect`は`RenderStep.transform`に世界アフィンを渡し完全適用する

## 恒久IDの範囲(2026-07-12決定)

独立して選択・並べ替え・Undo対象になる永続要素はIDを持つ。`LayerId` / `AssetId` / `TrackId`に加え、**`EffectInstanceId`と`KeyframeId`を追加**する。採番は再利用禁止。複製時は新規採番(サブツリー内リンクは新IDへ再写像 — D2)。

## CompositionとExportJobの境界(2026-07-12決定)

- **Composition**が所有する創作意味: `aspect` / `fps` / `duration` / **`background`(色)**。**PARはv1で1:1固定**(フィールドを持たない=常に正方画素)
- **ExportJob**が所有する: 出力解像度・書き出し範囲・codec/mux設定。Compositionのaspectや時間意味論を変更しない

## 未決事項

### 発注ブロッカー(M2再確定まで新規発注禁止)

「フォローアップ候補」ではない。これらが仕様改訂PRで閉じるまで **D1d/D2/D3-dep/D4/D5/D6/M3 を発注しない** — [発注ゲート停止](../reviews/2026-07-12-M2-order-gate-halt.md)。

#### 1. Command(永続形式・transaction・アドレス)

- 永続形式と `version`
- 単体 Command と複合 transaction の境界
- commit 済み判定
- Undo/Redo 操作自体をどう記録するか
- redo 分岐の破棄規則
- 旧 Command を新 Document へどうリプレイするか
- stable ID による編集対象アドレス
- coalescing 規則と journal replay 結果の一致
- コマンド粒度(プロパティ単位 vs 操作単位)と UI(M3)編集状態の責務分担

#### 2. Journal(耐久性・locking・checkpoint)

- `apply()` 成功時点で fsync 済みか / 数百msバッチならクラッシュ時許容損失は何msか
- ENOSPC / fsync 失敗時、メモリ上だけ編集を続けるか(続けるなら永続契約の破たんをどう見せるか)
- snapshot と journal のどちらを先に永続化するか
- checkpoint 中の kill からどう戻るか
- 中間破損と不正テールの区別
- **プロセス単一writer**: プロジェクトロック・stale lock 回収・外部更新検知・競合時 read-only([SQLite howtocorrupt](https://www.sqlite.org/howtocorrupt.html)の二重writer禁忌)
- WAL 相当に必要な一体要素: commit record / 単一writer / reader end mark / checkpoint 順序([SQLite WAL](https://www.sqlite.org/wal.html))。checksum だけでは不足

#### 3. Audio clock(アンダーラン・所有権・再同期)

- 「デバイスクロック主」と「アンダーラン時は無音出力しつつ論理クロックを進めない」の矛盾解消
- 再生位置の正本: 供給済みサンプル数 vs DAC 到達予測時刻(cpal `OutputStreamTimestamp`)
- アンダーラン時の所有権と復帰時の再同期
- 本ファイル「音声トランスポート設計」節は方向性のみ — 状態機械で書き直すまで D4/D5 禁止

#### 4. D5 レンダ進捗とモード遷移

- 「レンダ進捗」はフレーム並列では単調一値ではない。クロック候補の定義(例: 連続提示可能最大 PTS)と、先読み完了を数えるか否か
- 「最も近いフレーム」採否(最大半フレーム映像先行の許容)
- モード遷移: 閾値・ヒステリシス・速度推定窓・変化率上限・resampler 位相連続
- seek / pause / loop / device loss 時の遷移
- A/V drift の許容を ms または sample で定量(「10分でドリフトなし」はゼロか±1 sampleか±1 frameかを選ぶ)

#### 5. Asset 指紋(D1a 完了条件漏れ)

- 調査メモ提案の **version 付き統一型** vs 現状の未検証 `String` 群。D1a「完了」では覆えない — 型を入れるか、明示的不採用の仕様改訂が必要

#### 6. D8 スナップショット性能

- `DocumentWriter::snapshot` = `Arc::new(doc.clone())` の deep clone コスト。D8 完了前に「10万キー・毎秒 N 編集」負荷試験を必須(M3 UI フリーズとして初めて発覚させない)

#### 7. D1a/D1b 意味論ギャップ(再開チケットの完了条件)

判定基準は「ユーザーデータへ不可逆に焼くかどうか」。詳細は[D1調査メモ](../reviews/2026-07-12-d1-spec-holes-prior-art.md)。代表:

- 同一トラック内の重なり規則 / グループの時間領域(TimeMap無しの宣言) / 拍0・拍子
- Driver/Link の評価時刻領域 / 同時刻キー / レイヤーフラグ(非表示は書き出しに含まれないか) / 負スケール
- 複製時の ID 再写像(D2 と接続) / 統一依存グラフ(D3-dep)

既に実装・追認済みでブロッカーから外したもの: TimeMap+OverrunMode、EffectInstanceId/KeyframeId 台帳、Composition.background / ExportJob 拡張席、BlendMode 未知 variant(描画 Normal 縮退)。**実装済みでも「D1a/D1b 全体完了」には戻さない** — 上記残りが閉じるまで再開のまま。

### 低優先(M3で決めてよい)

- 波形表示用のピークデータ生成の持ち場(海苔波形前提・ブロッカー性なし — D1調査メモ残件)
