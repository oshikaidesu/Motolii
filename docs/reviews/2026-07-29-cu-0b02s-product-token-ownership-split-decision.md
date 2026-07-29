# CU-0B02S 製品token所有と接続粒の分割決定

- 日付: 2026-07-29
- 状態: **決定 / CU-0B02S DONE**
- 親: `CU-0B02` / `U0e-3` **SPLIT**
- 次の一粒: `CU-0B02T` **PRODUCT-ASSET / DO**

## 1. 決定

G0-6Hで受理したDark外観を、Reactとnativeが別々に写経せず使える単一の
製品token正本へ移す。親`CU-0B02`を次へ分割する。

| ID | 状態 | 一成果 |
|---|---|---|
| `CU-0B02S` | `SPEC / DONE` | 現行supplier事実、単一owner、生成境界、後続粒を決定 |
| `CU-0B02T` | `PRODUCT / DO` | product-owned Dark DTCG sourceとgenerator v2の決定的Rust/CSS/manifest生成物を導入 |
| `CU-0B02R` | `PRODUCT / WAIT` | accepted React routeをgenerated CSS consumerへ反転し、legacy style scrapeをsupplierから退役 |
| `CU-0B02N` | `PRODUCT / WAIT` | 同じ生成bundleのRust adapterをnative shell styleへ接続 |
| `CU-0B02C` | `PRODUCT / WAIT` | 既存React component state sourceをR0〜R4と同じ直接所有移管で製品化 |
| `CU-0B02I` | `SPEC / WAIT` | 統一icon grid/stroke systemの採択判断。`CU-0B02T`完了まで着手せず、実装・置換はまだ行わない |

順序は`CU-0B02T → CU-0B02R → CU-0B02C`と
`CU-0B02T → CU-0B02N`の二枝とする。
`CU-0B02I`は採択正本が無いため`CU-0B02T`後の独立decisionであり、Unicode glyphを
製品icon体系と読み替えない。React/native Host transport、Document projection、typed intent、
WebView統合、W0bはこの列へ含めない。

## 2. 現行コード事実

accepted route `#plugin-browser-candidate`は
`docs/mocks-ui/src/legacy/legacySource.js`の`?raw` importと正規表現により、
変更禁止の`docs/mocks/m3-vism-host-boundary.html`内`<style>`から
`--bg`、`--panel`、`--ink`等を実行時抽出している。したがってaccepted routeの
実supplierはproduct-owned CSSではない。

一方、`docs/mocks-ui/src/tokens/mock-candidates.css`にも同系統のDark tokenがあり、
standalone diagnostic/primitives側へ供給している。既に二つのsupplierがあり、
両者は完全閉集合ではない。特にlegacy側の`--object-1`〜`--object-6`は
Timeline object slot色で、任意/object色のDocument意味が未決である。

`crates/motolii-ui-token-gen`のv1はDTCG 2025.10の
color/dimension/duration/cubicBezierを検証し、fixture用`tokens.rs`とmanifestだけを
生成する。製品source、CSS生成、React/native consumerには未接続である。
`crates/motolii-ui`は既定egui visualsのままで、製品token adapterを持たない。

統一icon grid/stroke systemは存在しない。現行product sourceには
`EasingTriggerCandidate.jsx`、`KeyToolsCandidate.jsx`、
`InspectorCandidate.jsx`のinline SVGと、`◆`、`▶`、`⌘`、`✓`、`■`、`□`等の
Unicode glyphが混在する。これらはG0-6Hで外観ごと受理された現行presentation事実だが、
統一icon systemの採択正本ではない。

## 3. 単一製品owner

`CU-0B02T`は次の閉じたbundleを新設する。

- source root: `ui/motolii-tokens/sources/motolii-dark.json`
- generated root: `ui/motolii-tokens/generated/`
- generated files: `tokens.rs`、`tokens.css`、`manifest.json`
- generator: `crates/motolii-ui-token-gen`
- generator version: `2`

DTCG JSONだけを値の正本とし、Rust/CSS/manifestはcommit対象の決定的生成物とする。
v1 fixture bundleはv1契約の回帰fixtureとして残し、product bundleの存在を理由に
遡及変換しない。generatorへpathやtheme IDから推測しない明示的な閉enum
bundle profile `v1-fixture` / `v2-product`を追加し、version、出力集合、header、
manifestをprofileから決める。`v1-fixture`は既存2出力、v1 header、manifest byteを
完全維持する。`v2-product`だけが既存4型の意味・検証・入力hash規則を変えず、
CSS出力と3-file manifestを追加する。CLI/libraryはprofile指定を必須とし、
未知profileとprofile/output不一致を型付き拒否する。CSS variable名はtoken pathのsegmentを
ASCII `-`で連結した`--motolii-<path>`とし、変換後衝突を型付き拒否する。
CSS値は検証済み値から決定的に出力し、手編集を`check`で拒否する。

