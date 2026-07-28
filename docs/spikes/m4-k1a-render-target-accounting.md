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
