# Core / plugin境界lineageの価値回収（Unit 3A、2026-07-23）

状態: **決定**（歴史文書12 blobの処分、Core用語の現行分界を正本化）

2026-07-25限定改訂: 本書が回収した「作品意味と回復可能性をHostが所有する」規律は維持する。[制御されたMicrokernelとHost capability module並列化決定](2026-07-25-controlled-microkernel-host-module-parallelism-decision.md)により、Host ownershipはauthority ownershipであり、具体実装は製品buildへ明示的にadmitされたHost capability moduleへ分離可能と明確化した。本書の「pluginへ渡さない」を、内部provider化まで禁止する根拠にしない。

対象: `extensible-core-model`、その先例翻訳、M1 plugin境界監査、撤回済みM2コア締結宣言の4 path。

関連: [全歴史coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)、[小さなコア](../extensible-core-model.md)、[surface実装と拡張所有の軸分離](2026-07-22-m3-surface-extension-axis-separation.md)、[plugin authoring](../plugin-authoring.md)

## 1. 結論

4 path / 12 blobを、初版全文、全分岐版、版間diff、現行コード事実で処分した。この単位から復活させる未実装機能契約は無い。価値は、歴史上同じ「コア」が別の三つの意味に使われたことを分け直す点にある。

1. **小さなCore**は作品意味と回復可能性を守るarchitectural roleである。
2. **M1 plugin境界の凍結前監査**は、評価pluginがHostの実行・resource境界へ実際に参加できるかを閉じた実装監査である。
3. **M2コア締結**はmilestoneの証跡宣言だったが、見逃したP1により撤回され、後の基盤再締結ゲートへ置換された。

これらは`motolii-core` crate、native／Reactのsurface、first／third-partyの供給元を一つに分類する語ではない。現行の正規形は次である。

| architectural role | 例 | 現行判定 |
|---|---|---|
| Core kernel | Document、stable ID、時刻、D2、Undo、評価順、Preview/Export、cache/resource | 欠落可能なpluginへ渡さない |
| bundled first-party Host module | native Stage/Timeline、React Browser/Inspector | 標準製品面。runtimeにかかわらずpluginではない |
| first-party plugin | Opacity、Sine、Radial Repeater | 公開境界だけで作る実行可能な手本 |
| third-party plugin | 将来のcommunity Effect/Tool等 | capabilityは共有し得るがtrust/sandbox/permissionは別審判 |

`motolii-core` crateは`FrameDesc`や`RationalTime`等の共有基礎型を置く実装packageであり、上表のCore kernel全量または所属判定表ではない。FrameDesc自身の全履歴処分はUnit 3Cへ分ける。

## 2. 個別処分

| 歴史path / blob | 分類 | 判定 | 現在の回収先 |
|---|---|---|---|
| `docs/extensible-core-model.md` / `73df25ae`,`2684ad09`,`51390ccc` | **現行規範へ吸収 + 成立理由 + 停止線** | 初版の小さなHost責任、責任寿命、昇格、個体性を維持。第2版の未知domain/capability追加原則、第3版のcreator-author連続体も現行へ累積済み。説明用`EvaluatedDomain`等を公開型へしない停止線も生存 | [小さなコア](../extensible-core-model.md)、本書§3 |
| `docs/reviews/2026-07-17-extensible-core-prior-art-translation.md` / `ec6c055d`,`085477e3` | **成立理由 + 再入場候補 + 負例** | 四段、identity、集合所有、上限非焼き込み、Preview縮退の先例は原則の補強。介入正本の逆転、四段の利用者文法、縮退軸契約、遊びの判定は未決のまま保持。AM観察追補を機能不在の悉皆証明にしない | [先例翻訳](2026-07-17-extensible-core-prior-art-translation.md)、M5 P0I |
| `docs/reviews/2026-07-10-M1-plugin-boundary-review.md` / `4e945c18`,`3a00ece`,`8d2704f0`,`fc7da05a` | **実装済み契約 + 成立理由 + 負例** | 未接続registry、毎frame resource生成、AssetRef欠落、全種別lookup欠落、ping-pong欠落を順次閉じた監査。static lifetimeは意味凍結から明示除外した。現在の動的配布契約完成とは読まない | 本書§4、[M1仕様](../specs/M1-vertical-slice.md)、[plugin resources](../plugin-resources.md) |
| `docs/reviews/2026-07-14-m2-core-closure.md` / `9507da52`,`fd929607`,`ed9f9f23` | **撤回 + 負例 + 現行規範へ吸収** | 初版の単独締結はP1二件の見逃しで失効。修復済みだけで再宣言せずA〜C証跡の別ゲートへ移した訂正を維持する。初版の完了表を現行statusへ戻さない | [撤回文書](2026-07-14-m2-core-closure.md)、[M2基盤再締結ゲート](2026-07-15-m2-foundation-reclosure-gate.md) |

## 3. 「小さなCore」へ残す意味

### 3.1 小さいのは機能数ではなく、意味所有の種類

Coreは全標準機能の実装を一枚岩で抱えない。一方、次をpluginへ投棄して見かけ上だけ小さくしない。

