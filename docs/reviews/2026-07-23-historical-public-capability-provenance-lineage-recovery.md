# 公開capability／provenance lineageの価値回収（Unit 3B-runtime-B2-A、2026-07-23）

状態: **縮小採用**（11 blobの処分、外部crate実証と第三者runtimeの分離）

対象: VSM-A1公開crate境界9版、surface実装／拡張所有の軸分離1版、Creator / Developer連続体1版。

関連: [全歴史coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)、[A1公開crate境界](2026-07-17-vism-a1-public-crate-boundary-spec.md)、[軸分離決定](2026-07-22-m3-surface-extension-axis-separation.md)、[Creator / Developer連続体](2026-07-22-creator-developer-continuum-decision.md)

## 1. 結論

3 path / 11 blobを、A1初版全文、全13変更commitと分岐版の親子diff、後続2決定全文、現行コード事実で処分した。

```text
motolii-plugin内部の参照Opacity
  → 公開façadeを一依存で使う外部crate
  → first-party composition rootへ集約
  → pixel / purity / dependency / public-path負例で移動同値を実証
  → Opacity、Sine、Radial Repeaterへ拡張
  → UI runtime・architectural role・provenance/trustを別軸化
  → user/developerの参加資格は薄くし、security責任は維持
```

現行へ維持する判断は次の五つである。

1. A1の「外部plugin crate」は`motolii-plugin`実装crateの外にある、同じrepository/workspaceへ静的に組み立てるbundled first-party crateを指す。第三者packageのinstall/load/unload、ABI、sandbox、署名、version併存の実証ではない。
2. first-party無特権は有効な反証法である。通常依存を公開façade一つへ閉じ、private Host path、vendor API、panic、別wgpu版、testkit逆依存を拒否し、Host側からpixel/purity/catalog parityを審判する。
3. public façadeとfirst-party composition rootは別責任である。作者crateは表現を実装し、Host内部rootが既定catalog/executor、必須capability、予約ID、型付き組み立て失敗を所有する。
4. OS topology、native/React/headless presentation、Core/Host/plugin role、first/third-party provenance/trustを別軸にする。first-party source、React component、native surfaceという属性だけでplugin公開や信頼を推論しない。
5. user/developerの固定身分と学習断絶は薄くするが、untrusted/reviewed/bundled、permission、resource、single writer、migration、Host保守責任は消さない。多数作者戦略は無審査code実行や「本体が穴を放置する」戦略ではない。

## 2. lineage別の処分

| lineage | 分類 | 判定 | 現在の回収先 |
|---|---|---|---|
| A1初版→preflight補強 | **現行規範 + 成立理由** | 一依存façade、allowlist、Host側審判、pixel移動前基線、feature-unification限界を維持 | [A1公開crate境界](2026-07-17-vism-a1-public-crate-boundary-spec.md) §2〜3/5 |
| A1-1→A1-2→A1-3主lineage | **実装済み公開能力** | façade、composition root、Opacity外部化を段階実装。型付きerror、完成値を返すroot、必須capability早期拒否を現行へ戻す | 同§4/6/7、本書§3 |
| A1-3短縮分岐 | **成立した結果 + 消えた詳細** | `2165f590`は完了状態を持つが、別枝のexact root/error/Opacity export追補を欠く。現行コードが満たす部分だけ現行A1へ再統合し、古いcommit pinは現在の入場条件にしない | 同§2.2/4 |
| surface／拡張所有の軸分離 | **現行決定 + 負例** | native/React、Core/plugin、first/third、trusted/isolatedを独立判定。G0-9合格でG0-3を解除しない | [軸分離決定](2026-07-22-m3-surface-extension-axis-separation.md) |
| Creator / Developer連続体 | **現行決定 + Host責任** | Use→Tune→Compose→Inspect→Fork→Author→Publishを連続化。trust境界と標準体験、配布診断、保守をHostに残す | [連続体決定](2026-07-22-creator-developer-continuum-decision.md) |

## 3. A1がコードで証明した範囲

### 3.1 作者crate

現行のOpacity、Sine、Radial Repeater crateは通常依存を`motolii-plugin`一つに閉じ、façade経由のwgpu、bytemuck、shared types、traitだけを使う。これにより次を証明する。

- private Host crateへ依存せずに現行plugin traitを実装できる。
- Hostと作者が同じwgpu/type versionを使える。
- source移動前後でcontract ID、parameter、pixel、purityを維持できる。
- first-partyの裏口をdependency/source scanで機械拒否できる。

