# Vism／Kit／distribution lineageの価値回収（Unit 9A、2026-07-23）

状態: **縮小採用**（29 blobの処分、Kit責任とhostless配布候補の再接続）

> **後続決定（2026-07-23）**: 本書で別責任として回収した`Plugin Set`は、[Kit / Plugin Set統合決定](2026-07-23-vism-kit-rack-unification-decision.md)により独立artifactとして廃止した。接続済み一式はRack型Vism Kit、推薦だけの集合はcurator list／feedへ分ける。以下は当時のlineage処分記録として維持する。

対象: `vism-package-concept.md` 4版、`vism-kit-model.md` 3版、`vism-implementation-plan.md` cutoff 22版。

関連: [全歴史coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)、[Vismコンセプト](../vism-package-concept.md)、[Vism / Kitモデル](../vism-kit-model.md)、[Vism実装計画](2026-07-17-vism-implementation-plan.md)、[「負けた仕様」の価値回収](2026-07-23-losing-specification-value-recovery.md)

## 1. 結論

3 path / 29 blobを、初版全文、全変更commit、並行branchの同一内容、現行正本と照合して処分した。

```text
Vism = 持ち運べる一つの表現
  ├─ plugin capability = Host内部の実行責任
  ├─ payload = Declarative / WGSL / source build / WASM / nativeの比較対象
  └─ package = identity・互換・由来・要求能力（形式未決）

Vism Kit = providerとconsumerを用途へ組みProjectへmaterialize

distribution = catalog / Plugin Set / Project Lock / artifact / install store
               （Kitとは別責任、hostlessは比較候補）
```

重要な判定は次の五つである。

1. **Kitは消えても宙ぶらりんでもない。** provider選択、型付き接続、初期値、素材要求、preflight、1 macro materializeという責任は初版から維持されている。未決なのは公開schemaと実装方式であり、VSM-B0/B1/B2、atomic batch、B2Iを飛ばせないためコード化されていない。
2. **旧plugin ecosystemのKitは別概念だった。** Unit 1で仮称`Plugin Set`（人へ渡す導入入口）と`Plugin Lock`（作品再現）へ分解した。今回、再現対象であることを明確にするため正本上の呼称を`Project Lock`へ寄せるが、公開名称とschemaは未決のままである。
3. **hostless／GitHub配布はVism三文書から脱落した主張ではない。** cutoff 29 blobにはGitHub、hostless、tapの仕様が存在せず、別系譜の旧`plugin-ecosystem.md`に由来する。Motolii運営の常設配布backendへ依存しないことは運用原則へ昇格し、GitHub／静的index等の具体はVism packageへ黙って統合せず`VSM-B3H`で比較する。
4. **nativeに評価することとnative binaryを配ることは別である。** Declarative、WGSL、source＋Host build、WASM、nativeは、可搬性、権限、sandbox、再現性、作者DXを別々に比較する。現行static bundled first-party実証をthird-party install/loadの証拠にしない。
5. **Vism履歴に失われた中核仕様はなかった。** packageの後続版はUser library投影とcreator/developer連続体を追加し、Kit後続版はAccessKit語彙と音楽中心メタファーを訂正した。implementation plan 22版は主にA1〜A3の進捗を累積し、意味の削除ではなく現行事実への更新だった。

## 2. Kitが未実装に見える理由

Kitはconceptだけ先に決め、恒久形式を後ろへ送る意図的な順序にある。

| 段階 | 証明するもの | 現在地 |
|---|---|---|
| VSM-A7 | 既存BPM→DataTrack→既存parameterだけで時間値を渡せる | 完了。consumer input portは未証明 |
| VSM-B0 | package／entry／Kit／Project instance／artifactのidentityを分離 | 待ち |
| VSM-B1 | Vism／Kit／Preset／Asset／Bake／Projectの成果物境界 | 待ち |
| VSM-B2 | DataTrack、typed consumer、Authoring Toolの三方式からmaterialize意味を決める | B0/B1待ち |
| atomic batch | 全体preflight後だけ1 Undoでcommitし、失敗・Cancel・staleは変更ゼロ | 未実装 |
| VSM-B2I | 採用方式だけを製品実装する | 上記待ち |

したがって、現行`DataTrack<Value>`や`DocumentWriter::apply_command`をKitと改名することも、`KitDefinition`を先に永続化することも誤りである。Kitをロード時に常駐するruntime、独自graph editor、更新で既存Projectを書き換えるlinked templateへ拡張する決定もない。

## 3. 配布責任を六つへ分ける

