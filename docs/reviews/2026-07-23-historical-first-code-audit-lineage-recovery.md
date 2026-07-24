# 第一コード監査lineageの価値回収（Unit 4C-3、2026-07-23）

状態: **観察**（cutoff 2 historical blobの処分完了）

対象: `docs/reviews/2026-07-11-code-audit-pre-m2.md`のcutoff全2版。

関連: [第一コード監査](2026-07-11-code-audit-pre-m2.md)、[M2入場条件](2026-07-11-M2-entry-gate.md)、[M3/M4 gate台帳](2026-07-12-M3-M4-gate-ledger.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

第一監査は対象SHA `f020ec8`の44ファイルを、plugin、時間、GPU/media、schema、色/座標、test施行の6面から点検した。第二版はmain進行直後にG-7、G-1、T-1の状態変化と対象SHAを追記しており、「監査結果を固定事実にせず採用時に現物確認する」という現在の回収規律の原型でもある。

2026-07-23の現行コードへ再照合すると、M2入場項目の大半は実装・試験済みである。一方、後続gateへ送られた候補の一部は、M2/M3の進捗表示に隠れて今も残る。価値回収は次の四処分とする。

- **実装済みを閉じる**: RenderCtx、typed param解決、registry purity、GPU必須CI、scaffold、未知plugin層別、時刻正準化、Document境界、色保存意味、canonical型、LayerId、test保護、GPU origin/health、FrameReader cancel。
- **残件を現行gateへ戻す**: format付きPipelineCache key、動的plugin ID寿命、raw duration、Value正準hash/区間query、refcount pool、VRAM生成口、1-frame encoder、Rec709変換点、PAR、GPU premul adapter、render-desc対応表、Encoder/YUV寿命診断。
- **縮小して残す**: category統制とpanic走査をPB-8一括完了としない。typed `ViewportTransform`とplugin公開面走査は成立したが、categoryは現在も自由文字列である。
- **復活させない**: Slint前提、ProjectV1増築、当時のfile:line、test総数、全項目をM2へ差し戻す案、具体的な古いAPI skeleton、ゴールデンを実装変更と同時に書き換える案。

## 2. 二版の処分

| blob | 変化 | 現在の判定 |
|---|---|---|
| `83bffb87` | 6面、P/T/G/F/C/Eの初回監査とPB/TM/GR/SC/CQ/EN/LG ticket案 | 成立理由とIDを保持。対象SHAのコード事実はarchive |
| `bdff5f23` | 対象SHA、main進行、G-7/G-1/T-1の同日訂正を追加 | **検証規律として保持**。最新main再照合なしの採用を禁止 |

## 3. 現行処分表

### 3.1 実装済み

| 群 | 現行証拠 |
|---|---|
| PB-1〜PB-5 | `RenderCtx`、`NodeDesc::resolve_params`、`assert_registry_pure`、`MOTOLII_REQUIRE_GPU=1`、scaffoldのpurity/golden/ParamDef |
| PB-9 | 未知／未来pluginのopen診断・execution拒否 |
| TM-1〜TM-5 | Document exportはtimeline frame起点、RationalTime/Fps/TimeMap正準化、半開duration、`try_to_frame_round` |
| GR-2/GR-5〜GR-7 | `GpuOrigin`のsync readback拒否、render/UI workerのhealth検査、stderr drain、FrameReader cancel/kill分離 |
| SC-1〜SC-3 | ProjectV1非継承、Document≠ExportJob、extra/min reader/revision、`LayerId` |
| CQ-1/CQ-3〜CQ-5 | 保存Color意味、canonical型、Draft aspect保全、planar cameraとrender入力 |
| EN-1〜EN-5/LG-1 | protected oracle、tolerance、CI pin、single-writer走査、proptest、semantic/provisional分類 |

TM-1について、legacy `export_overlay_video`はidentity以外を拒否したままだが、製品側`export_document_video`は`timeline_time → Document graph → source_time → source frame`へ反転済みである。古い入口の制約を製品export未実装と読み替えない。

### 3.2 部分または未実装

| ID | 現行事実 | 予定先／停止線 |
|---|---|---|
| PB-6 | `PluginContract.migrations`とprepared resolutionはあるが、descのID/version/param schema snapshot比較は無い | migration実装をschema版上げ忘れ検出まで完了した証拠にしない |
| PB-7 | `PipelineCacheKey`は`id+wgsl`のみでtarget formatを持たず、pipelineは`Rgba8Unorm`固定 | 公開runtimeとM4/M5 format解凍をまたぐ。場当たり的なstring key追加禁止 |
| PB-8 | `ViewportTransform`はtyped Result、plugin公開面panic scanもある。`NodeDesc.category`は非空だけを検査 | taxonomyをUI表示名や将来manifestと混ぜず個別に閉じる |
| PB-10 | `PluginId`/`ParamDef.id`/pipeline keyが`&'static str` | Unit 3Bのthird-party runtime停止線。先にString化してABI/package形式を発明しない |
| TM-6 | `MediaInfo.duration`は今もfps gridへ丸め、raw container durationを保持しない | audio/containerの真の終端利用者と同時に閉じる |
| TM-7 | `Value`の正準hash、`keys_in(range)`、`next_key_after`が無い | M4 cache key設計で-0/NaN/範囲意味を同時決定 |
| GR-1 | M3は独立display copyでtearingを防いだが、中間textureのrefcount poolは無い | M4 K1。UI対策をcache pool実装済みの証拠にしない |
| GR-3 | texture作成はGPU/nodes/render/UIへ分散し、VRAM計上口が一つでない | M4 K1 budget前。直接生成の全面置換を別境界にする |
| GR-4 | nodesにper-render uniform作成と複数encoder/submitが残る | 性能審判と1-frame ownershipを決めてから閉じる |
| CQ-2 | ColorSpace tagは増えたがYUV→renderはRec709 gammaをSrgb descとして扱う近似のまま | M5の一点色変換。数値変更と経路予約を混ぜない |
| CQ-6 | `FrameDesc`は6意味のままPAR/rotationなし。constructor/serde安全はGAP-17にも残る | PARをUI都合で追加せず、media/cache利用者と解凍 |
| CQ-7 | CPU色のpremultiply helperはあるが、Vello等のGPU texture境界用単一adapterは無い | K6/M5 textの先行条件 |
| CQ-8 | `validate_render_desc`はRGBA8+Srgbを直接比較する固定関数 | supported tuple表を公開APIへ先に焼かない |
| LG-2 | YuvToRgbaの2-convert寿命はcommentのみ。`Encoder::Drop`はfinish忘れを診断せずkillする | M4 job/lifetime境界で型または診断へする |

この表は一括実装指示ではない。IDは責任境界を保つために残し、それぞれの利用者と審判が揃った段階で解凍する。

## 4. 成立理由として残すもの

監査の最大の価値は個別修正案より、作者数またはplugin数で増幅する穴を先に塞ぐという判断軸にある。

1. plugin契約の破壊、silent fallback、opt-in検査は作者数で乗算する。したがって公開trait、typed error、登録時検査、scaffoldを先に閉じる。
2. cache key、色変換、texture寿命、IDは後続利用者ごとに別形式を作ると合流不能になる。利用者が揃うまで予約と停止線を維持する。
3. CIが緑でもGPU試験がskipされていれば審判は空である。依存欠落を赤にする仕組みそのものを試験する。
4. 監査のfile:lineは採用時に最新mainへ再照合する。途中で解消した指摘を残件数へ水増ししない。

これはcreator/developer連続体とも接続する。作者を増やす戦略は、作者全員へ注意力を要求するのではなく、誤りの乗算点をHostとCIで一度だけ閉じる時に強くなる。

## 5. 復活させないもの

- Slint API、当時の`ProjectV1`、旧render skeletonを現在の製品境界として復活させること。
- PB/TM/GR/CQを一つの「監査cleanup」PRへ束ねること。
- `PipelineCacheKey`へformat文字列だけを足し、layout/sample count/blend等のkey責任を未決のまま完成扱いすること。
- `&'static str`を一斉String化し、それをthird-party load/package/ABI成立と呼ぶこと。
- PAR/rotationをFrameDescへUI要望だけで追加し、serde/cache/plugin façadeへの影響を飛ばすこと。
- Rec709とsRGBの数値変換を既存golden更新で通すこと。
- `GpuCtx::create_texture`の名だけを作り、直接作成が残るままVRAM予算一元化を称すること。
- historical test数や当時のGPU skip結果を現在の品質証拠にすること。

## 6. 固定歴史出典とcoverage

初版`83bffb87`を全文で読み、第二版`bdff5f23`の差分を確認した。処分した2 blobの完全SHAは`evidence/historical-value-recovery/disposition-receipts/04c3-first-code-audit.tsv`を正本とする。cutoff総数1,797のうち処分済みは251、未処分は1,546である。
