# G0-6H-V1P capture前提の再選定

- 日付: 2026-07-28
- 状態: **決定**
- G0-6H-V1P: **DO**

## 背景

`G0-6H-V1` の Opus 5 order draft は `ORDER: STOP` となった。`G0-6H-V1S`
は screen 1 を development 専用 typed fixture projection、screen 2〜5 を同一
route の既存 interaction で到達すると決めたが、現行コードには screen 1 の injection
seam が無く、後者の具体操作列と capture 時の font fixture 判定点も発注可能な閉集合に
なっていない。

## 現行コード事実

1. `#plugin-browser-candidate` は `LegacyHostBoundaryScreen` から product-owned
   `DiscoveryBrowserCandidate` を組み立てる。
2. 既存 `__MOTOLII_REFERENCE_FIXTURES__` は `#reference/*` 専用であり、candidate
   route は読まない。
3. screen 2〜5 の既存操作候補は、現行テストから初期状態、Interval Easing Editor、
   `Hand`、`Relative Move` まで一意に追跡できる。
4. candidate route は legacy の `Inter, ui-sans-serif, system-ui, ...` stack を使い、
   `#reference/*` 専用の `MotoliiReferenceInter` assertion を共有していない。

## 選定

`G0-6H-V1` を一時 `WAIT` へ戻し、docs-only `G0-6H-V1P` を唯一の次粒にする。
`G0-6H-V1P` は次の三問だけを裁定する。

1. product componentの二重copy、capture後DOM mutation、公開API、Document、production
   catalog意味を増やさず、screen 1へ development 専用typed projectionを渡す
   mock-owned seamと許可file境界。
2. screen 2〜5の既存操作列と、既存stable ID / ARIA / class / visible stateだけを使う
   capture-ready oracleの閉集合。
3. candidate routeの外観を変更せず、generation manifestへliteralに記録する
   font fixture軸の観測点とfallback時の停止条件。

## 非目標

- React / CSS / script / test / fixture / PNG / manifest / media byteの変更。
- seam、prop、schema、command、selectorの具体実装。
- route、hash key、query、visual threshold、golden、token、human sessionの変更。
- `Starter Media` のProject / production / Document / public / plugin正本化。
- `G0-6H`完了、`U0e-3`解禁、M3製品sliceの順序変更。

## 次の一粒

**`G0-6H-V1`** — [G0-6H-V1P裁定](2026-07-28-g0-6h-v1p-current-route-capture-prerequisite-decision.md)を完了したうえで、同じ`V1`を再び`DO`へ戻す。別の実装粒を新設しない。
