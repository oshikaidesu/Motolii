# M2入場条件(2026-07-11)

ステータス: **未達**(全緑でM2-D1以降の並列発注を解禁する)

## これは何か

凍結ゲート(2026-07-10宣言)が「**何を凍結してから並列化するか**」だったのに対し、これは「**何を塞いでからM2に入るか**」のエントリーゲート。M2が怖い理由は3つの重なりに名前が付く — ①**恒久性**(Documentスキーマとジャーナルはユーザーデータとして永続化され、以後の間違いは全部マイグレーション負債になる) ②**並列化の初陣**(凍結ゲート後、複数エージェントが同時にコードを積む最初のフェーズ) ③**検証の弱さ**(M2のバグはピクセルに出ず、数週間後の破損プロジェクトとして出る — その検出基盤が未整備)。

**本ゲートは新規の発明を含まない。** 全項目が既存文書の所見の翻訳である: [実コード監査](2026-07-11-code-audit-pre-m2.md)のチケット(PB/EN/SC/CQ/TM)、[M2仕様の実装ガード](../specs/M2-document-model.md)、[pitfalls H群](../pitfalls-and-roadmap.md)。出典列で逆引きできる。

## 記入規則(R1追補の教訓)

- 完了したらチェックを入れ、**項目にコミットSHA(またはPR番号)とテスト名を併記する**(証跡なしの[x]を作らない — [R1追補](2026-07-09-R1-export-review.md)記録上の問題2)
- 各項目は1タスク=1PR粒度。着手前に対象所見のfile:lineを**最新mainで再確認**する(監査はf020ec8基準。LLM監査の出力は採用前に現物確認 — 監査レポート冒頭の検証注記)
- 完了条件を満たせない事情が判明したら、黙って緩めず本ファイルを改訂してから進める(H-3: 未決を勝手なデフォルトで埋めない)

## A. 審判を起こす(テスト施行層 — 並列化の前提)

並列エージェントの安全モデルは「cargo test全緑=正」だが、現状その審判自体に穴がある。**塞ぐ前に並列化すると、穴を通ったコードが山積みになってから気づく。**

