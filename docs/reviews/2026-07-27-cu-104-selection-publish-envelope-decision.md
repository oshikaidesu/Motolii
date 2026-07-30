# CU-104 selection publish envelope decision

日付: 2026-07-27
状態: **決定**
粒: CU-104 DONE

## 1. 適用範囲とauthority

- 本決定は実装を伴わず、docs-only粒として CU-104 の4点だけを閉じる。
  1. primary envelope の owner / visibility
  2. envelope のfield閉集合（既存fieldの再利用）
  3. `projection_generation` の進行・不進行規則
  4. Apply / Undo / Redo成功時およびPlace receipt時の reconcile→publish 時点
- 決定入力は次を採用する。別文書を越えて新規契約を作らない。
  - `docs/reviews/2026-07-23-historical-d2-selection-timeline-lineage-recovery.md` §5
  - `docs/specs/M3-ui-integration.md`（U2h行）と運用順
  - `docs/reviews/2026-07-24-m3-vertical-slice-execution-decision.md` CU-104行
  - `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md` CU-104行
  - `docs/decision-index.md` primary selection行
  - `docs/implementation-ledger.md` CU-104行

## 2. 現在のコード事実

- `crates/motolii-ui/src/document_edit_runtime.rs` の `PublishedDocument` は
  `kind: DocumentEditActionKind` / `revision: u64` / `snapshot: Arc<Document>` の3fieldで構成される private envelope（`pub(crate)`）。
- `process_next` は `Apply / Undo / Redo` 1件ずつ consume し、成功時のみ `Some(PublishedDocument)` を1回返す。失敗時は `Err`。
- `revision` は `DocumentWriter::revision` 由来。
- `publish_document_snapshot` は `current_document` を更新し、`render_client.submit` の返す generation は render queue 用。projection generation と分離。
- `LayerId` は `pub struct LayerId(u64)`。
- selection再検証は既存の再帰存在判定 `find_envelope`（存在oracle）を利用可能。
- 現行実装には `primary` / `projection_generation` / selection reconcile を含む構成は未導入。

## 3. D1 owner / visibility（決定）

- **owner**: `primary: Option<LayerId>` は `motolii-ui` Host event-loop runtime（`MotoliiApp` 側の transient field）1か所の所有のみ。
  - `motolii-doc` / `DocumentWriter` / `Document` / React / native surface は所有しない。
  - 既設の `PublishedDocument` を拡張して同一経路を使う。
- **visibility**: private transientとし `pub(crate)` のまま。`pub`化・re-export・`serde`導線・motolii-ui外型公開はしない（GR-UI-6）。
- `decision-index` の `PublishedUiState` は逆引きキーワードの識別子であり、型名として確定しない（本決定では採用しない）。
- 既存authorityでないfield追加はしない。

## 4. D2 field閉集合（決定）

`PublishedDocument` は private envelope に対し、次を閉集合として持つ。

| field | 型 | 由来 |
|---|---|---|
| `kind` | `DocumentEditActionKind` | 既存`document_edit_runtime.rs` |
| `revision` | `u64` | `DocumentWriter::revision` |
| `snapshot` | `Arc<Document>` | 既存`PublishedDocument` |
| `primary` | `Option<LayerId>` | 歴史回収§5.1 の明示 owner/type |
| `projection_generation` | `u64` | 既存 `revision` と同等幅の private counter |

- `projection_generation` はprivate transient session内counterであり永続化しない。public API / serde / Document schema に含めない。
- 追加fieldは持たない（2nd envelope、2nd publish path 作成しない）。

## 5. D3 `projection_generation` 更新規則（決定）

### 進む場合
- Apply / Undo / Redo の成功 publish（journal段は本粒の非対象）
- 受理された selection-only 変更（有効な `ReplacePrimary` / 非noneの `ClearPrimary`）
- いずれも該当1 actionごとに `projection_generation +1`。`+1` 以外の増分なし。

### 進まない場合
- `same-id ReplacePrimary`
- already-none の `ClearPrimary`
- unknown / table-only ID の typed reject
- Apply / Undo / Redo の失敗
- queue 空 (`Ok(None)`)
- diagnostic / target / preview / cancel 等の非適用系フロー

