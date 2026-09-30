# Vism resource capability law — WGSL が見える世界は Host が決める

日付: 2026-09-30  
状態: **DRAFT / contract proposal — runtime変更なし**

関連正本: [Vism package concept](../vism-package-concept.md)、[Vism / Kit model](../vism-kit-model.md)、[Shadertoy import](../vism-shadertoy-import.md)、[effect extent and spill](2026-09-12-effect-extent-and-spill.md)、[transmission backdrop](2026-09-08-transmission-backdrop.md)、[vgpu → Vism](../../motolii/reference/vgpu-vism.md)。

## 1. 裁定案

> **A Vism can only observe and modify resources explicitly granted by its Host evaluation scope.**

WGSLを「安全な言語」へ縮めない。WGSLは与えられたGPU resourceの中では自由に計算してよい。  
代わりに、HostがVismへbindするresourceを表現の意味から決める。持っていないtexture / buffer / history / backdrop / layerにはアクセスできない。

これは新しいplugin分類ではない。既存Vism憲法の
「shader + 宣言された型付きinput + hostが渡す時間/値」
「resource/cache/stateはHost所有」
「first-partyも公開境界を通る」
をGPU bindingまで具体化する。

## 2. 現在すでに成立している部分

現行 `VismProgram` は、manifestが宣言したimage/parameterとHost所有の中間targetだけをbindする。shaderがDocumentや他layerを探索する口はない。

既存の明示的な例:

| 意味 | 現在の宣言/経路 | Hostが渡すもの |
|---|---|---|
| 通常effect | image input | 自分のresolved picture |
| multipass | `PASSES/TARGET` | Host所有scratch target |
| feedback | persistent target | Host所有prev/next |
| backdrop | `BACKDROP_INPUT` | 自分より下の合成 |
| named layer | image + `LAYER` | 指定LayerIdのresolved picture |
| time remap | `TIME_OFFSET/TIME_AT` | Hostが評価した別時刻のpicture |
| matte | matte Vism | source + explicit matte |
| clock | `TIME/TIMEDELTA/FRAMEINDEX` | Host clock |

したがって問題は「WGSLが何でも読める」ことではなく、**Hostが暗黙に広すぎるresourceをbindしないこと**である。

## 3. Scope と capability を混ぜない

**scope** = Vismがどの意味へ配置されているか。  
例: layer/object、composition、material、environment。

**capability/input** = その評価で何を観測できるか。  
例: source、backdrop、matte、history、named layer、environment。

同じobject/layer scopeでも、通常effectはsourceだけ、Background Copy/Glassは明示されたbackdropを追加で持てる。

「object effectだからbackgroundを絶対読まない」ではなく、**backdropを宣言していないobject effectにはbackground bindingが存在しない**を契約にする。

## 4. 最小のresource law

### VISM-R1 — Source locality

通常のlayer/object effectへ渡す既定imageは、そのlayer/objectのresolved pictureだけ。

Composition全体を便利な既定sourceとして渡してはならない。

### VISM-R2 — Backdrop is opt-in

下の合成を読めるのは、既存の `BACKDROP_INPUT` 等で明示的に要求したVismだけ。

backdropを要求しないeffectのbinding layout / source listにbackdropを追加しない。

### VISM-R3 — Named foreign layers are explicit

別layerを読むにはtyped LayerId入力が必要。名前検索、暗黙の「隣」、Document全探索からshader inputを作らない。

既存のclip-to-below等、別の明示意味を持つHost operationはその契約を正本とする。

### VISM-R4 — History is Host-owned

feedback/historyはHostがprev/next lifecycleを所有する。Vism instanceが隠れたpersistent GPU stateを所有しない。

scrub/restart/reuseの意味はHostが決める。

### VISM-R5 — Intermediate targets are Host-owned

multipass targetはVismが宣言できるが、allocation/reuse/lifetimeはHost所有。Vismはraw device resource discoveryをしない。

### VISM-R6 — Output cannot mutate unrelated inputs

Vismの出力はHostが指定したdestinationへだけ書く。他layer、backdrop、matte、history inputをin-placeで変更しない。

### VISM-R7 — Spill is not backdrop mutation

Glow/blur/shadow等がsource coverage外へ新しいpixelを生成することは合法。  
それは「背景を書き換える」ことではない。

