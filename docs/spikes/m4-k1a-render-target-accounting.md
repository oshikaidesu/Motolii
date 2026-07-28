# M4-K1a render-target accounting

状態: **OWNER PASS / K1a未完了**

## 目的

最初の実GPU ownerとして`RenderSession`の中間`RenderTargetPool`をResourceLedgerへ接続し、
admission→texture生成→ownerとgrantの同寿命を製品codeで通す。

## 境界

- `RenderSession::new_accounted`はHostがResourceLedgerと`TextureAllocationAlignment`を明示注入する
- Auto budget、alignment既定値、GPU名による分岐は持たない
- pool ownerは`render-target`、tierはVRAM、session中はpinned
- `AccountedRenderTarget`が裸のtextureとprivate grantを同居所有する
- poolから返すtexture cloneは一回の`render_graph_cached`呼出し内だけで使い、poolが先にDropしない
- pluginの`TextureRef`、Document、serdeへledger/grant/budget/alignmentを出さない

descriptorは`motolii-nodes::rgba_render_target_descriptor`を生成と見積りで共有し、
usage/format/sample/mipの二重定義を避ける。複数初期targetは`admit_batch`で全件preflightしてから
GPU textureを作る。

## 自動審判

1. 2枚poolの生存中はdescriptor見積り2枚分だけVRAM usageがあり、session Drop後ゼロ
2. 2枚目がcapを越えるbudgetではtextureを一枚も持つpartial poolを残さずusageゼロ
3. resize時は旧poolを保持したまま新2枚をpreflightし、拒否時は旧generation/usageを維持
4. branchのlive入力が2枚を占有中、3枚目をcap前にpoolへ追加せず型付き拒否する
5. Document/plugin sourceにResourceLedger等のHost accounting型が現れない
6. 従来のunaccounted `RenderSession::new`と既存pixel/alias試験は変わらず通る

## 未接続と停止線

- `transparent`/`solid` source upload
- `RenderedFrame` output
- YUV plane/output pool
- node uniform、pipeline uniform、copy-out staging、preview display
- 製品UI/exportからのaccounted constructor利用

特にYUV outputと`RenderedFrame.texture`はbare `wgpu::Texture`をowner外へ渡せる。grantをpool/frameへ
置くだけでは外部cloneが長生きした時に台帳が早期解放されるため、K1bの参照handleまたはowned
return契約なしに「接続済み」としない。

既存`RenderSession::new`は移行中のunaccounted経路として残る。この一owner成功だけをK1a完成、
hard budget製品強制、最低スペック成立の証拠にしない。
