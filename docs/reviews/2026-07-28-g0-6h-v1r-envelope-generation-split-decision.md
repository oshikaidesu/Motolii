# G0-6H-V1R envelope / generation分割の裁定

- 日付: 2026-07-28
- 状態: **決定**
- G0-6H-V1R: **DONE**

## 背景

`G0-6H-V1`のOpus 5 order draftは、screen 1の表示境界と新しいevidence
generationを一粒へ束ねると閉じられず、三度`ORDER: STOP`となった。Fable 5の
read-only助言と現行コードを再照合し、未決だった表示4点と契約境界を次のとおり処分する。

## R-1 Inspector

- screen 1ではlegacy fixture scriptを実行しない。
- この経路の既存Inspectorは子を持たない`#inspector` containerであり、これを空投影とする。
- `.panel-head`やplaceholderを合成せず、第二component、第二state owner、追加copyを作らない。

## R-2 Stage

- `.scene-copy`、`.rings`、`.selection-bounds`、`.motion-path`を作品内容として除外する。
- `.stage-hud`と`.stage-badge`もactive gesture / plugin production statusの投影なので、empty Projectでは除外する。
- `#stage`、output frame、panel chrome、transport、playheadは維持する。

## R-3 Timeline

- `TimelineCandidate`へdevelopment専用の空projectionを描画前に渡し、object / bar / key / selectionを0件にする。
- ruler、lane chrome、transport、`KEYS / LAYERS` panelは維持する。

## R-4 Browser

- 同じproduct-owned `DiscoveryBrowserCandidate`と既存`AssetTile`を使う。
- source railへ新しい`Starter Media`文言を足さない。genericな既存`All Media`だけを残し、Project / Recent / Registered folders / Collections / Tags / Packsのfixture contentは0件にする。
- 4 tileはcapsuleの`path` basenameをname、literal `mediaType`をmetaとし、既存preview classだけへ全域写像する。
- Project origin、production Registered folder、catalog status、Document意味を付けない。

## R-5 ready oracle

- screen 1だけにdevelopment専用のrender-time ready属性を付ける。
- `data-parity-ready`は付けず、legacy scriptの成功意味を変更しない。
- ready属性、visibleな`#project-browser`、activeなMedia tab、4 asset tile、Stage / Inspector / Timelineの作品内容0件を同時に審判する。

## R-6 契約境界の分割

親`G0-6H-V1`を次の二粒へ分割する。

1. `G0-6H-V1E`: typed envelope、screen 1空投影、ready oracle、通常route不変を閉じるpresentation粒。
2. `G0-6H-V1G`: V1E完了後、5 normal / 30 PNG、別rootのimmutable generation、manifest、read-only verifierを閉じるevidence粒。

旧`#reference/*` generationのschema / root / commandは変更しない。`V1G`の新root、
manifest field、command、hash closureはV0 V-7がV1へ委ねたmechanicsとして、V1Gの
closed order内で一意に固定する。

## R-7 Browser固定source lineage

- 固定commit `56c318edcddab7cf95d263cc2f7dd2b4e6791134`のBrowser component
  blobは、所有移管時のsource lineageとしてhashを保持する。
- `G0-6H-V1E`は移管後のproduct-owned component改善であり、通常route不変を条件に
  development専用projection seamを同じcomponentへ追加してよい。
- `source-provenance.json`へpost-promotion changeのtask、対象file、理由を追加し、
  Browser ownership guardは旧blob hash、変更後componentのcurrent hash、通常route
  不変oracleを同時に審判する。
- Browser CSSとpatternの固定byte一致は維持する。
- これはvisual threshold / goldenの更新ではなく、旧sourceを捨てずに移管後変更を
  追跡するprovenance再締結である。test削除、期待値緩和、source copyは許可しない。

## R-8 development carrier

- carrierはVite標準modeの`current-route-capture`を採る。
- `docs/mocks-ui`のmock consumerだけが`import.meta.env.MODE`を読み、mode一致時だけ
  typed envelopeを描画前に渡す。
- Vite config、route、hash key、query、window global、新しいserved entryは追加しない。
- 通常の`vite` / `vite build`ではmode不一致となり、既存route出力を変更しない。

## 非目標

- 本粒でReact / CSS / script / test / fixture / image / manifestを変更すること。
- `G0-6H`完了、human session実施、token採択、`U0e-3`解禁。
- product runtime、Document、公開API、plugin契約、永続形式への意味追加。

## 次の一粒

**`G0-6H-V1E`**だけを`DO`とする。`G0-6H-V1G`は`V1E`完了まで`WAIT`とする。
