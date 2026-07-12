# M2入場条件(2026-07-11。同日改訂: ゲート運用レビュー7点を反映)

ステータス: **未達**(全項目達成で、Documentスキーマに触るタスクの並列発注を解禁する)

## これは何か

凍結ゲート(2026-07-10宣言)が「**何を凍結してから並列化するか**」だったのに対し、これは「**何を塞いでからM2に入るか**」のエントリーゲート。M2が怖い理由は3つの重なりに名前が付く — ①**恒久性**(Documentスキーマとジャーナルはユーザーデータとして永続化され、以後の間違いは全部マイグレーション負債になる) ②**並列化の初陣**(凍結ゲート後、複数エージェントが同時にコードを積む最初のフェーズ) ③**検証の弱さ**(M2のバグはピクセルに出ず、数週間後の破損プロジェクトとして出る — その検出基盤が未整備)。

**性格**: 本ゲートは既存所見の翻訳を基礎とし、**新規リスクの追加はない**。全項目が既存文書へ逆引きできる([実コード監査](2026-07-11-code-audit-pre-m2.md)のチケットPB/EN/SC/CQ/TM系、[M2仕様の実装ガード](../specs/M2-document-model.md)、[pitfalls H群](../pitfalls-and-roadmap.md))。ただし**翻訳の過程で決定を要した箇所がある** — 実装境界(M2E-12)・保護対象の選定(M2E-2)・型配置(M2E-15)・色契約(M2E-13)・時刻の許容範囲(M2E-16)・互換性の扱い(M2E-17)。これらは【決定】タグで明示し、決定内容と根拠を項目内に書く(H-3: エージェントの裁量で埋めさせない)。

## ゲートが止める対象(過剰包摂の防止)

本ゲートが発注を止めるのは **Documentスキーマ/ジャーナルに触るタスク: D1・D2・D3・D7・D8**(D5はD3/D4依存のため推移的に後)。
**D4(音声デコード)・D6(書き出しmux)は対象外** — Documentスキーマから独立しており(依存は凍結ゲートのみ)、本ゲートの根拠(恒久スキーマの保護)が当てはまらないため、凍結ゲート達成のみで着手できる。
ただしD6のうち**M2実装ガード9(欠落プラグイン時の最終書き出し拒否)はD1との結合部分**なので、D6本体(基本mux)は先行可・このD1連携だけ後続とする(誤発注防止の注記)。

## 記入規則(R1追補の教訓)

- 完了したらチェックを入れ、**項目にコミットSHA(またはPR番号)とテスト名を併記する**(証跡なしの[x]を作らない — [R1追補](2026-07-09-R1-export-review.md)記録上の問題2)
- 各項目は1タスク=1PR粒度。着手前に対象所見のfile:lineを**最新mainで再確認**する(監査はf020ec8基準。LLM監査の出力は採用前に現物確認 — 監査レポート冒頭の検証注記)
- 完了条件の判定方式を項目ごとに明示する: **[自動]**=CI/テストで機械判定、**[レビュー]**=文書マージ・人間承認で判定(存在確認しかできない項目を「自動判定」と偽らない)
- 完了条件を満たせない事情が判明したら、黙って緩めず本ファイルを改訂してから進める(H-3)

## A. 審判を起こす(テスト施行層 — 並列化の前提)

並列エージェントの安全モデルは「cargo test全緑=正」だが、現状その審判自体に穴がある。**塞ぐ前に並列化すると、穴を通ったコードが山積みになってから気づく。**

