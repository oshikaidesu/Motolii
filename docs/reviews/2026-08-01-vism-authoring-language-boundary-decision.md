# Vism作者programの言語境界決定

状態: **決定**。一般creator-authorがprogramを書く段の公式作者言語をTypeScriptとする。
これはlive runtime、inline式、package、payload、loader、公開ABIの実装許可ではない。

関連:
[Vism意味SDK](2026-08-01-vism-semantic-sdk-cavalry-translation-decision.md)、
[Vism Inspector・作者source・Automation責任境界](2026-08-01-vism-inspector-source-automation-boundary-decision.md)、
[作者連続性と変更カプセル](2026-07-31-authoring-continuity-capsule-goal-contract.md)、
[Vism作者journey](2026-07-27-vism-authoring-journey-decision.md)、
[Vism実装計画](2026-07-17-vism-implementation-plan.md)、
[Vism concept](../vism-package-concept.md)、
[開発体験](../dev-experience.md)

## 1. 決定

Motoliiで一般creator-authorが**programを書く段**の公式作者言語は、TS風の独自言語ではなく
TypeScriptとする。初心者の第一入口をcodeへ変える決定ではない。通常利用者は引き続きHost operation、
typed recipe、Parameter Panelから始め、必要になった時だけ同じ表現identity、Preview、診断を保ったまま
TypeScript sourceへ進む。

「TypeScript対応」はNode、npm、DOM、browser、filesystem、network、process、特定JS engineとの互換を
意味しない。作者sourceの正本は、次の四点を一組にしたversion付きcompatibility profileとする。

1. 固定したECMAScript editionのstrict実行意味からHostが閉じた決定論subset。
2. 固定したTypeScript compiler versionの構文と型検査。
3. Host所有のversion付きSDK型宣言。
4. Host所有のpositive／negative conformance corpus。

最初のprofile候補`MTS-1`は**ECMAScript 2024 + TypeScript 7.0.2**を基線にする。ただし
`LANG-TS-F0`がallowlist、診断、同値oracleを閉じ、反対側reviewを通るまで製品対応を名乗らない。
F0では固定compilerを隔離したauthoring toolchain入力として使ってよいが、製品runtime、Document、Vism payload、
Cargo依存へ昇格させない。version更新は黙示追随でなく、profile追加、conformance、migration、
last-goodを伴う明示変更とする。React UIやmockが使用するTypeScript versionを作者profileの正本にしない。
TypeScript 7.0は安定したprogrammatic APIをまだ提供しないため、F0は固定CLIと出力診断だけを使い、
内部compiler API、AST shape、language-service pluginを公開SDKやHost契約へ焼かない。

## 2. TypeScript、WGSL、Rustは競合する三択ではない

| 層 | 責任 | 通常作者への見え方 | 決めないこと |
|---|---|---|---|
| Host operation／typed recipe | codeなしで最初の表現を作り、型付き意味を保持する | 一つの作者面とPreview | recipe schema、万能DSL |
| TypeScript | Path、Text、Instance、Data、parameter等を扱う作者program | programを書く段の公式source | engine、inline式、package形式 |
| WGSL | pixel、texture、mask、fieldのGPU kernel | 必要時だけ同じVism内で段階開示する高度面 | 埋込構文、物理file数、runtime binding形式 |
| Rust | Host、admitted capability module、first-party参照実装 | 通常Vism作者の入口にしない | third-party配布ABI、editor process内native dylib |

WGSLはTypeScriptの代替となる万能作者言語でも、初心者へ必須の第二言語でもない。既定のGPU処理は
Host operation／providerで成立させ、高度作者だけが同じVismのGPU kernel席を開く。内部artifactが複数でも、
Hostがclosure、binding、compile診断、last-good Preview、採用を一つの変更カプセルへ閉じる。

Rustは製品内部で引き続き重要だが、作品意味、Vism identity、一般作者source、配布ABIへ焼かない。
first-partyがRustで実装されることを、第三者へRustを要求する根拠にしない。

## 3. `MTS-1`で閉じる最低互換面

`MTS-1`は`strict`型検査とclosed allowlistを使い、ambient authorityを0にする。標準library全体を
暗黙に開かない。少なくとも次はsourceから直接利用できない。

- `eval`、`Function` constructor、dynamic `import()`、任意module resolution。
- `Date`、timer、wall clock、`Math.random`、未宣言random。
- `Intl`、ambient locale、timezone、environment。
- `WeakRef`、`FinalizationRegistry`、`Atomics`、`SharedArrayBuffer`。
- DOM、Node API、npm package、filesystem、network、process、clipboard。
- `declare global`、副作用登録、Host scene全体、名前検索、identity採番。

