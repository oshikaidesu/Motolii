# CU-108RDS drop release routing repair選定

- 日付: 2026-07-30
- 状態: **決定**
- `CU-108RDS`: **DONE**
- 次PRODUCT-ASSET `DO`: `CU-108RD`

## 1. 再現事実

固定Macの通常製品routeで、同じ`LeftMouseUp`のlogical pointが次の順で記録された。

```text
kind=appkit-release
kind=place-release generation=...
kind=place-command
kind=document-publish route=place
kind=timeline-hit ... hit=None
```

baselineではAppKit local monitorが`LeftMouseUp`を通常click inboxへ積む一方、Place releaseは
global pressed-button pollから別に生成される。保護中の証拠差分はexact pointを保持する
release slotを追加したが、同じupを通常click inboxにも積む二重sinkを残す。どちらもPlace
terminal後に通常click inboxを排出する効果は同じである。Stage内releaseはlayout上Timeline外
なので観測例の`hit=None`はselectionを変えず、Place admission / deliveryもD2を一回へ
制限する。このためDocument、journal、Undoの破壊またはD2二重commitは確認されず、P0ではない。

一方、Timeline上でreleaseした場合はPlaceが`OutsideStage`で非commitになった後、同じupが
`TimelineHit::Bar`または`TimelineHit::None`としてprimaryを置換またはclearできる。click inboxは
後続turnで現在のlayout / projectionに対してhit-testされるため、releaseの遅延と次event依存も
排除できない。これは通常操作で到達するP1であり、main統合を停止する。

既存のcapture、terminal classification / admission / delivery、Timeline selection試験は個別には
緑だが、一つのupが一つのconsumerだけへ届く排他を検査していない。

## 2. 選定した修復境界

`CU-108RD`はHost private pointer境界だけを修復する。

1. AppKitの一つの`LeftMouseUp`を、内部単調sequenceとexact top-down logical pointを持つ
   一つのprivate queue entryとして生成する。
2. active Placeはarm後に生成された最初のentryを一回だけclaimする。claim済みentryは
   通常clickへ配送しない。
3. arm前の通常clickはPlaceに奪わせず、Place完了後の通常clickも一回だけ維持する。
4. global button-upだけではcommitせず、実`LeftMouseUp`を待つ。時間fallbackを置かない。
5. exact point、wake、focus loss / Escape、latest layout以降の既存terminal分類、
   generation high-water、D2 / primary ownerを変えない。

monitor callback時点のactive flagだけでrelease / clickを分類する案は棄却する。typed intentの
armとAppKit event dispatchの順序が逆転した場合、実releaseを通常clickへ分類してPlaceを
終了不能にするためである。座標または時刻で後段抑制する案も、event identityを再発明して
二重sinkを残すため棄却する。

## 3. 変更許可と非目標

実装発注の閉じた候補allowlistは次だけとする。

- `crates/motolii-ui/src/host_pointer_capture.rs`
- `crates/motolii-ui/src/browser_host_runtime.rs`
- R-DROP sequenceをPlace / Timeline traceへ渡すために必要な
  `crates/motolii-ui/src/product_runtime.rs`
- R-DROP専用Transient trace ownerを追加する場合だけ
  `crates/motolii-ui/src/ui_numeric_trace.rs`と`crates/motolii-ui/src/lib.rs`
- R-DROP排他の専用test fileを追加する場合だけ
  `crates/motolii-ui/tests/cu108rd_drop_release_exclusivity.rs`

Document、journal、Undo、plugin契約、公開API、serde、React、Stage / Timeline外観、
native renderer、generated asset、golden、visual thresholdは変更しない。既存試験の期待値を
書き換えて通さない。

## 4. 必須負例と数値oracle

- armed後の一upはPlace release一回で、clickは0。
- idle中の一upはclick一回で、Place releaseは0。
- arm前click A、arm、up Bの順で、BだけをPlaceがclaimしAは通常clickとして残る。
- claim後の通常click Cは一回だけ届く。
- exact AppKit pointは再取得したglobal pointで置換しない。
- Escape / focus loss後のupをPlace releaseとして再利用しない。
- OutsideStage over TimelineはD2 0、primary変更0。
- duplicate / stale generationは既存high-waterを迂回しない。
- 一つの`up_sequence=N`について`place-release`と`timeline-hit`の出現数合計は常に1。

## 5. STOP

- Escape / focus loss後の物理upを通常clickとして扱う意味を本粒で新しく決める必要がある。
- public API、Document、journal、Undo、plugin契約、永続形式の変更が必要になる。
- 時刻fallback、座標一致、期待値変更、lint抑制で排他を擬似実装したくなる。
- `poll_host_input`、terminal classifier / admission / delivery、D2 ownerの意味変更が必要になる。
- R-DROP以外のUI修理を同じ差分へ入れる。

