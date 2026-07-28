# M4-K1a ResourceLedger core

状態: **CORE PASS / K1a未完了**

## 目的

owner別・tier別・resident/pinned別のhard budgetをallocation前に判定し、grantの寿命だけで
解放量が戻るHost内部台帳を製品codeへ置く。Auto予算値、eviction、階層移動は決めない。

## 責任処分

```text
RESPONSIBILITY DISPOSITION: BUILD
EXISTING ROUTE: wgpu allocator reportは診断補助のみ。製品内にhard-cap台帳なし
OWNED RESIDUE: Host owner label、VRAM/RAM/disk tier、resident/pinned、admission、RAII release、snapshot
IMPORTED RESPONSIBILITY: std Mutex/Arc/collections、thiserror
EXIT: motolii-gpuのHost APIに閉じ、Document/plugin/serde/UI toolkitへ型を出さない
RETIREMENT: K1a完成後も正本台帳として維持。test-only ContractLedger modelは製品完成証拠に使わない
```

## 固定した意味

- `ResourceOwner`は開いた診断labelであり、render/UI/nodeの閉じたenumをGPU基盤へ焼かない
- `MemoryTier`は`Vram / Ram / Disk`
- `ResourceRequest`はowner、tier、bytes、pinnedを持つ
- admissionはtier hard capと、設定時だけVRAM+RAM共有capをallocation前に判定する
- 拒否はowner、tier、要求量、使用量、予算、発火したcapを保持する
- `ResourceGrant`はallocation IDを所有し、Drop時にadmit済みrecordの量を返す
- 呼び手はrelease量を再申告しない
- snapshotはtier totalとowner/tier別resident/pinnedを返す
- allocator report、GPU名、空きVRAM、製品Auto値を判定入力にしない

## 自動審判

`motolii-gpu` unit testは次を固定する。

1. grant Drop後にresident/pinned/owner usageがゼロへ戻る
2. tier cap超過はusageを変更せず、完全な型付き診断を返す
3. UMA共有capは個別cap内でも合算超過を拒否する
4. 同一ownerのVRAM/RAM、resident/pinnedを別々に観測できる
5. 空ownerと0 byte要求を型付き拒否し、usageを変更しない

## この粒が証明しないもの

- `GpuCtx`と全raw allocation siteの台帳接続
- descriptorからの全texture/buffer見積り
- decoder subprocess、surface、DPB、外部import memory
- eviction、pin解除、VRAM↔RAM↔disk移動、single-flight
- device lost後のledger再生成
- 製品のbudget既定値、比率、hysteresis

## 次の判定

**PASS**: coreの型と負例は成立した。次は一括wrapperを作らず、inventoryのowner seatごとに
descriptor見積り→admission→create→grant同寿命を接続する。

**STOP**: core単体をK1a完成と呼ばない。既存`GpuCtx::new_*`へ未計測のAuto値を入れず、
unbounded既定をhard cap成立と称さない。
