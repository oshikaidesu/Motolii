# M5 Render Contribution alpha意味decision

作成日: 2026-07-29

状態: **決定／P2D-RCT1 DONE**

## 1. Authorityと範囲

本decisionは[M5仕様](../specs/M5-3d-and-post.md)、
[Render Contribution統合decision](2026-07-29-m5-render-contribution-integration-decision.md)、
[typed seam decision](2026-07-29-m5-render-contribution-typed-seam-decision.md)を正本とし、
F2／F3のalpha意味とfailureだけを閉じる。provider先例は方式選択の根拠にしない。

公開Rust item、Document／schema、OIT方式、render phase、sort、resource、budgetは変更しない。

## 2. 明示alpha class

alpha classは型付き要求が明示する意味であり、Hostが現在の画素、provider／material ID、
package provenance、first-partyかどうか、名前、magic thresholdから推測しない。

| class | 保証意味 | depth参加 |
|---|---|---|
| opaque | coverage全域が不透明 | 通常のdepth test／writeへ参加できる |
| cutout | coverageが明示的に二値 | covered sampleだけが通常のdepth test／writeへ参加し、uncovered sampleはcolor／depthの双方へ寄与しない |
| soft alpha | fractional coverage／transmittanceを持ち、可視結果が順序依存 | opaque／cutout depth writerへ黙示格上げしない |

二値coverageをthreshold、analytic mask、alpha-to-coverage等のどの方式で生成するかは未決とする。
実際の出力が宣言保証を満たせないcontributionはsoft alphaを要求するか、真実な複数contributionへ
分けなければならない。Hostが画素検査でclassを上げ下げしない。

## 3. policyとの関係

`Layer Order`は既存のauthoring order visibilityを維持する。`Group Depth`／`AE-style Bins`では
opaqueとcutoutを共有depthへ参加させられる。soft alphaは、別decisionで採択された
transparent-intersection保証を選択policyと実行環境が提供する場合だけadmitできる。

`P2D-RCO1`完了前は、soft alphaを共有depthで解けない組合せに対する型付き拒否が正しい挙動である。
`Layer Order`、opaque、cutout、別policy、部分admissionへ黙ってfallbackしない。

## 4. typed failure意味

failureは少なくとも次の意味を区別する。具体Rust名とpayload形は実装decisionへ残す。

1. 選択policy／能力では要求alpha保証を提供できない。
2. alpha保証が欠落、不正、または相互矛盾している。
3. conformanceで黙示promotion／fallbackが検出された。

拒否は要求全体に対してatomicであり、要求したalpha classとpolicy文脈、fallback未適用を観測できる。
拒否によってDocument、既存2D composition、既存画素を変更せず、代替policyを選ばない。

## 5. F2／F3 oracle

### F2 cutout

- 前景cutoutのcovered領域は後景を遮蔽する。
- hole／uncovered領域はcolorもdepthも書かず後景を見せる。
- Z交差でcovered領域の勝者が反転する。
- fractional edgeをsolid occluderへ変えない。

### F3 soft alpha

- 二つのfractional surfaceで`A over B`と`B over A`が異なることを順序依存の意味oracleにする。
- このfixtureは具体OIT方式や正解pixelを選ばない。
- 非対応の共有depth要求は型付き拒否され、成功pixel、別policy、opaque／cutoutへfallbackしない。

## 6. 必須負例

- cutoutのuncovered領域がcolorまたはdepthを書く。
- soft edgeがopaque occluderまたはcutout depth writerになる。
- Hostが画素走査、provider／material ID、provenance、magic thresholdでclassを推測する。
- false binary保証をadmitし、描画後の見た目だけで失敗を発見する。
- 非対応soft alphaを部分admission、`Layer Order`、別class／policyへfallbackする。
- failureを非型付き文字列へ潰す、Document／既存画素を変える、代替policyを選ぶ。
- First Vism専用alpha class、key、registry、経路を作る。

## 7. 非決定

threshold値／animation、alpha-to-coverage、hash／dither、MSAA sample意味、depth format／compare、
transparent sort、phase／queue、OIT／deep／weighted方式、近似誤差、Quality差、GPU feature matrix、
budget、公開API、Document／serde／wireは決めない。

alpha classをDocumentへ保存する決定も行わない。現時点では型付きrender要求の意味である。

## 8. STOP

- F2／F3を閉じるためにOIT、sort、phase、Quality、budget、公開API、schemaを同時決定する必要がある。
- cutoutに普遍thresholdまたは画素走査が必要だと判断した。
- soft alpha対応のためwhole-request admissionまたはfallback禁止を弱める必要がある。
- placeholder pixel／製品UI、premultiplied RGBA、`FrameDesc`、P3 Observationを発明する必要がある。

## 9. 後続

`P2D-RCF1`は本decisionのF2／F3を共通harnessへ取り込める。
`P2D-RCO1`はfractional alphaをopaque depthへ格上げしない不変条件を入力に、
方式、保証、Quality、budget、unsupported matrixを別に選ぶ。
`P2D-RCD2`はalpha class、fallback、OIT modeを永続化しない。
