# M5 soft alpha OIT disposition

作成日: 2026-07-29

状態: **延期／P2D-RCO1 v1 decision DONE**

## 1. Authority

[pitfalls and roadmapのやらないことリスト](../pitfalls-and-roadmap.md)はsoft alphaの完全OIT／
deep compositingをv1スコープ外とし、opaque／cutoutのGroup DepthだけをP2Dのv1対象にしている。
[alpha意味decision](2026-07-29-m5-render-contribution-alpha-semantics-decision.md)は、
shared depthで保証できないsoft alphaをwhole-request typed refusalにできる。

したがって`P2D-RCO1`はv1方式を採択せず、非対応意味と将来再入場gateを閉じる。

## 2. v1決定

- `Group Depth`／`AE-style Bins`のshared-depth参加はopaque／cutoutまで。
- soft alphaのshared-depth交差はv1でadmitしない。
- unsupported feature、capacity、budgetを理由に`Layer Order`、cutout、opaque、別policyへfallbackしない。
- 拒否要求からpartial／replacement contributionまたはcanonical outputを生成しない。
- DocumentへOIT mode、sort priority、phase、fallback、Quality別方式を保存しない。
- First Vism／first-partyに専用soft-alpha経路を与えない。

これはsoft alpha一般を禁止する決定ではない。既存`Layer Order`の通常premultiplied compositionと、
将来の追加transparent-intersection保証を区別する。

## 3. RCF1へのoracle

v1のF3実行fixtureは次を合格とする。

- fractional alpha要求をopaque／cutout depth writerへ格上げしない。
- shared-depth soft alpha要求が構造化されたtyped unsupportedになる。
- admission結果とdiagnosticが同一入力で反復一致する。
- Document、他要求の既存画素、policy identityを変更しない。
- success pixel、別policy、replacement contributionを返さない。

対応soft-alpha pixel goldenはv1 completionに含めない。

## 4. 将来比較gate

v1へ再採択するには、先にscopeを改訂し、方式ごとの一次資料capsuleとprivate GPU spikeを作る。
engineの実装数やRCS1 private shaderを方式根拠にしない。

最小fixtureはopaque blockerと、premultiplied alpha 0.5の赤／青二面を使い、画面左右でanalytic depthを
交差させる。submission orderをA→B／B→Aで反転し、OITを名乗る候補は同じ結果を出す。
opaque blocker、group外baseline、DRAFT／FINAL共通評価、反復決定性も観測する。

比較時は少なくとも次を分ける。

| candidate class | 役割／限界 |
|---|---|
| typed unsupported | v1 baseline。featureなしだが意味を偽らない |
| global back-to-front sort | 単一acyclic orderだけのnegative control。交差面の一般解ではない |
| weighted／accumulation OIT | approximate候補。色／opacity誤差とprecisionを実測する |
| bounded per-pixel fragment list | represented fragment内のreference候補。capacity／overflow／hard capを実測する |
| deep samples | 現行contribution／resource責任を越えるfuture research。v1候補にしない |

alpha scissor、hash、depth prepassでcoverage classを変える方式はsoft-alpha OIT候補にしない。

## 5. 必須計測

- 対象GPU featureとbackend support。黙示backend fallbackなし。
- 1080pのDRAFT／FINAL、overdraw 2／8／40でGPU time、working bytes、pass／draw／dispatch数。
- weighted候補の色／opacity／precision誤差。
- list候補のcapacity、overflow、data-dependent work、admission前上限。
- submission permutationと反復決定性。
- opaque／cutout geometryとのdepth test。

exactとapproximateは別の型付き保証とする。DRAFTで黙って保証を弱めず、弱いtierが必要なら追加能力として
admitする。overflow、feature不足、budget拒否はpartial pixelを返さない。

具体method採択にはM4-K1／`P2D-RCBUD1`のHost hard-budget ownerが必要である。
numeric tolerance、Auto budget、製品SLOをprivate spikeだけから固定しない。

## 6. STOP

- v1 scope改訂前にOIT／deep方式を採択する。
- provider capsuleやengine多数決をMotolii方式の証拠にする。
- 交差fixture、submission permutation、overdraw／budget計測前に方式を選ぶ。
- golden toleranceを緩め、approximateをexactと称する。
- phase／queue／sort key、公開API、schema、P3 Observationを同時決定する。
- soft alphaをcutout／opaqueへ再分類する。
- scene-color／refraction／copy lifetimeを同じdecisionへ取り込む。