既存の `PADDING` / `SPILL` がこの意味を持つ。テストでは次を区別する:

1. source由来のhaloが背景の上へ合成される — 合法
2. backdrop capabilityを持たないeffectが背景そのものの色/alphaを入力として変質させる — 違法

### VISM-R8 — Global resources are never ambient

time、camera、audio、environment、composition picture等を「どのshaderからも見えるglobal」にしない。必要なものだけHostが宣言された入力として供給する。

既存clockはHost提供の明示reserved inputとして扱う。

### VISM-R9 — Document mutation is not a shader capability

WGSLはDocument、Undo、selection、Timeline、plugin instanceを変更しない。GPU resultをDocument truthへ逆流させない。

### VISM-R10 — First-party has no privileged binding

Glow、Blend、Matte、Glass等のfirst-party Vismだけが非公開texture/bufferへアクセスする抜け道を作らない。共通能力が必要ならVism contractの能力として審判する。

## 5. Shadertoyとの対応

Shadertoyの重要な先例はshaderのコードではなくHost binding modelである。

shaderは `iChannel0..3` を自力で発見しない。Hostが各render passのinput/outputを決める。Buffer feedbackもHostの配線である。

MotoliiのShadertoy importも同じ原則にする:

- generator: sourceを必要としない
- ordinary effect: declared sourceだけ
- backdrop effect: explicit backdrop input
- feedback: Host-owned previous target
- extra channel: user/manifestが宣言したresourceだけ

「Shadertoy互換だからcomposition textureを自動でiChannelへ入れる」は禁止。

## 6. 必須のconformance fixtures

runtime変更を始める前に、現在の経路が次を満たすか既存testを棚卸しし、無いものだけ追加する。

### C1 — object-local isolation

背景B + object A。Aに通常Vism effectを適用。

Aの外で、effect on/offによってBそのもののresolved pixelsが変わらないこと。  
Glow等のspillを持つeffectはfixtureから除くか、source由来spillとbackground mutationを分離して判定する。

### C2 — explicit backdrop

同じfixtureでbackdrop非宣言Vismにはbackdropがbindされず、`BACKDROP_INPUT`を宣言したVismだけが下の合成を読める。

### C3 — matte isolation

matteはsourceと明示matteだけを読む。matte sourceは補助入力として扱い、二重描画しない。

### C4 — history ownership

feedback Vismをscrub/restart/reuseし、HostのFeedbackStep契約どおりの結果になる。shader/instance側の隠れstateに依存しない。

### C5 — first-party parity

first-party Blend/Matte/Glow等が第三者Vismでは使えないprivate binding pathを要求していないこと。

## 7. 監査対象

次の実装で「宣言より広いresource」が渡っていないか確認する:

- `compositor/effects/vism.rs`
- image source construction / `sequential_inputs`
- `BACKDROP_INPUT`
- `TYPE: layer` / `LAYER`
- matte
- blend
- feedback state
- glass/transmission/environment
- Shadertoy lowering
- FrameGraph effect inputs

特に、便利さのためにcomposition-sized textureを通常effectのsourceとして渡していないかを調べる。

## 8. 今回決めないもの

- compute Vism / storage bufferの公開contract
- audio input
- dynamic Vism package loader
- 新しいuniversal capability enum
- Vism package manifestの恒久field
- Camera/environmentの最終公開型
- Kit runtime

現行意味を監査する前にpermission systemを新設しない。

## 9. 実装順

1. 現行binding inventoryを作る。
2. C1–C5を既存testへ対応付ける。
3. 修正前に落ちるfixtureがある場合のみruntimeを変更する。
4. capability enum/schemaを足す前に、既存ISF/Vism宣言（image, BACKDROP_INPUT, LAYER, persistent target等）で法律を表現できるか確認する。
5. first-partyの抜け道が見つかったら、個別例外ではなく共通Vism boundaryへ戻す。

## 10. 完了条件

この草案の価値はrule数ではなく、次が成立すること:

> **WGSLは強いままでよい。Vismが見える世界はHostが宣言したresourceだけであり、scope外の意味はそもそもbindされない。**

Object-local effectへbackgroundを渡さなければ、「objectだけに掛けたかったshaderが背景まで加工する」はshader作者の注意力に依存せず構造上起こせない。
