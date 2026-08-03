# SDK-S0S — Path2D意味fixture責任仕様

状態: **仕様案／独立review待ち**（言語非依存fixtureのみ。公開SDK、TypeScript runtime、package、Document形式は未成立）

## 1. 利用者成果

Cavalryの強みをObject Modelの模倣ではなく、「選択中の表現へ型付きPathを渡し、意味のあるoperationを適用し、結果と失敗理由をInspectorから読める」連続性としてMotoliiへ落とす。

最初のsliceは`Path2D → Path2D`の純粋offsetだけとする。作者は一つの可視recipeで有限な距離を変更し、通常VismのInspectorから作用先、入力、出力、単位、space、temporal mode、診断を確認できる。TypeScript、WGSL、Rust、engine、payloadを選ばせない。

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| `AUTHORITY` | [PathOp意味論表](../specs/M2-document-model.md#pathop意味論表d1i-2--決定-2026-07-13)、[意味SDK決定](2026-08-01-vism-semantic-sdk-cavalry-translation-decision.md)、[作者連続性](2026-07-31-authoring-continuity-capsule-goal-contract.md)、[Inspector境界](2026-08-01-vism-inspector-source-automation-boundary-decision.md) |
| `INTERNAL TARGET` | `motolii_doc::pathgeom::{Path, Contour, Vertex, Point, ResolvedPathOp, apply}`、D1i-2 offset golden、U4a-1 parameter control model |
| `OWNER` | Path意味はM2 PathOp authority、`docs/reviews/evidence/sdk-s0-path2d/`は言語非依存fixture dataの正本置き場、`motolii-doc` testとLANG-TS-F0はconsumer、test-only adapterはHost test infrastructure、Inspector projection候補はM3のread-only projection。作者programはDocument writer、GPU resource、scene storeを所有しない |
| `WRITE ROUTE` | SDK-S0Iはtest-only fixtureをnative oracleへ入力し、同じcontract projectionをheadlessに検査する。製品Documentやrender routeへ書き込まない |
| `GAP` | native PathOp意味とgoldenは存在するが、作者言語から独立したtyped input／parameter／output／diagnostic contractとInspector投影を同じfixtureで検査する席がない |
| `RESOLUTION ROUTE` | 既存PathOpとgoldenをoracleとして再利用し、test-only contract projectionを追加する。公開Rust型、Document schema、製品diagnostic enumは新設しない |
| `DISPOSITION` | `PASS / SPEC`。本文をSDK-S0S、後続のtest-only実装をSDK-S0I、TypeScript consumerをLANG-TS-F0へ分離する |

## 3. 採用する意味profile

fixture内の識別子は試験データのlabelであり、公開TypeScript名、module path、package ID、永続operation IDではない。

| 面 | SDK-S0 fixture意味 |
|---|---|
| 表示名 | `Offset Path`（fixture label） |
| typed input | named `source`; `Path2D`; canonical local space; immutable |
| parameter | named `distance`; finite scalar; canonical length; default `0`; Inspectorで数値controlへ投影可能 |
| profile定数 | line join=`miter`; miter limit=`4.0`。v1 PathOp authorityの一ケースを選ぶだけで、隠れた利用者parameterを一般化しない |
| typed output | named `result`; `Path2D`; inputと同じcanonical local space; newly owned value |
| temporal mode | pure。時刻、frame、selection、scene lookupへ依存しない |
| capability | Path2D offsetだけ。fixture labelを公開capability namespaceへ昇格しない |
| diagnostics | missing input、type mismatch、space mismatch、non-finite distance、open contour unsupported、budget exceededを区別するfixture-local reason |

raw object／JSON map／scene objectを作者入力にしない。Pathの内部Rust fieldをTypeScriptへそのまま公開せず、fixture dataは意味比較用のtest representationに限定する。

## 4. Native oracleと同値判定

SDK-S0Iのnative oracleは新しいoffset実装を書かず、有限値とinput contractをpreflightした後、次へlowerする。

```text
motolii_doc::pathgeom::apply(
  source,
  ResolvedPathOp::Offset {
    distance,
    line_join: Miter,
    miter_limit: 4.0,
  },
  0.0,
)
```

この記述は`ResolvedPathOp`を公開SDKへ昇格する決定ではない。現在の実装をoracleとして固定する内部接続である。意味変更はD1i-2 goldenを書き換えず、M2 authorityの新variant手続きへ戻す。

同値は最低限、輪郭数、open／closed、vertex順、point、相対in／out tangentを比較する。float toleranceは既存D1i-2 goldenの`1e-6`を再利用し、LANG-TS-F0都合で緩めない。

## 5. 必須case matrix

| ID | 入力 | parameter | 期待 |
|---|---|---|---|
| S0-P1 | corner頂点だけのclosed square | `0` | 実行時native oracleと同値で、Path意味が恒等。独立した数値goldenをfixture側で発明しない |
| S0-P2 | closed square | 正の有限距離 | D1i-2 `offset_miter_square_expands_by_distance_with_sharp_corners`と同じ意味結果 |
| S0-P3 | closed square | 負の有限距離 | D1i-2 `offset_negative_distance_shrinks_inward`と同じ意味結果 |
| S0-P4 | cubic Bezierを含むclosed contour | 正の有限距離 | 既存D1i-2 testと同じくchord-only入力の結果と異なり、実行時native oracleと同値。新しいBezier数値goldenをfixture側で発明しない |
| S0-N1 | open contour | 有限距離 | `open contour unsupported`。恒等、close補完、raster fallbackをしない |
| S0-N2 | closed contour | NaN／±Infinity | `non-finite distance`。native oracleを呼ばない |
| S0-N3 | input欠落または非Path | 任意 | missing／type mismatchを区別し、空Pathへ縮退しない |
| S0-N4 | World／View等の別space Path | 任意 | space mismatch。暗黙変換しない |
| S0-N5 | 同じinputを二consumerへ渡す | 異なる距離 | 一方の評価が他方のinput／resultを変えない |
| S0-N6 | fixture-local vertex／contour上限超過 | 任意 | typed budget failure。部分Pathを成功結果にしない |
| S0-N7 | 同じinput／distanceを異なるfixture文脈時刻で評価 | 同じ有限距離 | 結果が同一。offset profileが時刻をambient入力にしない |

S0-P2／P3の数値を新しく手で発明せず、既存D1i-2 testからfixtureへ一方向に写す。S0-P4は既存testが持つ差分性質と実行時native oracle同値だけを検査し、数値期待値を新しく凍結しない。Bezier数値goldenが必要ならSDK-S0Iを止め、M2 PathOp authorityの新variant／golden手続きへ戻す。保護goldenの期待値変更やfixtureだけのspecial caseは禁止する。

S0-N6の上限は小さい明示定数をfixture dataへ置くtest-only budgetであり、製品resource budget、scheduler、Vism package契約の先取りではない。製品意味が必要になった時点で本caseを止め、該当authorityへ送る。

## 6. Contract projection

headless projectionはsource本文を読まず、最低限次を復元できなければならない。

- Vism／fixture label
- `Path2D(local) → Path2D(local)`
- `distance: canonical length / finite`
- `pure` temporal mode
- Path offset capability要求
- case失敗時のreasonと対象input／parameter

U4a-1の既存parameter control modelは`F64` controlへ写せることの先例として使うが、SDK-S0Iで製品Inspectorへ接続しない。`ParamDef`、`NodeDesc`、`DiagnosticEnvelope`を意味SDKの公開形式として再利用・拡張する判断もしない。LANG-TS-F0はこのprojectionと同じ意味をconsumerとして読む。

## 7. SDK-S0Iの変更許可候補

SDK-S0Sの独立reviewでownerと依存closureが閉じた後、SDK-S0Iを別粒として次のtest-only面へ限定する。

- `docs/reviews/evidence/sdk-s0-path2d/`（言語非依存fixtureと期待値。製品serdeではない）
- `crates/motolii-doc/tests/sdk_s0_path2d_semantics.rs`（既存native oracleとの同値・負例）
- `docs/reviews/2026-07-17-vism-implementation-plan.md`
- `docs/implementation-ledger.md`

`motolii-doc`／`motolii-plugin`／`motolii-ui`の公開API、runtime source、Document／serde、golden期待値、TypeScript toolchainを変更しない。fixtureをLANG-TS-F0から再利用するためにprivate test helperの重複が必要に見えた場合はSDK-S0Iを止め、consumer-neutralなtest infrastructureのownerを別仕様で閉じる。

## 8. 必須負例とSTOP

次を拒否する。

- Cavalry互換namespace、scene-wide lookup、mutable object graphを追加する。
- `Path`／`ResolvedPathOp`の現行Rust形をそのまま公開作者SDKにする。
- fixture JSONをDocument、Vism package、plugin manifestの形式として読む。
- offsetのためにCPU rasterize、GPU readback、raw `wgpu` resourceを導入する。
- open pathを暗黙close、non-finiteを0、wrong spaceをlocalへ補正する。
- Inspector projectionを任意HTML／React、専用editor、ambient globalから作る。
- SDK-S0IでTypeScript compiler、JS engine、npm、Node、module resolver、loaderを追加する。
- Path boolean、Instance、Mesh、Particle、Field、Simulationを「将来必要」と同じfixtureへ足す。

次が必要なら該当粒をSTOPする。

- 公開TypeScript API名、module path、versioning policyの決定。
- Document／serde、公開Rust API、plugin／package／runtime契約の変更。
- M2 PathOp authorityまたは保護goldenの変更。
- U4a／製品Inspector／通常render routeへの接続。
- consumer-neutral fixture ownerを作るための新crate／汎用framework。

## 9. 後続順序

1. SDK-S0Sを独立reviewし、P0/P1=0へ閉じる。VSM-A4Sはowner／artifactを共有しない別laneであり直列依存にしない。
2. SDK-S0Iでlanguage-neutral fixtureとnative oracle同値をtest-onlyに実装する。
3. LANG-TS-F0は同じfixtureのTypeScript source候補だけをconsumerにし、意味結果と診断を比較する。
4. LANG-TS-F1はVSM-C2後にfeedback、last-good、hard budget、隔離を測る。

SDK-S0I合格はTypeScript対応、local Vism、package、runtime、製品Inspector完成を意味しない。

## 10. 独立review

2026-08-01、Claude Code経由のFable 5へA4S spot reviewと分離して本文を検収させた。初回は`REVISE（P0=0、P1=2、P2=3）`。

- P1: S0-P4が存在しないBezier数値goldenを参照していたため、既存testの差分性質＋実行時native oracle同値へ修正した。
- P1: VSM-A4Sを共有artifact／ownerのない直列依存としていたため、SDK-S0S/IをM2 PathOp＋意味SDK決定の独立laneへ戻した。
- P2: pure temporal負例、fixture-local budget、evidence directoryのdata ownerを明記した。

修正後のspot reviewは`ACCEPT（P0=0、P1=0、P2=3）`。P2のS0-P1根拠、台帳確認日、review mirror同期をCodexが採用した。S0-P1はcorner頂点だけのclosed squareに限定して実行時native oracle同値とし、新しい数値goldenを作らない。本変更のmain統合でSDK-S0Sを仕様完了とする。