### 原則
- `revision` と同時に進めるのは Document変更を含む成功 action のみ。
- selection-only変更は `revision` を進めない。
- `projection_generation` を `revision` から導出しない。
- surface別counterと混用しない。
- `render_client.submit` が返す render generation を代用しない。
- 進退は飛ばし・巻戻しなし。

## 6. D4 reconcile / publish時点（決定）

- 1 accepted action = 1 reconcile = 1 publish。
- order:
  1. Apply/Undo/Redo 成功時に `snapshot` を確定
  2. `primary` の再検証を、対象 `snapshot` に対して再帰 `find_envelope` で実施
  3. dangling は `None` clear、valid は ID保持
  4. `projection_generation` の更新規則に従い更新（該当時のみ）
  5. envelope を1回 publish
- **Redo non-restoration**: Undo/Redo時も `primary` は直近履歴の再生対象として復元しない。
- **Place receipt（non-impl）**:
  - 成功receiptの `LayerId` は同一host turnで `primary` を replace
  - `Document` と同じ envelope で publish 1回のみ
- 失敗時は publish 0、`projection_generation`不変、`primary`不変、snapshot/revision/history不変。

## 7. 必須正例

- **P1 Apply成功**  
  journal段の後、live apply → `revision +1` → reconcile（valid retain）→ `projection_generation +1`（selection-onlyでない場合）→ publish 1。
- **P2 Undo成功でprimary dangling**  
  live undo → `revision +1` → reconcileで `primary = None` → generation +1 → publish 1。
- **P3 Redo成功**  
  live redo → `revision +1` → reconcileのみ（clear済みprimaryを復元しない）→ generation +1 → publish 1。
- **P4 Place receipt**  
  成功receipt新 `LayerId` を同一Host turnで `primary` replace → 同一 envelope publish 1回。
- **P5 selection-only valid `ReplacePrimary`**  
  `revision` 不変、Document/serialize/history/journal 不変、`projection_generation +1`、publish 1。

## 8. 必須負例

- **SN1 selection persistence**  
  `primary`/`projection_generation` を Document / serde / journal / Undo/Redo履歴 / ProjectSession / workspace profile / user settings に保存しない。
- **SN2 split channels**  
  Document と selection を別channel・別publish・別revisionで送らない（envelope 1つ）。
- **SN3 surface-local stores**  
  Stage / Timeline / Inspector / KEYS / LAYERS / Easing trigger / React が独自selection storeを持たない。
  全surface は同一 envelope の read-only projection を読む。
- **SN4 generation drift**  
  surface別counter、`revision` からの導出、render generation の流用、+2以上の飛び、巻き戻しは禁止。
- **SN5 rejected/no-op publication**  
  same-id replace、already-none clear、unknown/table-only reject、Apply/Undo/Redo失敗で publish と generation 前進をしない。
- **SN6 publish-before-reconcile**  
  reconcile 前に publish しない。1 action の二重 publishと、publish後に同一actionへfollow-up reconcileすることを禁止する。後続の別accepted actionは§6 D4の順序で通常どおりreconcileしてpublishする。
- **SN7 Redo selection restoration**  
  Redo で過去のselectionを implicit restore しない。

## 9. 非目標・STOP・後続handoff

- 非目標: Rust/JS/fixture/guard test/golden / threshold 変更、U2h-1実装、CU-109 journal runtime配線、CU-110 Place、CU-111 prepared Undo/Redo actions、CU-106 consumer接続、CU-0A08BT/IT、U2h-2 additive/range/marquee/AX、公開API・Document・serde・journal・Undo/history・ProjectSession / workspace永続。
- STOP条件のいずれかが顕在化した場合は実装を止めて Codex へ報告し、`ORDER: STOP` とする。
  - 新規field/type/name が必要になる
  - `PublishedDocument` 拡張ではなく第2 envelope が必要に見える
  - 既存authorityと矛盾した保存先への書込みが自然に見える
  - 既存non-goals以外への実装寄り決定が必要になる
- 完了後の次粒は本文書では定義しない。  
  `"CU-104完了後の粒は改めて明示選定する"` を保持する。
