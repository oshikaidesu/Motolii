# VSM-B0 identity期待値マトリクス — package／entry／Kit／Project instance／artifact

作成日: 2026-08-17

状態: **意味fixtureのみ**。本書は`VSM-B0`の期待値表であり、schema、公開型、manifest、コードの所有者ではない。Rustの型・trait・enum variant、manifestのkey、直列化形式、migration方式、typed portの方式を規範として提案しない。ケース5・6の**採否も決めない**（期待値の言語化だけを行う）。

関連正本: [Vism実装計画 §3・§4](2026-07-17-vism-implementation-plan.md)、[Vism / Kitモデル](../vism-kit-model.md)、[Vismコンセプト](../vism-package-concept.md)、[決定逆引き台帳](../decision-index.md)

## 0. 読み方

### 0.1 三つの軸

- **ケース6件**: 1〜4は[実装計画 §3](2026-07-17-vism-implementation-plan.md)の原文（`docs/reviews/2026-07-17-vism-implementation-plan.md:74-77` [権威]）。5〜6は`docs/reviews/2026-07-17-vism-implementation-plan.md:79` が「一package複数entryは同じlifecycle／compatibility責任から分離できない場合だけ比較する」として未決に置いた軸を、期待値の形で言語化するために追加した。
- **操作6件**: `rename`（packageの表示名だけを変える。配布上の識別子は変えない）／`update`（同じpackageを新しいversionで入れ替える。識別子は同じ）／`duplicate`（一つの作品内で、そのVismを使っている箇所を複製する）／`missing`（参照されているpackageが環境に存在しない状態でProjectを開く）／`reinstall`（同じpackageを一度削除し、同じversionを入れ直す）／`fork差替え`（別の作者が作った別packageへ参照先を差し替える）。識別子そのものの変更は本書の軸に含めない（`VSM-B3`のmigration領域である）。
- **identity5件**: `package` / `capability entry` / `Kit` / `Project instance` / `artifact`。定義と所有者は `docs/reviews/2026-07-17-vism-implementation-plan.md:64-68` [権威] と `docs/vism-kit-model.md:294-298` [権威] に従う。

各ケースは、その構成が一つのProjectから参照されている状態を前提に操作を適用する。`duplicate`と`missing`の語義自体がProjectの存在を前提にしているためである。

### 0.2 根拠の表記

- `[権威]` = identity期待値の根拠として引ける文書。`docs/reviews/2026-07-17-vism-implementation-plan.md` の §3（`docs/reviews/2026-07-17-vism-implementation-plan.md:58-81`）と §4 の`VSM-B0`行（`docs/reviews/2026-07-17-vism-implementation-plan.md:162`）、`docs/vism-kit-model.md`、`docs/vism-package-concept.md`、`docs/decision-index.md` のVism関連行に限る。
- `[現状]` = 何が現在実装されているかの記述。identityの期待値そのものの根拠にはしない。

### 0.3 セルの数え方

セル総数は 6ケース × 6操作 × 5identity = **180**。各セルは次のいずれかで埋まっている。

- 期待値（`保持` / `変化` / `新規採番` / `参照切れ`）＋根拠
- `該当なし` ＋ 理由1行
- `UNDETERMINED:` ＋ どの文書にどういう決定が要るか

`UNDETERMINED:` を含むセルは`UNDETERMINED`として数える。

### 0.4 本書が決めないこと

- 一package複数entryの採否（`docs/vism-package-concept.md:282` [権威] が「1 package内のcapability数」を未決としている）。
- provider→consumerの接続方式とmaterialize Kitの実装方式（`docs/reviews/2026-07-17-vism-implementation-plan.md:162` [権威] の後続 `VSM-B1`／`VSM-B2`）。
- `PluginId(pub &'static str)`（`crates/motolii-plugin/src/contract.rs:9` [現状]）をVism identityと宣言すること。`docs/reviews/2026-07-17-vism-implementation-plan.md:70` [権威] がこれを禁じ、同行がinstance identityをpackage versionやTimeline上の並び順から導出することも禁じている。
- `docs/decision-index.md:215` [権威] の圧縮映像・撮影起点3D観察行は、同行自身が「現行VSM-A4I/A5/B0〜B2、公開APIには接続しない」と処分している。本書へ接続しない。

## 1. UNDETERMINED の総数と内訳

**総数: 36 / 180**（初版30。反対側レビュー後の訂正で6件増えた。§8参照）。

180セルの分類は次のとおりである。

- 期待値＋根拠: **126**
- `該当なし`＋理由: **18**（Kit接続を含まないケース1・2・5の `Kit` identity。3ケース × 6操作）
- `UNDETERMINED:`: **36**

`UNDETERMINED`の内訳は6種であり、いずれも6ケース全部に同じ形で現れる（6種 × 6ケース = 36）。

| # | 未決の主題 | どの文書にどういう決定が要るか | 該当セル | 件数 |
|---|---|---|---|---|
| U1 | version更新をまたぐentry identity | `docs/vism-package-concept.md` §4.1／§10「1 package内のcapability数」に、package version更新をまたいでcapability entry identityを保持するか、entryの追加・削除・改名を許すか、Projectが保存した`selected capability / entry`をどう再解決するかの決定が要る | 全6ケース × `update` × `capability entry` | 6 |
| U2 | fork差替え後のProject instance identity | `docs/vism-kit-model.md` §5（linked Kit／Kit更新の追従が「将来の別審判」に置かれている段）に、materialize済みProjectの参照先packageをforkへ差し替えた時、置換される側のProject instance identityを保持するか新規採番するかの決定が要る | 全6ケース × `fork差替え` × `Project instance` | 6 |
| U3 | 表示名とartifactの関係 | `docs/vism-package-concept.md` §4.2に、表示名が配布artifactの内容に含まれるか（＝表示名だけの変更が別のartifact identityを生むか）の決定が要る | 全6ケース × `rename` × `artifact` | 6 |
| U4 | Projectがartifact identityを固定するか | `docs/vism-kit-model.md` §1.1のProject Lock行に、作品再現のためにProjectがartifact identityを固定するかの決定が要る。固定しないなら欠落時にProject側へartifact identityは存在しない | 全6ケース × `missing` × `artifact` | 6 |
| U5 | 同一version再導入とartifactの同一性 | `docs/vism-package-concept.md` §4.2／§10「source / native binary / WGSLの同梱方式」に、同一versionの再導入が同一artifact identityを再現するか（Host build成果を同一artifactと見なすか）の決定が要る | 全6ケース × `reinstall` × `artifact` | 6 |

| U6 | capability entry identity の scope | `docs/vism-kit-model.md` の identity 表に「entry identity が package に閉じるか」の決定が要る。閉じるなら別packageのentryは別identity、閉じないなら二つのpackageが同じentry IDを持ちうる。`docs/vism-kit-model.md:300` [権威] は package identity を entry ID へ流用することを禁じ、entry IDがpackageから導出されないことは示すが、scopeは決めていない | 全6ケース × `fork差替え` × `capability entry` | 6 |

この6種はすべて`docs/reviews/2026-07-17-vism-implementation-plan.md` §3、`docs/vism-kit-model.md`、`docs/vism-package-concept.md`、`docs/decision-index.md` のVism関連行のいずれにも決定が無い。埋めるには上表の右列に名指しした文書へ決定を加える必要がある。ケース5・6を追加したこと自体は`UNDETERMINED`を増やさなかった。ケース固有の未決（一package複数entryの採否、kindを跨いだ参照key）は、既存のU1へ同じ形で収束するためである。