## 6. 次ゴールhandoff（実装開始指示ではない）

次ゴールのvisual oracleは固定React
`#plugin-browser-candidate` / commit
`56c318edcddab7cf95d263cc2f7dd2b4e6791134`とする。全部をReact化せず、React所有面は
直接source移管し、Stage viewportとTimeline time surface / Graph / Depth /
Easing curveはnative ownerのまま同じ外観と操作を再現する。

- Wave 0: 本P1処分と現差分の安全なmain統合。未完なら次実装をdispatchしない。
- Wave 1: Browser / Inspector / Stage chrome / Timeline KeyTools + native time surface /
  Easing trigger + native popup / shell-resizable panels / Settingsをpanel別read-only
  inventory / parity packetとしてOpus 5へ渡す。コード変更0。
- Wave 2: presentation ownershipだけを扱う。独立React source不在面は固定mock内で同形React化、
  oracle合格、product ownership、mock consumer反転の順に進め、Host接続と束ねない。
- Wave 3: 一面一境界でrevision付きread model / typed intentへ交換する。React local stateは
  hover / popover / focus等だけとし、Document / selection / Undo / session ownerを増やさない。
- Wave 4: shared Vite、generated bundle、asset router、product runtime、provenance、
  coordination docsを一件ずつ直列統合する。
- Wave 5: native Stage / Timeline / Easing / Graph / Depthを別粒でparity化し、
  golden / visual thresholdを書き換えて通さない。

主要STOP候補はlegacy parser / global script残存、不足projection、mock stateの二重owner、
ResizablePanelLayoutとnative topologyの競合、Stage / Timelineの二重owner、
Browser一次分類P41未統一、未接続operationをno-op完成扱いすること、generated集中点、
platform別scroll / focus / IME / AX / z-order、構造化数値ログと期待値assertの欠落である。

二重owner禁止、Document / User settings / Workspace profile / Project session /
Transient / local presentationの分類、未決の公開API・永続形式・plugin契約を先取りしない規律は
既決の横断法であり、後続粒で意味を再審議しない。各packetはdecision-index、GR-UI、GR-PV、
React直接移管契約への既法mappingと負例検収へ落とす。既存の内部parameter / owner表 /
typed commandに対応先がなく、要求達成に恒久意味の新設または変更が本当に必要な粒だけを
局所`ORDER: STOP`へ戻す。このSTOPは契約を発明する施工だけを止める信号であり、親taskの
終端、無期限`WAIT`、成果放棄にしない。

対応先不在または契約矛盾の粒は、次の既存契約接続票を先に作る。

```text
AUTHORITY → INTERNAL TARGET → OWNER → WRITE ROUTE → GAP
          → RESOLUTION ROUTE → DISPOSITION
```

`DISPOSITION`は`PASS / REDUCE / RESOLVE`の閉集合とする。`RESOLVE`は親taskを止めず、
既存targetの`REUSE → REMAP → REDUCE`、Opus 5 read-only相談、共有境界または恒久契約なら
Fable 5 read-only相談、Codex `SPECIFY`の順に、新しい証拠を伴って前進する。同じ問い、
同じ証拠、同じ相談の反復は禁止する。

製品scope内ならCodexが推奨targetを仕様と拒否試験へ固定し、独立した仕様解決粒を
implementation ledgerへ、新しい決定をdecision-indexへ登録してから実装粒を再投入する。
新しい利用者権限、製品scope、不可逆な外部契約だけを当該粒のユーザー選択へ返す。
接続可能な粒と無関係laneは止めない。Wave 0が止めるのは本統合へ依存する後続実装dispatch
であり、read-only調査と無関係laneではない。正式orderは接続票が欠けていれば
`CODEX PRECHECK`を承認せず、`ORDER: STOP`は契約発明を止める施工上の局所guardに限定する。

後続候補はMedia sample placement、Blur first-party reference filter、
Inspector Position / Scale / Opacity / source-aware Color、native Timeline seek /
bar move / trimとする。共有Document / D2 / Undoを同時編集せず、read-only調査は並列、
実装と統合は一契約境界ずつ直列化する。

各review前にClaude Codeのcommand / version / 完全model availabilityを確認する。助言用途で
利用不能またはtimeoutなら、その事実を記録してCursor CLIのread-only助言へ切り替えられる。
ただし正式な「発注」の`claude-opus-5` order gateをCursor reviewで代替した扱いにはせず、
利用不能時は発注をSTOPする。
