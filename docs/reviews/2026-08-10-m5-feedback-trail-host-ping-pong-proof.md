# M5 Feedback trailのHost所有ping-pong proof

日付: 2026-08-10
状態: **決定／M5-FEEDBACK-P0 `DONE / PRIVATE PROBE`、SCR-4／製品runtimeは`WAIT`のまま**

## 1. Outcomeと境界

残像のうち、前frameの合成結果そのものを減衰して次frameへ重ねる再帰Feedbackについて、次の最小式が
wgpu texture上で成立するかだけを確認する。

```text
A0 = transparent
An = Current(n) source-over Decay(A(n-1))
```

履歴はplugin／shaderの`&self`でなくHost fixtureが所有する。2枚のRGBA textureをclip開始時に明示的に
transparent clearし、各stepでpreviousをsampleしてotherへ書くping-pongとする。target frameへのseekは
clip開始からfresh replayし、再生headやwall clockを入力にしない。

これはprivate feasibilityであり、[simulation model](../simulation-model.md) §7の製品render経路における
自己出力Feedback禁止を解凍しない。製品化は同文書のL3／StateTrackと
[plugin resource](../plugin-resources.md) §6のHost所有checkpoint境界に従う。

## 2. 既知実装preflight

| 項目 | 裁定 |
|---|---|
| MECHANISM CLASS | 再帰FeedbackのHost所有GPU履歴と決定的replay |
| KNOWN IMPLEMENTATION SEARCH | repoの`RenderSession` frame内ping-pong、M5-R0 offscreen wgpu fixture、TouchDesigner Feedback TOP／AviUtl frame buffer、動画codecのGOP型checkpoint+replay |
| CANDIDATES | 製品`RenderSession`流用、M5-R0へのprivate fixture追加、StatefulFilter |
| ADOPTION ROUTE | M5-R0のdevice／offscreen／readback patternを`REUSE`し、Host所有2 texture ping-pongとclip先頭replayを`PATTERN`転移 |
| REJECTED CANDIDATES | `RenderSession` poolは同一frame内中間targetでprevious-frame意味を持たない。StatefulFilterは純関数、seek、並列評価を壊すため恒久拒否 |
| THIN MOTOLII SEAM | explicit clear → previous texture input → decayed source-over → other texture → swap |
| THIN MOTOLII RESIDUAL | clip identity、fixed step、checkpoint identity／budget、区間無効化、Preview／Export、damage伝播 |
| RETIREMENT | 15 frame上限、手続きdisc、GPU readback、fixture公開関数を製品へ昇格しない |
| BUILD JUSTIFICATION | NONE |
| BUILD | FORBIDDEN |

## 3. 成立したproof

`spikes/m5-known-implementation/M5-R0/src/feedback.rs`は次を自動確認する。

- 2枚のHost所有RGBA textureを両方transparent clearしてからframe 0を評価する
- shaderはprevious textureと明示frame indexだけを受け、履歴を保持しない
- 移動discの過去位置に`0 < alpha < current alpha`のtrailが残り、未描画cornerは透明のまま
- 同じtarget frameをfresh初期条件から2回replayしたreadbackが完全一致する
- private proofの上限を越える要求は`ReplayTooLong`でtyped refusalする

## 4. 維持するWAITと再入場条件

このproofは`TemporalFootprint`のHost解決、Feedback executor、StateTrack、checkpoint store、RoD／RoI damage、
K7 bake置換のどれも実装しない。[implementation ledger](../implementation-ledger.md)の`K7c`と`SCR-4`は
`WAIT`のままで、M5製品runtimeやVism capabilityを`DONE`へ繰り上げない。

製品再入場は少なくとも`K7a → K7b → K7c`のartifact／区間無効化／再freeze境界が実在し、clip identity、
checkpoint間隔、VRAM admission、Draft／Final replay、damage伝播を一契約としてcompileできた時とする。

## 5. 非証明範囲

- 任意長seek、checkpointからの有界replay、disk／RAM退避
- K0 RoD／RoIに基づく部分更新、blur／transform footprint
- texture pool／ResourceLedger／pressure縮退
- 製品Stage、Document、plugin trait、Vism package、Preview／Export
- cross-device bit一致、実素材のtrail品質