| ID | 内容 | 出典 | 完了条件 | 規模 |
|---|---|---|---|---|
| [x] M2E-1 **(PR #33)** | CIに`MOTOLII_REQUIRE_GPU=1`を設定し、`gpu_or_skip`はこれが立っていればスキップでなくpanicする変換ロジックを実装。**証跡**: `skip_decision`/`apply_skip_decision`/`unavailable_dep`(純関数)+`ffmpeg_or_skip`/`tool_status`(未導入と実行失敗を区別)+`tests/skip_policy.rs`(判定行列4象限・`apply_forbid_panics`=should_panic負例・`ci_canary_gpu_and_ffmpeg_present`・**`no_hand_rolled_skip_paths_outside_testkit`=手書きスキップの走査deny**)。レビューで手書きスキップ12箇所の迂回を検出し全てポリシー経由に置換(走査denyが手動列挙の漏れ5箇所を実際に検出)。GPU/ffmpeg無し環境で変数を立てると実際に赤化することを実地確認済み | 監査PB-4(P-4③実測: GPU無し環境で141テスト全緑・1.2秒) | [自動] 通常CIジョブはREQUIRE_GPU=1で全緑。**負例は通常ジョブの環境欠損に依存させない**: (a)skip→panic変換ロジック自体の単体テスト(依存の有無を注入可能な形に分離)、または(b)依存を意図的に無効化した専用CIジョブがexpected failureになることの確認、のどちらかで判定 | 極小〜小 |
| [x] M2E-2 **(PR #35 / ruleset 18817145 / 実地 #42)** | 【決定】テスト資産の不可侵化。**保護対象を限定列挙する**: ①既存ゴールデン参照画像 ②既存受け入れテストの期待値(CPU参照実装=`yuv_to_rgba_reference`・`expected_*`系) ③tolerance定数(`testkit::tol`)。解除は実装者が自己完結できない形: CODEOWNERS+rulesetでコードオーナーレビュー必須+「テスト更新専用PR(保護領域のみ)」のみ通常マージ可。<br>**証跡**: コード=#35(`golden/**`+`cpu_reference/**`+`tol/**`+CODEOWNERS自己保護+`acceptance_oracles_live_only_in_protected_area`+`protected_diff_gate_*`+`scripts/check-protected-diff.sh`)。ruleset=`M2E-2 require code owner review` id=18817145(`require_code_owner_review=true`、Admin `bypass_mode=pull_request`)。実地=#42 で `reviewDecision=REVIEW_REQUIRED` / `mergeStateStatus=BLOCKED` を観測しクローズ。**履歴**: 2026-07-12 Cursor agent が `gh api` でruleset作成→試験PR→証跡記入(人間の自己クリック承認ループは使わない。単独Ownerはadmin bypassを例外手順として明文化)。 | 監査EN-1/EN-2の統合(E-1: 参照が被試験クレートのsrc内で保護不能。E-6: 参照ヘルパーの重複+製品コード依存の循環。改訂: 当初案の「テスト領域×src同時変更ラベルゲート」はTDDを一律例外化し、ラベルは実装者が自由に付けられるため機械化にならない — 保護対象の限定と解除権限の分離に変更) | [自動/設定確認] **承認強制とdiffゲートを別々に検証する**: ①保護領域への変更はCODEOWNERSでコードオーナーレビュー必須とし、ruleset/branch protectionにより承認なしではマージ不能(設定確認 — CODEOWNERS単体はCIを赤くしない) ②保護領域と**非保護パス**の同時変更を専用diffチェックでCI failにする(自動・負例込み) ③参照ヘルパーの重複が解消され集約先に1箇所(自動) | 小〜中 |
| [x] M2E-3 **(PR #40)** | CIの決定性ピン留め: runner固定(`ubuntu-24.04`)+`cargo test --locked --workspace`+mesa/ffmpegバージョンをCIログへ出力。<br>**証跡**: `runs-on: ubuntu-24.04` / Test=`cargo test --locked --workspace` / step `Log mesa/ffmpeg versions`(`ffmpeg -version`+`dpkg -s mesa-vulkan-drivers`)。判定は[レビュー]の存在確認 | 監査EN-3(E-3: 全て無ピン。lavapipe更新1つでtolerance 0/1のゴールデンが全赤→閾値bump圧力が最大化) | [レビュー] ci.ymlに3点が入っていることの確認(これは存在確認であり不変条件の証明ではない — ドリフト発生時の一次切り分けを可能にするのが目的) | 極小 |
| [x] M2E-4 **(PR #36 / #38)** | toleranceの定数化(`testkit::tol`)+呼び出し箇所の生数値リテラルをdenyする走査テスト+mean上限のassert追加。<br>**証跡**: #36=`tol::{GPU_RASTER_MEAN,mean_limit}`(保護のみ)。#38=`assert_rgba_close*`のmax+mean判定+全呼び出しを`tol::EXACT`/`GPU_RASTER`経由+`tests/tol_literals.rs`(`fixture_literal_tolerance_is_detected`/`fixture_bypass_forms_are_detected`/`fixture_definition_forwarding_tolerance_is_allowed`/`fixture_tol_constants_are_allowed`/`no_disallowed_tolerance_in_workspace_sources`)+mean負例`assert_close_rejects_uniform_shift_via_mean_limit`。走査は許可定数と`assert_rgba_close`定義内`tolerance`転送以外をすべて拒否 | 監査EN-2(E-2: 「閾値を1上げる」1文字diffはテスト改変に見えない=ルール6の死角) | [自動] リテラルtoleranceを書いたテストが走査で赤(負例込み)。既存呼び出しが定数経由。`tol`定数はM2E-2の保護領域に置く | 小 |
| [x] M2E-5 **(PR #47)** | 「motolii-doc外での`&mut Document`」をdenyする走査テスト(conformance.rsの基盤流用)。<br>**証跡**: `crates/motolii-doc/tests/mut_document_deny.rs` — 負例`fixture_same_line_mut_document_is_detected`/`fixture_linebreak_mut_document_is_detected`/`fixture_lifetime_mut_document_is_detected`/`fixture_path_qualified_mut_document_is_detected`、誤検出回避`fixture_comments_are_not_detected`/`fixture_strings_are_not_detected`/`fixture_identifier_boundary_avoids_false_positive`、実ツリー`no_mut_document_outside_motolii_doc`(motolii-doc内番兵つき)。コメント・文字列をマスクし、`&mut`改行分割とパス修飾(`motolii_doc::Document`)を検出 | 監査EN-4(E-4: `Document`はpubフィールド+Clone+pubコンストラクタで、単一writerは現状すり抜け可能) | [自動] 負例(テスト内のダミー違反)が赤になることを含めて緑 | 小 |
| [x] M2E-6 **(PR #60)** | proptestをworkspace依存に追加し、模範例1本(RationalTime roundtrip等)をmotolii-coreに置く。<br>**証跡**: `Cargo.toml` `[workspace.dependencies] proptest = "1"` + `motolii-core` dev-dep + `tests/proptest_example.rs`(`rational_time_add_sub_roundtrip`)。以降のプロパティテストはこの依存と型紙をコピーする | 監査EN-5(E-5: M2実装ガード12はプロパティテスト前提だが依存がゼロ件。各エージェントが独断選定すると分裂=H-3) | [自動] 模範例がCIで緑。workspace.dependenciesに追加済み | 極小 |

## B. 乗算する穴を塞ぐ(プラグイン境界 — M2でプラグイン量産が始まる前に)

プラグイン境界のエラーは費用がプラグイン数で乗算される(監査Part 1)。**参照プラグイン数個の今が唯一の安値点。**

| ID | 内容 | 出典 | 完了条件 | 規模 |
|---|---|---|---|---|
| [x] M2E-7 **(PR #62 / #64)** | `RenderCtx`導入(Filter/Composite traitの裸引数を`#[non_exhaustive]`構造体へ)。Quality/将来の予約(InstanceIndex/CompLookbehind/TemporalFootprint)の口を確保。**解凍手続き(3点セット)対象**<br>**証跡**: #62=`RenderCtx`/`TemporalFootprint`+Filter/Composite改訂+`dispatch_plugin`が`RenderCtx::new(t, quality)`渡し+参照プラグイン/scaffold/purity追随。解凍=[2026-07-12-M2E-7-render-ctx-thaw.md](2026-07-12-M2E-7-render-ctx-thaw.md)。#64=`FilterNode::render(&RenderCtx)`転送+`filter_node_forwards_draft_quality_in_render_ctx`/`plugin_dispatch_forwards_draft_quality_in_render_ctx`+scaffoldの未使用`RationalTime`除去。テスト`render_ctx_carries_quality_and_reserved_defaults`+既存ゴールデン不変 | 監査PB-1(P-1: 引数追加=全プラグイン破壊。render_graph_cachedがQualityを解像度に畳み込みDraft/Final判別不能) | [自動] 全参照プラグインがRenderCtx経由で既存ゴールデン不変。[レビュー] 解凍手続き文書化 | 中 |
| [x] M2E-8 **(PR #66)** | 型付きparamアクセサ(`require_f64`等+`PluginError::Param`)+`NodeDesc::resolve_params`一元化(未知ID→Err/欠落→default/型不一致→Err)+ロード時のvalue_type検証<br>**証跡**: #66=`PluginError::Param`/`require_*`/`NodeDesc::resolve_params`+参照から`.f64_or(`除去+`load_project_v1_from_str`でmigrate→resolve_params+`load_project_rejects_param_type_mismatch`/`resolve_params_fills_defaults_and_rejects_unknown_or_mismatch`/`reference_impl_does_not_call_silent_f64_fallback`+`plugin-authoring.md`で`require_*`推奨 | 監査PB-2/PB-3(P-2: `f64_or`のサイレントデフォルト=「もっともらしく間違う絵」。P-3: 解決ロジックがproject.rsに手書き) | [自動] 型不一致JSONがロード時に構造化エラー。`f64_or`が参照実装から消えている | 小〜中 |
| [x] M2E-9 **(PR #68 / #70)** | `PluginRegistry::iter(kind)`+`assert_registry_pure`(登録済み全プラグインへvalidate+purity一括適用)+LayerSource/Composite用purityヘルパー<br>**証跡**: #68=`PluginRegistry::iter`/`DynPlugin`+`assert_layer_source_pure`/`assert_composite_pure`/`assert_registry_pure`+`reference_registry_is_pure`/`registering_stateful_plugin_fails_registry_purity`+既存個別4本維持。<br>**#70(追補P1)**: GPU系purityをrenderごと別`queue.submit`に分割+`shared_uniform_stateful_filter_fails_purity_check`(共有uniform/`write_buffer`偽陰性の負例)。レビュー記録=[#70 review](https://github.com/oshikaidesu/Motolii/pull/70#pullrequestreview-4679136496)(単独OwnerのためCOMMENTED、M2E-2手順でadmin merge) | 監査PB-3(P-4①②: purityが手書き列挙のopt-in。登録=検査対象への反転) | [自動] レジストリに登録するだけで検査対象になることをテストで確認(未検査プラグインを作れない)。共有uniform状態付きFilterがpurityで赤 | 小 |
| [x] M2E-10 **(PR #72)** | new-pluginスキャフォールドにpurityスタブ+ゴールデンスタブ+ParamDef例を同梱<br>**証跡**: #72=`scripts/new_plugin.py`が製品(`use crate::`+ParamDef+validate)とtestkitテスト(purity+fail-closedゴールデン)を別成果物生成+`generated_artifacts_compile_in_self_crate_layout`(`MOTOLII_SCAFFOLD_FIXTURE`独自cfg+OUT_DIR、`--locked`、`--all-features`非依存)+`generator_writes_separate_plugin_and_test_artifacts`+`plugin-authoring.md`追随。follow-up=`40fbce4`(clippy expect_used allow)/`7dcae21`(rustfmt欠落path回避)。merge=`a3facc0` | 監査PB-5(P-5: LLMは型紙に無いものは書かない。purity普及率≒スキャフォールド同梱率) | [自動] 生成物にテスト3種が含まれ、scaffold検証テストがそれを確認 | 小 |

## C. D1が継承する罠を断つ(スキーマ/型の宣言と骨格 — D1発注の前提)

大半が「仕様への1文」または数行。**放置した場合だけ恒久マイグレーション負債になる**種類(監査2-C総括)。

| ID | 内容 | 出典 | 完了条件 | 規模 |
|---|---|---|---|---|
| [x] M2E-11 **(PR #74)** | M2仕様へ宣言5点を追記: ①D1はProjectV1を継承も移行もしない(version採番も独立) ②Document≠ExportJob(出力パス/エンコード設定は別構造) ③クリップin/out/durationは`RationalTime`(フレーム添字をスキーマに入れない) ④bpmは有理数(拍時刻がRationalTimeに畳める) ⑤ExportOverlayRequestはD3でDocument→render直結に置換(ジョブミラー温存禁止)<br>**証跡**: #74=`docs/specs/M2-document-model.md`「スキーマ境界の宣言(M2E-11)」節+ D1/D3行への織り込み | 監査SC-1(F-1: 両者が「v1」を名乗り関係未宣言 — エージェントはProjectV1増築を自然に選ぶ。F-5/F-6/F-11/F-10) | [レビュー] M2仕様の改訂がマージ済み(文書項目であり自動判定は不可能 — D1発注時の仕様書に含まれることが効果の実体) | 仕様のみ |
| [x] M2E-12 **(PR #75)** | 【決定】**D1-prelude**(D1から正式に切り出した骨格 — 境界の循環を解消): 予約4点=`#[serde(flatten)] extra`(unknown-keys保持)+`min_reader_version`+`DocumentWriter.revision: u64`+editへ「D2でapply(Command)に置換、呼び出し追加禁止」doc-comment。**preludeはversion/互換の枠と所有権骨格のみで、トラック/クリップ等のスキーマ本体を一切含まない**(本体=D1はゲート達成後)。M2仕様D1にもprelude切り出しを注記する<br>**証跡**: #75=`Document::{min_reader_version,extra}`+`DocumentWriter::revision`(edit/applyで加算)+edit/applyのD2置換doc-comment+`tests/unknown_keys_roundtrip.rs`(`unknown_keys_survive_json_roundtrip`/`unknown_keys_absent_yields_empty_extra`)。スキーマ本体(トラック/クリップ等)なし | 監査SC-2(F-2/F-3/F-8/F-9。改訂: 当初案は「D1発注前にD1の一部を実装する」循環だった — D1を骨格(prelude)と本体に分割し、前者を入場条件、後者をゲート対象と定義し直す) | [自動] unknown-keys roundtripテスト緑。[レビュー] preludeがスキーマ本体を含まないことのdiff確認 | 小 |
| [x] M2E-13 **(PR #77)** | 【決定】**色契約の決定(配線は含まない — M2E-18へ分離)**。決定済みの3層の区別を仕様へ明文化する: (1)**永続スキーマ上のColorの意味** — 採用決定: straight-alpha・非線形sRGB・各成分0-1(現実装の実態と一致) (2)**レンダ中間表現との区別** — 保存値とレンダ中間(premultiplied、将来はlinear)を混同しない。レンダ層への入力時に必要な変換を行うのはレンダ側の責務 (3)**合成空間はこの決定と別問題** — v1の合成はsRGB空間ブレンド(暫定、ゴールデン焼き込み済み=監査C-1)であり、「保存値がsRGB」から「sRGBでブレンドする」は帰結しない。linear premultiplied合成への移行はprecise_color配線(M2E-18)の先にある将来判断。value.rs:9の「リニア」コメントを決定に整合させる<br>**証跡**: #77=`docs/specs/M2-document-model.md`「色契約の宣言(M2E-13)」節+D1行織り込み+`Value::Color`/`ParamColorV1`/Overlay色/`ColorSpace::LinearRgb`コメント整合 | 監査CQ-1/F-4(コメント間で既に矛盾。キーフレーム済みカラー量産後の解釈変更は「マイグレーション不能な種類の破壊」。改訂: 当初案は①スキーマ意味論②コメント是正③precise_color配線の3論点を1項目に混載しており、①②=スキーマゲート/③=レンダ契約実装を分離) | [レビュー] 3層の区別が仕様(またはconcept)に明文化されマージ済み。[自動] コメント整合はdocテスト等では判定不能のためdiffレビューで確認 | 仕様+コメント |
| [x] M2E-14 **(PR #78)** | `CanonicalPoint`/`CanonicalSize`/`ViewportTransform`等をmotolii-coreへ移動(nodesはre-export)<br>**証跡**: #78=`motolii-core::canonical`+`ViewportTransformError`(ゼロ寸法はResult拒否)+nodes `pub use`/`NodeError::Viewport`+`motolii-doc/tests/canonical_from_core.rs`(`doc_crate_can_use_canonical_types_from_core`)+`rejects_zero_width`/`rejects_zero_height`/`rejects_zero_dimension_frame_desc` | 監査CQ-3(C-3: motolii-docはnodesに依存しない — 空間パラメータを持つ最初のM2エージェントが参照できる正準型が無く、独自表現を発明する) | [自動] 移動後全テスト緑。docクレートからCanonical型が参照可能(コンパイルが証明) | 小 |
| [ ] M2E-15 | 【決定】`LayerId` newtype予約。**配置はmotolii-doc所有と決定する** — 根拠: LayerIdはドキュメント内の恒久IDであり、シェイプ間リンク(LookAt/Follow/ParentRef)の参照解決はD3(doc→グラフ変換)で行い、**eval/プラグイン契約にはLayerIdを露出しない**(変換時に解決済みの具体参照へ落とす)。将来`Value::LayerRef`等でプラグイン境界に出す必要が生じたら、その時点で解凍手続きを通してcoreへの移動を判断する(依存方向: doc→eval→coreのため、evalが必要とするならcore行きになる)。「表示名はIDと別フィールド・ID再利用禁止」も仕様宣言 | 監査SC-3(F-7。改訂: 当初案の「coreまたはdocに存在」は配置をエージェント裁量に残しており、H-3で禁止した状態そのものだった — ゲート内で確定) | [自動] 型がmotolii-docに存在し、**IDのserde roundtripテスト・一意性(重複挿入拒否)テスト・再利用禁止(削除後の再割当が起きない採番)テスト**が緑 | 小 |
| [ ] M2E-16 | 【決定】時刻型のserde不変条件を厳密に定義して実装: **(a)`den == 0`は拒否(エラー) (b)分母は常に正(負の分母は符号を分子へ移して正規化 — 負の時刻・負の分子そのものは正当な値として保持する) (c)`gcd(abs(num), den) == 1`へ既約化 (d)`0/x`は`0/1`へ正規化 (e)`i64::MIN`の符号反転等のオーバーフローはchecked演算でエラーにする(panicさせない) (f)`TimeMap.speed`のM2で許す範囲を確定: `speed_num > 0`のみ受理(ゼロ拒否。負=逆再生はスキーマ表現としては将来拡張の席を維持しつつ、現段階はvalidateで明示拒否 — Issue #18の明示拒否路線と整合)**。`Fps`も同様(正のみ) | 監査TM-2/TM-5(T-2: JSONからdiv-by-zero panic注入可能、Hash/Ord前提が静かに壊れる — **ジャーナルの生命線**。[R1追補 追-1](2026-07-09-R1-export-review.md)と同件、a3a05d5でも現存確認済み。T-7: 同値写像が別ハッシュ。改訂: 当初案の「負・非既約は正規化」は「負の分母」と「負の時刻」を区別しておらず曖昧だった) | [自動] (a)〜(f)それぞれに対応する不正JSONがロード時エラー(または正規化)になるproptest+単体テスト緑。i64::MIN境界のテストを含む | 小 |
| [ ] M2E-17 | 【決定】duration/区間規約の確定: 「duration=総尺、区間は半開`[start, start+duration)`」を仕様宣言し、`export_frame_count`の`+1`と`ParamDriverContext`のフェンスポストを規約に合わせる。**互換性の扱いを明記する**: この変更でProjectV1の書き出しフレーム数が1減る(90フレーム素材で91→90)。ProjectV1は使い捨て(M2E-11①)でありユーザーデータの互換対象外、既存テスト(`from_frame(89)`系)の更新は「テスト更新PR」(M2E-2の手続き)として理由付きで行う — 規約変更に伴う正当な期待値更新であり、黙った書き換えと区別する | 監査TM-3(T-3: 2流儀混在+オフバイワンがテストに焼き込み済み。M4区間キャッシュとM2音声終端の前提。改訂: 既存挙動の変更を伴う点を互換性の決定として明示) | [自動] 規約に対応する境界テスト(総尺ちょうどのフレームが範囲外であること等)が緑。[レビュー] 仕様の規約明文化 | 小 |
| [ ] M2E-18 | `precise_color`の配線(口のみ): `Quality.precise_color`が`render_desc`/合成シェーダ選択の分岐点まで届く経路を作る(v1実装は恒等=sRGBブレンドのままでよい)。M2E-13(3)の「linear premultiplied合成への将来移行」の受け皿 | 監査C-1(precise_color未使用。M2期間にsRGBブレンド依存ゴールデンを増やさないための口) | [自動] precise_colorの値が合成経路の分岐点に到達することのテスト(恒等実装でも分岐点の存在をassertできる形にする) | 小 |

## 入場条件に含めない(M2期間中・M4前でよいと判断したもの)

| 項目 | 理由 |
|---|---|
| PB-6(migrations宣言化)/ PB-7〜PB-12 | マージ衝突・タクソノミー・&'staticはプラグイン数が実際に増え始めてからで間に合う。M2期間中の並走チケット |
| GR-1(refcountプール) | G-1の出力エイリアスはIssue #16で対処済み(専有出力+生存回避)。プール化はM4-K1の形の話であり、**M4入場条件の候補**。ただしM2-D3(木構造グラフ)着手時に生存回避のスケールを再評価する |
| TM-1(exportループの時刻駆動化) | 暫定拒否はIssue #18で実装済み。ループ反転はM2の実デコード再写像タスク**そのもの**であり、入場条件ではなくM2の作業 |
| GR-2〜GR-7 / CQ-2/CQ-4〜CQ-8 / LG群 | M3/M4との衝突回避が主目的。各フェーズの入場条件または期間中チケットへ(監査Part 4参照) |

## 宣言方針

1. 消化順: **M2E-1(審判の覚醒)→ M2E-2/M2E-4(テスト資産の保護)→ M2E-3(ピン留め)** → 残りのA群 → B群 → C群。M2E-3より先にM2E-1/2/4なのは、CIを固定しても審判自体がskipや自己参照で抜けられる状態では効果が限定されるため。**A群(M2E-1〜6)は完了**。**B群(M2E-7〜10)は完了**。**C群は M2E-11〜M2E-14 完了、次は M2E-15**
2. 全項目チェック+証跡が揃ったら、本ファイル冒頭のステータスを「**達成**」に書き換えて**D1/D2/D3/D7/D8の並列発注を解禁**する(D4/D6は本ゲートの対象外 — 上記「ゲートが止める対象」)
3. M2仕様([specs/M2-document-model.md](../specs/M2-document-model.md))は、ステータス行の着手条件に加えて**D1行の依存欄に本ゲートを明記**する(タスク表だけを見て発注するエージェントが素通りできないように)