## 2. ケース1 — 一つのVismがFilter entryを一つ持つ

原文は `docs/reviews/2026-07-17-vism-implementation-plan.md:74` [権威]。単一のVism packageが単一のFilter entryを持ち、それを一つのProjectが使っている構成である。`Kit`はこの構成に現れない（`docs/reviews/2026-07-17-vism-implementation-plan.md:76` [権威] で初めて現れる）。現行コードでこの形に最も近いのは同梱の`core.filter.opacity`だが、`PluginId`（`crates/motolii-plugin/src/contract.rs:9` [現状]）はpackage identityではない。

| 操作 | identity | 期待値 | 根拠 |
|---|---|---|---|
| `rename` | `package` | 保持。表示名は配布上の識別子ではなく、package identityは配布・更新をまたぐ | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/vism-kit-model.md:294` [権威] |
| `rename` | `capability entry` | 保持。entry identityの所有者はVismの表現契約であり、packageの表示名ではない | `docs/reviews/2026-07-17-vism-implementation-plan.md:65` [権威]、`docs/vism-kit-model.md:295` [権威] |
| `rename` | `Kit` | 該当なし。ケース1は単一Vism・単一entryの構成で、Kit接続が現れない | `docs/reviews/2026-07-17-vism-implementation-plan.md:74` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:76` [権威] |
| `rename` | `Project instance` | 保持。instance identityはProject Documentが所有し、package側の属性から導出しない | `docs/reviews/2026-07-17-vism-implementation-plan.md:67` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:70` [権威] |
| `rename` | `artifact` | UNDETERMINED: `docs/vism-package-concept.md` §4.2に「表示名が配布artifactの内容に含まれるか（表示名だけの変更が別artifact identityを生むか）」の決定が要る | `docs/vism-package-concept.md:281` [権威]（manifestは未決）、`docs/vism-kit-model.md:298` [権威] |
| `update` | `package` | 保持。識別子は同じで、package identityは更新をまたぐ | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/vism-package-concept.md:55` [権威] |
| `update` | `capability entry` | UNDETERMINED: `docs/vism-package-concept.md` §4.1／§10「1 package内のcapability数」に、version更新をまたぐentry identityの保持範囲と、entry追加・削除・改名の可否、`selected capability / entry`の再解決規則の決定が要る | `docs/vism-package-concept.md:282` [権威]、`docs/vism-package-concept.md:202` [権威] |
| `update` | `Kit` | 該当なし。ケース1にKit接続が現れない | `docs/reviews/2026-07-17-vism-implementation-plan.md:74` [権威] |
| `update` | `Project instance` | 保持。instance identityをpackage versionから導出しない | `docs/reviews/2026-07-17-vism-implementation-plan.md:70` [権威]、`docs/vism-kit-model.md:297` [権威] |
| `update` | `artifact` | 変化。新しいversionは同じsource／版から得た別の実体であり、別のartifact identityになる | `docs/vism-kit-model.md:298` [権威] |
| `duplicate` | `package` | 保持。作品内の複製はProject Documentの操作であり、配布面へ触れない | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:67` [権威] |
| `duplicate` | `capability entry` | 保持。複製されるのはFilter entryを使うProject instanceであって、entryそのものではない | `docs/reviews/2026-07-17-vism-implementation-plan.md:65` [権威]、`docs/vism-kit-model.md:297` [権威] |
| `duplicate` | `Kit` | 該当なし。ケース1にKit接続が現れない | `docs/reviews/2026-07-17-vism-implementation-plan.md:74` [権威] |
| `duplicate` | `Project instance` | 原本は保持、複製側は新規採番。各instanceはProjectが採番する | `docs/reviews/2026-07-17-vism-implementation-plan.md:67` [権威]、`docs/vism-kit-model.md:300` [権威] |
| `duplicate` | `artifact` | 保持。作品内複製はbuild・検証・署名を起こさない | `docs/reviews/2026-07-17-vism-implementation-plan.md:68` [権威]、`docs/vism-kit-model.md:298` [権威] |
| `missing` | `package` | 保持。環境に実体が無くてもProjectが持つ参照は削られず、再導入で復元できる | `docs/vism-package-concept.md:210` [権威]、`docs/vism-package-concept.md:213` [権威] |
| `missing` | `capability entry` | 参照切れ。Projectが保存した`selected capability / entry`は保持されるが解決できず、該当表現だけunavailableになる | `docs/vism-package-concept.md:202` [権威]、`docs/vism-kit-model.md:62` [権威] |
| `missing` | `Kit` | 該当なし。ケース1にKit接続が現れない | `docs/reviews/2026-07-17-vism-implementation-plan.md:74` [権威] |
| `missing` | `Project instance` | 保持。原本を保持し、無関係なDocument領域の編集を許可する | `docs/vism-package-concept.md:210-211` [権威]、`docs/vism-kit-model.md:64` [権威] |
| `missing` | `artifact` | UNDETERMINED: `docs/vism-kit-model.md` §1.1のProject Lock行に「作品再現のためにProjectがartifact identityを固定するか」の決定が要る | `docs/vism-kit-model.md:48` [権威]、`docs/vism-package-concept.md:200-204` [権威]（Project参照集合にartifact identityが無い） |
| `reinstall` | `package` | 保持。再導入でユーザー整理を動かさないことがstable package identityの要件である | `docs/vism-package-concept.md:55` [権威]、`docs/vism-package-concept.md:213` [権威] |
| `reinstall` | `capability entry` | 保持し再解決される。互換Vismの再導入後、保持したpayloadから復元する | `docs/vism-package-concept.md:213` [権威] |
| `reinstall` | `Kit` | 該当なし。ケース1にKit接続が現れない | `docs/reviews/2026-07-17-vism-implementation-plan.md:74` [権威] |
| `reinstall` | `Project instance` | 保持。同一instanceへ復元する。役割に欠落復元が含まれる | `docs/vism-kit-model.md:297` [権威]、`docs/vism-package-concept.md:213` [権威] |
| `reinstall` | `artifact` | UNDETERMINED: `docs/vism-package-concept.md` §4.2／§10「source / native binary / WGSLの同梱方式」に「同一version再導入が同一artifact identityを再現するか」の決定が要る | `docs/vism-package-concept.md:284` [権威]、`docs/vism-package-concept.md:100` [権威] |
| `fork差替え` | `package` | 変化。参照先が別作者の別package identityへ移る。元のpackage identityは改名も消滅もしない | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/vism-package-concept.md:314` [権威] |
| `fork差替え` | `capability entry` | UNDETERMINED: `docs/vism-kit-model.md` の identity 表に「capability entry identity が package に閉じるか」の決定が要る。閉じるなら別 package の entry は別 identity、閉じないなら二つの package が同じ entry ID を持ちうる。`docs/vism-kit-model.md:300` [権威] は package identity を entry ID へ流用することを禁じており、entry ID が package から導出されないことは示すが、scope は決めていない | `docs/vism-kit-model.md:295` [権威]、`docs/vism-kit-model.md:300` [権威] |
| `fork差替え` | `Kit` | 該当なし。ケース1にKit接続が現れない | `docs/reviews/2026-07-17-vism-implementation-plan.md:74` [権威] |
| `fork差替え` | `Project instance` | UNDETERMINED: `docs/vism-kit-model.md` §5に「参照先packageをforkへ差し替えた時、既存Project instance identityを保持するか新規採番するか」の決定が要る | `docs/vism-kit-model.md:182` [権威]、`docs/vism-kit-model.md:179` [権威] |
| `fork差替え` | `artifact` | 変化。別packageの別実体であり、別のartifact identityになる | `docs/vism-kit-model.md:298` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:68` [権威] |

## 3. ケース2 — consumer Vismが`BeatEvents`相当の型だけを要求する

原文は `docs/reviews/2026-07-17-vism-implementation-plan.md:75` [権威]。consumerは具体providerのIDを参照せず必要な型を宣言する（`docs/vism-kit-model.md:23` [権威]、`docs/vism-package-concept.md:177` [権威]）。provider選択はKitの仕事なので、この段では`Kit`は現れない（`docs/reviews/2026-07-17-vism-implementation-plan.md:76` [権威]）。本ケースの操作対象packageはconsumer package自身である。

このケースは将来意味の比較fixtureであり、現行APIで実装可能とはみなさない（`docs/reviews/2026-07-17-vism-implementation-plan.md:81` [権威]、`docs/vism-kit-model.md:314` [権威]）。現行`ParamDriverPlugin`に入力portが無いことは `crates/motolii-plugin/src/traits.rs:44-52` [現状] のとおりで、これは期待値の根拠ではない。

| 操作 | identity | 期待値 | 根拠 |
|---|---|---|---|
| `rename` | `package` | 保持。表示名は配布上の識別子ではない | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/vism-kit-model.md:294` [権威] |
| `rename` | `capability entry` | 保持。consumer entryのidentityは表現契約が所有し、表示名から独立している | `docs/reviews/2026-07-17-vism-implementation-plan.md:65` [権威]、`docs/vism-package-concept.md:79` [権威] |
| `rename` | `Kit` | 該当なし。ケース2は型要求だけの段で、provider選択＝Kitはケース3で初めて現れる | `docs/reviews/2026-07-17-vism-implementation-plan.md:75` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:76` [権威] |
| `rename` | `Project instance` | 保持。instance identityはProject Documentが所有する | `docs/reviews/2026-07-17-vism-implementation-plan.md:67` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:70` [権威] |
| `rename` | `artifact` | UNDETERMINED: `docs/vism-package-concept.md` §4.2に「表示名が配布artifactの内容に含まれるか」の決定が要る | `docs/vism-package-concept.md:281` [権威]、`docs/vism-kit-model.md:298` [権威] |
| `update` | `package` | 保持。識別子は同じで、package identityは更新をまたぐ | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/vism-package-concept.md:55` [権威] |
| `update` | `capability entry` | UNDETERMINED: `docs/vism-package-concept.md` §4.1／§10「1 package内のcapability数」に、version更新をまたぐentry identityの保持範囲と、要求する型の宣言が変わった時の再解決規則の決定が要る | `docs/vism-package-concept.md:282` [権威]、`docs/vism-package-concept.md:202` [権威] |
| `update` | `Kit` | 該当なし。ケース2にprovider選択＝Kitが現れない | `docs/reviews/2026-07-17-vism-implementation-plan.md:75` [権威] |
| `update` | `Project instance` | 保持。instance identityをpackage versionから導出しない | `docs/reviews/2026-07-17-vism-implementation-plan.md:70` [権威]、`docs/vism-kit-model.md:297` [権威] |
| `update` | `artifact` | 変化。新しいversionは別の実体であり、別のartifact identityになる | `docs/vism-kit-model.md:298` [権威] |
| `duplicate` | `package` | 保持。作品内の複製はProject Documentの操作である | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:67` [権威] |
| `duplicate` | `capability entry` | 保持。複製されるのはconsumer entryを使うProject instanceである | `docs/reviews/2026-07-17-vism-implementation-plan.md:65` [権威]、`docs/vism-kit-model.md:297` [権威] |
| `duplicate` | `Kit` | 該当なし。ケース2にprovider選択＝Kitが現れない | `docs/reviews/2026-07-17-vism-implementation-plan.md:75` [権威] |
| `duplicate` | `Project instance` | 原本は保持、複製側は新規採番。各instanceはProjectが採番する | `docs/reviews/2026-07-17-vism-implementation-plan.md:67` [権威]、`docs/vism-kit-model.md:300` [権威] |
| `duplicate` | `artifact` | 保持。作品内複製はbuild・検証・署名を起こさない | `docs/reviews/2026-07-17-vism-implementation-plan.md:68` [権威]、`docs/vism-kit-model.md:298` [権威] |
| `missing` | `package` | 保持。consumer packageが欠けてもProjectの参照は削られない | `docs/vism-package-concept.md:210` [権威]、`docs/vism-package-concept.md:213` [権威] |
| `missing` | `capability entry` | 参照切れ。要求する型の宣言ごと解決できず、該当表現だけunavailableになる | `docs/vism-kit-model.md:62` [権威]、`docs/vism-package-concept.md:202` [権威] |
| `missing` | `Kit` | 該当なし。ケース2にprovider選択＝Kitが現れない | `docs/reviews/2026-07-17-vism-implementation-plan.md:75` [権威] |
| `missing` | `Project instance` | 保持。原本を保持し、無関係なDocument領域の編集を許可する | `docs/vism-package-concept.md:210-211` [権威]、`docs/vism-kit-model.md:64` [権威] |
| `missing` | `artifact` | UNDETERMINED: `docs/vism-kit-model.md` §1.1のProject Lock行に「Projectがartifact identityを固定するか」の決定が要る | `docs/vism-kit-model.md:48` [権威]、`docs/vism-package-concept.md:200-204` [権威] |
| `reinstall` | `package` | 保持。再導入でユーザー整理を動かさない | `docs/vism-package-concept.md:55` [権威]、`docs/vism-package-concept.md:213` [権威] |
| `reinstall` | `capability entry` | 保持し再解決される。保持したpayloadから復元する | `docs/vism-package-concept.md:213` [権威] |
| `reinstall` | `Kit` | 該当なし。ケース2にprovider選択＝Kitが現れない | `docs/reviews/2026-07-17-vism-implementation-plan.md:75` [権威] |
| `reinstall` | `Project instance` | 保持。同一instanceへ復元する | `docs/vism-kit-model.md:297` [権威]、`docs/vism-package-concept.md:213` [権威] |
| `reinstall` | `artifact` | UNDETERMINED: `docs/vism-package-concept.md` §4.2／§10「source / native binary / WGSLの同梱方式」に「同一version再導入が同一artifact identityを再現するか」の決定が要る | `docs/vism-package-concept.md:284` [権威]、`docs/vism-package-concept.md:100` [権威] |
| `fork差替え` | `package` | 変化。consumerを別作者のfork consumer packageへ差し替えると、参照先が別のpackage identityになる | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/vism-package-concept.md:314` [権威] |
| `fork差替え` | `capability entry` | UNDETERMINED: `docs/vism-kit-model.md` の identity 表に「capability entry identity が package に閉じるか」の決定が要る。閉じるなら別 package の entry は別 identity、閉じないなら二つの package が同じ entry ID を持ちうる。`docs/vism-kit-model.md:300` [権威] は package identity を entry ID へ流用することを禁じており、entry ID が package から導出されないことは示すが、scope は決めていない | `docs/vism-kit-model.md:295` [権威]、`docs/vism-kit-model.md:300` [権威] |
| `fork差替え` | `Kit` | 該当なし。ケース2にprovider選択＝Kitが現れない | `docs/reviews/2026-07-17-vism-implementation-plan.md:75` [権威] |
| `fork差替え` | `Project instance` | UNDETERMINED: `docs/vism-kit-model.md` §5に「参照先packageをforkへ差し替えた時、既存Project instance identityを保持するか新規採番するか」の決定が要る | `docs/vism-kit-model.md:182` [権威]、`docs/vism-kit-model.md:179` [権威] |
| `fork差替え` | `artifact` | 変化。別packageの別実体であり、別のartifact identityになる | `docs/vism-kit-model.md:298` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:68` [権威] |

