# D1l新規Documentのv4到達境界 — 追補決定

日付: 2026-07-16
対象: D1l PR #173 / PR #197追補
状態: 【決定・2026-07-16 lint機構追補】（元決定の反対側レビューP0/P1=0。deprecated属性だけ[追補決定](2026-07-16-d1l-new-v1-lint-conflict-decision.md)で置換）

## 1. 発見した停止要因

PR #197はD1l Commandをversion 4へmigration済みのDocumentにだけ適用し、CommandとUndoは`version`/`min_reader_version`を変更しないと決定した。同時に、候補実装の公開`allocate_effect_*`は削除対象とした。

現行APIには`Document::new_v1()`しかなく、候補実装は`allocate_effect_*`の副作用でversion/minをv4へ上げている。このAPIを契約どおり非公開化すると、新規プロジェクトがv4に到達する正規経路が消える。テストがfieldを直接書き換えることで補ってはならない。

## 2. 採用する生成契約

### 2.1 製品の新規作成

`Document::new_current()`を製品の唯一の新規Document生成口とする。現行writerが新しく作るDocumentは次を満たす。

- `READER_VERSION == WRITER_VERSION == 4`
- `version == WRITER_VERSION`
- `min_reader_version == MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS`
- その他の初期値は従来の空Documentと同じ

現行は両定数が4のため、新規Documentは作成時点でD1l Commandの前提を満たす。将来のversion/min変更は個別の永続schema審判を要し、本決定で自動連動させない。新規作成に`Document::default()`や別の暗黙constructorを足さない。

### 2.2 `new_v1`の境界

`Document::new_v1()`は旧版fixture、migration、互換テストのために維持する。製品の「新規プロジェクト」経路はこれを呼ばない。`new_v1()`自体のversion/minを変えて旧fixtureの意味を壊さない。

**2026-07-16 lint競合追補**: `new_v1()`へRustの`#[deprecated]`属性は付けない。`--all-targets -D warnings`と既存semantic test byte不変を同時に満たせないためである。代わりに`#[doc(hidden)]`と下記AST policyを正本とする([lint競合決定](2026-07-16-d1l-new-v1-lint-conflict-decision.md))。利用を許可するのは次の閉じた範囲だけとする。

- `crates/motolii-doc/src/migrate.rs`のlegacy生成
- `#[cfg(test)]`で囲まれたunit test
- `crates/**/tests/**`の互換・migration fixture

非testの`src` targetからの呼び出しは0件に固定する。ポリシーテストは`#[cfg(test)]`領域と上記allowlistを識別し、新しい呼び出しを拒否する。散文のgrep確認だけを完了証跡にしない。

## 3. 既存DocumentとD1l Command

- 既存v1〜v3 Documentは、明示的なD1e v3→v4 inline Effect migrationでDefinition/Use形へ変換する。このv4 stepはD1l PR #173の必須範囲であり、別タスクへ逃がさない。`load_document`や`DocumentWriter::new`が黙ってupgradeしない。
- v1 journal recoveryはPR #197どおり、旧base/generationをメモリ上でD1e migrationしてからlegacy Editを適用する。原本は上書きしない。
- D1lの`prepare_create_effect`/`prepare_link_effect_use`/`prepare_copy_local_effect`と全v2 lifecycle Commandは、v4未満のDocumentを型付きerrorで拒否する。さらに版fieldだけを4へ直書きした未移行inline/hybrid文書をschema/validationで拒否する。版gateだけをmigration完了の証拠にしない。準備・apply・Undoのいずれもversion/minを変更しない。
- `DocumentWriter::new(Document)`は「渡されたDocumentの単一writerになる」だけとし、constructor内で版を上げない。製品の新規作成は`DocumentWriter::new(Document::new_current())`を使う。

## 4. 棄却する案

| 案 | 判定 | 理由 |
|---|---|---|
| `new_v1()`を黙ってv4へ変える | 棄却 | 旧版fixtureとmigration入力の意味を壊す |
| `prepare_*`がversion/minを上げる | 棄却 | 「準備はDocument全文不変」と衝突する |
| lifecycle Command applyがversion/minを上げる | 棄却 | PR #197のUndo等価とversion/min不変を壊す |
| 公開`allocate_effect_*`を版上げ口として残す | 棄却 | Writer prepare単一路とraw caller採番禁止に反する |
| `DocumentWriter::new`が全入力を自動upgradeする | 棄却 | 旧文書を開くことと明示migrationを混同する |

## 5. 自動審判

1. `READER_VERSION == WRITER_VERSION == MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS == 4`。`Document::new_current()`は`version == WRITER_VERSION`かつ`min_reader_version == MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS`で、save/load後も`OpenMode::ReadWrite`のまま不変。
2. `Document::new_v1()`は従来どおりv1のままで、互換fixtureが変わらない。
3. 上記allowlist外の非test `src` targetに`new_v1()`呼び出しを追加した負例がポリシーテストで落ち、既存semantic testにlint suppressionを足さず`cargo clippy --workspace --all-targets -- -D warnings`が緑。
4. v1〜v3の各3入力と、version/minだけを4に偽装したinline/hybrid入力で`prepare_*`とv2 lifecycle Commandが型付き拒否し、counterを含むDocument全文が不変。
5. v4の`new_current()`でcreate/link/copy-localの準備が成功し、準備前後のDocument全文が不変。
6. D1e v3→v4 inline Effect migration後とv1 journalのメモリ上recovery後が、同じv4構造検証と適用前提を満たす。

## 6. 非目標

`new_v1()`の削除、旧プロジェクトの暗黙migration、Default実装、version 5以降の設計、D3e、M3 UI。

## 7. 実装順序

1. 本決定の反対側レビューとmerge。
2. D1l実装PR #173でreader/writer/minの4化、`new_current()`、`new_v1()`allowlist gateを先に追加。
3. D1e v3→v4 inline migrationと、版偽装inline/hybrid拒否を追加。
4. PR #197のv2 Command、Writer prepare、journal adapter、Undo等価を実装。
5. 本書§5とPR #197§6を同じD1l完了証跡で閉じる。

D3eはD1lがmainで閉じるまで発注しない。
