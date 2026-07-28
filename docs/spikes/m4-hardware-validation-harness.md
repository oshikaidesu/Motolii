# M4 hardware validation harness

状態: **HARNESS PASS / 実機matrix未完了**

## 目的

最低スペックを机上のGPU名で決めず、同じJSON schemaで開発Mac、低スペックWindows、
将来のCI runnerを再計測する。性能閾値と製品Auto予算値はまだ固定しない。

## 記録する事実

- OS、CPU architecture、logical CPU数、物理RAM
- headless wgpu adapter名、backend、device type、driver
- FFmpeg build先頭行と、そのbuildが列挙するhardware acceleration方式
- process startup時間と取得可能なOSでのRSS
- 未配線bench slotとして40-layer render、decode需要matrix、音MAD編集密度

`ffmpeg -hwaccels`の列挙は、実際に対象codec・pixel format・GPU import経路で速いことを
証明しない。adapter名もVRAM空き量や安全なbudget値を証明しない。

## 再実行

```sh
MOTOLII_PERF_BASELINE_OUT=/tmp/motolii-perf.json \
  cargo test -p motolii-testkit --test perf_harness -- --nocapture
```

schema v2のJSONを機種ごとに保存し、同じfixture revisionとMotolii commitで比較する。
現在はpass/fail閾値を持たず、取得不能は`Unavailable`として記録する。

## 次の粒

1. decode需要を連続再生、seek storm、多数短clipへ分け、software decodeを基準線にする
2. OS hardware decodeは同じ入力・同じframe要求列で別routeとして測る
3. 音MAD fixtureはUIの主観でなく、要求生成、cancel、decode、upload、render、表示の
   raw時間とqueue深度を別々に記録する
4. 低スペックWindows実機なしに「AviUtl2より軽い」「スマホより速い」を合格にしない

このharnessは製品runtimeへimportしない計測資産であり、User settings、Document、
plugin契約へhardware情報を焼かない。
