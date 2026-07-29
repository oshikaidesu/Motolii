# M4 validation campaign checkpoint

状態: **観察 — 構造検証PASS / 製品hard budget・実機性能未完了**

基準: **この文書を含むcampaign checkpoint commit**

## 今回の問い

M3の背骨を待たずに、M4の正しさ、所有、階層、実機計測口をどこまで本番へ残せるか。
また、AviUtl系利用者を想定した低スペックWindowsで「緑のバーを気にしない」体験を
主張する前に、何を実機gateとして分離すべきか。

## 製品境界へ残したもの

- VRAM、RAM、diskと任意UMA shared capを持つ`ResourceLedger`
- admission前hard cap、型付き拒否、RAII grant、corruption時fail closed
- 複数allocationのatomic batch admission
- format、mip、sample、dimension、alignmentを含むchecked allocation estimator
- `RenderSession`の中間targetとtransparent/solid source textureの明示会計
- replacementを先にadmitし、拒否時に旧generation・pixel・usageを維持する規則
- raw GPU allocation seatのowner inventory guard

これらはcache hit率や性能閾値を仮定しない純粋な所有・会計基盤であり、
後続の実測値を差し替えても正しさ契約を変えない。

## testkitへ隔離したもの

- VRAM→RAM→disk transferのsource保持、二重常駐、LRU、pin/in-use、
  single-flight、cancel、stale generation負例
- size-keyed YUV lane plannerのatomic refusalとmixed-size再利用
- software／hardware-download decode需要matrix
- 1000短clip、最大active 4の音MAD編集密度fixture
- OS、CPU、RAM、GPU adapter、FFmpeg、RSSを記録するhardware facts
- 同じrecipeを別機種で再実行するmanifest schema v3と専用executor

test-only plannerやharnessを製品owner、公開API、Document、plugin契約へimportしない。

## 発見した停止線

1. **graph liveness**
   - 最大active 4でも未来inactive clipがframe 4で1015 stepsを作る
   - cacheで隠さずM3 graph-livenessへ渡す
2. **hardware-download**
   - 開発Macの720p fixtureではVideoToolbox→CPU downloadがsoftwareより全需要形で遅い
   - hardware codec有効化を高速routeと呼ばない
3. **YUV materialization**
   - 2面poolは3つ目のlive outputで1つ目を上書きする
   - product lifetime ownerとbudget入力前にlane plannerを昇格しない
4. **GPU surface import**
   - wgpu ExternalTextureはimport済みplane viewのlowering候補であり、
     OS decoder surfaceのimport境界ではない
   - native surface、backend import、lowering、fence、pixel oracleを別gateとする
5. **RenderedFrame**
   - publicなbare `wgpu::Texture`はgrantと同寿命にできない
   - private field＋`&wgpu::Texture` accessorだけでもraw cloneを保持できる
   - M3 deliveryとM4参照handleをまたぐ公開契約なので、反対側レビューなしに採択しない

2026-07-29にFable 5へ`RenderedFrame`所有案のread-only相談を試みたが、約3分応答がなく
中断した。model fallbackや未取得回答を証拠にせず、公開契約は停止線のまま残す。

## 再実行gate

manifest schema v3は全gateを`pending`で出力する。専用executorは同じcommit／manifestを
完全一致で確認し、既存artifactを上書きせずcommand単位の結果を保存する。

| gate | 必要な証拠 |
|---|---|
| `low_spec_windows` | 同じcommit／fixture／bundleを対象persona実機で実行 |
| `native_decoder_surface_import` | CPU raw pipeなしのsurfaceとdevice identity |
| `wgpu_external_texture_lowering` | import済みplaneと明示color descriptorからRGBA |
| `surface_lifetime_fence` | GPU完了前のpool再利用・grant解放を拒否 |
| `gpu_surface_pixel_oracle` | 同じsource/time/rotation/colorで宣言済み画素審判 |
| `product_preview_path` | decode、upload/import、render、display、cancel、queue depth |

未決policy値は引き続き`null`である。

- VRAM hard budget
- texture allocation alignment
- YUV live lane cap

## 発注へ切り替える境界

現在のdocs、純粋ロジック、harness調整はCodexが直接小粒で施工する。
次のどれかが閉じた時、正式なOpus 5→Spark→Grok発注を一粒ずつ使う。

1. `RenderedFrame`公開所有契約が反対側レビューを通り、変更fileと負例が閉じた
2. macOSまたはWindowsのnative decoder adapterについて、採択dependency、device identity、
   handle lifetime、変更許可file、pixel oracleが閉じた
3. 低スペックWindows bundleから数値policyを採択する独立仕様粒ができた

native adapterをmacOS／Windows同時発注しない。platform harness、製品adapter、
ResourceLedger接続、preview統合を一粒へ束ねない。

## 検証証跡

基準commitで次を通した。

```text
cargo test --workspace --locked
cargo check --target x86_64-pc-windows-gnu -p motolii-testkit --lib --bins --tests
cargo clippy -p motolii-testkit --all-targets -- -D warnings
./scripts/check-docs.sh
```

実bundleも生成し、`schema_version: 3`、command 6件、external gate 6件を確認する。
低スペックWindows実測と製品Previewは外部状態が必要な最終gateであり、このcheckpointで
合格へ変更しない。

## 2026-07-29 local full replay

clean commit `f321a5d87ce89e5cd93d4a723dc496e1ac5024f3`でbundleを新規生成し、
専用executorから6 commandを直列実行した。verifier結果は次のとおり。

```text
local_evidence_valid: true
verified_commands: 6
failures: 0
external_gates_pending: 6
```

softwareとVideoToolbox hardware-downloadは同じ3,517,051-byte fixture、
SHA-256 `9bc6ac659282be60c973d7ef292473b62f87cd85912e8eff5192dd0ef7cdb497`
を使用した。frame 0は1,382,400 bytes中差分0だった。

```text
software command sequential 120 frame: 91.89 ms
VideoToolbox download sequential 120 frame: 277.24 ms
software command parallel wall: 171.69 ms
VideoToolbox download parallel wall: 418.24 ms
```

これはlocal harnessの閉包証拠であり、hardware-downloadがsoftwareより遅いという従来観測を
同一fixture digestつきで再現しただけである。native GPU surface import、低スペックWindows、
製品Previewの性能を証明しない。
