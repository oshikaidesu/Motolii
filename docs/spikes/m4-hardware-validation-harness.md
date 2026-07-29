# M4 hardware validation harness

状態: **REEXECUTABLE BUNDLE PASS / 実機matrix未完了**

## 目的

最低スペックを机上のGPU名で決めず、同じJSON schemaで開発Mac、低スペックWindows、
将来のCI runnerを再計測する。性能閾値と製品Auto予算値はまだ固定しない。

## 記録する事実

- OS、CPU architecture、logical CPU数、物理RAM
- headless wgpu adapter名、backend、device type、driver
- FFmpeg build先頭行と、そのbuildが列挙するhardware acceleration方式
- process startup時間と取得可能なOSでのRSS
- 未配線bench slotとして40-layer render
- 配線済みのdecode需要matrixと音MAD編集密度

`ffmpeg -hwaccels`の列挙は、実際に対象codec・pixel format・GPU import経路で速いことを
証明しない。adapter名もVRAM空き量や安全なbudget値を証明しない。

hardware memory factsの取得元はOSごとに閉じる。

| OS | total physical memory | process resident memory |
|---|---|---|
| Windows | `GlobalMemoryStatusEx().ullTotalPhys` | `K32GetProcessMemoryInfo().WorkingSetSize` |
| macOS | `sysctl hw.memsize` | `ps rss` |
| Linux | `/proc/meminfo`の`MemTotal` | `/proc/self/status`の`VmRSS` |

Windows adapterは既存依存閉包の`windows-sys`をtestkitからtarget限定で直接利用する。
製品runtime、Document、plugin APIへOS handleやWindows型を出さない。Windows targetへの
cross-`cargo check`でAPI/feature closureを固定し、値が正であることはWindows実機testで審判する。

## 再実行bundle

最初に機種ごとの出力directoryへ、実行commit、hardware facts、全コマンドのargv、
必要な環境変数、各結果が証明しない範囲を固定する。

```sh
cargo run -p motolii-testkit --bin m4_validation_bundle -- \
  /tmp/motolii-m4-validation \
  dev-mac development-mac ac automatic 1920 1080
```

引数は順に、出力先、匿名化可能な機体ラベル、意図したpersonaラベル、`ac|battery`、
OS上の電源モード名、測定時の表示幅・高さである。personaラベルは測定者の宣言であり、
最低スペック資格の合格判定ではない。

Windows PowerShellでは同じ条件を明示し、出力先だけをWindows pathへ変える。

```powershell
cargo run -p motolii-testkit --bin m4_validation_bundle -- `
  C:\temp\motolii-m4-validation `
  hand-me-down-01 low-spec-windows-candidate ac balanced 1920 1080
```

生成するのは次の3ファイルだけである。

- `hardware.json`: schema v2のOS、CPU、RAM、wgpu adapter、FFmpeg facts
- `context.json`: schema v1の機体／personaラベル、電源条件、表示解像度
- `manifest.json`: schema v5のsoftware decode、hardware-download、音MAD、ResourceLedger、
  階層転送、YUV lane plannerの再実行recipeと分解済み外部gate

`manifest.json`の`program`と`args`を配列のままrunnerへ渡す。shell文字列へ再結合する
必要はない。`env`の値はbundle直下の相対file名であり、executorだけが現地の絶対pathへ解決する。
`required_user_env`は実行者がOSに合わせて必ず指定し、
`optional_user_env`は実素材等の任意入力である。hardware-downloadでは少なくとも
`MOTOLII_DECODE_HWACCEL`と`MOTOLII_DECODE_HW_OUTPUT_FORMAT`が必須であり、未指定のsoftware実行を
hardware結果として保存しない。

bundle作成時点ではdecode／音MADの計測を自動実行しない。実機の素材、OS固有surface、
release実行時間を実行者が確認してからmanifestの個別commandを走らせ、同じdirectoryへ結果を
追加する。

個別commandはrepository rootから専用executorで実行する。

```sh
cargo run -p motolii-testkit --bin m4_validation_run -- \
  /tmp/motolii-m4-validation decode-software
```

Windows PowerShell:

```powershell
cargo run -p motolii-testkit --bin m4_validation_run -- `
  C:\temp\motolii-m4-validation decode-software
```

Windows実機の一括handoff例は次のとおり。`d3d11va`／`d3d11`／`nv12`はWindowsの第一候補であり、
対象FFmpeg buildとGPU driverでの成立を実commandが審判する。

```powershell
$m4Bundle = "C:\temp\motolii-m4-validation"
$env:MOTOLII_DECODE_HWACCEL = "d3d11va"
$env:MOTOLII_DECODE_HW_OUTPUT_FORMAT = "d3d11"
$env:MOTOLII_DECODE_HW_SURFACE_FORMAT = "nv12"

$m4CommandIds = @(
  "decode-software",
  "decode-hardware-download",
  "audio-mad-graph-demand",
  "resource-ledger-contract",
  "tier-transfer-contract",
  "yuv-materialization-plan-contract"
)