| 責任 | 何を固定するか | Kitとの関係 |
|---|---|---|
| Vism package | 一表現のidentity、実装、要求能力、互換範囲 | Kitが要求できる単位 |
| Vism Kit | 複数表現の型付き構成とmaterialize | 作品構成そのもの |
| Plugin Set（仮称） | 他者へ紹介する導入候補集合 | Kitを含め得るが同一ではない |
| Project Lock（仮称） | 作品が実際に解決したversion、source、artifact | materialize後の再現を助けるがKit identityではない |
| catalog／分散index | package、Kit、Setの発見metadataと取得先 | 実体・install状態の正本ではない |
| install store／loader | 検査済みartifactの端末配置と実行 | Hostのtrust／runtime責任 |

この分離により、GitHub経由の静的index／releaseを使っても、GitHub URL、tag、repository名をVism identityへ流用せずに済む。Motolii projectは中央配布backend、決済、人気集計を運営せず、単一serviceの存続を必須経路にしない。一方、source消失、tag差替え、mirror、署名失効、商用local package、offline、index競合はHost側の診断課題として残る。

## 4. lineage別の処分

| lineage | 分類 | 判定 | 回収先 |
|---|---|---|---|
| Vism package初版 | **現行規範 + 未決分離** | Vism／plugin kind／Project／Kit／payloadを分離。名称と`.vism`以外の形式は未決 | [Vismコンセプト](../vism-package-concept.md) |
| package境界追補 | **現行規範** | User libraryのFolder/Label/Historyをstable package identityへ結び、pathから導出しない | 同§4/現在地 |
| creator連続体追補 | **成立理由** | Vism／Kitをdeveloper専用最終段にせず作者成果の昇格経路にする | [連続体決定](2026-07-22-creator-developer-continuum-decision.md) |
| Kit初版 | **現行規範 + 停止線** | 型を要求しproviderをKitが選ぶ。v1はpreflight＋1 macro materialize、runtime常駐なし | [Kitモデル](../vism-kit-model.md) |
| AccessKit語彙訂正 | **archiveのみ** | Slint固有記述をtoolkit非依存へ直した編集。製品意味の変更なし | 現行Kit本文 |
| 演奏セット→用途セット | **撤回反映** | 音楽を全面化する比喩を撤回。BPM例はtyped providerのfixtureであって製品identityではない | 現行Kit本文 |
| implementation plan 22版 | **進捗系譜** | A0〜A3の完了証跡とUI toolkit語彙を累積更新。旧WAITや旧M3停止文を復活させない | [Vism実装計画](2026-07-17-vism-implementation-plan.md) |
| native/WASM/source/WGSL | **再入場候補 + 負例** | payload classを一つのruntime語へ畳まない。B4/C1/C2で比較後に採否 | 同Phase B/C |
| hostless GitHub distribution | **運用原則 + 別系譜の方式候補** | Motolii運営の常設backend非依存を固定し、Vism Kitへ統合せずB3Hで具体topologyを反証 | 同`VSM-B3H`、[Unit 1](2026-07-23-losing-specification-value-recovery.md) |

## 5. 復活させない具体とSTOP線

- BPM、Beat、演奏をMotolii全体またはKitの必須identityにしない。
- `.vism`の決定からZIP、JSON、directory、MIME、OS associationを逆算しない。
- `PluginId`、package ID、entry ID、Kit ID、Project instance ID、artifact hash、GitHub URLを流用しない。
- KitをPlugin Set、Project Lock、catalog、install store、runtime loaderの総称にしない。
- GitHubから取得できることを署名、信頼、再現性、availabilityの証明にしない。
- Project openからnetwork、install、build、任意code実行を起こさない。
- static bundled first-party crateをnative plugin ABI、WASM sandbox、third-party runtimeの完成と称さない。
- `NodeDesc`へauthor、license、signature、download URLを足してmanifestの代用にしない。
- VSM-B0/B1/B2とatomic batchを飛ばしてKit schemaまたはmaterializerを実装しない。

## 6. 固定歴史出典

`vism-package-concept.md`は初版blob `d233931d`、並行branchを含む境界追補`bcc7c414`／`dbf78007`、creator連続体追補`49440c08`を全文または親差分で読んだ。`vism-kit-model.md`は初版`e7fb5f50`、toolkit語彙訂正`68d7a9a5`、音楽メタファー撤回`176bc5cb`を照合した。

`vism-implementation-plan.md`はcutoff 22 unique blobを、初版全文と全31変更commitの親差分で確認した。同一内容の並行commitを別版として数えず、各unique blobは個別にreceiptへ登録した。旧版だけに残る実質行は過去のM3停止状態とA3進捗表示であり、現行へ復活させる仕様ではない。29 blobの完全SHAは`09a-vism-kit-distribution.tsv`を正本とし、本書でDISPOSITIONEDとする。

Unit 9の残りには、旧plugin ecosystem以外のcommunity／catalog／distribution文書、third-party install/load/trust runtimeの横断lineageがある。今回の29 blobをもってUnit 9全体または3B-runtime-B2-B全体の完了とはしない。
