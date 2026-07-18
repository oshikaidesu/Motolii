# Vism-ready化提案の反対側レビュー採否

作成日: 2026-07-17

状態: **採否決定／Vism実装許可ではない**。対象は「既存pluginをOpacity、Sine、BPM、Kitの順にVism-ready化する」という会話上の要約と、それが[Vism実装計画](2026-07-17-vism-implementation-plan.md)および[Vism / Kitモデル](../vism-kit-model.md)へ与える誤読である。添付された独立レビューを実コードで再確認し、採用・縮小を記録する。

## 1. 結論

反対側レビューの主要指摘はすべて採用する。対象選定としてOpacity／Sineを使う方向は維持するが、順序と公約を次へ戻す。

```text
VSM-A0  現行境界のinventory
  ↓
VSM-A7  現行BPM→DataTrack→既存parameterだけの意味spike
  ↓
VSM-A0D migration／doc既知表の所有処分
  ↓
VSM-A0S contract catalog／prepared migration仕様
  ↓
VSM-A0I-1〜3 採択仕様の直列実装
  ↓
VSM-A1  Opacity外部crate実証
  ↓
VSM-A2  Sine外部crate実証
  ↓
VSM-B0/B1 identity／成果物境界
  ↓
VSM-B2 provider／consumer／Kit方式決定
```

現行APIに入力portを持つconsumer pluginはない。A7で「consumer Vism」を実装したと称さず、`DataTrackId`から既存`DocParam::Data`を駆動する経路だけを使う。Kit materializeの実コードは、batch全体のpreflight／atomic commit境界が成立するまでWAITとする。

## 2. コードで確認した事実

### 2.1 ParamDriverは入力portを持たない

`ParamDriverContext`は`start`、`duration`、`sample_rate`だけを持ち、`ParamDriverPlugin::build_track`もcontextとresolved paramsから`DataTrack`を返す。別Vismのtyped outputを受けるportはない。

現行の接続は`DataTrackId`を持つ`ParamSource::Data`／`DocParam::Data`がparameter評価時にtrackを読む構造である。したがって、今すぐ可能なのは「providerが値列を作り、既存parameterが読む」までである。

### 2.2 Sine migrationはfirst-party直書き

`motolii-plugin::migrate_plugin_params`は`core.param.sine`をmatchし、v1の`amp`をv2の`amplitude`へ変える処理をAPI crate内に持つ。Sine本体だけを外部crateへ移しても、migration責任はfirst-party内部へ残る。

これはSineが悪いのではなく、migration登録／実行境界がまだ第三者へ開いていない証拠である。VSM-A2を「first-party無特権の完了」とする前に所有処分が必要である。

### 2.3 Documentは既知plugin契約を手書きでミラーする

`motolii-doc::param_expect`はplugin ID、kind、version、parameter制約を静的に列挙する。層分離のための現行v1実装としては意図的だが、外部crate化しただけではfork／第三者pluginが既知契約へ昇格しない。

runtime registryへ即置換するとは決めない。Document検査の決定性、未知保持、旧Project互換を壊さないより小さい静的案も含め、VSM-A0Dで処分する。

### 2.4 同一Gestureはbatch atomicityではない

`DocumentWriter::apply_command`は一つのcommandを`UndoHistory::push`へ渡し、`push`は先に`command.apply(doc)`してから同じGestureのMacroへ積む。複数commandの途中で後続が失敗した場合、先行commandを自動rollbackするbatch APIは現存しない。

したがって「1 Undo」と「全失敗時Document変更ゼロ」は別保証である。Kit展開にはM3-U9a相当の開始snapshotに対するbatch全体preflightと一括commitが必要である。

## 3. 指摘の採否

| ID | 指摘 | 判定 | 反映 |
|---|---|---|---|
| CR-1 | A0 inventoryが要約から消えた | **採用** | A0を最初の仕様PRとして再固定 |
| CR-2 | A7／4手目で存在しないconsumer APIを前倒しした | **採用** | A7をDataTrack→既存parameterへ縮小。consumer方式はB2へ |
| CR-3 | A1/A2では落ちない完了条件が多い | **採用** | A1/A2はprivate依存負例、golden、purity、ID/version不変へ限定 |
| CR-4 | 2本でPhase A境界成立／量産可能は出口条件の切下げ | **採用** | A3、A6、A5を維持し、Phase A前の量産公約を削除 |
| CR-5 | Sine migrationとdoc既知表がfirst-party特権として残る | **採用** | VSM-A0Dで所有を決定し、VSM-A0S仕様→VSM-A0I-1〜3実装を通してからA1/A2へ |
| CR-6 | Vism候補の三問はadmission testとして弱い | **採用** | 候補トリアージへ格下げ。entry／fixture処分を先取りしない |
| CR-7 | Gesture MacroだけではKit展開の全失敗atomicityを保証しない | **採用** | 意味決定B2と実装B2Iを分離。B2Iはatomic batch待ち |
| CR-8 | A5とM2再締結レーンの衝突確認が必要 | **採用済みを維持** | A5のWAITと衝突確認を残す |

## 4. 完了条件の再配置

設計全体として必要でも、Opacity／Sineのcrate分離では落ち得ない条件をA1/A2へ置かない。

| 条件 | 置き場 |
|---|---|
| private Motolii crate依存禁止 | A1/A2、A4のCI負例 |
| 既存pixel／値列、ID、version不変 | A1/A2 |
| purity、GPU、非有限入力、typed error | A1/A2／既存conformance |
| migrationを第三者と同じ境界で扱える | A0D→A2 |
| doc既知契約とregistryの責任 | A0D |
| 他Vism ID非参照、typed接続 | A7では既存DataTrack結線、一般化はB2 |
| 欠落／未来版／strict export | A5。A1/A2のcrate分離だけでは達成判定しない |
| KitのCancel／失敗変更ゼロ | B2I。atomic batch成立後 |
| custom UIなしで全param操作 | M3-U4a／V2-8。現行A1/A2の弁別条件にしない |
| first-partyと第三者が同じconformance | A4と将来のout-of-tree fixture。第三者不在の一度きり確認にしない |

## 5. Vism候補の扱い

OpacityとSineを最初に使う理由は、既にコードとテストがある最小Filter／ParamDriverだからである。「必ず独立Vismとして配布する」と決定したためではない。

- Clearは入力／出力／parameterを持ってもconformance fixtureのままが自然かもしれない。
- Sineはprovider VismとCore primitiveの両候補である。
- OpacityとTintを別Vismにするか、一package複数entryにするかも未決である。

ユーザーが名前で探すか、独立した契約があるか、独立lifecycleに意味があるかという質問は候補発見に使う。最終処分はidentity、Kit、更新、欠落、entry粒度のfixture後に行う。

## 6. 修正後の停止線

- VSM-A0なしでplugin外部crate化を始めない。
- VSM-A0I-1〜3なしでOpacity／Sineを「first-party無特権」と完了判定しない。
- 現行ParamDriverへ入力portを静かに追加しない。
- A7をconsumer Vism／Kit実装と呼ばない。
- 同一Gestureへ複数commandを積むだけでatomic batchと呼ばない。
- Opacity／Sineの二本だけでPhase A完了、Vism量産可能と公約しない。
- package、entry、Kit、Project instance、artifact identityのfixture前にKit schemaを作らない。

この処分により、既存pluginのVism-ready化は後退しない。むしろ、最初の二本が必ず踏むmigrationとDocument検査の継ぎ目を先に可視化し、後続Vismがfirst-party用の追記を要求しない境界を作る。
