# CU-104E projection generation枯渇境界決定

日付: 2026-07-27
状態: **決定**
粒: CU-104E DONE

## 1. 適用範囲とauthority

- 本決定は実装を伴わず、docs-only粒として CU-104 が閉じ残した `projection_generation: u64` の**枯渇境界だけ**を閉じる。
- CU-104 §5は「該当1 actionごとに `projection_generation +1`。`+1` 以外の増分なし」「進退は飛ばし・巻戻しなし」と決めたが、`projection_generation == u64::MAX` の場合に `+1` と no-rewind を同時に満たす規則を書いていなかった。U2h-1I 事前審査でこの穴が実装不能点として顕在化した。本粒はその穴だけを埋め、`U2h-1I` を実装着手可能へ戻す。
- 決定入力は次を採用する。これら以外の文書を新たに authority へ昇格させない。
  - [CU-104 selection publish envelope決定](2026-07-27-cu-104-selection-publish-envelope-decision.md)
  - [U2h-1 primary selection implementation split決定](2026-07-27-u2h-1-primary-selection-implementation-split-decision.md)
  - [CU-G03D edit durability ordering決定](2026-07-26-cu-g03-edit-durability-ordering-decision.md) §3（non-live prepare / preflight 段）
  - [M3-ui-integration.md](../specs/M3-ui-integration.md) U2h 行
  - [implementation-ledger.md](../implementation-ledger.md) CU-104E 行

## 2. 現在のコード事実

BASE_SHA `2a4afcc71a7ae53858af0633a26edd511b48a346` 時点の `crates/motolii-ui/src/document_edit_runtime.rs`。本節以外の repo 探索を根拠にしない。

| # | 事実 |
|---|---|
| 1 | `:93-113` `DocumentEditRuntime::process_next(&mut self, queue) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError>` が唯一の choke point |
| 2 | `:97-99` 先頭で `queue.pop_front()` を1回だけ行い、空なら `Ok(None)` |
| 3 | `:101-107` popした1 actionを `apply_macro` / `undo` / `redo` へ流す。`?` で早期returnした場合も、そのactionは既にqueueから取り除かれている（既存test `failed_d2_action_is_consumed_without_snapshot_or_history_change` `:281-311` が「失敗actionは消費され、revision/history/snapshotは不変」を固定済み） |
| 4 | `:108-112` `PublishedDocument` の構築siteは**この1箇所だけ** |
| 5 | `:121-125` `PublishedDocument` は `kind` / `revision: u64` / `snapshot: Arc<Document>` の3 field、`pub(crate)` |
| 6 | `:140-146` `DocumentEditRuntimeError` は `#[error(transparent)]` の `Command(CommandError)` / `Undo(UndoError)` の2 variantのみ、`pub(crate)`、`thiserror` |
| 7 | `:40-68` `push_prepared` は `Result<(), DocumentEditDispatchError>`、`push_undo` / `push_redo` は返り値なし（infallible） |
| 8 | `crates/` 全体に `primary` / `projection_generation` / `ReplacePrimary` / `ClearPrimary` は未導入（U2h-1S §2事実4） |
| 9 | `revision` は `DocumentWriter::revision` 由来で、runtimeが独自に増分していない |

したがって枯渇規則は「既存の1 choke point」「既存の1構築site」「既存の private error enum」の内側で閉じられる。新runtime、新queue、新publish経路は不要である。

## 3. E1〜E7（枯渇境界の確定意味）

- **E1 判定点**: 枯渇判定は既存 `DocumentEditRuntime::process_next` の内側だけで行う。1 actionを `pop_front` した**後**、`DocumentWriter` へのいかなるmutation（`apply_macro` / `undo` / `redo`）より**前**にpreflightする。CU-G03D §3の段構成では、この判定は **non-live prepare / preflight段**に属し、journal段へ進まない（journal記録を残さない）。pop前に判定すると枯渇actionがqueue先頭に残り永久詰まりとなり、残余queueのdrainability（E3）と矛盾するため、判定はpop後に置く。
- **E2 typed拒否**: 枯渇時は `Err` を返す。error識別子は `ProjectionGenerationExhausted` とし、既存の private `DocumentEditRuntimeError`（`pub(crate)`、`thiserror`）へvariantとして追加する。文字列へ潰さない。`pub` 化・re-export・serde・`motolii-ui` 外への露出はしない。
- **E3 action消費**: 枯渇したactionはちょうど1回消費される（既存の失敗経路と同じ挙動、§2事実#3）。再enqueue・自動retryをしない。残りのqueueは通常どおりdrainできる（次の `process_next` は次のactionを普通にpopする）。
- **E4 不変条件**: 枯渇拒否時、`Document`、Undo/Redo history、`revision`、`primary`、`projection_generation`、published snapshot はいずれも不変。publishする envelope は 0件（CU-104 §8 SN5と同型）。
- **E5 retry / poison非先取り**: 自動retryしない。本粒ではpoison状態・session全体の書込拒否・recover/reopen導線を**新設しない**。枯渇のfailure分類（poisonすべきか否か）は `CU-109` へ引き渡す。
- **E6 算術**: wrap（`u64::MAX` → 0）、saturation（`u64::MAX` に留める）、panic / `debug_assert` / `unwrap` のいずれも禁止。正常成功は従来どおり厳密に `+1` のみ（CU-104 §5不変）。`u64::MAX` は「到達可能だが前進不能な終端値」として扱う。`u64` 幅の実効的到達不能性を「だから決めなくてよい」根拠にしない。
- **E7 入力面不変**: `DocumentEditQueue` の `push_prepared` / `push_undo` / `push_redo` のsignatureと可謬性を変えない。queue API・入力面・intent面に枯渇concernを漏らさない。

