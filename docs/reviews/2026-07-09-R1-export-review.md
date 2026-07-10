# コードレビュー所見 2026-07-09 (R1/Quality・export・cli周辺)

対象: R1実装時点の未コミット作業ツリー(R2は対象外)。8観点ファインダー+検証パスで確定した10件。
修正したらチェックを入れ、修正コミットにこのファイルの項番を書くこと。

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
