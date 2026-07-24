# Historical-only React / WebView lineageの価値回収（Unit 2B、2026-07-23）

状態: **決定**（歴史文書17 blobの処分、built-in WebView Host不変条件の再採択、製品縦切りの読み替え）

対象: React mock台帳、Browser egui翻訳spike、四面同期vertical slice、built-in WebView入場、旧H1 exact contract、visual oracle route分離のhistorical-only 6 path。

関連: [全歴史coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)、[UI runtime責任境界](../ui-runtime-architecture.md)、[React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)、[surface実装と拡張所有の軸分離](2026-07-22-m3-surface-extension-axis-separation.md)

## 1. 結論

6 path / 17 blobを、初版全文と版間diffで処分した。歴史系列は一つの実装案ではなく、次の訂正を重ねた経路だった。

```text
HTMLをReact bridgeで保存
  → Browserだけを制限付きIRでeguiへ翻訳するspike
  → React chrome / native Stage・Timelineを同じHost snapshotへ接続する縦切り
  → built-in WebViewだけのoffline bundle / typed Host foundation
  → 縮約product leafとoracle routeを分離する旧H1a案
  → 固定React sourceを直接product ownerへ移す現行契約
```

現行へ戻す価値は三つある。

1. **presentation runtimeと拡張分類は直交する。** React面とnative面はいずれもbundled first-party Host moduleになり得る。WebView成立をcommunity plugin公開許可へ、native描画をCore所属へ読み替えない。この決定は現行の[軸分離](2026-07-22-m3-surface-extension-axis-separation.md)へ既に回収済みである。
2. **旧H1aの縮約product leafは復活させない。** 現行契約は固定commit `56c318edcddab7cf95d263cc2f7dd2b4e6791134`のsource assetを直接所有移管し、mockをconsumerへ反転する。Browser JSON→egui変換、legacy bridgeを製品runtimeにする案、別の縮約React leafを作ってpixel差だけ詰める案は失効した。
3. **built-in WebView Hostの安全境界は未回収だった。** 現行treeには`ui/motolii-web`、Web protocol、offline asset manifestが存在せず、直接移管契約は後続を「H1b WebView Host」と一行でしか示していない。旧exact contractのversion・role・数値をそのまま復活させず、offline、closed typed transport、Host epoch、strict origin、fail-closed lifecycleという不変条件をG0-9の将来contractへ再採択する。

四面同期のvertical sliceも価値を保つ。ただし旧「React Easing Panel全体」ではなく、現行どおりReact trigger / current-value summaryからnative Easing popupを開く。旧枝番と「今晩」という優先順位は再発効せず、具体D2・selection・Timeline契約はUnit 2Cで別に処分する。

## 2. 個別処分

