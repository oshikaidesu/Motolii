# M4-K1a resource owner inventory

状態: **VALIDATION PASS / K1a未完了**

## 目的

ResourceLedgerの型を先に発明せず、現行製品codeでGPU/RAMを確保しているseatを列挙し、
新しいraw GPU allocationやnative external texture importが無会計で入ることを自動拒否する。

## 責任処分

```text
RESPONSIBILITY DISPOSITION: REUSE
EXISTING ROUTE: 現行wgpu生成点、FrameReader、K1a hard budget仕様
OWNED RESIDUE: Motolii固有owner seat、tier、lifetime、事前見積り、拒否理由
IMPORTED RESPONSIBILITY: wgpu resource descriptor。ffmpeg子process内部memoryは観測対象であり正本ではない
EXIT: inventory guardはtest-only。後続製品codeはguardをimportしない
RETIREMENT: ResourceLedger接続後もraw allocation再流入のguardとして維持する
```

## 現行owner seat

| seat | 現行生成点 | tier | lifetime | K1a接続時の停止線 |
|---|---|---|---|---|
| `source-upload` | `motolii-gpu::upload_rgba` | VRAM | caller / render session | `RenderSession`内transparent/solidは`session-source`として接続済み。その他callerは未接続 |
| `decode-materialization-pool` | `YuvToRgba::SizePool`のY/U/V、RGBA×2、uniform | VRAM | converter size generation | 寸法変更時は旧pool解放と新pool admissionを原子的に扱う |
| `render-target` | `create_rgba_render_target`、`RenderTargetPool` | VRAM | render session / graph liveness | 明示accounted constructorは接続済み。legacy constructorとextra targetを未会計のまま完成扱いしない |
| `rendered-frame` | `create_owned_output_texture` | VRAM | consumer handle | worker generation失効だけで生存handleを過小計上しない |
| `preview-display` | `DisplaySlot` | VRAM | UI display generation | UI toolkitを台帳ownerにせずHost seatへ翻訳する |
| `copy-out-staging` | `RgbaDownloader` | VRAM-visible buffer | downloader size generation | exportとcache copy-outを同じ未分離budgetへ黙って混ぜない |
| `pipeline-uniform` | `PipelineCache` | VRAM | pipeline cache entry | pipeline本体のbackend内部memoryをdescriptor byteと偽らない |
| `node-uniform` | Overlay/Composite/Mask/Affine | VRAM | nodeまたはrender call | frame loop内生成を恒久化せず、再利用ownerへ移してから会計する |

`motolii-testkit`自身のtest textureは製品owner inventoryから除外する。

## CPU / external memory

| seat | 現行事実 | 会計可能範囲 | 停止線 |
|---|---|---|---|
| `decoded-cpu-frame` | `FrameReader::next_frame`が毎frame `Vec<u8>`を確保 | `FrameDesc::data_size()` | persistent reader/prefetchの同時frame数が無い間はRAM cache成立としない |
| `ffmpeg-process` | 子process、pipe、codec内部buffer | process RSS等の実測概算 | Host hard capで厳密に予約できるmemoryと称さない |
| `decoder-surface` | 未実装 | surface descriptor / pool count候補 | 数えられないhardware surfaceを製品へimportしない |
| `decoder-dpb` | 未実装 | codec/profile別の実測候補 | 固定係数を製品既定へ焼かない |
| `imported-texture` | 未実装 | native allocation/import情報候補 | device identity、resident bytes、lifetimeを閉じるまで拒否 |

## 自動審判

`crates/motolii-testkit/tests/resource_owner_inventory.rs`は次を固定する。

1. 製品`crates/*/src`のraw `create_texture` / `create_buffer` / `create_buffer_init`
   を持つfile集合とcall数がinventoryと一致する
2. 各fileに少なくとも一つのowner seatがある
3. source upload、decode pool、render target、rendered frame、display、staging、
   pipeline uniform、node uniformを別seatとして維持する
4. `wgpu_hal`、IOSurface、D3D11 texture、DMA-BUF、ExternalTexture等を、
   ResourceLedger entryなしに製品sourceへ追加できない
5. 各seatにlifetime classとpeak multiplicityの根拠があり、call数だけを容量証明にしない
6. ResourceLedger/budget/alignment型をDocument/plugin sourceへ露出しない

このguardはRust ASTやwgpu allocation実体の証明ではない。callsiteが移動・増減した時に、
owner分類を更新せず通過することを防ぐ変更検知器である。

## この粒が証明しないもの

- 全ownerの実割当接続、backend padding、allocator report差分
- ffmpeg子processの厳密hard cap
- hardware decoder surfaceのresident bytes
- pipeline/shader/sampler/driver内部memoryの正確なbyte数
- VRAM/RAM/diskの製品既定値

## 次の判定

**PASS**: inventory guard 4 testは全緑。Opus 5のread-only助言を現行仕様・codeへ再照合し、
閉じたowner enumを`motolii-gpu`公開APIへ出す案と、台帳より先に`OwnerId` registryを作る案を棄却した。
owner identityは開いた診断labelとし、最初の実ownerとして明示budgetの`RenderTargetPool`を接続した。

**STOP条件**: raw allocation数一致を「全memory会計済み」と読み替える、またはffmpeg/native
decoderの不可視memoryをゼロとしてhard capを主張する。
