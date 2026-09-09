# 共有反射の内容cacheと比較

状態: **縮小採用**。2026-09-09利用者「その実装に進もう。比較結果まで算出」。[R0/R1計画](2026-09-09-qualiarts-rendering-next-plan.md)の範囲。基準rootは`c7c05dab`、forkは`e71fbcd6a7345833b5827f7655b458c929fce1a0`。開始時に別タスクの計画文書と3索引/採用文書が未commitで存在し、これらの変更は本施工から分離する。

## 借りる機構と今回の境界

QualiArtsの一次資料と転移条件は上記計画を参照。入力一致による再利用は既存Engineの素材/形状cacheを先例に、re_rendererのTexturePoolのhandleとimport所有権を確認した。poolが保持していても外部所有textureの内容は変わり得るので、forkにimport判定の読出しを追加した。

GPU時刻は[wgpu 29 CommandEncoder](https://docs.rs/wgpu/29.0.0/wgpu/struct.CommandEncoder.html#method.write_timestamp)のtimestamp queryと[Queueのtimestamp period](https://docs.rs/wgpu/29.0.0/wgpu/struct.Queue.html#method.get_timestamp_period)を使う。対応するheadless deviceだけfeatureを有効化し、比較時だけqueryを挿入。製品の通常描画にreadback待ちを増やさない。

cacheはCompositor全体で直前の1組のみ。sceneごとの無制限な保持はしない。入れ子で交互に別sceneを評価するとmissになるが、追加entryは作らない。入力画像128MiB、入力数16384、入力/保持shaderのmetadata概算8MiBを上限とし、不明な外部画像・点群はbypass。既存のcapture atlasと6面（face最大512）は1組を使う。保持bytesはcacheが参照を延命する入力画像の論理bytesであり、GPU全体の実占有量ではない。

keyは評価済み配置・projection camera・coverage/opacity・clip・material/field parameter・program・catalog世代・素材upload revision・環境画像/変換/強さ・comp寸法。時間そのものを鍵にせず、評価された値を比べる。probe位置と範囲はこれらから決まる。主カメラで作者の作品値を書き換えない。hashだけを一致判定に使わない。

## 比較方法

`reflection_compare`は既存の1280×720 Repeater作品を使用。cache off/onの2つのEngineを同一Document・時刻で交互順に実行し、全frameの全画素一致を必須にする。静止と毎frameの受け手rotation変更をそれぞれwarmup3回＋20sampleで比較する。

CPU preparation（総時間から下記3区間を除いたもの）、submit、GPU完了待ち、readback回収を分離。GPU timestampは最後のsubmitted batch内（反射・合成・screenshot転送を含む）で、先行upload/submitは含まない。GPU時間とCPU待ち時間は重なるため加算しない。通常のexport描画と同じreadback経路で比較するが、実窓FPSの測定ではない。

## 結果と採否

**1組の完全入力一致cacheを既定で採用する。平面反射・SSR・PPRは今回入れない。** 最終forkは`530bdf86907e6e3e0599938c12879815e0a012dc`。raw比較は最終コードの同じbinaryを使った独立2回の起動、各mode合計40sample。warmupを含む**276組・552frameの全画素一致**を確認した。

| 個数 | 静止 cache off → on | 総時間削減 | 動的 cache off → on | 動的key判定 |
| --- | ---: | ---: | ---: | ---: |
| 10 | 9.435 → 5.569 ms | 41.0% | 9.431 → 9.510 ms | 0.007 ms |
| 100 | 9.786 → 5.203 ms | 46.8% | 9.608 → 9.761 ms | 0.010 ms |
| 1000 | 22.384 → 11.407 ms | 49.0% | 28.184 → 21.566 ms | 0.052 ms |

すべて総時間の中央値。静止では12面撮影が0回へ減る。1000個のCPU preparation中央値も15.651→7.920msへ減った。動的では双方12面撮影であり、表の時間差をcacheの高速化とは解釈しない。対応する同frameの時間差（on−off）の参考bootstrap 95%区間は、10個 −0.069〜+0.324ms、100個 −0.378〜+0.531ms、1000個 −7.543〜+0.579msで、いずれも0を含む。この機器・作品・sample数で明確な動的悪化は確認できなかったが、動的作品を速くする機構ではない。機械負荷による待機時間の振れが大きく、これを一般的な性能保証や厳密な統計保証にしない。

入力textureの保持は今回の作品で18,939,904 bytes（既存素材への参照延命）。1組のatlas＋6面は上限face512で約22MiB、入力metadata上限は概算8MiB。これらはcacheの論理上限であり、re_renderer全体のpool・staging・ドライバの総VRAM上限ではない。cacheの上限超過は再撮影へ戻り、画の品質を変えない。

[集計JSON](assets/2026-09-09-reflection-cache/summary.json)・[再集計script](assets/2026-09-09-reflection-cache/summarize.py)。同directoryに全raw sampleを保持する。比較作品は[既存生成器](assets/2026-09-09-shared-reflections/create_scene.py)で作る。

```sh
cargo build -p motolii-render --example reflection_compare
motolii/target/debug/examples/reflection_compare /path/to/repeater-1000.rrd /path/to/result.json
python3 docs/reviews/assets/2026-09-09-reflection-cache/summarize.py
```

### 計時の制約と、比較前に直した問題

GPU timestampは480sample中75sampleで終了値が開始値を下回った。絶対値化して補正せず`non_monotonic_timestamp`として欠測にする。wgpuはencoder内の時刻とcommandの順序保証に制約があると明記している。今回のMetal経路のGPU列は**全体として信頼できる費用の定規にせず**、raw診断だけに残した。採否は全画素一致、総時間、CPU区間、撮影数で判断した。CPU待ち時間をGPU時間と呼ばない。

readback経路はflush時とreadback回収前で二度`begin_frame`を呼び、静止画像のtexture lookupを失効させていた。上流`GpuReadbackBelt::readback_next_available`自身が完了chunkを回収することを確認し、後者の不要なframe進行を削除した。この修正はoff/on双方へ適用したため、表は**この修正後における内容cache単体の比較**であり、旧commitとの全差分比較ではない。

素材の同一性はrendererへ届いたuploadのrevision/保持handleで判定する。既存の素材cacheへ同じpathのファイル更新を自動検知させる拡張は、この施工に含めていない。

## 検証

- render suite **52/52通過**。最後の同一program連続判定の軽量化後もcache比較testを再実行して通過。
- 追加検証はmaterial、配置・回転、sender opacity、field追加/evolution、Undo/Redo、明示evictionでcache off/onを全画素比較。同一入力のhitを確認。blurによる外部所有の中間画像はbypassし、古い像を再利用しない。
- 既存の画面外sender、カメラ往復、透過順序、clip、preview/export一致も通過。
- `cargo clippy -p motolii-render --all-features --tests --examples`と最終native build完走（warningあり）。owned-budgetは既存testと同じ全Rust source計数で全pattern一致。Stage5検査通過。
- docs checkerの既存「比率 aspect」行の状態`未決`は別タスクの裁定として変更しない。hygieneは旧UIの2804行/長大file23件で不通過。build後のincremental sessionも69で上限60を超えた。cacheを削除して数だけを合わせることはしていない。
- 最終nativeの実窓で1000複製と反射を表示し、roughness 0.02→0.30の編集が届くことを確認した。続くUndoメニュー操作中、Flutterの`AccessibilityBridge::CreateRemoveReparentedNodesUpdate()`でSIGSEGV。実装前の同日08:14の記録と例外・先頭3frameが一致した。[クラッシュ比較抜粋](assets/2026-09-09-reflection-cache/window-crash-comparison.json)。Rust panicやGPU validation errorはログに無かった。既存Flutter問題の再発と判断し、UI/SDKの修正へ範囲を広げていない。**今回の実窓Undo検収は未完**。Engine側のUndo/Redo・cache off/on画素比較は通過している。
- QA作品の変更は保存していない。元作品は前回と同じ`~/.local/state/motolii-stage5/checkpoints/shared-reflection-2026-09-09/motolii-ux-current-2026-09-05.rrd`から復帰し、frame 0のRectangle/Textと未変更状態を実窓で確認した。元のDocumentsファイルはそのまま。再起動により一時的なUI状態は初期化された。
