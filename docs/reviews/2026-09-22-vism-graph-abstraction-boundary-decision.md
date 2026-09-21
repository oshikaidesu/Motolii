# VismとGraphの抽象境界決定 — typed semantic componentを作者単位にする

作成日: 2026-09-22

状態: **決定**。VismをWGSL shader、Host plugin kind、FrameGraph nodeそのものとして定義せず、**型付き入出力とparameterを持つ一つのsemantic component**として扱う。Vismの内部実装は一つのprimitiveでも複数nodeからなるsubgraphでもよく、Hostはそれを実行用IRへlowerする。

関連正本: [Vismコンセプト](../vism-package-concept.md)、[Vism / Kitモデル](../vism-kit-model.md)、[Vism作者programの言語境界](2026-08-01-vism-authoring-language-boundary-decision.md)、[Vism Inspector・作者source・Automation責任境界](2026-08-01-vism-inspector-source-automation-boundary-decision.md)

## 1. 決定

Vismの作者・利用者向け境界を次で固定する。

> **Vismは、安定したidentityと型付きcontractを持ち、内部ではHost primitive、TypeScript、WGSL、Simulation、またはそれらからなるsubgraphへ展開できる一つの表現単位である。**

Vismの公開意味は少なくとも次から成る。

- stable identity / version
- typed input / output
- parameter / default
- space / temporal mode
- required capability
- diagnostics / unavailable理由

内部のnode数、shader pass数、worker数、GPU/CPUの分担はVismの公開identityではない。

したがって次を区別する。

```text
作者・利用者の接続
Vism ──typed value──> Vism
        ↓ Host compile / lower
内部実行
Execution Graph / FrameGraph
        ↓
CPU / GPU / decoder / simulation
```

Vism同士のtyped connectionを編集することと、Host内部のFrameGraphを直接編集することを同一視しない。

## 2. Vismはgraph macroに似るが、graph macroそのものではない

「複数nodeを一つに畳んだgraph macro」はVismを理解する有力な類比である。ただし、それだけを定義にしない。

一つのVismは次のどれでもよい。

```text
A. 一つのHost primitive
   [MIDI Events]

B. 一つのGPU kernel
   [Glow]
      ↓
   WGSL pass

C. 複数処理のsemantic subgraph
   [MIDI Bounce]
      ↓
   Note Filter → Envelope → Remap → Transform

D. Host Simulationを要求するcomponent
   [Physics Behaviour]
```

よって、Vism identityと内部subgraph identityを一致させない。内部実装を変更しても、公開contractと互換意味が保たれるなら同じVismとして更新できる。

例:

```text
v1 implementation:
MidiEvents → NoteFilter → Envelope → Remap

v2 implementation:
MidiEvents → PatternDetector → Envelope → Curve

public contract:
MidiEvents → Transform
Strength / Attack / Release
```

内部node構成をProjectの永続identity、Vism package identity、互換判定へ焼かない。

## 3. WGSL、TypeScript、SimulationはVismの「種類」ではない

WGSLをVismと同義にしない。

```text
Vism
 ├─ Host recipe / primitive
 ├─ TypeScript semantic program
 ├─ WGSL kernel
 ├─ Host Simulation / StateTrack
 └─ 上記の組合せ
```

WGSLはpixel、texture、field等をGPUで実装する一つの実行席である。MIDI provider、Beat detector、Data mapper、Path generator等、GPUを必要としないVismも同じVism境界に乗る。

同様にTypeScriptはVismを書く作者sourceの席であり、Vismそのものではない。RustはHost実装であり、通常作者のVism identityではない。

通常UIへ `WGSL Vism`、`TypeScript Vism`、`Rust Vism` のような実装分類を第一語彙として出さない。

## 4. Vism graphとFrameGraphを分離する

利用者が接続するgraphは**authoring / semantic graph**である。HostのFrameGraphは**execution IR**である。

```text
Flutter / Inspector / connection UI
          ↓
Vism instances + typed connections
          ↓
Project semantic graph
          ↓ compile / lower
FrameGraph / execution graph
          ↓ schedule / cache / time edges
Host adapters / GPU / decoder / simulation
```

この分離により、次を可能にする。

- UIを変えてもexecution IRの契約を壊さない。
- FrameGraphの最適化、fusion、cache、parallelismを利用者graphへ露出しない。
- 一つのVismを複数nodeへlowerできる。
- 複数のVismを内部で共有nodeへ畳める。
- GPU pass数とVism数を一致させなくてよい。
- execution plannerのculling、prefetch、cache policyを作品意味にしない。

FrameGraphの`NodeKind`、`NodeKey`、cache key、GPU resource handleをVism packageやProjectの公開形式にしない。

## 5. 接続変更は作品編集であり、Host再buildではない

