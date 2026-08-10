# storage-to-GPU direct I/Oが裏付ける最小Core境界（2026-08-06）

状態: **観察／既決の最小Core境界を補強／BUILD FORBIDDEN**

対象: [memory model](../memory-model.md)、
[M4既知実装採択・並列実装地図](../m4-known-implementation-adoption-map.md)、
[plugin resource境界](../plugin-resources.md)、[絶対規律](../README.md#全体で守る規律コードレビュー最重視項目)

## 1. 結論

NVIDIAによるcuFile APIと下層storage stackのオープン化発表は、disk／storage上の検証済みartifactを
CPU system memoryのbounce bufferへ常に固定せず、Host所有GPU resourceへ直接昇格できる機構classが
実在し、複数vendor／platformへ共通化しようとする業界側の動きがあることを裏付ける。

この調査から新しい公開境界を決定するのではない。既決のVRAM常駐、Host専権cache、vendor／OS非依存契約を
「Coreはartifactの意味、Hostは到達経路」と読む設計姿勢に外部の既知機構が整合する、という観察に限定する。

Motoliiの最小Coreは、artifactの同一性、descriptor、作品時刻／Qualityとの関係、利用可能なGPU resourceの
意味だけを所有する。storageからそのresourceへ到達した具体経路はHost privateな実行policyであり、
Document、journal、公開plugin API、Vism package、cache keyの作品意味へ入れない。Hostは能力、hard budget、
format、failureを観測してportable stagingまたは将来のplatform direct I/Oを選び、backend差でpixel、
Preview／Export意味、永続形式、typed failureの意味を変えない。

これはcuFileを採用する決定でも、新しいtrait、設定UI、M4親項目、implementation ledgerの`DO`を作る決定でも
ない。具体backend名と公開型は、実コード、license、supported platform、wgpu resourceへの安全な接続、
代表Motolii workloadのend-to-end実測が揃うまで固定しない。

## 2. 一次資料で確認した事実

### 2.1 NVIDIAの発表

NVIDIAは2026-08-04の公式blogで、cuFile APIとその下の垂直storage software stackをopen source化すると
発表した。GPUがstorageを直接read／writeできること、Google、Intel、NVIDIA、Metaを初期maintainerとする
新しい公開先としてxio-sigを示し、複数software／hardware platformへ最適化できる共通面を意図している。
Storage-Next／SCADAはAI infrastructureを主対象にし、Motolii向け製品APIやwgpu接続を提示していない。

### 2.2 既存GPUDirect Storageの成立範囲

NVIDIA GPUDirect Storageの公式Design Guideは、storageとGPU memory間のdirect DMAによりCPU system memoryの
bounce bufferを避ける機構を説明する。同時に、CPUが転送データを前後でparse／processするapplicationでは
利益がなく、現行適用条件としてCUDA、cuFile、対応NVIDIA GPU、明示I/O、pinned GPU memoryを挙げる。
Overview／Troubleshooting Guideは、`libcufile.so`、CUDA runtime、Linux kernel component等のdeployment責任と、
未対応条件ではPOSIX／CPU pathへ戻るcompatibility modeを明記する。

したがって一次資料が証明するのは「GPU-first／lastのartifact I/OでCPU bounceを除去できる既知機構」と
「高速pathとportable fallbackを同一application-facing面の内側で選べる先例」である。macOS／Windowsの
Motolii通常製品route、wgpu textureへのdirect DMA、圧縮動画decode、色変換、pixel parityは証明しない。

### 2.3 xio-sigの公開状態

2026-08-06にGitHub organizationと固定commit
[`4c8172b63afc4e96f2a8938278927f9ba80ca6ae`](https://github.com/xio-sig/.github/commit/4c8172b63afc4e96f2a8938278927f9ba80ca6ae)
を確認した。profile READMEは次の4 projectを予定する。

- `cuFile`: 複数platformへ拡張可能な統一API
- `cuFileConformance`: 相互運用性を検査するconformance suite
- `libxFile`: 複数platform向けuser-level implementation
- `xioLinux`: 必要なkernel enhancementを持つdownstream Linux kernel

同READMEはfounderが全layerを統合・検証しcommit arrangementを整えた後にcodeが現れると明記する。
観測時のpublic repositoryはApache-2.0の`.github`一つであり、上記4 projectのsource、release、API、CI、
platform matrix、個別licenseはまだ公開されていない。発表とprofile READMEを、利用可能なdependencyまたは
wgpu interop成立へ繰り上げない。

## 3. 現行Motoliiとの照合

現行`motolii-media::FrameReader`はffmpeg子processからraw YUV420pをCPU `Vec<u8>`へ読み、
`motolii-gpu::YuvToRgba`は各planeを`wgpu::Queue::write_texture`でGPU textureへ転送する。cuFileはcodec decodeを
行わず、公式資料もCPUがdataをprocessする場合は利益がないとしているため、現在のMP4→ffmpeg pipeへcuFileを
足してもCPU frameとuploadを除去できない。

近い将来候補は、圧縮source一般ではなく、K7 bake、K8全曲Draft、proxy、将来のpoint-cloud node等、
検証後のbytesをGPUがfirst／last touchできるGPU-ready artifactである。それでもK1cの階層admission、完全key、
artifact integrity、bounded lifetime、device loss、portable fallbackが先であり、direct pathを新しいownerにしない。

## 4. 最小CoreとHostの責任境界

### Core／作品側が所有するもの

- version付きartifact identityと内容検証
- consumerが要求するdescriptor、time、Quality、色／premul意味
- backend非依存の成功結果とtyped failure
- Preview／Export同一評価、cacheの意味透明性

### Host privateが所有するもの

- portable staging、platform direct I/O、compatibility fallbackの選択
- GPU resource allocation、registration、lifetime、hard-budget admission
- driver／filesystem／alignment／topology capabilityの観測と診断
- backend別依存、license、platform support、device loss／cancel／cleanup

### 公開面へ出さないもの

- `cuFile`、CUDA、Metal、DirectStorage等のbackend名とhandle
- direct pathの有無、cache warmth、residencyを作品pixelの隠れ入力にする状態
- backend選択を保存するDocument／journal／Vism field
- backendごとの別render、別decode意味、別色変換

将来利用者設定が必要でも既定はHost選択の`Auto`とし、portable強制はdiagnostic／compatibility用のUser settingsに
限定する。vendor名付きtoggleを作品設定にしない。この観察だけで設定項目を実装しない。

## 5. 既知実装preflight

```text
MECHANISM CLASS: verified artifactをstorageからHost所有GPU resourceへ昇格する明示I/O
KNOWN IMPLEMENTATION SEARCH: current ffmpeg/wgpu upload、memory-model、M4 map、NVIDIA GDS公式資料、xio-sig固定README
CANDIDATES: 現行portable CPU staging、NVIDIA cuFile/GDS、将来xio-sig cuFile/libxFile/conformance
ADOPTION ROUTE: 現行portable routeはREUSE。direct I/OはPATTERN（観察のみ）で、dependency採択なし
REJECTED CANDIDATES: cuFile即時依存 :: CUDA/NVIDIA責任、wgpu seam未成立、xio-sig実コード未公開、Mac v1不適合
THIN MOTOLII SEAM: 将来のK1c/K7/K8/GpuAssetCache内のHost-private artifact promotion
THIN MOTOLII RESIDUAL: identity、integrity、hard admission、pixel parity、fallback、device loss fixture
IMPORTED RESPONSIBILITY: 現時点NONE。backend採択時にlicense/driver/kernel/filesystem/platformを再計上
EXIT: source/release/platform matrixとwgpu接続、代表workload実測が無ければ観察のまま
RETIREMENT: NONE。portable routeを常設し、direct pathは置換でなくoptional acceleration
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN
```

## 6. 再入場条件とoracle

次をすべて満たす時だけ、M4既知実装地図の独立した採択probe候補として再評価する。

1. xio-sigまたは別のmaintained primary implementationにsource、license、release、platform matrix、conformanceがある。
2. Host所有wgpu buffer／textureへbackend固有型を公開せず接続でき、portable routeと同じdescriptorを消費する。
3. K1cのallocation前hard admission、bounded in-flight bytes、cancel、device loss、partial publication 0を維持する。
4. GPU-ready artifactでportable stagingとdirect pathを比較し、end-to-end frame latency、CPU使用率、転送byte、
   throughputの少なくとも一つに代表MV workload上の有意な改善がある。
5. backend有無、warm／cold、fallback後で同一machine／driverのpixel、time、color、premul、Final結果が一致する。
6. 圧縮動画へ広げる場合はhardware decode／GPU surface共有を別mechanismとして先に閉じ、cuFileをdecodeと呼ばない。

未達時は`REUSE / REMAP / REDUCE`へ戻し、架空trait、vendor switch、第二GpuAssetCache、独自cross-platform direct-I/O
frameworkを作らない。

## 7. 資料

- [AUTOMATONの記事（発見経路）](https://automaton-media.com/articles/newsjp/20260806-458774/)
- [NVIDIA公式発表: As AI Increases Demands on Memory, Storage Steps Up](https://blogs.nvidia.com/blog/ai-storage-fms/)
- [xio-sig profile README固定commit](https://github.com/xio-sig/.github/blob/4c8172b63afc4e96f2a8938278927f9ba80ca6ae/profile/README.md)
- [NVIDIA GPUDirect Storage](https://docs.nvidia.com/gpudirect-storage/)
- [NVIDIA GPUDirect Storage Design Guide](https://docs.nvidia.com/gpudirect-storage/design-guide/index.html)
- [NVIDIA GPUDirect Storage Overview Guide](https://docs.nvidia.com/gpudirect-storage/overview-guide/index.html)
- [NVIDIA GPUDirect Storage Troubleshooting Guide](https://docs.nvidia.com/gpudirect-storage/troubleshooting-guide/index.html)
- [現行decode path](../../crates/motolii-media/src/decode.rs)
- [現行YUV upload path](../../crates/motolii-gpu/src/yuv.rs)