foreach ($m4CommandId in $m4CommandIds) {
  cargo run -p motolii-testkit --bin m4_validation_run -- $m4Bundle $m4CommandId
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

cargo run -p motolii-testkit --bin m4_validation_verify -- $m4Bundle
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Compress-Archive -Path "$m4Bundle\*" -DestinationPath "$m4Bundle.zip"
```

先に`ffmpeg -hwaccels`で`d3d11va`が列挙されることは必要条件の一つだが、十分条件ではない。
hardware commandが失敗した場合、software結果をhardware結果へ改名せず、そのbundleを不完全なまま
保全して原因を記録する。QSV、DXVA2等へ切り替えるなら新しい空bundleを作り、contextとrouteを
別証拠として収集する。

executorは現在のcommitとschemaからpath非依存manifestを再生成し、既存JSONとの完全一致を要求する。
tracked、staged、untracked差分があるworktree、必須環境変数の未指定・空値、
既存artifact／log／run recordへの上書き、成功終了後の期待artifact欠落をfail closedで拒否する。各実行は
`run-<command-id>.json`、stdout、stderrを保存し、exit code、所要時間、artifact byte数を記録する。
manifest、hardware、context、stdout、stderr、結果artifactのSHA-256も記録し、後から同名fileへ
差し替えた結果を同じ実行として扱わない。環境変数の値はrun recordへ複製せず、
指定済みの名前だけを記録する。
run recordのlog／artifact pathもbundle直下の相対file名だけを記録し、`..`、絶対path、
subdirectoryを拒否する。これにより完成bundleを別directory・別OSへコピーしても、内容とrevisionが
同じならverify／matrix入力に使える。コピー後の検証で元の絶対pathを信頼しない。
収集機ではbundleが記録したcommitをcheckoutし、archiveを新しいdirectoryへ展開してからverifierを
再実行する。異なるHEADで旧bundleを受理する互換fallbackは持たない。

全command収集後は専用verifierを実行する。

```sh
cargo run -p motolii-testkit --bin m4_validation_verify -- \
  /tmp/motolii-m4-validation
```

verifierは全commandのrevision、manifest/hardware/context digest、exit、log/artifact digest、
software/hardware decodeのfixture identityを照合する。hardware factsのOS、architecture、
total memory、全必須sampleのRSSとstatus、contextの必須項目と値域も確認する。ただし合格するのは
`local_evidence_valid`だけであり、`low_spec_windows`、GPU surface import、製品Preview等の
外部gateは`external_gates_pending`として残す。

## 機種間matrix

2台以上の完全なbundleを集めた後は専用比較器で同じrevision／fixtureだけを並べる。

```sh
cargo run -p motolii-testkit --bin m4_validation_compare -- \
  /tmp/motolii-m4-validation-dev \
  /tmp/motolii-m4-validation-low-spec-windows
```

比較器は各bundleへ単体verifierを再適用し、一件でも不完全、改変済み、別revisionなら拒否する。
decode fixtureのbyte数またはSHA-256が異なるbundleも拒否する。出力するのは次の生値だけである。

- contextの機体／personaラベル、電源、表示解像度
- OS、architecture、logical CPU数、物理RAM、wgpu adapter facts
- 同一command内software／hardware-downloadの120-frame sequentialと8-request parallel wall
- frame 0 differing bytes
- 音MAD fixtureのclip／effect数、最大active slot、最大graph steps、sequential／scrub最大時間

`cargo run`全体の時間、compile時間、run record生成時間は性能値へ混ぜない。
matrix schema v1は`thresholds_selected: false`、`repetition_policy_selected: false`、
`low_spec_windows_gate_closed: false`を固定する。
比率、順位、合否、製品budgetは出力しない。単回値の比較を統計的な性能差や最低スペック認定へ
昇格させず、warm-up、反復数、集約統計、外れ値規則は実機matrix取得後の別粒で決める。

機種間比較は同じfixture revisionとMotolii commitで行う。現在はpass/fail閾値を持たず、
取得不能は`Unavailable`として記録する。

Windows実機で`total_memory_bytes`または各sampleの`idle_rss_bytes`が`null`なら、そのbundleは
最低スペック比較へ採用しない。API取得失敗を0 bytesや推定値へ置き換えない。
`context.json`の`intended_persona`へ`low-spec-windows`と書くだけでもgateは閉じない。
現時点では対象personaの数値資格自体が未決であり、同じ機体でも電源・表示条件が異なるrunを
同一条件として比較しないための来歴としてのみ使う。

## 観測とpolicyの分離

`unresolved_policy_inputs`の`selected_value`は意図的に`null`で固定する。

- VRAM hard budget
- texture allocation alignment
- YUV live lane cap

これらはhardware factsから自動計算しない。低スペックWindowsのworking set、
backend別allocation観測、製品lifetime ownerを得た後、別のpolicy採択として決める。
manifestへ数値を手入力して製品既定値の正本にしない。

`external_gates`は`low_spec_windows`、`native_decoder_surface_import`、
`wgpu_external_texture_lowering`、`surface_lifetime_fence`、
`gpu_surface_pixel_oracle`、`product_preview_path`を`pending`で列挙する。
一台のMac、CPU download、個別benchだけでは自動的にpassへ変わらない。

GPU関連gateを一つへ畳まない。native decoder surfaceの取得、OS/backend固有import、
import済みplane viewのwgpu lowering、GPU完了までのsurface寿命、画素審判は別責任である。
詳細は[GPU surface import境界](m4-gpu-surface-import-boundary.md)を正本とする。

## 残る粒

1. OS hardware decodeは同じ入力・同じframe要求列で、native surface取得、import、
   lowering、renderを分けて測る
2. 音MAD fixtureは要求生成に続き、cancel、decode、upload/import、render、表示の
   raw時間とqueue深度を別々に記録する
3. 低スペックWindows実機なしに「AviUtl2より軽い」「スマホより速い」を合格にしない

このharnessは製品runtimeへimportしない計測資産であり、User settings、Document、
plugin契約へhardware情報を焼かない。
