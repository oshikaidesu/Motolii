# CU-201N-S Timeline snap契約決定

- 日付: 2026-08-01
- 状態: **SPEC DONE**
- 親: CU-201 / U3b / VS-2

## 1. 結論

固定Mac Local Alphaのsnapを、native Timelineの一つのinterval gesture内で使う
crate-privateな純粋選択へ限定する。Document command、schema、journalへsnap情報を保存しない。

- moveは移動中Clipの左端と右端を候補edgeとし、選ばれた一つの補正量を区間全体へ加える
- trimは利用者が掴んだin/out edgeだけを候補edgeとする
- targetは現行snapshot内の**別Clipのin/out edge**と`Composition.fps`のframe gridだけ
- playheadは正本上Project session ownerだが通常製品routeに値providerがまだ無いため、本粒では候補へ入れない
- beat、user marker、source frame、keyframe、ripple/collision/laneは候補へ入れない

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| `AUTHORITY` | [CU-201S](2026-08-01-cu-201-u3b-move-trim-snap-responsibility-split-decision.md)、[CU-201M-S](2026-08-01-cu-201m-s-clip-start-command-contract-decision.md)、[CU-201T-S](2026-08-01-cu-201t-s-clip-trim-timemap-contract-decision.md)、[M3 U3b](../specs/M3-ui-integration.md) |
| `INTERNAL TARGET` | `TimelineBar::{layer,start,end,x_start,x_end}`、`Composition::fps`、`RationalTime::{try_to_frame_round,try_from_frame}`、`SetClipStart` / `TrimClipIn` / `TrimClipOut` |
| `OWNER` | snap candidateとpreviewはHost Transient。確定intervalはDocument、hoverはlocal presentation |
| `WRITE ROUTE` | pointer down snapshot → transient candidate → terminalでlive generation再照合 →既存Writer prepare → journal-first D2 |
| `GAP` | target集合、順位、閾値、no-snap、stale/cancelが未決だった |
| `RESOLUTION ROUTE` | 現行projectionとfps変換を`REUSE`し、実在providerだけへ`REDUCE` |
| `DISPOSITION` | `PASS`。次は`CU-201P`でprivate gestureを既存commandへ接続する |

## 3. 許容距離と座標

許容距離は**8 logical px以下**とする。physical pxやDPI scaleを保存・比較しない。
現在表示中のtime surface幅とviewportから、candidate edgeとtarget timeの双方をlogical xへ投影し、
その差の絶対値で判定する。zoom、window resize、DPI変更の後はそのframeの最新layoutから再計算する。

frame targetはcandidateのexact `RationalTime`を`Composition.fps`で
`try_to_frame_round`し、`try_from_frame`で正準時刻へ戻す。f64×fpsの独自丸めを作らない。
他Clip edgeはsnapshotのexact `RationalTime`をそのまま使う。

## 4. target選択

対象Clip自身の両edgeは除外する。現在viewport外、非正のtime surface、非有限座標、算術overflowで
投影不能なtargetも除外する。残るtargetから次の辞書順で一件だけを選ぶ。

1. 必要なlogical補正量の絶対値が小さい
2. exact tieでは他Clip edge、frame gridの順
3. target timeが小さい
4. 他Clip edge同士なら`LayerId`が小さい
5. 同じLayerId/timeならin、outの順
6. moveの左右edgeが同じtargetへ同距離なら左edge、右edgeの順

frame gridは全時刻に存在するため、距離より種別優先を先に置かない。これにより近いitem edgeを
遠いframeへ押しのけず、同距離だけを安定に解決する。

## 5. no-snap、stale、cancel

8 logical px以内にeligible targetが無ければ、snap補正0のunsnapped candidateを使う。
snap選択失敗をDocument errorへ変換せず、terminalの既存Writer prepareがraw candidateを検証する。

pointer down時に`LayerId`、edge、開始interval、projection generation、layout epochを一度snapshotする。
drag中のDocument write、journal、history、published snapshotは0。release時に次のどれかが成立すれば
commandを作らず終了する。

- Escape、capture loss、window focus loss
- active generation不一致、duplicate/stale terminal
- target Clipがlive snapshotで消失またはClipでなくなった
- layout epochまたはprojection generationがpointer down時から変わった
- releaseがtime surface外

成功releaseだけがlive Writer prepareを一回呼び、same-valueは既存`Ok(None)`、変更値は1 command / 1 Undo。

## 6. CU-201Pの必須oracle

1. moveは左右edgeの最小補正を区間全体へ加え、duration / TimeMap / parent / identity不変
2. trimは掴んだedgeだけを補正し、T-Sのin/out不変条件を維持
3. 距離優先、exact tie、LayerId、edge、moving edgeの全tie-breakが決定的
4. 8 logical px境界は成功、境界外はunsnapped。DPI変更でlogical判定は同一
5. frame targetは正準fps変換だけを使い、29.97fpsを含む
6. 自身edge、viewport外、beat、marker、keyframe、provider不在playheadは候補0
7. drag中write 0、release 1 Undo、same-value / cancel / stale / outsideはwrite 0
8. move/trim後のpublished snapshotをStage / Timeline / Inspectorが同じgenerationで読む

## 7. 非目標とSTOP

- snap toggle、modifier、一時無効化、磁力強度settingを追加する
- playhead providerを本粒で新設する、または固定0をplayheadとして扱う
- beat/user marker/source frame/keyframeを候補にする
- collision、ripple、lane変更、roll/slip/retimeを追加する
- snap結果、logical px、layout、gestureをDocument/journalへ保存する
- raw UI mutation、第二writer、汎用gesture/snap frameworkを作る
- visual threshold/goldenを変更して合格させる