時刻、seed付き乱数、math、asset、analysis、GPU kernel等が必要なら、Host SDKのtyped inputまたは
宣言capabilityとして受け取る。entryはtyped入力と解決済みcapabilityから候補結果を返す純関数とし、
Hostのsingle writer、resource admission、deadline、preflight、atomic adoptionを迂回しない。

具体的なglobal allowlist、module syntax、entry signature、数値許容差、SDK module名は`LANG-TS-F0`の
conformance authorityで閉じる。未確定部分を`lib.dom.d.ts`、`@types/node`、UI build設定から流用しない。

## 4. engineとpackageを同時に固定しない理由

TypeScript sourceとHost capability契約は作者が保持し続ける正本である。JS engine、bytecode、WASM、
native artifact、GPU派生物は、その正本にconformする交換可能な実行候補またはcacheである。
engine固有AST、bytecode、build pathをDocument、Project、Vism identityへ保存しない。

engine未決は言語未決への後退ではない。ただし、二つのengineが同じconformance corpusで一致しない場合に
「どちらもTypeScript対応」とは扱わない。hard budget、deadline、停止、隔離、他Vism継続を満たすengineが
成立しなければ、live runtime粒だけをSTOPして言語方針の再入場審判へ戻す。

v1でnpm ecosystemを実行環境として抱え込まない。依存、module、closure、lock、admissionは後続fixtureで
閉じ、未知importを暗黙installしない。TypeScript compilerをauthoring toolchainへ固定することと、
Node/npm runtimeをVismへ公開することを同一視しない。

## 5. 比較案の処分

| 候補 | 処分 | 理由 |
|---|---|---|
| TypeScript | **採用** | semantic reach、一般IDE／LSP／LLM資産、creatorからprogramへの連続性を最も保ちやすい。決定論とauthorityはHost profileで閉じる |
| Lua／Luau | **棄却** | 小さなruntimeとsandbox志向は強い反例だが、Motoliiの公式作者型面を別文化へ固定する利得がTSの学習・型資産を上回らない。将来engine比較の先例としては残す |
| 独自DSL／TS風言語 | **棄却** | tooling、教材、diagnostic、互換責任をMotoliiへ集中させ、Adobe Script型の行き止まりを再生産する |
| Rust | **棄却** | Host実装には適するが、10分fork、短いfeedback、第三者配布、初心者経路の公式入口には重い |
| language-neutral recipeだけ | **縮小採用** | 基層として必要だが、parameter mappingを越える作者programの天井にはしない |
| 複数言語を同格公開 | **棄却** | 初期の診断、教材、conformance、runtime、配布負担を分裂させ、一つの作者面を壊す |

Lua／Luau、Rust、WGSL、将来WASMの存在を否定する決定ではない。一般作者sourceを複数の同格入口にせず、
Host typed capabilityの下で各責任を分ける決定である。

### 5.1 旧`Motolii ShapeScript`の処分

M3-U9bで予約していた`Motolii ShapeScript`を、TypeScriptと別の独自言語としては**棄却**する。
U9bが保持する価値は、正準座標のPath／Shape／Group API、有限one-shot実行、明示seed、typed D2 batch、
1 Group／1 Undoというlanguage-neutralなmaterialize契約である。これを`MTS-1`上のHost SDKとして再配置し、
一般作者はTypeScript sourceから呼ぶ。`ShapeScript`は歴史的task名にだけ残し、製品言語名、独自syntax、
別runtime、別source extensionとして実装しない。U9aのruntime非依存batch境界は維持し、U9bの実行は
`LANG-TS-F0`と`VSM-C2`の成立前に開始しない。

## 6. 最初の証拠を二段に分ける

### LANG-TS-F0 — 意味と診断

engineを製品へ常駐させず、headless dev harnessだけで次を比較する。

1. 同じ小表現をHost typed recipeと`MTS-1` sourceで記述する。
2. TypeScript側を検査し、同じprepared recipeまたは同じ型付き結果へ決定的にmaterializeする。
3. 同じ`RationalTime`と入力で結果が一致する。
4. §7のうちengine不要な`MTS-N1`〜`MTS-N5`、`MTS-N7`、`MTS-N9`と、典型的な作者error
   10件のうち8件以上を、Rust／WGSL語彙なしの行動可能な診断にする。`MTS-N6`／`MTS-N8`はF1／C2へ送る。
5. 既存Vismのforkから最初の表現変更までを10分以内に完了できるか測る。

固定TypeScript compilerはこのfixtureのauthoring toolchain入力であり、Node／npmをVism runtimeへ開く根拠にしない。
これはlive runtime、local loader、Document field、package schema、一般利用者UIを証明しない。

### LANG-TS-F1 — feedback速度と回復