## 4. ケース3 — Kitがprovider Vismとconsumer Vismを選んで接続し、Projectへmaterializeする

原文は `docs/reviews/2026-07-17-vism-implementation-plan.md:76` [権威]。5つのidentityがすべて現れる唯一の最小構成である。v1 Kitはmaterialize型で、展開後はKit runtimeがなくても通常のProject意味が残り、Kit更新で既存Projectを自動変更しない（`docs/vism-kit-model.md:178-179` [権威]）。展開されたVismのidentity、version、payloadはProjectが通常規則で保持する（`docs/vism-kit-model.md:180` [権威]）。操作は接続されたprovider packageに対して適用したものとして読む。

| 操作 | identity | 期待値 | 根拠 |
|---|---|---|---|
| `rename` | `package` | 保持。表示名は配布上の識別子ではない | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/vism-kit-model.md:294` [権威] |
| `rename` | `capability entry` | 保持。Kitが記録した型付き接続はentry identityを見ており、表示名を見ていない | `docs/vism-kit-model.md:295` [権威]、`docs/vism-kit-model.md:101` [権威] |
| `rename` | `Kit` | 保持。Kit identityはKit作者／配布系が所有し、構成要素の表示名から独立している | `docs/reviews/2026-07-17-vism-implementation-plan.md:66` [権威]、`docs/vism-kit-model.md:296` [権威] |
| `rename` | `Project instance` | 保持。materialize後の各instanceはProjectが採番したものである | `docs/vism-kit-model.md:300` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:70` [権威] |
| `rename` | `artifact` | UNDETERMINED: `docs/vism-package-concept.md` §4.2に「表示名が配布artifactの内容に含まれるか」の決定が要る | `docs/vism-package-concept.md:281` [権威]、`docs/vism-kit-model.md:298` [権威] |
| `update` | `package` | 保持。識別子は同じで、package identityは更新をまたぐ | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/vism-package-concept.md:55` [権威] |
| `update` | `capability entry` | UNDETERMINED: `docs/vism-package-concept.md` §4.1／§10「1 package内のcapability数」に、version更新をまたぐentry identityの保持範囲と、Kitが宣言した各entryの型付きinput／output対応をどう再解決するかの決定が要る | `docs/vism-package-concept.md:282` [権威]、`docs/vism-kit-model.md:101` [権威] |
| `update` | `Kit` | 保持。Kitは必要なVism identityと互換versionを宣言するだけで、Kit identityはpackage versionから導出しない | `docs/vism-kit-model.md:101` [権威]、`docs/vism-kit-model.md:296` [権威] |
| `update` | `Project instance` | 保持。instance identityをpackage versionから導出せず、展開されたVismのidentity・version・payloadはProjectが通常規則で保持する | `docs/reviews/2026-07-17-vism-implementation-plan.md:70` [権威]、`docs/vism-kit-model.md:180` [権威] |
| `update` | `artifact` | 変化。新しいversionは別の実体であり、別のartifact identityになる | `docs/vism-kit-model.md:298` [権威] |
| `duplicate` | `package` | 保持。作品内の複製はProject Documentの操作であり、配布面へ触れない | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:67` [権威] |
| `duplicate` | `capability entry` | 保持。複製されるのは展開済みinstanceであってentryではない | `docs/reviews/2026-07-17-vism-implementation-plan.md:65` [権威]、`docs/vism-kit-model.md:297` [権威] |
| `duplicate` | `Kit` | 保持。展開後はKit runtimeがなくても通常のProject意味が残るため、複製はKitに触れない | `docs/vism-kit-model.md:178` [権威]、`docs/vism-kit-model.md:296` [権威] |
| `duplicate` | `Project instance` | 原本は保持、複製側は新規採番。Kit identityをProject instance identityへ流用しない | `docs/vism-kit-model.md:300` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:67` [権威] |
| `duplicate` | `artifact` | 保持。作品内複製はbuild・検証・署名を起こさない | `docs/reviews/2026-07-17-vism-implementation-plan.md:68` [権威]、`docs/vism-kit-model.md:298` [権威] |
| `missing` | `package` | 保持。展開前ならKitが依存不足として診断し、展開後ならProjectの参照が削られずに残る | `docs/vism-kit-model.md:63` [権威]、`docs/vism-package-concept.md:210` [権威] |
| `missing` | `capability entry` | 参照切れ。該当表現だけunavailableになり、接続先のconsumerは原本のまま残る | `docs/vism-kit-model.md:62` [権威]、`docs/vism-package-concept.md:202` [権威] |
| `missing` | `Kit` | 保持。Kit identityは残り、欠落は展開前の依存不足診断として現れる。展開後はKitなしでProject意味が残る | `docs/vism-kit-model.md:63` [権威]、`docs/vism-kit-model.md:178` [権威] |
| `missing` | `Project instance` | 保持。原本を保持し、無関係なDocument領域の編集を許可する | `docs/vism-package-concept.md:210-211` [権威]、`docs/vism-kit-model.md:64` [権威] |
| `missing` | `artifact` | UNDETERMINED: `docs/vism-kit-model.md` §1.1のProject Lock行に「Projectがartifact identityを固定するか」の決定が要る | `docs/vism-kit-model.md:48` [権威]、`docs/vism-package-concept.md:200-204` [権威] |
| `reinstall` | `package` | 保持。再導入でユーザー整理を動かさない | `docs/vism-package-concept.md:55` [権威]、`docs/vism-package-concept.md:213` [権威] |
| `reinstall` | `capability entry` | 保持し再解決される。互換Vismの再導入後、保持したpayloadから復元する | `docs/vism-package-concept.md:213` [権威] |
| `reinstall` | `Kit` | 保持。再導入はinstall storeの操作であり、Kitの版に触れない | `docs/vism-kit-model.md:296` [権威]、`docs/vism-package-concept.md:215` [権威] |
| `reinstall` | `Project instance` | 保持。同一instanceへ復元する | `docs/vism-kit-model.md:297` [権威]、`docs/vism-package-concept.md:213` [権威] |
| `reinstall` | `artifact` | UNDETERMINED: `docs/vism-package-concept.md` §4.2／§10「source / native binary / WGSLの同梱方式」に「同一version再導入が同一artifact identityを再現するか」の決定が要る | `docs/vism-package-concept.md:284` [権威]、`docs/vism-package-concept.md:100` [権威] |
| `fork差替え` | `package` | 変化。provider参照が別作者の別package identityへ移る。consumer packageは保持される | `docs/vism-kit-model.md:238` [権威]、`docs/vism-package-concept.md:314` [権威] |
| `fork差替え` | `capability entry` | UNDETERMINED: `docs/vism-kit-model.md` の identity 表に「capability entry identity が package に閉じるか」の決定が要る。閉じるなら別 package の entry は別 identity、閉じないなら二つの package が同じ entry ID を持ちうる。`docs/vism-kit-model.md:300` [権威] は package identity を entry ID へ流用することを禁じており、entry ID が package から導出されないことは示すが、scope は決めていない | `docs/vism-kit-model.md:295` [権威]、`docs/vism-kit-model.md:300` [権威] |
| `fork差替え` | `Kit` | 保持。展開後の差替えは通常のProject編集であり、既に使ったKitの版を変えない | `docs/vism-kit-model.md:178` [権威]、`docs/vism-kit-model.md:179` [権威] |
| `fork差替え` | `Project instance` | UNDETERMINED: `docs/vism-kit-model.md` §5に「materialize済みProjectのprovider packageをforkへ差し替えた時、置換される側のProject instance identityを保持するか新規採番するか」の決定が要る。consumer側instanceが不変であることは `docs/vism-kit-model.md:238` [権威] が示すが、置換側は未決 | `docs/vism-kit-model.md:182` [権威]、`docs/vism-kit-model.md:238` [権威] |
| `fork差替え` | `artifact` | 変化。別packageの別実体であり、別のartifact identityになる | `docs/vism-kit-model.md:298` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:68` [権威] |

