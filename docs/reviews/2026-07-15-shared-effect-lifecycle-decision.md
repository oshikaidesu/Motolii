# Shared Effect lifecycle決定（GAP-14 / D1l実装ゲート）

ステータス: **【決定】**（[#166](https://github.com/oshikaidesu/Motolii/issues/166)）。  
正本の前提: [Explicit Definition/Use](2026-07-15-relative-scope-duplicator-decision.md)、[先例境界監査](2026-07-15-prior-art-complaint-boundary-audit.md)、[実装準備台帳](2026-07-15-implementation-readiness-ledger.md)。  
本書は共有recipeの**削除・切断・ローカル化・orphan**だけを固定する。Product schema/API・D1l実装・Timeline UI・Composite Set・Backdropは非目標。

## 0. 判定語

| 語 | 意味 |
|---|---|
| **採用** | D1lのcommand/validate完了条件に入れる |
| **縮小** | 採用するが適用条件を狭める |
| **延期** | v1 schemaに焼かず、別Issueで再判定 |
| **棄却** | 暗黙経路として禁止。再提案は仕様改訂必須 |

比較操作の結果語:

| 語 | Document上の結果 |
|---|---|
| **Reject** | Document不変。型付きエラー |
| **RemoveUse** | stackから当該`EffectUse`だけ消す |
| **Materialize** | Definitionをdeep-copyし、当該Useだけ新Definitionへ付け替える |
| **DeleteDefinition** | `effect_definitions`から1件消す（参照0のときだけ） |
| **OrphanKeep** | Useが0件のDefinitionを台帳に残す |
| **Cascade** | Definition削除と同時に全Useを消す（本決定では延期） |

## 1. 固定済み前提（再確認・変更しない）

- `EffectDefinition`と`EffectUse`は別identity。同一layer内でも同definitionの複数useを`EffectUseId`で区別する
- 各layer（およびGroup）のordered effect stackは`[EffectUse]`。共有は参照であり、隣接timelineやtarget集合をDefinitionへ保存しない
- Definitionの`plugin_id` / `effect_version` / `enabled` / `params` / `extra`変更は全Useへ意味反映（評価はD3e）
- 未知pluginを持つDefinitionはD1fどおり保持する。lifecycle操作で`extra`を落としてはならない

## 2. 候補比較（操作ごと）

### 2.1 参照中Definitionを削除

| 候補 | 判定 | 理由 |
|---|---|---|
| Reject（`DefinitionInUse`） | **採用** | 共有recipeを黙って消さない。stack位置・他layerのUseをUI都合で巻き込み削除しない |
| Cascade（全Use削除＋Definition削除） | **延期** | 便利だが破壊範囲が広い。v1は明示2段（各Useを外す→未参照Delete）にする。将来`DeleteDefinitionAndUses`を別commandとして再判定可 |
| Materialize全UseしてからDelete | **棄却** | 削除操作が所有変更を暗黙に起こす。Copy Localと分離する |

### 2.2 1つのUseをUnlink

| 候補 | 判定 | 理由 |
|---|---|---|
| RemoveUse（Definitionは残す） | **採用** | Use identityだけを外す。他Useの共有は維持 |
| Unlink時に自動Copy Local | **棄却** | UnlinkとMake Uniqueを同一gestureに混ぜない |
| Unlinkで参照先Definitionも消す | **棄却** | orphan方針（§2.5）と矛盾し、他Useを壊す |

### 2.3 shared UseをCopy Local / Make Unique

| 候補 | 判定 | 理由 |
|---|---|---|
| Materialize（deep-copy→当該Useだけ付け替え） | **採用** | Resolve Shared Node / Nuke Cloneの「独立化」に対応。他Useは旧Definitionのまま |
| paramsだけ複製してplugin/extraを捨てる | **棄却** | 未知plugin保持（D1f）違反 |
| 全Useを一斉にunique化 | **延期** | v1は1 Use単位。一斉操作は別macro |

Copy Localの複製範囲（必須）: `plugin_id`, `effect_version`, `enabled`, `params`, `extra`（未知field含む）。新しい`EffectDefinitionId`を割り当て、対象`EffectUse.definition_id`だけ更新。旧Definitionの`use_count`が0になればorphan（保持）。

### 2.4 最後のUseを削除

| 候補 | 判定 | 理由 |
|---|---|---|
| RemoveUse＋OrphanKeep | **採用** | 「最後の参照を外す」と「recipeを捨てる」を分離する |
| RemoveUse＋eager GC | **棄却** | 再接続・再共有の余地を黙って消す |
| RemoveUse＋save時GC | **棄却** | 保存が破壊操作になる |

### 2.5 未参照Definitionの保持 / GC

| 候補 | 判定 | 理由 |
|---|---|---|
| OrphanKeep（明示`DeleteDefinition`まで残す） | **採用** | migration直後の1:1やCopy Local後の旧recipeを失わない |
| Purge unused definitions | **延期** | 明示コマンドとして将来可。自動では動かない |
| save/loadでorphan除去 | **棄却** | roundtrip不変条件違反 |

未参照Definitionの削除は **`DeleteDefinition`（use_count==0）のみ採用**。

### 2.6 未知pluginを持つDefinitionへの同じ操作

| 候補 | 判定 | 理由 |
|---|---|---|
| 既知pluginと同一lifecycle | **採用** | 特例経路を作るとF-9席が崩れる |
| 未知なら削除Reject強化 / 強制materialize | **棄却** | D1fの「開いて保持」と衝突 |

Copy Localは`extra`をbyte同等で複製する（キー欠落・並び替えによる意味変質を禁止。serde mapはD1f既存規約に従う）。

## 3. Document before / after表

表記: `Def(D)` = Definition、`Use(U→D)` = UseがDを参照、`stack[L]` = layer Lのeffect stack。初期例:

```text
before共通:
  definitions: { D1(shared recipe), D2(unused orphan) }
  stack[A]: [Use(U1→D1), Use(U2→D1)]
  stack[B]: [Use(U3→D1)]
```

### 3.1 Delete Definition while used

| 操作 | after | Undo 1回 |
|---|---|---|
| `DeleteDefinition(D1)` | **Reject** `DefinitionInUse { id: D1, use_ids: [U1,U2,U3] }`。Document不変 | 適用なし |

### 3.2 Unlink Use

| 操作 | after | Undo 1回 |
|---|---|---|
| `UnlinkUse(U2)` / `RemoveEffectUse(U2)` | `stack[A]=[U1→D1]`。`D1`残存。`U2`消滅 | 復元: `U2`を同一indexへ、同一`definition_id`で戻す |

### 3.3 Copy Local

| 操作 | after | Undo 1回 |
|---|---|---|
| `CopyLocal(U3)` | `definitions`に`D3=deep_copy(D1)`追加。`U3→D3`。`U1/U2`は`D1`のまま | 復元: `U3→D1`、`D3`削除（他から未参照であること） |

`U3`が`D1`の最後の参照だった場合、afterは`D1` orphan + `D3`参照1。Undoは`D1`再利用へ戻し`D3`を消す。

### 3.4 Delete last Use

| 操作 | after | Undo 1回 |
|---|---|---|
| `UnlinkUse(U1)` after already unlinked U2,U3 | `stack[A]=[]`（他layerもD1参照なし）。`D1`は**OrphanKeep** | 復元: 最後のUseを戻す。`D1`は元から台帳にあるので再作成不要 |

### 3.5 Delete unused Definition

| 操作 | after | Undo 1回 |
|---|---|---|
| `DeleteDefinition(D2)`（参照0） | `D2`削除 | 復元: `D2`全文（plugin/params/extra含む）を同IDで戻す |
| `DeleteDefinition(D1)` while orphan | 同上 | 同上 |

### 3.6 再保存 roundtrip

| 状態 | roundtrip後 |
|---|---|
| orphan Definitionあり | 同一ID・同一fieldで残る。validate成功 |
| shared Definition + 複数Use | 参照整合を保ったまま残る |
| Copy Local直後 | 新Definitionと付け替えUseが残る。旧共有も残る |

## 4. 不変条件（D1l validate / command）

1. **参照整合**: すべての`EffectUse.definition_id`は`effect_definitions`に存在する。欠落はload/validateで型付き拒否（黙ってdropしない）
2. **ID一意**: `EffectDefinitionId` / `EffectUseId`はDocument内で一意。再利用禁止（既存LayerId台帳と同型）
3. **orphan許可**: `use_count==0`のDefinitionを許可し、JSON roundtripで保持する
4. **DeleteDefinition門**: `use_count>0`なら`DefinitionInUse`でReject。成功時は台帳から1件削除のみ（Useを触らない）
5. **Unlink/RemoveUse**: 対象Useだけ削除。Definitionは触らない
6. **CopyLocal**: 新Definitionは旧のdeep-copy。対象Useの`definition_id`のみ更新。他Use不変
7. **未知plugin**: lifecycle成功パスで`extra`/未知fieldを喪失しない
8. **1 Undo**: `DeleteDefinition`(成功時) / `UnlinkUse` / `CopyLocal` はそれぞれ1 gesture = 1 Undo。部分適用しない
9. **inline migration**: 既存`EffectInstance`→Definition1+Use1は共有ゼロ。orphanを新たに作らない（1:1）

## 5. D1lへ追加する自動試験（実装時の完了条件へ転記）

1. `delete_definition_while_used_is_rejected` — shared 3 Use、Document不変、typed `DefinitionInUse`
2. `unlink_one_use_keeps_definition_and_other_uses`
3. `copy_local_retargets_only_that_use_and_preserves_extra` — 未知plugin `extra` roundtrip
4. `unlink_last_use_keeps_orphan_definition`
5. `delete_orphan_definition_then_undo_restores_same_id_and_fields`
6. `orphan_definition_survives_save_reload`
7. `copy_local_then_save_reload_preserves_two_definitions`
8. `delete_definition_unused_is_one_undo` / `unlink_is_one_undo` / `copy_local_is_one_undo`
9. migration fixture: inline EffectInstance → 1 Def + 1 Use、画素/order/extra不変（既存D1l条件と結合）

## 6. D1l依存・完了条件への反映

- **依存**: GAP-14は本決定で閉じる。D1l schema実装を**解禁**する（Issue化可）
- **D1l完了条件に追加**: 上記§4不変条件と§5試験。特に「参照中DeleteはReject」「orphan保持」「Copy Local deep-copy」「各操作1 Undo」
- **延期の明示**: Cascade delete-all-uses、Purge unused、一斉Make UniqueはD1l完了条件に入れない
- **U2g/K2**: UIのunlink/copy-localラベルとinvalidationは本決定のcommand意味に従う。線routingは可変のまま

## 7. 非目標（再掲）

Product公開APIの最終型名確定以外の実装、Timeline gutter、Composite Set、Backdrop評価地点、Adjustment Layer型「下全部」、万能include/exclude式。

## 8. 変更履歴

| 日付 | 内容 |
|---|---|
| 2026-07-15 | GAP-14初回決定。参照中Delete=Reject、Unlink=RemoveUse、Copy Local=Materialize、orphan=Keep、未知plugin同一規則 |
