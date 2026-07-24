# G0-9L L1 renderer計測の正本修正

作成日: 2026-07-24
状態: **決定 / Grok R2 P0/P1/P2=0 / G0-9L停止**

## 1. 検出した不整合

[G0-9段階化決定](2026-07-23-m3-g0-9-staged-platform-gates.md) L1は
`direct wgpu`、`direct wgpu + Vello局所pass`、`egui baseline`の三方式を列挙していた。
一方、上流の[renderer再選定](2026-07-21-native-surface-renderer-reselection.md)と
[拡張サーチ](2026-07-21-native-surface-renderer-extended-search.md)は、direct wgpu primitive
batchとVello局所passを別renderer候補ではなく一つの製品候補stackとしている。
CU-0G02もこの候補stackとegui baselineの二方式を同条件で測定し、粒の完了条件は満たした。

CU-0G05Lの反対側reviewで、三方式列挙と、L1が要求するGPU rawをCU-0G02が保存して
いないことを検出した。個別粒のDONEをgate合格へ読み替えず、G0-9Lを停止して正本と
証拠を分けて修復する。

旧[renderer再選定](2026-07-21-native-surface-renderer-reselection.md) §5のA=pure
direct / B=A+Velloはprimitiveと局所Velloの責任境界を確かめるisolated capability
spikeとして残す。text/icon量を揃えたplatform比較ではないためL1 armへ数えず、未実施の
A/B比較をCU-0G02または本修正で合格にしない。

## 2. 裁定

L1の比較armは次の二つとする。

1. `direct_vello`: direct wgpu primitive batchを主経路とし、複雑path/textだけを
   Vello局所passで描く製品候補stack
2. `egui_vello`: 同じVello局所assetを維持し、egui integration costを加えた現行baseline

pure direct wgpuを第三armにしない。同じtext/icon量を満たすには別glyph rendererの新造か
fixture縮退が必要で、製品候補でも同条件比較でもなくなるためである。この修正は
CU-0G02の既存rawをL1合格へ昇格しない。

## 3. CU-0G02Bで追加するraw

既存windowed Timeline spikeだけを`REUSE / WRAP`し、両armへ同一のGPU timestamp queryを
追加する。新しいrenderer、profiler framework、製品API、Document、公開契約は作らない。

- CPU: 既存のframe、present、input-to-present wall time
- GPU: 同一pass境界のtimestamp query raw。query resolve/mapは測定loop外で行い、
  frame loopの同期waitとpixel readbackを増やさない
- memory: RSS raw。Apple unified memoryから専用VRAM値を推測しない
- resource: initialization / warm-up / measuredの生成回数、pixel readback 0
- provenance: rustc、cargo、locked dependency、実行commit、固定Mac構成をrawへ埋める

instrumentation後の同一binaryを使い、同じsessionで`direct_vello`、`egui_vello`を逐次
再実行する。scenario/input/source/font/glyph digest、window、present mode、WebView枚数、
warm-up、測定時間が一致しなければ不合格とする。既存CU-0G02 rawとの数値連結や、
片armだけの再実行を禁止する。

## 4. 状態と非目標

- CU-0G02は定義済み二方式のCPU/input/RSS比較として`DONE`を維持する
- CU-0G02BとCU-0G05Lは未完了で、`G0-9L: PASS`を宣言しない
- 絶対閾値、renderer勝者、egui削除を決めない
- GPU timestampを製品telemetry、profiling API、常設resourceへ昇格しない
- W0b、H1b、Motolii Studio Preview、製品window、G0-9D、G0-6H、G0-3/GAP-13を解禁しない

## 5. 必須負例

- `direct_vello`からVelloを外してtext/icon量を変えた第三armを同条件と呼ぶ
- CPU wall timeをGPU rawと呼ぶ
- Apple unified memoryの空き容量をVRAM予算または使用量と呼ぶ
- query result取得のためframe loopへ同期wait、map、pixel copyを入れる
- CU-0G02の既存rawへGPU値やtoolchainを後付けする
- 計測を通すためfixture、digest、期待値、visual thresholdを変える
- 本修正だけでG0-9LをPASSにする

## 6. 完了条件

本修正は反対側review P0/P1=0後にだけ決定へ上げる。その後CU-0G02Bを別粒・別commitで
実測し、同じく反対側review P0/P1=0まで証拠を採用しない。CU-0G05Lは両者の完了後に
manifestを再構築する。

Grok R1のP1（topology正本に残った三arm表現）とP2を全件反映し、topology、
renderer再選定、UI runtime、段階化L1を同じ意味へ揃えた。R2はP0/P1/P2=0、
`VERDICT: ACCEPT`だったため、本修正を決定としCU-0G02Bだけを次の実測粒へ上げる。