## 5. ケース4 — fork Kitが同じconsumerへ別providerを接続する

原文は `docs/reviews/2026-07-17-vism-implementation-plan.md:77` [権威]。標準Kitとfork Kitが同じconsumerへ別のproviderを接続する（`docs/vism-kit-model.md:234-235` [権威]）。consumer Vismを変えずproviderだけを差し替えられるのが望ましい（`docs/vism-kit-model.md:238` [権威]）。したがって本ケースには**二つのKit identity**が同時に存在する。操作はproviderのpackageに対して適用したものとして読む。

| 操作 | identity | 期待値 | 根拠 |
|---|---|---|---|
| `rename` | `package` | 保持。表示名は配布上の識別子ではない | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/vism-kit-model.md:294` [権威] |
| `rename` | `capability entry` | 保持。どちらのKitから見てもentry identityは表現契約の所有物である | `docs/vism-kit-model.md:295` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:65` [権威] |
| `rename` | `Kit` | 保持。標準Kit・fork Kitとも自分のKit identityを保つ | `docs/vism-kit-model.md:296` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:66` [権威] |
| `rename` | `Project instance` | 保持。instance identityはProject Documentが所有する | `docs/reviews/2026-07-17-vism-implementation-plan.md:67` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:70` [権威] |
| `rename` | `artifact` | UNDETERMINED: `docs/vism-package-concept.md` §4.2に「表示名が配布artifactの内容に含まれるか」の決定が要る | `docs/vism-package-concept.md:281` [権威]、`docs/vism-kit-model.md:298` [権威] |
| `update` | `package` | 保持。識別子は同じで、package identityは更新をまたぐ | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/vism-package-concept.md:55` [権威] |
| `update` | `capability entry` | UNDETERMINED: `docs/vism-package-concept.md` §4.1／§10「1 package内のcapability数」に、version更新をまたぐentry identityの保持範囲と、両Kitが宣言した互換versionの範囲外へ出た時の再解決規則の決定が要る | `docs/vism-package-concept.md:282` [権威]、`docs/vism-kit-model.md:101` [権威] |
| `update` | `Kit` | 保持。fork固有能力は名前空間、version、非互換理由を宣言するが、それはKit identityではない | `docs/vism-kit-model.md:238` [権威]、`docs/vism-kit-model.md:296` [権威] |
| `update` | `Project instance` | 保持。instance identityをpackage versionから導出しない | `docs/reviews/2026-07-17-vism-implementation-plan.md:70` [権威]、`docs/vism-kit-model.md:180` [権威] |
| `update` | `artifact` | 変化。新しいversionは別の実体であり、別のartifact identityになる | `docs/vism-kit-model.md:298` [権威] |
| `duplicate` | `package` | 保持。作品内の複製はProject Documentの操作である | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:67` [権威] |
| `duplicate` | `capability entry` | 保持。複製されるのは展開済みinstanceであってentryではない | `docs/vism-kit-model.md:297` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:65` [権威] |
| `duplicate` | `Kit` | 保持。どちらのKitも展開済みで、複製はKit runtimeを必要としない | `docs/vism-kit-model.md:178` [権威]、`docs/vism-kit-model.md:296` [権威] |
| `duplicate` | `Project instance` | 原本は保持、複製側は新規採番。Kit identityをProject instance identityへ流用しない | `docs/vism-kit-model.md:300` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:67` [権威] |
| `duplicate` | `artifact` | 保持。作品内複製はbuild・検証・署名を起こさない | `docs/reviews/2026-07-17-vism-implementation-plan.md:68` [権威]、`docs/vism-kit-model.md:298` [権威] |
| `missing` | `package` | 保持。片方のproviderが欠けても参照は削られず、もう一方のKitが選んだproviderには波及しない | `docs/vism-package-concept.md:210` [権威]、`docs/vism-package-concept.md:314` [権威] |
| `missing` | `capability entry` | 参照切れ。欠けたprovider側entryだけがunavailableになり、consumer entryは保持される | `docs/vism-kit-model.md:62` [権威]、`docs/vism-kit-model.md:23` [権威] |
| `missing` | `Kit` | 保持。欠落は展開前の依存不足診断として現れ、Kit identity自体は残る | `docs/vism-kit-model.md:63` [権威]、`docs/vism-kit-model.md:296` [権威] |
| `missing` | `Project instance` | 保持。原本を保持し、無関係なDocument領域の編集を許可する | `docs/vism-package-concept.md:210-211` [権威]、`docs/vism-kit-model.md:64` [権威] |
| `missing` | `artifact` | UNDETERMINED: `docs/vism-kit-model.md` §1.1のProject Lock行に「Projectがartifact identityを固定するか」の決定が要る | `docs/vism-kit-model.md:48` [権威]、`docs/vism-package-concept.md:200-204` [権威] |
| `reinstall` | `package` | 保持。再導入でユーザー整理を動かさない | `docs/vism-package-concept.md:55` [権威]、`docs/vism-package-concept.md:213` [権威] |
| `reinstall` | `capability entry` | 保持し再解決される。保持したpayloadから復元する | `docs/vism-package-concept.md:213` [権威] |
| `reinstall` | `Kit` | 保持。再導入はinstall storeの操作であり、どちらのKitの版にも触れない | `docs/vism-kit-model.md:296` [権威]、`docs/vism-package-concept.md:215` [権威] |
| `reinstall` | `Project instance` | 保持。同一instanceへ復元する | `docs/vism-kit-model.md:297` [権威]、`docs/vism-package-concept.md:213` [権威] |
| `reinstall` | `artifact` | UNDETERMINED: `docs/vism-package-concept.md` §4.2／§10「source / native binary / WGSLの同梱方式」に「同一version再導入が同一artifact identityを再現するか」の決定が要る | `docs/vism-package-concept.md:284` [権威]、`docs/vism-package-concept.md:100` [権威] |
| `fork差替え` | `package` | 変化。参照するproviderが別作者の別package identityへ移る。consumer packageは保持される | `docs/vism-kit-model.md:234-235` [権威]、`docs/vism-kit-model.md:238` [権威] |
| `fork差替え` | `capability entry` | UNDETERMINED: `docs/vism-kit-model.md` の identity 表に「capability entry identity が package に閉じるか」の決定が要る。閉じるなら別 package の entry は別 identity、閉じないなら二つの package が同じ entry ID を持ちうる。`docs/vism-kit-model.md:300` [権威] は package identity を entry ID へ流用することを禁じており、entry ID が package から導出されないことは示すが、scope は決めていない | `docs/vism-kit-model.md:295` [権威]、`docs/vism-kit-model.md:300` [権威] |
| `fork差替え` | `Kit` | 保持。展開後の差替えは通常のProject編集であり、既に使ったKitの版を変えない。標準Kitとfork Kitが別identityを持つのはケース4の構成であって、この操作の効果ではない | `docs/vism-kit-model.md:178` [権威]、`docs/vism-kit-model.md:179` [権威] |
| `fork差替え` | `Project instance` | UNDETERMINED: `docs/vism-kit-model.md` §5に「materialize済みProjectのprovider packageをforkへ差し替えた時、置換される側のProject instance identityを保持するか新規採番するか」の決定が要る。consumer側instanceが不変であることは `docs/vism-kit-model.md:238` [権威] が示すが、置換側は未決 | `docs/vism-kit-model.md:182` [権威]、`docs/vism-kit-model.md:238` [権威] |
| `fork差替え` | `artifact` | 変化。別packageの別実体であり、別のartifact identityになる | `docs/vism-kit-model.md:298` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:68` [権威] |

## 6. ケース5 — 一つのpackageが異なるkindのentryを複数持つ

本発注で追加した軸である。`docs/reviews/2026-07-17-vism-implementation-plan.md:79` [権威] は「一package複数entryは同じlifecycle／compatibility責任から分離できない場合だけ比較する」として未決に置き、`docs/vism-package-concept.md:42` [権威] も「複数entryは同一lifecycle／compatibility責任から分離できない場合の候補であり、万能bundleの既定にはしない」としている。`docs/vism-package-concept.md:282` [権威] では「1 package内のcapability数」自体が未決である。**本節はこの構成が成立すると仮定した場合の期待値を言語化するだけで、採否は決めない。**

仮定から直ちに従うのは「同一package内の全entryは同じlifecycleとcompatibility責任を共有する」ことだけである（`docs/reviews/2026-07-17-vism-implementation-plan.md:79` [権威]）。Kit接続はこのケースに現れない。

現行コードでは一つのplugin idが `vendor.kind.name` 形式でちょうど一つのkindを表し、中央セグメントが登録kindと一致しないと拒否される（`crates/motolii-plugin/src/contract.rs:467-492` [現状]）。これは現状記述であり、期待値の根拠ではない。

| 操作 | identity | 期待値 | 根拠 |
|---|---|---|---|
| `rename` | `package` | 保持。表示名は配布上の識別子ではなく、entry数と無関係である | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/vism-kit-model.md:294` [権威] |
| `rename` | `capability entry` | 保持。package内の全kindのentryが同時に影響を受けない。entry identityの所有者は表現契約である | `docs/vism-kit-model.md:295` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:65` [権威] |
| `rename` | `Kit` | 該当なし。ケース5は単一package内部のentry構成であり、Kit接続を含まない | `docs/reviews/2026-07-17-vism-implementation-plan.md:79` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:76` [権威] |
| `rename` | `Project instance` | 保持。どのkindのentryを使うinstanceもProject Documentが所有する | `docs/reviews/2026-07-17-vism-implementation-plan.md:67` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:70` [権威] |
| `rename` | `artifact` | UNDETERMINED: `docs/vism-package-concept.md` §4.2に「表示名が配布artifactの内容に含まれるか」の決定が要る | `docs/vism-package-concept.md:281` [権威]、`docs/vism-kit-model.md:298` [権威] |
| `update` | `package` | 保持。識別子は同じで、package identityは更新をまたぐ。複数kindのentryを持っても分割されない | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:79` [権威] |
| `update` | `capability entry` | UNDETERMINED: `docs/vism-package-concept.md` §4.1／§10「1 package内のcapability数」に、version更新をまたぐentry identityの保持範囲と、kindごとのentryを個別に追加・削除・改名できるかの決定が要る。`docs/reviews/2026-07-17-vism-implementation-plan.md:79` [権威] により全entryが同一lifecycleを共有する点だけは確定している | `docs/vism-package-concept.md:282` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:79` [権威] |
| `update` | `Kit` | 該当なし。ケース5は単一package内部のentry構成であり、Kit接続を含まない | `docs/reviews/2026-07-17-vism-implementation-plan.md:79` [権威] |
| `update` | `Project instance` | 保持。instance identityをpackage versionから導出しない。異なるkindのentryを使う複数instanceが同時に更新をまたぐ | `docs/reviews/2026-07-17-vism-implementation-plan.md:70` [権威]、`docs/vism-kit-model.md:180` [権威] |
| `update` | `artifact` | 変化。全entryが同一lifecycleを共有するため、一つの新しいartifact identityへまとめて移る | `docs/vism-kit-model.md:298` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:79` [権威] |
| `duplicate` | `package` | 保持。作品内の複製はProject Documentの操作である | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:67` [権威] |
| `duplicate` | `capability entry` | 保持。複製されるのは一つのentryを使うProject instanceであって、package内のentry集合ではない | `docs/vism-kit-model.md:297` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:65` [権威] |
| `duplicate` | `Kit` | 該当なし。ケース5は単一package内部のentry構成であり、Kit接続を含まない | `docs/reviews/2026-07-17-vism-implementation-plan.md:79` [権威] |
| `duplicate` | `Project instance` | 複製した箇所のinstanceだけ新規採番、原本は保持。同一packageの別kind entryを使う別instanceは影響を受けない | `docs/vism-kit-model.md:300` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:67` [権威] |
| `duplicate` | `artifact` | 保持。作品内複製はbuild・検証・署名を起こさない | `docs/reviews/2026-07-17-vism-implementation-plan.md:68` [権威]、`docs/vism-kit-model.md:298` [権威] |
| `missing` | `package` | 保持。参照は削られず、再導入で復元できる | `docs/vism-package-concept.md:210` [権威]、`docs/vism-package-concept.md:213` [権威] |
| `missing` | `capability entry` | 参照切れ。全kindのentryが同時に解決できなくなる（全entryが同一lifecycleを共有するため）。該当表現だけunavailableになる | `docs/reviews/2026-07-17-vism-implementation-plan.md:79` [権威]、`docs/vism-kit-model.md:62` [権威] |
| `missing` | `Kit` | 該当なし。ケース5は単一package内部のentry構成であり、Kit接続を含まない | `docs/reviews/2026-07-17-vism-implementation-plan.md:79` [権威] |
| `missing` | `Project instance` | 保持。どのkindのentryを使うinstanceも原本を保持し、無関係なDocument領域の編集を許可する | `docs/vism-package-concept.md:210-211` [権威]、`docs/vism-kit-model.md:64` [権威] |
| `missing` | `artifact` | UNDETERMINED: `docs/vism-kit-model.md` §1.1のProject Lock行に「Projectがartifact identityを固定するか」の決定が要る | `docs/vism-kit-model.md:48` [権威]、`docs/vism-package-concept.md:200-204` [権威] |
| `reinstall` | `package` | 保持。再導入でユーザー整理を動かさない | `docs/vism-package-concept.md:55` [権威]、`docs/vism-package-concept.md:213` [権威] |
| `reinstall` | `capability entry` | 保持。全kindのentryが同時に復元され、保持したpayloadから復元する | `docs/vism-package-concept.md:213` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:79` [権威] |
| `reinstall` | `Kit` | 該当なし。ケース5は単一package内部のentry構成であり、Kit接続を含まない | `docs/reviews/2026-07-17-vism-implementation-plan.md:79` [権威] |
| `reinstall` | `Project instance` | 保持。全kindのentryを使う各instanceが同一instanceへ復元する | `docs/vism-kit-model.md:297` [権威]、`docs/vism-package-concept.md:213` [権威] |
| `reinstall` | `artifact` | UNDETERMINED: `docs/vism-package-concept.md` §4.2／§10「source / native binary / WGSLの同梱方式」に「同一version再導入が同一artifact identityを再現するか」の決定が要る | `docs/vism-package-concept.md:284` [権威]、`docs/vism-package-concept.md:100` [権威] |
| `fork差替え` | `package` | 変化。参照先が別作者の別package identityへ移る。fork側が同じkind組のentryを備える保証は無い | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/vism-package-concept.md:282` [権威] |
| `fork差替え` | `capability entry` | UNDETERMINED: `docs/vism-kit-model.md` の identity 表に「capability entry identity が package に閉じるか」の決定が要る。閉じるなら別 package の entry は別 identity、閉じないなら二つの package が同じ entry ID を持ちうる。`docs/vism-kit-model.md:300` [権威] は package identity を entry ID へ流用することを禁じており、entry ID が package から導出されないことは示すが、scope は決めていない | `docs/vism-kit-model.md:295` [権威]、`docs/vism-kit-model.md:300` [権威] |
| `fork差替え` | `Kit` | 該当なし。ケース5は単一package内部のentry構成であり、Kit接続を含まない | `docs/reviews/2026-07-17-vism-implementation-plan.md:79` [権威] |
| `fork差替え` | `Project instance` | UNDETERMINED: `docs/vism-kit-model.md` §5に「参照先packageをforkへ差し替えた時、既存Project instance identityを保持するか新規採番するか」の決定が要る。複数kindのinstanceが同時に差し替わる場合の扱いも同じ決定に属する | `docs/vism-kit-model.md:182` [権威]、`docs/vism-kit-model.md:179` [権威] |
| `fork差替え` | `artifact` | 変化。別packageの別実体であり、別のartifact identityになる | `docs/vism-kit-model.md:298` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:68` [権威] |

