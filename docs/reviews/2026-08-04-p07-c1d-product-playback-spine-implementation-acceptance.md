# P07-C1D product playback spine implementation acceptance

- 日付: 2026-08-04
- 実装commit: `ea69f5ca`
- 状態: **DONE / ACCEPTED（`P07-C1D`、code/main） / EXTERNAL_GATE_PENDING**
- 正本: [P07-C1D product playback spine contract](2026-08-04-p07-c1d-product-playback-spine-contract.md)

## 1. 受入結果

commit `ea69f5ca` は通常製品React Stageの`#play`をexact
`{"kind":"toggle-playback"}`として既存Hostのbounded inboxへ渡し、`ProductApp`をone
`PlaybackSession`のlifetime ownerへ接続した。program preparationはgeneration-qualifiedであり、pause、
composition end、ruler scrub、Document mutationはactive/preparing sessionをretireする。

再生中のcurrent timeはexisting audio-device `Transport::next_frame_plan()`だけが所有し、返された
`FramePlan::timeline_time`をexisting `editor_playhead`へ直接採用する。同じownerからStage render、Stage
transport publish、native Timelineへ投影し、wake/repaint counterやReact timerによるsecond clockを作らない。

Inspectorは既存product-owned React assetのままである。native Timelineのkey marker geometryは変更せず、
marker xはtime projection、clip/bar widthはduration、zoom/viewportはprojectionだけを担う。shape/type/contentに
応じたdynamic marker width、Inspector copy、汎用transport controller、Space/seek/JKLは追加していない。

## 2. validation

- `cargo fmt --check`: PASS
- `cargo test --locked -p motolii-transport`: PASS
- `cargo test --locked -p motolii-audio`: PASS（hardware smokeは既定どおりignored）
- `cargo test --locked -p motolii-ui`: lib 223/223および対象integrationはPASSしたが、既存
  `raw_input_boundary`が`product_easing_popup.rs`と`product_runtime.rs`のraw `WindowEvent`を検出してFAIL
- Stage playback route / intent guards: PASS (2/2)
- Browser ownershipを含むcombined guard: 16/17 PASS。残る1件は既存Browser/Easing assetのfixed SHA
  `6ae4cf7e...`と現物`fc2ad80b...`の不一致で、本差分のplayback routeではない
- `npm run check:host`: PASS
- normal `npm run build:host`二回: generated tree byte/hash stable
- `git diff --check`: PASS

したがってrepository全体greenは主張しない。二つの既知redは本契約のoracleを阻まず、この受入からguard、
raw-input authority、Easing assetを変更する権限を発生させない。

## 3. 独立reviewとfinding

fresh Opus read-only reviewのfocused rereviewは`ACCEPT / P0=0 / P1=0 / P2=0 /
EVIDENCE_GAP=none`を返し、`FramePlan::timeline_time`からeditor owner、Stage publish/render、native Timelineまでの
direct routeと、counter不変時にtimeが進まないnegative caseを受理した。reviewerはmutationしていない。

先行reviewでは、cancel/staleとなったprogram preparationがworkerへ移したcacheのwarmthを保持しない可能性を
P2として挙げた。正しさ、single-session、clock authorityには影響しないためfindingとして保存し、別施工を
本受入から開始しない。

## 4. 残るgate

real default-device playback、audible pause/end、focus/affordance、visual motionは利用者方針どおりM3-final
`EXTERNAL_GATE_PENDING`へ集約する。P07-C3のreal-material clock measurementも別gateのままであり、automated
validationやLLM reviewで代替しない。M3全体の完了は本受入だけでは主張しない。