**CU-104 §8 SN4 との整合**: 枯渇は「前進しない」だけであり、巻戻し・飛ばし・別counterの導入ではない。SN4（generation drift禁止）と矛盾しない。

**却下した分岐**: `projection_generation` を据え置いたまま envelope を publish する案、および枯渇時に `revision` だけ進める案は取らない。Document変更が起きないため `revision` も進まない。

## 4. 必須正例

- **EP1 通常前進**: `projection_generation == u64::MAX - 1` で accepted action → `u64::MAX` へ `+1`、publish 1。従来と同一。
- **EP2 枯渇拒否（Apply）**: `projection_generation == u64::MAX` で Apply → `DocumentWriter` mutation前にpreflight拒否 → `ProjectionGenerationExhausted` を返す → 当該actionは1回消費 → publish 0 → `Document` / history / `revision` / `primary` / `projection_generation` 不変。
- **EP3 枯渇拒否（Undo / Redo）**: EP2と同一（`kind` によらず同じ規則）。
- **EP4 残余queueのdrainability**: 枯渇拒否の後も、queueに残る action は次の `process_next` で通常どおりpopされる。自動retryも自動破棄も起きず、各actionは同じ規則で決定的に拒否される。

## 5. 必須負例

- **EN1** wrap / saturation / panic / `debug_assert` / `unwrap` による枯渇処理。
- **EN2** `projection_generation` を据え置いたまま envelope を publish する（generation不変publish）。
- **EN3** 先にmutationしてから枯渇を検出し、rollback / 巻戻しで辻褄を合わせる（CU-104 §8 SN4のno-rewindに反する）。
- **EN4** 枯渇actionについて journal commit まで進んでから拒否する（CU-G03D non-live preflight段の逸脱）。
- **EN5** `push_prepared` / `push_undo` / `push_redo` を可謬化する、または queue / 入力 / intent APIへ枯渇concernを露出する。
- **EN6** 本粒でpoison状態、session全体の書込拒否、recover/reopen導線を新設する（`CU-109` の先取り）。
- **EN7** 自動retry、再enqueue、残余queueの一括破棄、静かなno-op成功。
- **EN8** `projection_generation` / 枯渇errorを公開API・`Document`・serde・journal・plugin契約・`motolii-ui` 外型へ露出する。`u64` 幅の変更・別型化。
- **EN9** revision overflow、`revision` 側の枯渇規則、`u64` 型そのものの再決定を本粒で行う。
- **EN10** 第2 envelope、第2 publish経路、第2 counter、別selection構造体の導入（CU-104 §4 / U2h-1S §4に反する）。
- **EN11** CU-104 / U2h-1S の決定済み事項（owner / visibility / field閉集合 / `+1` 規則 / reconcile時点 / P1〜P5帰属）の再決定・上書き。

## 6. 非目標

- Rust / JS / fixture / guard test / golden / threshold / 期待値の変更。
- `revision` overflow、`revision` 側の枯渇規則。
- 公開API、`Document`、serde、journal、Undo/history、`ProjectSession`、workspace/user settings、plugin契約。
- `u64` 型そのものの変更・幅拡張・別型化。
- CU-104 §7 の **P1 / P2 / P3**（U2h-1I帰属）の実装、**P4**（CU-110帰属）、**P5**（U2h-1P帰属）。
- `CU-109` / `CU-110` / `CU-111` / `CU-106` の実装または意味変更。
- `CU-0A08BT` / `CU-0A08IT` / `U4a-2` / `U2c-2`、Host transport、typed intent、JSX binding、drag payload、`S`行。

## 7. STOP

1. 枯渇規則を書くために、CU-104 / U2h-1S / CU-G03D の決定済み事項を変更・上書きする必要が見える。
2. `crates/` または `ui/` の編集が必要に見える（本粒はdocs-only）。
3. 枯渇拒否を書くために、poison状態・session全体拒否・recover/reopen導線を本粒で決めないと閉じない（`CU-109` へ返す）。
4. `revision` overflow、`u64` 幅、公開API、`Document` / serde / journal / plugin契約の新規決定が必要になる。
5. 枯渇preflightを既存 `process_next` 内に置けず、新runtime・新queue・第2 publish経路が必要に見える。

## 8. handoff

CU-104E merge後、PRODUCT-ASSET laneの次実装粒は `U2h-1I`（既存private Apply/Undo/Redo publication経路への `primary` / `projection_generation` 追加、CU-104 §7 P1/P2/P3、`document_edit_runtime.rs:108` と `find_envelope` 再利用）。`U2h-1P` はその後続。`CU-109` / `CU-110` / `CU-111` / `CU-106` は本粒でも `U2h-1I` でも束ねない。枯渇failureの poison 分類は `CU-109` へ引き渡す。