## 7. ケース6 — ケース5のpackageのentryを、別packageのentryがKitを介して参照する

本発注で追加した軸である。参照の形はKitを介した接続とし、直接参照は扱わない（ケース3・4と同じ接続方式で、providerとconsumerのkindが異なる場合である）。Vismは別VismのIDを直接要求せず必要な型を宣言し、Kitが具体的なproviderを選ぶ（`docs/vism-kit-model.md:23` [権威]、`docs/vism-package-concept.md:177` [権威]）。typed interfaceは構成の後段でも再利用でき、組み合わせごとに専用kindを増やさない（`docs/vism-kit-model.md:112-114` [権威]）。

ケース5と同じく、一package複数entryの成否そのものは未決である（`docs/vism-package-concept.md:282` [権威]）。またprovider→consumer接続の**方式決定**は`VSM-B2`であり、本書では決めない（`docs/reviews/2026-07-17-vism-implementation-plan.md:162` [権威]、`docs/vism-kit-model.md:314` [権威]）。kindを跨いだ参照が何をkeyにするか（package＋entry identityか、型だけか）は`docs/decision-index.md:301` [権威] がVSM-B0/B2の比較対象として公開schema化を停止している。

現行コードには対応物が無い。`ParamDriverPlugin`に入力portは無く（`crates/motolii-plugin/src/traits.rs:44-52` [現状]）、id中央セグメントはちょうど一つのkindに縛られる（`crates/motolii-plugin/src/contract.rs:467-492` [現状]）。いずれも現状記述であり、期待値の根拠ではない。

