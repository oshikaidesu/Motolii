# M5 Render Contribution conformance harness decision

作成日: 2026-07-29

状態: **決定／P2D-RCF1 DONE**

## 1. Authorityと完成範囲

本decisionは[Render Contribution統合decision](2026-07-29-m5-render-contribution-integration-decision.md)の
F1〜F6、[typed seam decision](2026-07-29-m5-render-contribution-typed-seam-decision.md)、
[alpha意味decision](2026-07-29-m5-render-contribution-alpha-semantics-decision.md)を正本とする。

`P2D-RCF1`で閉じるのはprovider中立なblack-box harness契約、F1〜F6の責任分担、
First Vismの無特権条件である。公開Render Contribution実装、実行可能harness、
First Vismの具体表現／製品／package／UIは後続実装粒へ分ける。

## 2. black-box観測契約

各caseは次だけを観測する。

1. 固定case入力と要求能力。
2. whole-request admissionの全成功または構造化された型付き拒否。部分成功はない。
3. admission前後のDocument／snapshot不変性。
4. canonical RGBA結果、または拒否時の既存画素不変。
5. 同一入力反復の決定性。
6. DRAFT／FINALが同じ評価関数を通り、差は`Quality`契約だけであること。
7. 製品route成立後、Preview／Exportの両入口が同じ評価へ合流すること。
8. 診断がprovider identity／provenanceでなく不足能力、未知要求、不正依存で分類されること。

registryの数／配置、provider ID、engine phase、sort key、shader、bind group、depth format、
pass数、package identity、P3 Observation具体型は観測契約へ含めない。

## 3. ownerと配置

| 対象 | owner／配置 |
|---|---|
| case入力、期待値、artifact | test-only fixture |
| admission、ordering、shared resource、diagnostic、failure | Host |
| world／camera／Observation／transform／layer order／Quality／FrameDesc | Hostが与えるread-only入力 |
| contributionの具体評価 | fixture provider |
| Document | read-only。成功／拒否のどちらでもharnessは変更しない |
| tolerance、画像比較、GPU skip規律 | `motolii-testkit`既存helper |
| seat固有caseと製品route adapter | `motolii-render`側integration test |

`motolii-testkit`から`motolii-render`への依存を追加しない。共通画像比較は
`compare_rgba_labeled`、`assert_rgba_close_with_artifacts`、`tol::EXACT`／`GPU_RASTER`、
`gpu_or_skip`を再利用し、thresholdやGPU skipをfixtureごとに複製しない。

`P2D-RCS1`の二面反転、未使用baseline、group外比較、反復一致というfixture patternは再利用できるが、
private setup、shader、resource配置、helper、型を公開harnessへ昇格またはcopyしない。

## 4. F1〜F6責任表

| fixture | 共通harnessが固定するoracle | seat固有／後続 |
|---|---|---|
| F1 opaque | A-near／B-nearでoverlap winner反転、外側不変、反復決定性、DRAFT／FINAL共通評価 | geometry／Observation adapterは公開seam実装後 |
| F2 cutout | coveredが遮蔽、holeはcolor／depth不参加、fractional edgeをsolid化しない | 実行fixtureはRCT1を入力に実装 |
| F3 soft alpha | 順序依存、非対応はtyped refusal、別class／policyへfallbackなし | 対応時のpixel保証はRCO1後 |
| F4 scene color | snapshot／range／order／failureを独立slotとし、別Export経路／隠れcopyを禁止 | RCFP1、RCR1、RCP1後 |
| F5 unknown／不足 | whole-request typed refusal、Document／既存2D pixel不変、部分admission／provenance分岐なし | 型付き要求／failure実装後 |
| F6 未使用 | baseline pixel不変、group外不変、反復決定性、同一評価関数 | 実Preview／Export E2E adapterは製品route成立後 |

F1〜F6はP2D全体のAE-style Bins、Undo、selection、policy切替不変fixtureを置換しない。

## 5. First Vism無特権契約

First Vismは将来選ばれた最初の表現に課すconformance役割だけを意味する。

- Host private crateへ依存せず、他providerと同じ公開seamを使う。
- synthetic second fixture providerと同じcase corpus／runnerを使う。
- harnessにFirst Vism ID、first-party provenance、専用feature、専用order／registry分岐を置かない。
- 追加時に既存provider source、期待値、Document schemaを変更しない。
- 対応能力だけ成功し、未対応能力をfake successにしない。
- 公開seamが無い間はprivate RCS1 adapterで成立を偽装しない。

## 6. 必須負例

- unknown要求を`Layer Order`、opaque、既存2Dへfallbackする。
- capability集合の一部だけ受理して描画する。
- rejection後にDocument、cache正本、既存pixelを変える。
- First Vism ID／provenanceでHostが分岐する。
- private RCS1型、shader、registry、raw callbackをharnessへ昇格する。
- F2／F3を空case、opaque代用、smoke testで対応済みと称する。
- Preview／Exportでfixture注入先または評価関数を分ける。
- tolerance／golden期待値を変更して合格させる。
- GPU欠落を手書きreturnで無音skipする。
- semantic admissionをraw JSON、`Any`、private型走査で判定する。

## 7. 実装分割

| ID | 実装粒 | 解禁条件 |
|---|---|---|
| `P2D-RCF1I-BASE` | F1／F5／F6 executable harness | 公開typed seam実装 |
| `P2D-RCF1I-ALPHA` | F2／F3 executable harness | P2D-RCF1I-BASE、P2D-RCT1。対応soft-alpha pixelはP2D-RCO1後 |
| `P2D-RCF1I-SCENE` | F4 executable harness | P2D-RCFP1、P2D-RCR1、P2D-RCP1 |
| `P2D-RCF1I-VISM` | concrete First Vism fixture | 製品表現選定、共通公開seam、対象能力decision |

## 8. STOP

- harness decisionに公開trait／registry、Document／schema、P3 Observation具体型が必要になる。
- RCS1 private spikeをreference provider／公開fixtureとして転用する必要がある。
- First Vismの具体表現、package、配布、UIを選ばないと進められない。
- 実製品routeが無いのにtest-only adapterでPreview／Export完成を称する。
- `motolii-testkit`の逆依存またはhelper複製が必要になる。
