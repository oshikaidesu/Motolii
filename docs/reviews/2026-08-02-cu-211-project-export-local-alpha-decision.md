# CU-211 — Local Alpha Project Save / reopen / Export 接続決定

状態: **縮小採用**（2026-08-02）

## 目的

固定Mac Local Alphaの通常製品windowで、CU-210Rの再生後に作品を保存し、同じproject identityから再入場し、同じ製品面から書き出せる一続きの接続を閉じる。

## 既存target

- Save: `DocumentEditRuntime::save_checkpoint` → 既存 `ProjectSession::save_with_journal` の checkpoint 経路
- reopen: shell の既存 `ProjectSession::open`／journal recovery（別プロセス再入場。in-process session交換は持ち込まない）
- Export: 既存 `motolii-export::ExportJob` → `export_document_video` → headless `GpuCtx`／first-party runtime
- 製品面: 既存 Stage React source asset → strict Host IPC → `ProductApp` が唯一の状態所有者

## 採用範囲

- Stage transportへ `保存` / `書き出し` の typed intent を追加する。
- Saveは現在のproject identityへの明示checkpointとし、Save-As・file dialog・新しいDocument形式は追加しない。
- Exportは製品event loopを止めないheadless workerで実行し、`READY → EXPORTING → EXPORTED`（または `EXPORT ERROR`）をStageへ投影する。
- Export先は現在のproject pathと同じdirectoryの`<stem>-export.mp4`。既存pathがあれば成功したtempだけをatomic replaceする。
- Rectangle-only作品はExportJob既存のDocument frame graphをHD（1920×1080）へ解決する。これはLocal Alphaの非永続出力既定値であり、Documentへ焼かない。

## 非目標 / 残件

- Save-As、native file dialog、in-process reopen、export settings、cancel、progress percentage、mixed AudioProgram／Soundtrackの製品検収はこの粒の完了条件に含めない。
- P08-C1/C2およびP12-C1全体を完了扱いにせず、Local Alphaの接続証拠だけを縮小採用とする。
- Export workerはDocumentを書き換えず、UI共有GPUをreadbackへ使わない。

## 必須負例・検証

- Host action messageは`deny_unknown_fields`、bounded inbox、未知kind拒否であること。
- Exportは成功してから最終pathをrenameし、encode／mux失敗時にfinal artifactを作らない。
- Save成功後に同じproject pathを再openし、Rectangle／keyframe／EasingのDocument stateが復元すること。
- 通常製品windowで `保存 → 再入場 → 書き出し` を行い、生成MP4を`ffprobe`で検査すること。

## 検収記録

- Opus 5（`claude-opus-5`、`effort=low`、fresh read-only session）は同一diffをbounded reviewし、初回のP1（Save失敗表示／編集後status残留／配置経路のstatus残留）を局所修正後、`VERDICT: ACCEPT`、P0/P1なしで閉じた。
- `cargo clippy -p motolii-ui --all-targets -- -D warnings`、Stage Host test、`cargo check -p motolii-ui`、`cargo test -p motolii-export --lib`、`npm run check:host`、`./scripts/check-docs.sh`、`git diff --check`を通過した。
- 通常製品windowで `SAVED`、同一path再入場後のRectangle／keyframe 2本、`EXPORTING → EXPORTED 300 frames`を確認し、出力はffprobeでH.264／1920×1080／300 frames／10.0秒だった。

## ルート運用

通常粒のOpusは速度優先のbounded read-only検収（low effort、同一diff、最大一回の再確認）とする。runner、AGENTS.md、route契約、モデル順序は変更しない。