既存Vismとtyped portを接続し直すことは、Rust source変更ではなくProjectのauthoring data変更である。

```text
Flutter
  ↓
Vism connectionを変更
  ↓
Document revision
  ↓
Hostが実行graphを再compile
  ↓
Preview
```

したがって、例えば

```text
Kick → Scale
```

を

```text
Kick → Rotation
```

へ変えるためにMotolii本体を再buildする必要はない。

Host再buildが必要になり得るのは、公開Host capabilityや新しいnative primitiveなど、既存contractで表せない基盤能力を追加する時である。WGSL/TypeScript等の作者sourceについては、それぞれのadmission / compile契約に従い、Rust再buildと同一視しない。

## 6. MIDI / 音反応を基準fixtureにする

この境界を反証しやすい例としてMIDIを使う。

同じ`MidiEvents` providerから、異なるconsumerへtyped接続できることを要求する。

```text
MIDI File / Live MIDI
        ↓
    MidiEvents
     ├────────→ Timing / Trigger
     │               ↓
     │          Property / Track
     │
     ├────────→ Note Geometry
     │               ↓
     │            Scene
     │
     ├────────→ Particle Emitter
     │
     └────────→ Effect / Camera parameter
```

「MIDI Import」という巨大な専用pluginを作ることを要求しない。

同じ入力から、

- timingとしてBakeする
- live animation sourceとして読む
- geometryへ変換する
- particle triggerへ渡す
- effect parameterへ渡す

をtyped connectionの違いとして表せることを目標とする。

これはAbleton Liveのtimeline/automationとTouchDesigner型dataflowの能力を混ぜるという意味ではなく、時間とdataを同じtyped authoring graphからHostへ渡せることを示すfixtureである。

## 7. Vism Kitとの境界

Vismは一つのsemantic component、Kitは複数Vismを接続して一つの用途へした作者成果である。

```text
primitive / implementation
        ↓
      Vism
        ↓ typed connections
     Vism Kit
        ↓ materialize
      Project
```

例:

```text
Music Reactive Kit

Audio Analysis
      ↓
Beat Envelope ──> Scale
      ↓
Color Mapper  ──> Color
      ↓
Particle Burst
```

Kitは接続、初期値、公開control、asset requirementを持てる。利用者はKitを一つの完成した入口として使え、必要なら展開後のVism instanceと接続を編集できる。

ただし「一つのVismの内部実装を必ず一般利用者が任意に展開・再配線できる」とは決めない。内部subgraphはVismの実装詳細であり、作者sourceやadvanced inspectionとして見せるかは別契約である。

## 8. UIへの含意

初心者へraw node graphを必須にしない。

例えば内部では、

```text
MidiEvents → Filter → Envelope → Remap → Scale
```

でも、通常UIでは

```text
MIDI Bounce

Track:    [Drums]
Strength: [----o--]
Attack:   [20 ms]
Release:  [120 ms]
```

だけを見せてよい。

Hostは同じVism contractからInspector、接続可否、parameter UI、diagnosticsを投影する。

Advanced UIで接続関係を見せる場合も、原則としてVism / typed portのsemantic graphを見せる。FrameGraphの内部node、GPU pass、cache stateはdeveloper diagnosticsへ分離する。

## 9. 必須負例

次を禁止する。

1. `Vism = WGSL shader` と定義し、MIDI/Data/Host-only表現を別plugin体系へ追い出す。
2. `Vism = FrameGraph Node` と定義し、内部最適化単位を公開ABIへする。
3. Vism内部subgraphのnode idやnode数をProject互換identityにする。
4. typed connectionの変更にMotolii本体のRust再buildを要求する。
5. 一つのVismを複数nodeへlowerできない設計にする。
6. 複数Vismから同じimmutable node/valueを共有できない設計にする。
7. KitとVismを同一視し、用途構成を一つの巨大Vismへ詰める。
8. FrameGraphのcache/culling/fusionを利用者の作品意味へ昇格する。
9. 実装言語を通常UIの主要分類にする。

## 10. この決定が固定しないもの

本決定から次を逆算しない。

- 公開graph editorの具体UI。
- Vism package manifest schema。
- Vism内部subgraphを保存する形式。
- TypeScript SDK API。
- WGSL binding syntax。
- Kit file format。
- live MIDI transport / Ableton Link / OSC対応。
- Vismを一操作でexpand/collapseするUI。
- third-party Vism runtimeの具体engine。
- FrameGraphの公開API。

固定するのは**責任境界**だけである。

```text
Vism     = 人が扱うtyped semantic component
Kit      = 複数Vismを接続した用途
FrameGraph = Hostが実行する内部IR
WGSL/TS/Simulation = Vismを実装する席
```
