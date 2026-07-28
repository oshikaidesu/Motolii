# M4-K1a render-target accounting

状態: **OWNER PASS / K1a未完了**

## 目的

最初の実GPU ownerとして`RenderSession`の中間`RenderTargetPool`とsession内
`transparent`/`solid` source textureをResourceLedgerへ接続し、
admission→texture生成→ownerとgrantの同寿命を製品codeで通す。

## 境界

- `RenderSession::new_accounted`はHostがResourceLedgerと`TextureAllocationAlignment`を明示注入する
- Auto budget、alignment既定値、GPU名による分岐は持たない
- pool ownerは`render-target`、tierはVRAM、session中はpinned
- source ownerは`session-source`、tierはVRAM、session中はpinned
- `AccountedRenderTarget`が裸のtextureとprivate grantを同居所有する
- `AccountedSourceTexture`もcached source textureとprivate grantを同居所有する
- poolから返すtexture cloneは一回の`render_graph_cached`呼出し内だけで使い、poolが先にDropしない
- pluginの`TextureRef`、Document、serdeへledger/grant/budget/alignmentを出さない

descriptorは`motolii-nodes::rgba_render_target_descriptor`を生成と見積りで共有し、
source uploadは`motolii-gpu::rgba_upload_descriptor`を共有してusage/format/sample/mipの
二重定義を避ける。複数初期targetは`admit_batch`で全件preflightしてからGPU textureを作る。
sourceの色・寸法変更も新generationをadmitしてから旧grantを落とし、拒否時は旧textureと使用量を
維持する。

## 自動審判

1. 2枚poolとtransparent/solid sourceの生存量がowner別に一致し、session Drop後ゼロ
2. 2枚目がcapを越えるbudgetではtextureを一枚も持つpartial poolを残さず、先行source usageだけを維持
3. resize時は旧poolを保持したまま新2枚をpreflightし、拒否時は旧generation/usageを維持
4. branchのlive入力が2枚を占有中、3枚目をcap前にpoolへ追加せず型付き拒否する
5. Document/plugin sourceにResourceLedger等のHost accounting型が現れない
6. 従来のunaccounted `RenderSession::new`と既存pixel/alias試験は変わらず通る
7. transparentとsolidの併存を2 source分として数える
8. source replacement拒否は旧pixelとusageを維持し、成功時は旧generationを解放する
9. accounted/unaccounted solid経路のpixelが一致する

## 未接続と停止線

- `RenderedFrame` output
- YUV plane/output pool
- node uniform、pipeline uniform、copy-out staging、preview display
- 製品UI/exportからのaccounted constructor利用

特にYUV outputと`RenderedFrame.texture`はbare `wgpu::Texture`をowner外へ渡せる。grantをpool/frameへ
置くだけでは外部cloneが長生きした時に台帳が早期解放されるため、K1bの参照handleまたはowned
return契約なしに「接続済み」としない。

pool cloneが一回のrender呼出し内だけという境界も、現状はfirst-party plugin規律を前提にする。
公開`TextureRef.texture: &wgpu::Texture`からthird-party pluginがraw cloneを保持できるため、
scannerやprivate grantだけを強制所有権の証明にしない。plugin texture ABIの変更はこの粒で行わない。

また製品accounted constructorにはbudgetとbackend allocation alignmentの供給元が必要だが、
Auto値やGPU名分岐は未採択である。供給元決定前にUI/exportへ接続しない。

既存`RenderSession::new`は移行中のunaccounted経路として残る。この一owner成功だけをK1a完成、
hard budget製品強制、最低スペック成立の証拠にしない。

## 2026-07-29 budget provenance / reference handle判定

状態: **STRUCTURAL DECISION / PRODUCT WIRING STOP**

### budgetとalignmentの供給責任

`ResourceBudgets`の製品ownerはrenderer、Document、pluginではなくHost runtime bootstrapとする。
User settingsの`Auto`または明示絶対上限を、起動時にHostがVRAM/RAM/disk/shared capへ解決し、
同じledgerをrender、decode、display、copy-out ownerへ注入する。UI toolkit、個別`RenderSession`、
pluginが別予算を作らない。

ただし`Auto`の数値は未決である。GPU名、総RAM、`ffmpeg -hwaccels`、一台のMac観測から算出せず、
[hardware validation bundle](m4-hardware-validation-harness.md)の低スペックWindows gateと
製品preview計測後にUser settings policyとして採択する。現在の製品callerが
`RenderSession::new_accounted`へ移れないのは、constructor不足でなくこのpolicy入力がまだ無いためである。

`TextureAllocationAlignment`はbackendから必ず取得できる事実値とは扱わない。descriptorのtight byte数と
backend allocatorの実使用差を診断し、portableな保守的accounting policyを別途採択してHostから注入する。
実装者が`row_bytes=1 / allocation_bytes=1`やcopy alignmentをproduct既定へ流用しない。

### 既存型を変えないreference handle案の判定

現行公開型を変えずに、underlying allocationとgrantの寿命を完全に一致させる案は無い。

| 候補 | 判定 | 理由 |
|---|---|---|
| `RenderedFrame`へprivate grantを同居 | **正しい方向 / 仕様粒が先** | textureとgrantを同じownerへできるが、全field publicの現行構築契約を閉じる公開API変更になる |
| 新しい`AccountedRenderedFrame` wrapper | **単独では不足** | consumerが裸の`RenderedFrame`/textureへ剥がすとgrantを先にDropできる |
| session所有のbounded output pool | **棄却** | `RenderedFrame`は次renderとsession寿命を越えてpixelを保持できる契約であり、session Drop時に過小計上する |
| raw texture identity registry | **棄却** | `wgpu::Texture::clone`と全Dropを追跡できず、hidden global stateと二重ownerを作る |
| plugin conformance scannerだけでclone禁止 | **補助のみ** | 純関数違反の検出には使えるが、safe Rust型としてunderlying allocation寿命を証明しない |

`RenderedFrame`はK1bの参照handle契約とM3 worker/display deliveryを同時に再照合し、
opaqueな所有型、private field、accessor、consumer移行、旧構築口の扱いを仕様で先に決める。
それまではoutput textureをaccountedと称しない。

plugin `TextureRef`はborrow自体の寿命はrender callへ閉じるが、publicな
`&wgpu::Texture`からthird-party pluginがraw handleをcloneして内部保持できる。
これはplugin純関数契約違反として拒否すべきだが、K1aだけで能力を型から除去できない。
texture view作成までopaque Host commandへ変える案はplugin ABIの大きな変更なので、このowner粒へ
押し込まない。first-party pluginがcall外へ保持しないことと、任意third-party codeに対する
強制所有権を同じ証明として扱わない。

### 次の許可粒

1. `RenderedFrame`所有変更の仕様粒: M3 delivery利用者とK1b handle要件を閉じ、公開互換方針と負例を決める
2. Host budget policyの計測粒: bundleを低スペックWindowsと製品previewで実行し、観測とpolicy採択を分離する
3. YUV materialization製品粒: 上記handleとbudget入力が成立後、size-keyed lane poolをatomic admissionへ接続する

この順序より前に`RenderSession::new`を削除する、仮の巨大budgetで製品callerを移す、
grantをsessionまたはglobal registryへ置く、plugin ABIを局所変更する場合はSTOPする。