| ID | 内容 | 出典 | 完了条件(自動判定) | 規模 |
|---|---|---|---|---|
| [ ] M2E-1 | CIに`MOTOLII_REQUIRE_GPU=1`を設定し、`gpu_or_skip`はこれが立っていればスキップでなくpanic。GPU/ffmpeg無しで必ず赤になるカナリアテストを常設 | 監査PB-4(P-4③実測: GPU無し環境で141テスト全緑・1.2秒) | REQUIRE_GPU下でlavapipe/ffmpegが無いとカナリアが赤。通常環境で全緑 | 極小 |
| [ ] M2E-2 | ゴールデン参照(CPU参照実装・`expected_*`ヘルパー)をsrc同居からtestkit(または独立領域)へ分離し、CIに「テスト領域とsrc/**の同時変更はラベル必須」のdiffゲートを追加 | 監査EN-1(E-1: 参照PNGゼロ・参照が被試験クレートのsrc内)、EN-1と同時にE-6(`expected_rect_frame`のバイト同一重複+製品コード依存の循環)を解消 | ラベル無しで両領域に触るPRがCIで赤。重複ヘルパーが1箇所に集約 | 小 |
| [ ] M2E-3 | CIの決定性ピン留め: runner固定(ubuntu-24.04等)+`cargo test --locked`+mesa/ffmpegバージョンをCIログへ出力 | 監査EN-3(E-3: 全て無ピン。lavapipe更新1つでtolerance 0/1のゴールデンが全赤→閾値bump圧力が最大化) | ci.ymlに3点が入っている | 極小 |
| [ ] M2E-4 | toleranceの定数化(`testkit::tol`)+呼び出し箇所の生数値リテラルをdenyする走査テスト+mean上限のassert追加 | 監査EN-2(E-2: 「閾値を1上げる」1文字diffはテスト改変に見えない=ルール6の死角) | リテラルtoleranceを書いたテストが走査で赤。既存呼び出しが定数経由 | 小 |
| [ ] M2E-5 | 「motolii-doc外での`&mut Document`」をdenyする走査テスト(conformance.rsの基盤流用) | 監査EN-4(E-4: `Document`はpubフィールド+Clone+pubコンストラクタで、単一writerは現状すり抜け可能) | 負例(テスト内のダミー違反)が赤になることを含めて緑 | 小 |
| [ ] M2E-6 | proptestをworkspace依存に追加し、模範例1本(RationalTime roundtrip等)をmotolii-coreに置く | 監査EN-5(E-5: M2実装ガード12はプロパティテスト前提だが依存がゼロ件。各エージェントが独断選定すると分裂=H-3) | 模範例がCIで緑。workspace.dependenciesに追加済み | 極小 |

## B. 乗算する穴を塞ぐ(プラグイン境界 — M2でプラグイン量産が始まる前に)

プラグイン境界のエラーは費用がプラグイン数で乗算される(監査Part 1)。**参照プラグイン数個の今が唯一の安値点。**

| ID | 内容 | 出典 | 完了条件(自動判定) | 規模 |
|---|---|---|---|---|
| [ ] M2E-7 | `RenderCtx`導入(Filter/Composite traitの裸引数を`#[non_exhaustive]`構造体へ)。Quality/将来の予約(InstanceIndex/CompLookbehind/TemporalFootprint)の口を確保。**解凍手続き(3点セット)対象** | 監査PB-1(P-1: 引数追加=全プラグイン破壊。render_graph_cachedがQualityを解像度に畳み込みDraft/Final判別不能) | 全参照プラグインがRenderCtx経由で既存ゴールデン不変。解凍手続き文書化 | 中 |
| [ ] M2E-8 | 型付きparamアクセサ(`require_f64`等+`PluginError::Param`)+`NodeDesc::resolve_params`一元化(未知ID→Err/欠落→default/型不一致→Err)+ロード時のvalue_type検証 | 監査PB-2/PB-3(P-2: `f64_or`のサイレントデフォルト=「もっともらしく間違う絵」。P-3: 解決ロジックがproject.rsに手書き) | 型不一致JSONがロード時に構造化エラー。`f64_or`が参照実装から消えている | 小〜中 |
| [ ] M2E-9 | `PluginRegistry::iter(kind)`+`assert_registry_pure`(登録済み全プラグインへvalidate+purity一括適用)+LayerSource/Composite用purityヘルパー | 監査PB-3(P-4①②: purityが手書き列挙のopt-in。登録=検査対象への反転) | レジストリに登録するだけで検査対象になることをテストで確認(未検査プラグインを作れない) | 小 |
| [ ] M2E-10 | new-pluginスキャフォールドにpurityスタブ+ゴールデンスタブ+ParamDef例を同梱 | 監査PB-5(P-5: LLMは型紙に無いものは書かない。purity普及率≒スキャフォールド同梱率) | 生成物にテスト3種が含まれ、scaffold検証テストがそれを確認 | 小 |

## C. D1が継承する罠を断つ(スキーマ/型の宣言と骨格 — D1発注の前提)

大半が「仕様への1文」または数行。**放置した場合だけ恒久マイグレーション負債になる**種類(監査2-C総括)。

| ID | 内容 | 出典 | 完了条件(自動判定) | 規模 |
|---|---|---|---|---|
| [ ] M2E-11 | M2仕様へ宣言5点を追記: ①D1はProjectV1を継承も移行もしない(version採番も独立) ②Document≠ExportJob(出力パス/エンコード設定は別構造) ③クリップin/out/durationは`RationalTime`(フレーム添字をスキーマに入れない) ④bpmは有理数(拍時刻がRationalTimeに畳める) ⑤ExportOverlayRequestはD3でDocument→render直結に置換(ジョブミラー温存禁止) | 監査SC-1(F-1: 両者が「v1」を名乗り関係未宣言 — エージェントはProjectV1増築を自然に選ぶ。F-5/F-6/F-11/F-10) | M2仕様の改訂がマージ済み | 仕様のみ |
| [ ] M2E-12 | Document骨格の予約4点: `#[serde(flatten)] extra`(unknown-keys保持)+`min_reader_version`+`DocumentWriter.revision: u64`+editへ「D2でapply(Command)に置換、呼び出し追加禁止」doc-comment | 監査SC-2(F-2: unknown-keys黙殺=前方互換ガード違反が初リリースから発生。F-3: min_reader_versionは旧リーダーが知らないと機能しない。F-8/F-9) | unknown-keys roundtripテスト緑 | 小 |
| [ ] M2E-13 | 【**要判断**】スキーマの`Color`の定義を確定する(推奨: **sRGB(非線形)・straight・0-1**=現実装の実態。逆=リニアを選ぶならレンダ側に変換を入れてから)。value.rs:9の「リニア」コメントを実態に整合させ、`precise_color`の配線(口のみ、実装は恒等)を通す | 監査CQ-1(F-4: コメント間で既に矛盾。キーフレーム済みカラー量産後の解釈変更は「マイグレーション不能な種類の破壊」。C-1: sRGBブレンドがゴールデンに焼き込み済み) | 決定が仕様に明文化され、コメントと整合。precise_colorがrender_descまで届く | 小 |
| [ ] M2E-14 | `CanonicalPoint`/`CanonicalSize`/`ViewportTransform`等をmotolii-coreへ移動(nodesはre-export) | 監査CQ-3(C-3: motolii-docはnodesに依存しない — 空間パラメータを持つ最初のM2エージェントが参照できる正準型が無く、独自表現を発明する) | 移動後全テスト緑。docクレートからCanonical型が参照可能 | 小 |
| [ ] M2E-15 | `LayerId` newtype予約+「表示名はIDと別フィールド・ID再利用禁止」の仕様宣言 | 監査SC-3(F-7: ID体系不在のままLookAt/Follow参照を足すと文字列/型付きの二重方式が恒久化) | 型がcoreまたはdocに存在 | 極小 |
| [ ] M2E-16 | 時刻型のserde不変条件: `RationalTime`/`Fps`を`try_from`正規化(den=0拒否・負/非既約は正規化)、`TimeMap` speedのgcd正規化+符号寄せ | 監査TM-2/TM-5(T-2: JSONからdiv-by-zero panic注入可能、Hash/Ord前提が静かに壊れる — **ジャーナルの生命線**。[R1追補 追-1](2026-07-09-R1-export-review.md)と同件、a3a05d5でも現存確認済み。T-7: 同値写像が別ハッシュ) | 不正JSON(den=0等)がロード時エラーになるproptest/単体テスト緑 | 小 |
| [ ] M2E-17 | duration/区間規約の確定: 「duration=総尺、区間は半開`[start, start+duration)`」を仕様宣言し、`export_frame_count`の`+1`と`ParamDriverContext`のフェンスポストを規約に合わせる | 監査TM-3(T-3: 2流儀混在+オフバイワンがテストに焼き込み済み。M4区間キャッシュとM2音声終端の前提) | 規約が仕様に明文化され、`from_frame(89)`系テストが総尺流儀で更新 | 小 |

## 入場条件に含めない(M2期間中・M4前でよいと判断したもの)

| 項目 | 理由 |
|---|---|
| PB-6(migrations宣言化)/ PB-7〜PB-12 | マージ衝突・タクソノミー・&'staticはプラグイン数が実際に増え始めてからで間に合う。M2期間中の並走チケット |
| GR-1(refcountプール) | G-1の出力エイリアスはIssue #16で対処済み(専有出力+生存回避)。プール化はM4-K1の形の話であり、**M4入場条件の候補**。ただしM2-D3(木構造グラフ)着手時に生存回避のスケールを再評価する |
| TM-1(exportループの時刻駆動化) | 暫定拒否はIssue #18で実装済み。ループ反転はM2の実デコード再写像タスク**そのもの**であり、入場条件ではなくM2の作業 |
| GR-2〜GR-7 / CQ-2/CQ-4〜CQ-8 / LG群 | M3/M4との衝突回避が主目的。各フェーズの入場条件または期間中チケットへ(監査Part 4参照) |

## 宣言方針

1. A群(審判)→ B群(プラグイン境界)→ C群(宣言・骨格)の順を推奨(Aが先なのは、B/C以降の全PRがAの審判の下で検証されるため)
2. 全項目チェック+証跡が揃ったら、本ファイル冒頭のステータスを「**達成**」に書き換えてM2-D1以降の並列発注を解禁する
3. M2仕様([specs/M2-document-model.md](../specs/M2-document-model.md))のステータス行に本ゲートへの参照を置く(D1の依存に「M2入場条件 全緑」を含める)