操作は**参照される側**（ケース5のpackage＝provider）に対して適用したものとして読む。

| 操作 | identity | 期待値 | 根拠 |
|---|---|---|---|
| `rename` | `package` | 保持。表示名は配布上の識別子ではなく、参照側からの解決に影響しない | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/vism-kit-model.md:294` [権威] |
| `rename` | `capability entry` | 保持。参照側は型を宣言しKitがproviderを選ぶ構成であり、どちらの側のentry identityも表示名から独立している | `docs/vism-kit-model.md:23` [権威]、`docs/vism-kit-model.md:295` [権威] |
| `rename` | `Kit` | 保持。Kitはkindの異なるprovider entryとconsumer entryの型付きinput／output対応を宣言するだけで、表示名を宣言しない | `docs/vism-kit-model.md:101` [権威]、`docs/vism-kit-model.md:296` [権威] |
| `rename` | `Project instance` | 保持。materialize後の各instanceはProjectが採番したものである | `docs/vism-kit-model.md:300` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:70` [権威] |
| `rename` | `artifact` | UNDETERMINED: `docs/vism-package-concept.md` §4.2に「表示名が配布artifactの内容に含まれるか」の決定が要る | `docs/vism-package-concept.md:281` [権威]、`docs/vism-kit-model.md:298` [権威] |
| `update` | `package` | 保持。識別子は同じで、package identityは更新をまたぐ。参照側のpackage identityも独立して保持される | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/vism-package-concept.md:314` [権威] |
| `update` | `capability entry` | UNDETERMINED: `docs/vism-package-concept.md` §4.1／§10「1 package内のcapability数」と`docs/decision-index.md:301` のDataTrack identity比較（VSM-B0/B2）に、version更新をまたぐentry identityの保持範囲と、kindを跨いだ参照が何をkeyにするか（package＋entry identityか型だけか）の決定が要る | `docs/vism-package-concept.md:282` [権威]、`docs/decision-index.md:301` [権威] |
| `update` | `Kit` | 保持。Kitは必要なVism identityと互換versionを宣言するだけで、Kit identityはprovider versionから導出しない | `docs/vism-kit-model.md:101` [権威]、`docs/vism-kit-model.md:296` [権威] |
| `update` | `Project instance` | 保持。instance identityをpackage versionから導出せず、展開されたVismのidentity・version・payloadはProjectが通常規則で保持する | `docs/reviews/2026-07-17-vism-implementation-plan.md:70` [権威]、`docs/vism-kit-model.md:180` [権威] |
| `update` | `artifact` | 変化。provider packageの全entryが同一lifecycleで一つの新しいartifact identityへ移る。参照側packageのartifact identityは変わらない | `docs/vism-kit-model.md:298` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:79` [権威] |
| `duplicate` | `package` | 保持。作品内の複製はProject Documentの操作であり、参照関係の配布面へ触れない | `docs/reviews/2026-07-17-vism-implementation-plan.md:64` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:67` [権威] |
| `duplicate` | `capability entry` | 保持。複製されるのは接続済みinstanceであって、provider entryでもconsumer entryでもない | `docs/vism-kit-model.md:297` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:65` [権威] |
| `duplicate` | `Kit` | 保持。展開後はKit runtimeがなくても通常のProject意味が残るため、複製はKitに触れない | `docs/vism-kit-model.md:178` [権威]、`docs/vism-kit-model.md:296` [権威] |
| `duplicate` | `Project instance` | 原本は保持、複製側は新規採番。Kit identityをProject instance identityへ流用しない | `docs/vism-kit-model.md:300` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:67` [権威] |
| `duplicate` | `artifact` | 保持。作品内複製はbuild・検証・署名を起こさない | `docs/reviews/2026-07-17-vism-implementation-plan.md:68` [権威]、`docs/vism-kit-model.md:298` [権威] |
| `missing` | `package` | 保持。provider packageが欠けても参照は削られず、参照側packageのidentityは無傷で残る | `docs/vism-package-concept.md:210` [権威]、`docs/vism-package-concept.md:314` [権威] |
| `missing` | `capability entry` | 参照切れ（provider側）。参照側entryは保持される。consumerは型を宣言し具体providerのentry IDを参照しないためである | `docs/vism-kit-model.md:23` [権威]、`docs/vism-kit-model.md:62` [権威] |
| `missing` | `Kit` | 保持。展開前ならKitが依存不足として診断し、展開後はKitなしでProject意味が残る | `docs/vism-kit-model.md:63` [権威]、`docs/vism-kit-model.md:178` [権威] |
| `missing` | `Project instance` | 保持。原本を保持し、無関係なDocument領域の編集を許可する。参照側instanceも保持される | `docs/vism-package-concept.md:210-211` [権威]、`docs/vism-kit-model.md:64` [権威] |
| `missing` | `artifact` | UNDETERMINED: `docs/vism-kit-model.md` §1.1のProject Lock行に「Projectがartifact identityを固定するか」の決定が要る | `docs/vism-kit-model.md:48` [権威]、`docs/vism-package-concept.md:200-204` [権威] |
| `reinstall` | `package` | 保持。再導入でユーザー整理を動かさない | `docs/vism-package-concept.md:55` [権威]、`docs/vism-package-concept.md:213` [権威] |
| `reinstall` | `capability entry` | 保持。provider packageの全kindのentryが同時に復元され、参照側は保持したpayloadから復元する | `docs/vism-package-concept.md:213` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:79` [権威] |
| `reinstall` | `Kit` | 保持。再導入はinstall storeの操作であり、Kitの版に触れない | `docs/vism-kit-model.md:296` [権威]、`docs/vism-package-concept.md:215` [権威] |
| `reinstall` | `Project instance` | 保持。provider側・参照側とも同一instanceへ復元する | `docs/vism-kit-model.md:297` [権威]、`docs/vism-package-concept.md:213` [権威] |
| `reinstall` | `artifact` | UNDETERMINED: `docs/vism-package-concept.md` §4.2／§10「source / native binary / WGSLの同梱方式」に「同一version再導入が同一artifact identityを再現するか」の決定が要る | `docs/vism-package-concept.md:284` [権威]、`docs/vism-package-concept.md:100` [権威] |
| `fork差替え` | `package` | 変化。provider参照が別作者の別package identityへ移る。参照側packageは保持される | `docs/vism-kit-model.md:238` [権威]、`docs/vism-package-concept.md:314` [権威] |
| `fork差替え` | `capability entry` | UNDETERMINED: `docs/vism-kit-model.md` の identity 表に「capability entry identity が package に閉じるか」の決定が要る。閉じるなら別 package の entry は別 identity、閉じないなら二つの package が同じ entry ID を持ちうる。`docs/vism-kit-model.md:300` [権威] は package identity を entry ID へ流用することを禁じており、entry ID が package から導出されないことは示すが、scope は決めていない | `docs/vism-kit-model.md:295` [権威]、`docs/vism-kit-model.md:300` [権威] |
| `fork差替え` | `Kit` | 保持。展開後の差替えは通常のProject編集であり、既に使ったKitの版を変えない | `docs/vism-kit-model.md:178` [権威]、`docs/vism-kit-model.md:179` [権威] |
| `fork差替え` | `Project instance` | UNDETERMINED: `docs/vism-kit-model.md` §5に「materialize済みProjectのprovider packageをforkへ差し替えた時、置換される側のProject instance identityを保持するか新規採番するか」の決定が要る。参照側instanceが不変であることは `docs/vism-kit-model.md:238` [権威] が示すが、置換側は未決 | `docs/vism-kit-model.md:182` [権威]、`docs/vism-kit-model.md:238` [権威] |
| `fork差替え` | `artifact` | 変化。別packageの別実体であり、別のartifact identityになる | `docs/vism-kit-model.md:298` [権威]、`docs/reviews/2026-07-17-vism-implementation-plan.md:68` [権威] |

