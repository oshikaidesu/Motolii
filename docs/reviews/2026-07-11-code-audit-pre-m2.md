# 実コード監査: M2並列解禁前に詰めるべき設計箇所(2026-07-11)

ステータス: **所見報告**(チケット化・実装は未着手)

> **2026-07-23現行注記**: 本文の場所・現状・test数は対象SHA `f020ec8` 当時の監査記録であり、現在の欠陥一覧ではない。全2版を現行コードと再照合した処分は[Unit 4C-3回収](2026-07-23-historical-first-code-audit-lineage-recovery.md)を参照する。M2入場項目の多くは実装済みだが、PB-6〜PB-8/PB-10、TM-6/TM-7、GR-1/GR-3/GR-4、CQ-2/CQ-6/CQ-7/CQ-8、LG-2等は全体または一部が残る。

## 前提と手法

- 対象: 凍結ゲート宣言(2026-07-10)後のworkspace全クレート(44ファイル)。**対象SHA: `f020ec8`**(監査時のセッション作業ツリーの分岐点)
- **追記(2026-07-11 同日)**: 監査完了直後にmainが`a3a05d5`へ前進した(Issue #15〜#20の修正マージ)。状態が変わった所見: **G-7(stderr未ドレイン)はPR #30で修正済み** / **G-1の出力エイリアスはIssue #16(専有出力テクスチャ`create_owned_output_texture`+生存回避付き`acquire_render_target`)で対処済み**(M4向けrefcountプール化は未了のまま) / **T-1の暫定拒否(非恒等TimeMap×export→`InvalidRequest`)はIssue #18で実装済み**(時刻駆動へのループ反転はM2のまま)。RationalTime serde(T-2)・encoder finish後expect・ctx poisoned expect/take()は`a3a05d5`でも現存を確認済み。**その他の所見はf020ec8基準のため、チケット採用時に最新mainで再確認すること**(検証注記の運用どおり)。R1レビューの見落とし検証は[2026-07-09-R1-export-review.md 追補](2026-07-09-R1-export-review.md)を参照
- 方法: 6観点の独立監査 — ①時間モデル ②座標・色・Quality ③プラグイン契約・param同一性 ④スキーマ・所有権 ⑤GPU・メディアライフサイクル ⑥テスト施行層
- 既知残件([freeze-gate-remaining](2026-07-10-freeze-gate-remaining.md)のFG-C1〜C6消化済み・「スコープ外(M2以降・明示)」)と重複する指摘は除外した
- **検証注記: 本レポートはLLM監査の出力である。各所見は採用前にfile:lineの現物を必ず確認すること。**LLMが誤る可能性は十二分にある — 所見の記載は「証拠の座標+検証手順」であって「確定した事実」ではない。反証を試みて生き残ったものだけをチケット化する(Part 3には監査中に反証を試みて合格判定になった項目も残してある)
- 優先軸(2026-07-11 判断): **プラグイン境界のエラーはコンセプト直結の根幹問題**として最上位に置く。それ以外の領域は先駆者の解答が既に存在する(SQLite WAL、NLEの時刻駆動ループ、refcountプール、OCIO-shaped等)ため、「解答を適用できる口の確保+仕様への明文化」を今行い、本実装は各フェーズのタスクで行う

---

## Part 1: プラグイン境界(最優先 — コンセプトの成立可否に直結)

なぜ根幹か: このプロジェクトの賭けは「プラグインがLLMでも書ける」×「書いたものが自動検証で安全」([concept.md](../concept.md)、README「LLM-driven development」)。プラグイン境界のエラーは3種類あり、**いずれも費用がプラグイン数で乗算される**:

- **(a) 契約変更の破壊**: trait引数の追加=全プラグイン一斉改修。並列LLM開発では最悪の種類の破壊的変更
- **(b) サイレントな誤り**: 型不一致がデフォルト値で通り「もっともらしく間違う絵」が出る。エラーが出ないためゴールデン差分でしか気づけない
- **(c) 検査漏れ**: 検査がopt-inで、書き忘れた1個の隠れ状態プラグインがフレーム並列(絶対規律3)を恒久破壊する

AEの教訓([ae-pain-points.md](../ae-pain-points.md) C節)は「窓口が狭い」ことの害だったが、**窓口を広くした場合の害は「エラーの乗算」**であり、それを防ぐのがこのPartの全項目。

### P-1. Filter/Compositeに Context 構造体が無い — 引数追加=全プラグイン破壊【解凍手続き対象】

- **場所**: `crates/motolii-plugin/src/lib.rs:297-354`(`FilterPlugin::render` / `CompositePlugin::render` が gpu/pipelines/encoder/t/params/tex の裸引数列。`#[allow(clippy::too_many_arguments)]`付き)
- **現状**: LayerSourceには`LayerSourceContext`(lib.rs:105-109)、ParamDriverには`ParamDriverContext`があるのに、Filter/Compositeだけ裸引数。`Quality`・出力FrameDesc以外の文脈・正準ビューポートを渡す口が無い。`render_graph_cached`はQualityを`quality.render_desc()`で解像度に畳み込み(`motolii-render/src/lib.rs:324-326`)、**プラグインはDraft/Finalを判別できない**(`TextureRef.desc`の解像度からは「scale=2のDraft」と「元々半分の解像度」が区別不能)。この形ではモーションブラーの`effect_samples`削減が原理的に書けず、concept決定(Quality型にサンプル数の口)と矛盾する
- **さらに**: 凍結文書自身がF-7 `InstanceIndex`・F-11 `CompLookbehind`・F-12 `TemporalFootprint`の「配線はM2以降」を予約済み — **これらは全部per-call情報**であり、予約を配線するたびにtraitシグネチャ破壊=全プラグイン改修になる
- **顕在化**: M2開始前の今が唯一の安値点(参照プラグイン数個)。プラグイン100個後は100×改修
- **措置**: `(t, params)`以降を `#[non_exhaustive] pub struct RenderCtx { pub t, pub quality, /* 予約フィールド */ }` に畳む。LayerSourceContextと同型のパターン。凍結項目2(プラグインtrait)の変更なので**解凍手続き(理由+migrate+ゴールデン更新の3点セット)を通す**

### P-2. param型不一致がサイレント・デフォルト — 「もっともらしく間違う」レンダの温床

- **場所**: `crates/motolii-plugin/src/lib.rs:87-89`(`ResolvedParams::f64_or` = `unwrap_or(fallback)`)。参照プラグイン全部が同パターン(lib.rs:576-579, 660, 770-772, 979-988の`match ... _ => デフォルト`)。`crates/motolii-cli/src/project.rs:202-219`(JSONロード時に未知IDは弾くが**value_typeは一切検証しない**)。`motolii-eval/src/value.rs:29`(`Value::lerp`も型不一致でaを黙って返す)。`PluginError`にParamTypeMismatch相当のバリアントが無い(lib.rs:111-121)
- **現状の再現**: JSONに`"amplitude": {"Vec2": [...]}`と書くと、既知キー検査は通過 → `f64_or("amplitude", 1.0)`が黙って1.0にフォールバック → **エラーゼロで間違った絵が出る**
- **顕在化**: LLM量産ではparam型ミスが最頻出バグ。サイレントだとゴールデン差分でしか発見できず、発見コストが最悪
- **措置**: (a) `ResolvedParams::require_f64(id) -> Result<f64, PluginError>`系の型付きアクセサ+`PluginError::Param { plugin, id, expected, got }`を追加し、plugin-authoring.mdで`f64_or`を非推奨化。(b) ロード時に`ParamDef.value_type`と突き合わせる(validate_node_descのlib.rs:212-225と同じ`matches!`を再利用)

### P-3. ホスト側param解決が一元化されていない — RenderStep::Plugin経路はdesc照合ゼロ

- **場所**: `crates/motolii-cli/src/project.rs:194-219`(ParamDriverだけ手書きで migrate→未知ID検査→デフォルト充填)。`crates/motolii-render/src/lib.rs:84-89, 681-780`(`RenderStep::Plugin`の`params: ResolvedParams`は呼び出し側の生値をそのままdispatch — 欠落param検査・デフォルト充填・migrate呼び出しが無い)
- **確認済みの合格**: param同一性(安定ID)自体は全経路で守られている(配列位置結合はゼロ)
- **顕在化**: M2でFilter/Compositeの永続化を書くエージェントが、migrate呼び忘れ・型検査漏れを各自再発明する
- **措置**: `NodeDesc::resolve_params(&self, raw: &HashMap<String, Value>) -> Result<ResolvedParams, PluginError>`(未知ID→Err、欠落→default充填、型不一致→Err)をmotolii-pluginに1個置き、project.rsをこれに置換。P-2と同一PRで可能

### P-4. 検査を「登録した瞬間に検査対象」へ反転する(+検査の土台の穴)

- **場所**: `crates/motolii-testkit/tests/purity.rs`(参照プラグインを1個ずつ**手書き列挙**)。`PluginRegistry`(plugin/src/lib.rs:357-475)は**iter APIが存在せず**、「登録済み全プラグインへ一括検査」は書きたくても書けない。LayerSource/Composite用のpurityヘルパー自体が無い(testkit lib.rs:303-436は`assert_filter_pure`/`assert_param_driver_pure`のみ)
- **検査能力の限界(現状把握)**: 現行のpurity検査は「同一t+inputsで2回呼んで一致」= 呼び出し間で変わる隠れ状態(AtomicU32等)は捕まえるが、**tをキーにした状態・粗い時刻依存・プロセス間非決定性は素通り**する
- **土台の穴(実測)**: `gpu_or_skip`(testkit lib.rs:282-290)が無音スキップするため、**Vulkan ICD・ffmpegの無い環境で`cargo test --workspace`が141テスト全緑・1.2秒で「合格」する**(本監査で実測)。ゴールデン・purity・export系は一度も実行されていないのに緑=エージェントの自己検証がゼロ検証になる。CI側もmesaインストールが壊れた瞬間に同じ状態になる(ci.yml:21-23に既にaptのworkaround前科あり)
- **措置**:
  1. `PluginRegistry::iter(kind)`を追加し、testkitに`assert_registry_pure(&registry, probe_inputs)`を1本 — 登録済み全Filter/ParamDriver/Composite/LayerSourceへvalidate+purityを機械適用(「忘れられない構造」への反転)
  2. `assert_layer_source_pure` / `assert_composite_pure`を追加(CompositeはM2の主力種別)
  3. CIに`MOTOLII_REQUIRE_GPU=1`を設定し、`gpu_or_skip`はこれが立っていればスキップでなくpanic。「GPUが無ければ必ず赤になるカナリアテスト」1本を常設

### P-5. new-pluginスキャフォールドに purity テストとゴールデンが無い

- **場所**: `scripts/new_plugin.py`(生成物のテストは`generated_desc_passes_validate_node_desc`の1本だけ)。検証テスト`crates/motolii-plugin/tests/new_plugin_scaffold.rs:120`もdesc検証のみ。ParamDefの例示も無い(`params: vec![]`固定)
- **顕在化**: LLMは「型紙にあるものは書き、無いものは書かない」。purity検査の普及率はスキャフォールド同梱率にほぼ一致する
- **措置**: テンプレに (a) `assert_*_pure`呼び出しスタブ(`gpu_or_skip`付き)、(b) `assert_rgba_close`ゴールデンスタブ、(c) ParamDef 1個の例、を追加。数十行

### P-6. migrateが中央match+手書き範囲ロジック — 100プラグインでマージ衝突ホットスポット化

- **場所**: `crates/motolii-plugin/src/lib.rs:271-274`(`match plugin_id { "core.param.sine" => ..., _ => Ok(()) }`)、277-295(`from_version < 2 && to_version >= 2`の手書き段階判定)
- **欠陥**: (a) version>1の新プラグインは全員この1関数にarmを足す=並列エージェントのマージ衝突集中点。(b) 段階移行の枠が無く各自が範囲比較を手書き(v2→v3追加時にv1→v3合成を間違えるのが定番)。(c) migrationがNodeDescに紐付かないため「versionを上げたのにmigrateを書き忘れる」を機械検出できない。(d) migrate呼び出しはParamDriverロード経路のみ(project.rs:195)
- **措置**: `NodeDesc`に`migrations: &'static [ParamMigration]`(`{ to_version, apply: fn(&mut HashMap<String,Value>) -> Result<..> }`)を追加しホストが順次適用。conformanceに「version > migrations末尾+1なら赤」「migrate後のキー ⊆ desc.params」のレジストリ全周テストを追加

### P-7. 「versionを上げ忘れた」ことを機械検出できない

- **場所**: `crates/motolii-plugin/src/lib.rs:57-58, 182-184, 262-270`
- **確認済みの合格**: versionは不透明u32、fromはドキュメント保存値(`effect_version`、既定1)、downgradeはtyped error — 凍結どおり健全
- **欠陥**: paramsスキーマを変えたのにversionを据え置く事故はレビュー頼み
- **措置**: conformanceに「登録済み全descの(id, version, param id+型リスト)のスナップショット比較」テストを追加。スナップショット更新=意図的変更の宣言として機能する(G-1入場条件「param互換が並列安全の要」の機械化)

### P-8. PipelineCache: keyにformatが無い+uniform_bufferがキー単位共有

- **場所**: `crates/motolii-gpu/src/pipeline_cache.rs:11-15, 18-24, 47-60, 125, 143`
- **欠陥2件**:
  1. キーは`(id: &'static str, wgsl: &'static str)`のみで**ターゲットformatが入っていない**。fp16 Draft中間(performance-model約束)やM3のBgra8サーフェス対応で、同一WGSL・別formatのパイプラインがキャッシュ衝突する
  2. `uniform_buffer`がパイプラインと同居。現在はPluginステップごとにencoderをsubmitする(`motolii-render/src/lib.rs:397-415`)から動くが、M2以降で複数ステップを1 encoderにバッチ化した瞬間、**同一フィルタ2インスタンスの`write_buffer`が後勝ちになり両パスが同じuniformで描かれる**
  3. (小)対応レイアウトが`tex_sample_uniform4`1種のみ — 2テクスチャ入力・computeが要るプラグインはキャッシュ不可能で、F-10(ホスト所有)と§3-6(毎フレーム生成禁止)が両立できず各自が抜け道を掘る
- **措置**: `PipelineCacheKey`に`format: wgpu::TextureFormat`を追加(呼び出し側2箇所)。「1 submit 1インスタンス」制約をpipeline_cache.rsのdocとplugin-authoring §8に明文化(uniformのリング/dynamic offset化はM2でレイアウト種を増やす際に)

### P-9. categoryが自由文字列 — F-8タクソノミー崩壊が未ガード

- **場所**: `crates/motolii-plugin/src/lib.rs:188-190`(validateは非空検査のみ)。正準リストはdocs/plugin-authoring.md:33に「例:」として存在するだけ
- **欠陥**: tagsは小文字ascii強制(lib.rs:194-203)なのにcategoryは`"Color"` / `"color"` / `"Colour"` / `"色"`が全部通る。100プラグイン時にUIブラウザのカテゴリが発散(F-8そのもの)
- **措置**: `pub const CATEGORIES: &[&str]`をmotolii-pluginに置きvalidate_node_descで照合(5行)。スキャフォールドは既に正準値を吐くので追随コストゼロ

### P-10. panic禁止の機械検査がmotolii-plugin/src限定 — 他クレートに違反実例あり

- **場所**: 検査範囲は`crates/motolii-plugin/tests/conformance.rs:257-273`(`motolii-plugin/src`のみ走査)。違反実例: `crates/motolii-nodes/src/lib.rs:55`の`assert!(width_px > 0 && ...)` — 公開API`ViewportTransform::new`のpanic(AGENTS.md実装規約違反)
- **欠陥2件**: (a) M2でプラグインが別クレートに置かれた瞬間、panic検査ゼロカバレッジ。(b) 走査は「`#[cfg(test)]`はファイル末尾」前提(conformance.rs:186)なので、中間にtest modを書くファイルは後半が素通り
- **措置**: 走査対象を「プラグインを含む全クレートsrc/」に拡大。`ViewportTransform::new`は`Result`化。長期的には`[lints]`で`clippy::unwrap_used`等をdenyする方が堅い(pitfalls H-1のはしご)

### P-11. 未知プラグインIDの挙動が層で不統一(F-9パススルーが片肺)

- **場所**: `crates/motolii-cli/src/project.rs:190-191`(未知ParamDriver→**ロード失敗**)。一方docs/plugin-authoring.md:38は「未知idはロード失敗にしない(警告+パススルー)」を規約として明記。render側は`RenderError::UnknownPlugin`(motolii-render/src/lib.rs:779)
- **欠陥**: 契約書と実装が既に食い違っている。M2-D1を書くエージェントがどちらを型紙にするかで挙動が割れる
- **措置**: 「**プロジェクトロード=警告+保持(パススルー)** / **評価=typed error**」の層別を1箇所(motolii-doc層)で実装し、規約との一致テストを置く(M2-D1完了条件に既にあるが、方針の宣言はD1着手前に)

### P-12. `&'static str`汎用化がv2(dylib/WASM)と動的経路を構造的にブロック

- **場所**: `crates/motolii-plugin/src/lib.rs:15-16`(`PluginId(pub &'static str)`)、48-52(`ParamDef.id`)、70-90(`ResolvedParams`キー)。スキャフォールドテストが既に`Box::leak`を3連発している(tests/new_plugin_scaffold.rs:47-53)
- **凍結との関係**: 凍結項目16が「lifetimeは凍結対象外、String化は互換変更として許す」と明記済みなので**違反ではない**。ただしプラグイン数に比例して変更コストが増える型なので、通路の確保は早いほど安い
- **措置**: 全面String化は今やらない。`PluginId`を非公開フィールド+コンストラクタ化、`ResolvedParams`キーを`Cow<'static, str>`化して内部表現変更を可能にしておく

---

## Part 2: 先駆者の解答が存在する領域(詳細保持。対応は「口の確保+仕様明文化」を今、本実装は各フェーズ)

### 2-A. 時間モデル

**総括**: RationalTime規律は良好(演算・比較・キーフレーム時刻は有理数で一貫、**時刻の蓄積誤差経路は存在しない**)。問題は構造1点(T-1)と衛生数点。

#### T-1.【構造】書き出しループが「デコーダ駆動・フレーム連番」で、TimeMapは報告専用 — 実画素が写像を通らない

- **場所**: `crates/motolii-export/src/lib.rs:89-123`(`while reader.next_frame()`ループ)、`crates/motolii-media/src/decode.rs:96`(`pts = from_frame(next_frame_index, fps)`)
- **現状**: ループの主語がデコーダ。`frame.pts`(=ソースのフレーム連番/fps)を`timeline_time`として評価とレンダに渡し、`time_map.try_map`は`source_time`の**報告値**を計算するだけで、**どのソースフレームが合成されるかを変えない**。非恒等TimeMapを渡すと「報告されるsource_time」と「実際にデコードされた画素」が乖離する(現状でもJSONで表現可能な不整合)。render_frame自体は純f(t)だが、**駆動ループが「iteratorの次」を時刻源にしている**(GPU監査と時間監査が独立に同一指摘)
- **顕在化**: M2の実デコード再写像(FG-C2の「スコープ外」明示分)は、ループを「出力フレームn → timeline_time → try_map → ソースをシーク/リピート/スキップ」へ**反転**する必要がある。speed<1はフレーム重複、speed>1はスキップが要り、順次読みの`FrameReader`前提ごと変わる。M4もこのループにキャッシュを載せると二度作り直し
- **措置(先駆者の解答=NLE標準の時刻駆動ループ)**: 恒等のまま挙動を変えずにループを「出力インデックス駆動」(`for n in 0..count { let t = from_frame(n, fps); let src = try_map(t)?; ... }`)へ書き換え、`FrameReader`に「次に欲しいsource_time」を渡す口だけ用意(実装は連番のままassertで恒等を要求)。加えて**非恒等TimeMap+exportの組合せを当面`InvalidRequest`で拒否**し、乖離状態をゴールデンに焼かせない

#### T-2. `RationalTime`のDeserializeが不変条件(den>0・既約)を素通し — JSONからpanic/順序破壊を注入可能

- **場所**: `crates/motolii-core/src/time.rs:10`(deriveが非公開フィールドに直結。`new()`のみが正規化)
- **再現**: `{"t":{"num":1,"den":0}}`は受理され、`to_frame_floor`の`div_euclid`でdivide-by-zero panic(INF-7b違反)。`den<0`は`Ord`/`Hash`/`Eq`(time.rs:102-131はden>0・既約前提とコメント明記)を静かに壊す。非既約(2/4 vs 1/2)は`Eq`/`Hash`不一致
- **顕在化**: M2のジャーナルリプレイで不正値が混入すると「時々起きるオフバイワン」として現れ原因特定が最悪。M4はRationalTimeをキャッシュキーに含むためHash不一致=見えないキャッシュミス/誤ヒット
- **措置**: `#[serde(try_from = "RawRational")]`で`new()`経由に強制(den==0はエラー、負・非既約は正規化)。1ファイル数十行、スキーマ非破壊

#### T-3. 「duration」の意味が2流儀混在+実質オフバイワン(ストリーム終端が隠蔽)

- **場所**: `crates/motolii-media/src/probe.rs:170-174`(duration=**総尺**、round snap) vs `crates/motolii-cli/src/project.rs:168-169`(`to_frame_floor(fps) + 1` — durationを**最終フレームのPTS**として扱う。90フレーム=3.0s素材で91フレームと算出、今はデコーダEOFで止まるため無害に見える)。テスト(project.rs:643-648)は`from_frame(89)`=最終PTS流儀を焼き込み済み。`ParamDriverContext.duration`+`floor+1`のつじつま合わせがplugin/lib.rs:1017-1019と2クレートに分散
- **顕在化**: M4のキャッシュキーは時間**区間**。区間の開閉規約とduration定義が曖昧なままだと境界フレームで恒常的な無効化漏れ/過剰無効化。M2-D5でも「クリップ終端」が音声サンプル終端とズレる
- **措置**: 「**duration=総尺、区間はすべて半開`[start, start+duration)`**」を仕様宣言し、`export_frame_count`の`+1`を半開規約に修正、`ParamDriverContext`は総尺+`(0..n)`半開で定義し直す。テストの`from_frame(89)`も同時修正

#### T-4. 時刻→フレーム変換の丸め規則が4箇所4流儀

- **場所**: ①`time.rs:48` `to_frame_floor`(床、唯一の正規口) ②`probe.rs:172` `(secs*fps).round()`(四捨五入、f64経由) ③`decode.rs:46-47` シーク=`(start_frame-0.5)/fps`のf64→10進6桁文字列 ④`plugin/lib.rs:1018-1019` `floor(secs*rate)+1`(f64経由)
- **顕在化**: M4の区間キャッシュとM2 Transportのスクラブが別々の丸めを踏むと、同じtで違うフレームを指す1フレームズレがゴールデンに焼かれる
- **措置**: `motolii-core::time`に`to_frame_round(fps)`(有理数演算で最近傍)を追加しf64計算を置換。シーク秒文字列化もcoreの関数に集約し「時刻→フレーム/秒文字列はcoreの2関数のみ」とdoc宣言

#### T-5. ProjectV1が「タイムラインfps=入力素材fps」という単一グローバルfpsを暗黙化

- **場所**: `crates/motolii-cli/src/project.rs:23-41`(fpsフィールドが無い)、252-254(probe結果のfpsでstart/duration/DataTrackサンプルレートを全部決定)、`encode.rs:29`(出力`-r`も同一fps)
- **顕在化**: M2のDocumentはcomp fpsを持ち、クリップfps≠comp fpsになった瞬間`start_frame`の意味(どのfpsで数えた番号か)が曖昧になる
- **措置**: 型変更はせず仕様に明文化 —「v1の`start_frame`/`frame_count`は入力素材fps基準。M2のDocumentは`RationalTime`のin/out点を使い、フレーム添字をスキーマに入れない」(→ F-6と同件)

#### T-6. パラメータ評価が点評価のみ・Valueに正準ハッシュが無い — M4キー(node×区間×param-hash)の材料が未整備

- **場所**: `crates/motolii-eval/src/track.rs:54, 102`(点評価のみ。`keys()`公開は救い)、`crates/motolii-eval/src/value.rs:4`(`Value`は`PartialEq`のみ。f64ベースでEq/Hash無し、NaN/-0.0未定義)
- **顕在化**: M4側が独自のf64ビットハッシュをアドホックに書き始めると-0.0/NaN/非既約問題が散在する
- **措置**(実装不要、契約のみ): (a) `Value`に「正準バイト列エンコード経由でハッシュ(f64はto_bits、-0.0→+0.0正規化、NaN拒否)」をdoc宣言 (b) `KeyframeTrack`に`keys_in(range)`/`next_key_after(t)`のシグネチャだけ切る(実装はbinary_searchで数行)

#### T-7. `TimeMap`のEq/Hashが非正準+validateがspeed_den<0を素通し

- **場所**: `crates/motolii-core/src/time_map.rs:11-17`(derive Hash/Eq、speed 2/1と4/2は別値)、63-69(validateは`speed_den==0`のみ拒否)
- **顕在化**: try_mapの算術は`RationalTime::new`が正規化するので正しいが、**同値な写像が別ハッシュ**になる。M4でTimeMapがキャッシュキーに入ると偽ミス
- **措置**: コンストラクタとDeserializeでspeedをgcd正規化+den>0へ符号寄せ(逆再生speed_num<0は許容と明記)。写像結果不変のまま内部正規化は今なら無償

#### T-8. `MediaInfo.duration`のfpsグリッドスナップが音声主クロック(M2-D5)と衝突する

- **場所**: `crates/motolii-media/src/probe.rs:19-20, 170-174`(総尺を映像fpsグリッドへround snap、真の尺を破棄)。テスト:195が`den() <= 30000`という前提まで焼き込み
- **顕在化**: 音声主クロックの再生位置はn/48000でfpsグリッドに乗らない。probe段階で映像グリッドに丸めた尺しか残らないため、末尾の無音パディング/尻切れ判断ができない
- **措置**: スナップは維持しつつ`MediaInfo`に`duration_raw: Option<RationalTime>`(ffprobe文字列を10進exact→有理数、f64非経由)を並置。スナップは「映像フレーム数を数える用途専用」とdoc限定宣言

#### T-9. パラメータ評価の時間原点がソース絶対時刻に癒着(`start_frame`シフト)

- **場所**: `crates/motolii-export/src/lib.rs:105`(`overlay.eval(frame.pts, ...)`)+`decode.rs:96`。`start_frame>0`の書き出しではキーフレーム/DataTrackが**t=start_frame/fps起点**で評価される(タイムライン0起点でない)
- **顕在化**: M2で「クリップin点をずらしてもアニメはタイムライン時刻で評価」という当然の分離を入れた瞬間、start_frame付きプロジェクトの見た目が変わる
- **措置**(コード変更不要): 仕様に「v1では評価時刻=ソースPTS(タイムライン=ソースの縮退)。M2でtimeline_timeに再定義し、start_frame付きv1プロジェクトは移行時にキーフレームをシフト」と決定を明文化

#### T-10. ffmpegシークがf64秒の10進文字列契約(低)

- **場所**: `crates/motolii-media/src/decode.rs:46-47`。半フレームガードのおかげで現行CFR用途は堅牢
- **措置**: コメントに「M2でPTSシークへ置換予定、丸め規則はcoreへ移管」の一行のみ

#### T-11. f64リーク残存3点(低)

- (a) `DataTrack::eval`のサンプル位置posがf64(track.rs:108)— 線形補間なので視覚的に無害、離散値トラック追加時に境界1サンプルズレの芽 (b) `sample_count`のf64(plugin/lib.rs:1018)→T-4で解消 (c) `parse_duration_snapped`のf64 parse(probe.rs:172)→T-8で解消

### 2-B. GPU・レンダセッション・メディアライフサイクル

#### G-1.【構造】ピンポン2枚が出力テクスチャをエイリアス+「生存中間値≤2の線形グラフ」前提 — M3-U1/M2-D3/M4-K1の三方向で衝突

- **場所**: `crates/motolii-render/src/lib.rs:177-183, 235-251`(`acquire_ping`)、373(CompositeNormal出力もピンポンから取得)
- **現状2件**:
  1. `RenderedFrame.texture`はピンポンバッファのclone(Arc)。1フレームあたりの`acquire_ping`回数が偶数(現デモグラフは2回)だと**毎フレーム同じバッファが出力になる**。M3-U1の「完成テクスチャをチャネルでUIへ送る」方式では、UIが表示中のテクスチャをレンダスレッドが次フレームで上書きする(tearing)。wgpuのArcでuse-after-freeにはならないため**検証エラーすら出ず、意味的破壊が黙って起きる**。yuv.rs:61のSizePoolは同じ理由で出力2枚ピンポンを明示しているのに、render出力には保護が無い
  2. 木構造(M2-D3: マスク→グループ合成)では`composite(compA(x,y), compB(z,w))`の3回目のacquireが**まだ入力として生きているバッファを返す** — 同一パスでread/writeエイリアス。M4-K1の「参照カウントハンドルでノード出力をフレーム跨ぎRETAIN」もセッション回収前提のバッファでは毎回コピーが必要になり無効化される
- **措置(先駆者の解答=refcount付きプール。K1が要求するハンドル形状の前倒し)**: executorが1関数の今、2枚固定ピンポンを「生存解析ベースのacquire/release付き小プール(参照カウントハンドル)」に置換。K1はこのプールに予算とLRUを足すだけになる。最低限でも「`RenderedFrame`は次の`render_graph_cached`呼び出しで無効」をM3仕様に契約として明文化し、U1は表示前コピー設計にする

#### G-2. download_rgba/poll(Wait)の共有デバイス禁止が規約のみで構造的強制なし

- **場所**: `crates/motolii-gpu/src/transfer.rs:189`(`device.poll(PollType::Wait)`)、62(download_rgba)、`crates/motolii-export/src/lib.rs:61`(`export_overlay_video(gpu: &GpuCtx)`)
- **顕在化**: M3規約3「共有デバイスでのpoll(Wait)禁止」は型で守られておらず、`new_for_ui`のctxをexportに渡してもコンパイルが通る。誤用時の症状は「UIが数百msフリーズ」で原因特定が難しい
- **措置**: `GpuCtx`に生成起源(Headless/UiShared)タグを持たせ`wait_for_map`冒頭でUiSharedなら型付きエラー。またはexport引数を`HeadlessGpuCtx`ニュータイプに。数十行

#### G-3. FrameReaderは順方向専用・シーク=プロセス再起動・キャンセル不能

- **場所**: `crates/motolii-media/src/decode.rs:28-69`(open時`-ss`固定)、76-99(next_frameはタイムアウト無しブロッキングread)、134-143(`read_frame_at`は**1フレームごとにffmpeg spawn**)
- **評価**: API形状(`open(start_frame)`+`next_frame`)自体はM4プールの内部プリミティブとして延命可能(全書き直し不要)。問題はキャンセルが「所有者のDrop=kill」しかなく、ブロッキングread中のスレッドを外から解放できないこと
- **措置**: 仕様に「FrameReaderはプールの内部単位。プールは(asset×近傍位置)でreaderを再利用し、シーク距離が閾値超なら再起動」を契約化。コードは`next_frame`にキャンセルトークン(AtomicBool)チェック+子プロセスkillハンドルの分離を先に入れる

#### G-4. VRAM予算計上のフックが皆無・テクスチャ生成は5箇所に分散(今なら安い)

- **場所**: `transfer.rs:18`(upload_rgba — **呼ぶたび新規テクスチャ**)、`motolii-nodes/src/lib.rs:249`、`yuv.rs:255,322`、`pipeline_cache.rs:143`
- **顕在化**: memory-model P3「VRAM予算は自前管理」だがバイト数を数える場所がゼロ。M4後に生成箇所が散らばってからでは全crate横断改修
- **措置**: `GpuCtx::create_texture`ラッパー(サイズ集計カウンタ付き)へ一本化し、直呼びをレビュー/走査で禁止。K1は予算判定をカウンタに繋ぐだけになる

#### G-5. Overlay/Compositeが毎フレームuniform buffer+bind group新規作成、ステップ毎に個別submit

- **場所**: `motolii-nodes/src/lib.rs:433-439, 709-711`(create_buffer_init毎フレーム)。submit分散: `nodes:235,491`、`yuv.rs:236`、`render/lib.rs:415`
- **顕在化**: performance-model原則3「毎フレーム確保しない」に**コア自身が違反**(pipeline_cache.rs:23は正しく永続buffer+write_bufferをモデル化済みなのにノード側が使っていない)。フレームが多数の小submitに分かれ、M3でUI共有queueに混ざるとフレーム境界が観測不能・M4の計測フック(K1)の置き場も無い
- **措置**: ノード2個の今、uniformを永続buffer+write_bufferへ、フレーム全体を1 encoderに統合

#### G-6. GPU非同期エラーの検知がダウンロード経路にしかない — プレビューは事故を見ない

- **場所**: `ctx.rs:107`(`check_health`)の呼び出し元は`transfer.rs:171`(wait_for_map内)のみ。加えて`ctx.rs:93-96`コメント通り、device_lost/on_uncapturedは**デバイスあたり1スロット**で、Slintが後から登録するとコアのハンドラが黙って消える
- **顕在化**: M3プレビューは読み戻さないため、device lostが起きても`render_graph_cached`はOkを返し続け画面はゴミか黒
- **措置**: レンダループ毎フレーム`check_health()`+「デバイスコールバックはコアが唯一の登録者、Slint側登録禁止」をM3「デバイスとスレッドの規約」に4項目目として追記

#### G-7. ffmpegのstderrが終了時まで排出されない — パイプ詰まりデッドロックの芽(M1実装ガードG1と同件が現物に)

- **場所**: `decode.rs:55` / `encode.rs:54`(stderr piped)、読むのは終了後のみ(decode.rs:101-111 / encode.rs:78-89)
- **措置**: spawn直後にstderrを有界バッファへ吸うスレッド1本(両ファイル共通ヘルパ20行程度)

#### G-8. Encoder.finish()忘れはコンパイルが通る(低)

- **場所**: `encode.rs:78-97`。`finish(mut self)`が消費するのでfinish後のwrite_frameは型で防げており、export(export/lib.rs:134-141)はエラー時もfinishを呼ぶ正しい形。ただし「finishを呼ばずDrop」は無警告でmoovなしmp4
- **措置**: Dropで`stdin.is_some() && !thread::panicking()`なら警告ログ(またはdebug_assert)。呼び出し元が増えるM4ジョブ化の前に

#### G-9. YuvToRgba出力の保持契約がコメントのみ+変換器の共有単位が未定義(低)

- **場所**: `yuv.rs:61`「2回以上のconvertを跨いで保持する用途は想定しない」— M4キャッシュが踏む地雷がdoc commentにしか無い
- **措置**: M4仕様K4/K1へ転記(「デコード出力の保持はキャッシュ側がCOPYまたは専有テクスチャで」+「YuvToRgbaはストリーム毎所有」)

#### G-10. `render_frame()`は毎回RenderSession新規生成=毎フレームシェーダ再コンパイル(注意書きのみ)

- **場所**: `render/lib.rs:254-268`
- **措置**: M3-U1が使う入口は`render_graph_cached`である旨を仕様に一言

### 2-C. スキーマ・所有権(motolii-doc / ProjectV1)

#### F-1.【最重要】D1とProjectV1の関係がどこにも宣言されていない

- **場所**: `crates/motolii-cli/src/project.rs:22-41`の`ProjectV1`(version=1)と`crates/motolii-doc/src/lib.rs:17-21`の`Document`(version:1)が**両方「v1」を名乗り**、M2仕様のD1行にProjectV1への言及がゼロ
- **顕在化**: 「マイグレーション枠組み」の凍結約束があるため、M2エージェントは「既存のversion付きスキーマ=ProjectV1を移行対象として増築する」と読むのが自然。ProjectV1にレイヤー配列を生やした瞬間、その形(F-5/F-6/T-5の欠陥ごと)が永久マイグレーション対象になる
- **措置**(1文): 「**D1はProjectV1を継承も移行もしない。ProjectV1はM1 CLI専用の使い捨てで、Documentのversion採番は独立。export-projectはD3完了時にDocument読み込みへ置換**」をM2仕様へ

#### F-2. unknown-keysが黙って捨てられる(Documentは保存経路も持つ)

- **場所**: リポジトリ全体で`deny_unknown_fields`もflatten保持も**ゼロ**(grep確認)。`ProjectV1`はDeserializeのみ(旧バイナリ保存が新ファイルを壊す経路はまだ無い)。危険は`Document`(doc/lib.rs:16) — **Serialize+Deserialize両方**を持ち未知キー保持機構が無い
- **措置**: 骨格の今、`Document`へ`#[serde(flatten)] pub extra: serde_json::Map<String, Value>`を1フィールド予約+テスト1本(M2実装ガード7のroundtripを骨格レベルで先取り)

#### F-3. `min_reader_version`が無い(後から足すと意味がない前方互換フィールド)

- **場所**: `Document { version: u32 }`のみ(doc/lib.rs:18)。この種のフィールドは**旧リーダーが認識していて初めて機能する**ため、D1出荷後の追加ではワンサイクル無駄になる
- **措置**: `#[serde(default = 1)] min_reader_version: u32`相当を今1行追加

#### F-4. 色: リニアかsRGBかがコメント間で既に矛盾

- **場所**: `motolii-eval/src/value.rs:9`「RGBA(**リニア**、0.0-1.0想定)」 vs `project.rs:79`・`motolii-nodes/src/lib.rs:104-123`「straight RGBA, 0..1」(色空間無記載)。実パイプラインは`Rgba8Unorm`+`ColorSpace::Srgb`ターゲット(project.rs:256-262)へ**無変換で書く**ため事実上sRGB符号値として消費
- **顕在化**: キーフレーム済みカラーが量産された後に解釈を変えると全プロジェクトの見た目が変わる(マイグレーション不能な種類の破壊)
- **措置**(1文+コメント修正): 「スキーマの`Color`は**sRGB(非線形)・straight・0-1**。リニア化はレンダ層の責務」(逆を選ぶならレンダ側に変換を入れてから)。value.rs:9を実態に合わせ修正。2-D-1(sRGBブレンド)と同時に決める

#### F-5. ProjectV1は「レシピ」と「書き出しジョブ」の混載構造

- **場所**: `input`/`output`/`start_frame`/`frame_count`/`qp0`(project.rs:26-33)=ジョブ設定、`overlay`/`param_drivers`/`time_map`=レシピ。concept「Document=レシピのみ」に反する形。素材参照も`input: String`の単一キー
- **措置**(1文): 「Document ≠ ExportJob。出力パス・範囲・エンコード設定は別構造。Asset参照は初日から多重キー(M2実装ガード10)」

#### F-6. 時間範囲がフレーム添字(fps依存)で、OTIO約束と食い違う

- **場所**: `start_frame: i64` / `frame_count: Option<usize>`(project.rs:29-31)。キーフレーム(track.rs:25)とTimeMap(time_map.rs:12-17)は有理時刻で健全 — 違反はこの2フィールドだけ
- **措置**(1文): 「D1のクリップin/out/durationは`RationalTime`。フレーム添字をスキーマに入れない」(T-5と同件)

#### F-7. エンティティIDが存在しない(唯一のIDは任意文字列のDataTrackId)

- **場所**: LayerId/ClipIdは皆無。`DataTrackId(pub String)`(eval/lib.rs:23)はユーザー命名文字列。`ParamSource::Data{track: String}`の文字列参照方式が前例化しており、D1でLookAt/Follow用の型付き参照を足すと二重方式が恒久化する
- **措置**: D1着手前にnewtype `LayerId`(u64/ULID)を予約+「表示名はIDと別フィールド。ID再利用禁止」を仕様に。DataTrackIdは解析出力名なので文字列のままで可

#### F-8. DocumentWriter::editがコマンド外変更の正面玄関(F-2骨格自体は健全)

- **確認済みの合格**: `edit(&mut self, f: FnOnce(&mut Document))`(doc/lib.rs:55-57)は戻り値なし+借用がクロージャ内で終わるため`&mut`は型レベルで漏れない。内部可変性なし、第二の変更経路なし
- **問題**: 任意クロージャ変更にコマンド/状態ハッシュのフックが無く、D2「全編集=コマンド」と正面衝突。writerに世代番号も無い(決定性テストの席)
- **措置**: doc-commentに「editはD2で`apply(Command)`に置換される足場。呼び出し追加禁止」+`DocumentWriter`に`revision: u64`を今追加

#### F-9. WriterMessage::applyもコマンド履歴を素通り

- **場所**: doc/lib.rs:59-63(メッセージを直接mut適用)。M2方針「バックグラウンド成果はwriterが**コマンドとして**適用」に対し変換の席が無い=ジャーナル非記録のバックグラウンド変更(リプレイ非決定)の芽
- **措置**: コメント1行「applyはD2でメッセージ→Command変換に置換」

#### F-10. export要求型の4層コピー — 正準はBackgroundTextureRequest、ExportOverlayRequestはD3で消える運命と明記すべき

- **場所**: ProjectV1 → `PreparedProject`(**生と解決済みの二重保持**: project.rs:282-287、render_export_frame_rgbaは`self.overlay`と`self.project.time_map`を混用 project.rs:316-327)→ `ExportOverlayRequest`(export/lib.rs:20-33)→ `BackgroundTextureRequest`(render/lib.rs:40-49)
- **措置**(1文): M2仕様D3行に「ExportOverlayRequest形式のジョブミラーを温存せず、Document→render層リクエスト(BackgroundTextureRequest系)を直結」

#### F-11. BPM/音声の挿入点はクリーン — ただしBPMの型を先に決める

- **確認**: ProjectV1にも`Document`にも音声/BPMフィールドは皆無で衝突なし。注意はbpmを`f64`で持つと拍時刻(60/bpm秒)が非有理になりRationalTime系と混ざって丸め蓄積する点のみ
- **措置**(1文): 「bpmは有理数(またはミリbpm整数)で持ち、拍時刻がRationalTimeに畳めることをD1完了条件に含める」

#### F-12. untaggedラッパーの前例(低)

- **場所**: `ParamVec2V1`/`ParamColorV1`の`#[serde(untagged)]`(project.rs:70,77)。エラーメッセージ喪失+バリアント追加時の曖昧一致+unknown-keys保持との相性最悪。ProjectV1が使い捨て(F-1確定)なら実害ゼロ
- **措置**: D1が同じ糖衣を採るなら試行順序を仕様化したカスタムDeserializeで

### 2-D. 座標・色・Quality

#### C-1.【最重要】合成がsRGBガンマ空間ブレンドでゴールデンに焼き込み済み、`precise_color`は未配線

- **場所**: `crates/motolii-nodes/src/composite_blend.wgsl:41-45`(over合成をテクセル値のまま実行)、`crates/motolii-render/src/lib.rs:871-879`(`validate_render_desc`が`ColorSpace::Srgb`を**要求**)、`crates/motolii-core/src/quality.rs:14`(`precise_color`未使用)。`frame.rs:41`の`LinearRgb`コメント「合成・ブレンドはこの空間で行う」と実態が矛盾。`premul_over_u8`(render/lib.rs:1697)ベースの全ゴールデンがsRGBブレンド結果を正解として固定
- **顕在化**: M5(3D・リニアFP16中間)とperformance-modelのリニアブレンド化で、**composite/overlay/export系ゴールデン全regenerate+Draft(`precise_color:false`)との視覚差の再定義**が同時に来る。M2がこの出力に依存するゴールデンを増やすほど倍増
- **措置**: `precise_color`を`render_desc`/合成シェーダ選択まで**配線だけ**しておく(実装は恒等でよい)。ゴールデンに「v1=sRGBブレンドは暫定決定、リニア化でregenerate」のマーカーを付け、**M2期間中にこの出力依存ゴールデンを増やさない**規約を出す。F-4(スキーマ色の定義)と同時に決定

#### C-2. `Rgba8Unorm`が7箇所ハードコード+`PipelineCacheKey`にformatが無い

- **場所**: `motolii-gpu/src/pipeline_cache.rs:12-15, 125`、`motolii-nodes/src/lib.rs:259, 351, 635`、`motolii-gpu/src/yuv.rs:141, 265`、`motolii-gpu/src/transfer.rs:28, 96`(`unpadded = width * 4`のbpp決め打ち)、`motolii-render/src/lib.rs:872`
- **顕在化**: fp16 Draft中間の導入で全パイプラインのcolor target書き換え+**同一WGSL別formatのキャッシュ衝突**(P-8と同件)。M3のBgra8サーフェスでも同型
- **措置**: `PipelineCacheKey`にformat追加(P-8)。`FrameDesc.format → wgpu::TextureFormat`変換ヘルパーを`motolii-gpu`に1個作り直書きを順次置換。fp16実装自体は不要

#### C-3. 正準座標が「規約のみ」— `Value::Vec2`は無単位、`CanonicalPoint`がmotolii-nodes住まい

- **場所**: `motolii-eval/src/value.rs:5-13`(Vec2に空間単位の区別なし)、`motolii-nodes/src/lib.rs:16-42`(Canonical/Pixel型はnodesクレート内)、`motolii-plugin`の`ParamDef`(lib.rs:48-52)にも正準マーカー無し
- **顕在化**: M2の`motolii-doc`は**coreに依存しnodesに依存しない**。空間パラメータを持つ最初のM2エージェントが、参照できる正準型が無いため独自表現(生`[f64;2]`にpx値)を発明する確率が高い。「px param禁止」を守らせる型が届く場所に無い
- **措置**: `CanonicalPoint`/`CanonicalSize`/`ViewportTransform`/`PixelPoint`/`PixelSize`を`motolii-core`へ移動しnodesはre-export(数十行)。`ParamDef`に空間パラメータの注記(またはValueType追加)はM2着手前に判断

#### C-4. Rec709デコード出力を`ColorSpace::Srgb`として無変換で流している

- **場所**: `motolii-gpu/src/yuv.wgsl`(ガンマ保持で出力、リニア化は「後段の責務」とコメント)、`motolii-cli/src/project.rs:256-262`(render_descが`Srgb`固定)
- **現状**: BT.709 OETF ≠ sRGB伝達関数だが、変換点がどこにも無いまま「Srgbタグ」で下流へ。yuv_golden/swscale_referenceとexport系ゴールデンが「709ガンマ=sRGB扱い」を正解化
- **顕在化**: OCIO形の一点変換を入れる時、YUV→RGBは単点(合格)だが**伝達関数の帳尻がゴールデン全体に散っている**ため、変換を正しくすると映像系ゴールデン全滅
- **措置**: yuv出力のdescを正直に`Rec709`系でタグ付けし、「Rec709gamma→Srgb変換(v1は恒等近似と明示)」の関数を1個置いて経路だけ確保。数値は変えず変換点を作る

#### C-5. カメラが`dispatch_plugin`で`DEFAULT`固定、`RenderGraphInputs`に口が無い

- **場所**: `motolii-render/src/lib.rs:770-773`(`LayerSourceContext { camera: CompCamera::DEFAULT }`直書き)、157-163(`RenderGraphInputs`にcameraフィールド無し)
- **措置**: `RenderGraphInputs`に`camera: Option<CompCamera>`(None=DEFAULT)を今足す。`Default`実装済みなので既存呼び出しは無傷。数行

#### C-6. `Quality::render_desc`の整数除算でDraftのアスペクト(=正準幅)が変わる

- **場所**: `motolii-core/src/quality.rs:47-48`(`width/scale`,`height/scale`の切り捨て。例: 13×7 → 6×3で正準幅1.857→2.0)。FG-C5の重心パリティは許容0.05でこの歪みを隠している(render/lib.rs:1599-1602)
- **顕在化**: M3でプレビューが「Draftテクスチャ上に正準座標でギズモ描画」を始めた瞬間、奇数解像度でギズモと絵がズレる(原因究明コストが高いタイプ)
- **措置**: 丸め規則(偶数へ切り上げ等)を確定し、「正準aspectは常に**Finalのdesc**から計算」を`ViewportTransform`のdocに明文化(+from_descにfull_desc併用の口)

#### C-7. `CompCamera`の座標単位が未定義のままserde可能 — M2が確定前にスキーマへ焼く

- **場所**: `motolii-core/src/camera.rs:8-15`(`position: [f64;3]`の単位・正準高さとの関係が無記述。Serialize/Deserialize済み)
- **良い点**: fov_y度+Y-up+look-atという形はアスペクト非依存でpx混入も無く、M5に耐える形。near/far無しも今は正しい
- **措置**(コード変更ゼロ): docコメント1段落で「単位は正準空間(高さ=1.0)、DEFAULT(z=2.0, fov45°)は高さ≈1.66を写す」等の関係を今決めて書く

#### C-8. `FrameDesc`にPAR(ピクセルアスペクト)と回転が無い — M4キャッシュキーの前提穴

- **場所**: `motolii-core/src/frame.rs:56-65`。`MediaInfo`は`rotation`を持つ(project.rs:646)がFrameDescに落ちない。`ViewportTransform::from_desc`(nodes/lib.rs:62-64)は正方形ピクセル前提
- **顕在化**: アナモルフィック/スマホ回転素材で「幅=aspect」の正準前提が崩れる。M4のキャッシュキーがFrameDesc(Hash済み)を使うなら、後からのフィールド追加=全キー無効化+serde移行。HDRでは`ColorSpace`が伝達関数と原色を混載している点も同根(PQ/HLG追加でenum肥大)
- **措置**: `par_num/par_den`(デフォルト1:1)を今足すのが最安。最低でも直接構築を禁じ`packed()`/`yuv()`等の構築関数経由を徹底して追加コストを下げる

#### C-9. Overlayはバイナリカバレッジ(AA無し)+REPLACE — Vello移行(R8採用済)で形状ゴールデン全regenerate

- **場所**: `motolii-nodes/src/overlay_shapes.wgsl:49-77`(inside判定、エッジぼかし無し)、nodes/lib.rs:352,635(`BlendState::REPLACE`)
- **措置**(コード変更不要): 「overlay形状ゴールデンはVello置換時に全regenerate予定」のタグ付けと、M2〜M5期間中にこの系統のゴールデンを**増やさない**規約

#### C-10. straight→premul境界が「各ノードの実行時分岐」— Vello用の単一アダプタが未確保

- **場所**: 変換自体は`motolii-core/src/frame.rs:156-172`に集約(合格)。強制は`desc.premultiplied` boolの実行時チェック(nodes/lib.rs:829-839)のみで、`OverlayNode`は出力descで色を分岐(nodes/lib.rs:427-432, 496-507)。R8の条件「vello出力→合成の境界でstraight→premul変換1回」の受け皿となるGPU側アダプタが存在しない
- **顕在化**: M5テキスト(Vello draw_glyphs)とM4 SVG(K6)の2チームがそれぞれ境界変換を書くと、二重premul(暗くなる)や漏れ(黒フリンジ)が別々に混入
- **措置**: `motolii-gpu`に`straight_to_premul`のパス(または合成シェーダ入口の1関数)を用意し「Vello系出力は必ずここを通す」を規約化。本命のnewtype(`PremulTexture`)はM2の型整理と同時で可

#### C-11. `validate_render_desc`がエントリポイントでRGBA8+Srgb+premulを固定

- **場所**: `motolii-render/src/lib.rs:871-879`(+`validate_background_desc`:881-891)
- **顕在化**: C-1/C-2の帰結だがAPI面で独立に効く: M5のHDRリニア中間・M3のBgra8対応時、ここが全経路の関門でM2/M3/M5が同じ関数を同時に触る衝突点になる
- **措置**: 許容(format, color_space)の組を定数テーブルに括り出し「対応表を広げる」変更に変形(挙動不変、~15行)

### 2-E. テスト施行層(H-1/H-2の機械化はしごの適用先)

#### E-1. ゴールデン参照が「ファイル」でなく「実装と同居するコード」— ルール6(テスト不可侵)がパスで表現できない

- **場所**: リポジトリに参照PNGは**1枚も存在しない**(`**/*.png`ゼロ件)。参照は全てCPU参照実装: `crates/motolii-gpu/src/yuv.rs:381`の`yuv_to_rgba_reference`は**被試験実装と同一クレートのsrc内**。`crates/motolii-render/src/lib.rs:914-1558`のゴールデンテスト群は**src/lib.rsの`#[cfg(test)]`内**
- **顕在化**: specs/READMEルール6「ゴールデン改変を実装タスクに含めない」は、参照がsrcと同居している限り物理的に強制不能。保護は0%文書頼み
- **措置**(2段): ①【即日】CIステップ~20行: PRのdiffが`crates/*/tests/**`(または将来のgolden dir)と`crates/*/src/**`の**両方**に触れており`test-update`ラベルが無ければfail ②【M2前】src内ゴールデンテストと参照実装(`yuv_to_rgba_reference`、`expected_*`)を`tests/`または独立クレートへ移動してパスルールを有効化

#### E-2. 許容誤差がアドホック値 — 「閾値を1上げる」報酬ハックが検出不能

- **場所**: 呼び出し箇所ごとの生リテラル — `filter_node.rs:139,242,331`(0)、同391・`yuv_golden.rs:47`・`render/lib.rs:936,954,1083`(1)、`scripts/r9-verify.sh`は`R9_TOLERANCE:-8`。さらに`assert_rgba_close`(testkit lib.rs:243)は**maxのみ判定** — mean_abs_diffとdiffering_bytesは計算するが未アサート(「max=1だが全画素が1ずれ」の全体的な色ずれが合格する)
- **顕在化**: ルール6はテスト削除・期待値書換を禁じるが、**toleranceを1→4に上げる1文字diffは「テスト改変」に見えない**
- **措置**: testkitに`pub mod tol { pub const EXACT: u8 = 0; pub const GPU_RASTER: u8 = 1; }`を定義し、conformance.rsと同型のソース走査で「tolerance引数の生数値リテラル」をdeny。閾値変更をtestkitの1ファイル(=E-1のラベルゲート対象)に局在化。meanの上限もassertに追加

#### E-3. 決定性: lavapipe/ffmpeg/toolchainすべて無ピン、`--locked`なし

- **場所**: `.github/workflows/ci.yml:25`(`apt-get install -y mesa-vulkan-drivers ffmpeg` — ubuntu-latestの追随で黙って更新)、`rust-toolchain.toml`(`channel = "stable"`浮動)、ci.yml:34(`--locked`なし)
- **顕在化**: tolerance 0/1のゴールデンはlavapipeのラスタライズ変更1つで全赤化 → 並列エージェント全員が同時に「テストが壊れた」状態に落ち、閾値bump圧力(E-2)が最大化
- **確認済みの合格**: テスト内のrand/時刻依存は**なし** — 非決定性の源はこの環境ピン欠如のみ
- **措置**: `runs-on: ubuntu-24.04`固定+`cargo test --locked`+CIログに`dpkg -s mesa-vulkan-drivers`と`ffmpeg -version`を出力(1行×3)。M1実装ガードG8のlavapipeピン留めと同件

#### E-4. 凍結約束のうち「破ったら落ちるテスト」が無いもの(棚卸し)

| 凍結項目 | 現状 | ガード欠落 |
|---|---|---|
| FrameDesc | frame.rs 5テスト+コンパイラ | 契約の緩和(フィールド追加・意味変更)は無検出。semver-checksなし |
| レジストリdispatch | テスト済み | ○ |
| 正準座標 | filter_node.rsで複数解像度検証(良い) | 「絶対px引数の禁止」は無走査(H-1のdylintが将来扱い) |
| TimeMap try_map | 4テスト | ○(範囲は狭いが凍結範囲も狭い) |
| **単一writer** | DocumentWriter | **`Document`がpubフィールド+Clone+pubコンストラクタ**(doc/lib.rs:17-29)。任意クレートが`&mut Document`を自作可能。走査なし |
| param migrate | 実装+テスト1本+cli結線 | ○(sineのみのデモ)。Document自体の版マイグレーション関数は未存在(M2-D1待ち) |

- **措置**: conformance.rsの走査基盤を流用し「motolii-doc外での`&mut Document`トークン」をdenyする1テスト(30分仕事)。M2(全エージェントがDocumentを触る)の直前に最も効く

#### E-5. proptest不在 — M2直前の今が導入コスト最小点

- **確認**: Cargo.lockにproptest/quickcheck/arbitraryは**ゼロ件**。M2実装ガードはコマンドapply/revertのプロパティテストを要求予定
- **顕在化**: 各エージェントが独断でproptest/quickcheck/自作fuzzを選びパターン分裂(H-3の重複増殖と同根)
- **措置**: `[workspace.dependencies]`に`proptest = "1"`を追加し、**模範例1本**(RationalTimeの加減算roundtrip等)をmotolii-coreに置く(エージェントは既存パターンをコピーする性質を利用)

#### E-6. 重複参照ヘルパーが既に発生+参照が実装に部分依存

- **場所**: `expected_rect_frame`が`keyframe_export.rs:31`と`datatrack_export.rs:51`に**バイト単位で同一**のまま重複(AGENTS.mdが禁じるコピペ増殖の実例が凍結時点で既に樹内にある)。さらに両者とも`ViewportTransform::from_desc`(被試験の製品コード)で座標変換しており、**座標系バグは参照側にも同時に入る**(循環参照)
- **措置**: testkitに`cpu_reference`モジュールとして集約(E-1の移動と同時)。座標変換は参照側で独立式に書き下ろす

#### E-7. conformance.rsのpanic走査は回避可能(小)

- **場所**: `conformance.rs:186`は最初の`#[cfg(test)]`以降を免除 — ファイル先頭付近に`#[cfg(test)]`を置くと以降の製品コードが走査を逃れる
- **措置**: P-10と同件。`[lints]`でのdenyへ移行が本筋

#### E-8. CIの次の一手(費用対効果順・現状把握込み)

現状: fmt / clippy `-D warnings` / test / push+PR両トリガ(全ブランチ、二重実行は軽微)。欠落と優先順:
1. `MOTOLII_REQUIRE_GPU=1`+GPUカナリア(P-4③)— 数行、効果最大
2. テストパス×ラベルのdiffゲート(E-1①)— ~20行
3. `--locked`+runner/mesaピン(E-3)— 1行×3
4. cargo-semver-checks(凍結クレートへ、mainをbaseline)— H-1「凍結を機械契約に」の直実装
5. cargo-deny — 低優先(conformance.rsのベンダーdenyがF-9は既にカバー)
6. 仕様書整合チェック — 高コスト低精度、後回しで妥当

#### E-9. テスト速度は問題ではない(現状)

ホットラン1.2秒(GPU全スキップ時)。GPUテストも16×16〜64×64の小画像。**「遅くて検証を飛ばす」リスクより「速く全緑に見えて実は走っていない」リスク(P-4③)が支配的** — 速度投資よりskip可視化が先。

---

## Part 3: 健全確認済み(反証を試みて合格した項目 — 安心材料)

- **param安定ID**: 位置結合ゼロ。JSON→migrate→既知ID検査→desc順デフォルト充填→ID挿入(project.rs:194-219)まで一貫。migrateもIDベース
- **レジストリ**: 重複IDは同種別・跨種別ともtyped error(plugin lib.rs:421-437, 483-494)、`vendor.kind.name`命名+kindセグメント一致をregister時に強制(lib.rs:145-249)
- **CPUフレーム迂回路なし**: プラグイン境界はTextureRefのみ。ベンダーAPI/依存のdenyスキャンも負例つきで健在(conformance.rs)
- **予約の実在**: PluginKind::Simulation / ValueType::AssetRef / InstanceIndex / CompLookbehindが型として存在し、validateも予約種別を適切に素通し
- **時刻**: 有理数一貫、蓄積誤差経路なし。`as_seconds_f64`は表示用と明記(time.rs:37)、`segment_u`の区間内u設計はコメントで防御済み(track.rs:83-89)
- **スレッド準備**: RefCell/Rc/static mut/thread_localはワークスペース全域ゼロ(テスト用OnceLockのみ)。プラグインtraitはSend+Sync、GpuCtxはSend+Sync、`pollster::block_on`は起動時のみ — **M3レンダ専用スレッド+M4バックグラウンドuploadは現構造のまま成立する**
- **座標系の一点集約**: ViewportTransform(Y-flip含む)・YUV一点変換・premul変換関数の集約(frame.rs:156-172)はM1の約束通り健全
- **F-2骨格**: editは`&mut`漏洩なし、内部可変性なし、第二の変更経路なし(F-8の指摘は「D2への置換手順が未宣言」であって骨格の欠陥ではない)
- **Encoderの正常経路**: finish(mut self)消費でfinish後write_frameは型で防止済み。exportはエラー時もfinishを呼ぶ正しい形
- **テストの時刻/乱数依存**: なし(E-3の環境ピンが唯一の非決定源)

---

## Part 4: 提案チケット(1タスク=1PR粒度、優先順)

**凍結との関係**: PB-1(trait変更)とTM-1(export駆動反転)は凍結面に触れるため**解凍手続き(3点セット)対象**。他は凍結面の外(追加・強化のみ)。

### 最優先: プラグイン境界(Part 1)

| ID | 内容 | 対応所見 | 規模 |
|---|---|---|---|
| PB-1 | `RenderCtx`導入(Filter/Composite trait改訂)**【解凍手続き】** | P-1 | 中 |
| PB-2 | 型付きparamアクセサ+`resolve_params`一元化+ロード時型検証 | P-2, P-3 | 小〜中 |
| PB-3 | `PluginRegistry::iter`+`assert_registry_pure`+LayerSource/Composite purityヘルパー | P-4①② | 小 |
| PB-4 | CI: `MOTOLII_REQUIRE_GPU=1`+GPUカナリア | P-4③ | 極小 |
| PB-5 | new-pluginスキャフォールド拡充(purity/ゴールデン/ParamDef例) | P-5 | 小 |
| PB-6 | `NodeDesc::migrations`宣言化+descスナップショット比較テスト | P-6, P-7 | 中 |
| PB-7 | `PipelineCacheKey`にformat+「1 submit 1インスタンス」契約明文化 | P-8, C-2 | 小 |
| PB-8 | category統制語彙+panic走査全クレート化+`ViewportTransform::new` Result化 | P-9, P-10, E-7 | 小 |
| PB-9 | 未知ID挙動の層別統一(ロード=警告+保持/評価=typed error)の宣言 | P-11 | 仕様1文+M2-D1 |
| PB-10 | `PluginId`コンストラクタ化+`ResolvedParams`キーCow化 | P-12 | 小 |

### 高: 構造の反転(先駆者の解答の適用、今が最安)

| ID | 内容 | 対応所見 | 規模 |
|---|---|---|---|
| TM-1 | exportループの時刻駆動化+非恒等TimeMap×export拒否**【解凍手続き】** | T-1 | 小〜中 |
| GR-1 | RenderSession中間テクスチャをrefcount付きacquire/releaseプールへ(K1形状の前倒し) | G-1 | 中 |
| SC-1 | M2仕様へ宣言5点: D1はProjectV1非継承 / Document≠ExportJob / フレーム添字禁止 / bpm有理数 / ExportOverlayRequest廃止予定 | F-1, F-5, F-6, F-11, F-10 | 仕様のみ |

### 中: 口の確保(数行〜数十行)

| ID | 内容 | 対応所見 |
|---|---|---|
| TM-2 | RationalTime serde try_from正規化 | T-2 |
| TM-3 | duration半開区間規約宣言+`+1`修正 | T-3 |
| TM-4 | `to_frame_round`集約+f64変換置換 | T-4, T-11 |
| TM-5 | TimeMap speed正規化 | T-7 |
| TM-6 | `MediaInfo.duration_raw`並置 | T-8 |
| TM-7 | Value正準ハッシュ契約+区間クエリのシグネチャ宣言 | T-6 |
| GR-2 | GpuCtx起源タグ(Headless/UiShared)+poll(Wait)型ガード | G-2 |
| GR-3 | `GpuCtx::create_texture`一本化(VRAM計上フックの席) | G-4 |
| GR-4 | uniform永続buffer化+1 encoder統合 | G-5 |
| GR-5 | 毎フレーム`check_health`+デバイスコールバック所有者規約(M3仕様追記) | G-6 |
| GR-6 | ffmpeg stderrドレイン(M1実装ガードG1の実装) | G-7 |
| GR-7 | FrameReaderキャンセルトークン+killハンドル分離 | G-3 |
| SC-2 | Document: `flatten extra`+`min_reader_version`+`revision`+editコメント | F-2, F-3, F-8, F-9 |
| SC-3 | `LayerId` newtype予約+ID規約宣言 | F-7 |
| CQ-1 | スキーマ色の定義確定(sRGB/straight/0-1)+`precise_color`配線+v1暫定マーカー | F-4, C-1 |
| CQ-2 | Rec709正直タグ+変換関数の経路確保 | C-4 |
| CQ-3 | Canonical型のcore移動+re-export | C-3 |
| CQ-4 | Quality丸め規則確定+正準aspect=Final基準の明文化 | C-6 |
| CQ-5 | CompCamera単位のdoc宣言+`RenderGraphInputs.camera`口 | C-7, C-5 |
| CQ-6 | FrameDescにPAR追加(または構築関数経由の徹底) | C-8 |
| CQ-7 | straight→premul単一アダプタ(Vello受け皿) | C-10 |
| CQ-8 | validate_render_desc対応表化 | C-11 |
| EN-1 | ゴールデン参照の分離+diffゲートCI | E-1, E-6 |
| EN-2 | tolerance定数化+リテラルdeny走査+mean assert | E-2 |
| EN-3 | CIピン留め(`--locked`/runner/mesaログ) | E-3 |
| EN-4 | `&mut Document`走査テスト | E-4 |
| EN-5 | proptest導入+模範例1本 | E-5 |

### 低(規約・タグ付けのみ)

| ID | 内容 | 対応所見 |
|---|---|---|
| LG-1 | overlay形状ゴールデンの「Vello置換時regenerate」タグ+増殖凍結規約 | C-9 |
| LG-2 | Encoder Drop警告+YuvToRgba保持契約のM4仕様転記+render_frame入口注記 | G-8, G-9, G-10 |
| LG-3 | 評価時刻の意味論(v1=ソースPTS縮退)の仕様明文化 | T-9 |
