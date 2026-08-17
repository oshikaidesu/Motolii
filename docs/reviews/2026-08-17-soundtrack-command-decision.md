# Soundtrack Command family(N-SOUNDTRACK-WRITE)

作成日: 2026-08-17

状態: **決定**(実装済み・テスト付き)

関連: `crates/motolii-doc/src/command/variant.rs`(`Command::SetSoundtrack`)、`crates/motolii-doc/tests/m2_soundtrack_commands.rs`

## payload 形

- `Command::SetSoundtrack { old: Option<Soundtrack>, new: Option<Soundtrack> }`。近隣の `SetItemColor` / `SetBlendMode` と同じ決定済みold/new対称形で、inverseはold/newの入れ替え(専用Undo variantなし)。
- merge key: `CommandKind::SetSoundtrack` + `PropertyId::Soundtrack`。Documentに1本のsingletonなので `target_stable_id` は0固定。
- `stable_id_reservation` は `None`(新規stable identityを導入しない)。

## 検証時点

- **構築時**: gain等のpayload不変条件は `Soundtrack::try_new` と自前Deserializeが閉じる(journal decodeでも自動で効く)。Commandは検証済み値しか運べない。
- **apply時**: `new.asset` が `Document.assets` に不在なら `CommandError::Validate(DocumentError::UnknownAssetId)` で書かずに拒否(validateと同じ台帳基準)。`old` は inverse 用の記録であり照合しない(Set*家の非CAS慣行)。
- **journal**: v3 edit として永続化。v2 payload に現れた場合は `decode_edit` が拒否(AdmitAsset/RemoveAssetと同じcutover規律 — v2 writerがこのvariantを書けた事実はない)。journal版繰り上げは不要(externally-tagged enumへの追加variantは後方互換)。

## 非目標

CLI/UIからの配線、`DocumentWriter::prepare_set_soundtrack` ヘルパ(`command/mod.rs` の公開路を触るため別粒)、`motolii-audio`/`motolii-export` の変更(consumerは既に `doc.soundtrack` を読む)、Soundtrack schema の意味変更。
