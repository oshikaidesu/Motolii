# VSM-A3S — 一般 LayerSource lowering 仕様

作成日: 2026-07-18

状態: **仕様完了／コード変更なし**。本書は [VSM-A3D §11](2026-07-18-vism-a3d-radial-repeater-decision.md#11-a3s-handoff-質問未決のまま残す) の 9 質問をすべて閉じ、一般 `LayerSource` を **ID allowlist なし**で prepared recipe → runtime `RenderStep::Plugin` へ下ろす契約を固定する。実装は後続 VSM-A3 分割発注へ委ねる。

関連正本: [VSM-A3D Radial Repeater 採用](2026-07-18-vism-a3d-radial-repeater-decision.md)、[VSM-A0S Contract Catalog](2026-07-17-vism-a0s-contract-catalog-spec.md)、[VSM-A1S 公開 crate 境界](2026-07-17-vism-a1-public-crate-boundary-spec.md)、[VSM-A3R 調査](2026-07-18-vism-a3-external-expression-survey.md)、[Vism実装計画](2026-07-17-vism-implementation-plan.md)

ORDER: READY（文書のみ。`cargo test` は本発注の完了条件にしない）

---

## 1. 結論

| 論点 | 決定 |
|---|---|
| 公開 signature | `build_document_frame_graph`とplugin traitは変更しない。公開API面は、panicを避ける加算的`GraphError::PreparedLayerSourceMissing { layer: u64 }`だけ拡張する |
| 一般 LayerSource lowering | `GraphBuilder` が `PreparedDocumentPlugins` を保持し、prepared recipe だけを評価して `RenderStep::Plugin` へ下ろす |
| 旧 `core.layer_source.clear` 特例 | **一般化**。一時互換 allowlist は置かない |
| `CLEAR_LAYER_SOURCE` 定数 | graph 分岐条件としての使用を廃止。定数自体の削除要否は A3 実装 ticket の最小 diff に委ねる |
| 型付き拒否 | A0S `PluginDiagnosticReason`／`DocumentPluginError` を LayerSource にそのまま適用。新 variant は本仕様で発明しない |
| raw Document | catalog-backed LayerSourceは**prepared.params**だけを評価。built-in rectのraw param経路はA0Sどおり分離 |
| `doc.layer_source.rect` | catalog 外・built-in 分岐のみ（A0S 維持） |
| 画素契約 | union SDF＋1 回 coverage AA。Draft／Final は同一 `LayerSourcePlugin::render`（`Quality` 無し） |
| first-party 外部 crate | A1S 型紙踏襲。Host 必須 capability へ勝手に足さない |
| U4a 投影 model | 現行境界で足りる。A3／A3S で API を発明しない |

**現状 GAP（コード事実）**: `motolii-doc/src/graph.rs::build_source` は `CLEAR_LAYER_SOURCE` だけを `RenderStep::Plugin` へ下ろし、他は `UnsupportedSourcePlugin` とする。`build_document_frame_graph` は `prepare_plugins`＋診断を行うが、**prepared recipe を `GraphBuilder` に渡さず** raw `clip.params` を `resolve_plugin_params` で評価している。不足は公開 signature ではなく graph 内部の prepared 参照欠落である。

---

## 2. 公開／内部契約（Q1）

### 2.1 公開入口（変更なし）

```rust
pub fn build_document_frame_graph(
    doc: &Document,
    eval: EvaluationTime,
    desc: FrameDesc,
    data_tracks: &DataTracks,
    runtime: &PluginRuntime,
    project_root: Option<&Path>,
) -> Result<DocumentFrameGraph, GraphError>;
```

A0I-3 で graph／export は既に `PluginRuntime`（catalog＋executors）必須。本仕様はこの signature を維持する。

### 2.2 内部フロー（擬似・公開関数／trait追加なし）

```text
build_document_frame_graph
  → prepare_plugins(catalog) + execution_diagnostics(runtime)
  → diagnostics 空でなければ GraphError::PluginDiagnostics（既存）
  → GraphBuilder が PreparedDocumentPlugins を保持（内部）
  → ClipSource::Plugin:
       plugin_id == doc.layer_source.rect → 既存 build_rect_overlay（catalog 外）
       それ以外 → 一般 LayerSource lowering（下記）
```

### 2.3 一般 LayerSource lowering

1. `prepared.get(PluginSlotId::LayerSource(layer))` 必須。無い／診断付きは通常到達不能だが、公開 API で panic しないため、最小の `GraphError::PreparedLayerSourceMissing { layer: u64 }` を A3 実装 ticket で追加する。これは診断済み contract 欠落を偽装せず、prepare と graph traversal の内部不変違反を表す 1 variant に限る。類似 variant や文字列 error を増やさない。
2. recipe の `plugin_id` で既存
   `runtime.executors().layer_source_by_name(...)` を解決し、executorの
   `desc().id`を`RenderStep::Plugin`へ渡す（**文字列ID定数のallowlist禁止**）。
   事前の`execution_diagnostics`後に欠落した場合は、同じ
   `PluginSlotId`／`plugin_id`の`ExecutorMissing`を
   `GraphError::PluginDiagnostics`で返し、別の文字列errorを増やさない。
3. catalog-backed LayerSourceは`recipe.params`（prepared）だけを
   `eval_doc_param` → `NodeDesc::resolve_params` で評価する（raw
   `clip.params`を読まない）。catalog外built-in rectはこの一般経路へ入れない。
4. `RenderStep::Plugin { id, params, inputs: [], output }` を push する。
5. `min_inputs = max_inputs = 0` 以外の LayerSource contract は本 A3S の一般経路対象外である（A3D v1 は 0-input）。多入力が必要なら STOP → 仕様改訂。

### 2.4 公開 trait（変更なし）

```rust
LayerSourcePlugin::render(..., t, params, LayerSourceContext { camera }, output)
```

`LayerSourceContext` に `Quality` は無い。Filter 用 `RenderCtx.quality` とは別経路である。

---

## 3. 旧 `clear` 特例の処分（Q2）

**決定: 一般化。一時互換 allowlist は置かない。**

`core.layer_source.clear` は他の 0-input LayerSource と同一経路で `RenderStep::Plugin` へ下ろす。clear 専用分岐はA3-1aでprepared評価へ切り替えた後、A3-1cで削除する。A3-1c完了後の互換allowlist併存期間は置かない。

---

## 4. 型付き拒否分類（Q3）

A0S 表を LayerSource に適用する。新 `PluginDiagnosticReason` variant は本仕様で発明しない。

| 状態 | 分類 | graph／export |
|---|---|---|
| contract 欠落 | `PluginDiagnosticReason::ContractMissing` | `PluginDiagnostics` で不可 |
| contract-only（executor 無し） | prepare 成功＋`ExecutorMissing`（`execution_diagnostics`） | 不可 |
| kind 不一致（例: Filter ID を ClipSource に） | `DocumentPluginError::KindMismatch`（hard） | prepare 失敗 |
| 未来 version | `FutureVersion` diagnostic | 不可 |
| migration chain／conflict | 既存 A0S 理由 | 不可 |
| contract 違反 | `ContractViolation` hard | 不可 |
| 旧 `UnsupportedSourcePlugin(任意ID)` による「未登録は一律」 | **廃止対象**（一般化後）。executor／desc 解決失敗は診断または既存 Plugin エラーへ寄せ、ID 列挙へ戻さない |

prepare 成功かつ diagnostics 空のときだけ graph 構築が進む。診断済み degraded 状態を `UnsupportedSourcePlugin` で黙って拒否しない。

---

## 5. raw Document 不変（Q4）

| 不変面 | 拘束 |
|---|---|
| raw bytes | `prepare_plugins` は clone のみ。永続 bytes を書き換えない |
| revision／Undo | prepared 解決は Document 本体を mutate しない |
| unknown `extra` | 触らない |
| parameter 評価源 | catalog-backed LayerSourceは**prepared.params**のみ。catalog外built-in rectだけはA0SのDocument組込み意味としてraw `clip.params`を`build_rect_overlay`で評価する |
| declarative rename | A0S どおり prepare 時に適用。graph は rename 後の prepared を評価 |

**正例 fixture 名**: `P3_prepared_rename_raw_unchanged` — prepared rename 後 params で評価され raw Document 不変。

**負例 fixture 名**: `N5_unknown_id_contract_missing_raw_preserved` — 未知 ID は `ContractMissing`、raw 保持。

本ticketでprepared評価へ切り替えるのはLayerSource slotだけである。現行
`apply_effect`がFilter definitionのraw paramsを評価する既存GAPはA3-1a／1cへ
混ぜず、A0Iの独立follow-upへ送る。Filterまで同時変更したくなったらSTOPする。

---

## 6. `doc.layer_source.rect` 分離（Q5）

| 欄 | 拘束 |
|---|---|
| catalog／registry | 登録しない（A0S §4） |
| ID 衝突 | A0Sの未閉鎖gate。`FirstPartyError::ReservedBuiltinId { id: &'static str }`を加え、`first_party_runtime`構築時にcatalog／registryの双方を検査する |
| graph | `plugin_id == doc.layer_source.rect` の built-in 分岐のみ。`build_rect_overlay` 専用 |
| 禁止 | registry plugin への偽装、「rect も Plugin step」化 |

**正例 fixture 名**: `P4_rect_overlay_path_unchanged` — rect は引き続き Overlay 経路。

**負例 fixture 名**: `N6_rect_catalog_registration_rejected` — rectをcatalogまたは
registryへ登録したcompositionを
`FirstPartyError::ReservedBuiltinId { id: "doc.layer_source.rect" }`で拒否する。

---

## 7. 画素契約（Q6–Q7）

### 7.1 Coverage 結合＝円群 SDF の解析的 union（Q6-1）

画素中心 \(p\) と各円中心 \(c_i\)、半径 \(r\) に対し

\[
d(p) = \min_i(\lVert p - c_i\rVert - r)
\]

を先に求め、その union 境界へ AA coverage を **1 回だけ**適用する。円ごとの coverage を加算せず、\(1 - \prod_i(1 - C_i)\) の確率的合成も使わない。重なりで alpha を増やさない（A3D §3）。

### 7.2 AA（Q6-2）

正準高さ 1.0、square pixel より 1 pixel 幅 \(w = 1/\mathrm{FrameDesc.height}\) とし、

\[
C = \mathrm{clamp}(0.5 - d/w,\ 0,\ 1)
\]

を使う解析的 1-pixel transition。pixel index `(x,y)`（左上原点）から画素中心の
正準座標への写像は

\[
p_x = \frac{x+0.5-\mathrm{width}/2}{\mathrm{height}},\qquad
p_y = \frac{\mathrm{height}/2-(y+0.5)}{\mathrm{height}}
\]

とする。これにより原点中央・Y-up・高さ1.0・square pixelを固定する。
MSAA／スーパーサンプリング段をHost `Quality`に新設しない。現行render
validationはzero dimensionを拒否しないため、executorは
`output.desc.width == 0 || output.desc.height == 0`を既存
`PluginError::Render`でtyped拒否してから除算する。`max(height,1)`等のdefaultで
意味を捏造しない。

### 7.3 Draft／Final（Q6-3）

同一 `LayerSourcePlugin::render`。`LayerSourceContext` に `Quality` を追加しない。同一 `(t, params, FrameDesc)` なら同一結果を要求する。Filter 用 `RenderCtx.quality` と並行経路を作らない。

数値許容は既存 GPU golden／testkit の方針に従う。本仕様で新しい許容定数を製品契約として発明しない（実装 ticket が testkit 既存ヘルパーを再利用）。

**正例 fixture 名**: `P6_draft_final_same_t_params_desc` — Qualityを受けない同一
`(t, params, FrameDesc)`評価がDraft／Finalの呼称に関係なく同一画素になる。
製品Draftが別解像度を選んだ場合のbyte同一は要求しない。

**負例 fixture 名**: `N8_overlap_alpha_addition_rejected` — 重なり alpha 加算または `1-product` 合成は union SDF へ 1 回だけ coverage を掛ける契約に反する。

**負例 fixture 名**: `N10_zero_dimension_typed_rejection` — zero width／heightを
除算・GPU dispatch前に`PluginError::Render`で拒否する。

### 7.4 straight→premul×coverage と clear／Filter 接続（Q7）

| 欄 | 拘束 |
|---|---|
| Document／`ValueType::Color` | straight sRGB 0..=1（A0S） |
| Radial Repeater 出力 | premul `[r·a·C, g·a·C, b·a·C, a·C]`（A3D §3） |
| 接続確認（正例） | opaque `[r,g,b,1]`はcoverage `C`で`[rC,gC,bC,C]`、特に`C=1`で`[r,g,b,1]`。このopaque内部画素だけをclearの同色結果とexact比較する |
| 接続確認（負例） | 半透明は任意`a,C`で`[raC,gaC,baC,aC]`。重なりでもunion SDFの単一`C`を使い、alpha加算やclearの半透明実装との追認比較をしない |

clear の `LoadOp::Clear` 直書き実装の歴史的細部を本仕様で「正しい premul 正本」へ昇格しない。Repeater の正本は A3D §3。clear との接続は **Color 契約と premul 出力規則**で審判し、clear 実装の追認改変は本発注外である。

---

## 8. first-party 外部 crate／parity（Q8）

A3 実装時（本発注外）の拘束:

| 欄 | 値 |
|---|---|
| crate 配置 | `plugins/motolii-plugin-radial-repeater`（仮名可。ID は A3D 固定 `core.layer_source.radial_repeater` v1） |
| 依存 | `motolii-plugin` のみ（A1S allowlist） |
| 登録 | `motolii-plugins-firstparty` が contract＋executor 登録、catalog／executor ID・version parity |
| Host 必須 capability | 本 ID を envelope 必須として勝手に足さない（opacity のような Host 内 lower 必須ではない）。欠落は recipe 使用時 `ExecutorMissing` |
| 禁止 | 私有 Host crate、Slint、vendor API 依存 |

**正例 fixture 名**: `P7_firstparty_radial_repeater_parity` — firstparty catalog／executor に `core.layer_source.radial_repeater` v1 parity。

**負例 fixture 名**: `N9_private_dependency_slint_vendor_rejected` — private 依存・Slint・vendor API。

---

## 9. Contract 列挙と U4a handoff（Q9）

**決定: 現行境界で U4a への手渡しは足りる。A3／A3S で API を発明しない。**

### 9.1 十分とみなす投影面

| 源 | 投影内容 |
|---|---|
| Catalog `NodeDesc` | `display_name`／`category`／`tags`／`version`／`params` |
| `ParamDef` | `id`／`value_type`／`default`／`f64_domain` |
| Document raw | `DocParam` 値と値 source |
| `PluginDiagnostic` | 欠落・未来・executor 等 |

### 9.2 不十分（A3D 意味正本に置く。`ParamDef` 新 field で埋めない）

単位、+X 基準、CCW、union 説明、premul 規則は A3D §3–§4 を意味正本とする。製品 Panel conformance は M3-U4a＋必要なら GAP-13。M3 停止中の A3 完了条件は **非 UI Contract 列挙 fixture のみ**（A3D §6）。

**正例 fixture 名**: `P5_radial_repeater_contract_enumeration` — display_name／6 params／domain／default の Contract 列挙。

---

## 10. 後続 VSM-A3 の契約境界分割発注表

各 ticket は本 A3S の正負例・STOP・コマンドをコピーして発注する。1 ticket が複数境界を跨がない。

| 仮 ID | 単一境界 | 変更許可の方向性 | 非目標 |
|---|---|---|---|
| VSM-A3-1a（**完了**） | `PreparedDocumentPlugins`をGraphBuilderへ渡し、LayerSource slot取得＋最小`PreparedLayerSourceMissing` errorを追加。clear専用分岐は維持したまま、そのparamsだけprepared評価へ切り替える | `motolii-doc` graph（＋最小test） | 任意IDの一般lowering・clear分岐削除・Filter prepared化・plugin crate |
| VSM-A3-1b（**完了**） | `doc.layer_source.rect` reserved-ID 衝突を product composition root で型付き拒否する A0S 未閉鎖 gate | `motolii-plugins-firstparty`（＋最小 test） | graph lowering・plugin crate・schema |
| VSM-A3-1c | prepared LayerSourceを`layer_source_by_name`で一般loweringし、clear専用分岐と選択的`UnsupportedSourcePlugin`を削除 | `motolii-doc` graph（＋正負test） | prepared配線・composition root・plugin crate・Filter経路 |
| VSM-A3-2 | `radial_repeater` 外部 crate＋contract／executor＋firstparty 登録＋parity／allowlist | `plugins/*`＋firstparty | graph 再設計・画素意味変更 |
| VSM-A3-3 | VRAM golden／union・premul・純関数／Draft=Final | testkit／plugin shader | schema・Panel |
| VSM-A3-4 | 非 UI Contract 列挙 fixture（U4a 手渡し十分性） | 試験のみ | Slint・U4a 本体 |

依存順: A3-1aとA3-1bは本仕様だけに依存し、別境界としてどちらからでも統合できる。
A3-1cはA3-1a後。A3-2はA3-1c後（A3-1bとは独立）、A3-3／A3-4は
A3-2後（相互には独立、M3停止中でも可）。VSM-A3完了には1bを含む全件のmain到達が必要。

---

## 11. 正負 fixture 名一覧

### 正例

| ID | 名前 | 審判内容 |
|---|---|---|
| P1 | `P1_registered_zero_input_layer_source_no_allowlist` | 登録済み任意 0-input LayerSource ID（clear 以外の仮 ID 含む）が allowlist なしで `RenderStep::Plugin` になる |
| P2 | `P2_clear_general_path_same_semantics` | clear が一般経路のみで既存意味を維持（A3-1c 完了条件） |
| P3 | `P3_prepared_rename_raw_unchanged` | prepared rename 後 params で評価され raw Document 不変 |
| P4 | `P4_rect_overlay_path_unchanged` | rect は引き続き Overlay 経路 |
| P5 | `P5_radial_repeater_contract_enumeration` | Radial Repeater Contract 列挙（display_name／6 params／domain／default） |
| P6 | `P6_draft_final_same_t_params_desc` | 同一 `(t, params, FrameDesc)` のDraft/Final呼称で同一（Quality無し） |
| P7 | `P7_firstparty_radial_repeater_parity` | firstparty catalog／executor に `core.layer_source.radial_repeater` v1 parity |

### 負例

| ID | 名前 | 審判内容 |
|---|---|---|
| N1 | `N1_executor_missing_export_graph_rejected` | executor 欠落 → `ExecutorMissing`、export／graph 拒否 |
| N2 | `N2_contract_only_runtime_rejected` | contract-only runtime → 同上 |
| N3 | `N3_kind_mismatch_hard_prepare_fail` | kind 不一致 → `KindMismatch` hard |
| N4 | `N4_future_version_not_executable` | 未来 version → `FutureVersion`、実行不可 |
| N5 | `N5_unknown_id_contract_missing_raw_preserved` | 未知 ID → `ContractMissing`、raw 保持 |
| N6 | `N6_rect_catalog_registration_rejected` | rectのcatalog／registry登録 → `FirstPartyError::ReservedBuiltinId` |
| N7 | `N7_id_allowlist_unsupported_source_plugin_abolished` | ID allowlist／`UnsupportedSourcePlugin` での選択的許可の再導入 |
| N8 | `N8_overlap_alpha_addition_rejected` | 重なり alpha 加算または `1-product` 合成 |
| N9 | `N9_private_dependency_slint_vendor_rejected` | private 依存・Slint・vendor API |
| N10 | `N10_zero_dimension_typed_rejection` | zero dimensionを`PluginError::Render`でtyped拒否 |
| N11 | `N11_fictitious_quality_cpu_readback_rejected` | `LayerSourceContext` への架空 Quality／CPU readback |

---

## 12. 非目標

- コード・テスト・schema・crate・UI／Slint 実装（本発注）
- 新 `ValueType`／`ParamDef` field／widget hint／単位 metadata
- ID 特例 allowlist、lint 抑制、`allow`／`ignore` 追加
- 期待値・golden・fixture の削除・弱体化・書換え
- raw JSON／文字列 scanner で typed 境界迂回
- 公開 raw 割当／mutation API、`from_raw`、`peek_next`
- `serde(default)` で永続意味捏造、暗黙 migration、Document への kind mirror
- CPU frame 経路、架空 `Quality` 引数の `LayerSourceContext` 追加
- A3 Radial Repeater の実装、Duplicator／seed／Composite／Expression／Blender
- Host 全球 `count` ceiling、A3R／A3D 本文書換え
- A3 実装を本仕様内に「仮コード」として埋め込むこと

---

## 13. STOP 条件

次のいずれかに当たったら実装を止め、推測補完しない。

1. 許可外ファイル編集が必要に見える
2. A3D §11 のどれかが「後で決める」のまま残る
3. 新 ValueType／ParamDef field／Quality on LayerSource／Document kind mirror／serde default／ID allowlist／lint 抑制が必要に見える
4. 公開 raw API・生 JSON 走査・重複 planner が必要に見える
5. コード／テスト／crate／Slint／CPU frame へ膨張する（本発注文書作業を除く）
6. A3R／A3D／A0S と矛盾する別名・追加 param
7. 期待値／golden 書換えで通す発想
8. Q9 不足を A3 公開 API 発明で埋めようとする → M3 仕様改訂要求へエスカレーション

---

## 14. 検証コマンド

### 本発注（文書のみ）

```bash
# 許可面のみ
git status --short
git diff --stat

# A3S が §11 を閉じたことの機械確認（文書内見出し／質問番号）
rg -n 'Q1|Q2|Q3|Q4|Q5|Q6|Q7|Q8|Q9|ORDER|非目標|STOP|VSM-A3-1' \
  docs/reviews/2026-07-18-vism-a3s-layersource-lowering-spec.md

# 計画・README リンク
rg -n 'vism-a3s-layersource-lowering-spec|VSM-A3S' \
  docs/reviews/2026-07-17-vism-implementation-plan.md docs/README.md

# コード無差分
git diff --name-only | rg -v '^(docs/reviews/2026-07-18-vism-a3s-layersource-lowering-spec\.md|docs/reviews/2026-07-17-vism-implementation-plan\.md|docs/README\.md)$' \
  && echo 'UNEXPECTED_NON_DOC_FILES' || echo 'DOC_ONLY_OK'
```

`cargo test` は本発注の完了条件にしない。

### 後続 A3 実装 ticket の標準提出コマンド（参考）

```bash
cargo test -p motolii-doc
cargo test -p motolii-plugin
cargo test -p motolii-testkit --test purity
cargo test --workspace
```

各分割 ticket は上記に加え、当該 ticket の正負 fixture 名に対応する test を列挙する。

---

## 15. A3D §11 チェックリスト（9/9 回答済み）

| # | A3D §11 質問 | 本書での回答節 |
|---|---|---|
| Q1 | prepared recipe の kind 検査で、ID allowlist なしに 0-input LayerSource を render step へ下ろす公開 signature は何か | §2（関数／trait signature不変。内部prepared参照＋一般lowering、加算的typed error 1件） |
| Q2 | 旧 `core.layer_source.clear` 特例の処分 | §3（一般化。一時互換なし） |
| Q3 | executor 欠落／contract-only／kind 不一致／未来 version の typed rejection 分類 | §4 |
| Q4 | raw Document を不変のまま prepared params だけを渡す経路 | §5 |
| Q5 | built-in `doc.layer_source.rect` の分離維持 | §6 |
| Q6 | analytic coverage（AA、union 数値意味、Draft／Final 許容） | §7.1–§7.3 |
| Q7 | straight→premul×coverage と clear／Filter 色契約の接続確認 | §7.4 |
| Q8 | first-party 外部 crate 化時の contract／executor 登録と parity | §8 |
| Q9 | 現行 Contract 列挙と U4a 投影 model の十分性 | §9（現行境界で足りる） |

---

## 16. 既知の統合ゲート

- M3 再締結ゲート発効中 → UI／Slint／U4a 実装禁止
- A3 実装は本 A3S `ORDER` 相当の分割発注＋反対側レビュー P0/P1=0 後
- GR-PV: 恒久 schema／新 field を仕様名目で先焼きしない
- 画素正本は A3D。A3S は lowering／拒否／coverage 数値意味と handoff のみ

---

## 17. 後続 ticket 索引

| ID | 依存 | 内容 |
|---|---|---|
| VSM-A3-1a／1b／1c／2／3／4 | 本仕様と§10の個別依存 | §10 分割発注表 |
| VSM-A3 | VSM-A3-1a／1b／1c／2／3／4 | A3D v1 identity の外部 crate 実装と審判 |
