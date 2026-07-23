# コードレビュー所見 2026-07-09 (R1/Quality・export・cli周辺)

対象: R1実装時点の未コミット作業ツリー(R2は対象外)。8観点ファインダー+検証パスで確定した10件。
修正したらチェックを入れ、修正コミットにこのファイルの項番を書くこと。

> **歴史回収（2026-07-23）**: cutoff全5版は[Unit 5A](2026-07-23-historical-r1-export-gpu-safety-lineage-recovery.md)で処分済み。本文の`[x]`は現行完成証拠ではない。初期10件の主要修復は概ね維持されるが、continuous stderr drain／verified atomic artifact／teardown timeoutはGAP-26、GPU error taxonomy／poison typed failureはGAP-27で追跡する。部分mp4を成果物として残す追補案は現行M1 G3により棄却する。

## 重大(データ喪失・クラッシュ)

- [x] **1. エクスポート中断でmp4が壊れる** — `crates/motolii-export/src/lib.rs:89`
  GPUエラーが`?`で伝播すると`encoder.finish()`(94行)未実行のままEncoderのDropがffmpegをkill(encode.rs:92-96)、moov未書き込みで既書き出し分が全滅。
  修正方針: ループをclosure/内部関数化しエラー時もfinish(またはabortを区別するclose)を必ず通す。部分書き出しを成果物として残すか削除するかも明示。

- [x] **2. 一度のGPUエラーで恒久故障** — `crates/motolii-gpu/src/ctx.rs:213`
  `GpuRuntimeState.device_lost/uncaptured_error`にクリア経路がなく、一過性のuncaptured error 1回で以後の`check_health()`が全部Err。
  修正方針: device_lostは恒久でよい(実際に恒久)が、uncaptured側は「取り出したらクリア」(take方式)にするか、明示的な`reset_uncaptured()`を用意。

- [x] **3. プロジェクトJSONの負のstart_frameでpanic** — `crates/motolii-export/src/lib.rs:48`
  `assert!(start_frame >= 0)`が公開APIにあり、`ProjectV1.start_frame`(範囲チェックなし)から直接届く(project.rs:75-85)。CLIのexport-overlay経路は守られているがexport-project経路は素通し。
  修正方針: assertを`ExportError::InvalidRequest`等の型付きエラーに変更。

## 高(性能規律違反 — performance-modelの「確保・解放を毎フレームやらない」)

- [x] **4. 毎フレームのシェーダ/パイプライン再生成** — `crates/motolii-nodes/src/lib.rs:186,232,373,392`
  `OverlayNode::new`/`CompositeNode::new`が`create_shader_module`+`create_render_pipeline`を実行し、motolii-renderが毎フレームnewする(render_frame_with_background_texture 148/160行、render_graph 207/228行)。エクスポート全体で数千回のシェーダコンパイル。
  修正方針: ノードを一度作って使い回す(パイプラインはノードが保持、per-frameはuniform更新のみ)。

- [x] **5. 5秒タイムアウトが設定不可・リトライなし** — `crates/motolii-gpu/src/transfer.rs:10`
  高負荷/サーマルスロットリング下の正当な遅延で所見1の経路に入りエクスポート全滅。
  修正方針: タイムアウトを呼び出し側から渡せるようにする(エクスポートは長め/無制限+進捗ログ、対話系は短め)。

- [x] **6. 不変の透明テクスチャを毎フレームCPU生成+再アップロード** — `crates/motolii-render/src/lib.rs:144`
  全面ゼロのVec確保+write_textureが毎フレーム走る。foreground/outputターゲットも毎回新規確保。
  修正方針: 同一寸法の間は再利用(RgbaDownloaderと同じパターン)。

## 中(正しさ・UX)

- [x] **7. プロジェクトJSONの相対パスがCWD基準** — `crates/motolii-cli/src/project.rs:71`
  プロジェクトファイルの隣の素材が、実行ディレクトリ次第で見つからない。
  修正方針: `project_path.parent()`とjoinして解決。

- [x] **8. 引数なし起動で「unknown command: /path/to/motolii-cli」** — `crates/motolii-cli/src/lib.rs:68`
  argv0ストリップが`len > 1`条件で素通り。修正方針: 呼び出し側で`env::args().skip(1)`を渡し、ヒューリスティック自体を削除。

- [x] **9. 共有デバイスへのハンドラ上書き(将来リスク)** — `crates/motolii-gpu/src/ctx.rs:92`
  wgpuのコールバックスロットはデバイスに1つ・置換動作。`from_device_queue`が無条件に上書きするため、M3でホスト側もハンドラを持つ構成と衝突。
  修正方針: docコメントに単一スロット制約を明記+M3統合時に登録の所有者を1箇所に決める。

## 低(アーキテクチャ整合)

- [x] **10. パイプライン形状が2本番経路で重複** — `crates/motolii-render/src/lib.rs:148-160 vs 207-228`
  Solid→Overlay→Compositeの手組みがrender_graphと外部背景経路で二重。将来の経路変更で静かに乖離(B-4)。
  修正方針: 外部背景をLinearRenderGraphのプロデューサとして表現し経路を1本化(チケットR4と同時にやると自然)。

## 見送り(キャップ外・軽微)

