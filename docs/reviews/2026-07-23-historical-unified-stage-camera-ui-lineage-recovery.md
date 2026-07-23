# 統一Stage／Camera UI lineageの価値回収（Unit 4Q、2026-07-23）

状態: **UI境界は決定済み・未実装**（cutoff 2 historical blobの処分完了、旧M2 schema案は置換済み）

対象: [Stage／Output Frame／統一Camera設計](2026-07-14-unified-stage-camera-design.md)のcutoff全2版。

関連: [M2 camera全版回収](2026-07-23-historical-m2-camera-contract-lineage-recovery.md)、[M3仕様](../specs/M3-ui-integration.md)、[UI runtime architecture](../ui-runtime-architecture.md)、[小さなcore](../extensible-core-model.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

初版が一つの文書で提案したcamera schema/runtimeとStage UIのうち、前者は後のplanar v1決定へ置換され、後者は今も有効である。二版目はこの置換を明示しただけで、Stage View、Output Frame、off-frame編集の意味は変えていない。

現行M2は常在`PlanarOrthographic` camera、Document評価、preview/export接続まで実装済みである。一方、Stage実UIのU1fとcamera/object直接操作のU2dは未実装である。したがって本lineageから旧`position/target/Orthographic|Perspective` schemaを戻さず、UIの所有分離と負例だけを回収する。

この境界は二つの独立した軸を混ぜない例でもある。

| 軸 | Stageでの判定 |
|---|---|
| presentation runtime | 高頻度Preview／gizmo／hit-testはnative wgpu、低頻度header／transport／formはReact候補 |
| architectural role | Stage全体は作品を投影・編集するbundled first-party Host module。semantic Coreでもuser pluginでもない |
| provenance / trust | bundled first-partyであることはReact/nativeの選択と無関係。third-partyへ同じDocument権限を自動公開しない |

nativeで描くからCore、Reactで描くからplugin、first-partyだからnative、third-partyだからReact、という推論はいずれも誤りである。

## 2. 維持する三つの所有境界

| 操作 | 正本 | Final／serialize | Undo |
|---|---|---|---|
| Camera／Output Frame移動・zoom・roll | Document `CompCamera` | 変わる | D2、1 gesture=1 history |
| Stage View pan／zoom／Fit Output・Selection・All | Workspace／Project session候補 | 変わらない | Document履歴なし |
| Object配置・transform | Document world transform | 変わる | D2、1 gesture=1 history |

Camera toolとHand／Stage View toolはicon、枠形状、操作結果で区別する。pan/fitからDocument cameraを変更せず、Camera操作からworkspace viewだけを動かして成功に見せない。React/nativeのどちらがeventを受けても、最終intentとstate ownerは同じである。

## 3. Output Frame外も同じworld

- frame外は別preview camera、別Document、別時刻ではなく、同じcamera/world評価から派生する。
- frame内外でbounds、anchor、selection、hit-test、snapを維持する。不透明塗潰しや色だけで出力外を表さない。
- 近傍はDraft overscan、遠方はbounds等へ縮退できるが、対象を無言で消さず品質低下を診断可能にする。
- GPU同期readbackでvisible boundsを求めず、宣言boundsまたは非同期derived cacheを使う。
- Draft overscanは編集表示であり、Final apertureや書き出し範囲を広げない。
- K0 RoD/RoIは後続最適化で、U1fの成立依存ではない。K0前は保守的Draftで意味を先に閉じる。

## 4. 現行正本との照合

| 面 | 現在地 |
|---|---|
| camera schema/runtime | M2 D1j/D1k/D3fでplanar v1を実装済み。歴史初版のSpatial/Perspective形は不採用 |
| Stage View／Output Frame | M3仕様へ転記済み、U1fはBLOCKED・未実装 |
| direct Camera／off-frame object edit | U2dへ割当済み、U1f/U2c依存で未実装 |
| presentation | native Stage ownershipを採択。Reactは低頻度chrome候補で、Document正本を持たない |
| plugin境界 | Stageはbundled Host module。plugin UI公開契約G0-3とは別 |
| Spatial/Perspective | M5 P3で追加variantを再決定。旧schema案から直接実装しない |

M2 camera全5版はUnit 4Iで処分済みである。本単位はschema/runtimeを重複回収せず、そこから分離後も残ったUI意味を担当する。

## 5. 再入場条件

U1fは同じcamera/worldからframe内外を描くfixture、Stage View操作でDocument bytesとFinal pixelが不変な負例、off-frame選択、同期readbackなし、overscan計測を閉じる。U2dはcamera、workspace view、object transformの三intentを別ownerへ送り、DPI差でも同じ正規化gestureが同じdomain値になることを審判する。

React headerとnative Previewを組み合わせる場合も、Stageという一つのHost module、同じselection／snapshot／Undo ownerを維持する。DOM state、native widget state、catalog labelからDocument意味を推測しない。plugin UIやcommunity kitへ一般化したくなったらU1f/U2dへ混ぜずG0-3でSTOPする。

## 6. 復活させないもの

- 初版の`position/target/Orthographic|Perspective` schemaを現行M2へ戻すこと。
- `motolii-core`の古いcamera型やUI都合からDocument schemaを逆算すること。
- 2D専用worldと3D専用world、追加camera、group camera、camera layerをv1へ作ること。
- Stage View transform、DPI、logical/physical px、toolkit stateをDocumentへ保存すること。
- frame外用の第二camera／第二Document／第二selection ownerを作ること。
- Draft overscanをFinal品質・Final範囲へ昇格すること。
- native／ReactとCore／Host／plugin、first／third-partyを同じ分類軸として扱うこと。
- U1f/U2d未実装をcamera schema未実装と混同してM2を巻き戻すこと。

## 7. 固定歴史出典とcoverage

初版`9222eb26`を全文で読み、二版目`a462ee4e`のschema/runtime置換注記との差分を確認した。処分した2 unique blob（12,110 bytes）の完全SHAは`evidence/historical-value-recovery/disposition-receipts/04q-unified-stage-camera-ui.tsv`を正本とする。cutoff総数1,797のうち処分済みは337、未処分は1,460である。
