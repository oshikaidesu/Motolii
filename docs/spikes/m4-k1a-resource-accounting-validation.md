# M4-K1a resource accounting validation

状態: **VALIDATION PASS / K1a未完了**

## 目的

[M4 K1a](../specs/M4-cache-and-analysis.md)のうち、descriptor事前見積り、hard cap、
共有メモリ合算cap、型付き拒否に必要な最小契約を、公開API・Document・plugin契約を増やさず
検証する。K0のtest-only region modelは昇格・再利用しない。

## 責任処分

```text
RESPONSIBILITY DISPOSITION: BUILD
EXISTING ROUTE: RgbaDownloaderの256-byte row alignment計算のみ。ResourceLedger、descriptor見積り、admissionは該当なし
OWNED RESIDUE: Motolii Hostの割当事前見積り、hard cap、共有メモリ合算、拒否診断
IMPORTED RESPONSIBILITY: wgpu 29のTextureFormat block footprintとCOPY_BYTES_PER_ROW_ALIGNMENT
EXIT: wgpu型はmotolii-gpu内に閉じ、Document/plugin/serde面へ出さない
RETIREMENT: checked copy-buffer見積りは製品経路へ残す。texture/admission modelはtest-onlyで、K1a本体へ自動昇格しない
```

一般cache、allocator、schedulerは新設しない。wgpuのallocator reportは診断補助であり、この検証の
判定入力にしない。

## 成果物

| path | role |
|---|---|
| `crates/motolii-gpu/src/allocation.rs` | checked alignment/copy-buffer見積り。textureとadmissionのmodelは`cfg(test)`内 |
| `crates/motolii-gpu/src/transfer.rs` | `RgbaDownloader`が同じchecked見積りを製品経路で利用 |
| `crates/motolii-gpu/src/ctx.rs` | copy-buffer算術overflowの型付きruntime error |

## 検証した契約

1. RGBA8/RGBA16、mip、MSAA sample数で見積りが変わる
2. 2D array layerは各mipで維持し、3D depthはmipごとに縮小する
3. block圧縮formatとrow/allocation alignmentを数える
4. portableなwhole-texture footprintが無いformatを推測せず拒否する
5. 乗算・加算・alignment overflowを割当前に拒否する
6. tier cap超過時はusageを増やさず、owner/tier/request/used/budgetを保持する
7. UMA想定の共有capは、VRAM/RAM個別cap内でも合算超過を拒否する
8. release後のusageはゼロへ戻る
9. 既存download bufferのrow/total算術は同じchecked関数を通る

## 検証結果

```text
cargo test -p motolii-gpu --locked
test result: ok. 9 passed; 0 failed
origin_guard: 6 passed
swscale_reference: 1 passed
vism_a3_0_fullscreen_uniform16_cache: 2 passed
yuv_golden: 3 passed

cargo clippy -p motolii-gpu --all-targets --locked -- -D warnings
Finished successfully

cargo test --workspace --locked
Exit code: 0

./scripts/check-docs.sh
OK: docs整合チェック全項目通過
```

## 本番へ残せる部分

- `RgbaDownloader`のchecked row alignmentと総buffer量
- overflowをpanic/wrapさせず型付き失敗へ変える経路
- `TextureFormat::block_dimensions` / `block_copy_size`を使うdescriptor見積り手順

最後の項目は現時点ではtest-only modelである。K1a本体では、実際に作るtexture/bufferの全ownerを
一つの台帳へ接続できることを確認してから製品codeへ移す。

Opus 5のread-only再監査後、test-only admission modelは次を追加で固定した。

- admission量はdescriptor見積り由来の`EstimatedBytes`からだけ受ける
- grantがadmit時の量を保持し、Dropで返す。呼び手は解放量を再申告しない
- 個別capと共有capを拒否診断で区別する
- checkedな共有usage加算
- footprint不明のHost allocationとForeign memoryを別の型付き拒否にする

## この検証が証明しないもの

- K1a ResourceLedger本体、RAII handle、owner別resident/pinned accounting
- decoder surface、参照面、先読みring、外部/imported memoryの会計
- allocator実測との誤差、backend固有allocation padding
- VRAM/RAM/disk間の昇格・降格、copy-out、eviction
- device lost、OOM、並行admission、実GPU性能
- 製品のAuto budget、VRAM比率、先読み深度、閾値

## 次の判定

**PASS**: checked見積りを製品download経路へ残し、次粒でK1a owner inventoryと外部memoryの
「数えられない場合の拒否」を閉じる。

**STOP**: このprivate modelだけをResourceLedger完成、実機メモリ量、K1c階層成立の証拠として扱わない。