- Document identity、version、migration、欠落保持。
- single writer、typed command、Undo/Redo、journal。
- 時刻、型付きparameter、評価順、循環拒否、scope。
- Preview/Export同一評価、cache/invalidation、GPU resource lifecycle。
- 正準座標、色、Quality、失敗診断。
- selection、focus、Preview/Commit/Cancel、accessibilityの共通文法。

Timeline、Preview、Browser等の実UIはこの意味をread-only projectionし、typed intentへ戻すbundled Host moduleである。UIをReactかnativeで作る判断や、private crate境界で交換可能にする判断から、欠落可能なplugin化を導かない。

### 3.2 first-partyは「標準だからCore」ではない

表現固有の計算はfirst-partyでも公開plugin境界だけで作り、source、scaffold、fixture、負例、conformance testを次の作者へ渡す。公開境界で作れない時はfirst-party専用裏口を足さず、欠けた共通能力を一つだけ特定して審判する。

反対に、標準制作体験、Document可読性、plugin欠落診断、Undo、resource管理を「communityが作れるから」という理由で外へ投げない。人海戦術が強くなる条件は、作者へ基礎責任まで再実装させることではなく、表現固有の発明へ集中できるHost契約があることである。

## 4. M1 plugin境界の現行コード照合

| 旧所見 | 現行コード事実 | 処分 |
|---|---|---|
| graphがpluginを呼ばない | `RenderStep::Plugin`と`PluginRegistry` dispatchが存在 | 実装済み |
| 純関数とGPU resource再利用を両立できない | Host所有`PipelineCache`をrender contextから渡す | 実装済み |
| AssetRef語彙がない | `ValueType::AssetRef` / `Value::AssetRef`とDocument側`AssetId`検証が存在 | 実装済み。Importer/packageとは別 |
| `&'static`を将来ABIへ焼く | 現在も`PluginId(&'static str)`とstatic registryを使う | v1実装形。意味凍結の対象外、v2で再締結 |
| by-nameが一種だけ | Filter/Composite/LayerSource/ParamDriver全種別lookupが存在 | 実装済み |
| 未知pluginでDocumentを開けない | D1fがunknown/future pluginを保持してdegraded openし、exportは厳格拒否する | Document経路で実装済み。CLI固有入力のunknown ParamDriver typed errorと混同しない |
| 中間RTを再利用しない | `RenderSession`の`RenderTargetPool`と再利用試験が存在 | 実装済み |

M1監査は「現在のtraitがdynamic loader、WASM sandbox、source distribution、custom UIまで完成した」証拠ではない。特に所有/lifetime、ABI、version negotiation、trust、process isolation、package manifestはUnit 3B以後の別境界である。

## 5. M2早期締結から残す負例

初版はworkspace test緑と代表test一覧からM2 Coreを締結したが、終端audio flushの尺超過とfuture plugin versionのexport拒否迂回を見逃した。二件を直しただけの再宣言も採らず、意味論golden、失敗注入、構造監査を含むA〜C再締結ゲートへ移した。

したがって今後も次を禁止する。

- task表の完了数やworkspace test緑だけでcontract closureを宣言する。
- `core`という語から全crate、全UI、全plugin境界の完成を推論する。
- 一つの発見を修復しただけで、同じ種類の横断経路を再監査せず解除する。
- 撤回文書の古い「閉じたもの」表を、現在のstatus台帳として引用する。

M2基盤再締結は後続ゲートでmain発効済みであり、撤回状態が今もM3全体を停止しているという読みも誤りである。

## 6. 復活させない旧具体とSTOP線

- `EvaluatedDomain`、Queryable等のcapability名、介入envelope fieldを現行公開APIとして固定しない。
- `motolii-core` crateに置かれたことをarchitectural Core所属の証明にしない。
- native UIをCore、React UIをplugin、first-partyをCore、third-partyを常に別capabilityと短絡しない。
- 現行`&'static` registryをdynamic distribution ABIとして固定しない。
- M1実装済み証拠からWASM、custom UI、marketplace、Vism loaderの完成を主張しない。
- M2締結初版、当時のPR順、旧gate statusを現行発注条件へ戻さない。

## 7. 固定歴史出典

| lineage | 読み方 |
|---|---|
| extensible core | 初版`73df25ae`を全文、未知domain拡張版`2684ad09`、creator連続体版`51390ccc`までdiff確認 |
| prior-art translation | 初版`ec6c055d`を全文、AM実機観察追補`085477e3`をdiff確認 |
| M1 plugin boundary | 未解消版`4e945c18`、registry版`3a00ece`、PipelineCache/AssetRef版`8d2704f0`、ping-pong完了版`fc7da05a`を全差分確認 |
| M2 core closure | 初版締結`9507da52`、撤回`fd929607`、再締結ゲート移行`ed9f9f23`を全文/diff確認 |

これら12 blobは本書でDISPOSITIONEDとする。次のUnit 3Bはnative/WASM、plugin UI、公開能力、first/third-partyのlineageを別PRで処分し、FrameDesc共有型はUnit 3Cへ分離する。