- `gpu_or_skip`×5コピペ→motolii-testkitへ集約 / `CliError::Usage`への文字列潰し→`#[from]`で構造維持 / `tmp_dir`/`make_test_video`の二重定義 / `RgbaDownloader`のDefault+new重複 / LayerSource足場が未配線かつ`LayerSourceContext`にQualityの口がない(R4/M5配線時に要対応)

---

## 追補(2026-07-11 再監査: 本レポートの抜けと記録上の問題)

本監査(2026-07-09)を後から検証した結果、**監査時点で存在したのに拾えなかった所見4件**と、**記録方法の問題4件**を記録する。各項目は origin/main `a3a05d5` と監査時系のツリー(`f020ec8`)の両方に対して file:line の現物確認済み。

### 見落としていた所見(R1監査時点で存在)

- [ ] **追-1【重大・現存】外部JSONから不正な時刻型を生成できる** — `crates/motolii-core/src/time.rs:10`(`RationalTime`)/ `:67`(`Fps`)
  両型とも`Deserialize`をderiveしており、コンストラクタの検証を迂回して`den = 0`や負のfpsをJSONから作れる。その後の除算・フレーム変換でpanicする(`den<0`は`Ord`/`Hash`/`Eq`のden>0・既約前提も静かに壊す)。所見#3「負のstart_frame」と同じ**外部入力境界**の問題であり、R1監査の対象内だった。Issue #19(入力起因panicのResult化、close済み)のスコープにも含まれず、**a3a05d5でも未修正**。
  修正方針: `try_new`+カスタムDeserialize(`#[serde(try_from = ...)]`)、またはプロジェクト読込時の再帰的validate。→ [2026-07-11-code-audit-pre-m2.md](2026-07-11-code-audit-pre-m2.md) 所見T-2 / チケットTM-2として起票済み
- [x] **追-2【高・修正済み】ffmpeg stderrパイプのデッドロックを見落としていた** — 監査時点の`encode.rs:78-89` / `decode.rs:101-111`(stderrをpipeしたまま先に`wait()`)
  ffmpegが大量出力するとstderrパイプ(通常64KB)が満杯になり子プロセスが終了できず、`wait()`で永久ブロックする古典的デッドロック。**まさにexport/media周辺で監査時点にも存在しており、本レポートの明示的な見落とし**。後日Issue #20として発見され、PR #30(2026-07-11 マージ、`read_child_stderr`ヘルパー: 64KiB上限+読み切り→`wait()`順序+モックffmpegのタイムアウト付きテスト)で修正済み。監査履歴の正確性のためここに記録する
- [ ] **追-3【中・現存】`Encoder::write_frame()`をfinish後に呼ぶとpanic** — `crates/motolii-media/src/encode.rs:87`(a3a05d5)
  `self.stdin.as_mut().expect("encoder already finished")`が残存。Issue #19が「外部入力panicのResult化」を完了扱いにした後もこの経路はpanicのままで、一貫性を欠く。公開APIとしては`MediaError::EncoderAlreadyFinished`を返すべき。呼び出し元が1箇所の今は低リスクだが、M4のジョブ化で呼び出しが増える前に対処(同ファイルのDrop時finish忘れ警告は[audit G-8](2026-07-11-code-audit-pre-m2.md)参照)
- [ ] **追-4【中・現存】Mutex poisoningでGPUエラー処理自体がpanicする** — `crates/motolii-gpu/src/ctx.rs:111, 204, 213`(a3a05d5)
  `check_health()`とGPUコールバック内に`lock().expect("GPU runtime state poisoned")`が残存。GPU障害を型付きエラーに集約する設計なのに、**監視状態のpoisonで防御機構そのものがpanicへ逆戻り**する。最低でも回復不能な`GpuRuntimeError`として返す(poisonは「エラー処理中のどこかがpanicした」印であり、二重panicは診断を悪化させる)

### 記録上の問題(以後の監査プロセスへ反映)

1. **所見#2の修正方針が危うい**: 現行実装(`ctx.rs:118`)はuncaptured GPU errorを一度報告したら単純に`take()`して続行する。Validation / OutOfMemory / Internal を一律「一過性」とみなす根拠はなく、エラー種別ごとに「継続可能/フレーム失敗/device再生成」を分けるべき。M3プレビュー統合(読み戻しなし=事故が画面にしか出ない)の前に要判断 — [audit G-6](2026-07-11-code-audit-pre-m2.md)と同じ棚
2. **全項目が[x]なのに修正証跡がない**: 冒頭に「修正コミットに項番を書く」とあるが、レポート側に修正コミット・PR・確認テストへの逆リンクがなく、何を根拠に完了扱いにしたか後から追えない。以後のレビュー文書は**チェックを入れる際にコミットSHA/PR番号を項目へ併記**する(freeze-gate-remaining.mdはこの形式を守れている — テスト名併記)
3. **所見#1の完了条件が閉じていない**: 「部分ファイルを残すか削除するか明示」に対し実装は再生可能な部分mp4を残す方針だが、`Err`に出力パスや`frames_written`が含まれず、呼び出し側が「利用可能な部分成果物が存在する」と判断できない。エラー型への文脈追加をM2-D6(書き出しmux)前に
4. **対象時点が再現不能**: 「R1実装時点の未コミット作業ツリー」が対象で、監査対象SHAを復元できない。**以後の監査レポートには対象commit SHA・対象範囲・実行したテストを必記**する(本追補と[2026-07-11-code-audit-pre-m2.md](2026-07-11-code-audit-pre-m2.md)から適用)