## 8. 反対側レビューの結果（2026-08-17、Grok 4.6 xhigh、read-only）

起草と別familyの検査を一度通した。問いは「`UNDETERMINED` 30件は本当に未決か、それとも読み落としか」の一点に絞った。

**未決5問は全て `CONFIRMED-UNDETERMINED`。** 検査側は本書のREAD SETより広く、`2026-07-17-vism-a0d-contract-migration-ownership-decision.md`、`2026-07-17-vism-a0s-contract-catalog-spec.md`、`2026-07-27-vism-authoring-journey-decision.md`、`community-distribution-model.md`、`2026-07-23-vism-kit-rack-unification-decision.md`、`plugin-authoring.md` 他を辿った上で、いずれも答えが無いことを確認した。近傍に見える記述（A0Dの「entry ID + version を分ける」、plugin-authoringの「idはリネームしない」）は、`2026-07-17-vism-a0s-contract-catalog-spec.md:467` と本書AUTHORITY `:70` が Vism identity への流用を先に禁じているため答えにならない。

**誤りは埋まっている側にあった。** ケース4の `fork差替え × Kit` が、根拠として当該ケースの定義文（`2026-07-17-vism-implementation-plan.md:77`）とその例示（`vism-kit-model.md:234-235`）を引いており、循環していた。同じ操作がケース3・6では `vism-kit-model.md:178-179` を根拠に `保持` になっていたため、同一台帳から結論が割れていた。本書では `保持` へ揃え、ケース構成と操作の効果を書き分けた。

**指摘3件は処理した。**

1. `fork差替え × capability entry = 変化` — 全6ケースを `UNDETERMINED`（U6）へ倒した。`docs/vism-kit-model.md:295`（所有者）と `:300`（package identityをentry IDへ流用しない）は、fork packageが別のentry identityを持つことを述べていない。`:300` はむしろentry IDがpackageから導出されないことを示しており、二つのpackageが同じentry IDを持つ余地を閉じていない。entry identityがpackageに閉じるかが未決である以上、`変化` と断定できない。
2. `update × artifact = 変化` — 第二引用 `docs/vism-package-concept.md:123`（sourceとimmutable artifactの配布topology分離）を落とした。第一引用 `docs/vism-kit-model.md:298`（「同じsource／版から得た実体の由来」）だけで、版が変われば別実体になることは支えられる。値は変えていない。
3. `reinstall × capability entry = 保持` — 第二引用 `docs/vism-kit-model.md:295`（所有者行）を落とした。第一引用 `docs/vism-package-concept.md:213`（再導入後に保持したpayloadから復元する）が根拠として残る。値は変えていない。

この訂正で `UNDETERMINED` は30から36へ増えた。**未決が増えたのは後退ではなく、循環根拠と無支持の断定を取り除いた結果である。**
