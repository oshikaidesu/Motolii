# 状態返信契約の修正 — 2026-09-09

根拠はframe-pipeline-measurementsとresponsive-editor-architecture-audit。対象はui/native、Dart session、Swift host。GPU描画、同期、画質は変更しない。

実施順:
1. 読取snapshotを作品のdisplay revision・時刻・view条件・描画資源世代で再利用。保存差分用のsignatureはDocument revisionで再利用する。
2. 共通参照情報と作品snapshotに受信済みIDを持たせる。未受信側は完全な情報を得られ、通常返信は変わった情報とsession状態だけ。新しい窓のattachはbootstrap扱い。
3. nativeが実際の画像入力変更を返す。画像が必要な命令では前段の全状態生成を省き、描画後に送る。選択がpreviewを取り消した場合は描き直す。
4. deltaをSwiftの共有状態と各Dart sessionで合成する。legacyの完全返信・再生用liveLayers・未対応mockも扱う。
5. native契約テスト、Dart sessionテスト、build、同一書類ベンチと実窓操作。

維持条件: Document/Undoの正本は一つ。キャッシュは読取派生物。選択、下書き取消、保存/Undo、時刻変更、観測カメラ変更、初回描画後のbounds、新規書類、別窓bootstrapを確認する。古いIDを送ったクライアントへ省略した情報を渡さない。

作業前の対象4ファイルとnative libraryを `/tmp/motolii-status-contract-20260909/before/` に保存した。既存の未コミット変更があり、継続して別の編集も行われているため、対象差分のみを追記する。


## 実装結果

- `snapshot_cache.rs`: 読取snapshotを再利用し、`snapshotId`と`referenceId`で受信済み情報を省く。共通情報にはassetsも含む。書類identity・display revision・時刻・view条件・render countを使い、下書き取消や初回描画後のboundsを無効化する。保存差分signatureは書類identityとDocument revisionで再利用し、保存済みsignatureとの比較は毎回行う。
- FFI: `deferSnapshot`により画像入力が変わった命令は`needsRender:true`だけを先に返す。描画後に新しい状態を返す。変更がない場合は小さいsession状態を返す。
- Dart/Swift: 受信IDは編集命令とは別のtransport contextで渡す。Swiftは共有状態へdeltaを合成する。Dartは受信済み情報を保持し、nativeの再描画判定を使う。未対応のmock/従来返信には従来判定を残した。
- `contentRevision`で画像と下書き・viewの整合性を確認する。画像を変更しない選択返信は、表示済み結果の選択情報も更新する。
- UI側に利用箇所がないことを検索で確認した`blendPreviews`の生成を除去した。実際のBlend見本の新しいvisual-sample経路は変更していない。

## 検証

- native全suite: **41 passed / 1 ignored**。ignoredは既存の手動計測probe。
- Dart: 新規snapshot契約、既存visual selection、再接続、Stage observer/mountで**10 passed**。
- native build、Flutter/macOS Debug build、Stage 5 workspace check、diff whitespace check: passed。
- 更新版を再起動し同じ書類を開いた。実窓で選択変更、Inspector/選択枠更新、位置ドラッグ（X -485.00 → -464.27）、EditメニューのUndoで-485.00と保存済み表示へ復帰、再生のフレーム進行、停止、フレーム0へのシークを確認。検証操作はUndoし、書類は保存していない。
- Command-Zと数値欄のdouble clickはこの操作条件では反応を確認できず、Editメニューとドラッグで検証した。これらの入力挙動を今回直したとは言わない。

## 同一書類の比較

ビルド終了後に固定した更新後libraryと修正前libraryで比較。各40回、warmup 5回。CPU側のnative要求処理のみ。初期のビルド中計測は大きく変動したため比較値に使わない。

| 項目 | 修正前 | 修正後 |
|---|---:|---:|
| 選択返信の中央値 | 10.720ms | 0.864ms |
| 選択返信のp95 | 11.426ms | 0.916ms |
| 選択返信サイズ | 287,076 B | 1,154 B |
| 受信済み状態の問い合わせ中央値 | 9.976ms | 0.829ms |
| 完全状態返信の中央値 | 10.719ms | 1.911ms |

選択返信中央値は約92%短縮、サイズは約99.6%削減。これはFPSや全アプリの応答時間の改善率ではない。更新後の選択返信はneedsRender:falseで、native renderCountは増加していない。

[修正前](assets/2026-09-09-status-contract/before.json) / [修正後](assets/2026-09-09-status-contract/after.json) / [条件とlibrary hash](assets/2026-09-09-status-contract/conditions.json)。実行scriptとログは `/tmp/motolii-status-contract-20260909/`。

## 範囲と残り

Document変更の無効化はまだ保守的で、依存するlayerだけを再評価する一般的な仕組みではない。再生の評価共有、GPU完了待ち、Metal内部のcommand-buffer分割は今回変更していない。共通情報はsnapshot再生成時に比較するため、外部asset/font/catalogの変更検知全般を新たに実装したものでもない。

同時更新で検証を止めたcompile/境界違反は狭く修正した。text_formatの非公開module経由importを既存公開re-exportへ、PropertyId生成のStringを参照へ変更。新規error reportingのmain→bridge直接呼出しは既存session境界へ移した。既存の機能変更は戻していない。