これは**source-level author capability**の実証である。workspace外standalone build、package download、署名、runtime load、crash isolation、権限grant、複数version共存は証明しない。Cargo workspaceのfeature unificationによる偶発依存も、現行A1検査の既知限界として残る。

### 3.2 Host composition root

`motolii-plugins-firstparty`は完成catalog、完成registry、検証済みruntimeを作り、contract/executor mismatch、必須Opacity欠落、Document builtin予約ID混入を型付きerrorで拒否する。CLIやDocument経路が個別にpluginを足し引きする場所ではない。

このrootは**bundled product assembly**であって、第三者runtime registryやmarketplace resolverではない。将来community packageを導入しても、first-party既定集合、user-installed集合、trust policy、project lock resolutionを一つの可変registry関数へ混ぜない。

### 3.3 public closed-set fixtureの読み方

Opacity/Sine等のpublic-path closed-set testは「この参照実装が実際に使った公開面」を固定し、移動時にprivate pathが増えることを拒否する。Motolii全plugin能力の永久閉集合、将来Vism manifestのcapability vocabulary、第三者権限表ではない。新しい表現が正当な共通能力を必要とする場合は例外文字列を足さず、公開面レビューと反対側fixtureで追加する。

## 4. first-partyはroleでもtrustでもない

同じfirst-partyでも、標準Timeline/Stage/Browser/Inspectorはbundled Host module、Opacity/Sine/Radial Repeaterは公開plugin境界の実装である。前者をprivate crateへ分けてもcommunity差替えpluginにはならず、後者をHost binaryへ静的linkしてもCore kernelにはならない。

逆にthird-party候補もpresentation runtimeをまだ持たない。native binary、WASM、WGSL、source build、React panelのどれを選ぶかはpayload/runtime/sandboxの別審判で、provenance名から逆算しない。

## 5. creator/developer連続体へ残す実装審判

- codeを書かないcreatorは標準体験だけで作品を完成できる。
- tuning、composition、inspection、forkで対象・parameter・由来を失わない。
- first-party手本はpublic contractと同じfixtureで検査する。
- packageの作者、version、要求能力、permission、欠落時挙動をHostが説明する。
- 一packageの失敗をDocument、他package、無関係な作品領域へ波及させない。
- 反復需要をpreset→first-party→Host primitiveへ追加的に昇格し、Coreへ無制限に列挙しない。

これらはmarketplace、loader、manifestを既決にする要求ではない。distributionの具体はUnit 9、native/WASM payloadと実行隔離はB2-Bで処分する。

## 6. 復活させない旧具体とSTOP線

- A1の「external crate」をthird-party install/load済みと称さない。
- static bundled Rust crateをnative dynamic plugin ABIと呼ばない。
- `first_party_runtime()`を任意packageの可変登録口やmarketplace resolverへ一般化しない。
- public-path closed setを全作者の能力上限やpermission manifestへ転用しない。
- first-partyであることからCore所属、in-process許可、署名免除、無制限resourceを推論しない。
- React component/test語彙の共有を同一origin、process、permission、配布の共有へ広げない。
- user/developer境界を薄くするためにDocument writer、Undo、migration、sandboxを薄くしない。
- standalone buildを実証せず「repository外の第三者が一依存でbuild可能」と宣伝しない。
- 旧A1の固定commit列を現在の新規実装入場点として再利用しない。

## 7. 固定歴史出典

A1初版`678dc95d`を全文で読み、cutoff 9 blobが`git log --all -- <path>`のunique 9 blobへ一致することを確認した。13変更commitには同一内容の並行commitがあり、主lineageの`86129768`と短縮分岐`2165f590`を別に比較した。短縮分岐で落ちたcomposition signature、Opacity export、error保持、必須capability負例は現行コードと一致する範囲だけA1正本へ再統合した。

`c8109b90`（surface軸分離）と`18874d39`（Creator / Developer連続体）は単一版を全文で読み、現行decision index、plugin作者規約、first-party三crate、composition rootと照合した。11 blobの完全SHAは`03f-public-capability-provenance.tsv`を正本とし、これらは本書でDISPOSITIONEDとする。

Vism package／Kit／hostless配布、native/WASM/source/WGSL payload、third-party install/load/trust runtimeは未処分であり、B2-BとUnit 9へ残す。