| 歴史path / blob | 分類 | 判定 | 現在の回収先 |
|---|---|---|---|
| `docs/mocks-ui/README.md` / `831cc592`,`949a9f64`,`d6204e8a`,`65c8257d` | **成立理由 + 現行規範へ吸収 + 負例** | bridge→Browser候補→React現行入口という移行の証拠。stable ID、state owner未決を推測しない、fixtureと製品意味を分ける規律は維持する。旧HTMLを製品runtimeにすること、React→eguiを既定到達点にすること、prototypeの全機能を採択済みとみなすことは戻さない | [UI参照地図](../ui-reference-map.md)、[直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md) |
| `2026-07-20-m3-browser-panel-egui-taffy-spike.md` / `942b3265` | **成立理由 + 負例 + archiveのみ** | `egui_taffy`でrail/gridを描けた部分合格と、intrinsic text sizingでlabelが縦崩れした負例を保持。閉じたfixture、未知node拒否、1行truncate、toolkit型隔離は現行規律へ吸収済み。DOM/CSS解析やBrowser IR compilerを製品経路へ戻さない | [UI視覚言語](../ui-visual-language.md)、[UI runtime責任境界](../ui-runtime-architecture.md) |
| `2026-07-21-m3-tonight-product-vertical-slice-contract.md` / `68a60229`,`91c06d48`,`aae7f263`,`c4bd39d6` | **現行規範 + 成立理由** | 同じrevision付きHost snapshotからnative Stage / native Timeline / React Inspector・toolsへ投影し、各gestureを1 D2 macro / 1 Undoにする核は維持する。旧U2b-2-core等の優先順、100k証拠の用途、「Tonight」は現行task状態にしない | 本書§4、[製品モック回収計画](2026-07-21-m3-product-mock-recovery-plan.md)、[直接移管契約 §12](2026-07-22-m3-react-product-asset-promotion-contract.md#12-実行順) |
| `2026-07-22-m3-g0-9-builtin-webview-admission.md` / `4e8a551d`,`72c6442d`,`58e50c28` | **現行規範へ再採択 + 停止線** | community panelを含めずbuilt-in Host foundationだけをoffline bundle、typed handshake、opaque child WebView、Host lifecycleへ分割する境界を再採択。旧H1a/H1b/H2/H3番号と当時の`DO/WAIT`は戻さない | 本書§3、[UI runtime責任境界 §4.1](../ui-runtime-architecture.md#41-built-in-webview-hostの再入場条件)、[M3 G0-9](../specs/M3-ui-integration.md) |
| `2026-07-22-m3-g0-9-h1-exact-contract.md` / `9644c5a5`,`49dc2957`,`4d90711c` | **再入場入力 + 負例** | deterministic artifact、closed codec、epoch/sequence、bounded queue、strict custom origin、navigation denyは将来contractの必須論点として回収。縮約4 leaf、固定npm/wry/Chromium version、4 role閉集合、exact byte schema、当時のroute名と上限値は再検証なしに規範化しない | 本書§3 |
| `2026-07-22-m3-h1a-oracle-route-separation.md` / `07c39295`,`8999bdd0` | **成立理由 + 現行規範へ吸収 + archiveのみ** | 同一routeを置換するとlegacy scriptが非対象surfaceまで破壊した失敗と、product/diagnostic/oracleを分ける規律を保持。hidden compat DOM、legacy ID stub、test queryを製品契約へ入れない。旧17-state matrixは直接移管契約へ吸収済み | [直接移管契約 §10〜12](2026-07-22-m3-react-product-asset-promotion-contract.md#10-visual--interaction-oracle) |

## 3. built-in WebView Hostへ再採択する不変条件

### 3.1 scopeと順序

- 対象はproduct-owned React面を載せる**bundled first-party Host module**だけである。first-party plugin、third-party plugin、community custom UIの公開runtimeではない。
- 固定React sourceのR0〜R6所有移管、mock consumer化、projection / intent境界が先である。docs、mock、dev server、diagnostic routeをrelease sourceにしない。
- WebView Host、native surface統合、focus / IME / a11y、platform lifecycleはG0-9の停止線に従う。React packageがbuildできたことをplatform合格にしない。
- bundle、codec、1 child WebView、layout/focus/reload、macOS、Windowsの失敗原因を一つの実装単位へ束ねない。

### 3.2 bundleとload

1. production bundleはrepository内product sourceから決定的に作り、配信assetのpath、media type、byte length、hashを閉じたmanifestで列挙する。
2. release起動はnetwork 0を満たし、CDN、localhost、dev server、HMR、file URL、docs/mock routeへ依存しない。
3. Hostはmanifest掲載assetだけを固定originから返す。path traversal、二重decode、query/fragment、別authority、未掲載assetを拒否し、filesystem pathへ入力をjoinしない。
4. top-level navigation、new window、download、form、外部network、任意evalを既定denyにする。必要能力は後続のtyped capabilityとして個別に足す。
5. build tool、dependency version、manifest format、CSP、custom schemeの具体値は実装直前のcurrent codeと一次資料で再固定する。2026-07-22の旧数値をコピーしない。

### 3.3 typed transportとsession

1. Rust / Host所有のclosed message schemaを正本とし、Web側型とconformance vectorを決定的に生成する。generic `invoke`、raw Document、raw selection、OS/GPU handle、unbounded JSONを渡さない。
2. Hostがprotocol version、surface role、WebView instance epochをbootstrapする。Webがepochや正本revisionを発行しない。
3. direction、role、epoch、sequence/request IDを検査し、unknown kind/version/field、wrong direction、old epoch、duplicate/stale、oversize、過深nest、非finite、queue fullをtyped rejectする。
4. decode成功だけでsession stateを進めず、bounded event-loop inboxへのenqueue成功と同じ原子結果で進める。IPC callback内でDocument/D2へreentrant mutationしない。
5. Host送信は送達不明を成功扱いせず、必要ならprepare / commit / abortまたは同等の二相境界でsequence消費を決める。上限到達時にwrapせずinstance再生成を要求する。
6. exact field、role閉集合、byte/depth/queue上限、error priority、canonical encodingは現行projection/intentとplatform adapterが固まった後のclosed contractで決める。旧H1の値は候補証拠であって現在のwireではない。

### 3.4 ownershipとlifecycle

- 通常windowは1 top-level wgpu Surface内のnative Stage/Timeline viewportと、dock/tab stack単位のopaque child WebViewを使う。panel一つごとにWebViewを増やさない。
- Host coordinatorがlayout epoch、logical/physical bounds、focus移譲、instance epoch、reload、process lost、retry/backoffを所有する。古いlayout/instance epochを部分適用しない。
- WebViewはDocument、selection、Undo、semantic cacheを所有しない。reload/crash後は同じrevision付きHost snapshotから再投影する。
- repeated crash/loopはbounded retry後にpanelを停止できるが、Host Documentとnative Stage/Timelineを巻き込まない。renderer process分離だけでCPU/RAM強隔離完成を公約しない。
- macOSの合格をWindows WebView2、IME、DPI、a11y、process failureへ外挿しない。

### 3.5 exact contract再作成のSTOP線

次のいずれかが起きたら、旧exact contractを復元して埋めずにG0-9仕様へ戻す。

- R0〜R6より前に縮約product component、別CSS、legacy runtime importが必要になる。
- current source assetとprojection / intentが未確定なのにroleやwire fieldを固定したくなる。
- productionをdocs/mock/dev serverから起動する、またはvisual oracleをrelease module graphへ入れたくなる。
- WebView成立をcommunity plugin UI公開許可、同一origin/process/権限の根拠にしたくなる。
- raw JSON文字列走査、例外握り潰し、unknown fallback、unbounded queueでprotocolを成立させたくなる。
- Stage/TimelineをDOM/Canvasへ移す、transparent WebView、CPU pixel bridge、GPU handle共有が必要になる。

## 4. 製品縦切りとして維持する意味

履歴の「四面同期」は、surface数を増やすことではなく、ownerが一つであることのE2E審判として残す。

```text
product-owned React BrowserのRectangle intent
  → Host Transient preview（release前はDocument不変）
  → accepted releaseだけD2 single writerへ1 macro
  → 同じrevision付きsnapshot / LayerId
       → native Stage
       → native Timeline
       → product-owned React Inspector / KEYS-LAYERS
  → playheadへPosition Key追加
  → React triggerからnative Easing popupを開き左key outgoing Interpを変更
```

- Place、Add Key、Easing Applyはそれぞれ`1 gesture = 1 D2 macro = 1 Undo`である。一連の全操作を一つの巨大Undoへ束ねない。
- 各Undo / Redo後に全surfaceは同じHost revisionを投影する。Timeline bar、Inspector selection、Easing draftを第二Documentとして同期しない。
- drag / curve preview中はsemantic write 0、release時だけcommit 1、Escape / capture loss / focus lossはcommit 0である。
- Easingは左key outgoingだけを変え、key数・時刻・値を変えない。multi-key Graph Viewやglobal Auto Keyをこの縦切りへ混ぜない。
- Rectangle appearance、D2 fresh ID、Vector lowering、selection、headless Timeline等の未決・未実装をUI fixtureで埋めない。具体task状態は現行M2/M3仕様とUnit 2Cの処分を正とする。

## 5. 復活させない旧具体

- Browser JSON fixtureを`BrowserPanelSpec`へ生成し、`egui_taffy` rendererを製品Browserの正規経路にすること。
- archived HTML / `html-react-parser` / trusted legacy scriptをrelease bundleへ含めること。
- `ui/motolii-web`の縮約Browser / Inspector / KEYS-LAYERS / Easing leafを固定sourceの代替として新設すること。
- Node `22.16.0`、npm `10.9.2`、React `19.2.7`、Vite `6.4.3`、Playwright revision 1228、wry `0.55.1`を再調査なしに固定すること。
- `browser|inspector|keys-layers|easing`の4 role、wire 16,384 bytes、depth 8、queue 64、Ping 16、custom scheme名を現在の公開またはprivate contractとして即時復活させること。
- 旧`#plugin-browser-candidate`、`#product-browser-candidate`、test-only query、hidden compat DOMを製品routeとすること。
- egui/native/WebViewというruntime分類からCore、first-party plugin、third-party plugin、trustを推論すること。

## 6. 固定歴史出典

| lineage | 読み方 |
|---|---|
| React mock README | 初版`831cc592`を全文、bridge版`949a9f64`、Browser版`d6204e8a`、最終React台帳`65c8257d`までdiffで確認 |
| Browser egui/taffy spike | `git cat-file -p 942b32654576d56d5cb175dfd5f1f8067d310d08` |
| Tonight vertical slice | 初版`68a60229`を全文、`91c06d48`、`aae7f263`、`c4bd39d6`までdiffで確認 |
| built-in WebView admission | 初版`4e8a551d`を全文、分割版`72c6442d`、入場版`58e50c28`までdiffで確認 |
| H1 exact contract | 実装入場版`49dc2957`を全文、依存待ち版`9644c5a5`とoracle訂正版`4d90711c`のdiffを確認 |
| oracle route separation | 初版`07c39295`を全文、test-only presentation追補`8999bdd0`をdiffで確認 |

これら17 blobは本書でDISPOSITIONEDとする。旧文書のtask status、version、route、exact wireを現行正本として直接参照せず、本書と現行M3/UI契約の変換を経る。
