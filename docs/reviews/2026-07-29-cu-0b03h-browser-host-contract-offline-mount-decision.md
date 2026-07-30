# CU-0B03H Browser Host契約・offline mount決定

- 日付: 2026-07-29
- 状態: **実装完了 / DONE**
- 次の一粒: `CU-0B03` **PRODUCT-ASSET / DO**

## 1. 閉じた境界

H1bをVS-1のbuilt-in Browser一面へ限定した。`ui/motolii-web`が
`host.html`、product `DiscoveryBrowserCandidate` mount、closed codec、
決定的Vite bundle、SHA-256付きasset manifestを所有する。

Host→Web snapshotはexact 6 fields:

```text
version=1, direction=host-to-web, role=browser,
instance_epoch=u64 decimal string, sequence=u64 decimal string,
browser.rectangle_source={scope_ref,item_id}
```

Web→Host messageはexact 8 fields:

```text
version=1, direction=web-to-host, role=browser,
instance_epoch, sequence, kind=browser.place,
source={scope_ref,item_id}
```

IDは既存Browser decoderと同じUTF-8 128 bytes上限、messageは固定shapeから導く
1024 bytes上限とする。unknown/missing field、wrong version/direction/role、
非canonical u64、sequence枯渇、空/oversize IDをfail closedで拒否する。

## 2. runtime境界

Web側はHost注入のprivate `window.__MOTOLII_BUILTIN_HOST__`だけを読み、
`snapshot`と`postMessage`の2 field以外を拒否する。bridge不在時はmountせず失敗する。
fixture、localhost、HMR、raw Document、selection、Undo、OS/GPU handleを渡さない。

offline HTMLはCSPでnetwork、object、frame、form、baseを拒否し、相対assetだけを読む。
生成物closureは`.vite/manifest.json`、HTML、CSS、JSと`asset-manifest.json`で固定する。

## 3. 非目標

- wry child WebView生成、native window、Stage/Timeline viewport
- reload/crash/focus/resize後の再投影
- Place preview/terminal/admission/D2/Undo
- community panel、公開plugin UI、generic invoke
- token、visual、DOM/class/stable ID/ARIA変更

## 4. 証拠

- Browser Host codec + ownership: 10 pass
- `npm run build:host`: 68 modules、network入力0のoffline artifact生成
- `npm run check:host`: asset closure/hash一致
- package lock audit: vulnerability 0

次は`CU-0B03`で同じexact codecをnative Host側へ実装し、bounded event-loop inboxへ
decode/enqueueする。契約を再設計せず、WebView callback内でDocument/D2を直接呼ばない。
