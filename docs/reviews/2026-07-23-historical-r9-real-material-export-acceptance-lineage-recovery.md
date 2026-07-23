# R9実素材／書き出し受入の価値回収（Unit 5H、2026-07-23）

状態: **M1歴史sign-off維持／現行製品release受入は未成立**

対象: [R9実素材検証チェックリスト](2026-07-10-R9-real-material-checklist.md) cutoff全4版（11,273 bytes）

関連: [M1仕様](../specs/M1-vertical-slice.md)、[Unit 5E 色export回収](2026-07-23-historical-color-export-lineage-recovery.md)、[backlog](../backlog.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

R9は2026-07-10時点のM1出口について、合成fixtureの数値比較と、local 1080p／4K素材をQuickTimeで見る人間確認を別々に行った**歴史的milestone sign-off**である。この完了表示は維持する。

ただし素材bytes、hash、codec、command、tool／player／OS／GPU versionが固定されず、GUIはSlint＋M1 `ProjectV1`のstandalone spikeである。したがって現在のReact chrome＋native Stage、現行Document、配布codec、release matrixの受入証拠には再利用できない。後継の製品受入はGAP-32へ分離する。

## 2. 4版の差分と処分

4版の受入意味は同じで、CLI renameと凍結gateへのlink更新だけが変化した。並行最終版を優劣で捨てず、同一sign-off lineageとして全版を処分する。

| 歴史主張 | 処分 |
|---|---|
| 機械判定と主観判定の双方を要求 | **維持**。役割を分け、片方で他方を上書きしない |
| 640×360 synthetic smokeは実寸品質を判定しない | **維持**。配線smokeと実素材品質を分離する |
| qp0／yuv444p比較と配布用yuv420p再生を分ける | **維持**。検証codecと配布互換を同じ合否にしない |
| 実素材をrepositoryへcommitしない | **維持**。将来はlicensed／user-supplied manifestでprovenanceを固定する |
| QuickTimeでqp0が開かなかった | **観察として維持**。単一時点・単一playerであり普遍規則にしない |
| mismatch時にtolerance 16〜24を試す | **撤回**。可変閾値でacceptanceを通さず、失敗は原因分離へ戻す |
| 自動fail後もGUIを開く | **診断用に縮小維持**。目視で自動failを合格へ変更しない |

## 3. 現行コードとの照合

- `spikes/r9-preview`は現存するが、Slint 1.17とM1 `ProjectV1`を使う隔離spikeで、現行製品Stageではない。
- `verify-b4`はProjectV1を書き出し、先頭／中間／末尾3 frameの最大channel差をcaller指定`u32 tolerance`で判定する。既定8、`r9-smoke.sh`は24を注入する。
- CI fixtureは32×24、6 frame、grayscale寄りで、検査は3 frame、overlay alphaは0である。visible composite、chroma edge、1080p／4Kの証明ではない。
- 現行Documentにはpreview相当とexport roundtripを同じgraphで比較するD3e fixtureがあるが、test自身が製品Preview entrypoint未成立を明記する。
- pre-encodeの同一評価、RGB→YUV、codec roundtrip、player表示は異なる境界である。GAP-5／29／31をR9後継へ吸収しない。

## 4. GAP-32再入場条件

1. 現行Documentと製品Stage entrypointを使い、M1 ProjectV1／Slint spikeへ戻らない。
2. pre-encode同一評価はexactまたはspec-owned semantic oracle、encode／decode lossはcodec／pixel format別の固定metricで判定する。
3. user可変acceptance toleranceを廃し、diagnostic値と合否権限を分離する。
4. licensed／user-local manifestに素材hash、codec、resolution、duration、fixture feature、command、Motolii／ffmpeg／player／OS／GPU versionを記録する。
5. grayscale、chroma edge、visible alpha overlay、motion、rotation、range端を小さなmatrixへ含める。
6. 自動結果とhuman playback recordを別欄に保存し、どちらかのfailを他方で上書きしない。

## 5. 復活させないもの

- 2026-07-10のprivate file名、local path、人間sign-offを現行release証明にすること。
- `spikes/r9-preview`、Slint、ProjectV1を製品Stage authorityへ戻すこと。
- `--tolerance 16`〜`24`や環境変数でacceptanceを通すこと。
- max差だけ、3 frameだけ、alpha 0だけのfixtureを合成品質全体のoracleにすること。
- qp0／yuv444pのQuickTime観察を全player／全versionへ外挿すること。
- codec差に合わせてsemantic golden、tol定数、期待値を書き換えること。
- GAP-5、29、31と製品Stage／release受入を一発注へ束ねること。
- privateまたは権利不明の実素材をrepositoryへ追加すること。

## 6. 固定証跡とcoverage

4 blobの完全SHAはreceipt `05h-r9-real-material-export-acceptance.tsv`を正本とする。合計11,273 bytes。

本Unit後のstrict progressは364 / 1,797（20.3%）、未処分1,433である。