最初のsourceは人間受理済みDark一つだけとする。Light、custom、
high-contrast、theme preference、fallback、hot reloadを既定値で埋めない。
`--object-1`〜`--object-6`はsource閉集合へ含めない。

## 4. 最初の実装粒 `CU-0B02T`

`CU-0B02T`はtoken authorityだけを閉じる。

- v2 generatorのRust/CSS/manifest三生成物とread-only `check`
- accepted Dark role、spacing、radius、motionのうち既存DTCG 4型で表現でき、
  current supplier間で意味と値が照合できる閉集合
- product bundleのcompile/parse、決定性、手編集拒否、raw literal supplier拒否
- `docs/mocks/**`、accepted route、native style、React component byteは変更しない

raw color/spacing拒否は最初から製品package全体へ掛けない。
product token rootはsource JSON 1件とgenerated 3件の閉集合にし、余分な手書き
CSS/JS supplierを拒否する。generated CSSの値はgeneratorのbyte `check`が所有し、
raw-color scannerの対象にしない。既存
`docs/mocks-ui/scripts/reference-guard.mjs`のPostCSS
`RG-RAW-COLOR`走査と同testの負例を、共有pure scannerへ抽出して既存guardと
新しいproduct-token guardのsynthetic CSS/JS supplier負例から再利用する。
実repo scan rootは`ui/motolii-tokens/`、allowlistは
`sources/motolii-dark.json`とgenerated 3件だけとする。挙動と既存testを変えず、
既存leaf CSSの約40 raw colorをtoken導入前に一括修理しない。

`ui/motolii-web/guard-tests/browser-ownership.test.mjs`のCSS SHA pinは
`CU-0B02T`では不変とする。後続`CU-0B02R`でproduct CSS byteを変更する場合は、
受理済みDOM/class/stable ID/ARIA/interactionとvisual oracle不変を先に証明し、
token supplier反転に必要な対象pinだけを同じ粒で正当更新する。期待hashだけを
先に変える、または無関係driftを吸収する更新は禁止する。

## 5. React直接移管ラベル

1. `REACT AUTHORITY`: 対象はaccepted Browserを含む現行React面。正本は
   [React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)、
   UI runtime境界、`U0e-3`。本粒はpresentation byteを変更しない。
2. `SOURCE ASSET`: 固定SHA
   `56c318edcddab7cf95d263cc2f7dd2b4e6791134`のReact closureと、
   現行product-owned JSX/CSS closure。旧token supplier二系統は§2の事実として読む。
3. `PRESERVE`: DOM、class、stable ID、ARIA、interaction、visual state、G0-6Hで
   受理されたDark外観。
4. `REPLACE`: `CU-0B02T`ではconsumerを交換しない。後続`CU-0B02R`だけが
   legacy/mock token supplyをgenerated CSS importへ反転する。
5. `STATE OWNER`: token値は状態ではなくbuild-time product UI assetであり、
   該当stateなし。theme preferenceはUser settings未決、Document/Project
   session/Transientへ保存しない。component interaction stateは`CU-0B02C`まで
   既存local presentation ownerを維持する。
6. `DIAGNOSTIC ROUTE`: `#catalog`等のdevelopment routeはconsumerであり正本ではない。
   通常製品画面、diagnostic route、native shellは同じgenerated bundleを後続粒で読む。
7. `NEGATIVE ORACLE`: 二重DTCG、legacy/runtime import、opaque ID分岐、二重state、
   raw supplier、`--object-1..6`混入、Light/custom発明、threshold/golden変更を拒否する。
8. `STOP`: accepted Dark値を一意に照合できない、generator v2で既存v1意味を変える、
   公開API/Document/plugin/永続形式が必要、source asset byteやvisual oracle変更が
   必要になった時点で停止する。

## 6. 完了条件と非目標

`CU-0B02T`の必須審判は、generator unit test、v1 fixture `check`、
product bundle v2 `check`、profile省略/未知/profile-output不一致拒否、生成Rust compile、
生成CSS parse、入力順/mtime/別directory決定性、v1の既存byte完全維持、
三生成物の欠落・余分・1 byte drift拒否、CSS名衝突拒否、既存と新product-tokenの
`RG-RAW-COLOR`正負test、supplier raw literal負例、`cargo fmt --all --check`、
`./scripts/check-ui-toolkit-deps.sh`、`./scripts/check-docs.sh`である。

非目標はconsumer import、legacy extractor削除、React/native appearance変更、
component state promotion、icon採択、theme selector、Host transport、WebView、
Document/selection/Undo、golden/threshold更新である。
