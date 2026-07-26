# CU-102 fresh LayerId + AddTrackItem原子性決定

日付: 2026-07-26
状態: 決定
粒: CU-102
Phase / slice: M3 / VS-1 / SPEC
依存: CU-101 DONE、CU-G03 DONE（CU-G03D / CU-G03R）、D2 DONE、U2b-1 DONE、D1m DONE

## 1. 目的とauthorityの読み方

VS-1 Rectangle配置における **fresh `LayerId` 採番** と **`Command::AddTrackItem` 適用の原子性** の製品意味を、既存 D2、`LayerIdTable`、`DocumentWriter::apply_macro`、現行 `Command` のみを根拠に確定する。Rust・公開 API・Document schema・journal payload・test は変更しない。

authority は次の順に読む。

1. [D2 / selection / Timeline歴史回収 §3.1「到達意味」](2026-07-23-historical-d2-selection-timeline-lineage-recovery.md#31-到達意味)
2. 同文書 [§3.2「product Placeの閉じた意味」](2026-07-23-historical-d2-selection-timeline-lineage-recovery.md#32-product-placeの閉じた意味)
3. 現行 `crates/motolii-doc/src/ids.rs` の `LayerIdTable`、`crates/motolii-doc/src/command.rs` の `Command::AddTrackItem`、同 `lib.rs` の `DocumentWriter::apply_macro`

本決定は上記の後段として、歴史回収が述べた Place 終端と、台帳・コマンド適用の機械的事実を接合する。新 Command variant、公開 Place planner、公開 raw ID mint、汎用 transaction lifecycle、新 journal payload、Document field は作らない。

`docs/decision-index.md`、`docs/implementation-ledger.md`、M3 仕様、`docs/README.md` ファイルマップへの完了 mirror は、Grok 検収 `ACCEPT` 後に主担当 Codex が別変更で行う。本粒は reviews 索引 1 行と本決定文書のみを触り、台帳・索引・仕様の状態行を先取り更新しない。

## 2. 歴史回収との接続

[§3.1](2026-07-23-historical-d2-selection-timeline-lineage-recovery.md#31-到達意味) は、Host Transient Place drag において start / preview / cancel / stale / duplicate では semantic write 0 とし、**accepted terminal drop だけ**が、**同じ同期 call stack 内**で writer snapshot の `LayerIdTable` **clone** 上に候補 ID を作る、と述べる。CU-102 はこの clone 上の候補生成を、live 台帳を進めない作業用状態として確定する。

[§3.2](2026-07-23-historical-d2-selection-timeline-lineage-recovery.md#32-product-placeの閉じた意味) の composition・insertion・Transform・name・start/duration 等は Place の閉じた意味であり、本決定の主題ではない。**appearance（fill / stroke / color）は §3.2 どおり未決**であり、CU-102 は色・線・塗りを決めず、具体値も暫定 default も与えない。

## 3. zero live mint（live カウンタ不変）

drag 中も accepted terminal drop 時も、live `LayerIdTable` に対して `allocate` も `reserve` も呼ばない。live の採番カウンタ（`next`）は、成功経路で `Command::AddTrackItem` が live writer へ適用されるまで一切前進しない。

`LayerIdTable::allocate`（`ids.rs:161-170`）は `entries` へ挿入し `next` を進める。`reserve`（`ids.rs:174-182`）はエントリを作らず `next` だけ進める。いずれも live 台帳への呼び出しは terminal 適用前に禁止する。候補 ID の生値は clone 上の作業だけで得る。

## 4. 二つの終端 freshness 検査（正確に 2 件）

accepted terminal drop の commit 直前に、**再読み込みした live** `LayerIdTable` に対し、次の **2 件だけ**を行う。第三の検査は追加しない。

1. **検査 1**: 候補の生値（`LayerId.0`）が、当該 live 台帳の既存公開読み取り `peek_next()`（`ids.rs:156-158`）と等しい。
2. **検査 2**: 候補 ID が、当該 live 台帳に存在しない（既存 `contains`（`ids.rs:142` 付近）が偽）。

両検査は既存 `pub fn peek_next` と既存 `contains` の読み取りだけで成立する。新しい公開 entry point、raw mint API、`from_raw`、追加の台帳操作は不要である。journal v1 の production 経路が `doc.next_stable_id.peek_next()` を `expected_counter_before` として使う先例（`journal/v1_edit.rs:172`）と同型の「コミット前に live カウンタを読み、typed 比較する」形で足りる。

**作業 clone に対する「候補が clone 台帳に存在しない」ことは要求しない。** clone は mint 時点で `next` と候補の関係が自明であり、live との二重比較に判別力を足さない。第三の freshness 検査や clone 側不在チェックをプロトコルに含めない。

## 5. 絶対候補エントリ不在（台帳エントリの意味）

fresh create として数えるのは、候補 ID に対応する台帳 **エントリ** が、検査時点の **live** 台帳に存在しないことである。既存エントリの黙認や再利用を fresh create とみなさない。`insert`（`ids.rs:186-207`）が既存 id を `Duplicate`、退役 id を `Retired` で拒否するのと同型の判別を、終端検査で先に読み取りで固定する。

台帳エントリが live に生まれるのは、`Command::AddTrackItem` 適用の次の経路だけである。

- `command.rs:852-853` — `layer_names` を走査するループと、`if !doc.layers.contains(*id)` 分岐。
- `command.rs:854` — 上記分岐の真のとき `doc.layers.restore(*id, name.clone())?;` の呼び出し。

事前検査（`command.rs:838-849`）は失敗時にツリー・台帳を変更しない。`command.rs:851` のコメントどおり、更新は事前検査通過後に台帳→ツリーの順で確定する（`command.rs:857` で `insert`）。

## 6. single-entry oracle（1 Rectangle = 1 台帳エントリ）

`ensure_layer_names_match_item`（`command.rs:1103-1116`）は、`layer_names` のキー集合と `item` subtree が参照する `LayerId` 集合の一致を要求し、不一致は `CommandError::LayerNamesMismatch` とする。`collect_layer_ids`（`command.rs:1119-1126`）は `TrackItem` subtree から `LayerId` を深さ優先で収集する。

この二つを **single-entry oracle** と呼ぶ。Rectangle 1 件の `AddTrackItem` は、subtree が載せる `LayerId` が 1 つであるため、`layer_names` に載せられる台帳エントリは正確に 1 件に制限される。1 回の AddTrackItem で 2 つの fresh id を黙って載せる経路は、現行コマンド検証で成立しない。

## 7. 成功経路: 1 AddTrackItem / 1 apply_macro

成功経路は、組み立て済みの既存 `Command::AddTrackItem { parent, index, item, layer_names }` を **1 件**、`DocumentWriter::apply_macro` に **1 回**渡すだけである（`lib.rs:412-442`）。新 Command variant、公開 Place planner、公開 transaction lifecycle、汎用 batch API は作らない。

journal v1 は既に同形の `AddTrackItem` を `plan_add_track_item` 経由で plan する（`journal/v1_edit.rs:81-92` / `184-189`）。新 journal payload、新 Document field、`serde(default)` による欠損埋めは行わない。

**plan と live apply の間に** yield、別 edit、prepared request の再 queue を挟まない。terminal drop の同一同期 call stack 内で freshness 検査から `apply_macro` まで完結する。

成功時に得られる新 `LayerId` は **private receipt** である。同じ Host turn 内で terminal 化し、U2h-1 selection reconcile と atomic publish へ渡して消費する。公開 API、永続 Document、journal レコードへ receipt 専用フィールドを露出しない。

## 8. 失敗不変条件と候補の破棄

freshness 検査に失敗した場合、作業 clone と候補 ID を **破棄** し、後続の drag へ持ち越さない。候補の再利用、再検査、queue もしない。

`apply_macro` 失敗時は `lib.rs:417-437` により `doc`、Undo/Redo 履歴、`revision`、`next_gesture` を呼出前へ戻す。`AddTrackItem` は `command.rs:838-849` で事前検査のみを行い、失敗時はツリー・台帳を変更しない。

したがって失敗時（検査失敗、`apply_macro` 失敗、preflight 失敗を含む）は、Document、layer counter、Undo/Redo 履歴、revision、selection publish が呼出前と一致する。成功・失敗とも terminal とし、自動 retry しない。

## 9. CU-G03D 所有境界（再定義しない）

journal → live Apply/Undo/Redo → revision 進行 → selection reconcile → atomic publish 1 回の順序と、各段階の failure authority は [CU-G03 決定](2026-07-26-cu-g03-edit-durability-ordering-decision.md) の子粒 CU-G03D が所有する。CU-102 は durable commit と live apply の直列順、poison 時の処分、publish envelope の意味を上書き・再定義しない。本決定は VS-1 Rectangle の fresh id と AddTrackItem 1 件の意味論だけを固定する。

## 10. 正例（要約）

1. Terminal drop: clone 上で候補 1 つ → live を再読 → 検査 1・2 通過 → `apply_macro([AddTrackItem])` 1 回 → 台帳に 1 エントリ、ツリーに 1 item → private receipt を同一 turn で消費。
2. Stale terminal: 検査 1 または 2 失敗 → clone/候補破棄 → live 不変 → retry なし。

## 11. 必須負例

- live で `allocate` / `reserve` してから drag を続ける、または preview 中にカウンタを進める。
- freshness 検査を 3 件以上に増やす、または clone 上の候補不在を要求する。
- 1 gesture を複数 `AddTrackItem` や複数 `apply_macro` に分割して原子性を名乗る。
- freshness 失敗後に同じ候補を次の drag へ再利用する。
- 新しい公開 mint API や Place planner を CU-102 の完了条件に含める。
- appearance に具体値を与え、§3.2 の未決を埋める。

## 12. STOP（本決定の範囲外）

次のいずれかが必要になった時点で CU-102 を止め、仕様改訂または別粒へ委ねる。

- 公開 raw ID mint API、公開 Place planner、汎用 transaction lifecycle
- 新 Command variant、新 journal payload、Document schema 変更
- appearance（fill / stroke / color）の決定
- CU-109・CU-110・CU-111 の実装配線や順序の裁定
- 第三の freshness 検査、または作業 clone に対する候補不在の要求

## 13. 非目標

CU-110 は non-live command prepare と accepted terminal の配線を所有する。CU-102 は意味だけを決め、関数追加・module 追加・シグネチャ変更を発明しない。CU-109 / CU-111 の session poison、Undo/Redo prepared action、Browser projection（CU-G09 / CU-101）も本範囲外である。

後続粒の選定は本文書の範囲外。