`VSM-C2`相当のengine／隔離spike後に別粒で、p50 edit-to-Preview 2秒以内、再起動0、last-good維持、
deadline、hard budget、runtime failure後の他Vism継続を測る。F0合格をF1や製品対応の代わりにしない。

## 7. 必須負例

| ID | 入力／故障 | 必須結果 |
|---|---|---|
| MTS-N1 | Node／npm／DOM／filesystem／network／process import | 未宣言moduleとしてtyped reject。installしない |
| MTS-N2 | `Date`、timer、`Math.random`、`Intl`の直接利用 | typed reject。宣言capabilityへ戻す |
| MTS-N3 | `eval`、`Function`、dynamic import、global拡張 | typed reject |
| MTS-N4 | profileより新しいTS構文／lib | 黙示fallbackせずprofile versionを診断 |
| MTS-N5 | capability provider欠落 | preflight failure、adoption 0、last-good維持 |
| MTS-N6 | 無上限loop／再帰、deadline／budget超過 | typed runtime failure。成功や近似へ丸めない |
| MTS-N7 | sourceがfixture、expected、resource上限を変更 | reject。oracleはHost所有 |
| MTS-N8 | engine間でconformance結果が不一致 | 当該engineをblockし、片方を黙って正解にしない |
| MTS-N9 | 最初の変更にTSとWGSLの両方、複数file、seat配線を要求 | 初心者経路としてreject。Host operationか一つの作者面へ戻す |

## 8. STOPと未決

次は本決定から実装しない。

- live JS／TS runtime、QuickJS等のengine採択。
- inline式、Formula、`ParamSource`へのTS適用。これはPP-Gateを先取りしない。
- SDK公開API、entry signature、module resolver、npm互換、dependency schema。
- `.vism` container、manifest、payload、loader、install store、signature。
- WGSL埋込構文、include、binding schema、物理file数。
- Document field、serde面、公開ABI、editor process内native dylib。
- Motolii内の汎用code editor／IDE。

TS／JS固有AST、engine、compiler、bytecode、build pathを作品意味または永続形式へ追加する必要が生じた粒、
first-partyだけがprivate Host APIへ到達する粒、通常UIへ言語／runtime選択を出す粒はSTOPする。
実装順は`VSM-B4W → VSM-B4 → VSM-C2 → VSM-D3/D4`を維持し、必要なJS engine比較はC2の
authority、budget、failure matrixへ追加して独立に締結する。

## 9. Fable 5反対側レビューとCodex採否

2026-08-01、Claude Code経由のFable 5へ現行正本7件、コード事実、TypeScript／Lua／独自DSL／Rust／
language-neutral／複数言語の比較案をread-onlyで渡した。Web調査付き実行は無出力で停止したため、
同じmodelへReadだけで再依頼し、外部検索の主張を採否根拠にしなかった。

初回判定は`REVISE`。P0は、TypeScript／engine交換のnormative profile不在と、ECMAScript自身の
非決定・ambient機能を除外していない二件だった。両方を§1と§3へ採用した。P1は、初心者の第一入口と
program言語の混同、PP-Gate先取り、意味fixtureとlive性能gateの束ねすぎの三件で、すべて§1、§6、§8へ
採用した。Lua／Luauのsandbox志向を反例として残し、人気だけを採択理由にしない。

実diffの最終再審査は`ACCEPT（P0=0、P1=0）`。任意P2は、implementation ledgerの日付、F0対象負例、
旧ShapeScriptの処分の三件だった。前二件を台帳と§6へ反映し、ShapeScriptは§5.1の通り独自言語として
棄却してMTS-1 Host SDKへ再配置した。最終判定はFableの権威化ではなく、Codexが現行正本、コード不在事実、
一次資料、docs検査へ再照合した結果として採用する。

TypeScriptの`strict`／`target`／`lib`が別設定であることは
[TypeScript compiler options](https://www.typescriptlang.org/docs/handbook/compiler-options.html)と
[Type declarations](https://www.typescriptlang.org/docs/handbook/2/type-declarations)で再確認した。
[TypeScript 7.0公式発表](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/)は7.0.2の
固定toolchain候補とLSP資産を確認できる一方、7.0にprogrammatic APIがなく7.1以後の予定であることも明記する。
したがってF0はCLIに閉じ、これをengine、埋込compiler API、長期互換の証拠にしない。
`Math.random`のalgorithmがimplementation-definedであることは
[ECMAScript 2024](https://tc39.es/ecma262/2024/multipage/numbers-and-dates.html#sec-math.random)で再確認した。
WGSLはmodule、shader lifecycle、diagnosticを持つ独立shader言語であることを
[W3C WGSL](https://www.w3.org/TR/WGSL/)で再確認した。これらはMotoliiでTypeScriptを採択すべきこと、
engineの安全性、package設計を証明しない。
