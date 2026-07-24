# R1 export／GPU safety lineageの価値回収（Unit 5A、2026-07-23）

状態: **実装済み項目と未到達hardeningを再分別**（cutoff 5 historical blobの処分完了）

対象: [R1 export review](2026-07-09-R1-export-review.md)のcutoff全5版。

関連: [M1仕様](../specs/M1-vertical-slice.md)、[pre-M2 code audit](2026-07-11-code-audit-pre-m2.md)、[M3 GPU/runtime境界](../specs/M3-ui-integration.md)、[backlog](../backlog.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

初版の10所見は、未完了→全完了、`oc-*`→`motoly-*`→`motolii-*`の三段を経た。最終版はさらに監査漏れ4件と記録上の問題4件を加えた。名称変更2版は意味変更ではない。

現行コード照合では、初期10件の主要修復は概ね残っている。しかし「[x]」だけで完全成立とは言えず、後続M1 G1〜G8が要求するprocess/artifact reliabilityとGPU health分類は未到達である。M1 vertical sliceの完了を巻き戻さず、出荷hardeningをGAP-26／27へ分離する。

## 2. 初期10所見の現行処分

| # | 現在地 | 処分 |
|---|---|---|
| 1 error時finish | exportのoverlay／Document loopはloop errorを保持し`Encoder::finish()`後に返す | **実装済み**。render/download/write失敗でもfinishを通る直接負例は不足 |
| 2 uncaptured一回復帰 | `take()`で一度返した後は復帰 | **部分実装**。全errorを文字列一種へ潰し、一律一過性とする分類不足はGAP-27 |
| 3 negative start panic | typed `InvalidRequest` | **実装済み** |
| 4 per-frame pipeline生成 | export loopは長寿命`RenderSession`／cached graphを使用 | **実装済み**。単発APIをloopへ戻さない |
| 5 fixed 5秒timeout | downloaderがcaller timeoutを受け、exportは300秒 | **実装済み範囲あり**。retry／進捗providerは別要件 |
| 6 transparent／RT再生成 | transparent／Solid cache、中間RT pool | **実装済み**。最終frameの独立textureはM3寿命契約上必要でpool aliasへ戻さない |
| 7 relative path | project隣接基準 | **実装済み** |
| 8 argv0 | `args().skip(1)` | **実装済み** |
| 9 callback slot | `GpuCtx`が唯一の登録者というdoc／M3契約 | **境界決定・実装済み** |
| 10 render二経路 | external backgroundをgraphへ接続しcached renderへ統合 | **実装済み** |

## 3. 追補4所見の再判定

### 3.1 RationalTime／Fps Deserialize

custom Deserializeから`try_new`を通す現行実装と負例があり、修正済みである。歴史時点の「現存」を現在へ外挿しない。

### 3.2 ffmpeg stderr deadlock

現行はstdinを閉じた後、`wait()`より先にstderrをEOFまで読む。これは「wait先行」の古いdeadlockを直すが、G1の**処理中から専用threadで常時drain**ではない。ffmpegがframe入力中に大量stderrを出すとpipeが満ち、childがstdinを読めず、parentの`write_frame`も進まない相互停止が残る。

既存fixtureは1 frameを受け取った後にstderr floodし、`finish()`だけを別threadでtimeout監視する。長いstdin書込みと同時のstderr floodを再現しないため、G1完成証拠にしない。GAP-26でcontinuous drainとbounded diagnostic bufferを閉じる。

### 3.3 finish後write panic

`write_frame`内の`expect("encoder already finished")`は残るが、`finish(self)`がEncoderを所有権ごと消費する。safe Rustの公開経路から同じ値でfinish後にwriteすることはできない。歴史追-3は到達不能な誤所見として棄却し、`EncoderAlreadyFinished` variantを追加しない。

### 3.4 GPU mutex poison

`check_health`とdevice callbackに`lock().expect(...)`が残り、監視状態poison時はtyped GPU errorへ入らずpanicする。uncaptured errorもwgpuのValidation／OutOfMemory／Internal等を受けた時点で文字列化するため、継続、frame失敗、device再生成、停止を分類できない。GAP-27でtyped fatal／recoverable境界を閉じる。

## 4. 現行M1 G1〜G8との照合

| Guard | 2026-07-23 live判定 |
|---|---|
| G1 continuous stderr drain | **未実装**。finish時drainだけ |
| G2 ffprobe output verification | **未実装**。T9試験が特定fixtureを検証することと全export成功条件は別 |
| G3 temp→verify→atomic rename／no partial | **未実装**。overlayと音声なしDocument exportは最終pathを直接開く |
| G4 timeout付きDrop teardown | **未実装**。kill後`wait()`にtimeoutなし |
| G5 startup version/capability probe | **部分実装**。`verify_tool_versions()`はあるがCLI起動は存在確認警告だけ |
| G6 shell禁止＋Unicode path E2E | argv起動は成立、要求された全対象OS Unicode E2Eは未確認 |
| G7 bounded in-flight | 現行同期1-frame処理で実質bounded。将来pipeline化時に維持 |
| G8 pinned lavapipe＋全GPU timeout | runner imageは固定だがpackage versionはログのみ、全GPU per-test timeoutは未成立 |

G1〜G8は元から「M2以降の増強チケット候補」で、M1 exit demoの完了条件ではない。未到達を理由にM1の歴史的完了を撤回せず、出荷準備の現在地として追跡する。

## 5. artifact方針の更新

歴史追補は再生可能な部分mp4を残し、errorへpath／`frames_written`を載せる方向を述べた。その後のM1 G3は、tempへ書き、検証後だけatomic renameし、中断時は部分fileを最終pathへ残さない方針を正本化した。現在はG3を優先する。

現行exportは最終pathを直接`-y`で開く経路があり、既存の正常成果物を開始時に上書きし得る。loop errorでもfinishするため部分動画が残る。これは歴史所見1の「finishは通す」を満たしても、現行artifact契約を満たさない。GAP-26ではfinish保証とno-partial installを同時に審判する。

## 6. GAP-26／27の再入場境界

### GAP-26 ffmpeg process／artifact reliability

実装は一括PRにせず、(a) continuous stderr drain、(b) output verification＋temp/atomic install、(c) teardown timeout、(d) startup capability probeの順に閉じる。各段で既存正常export、音声mux、色tag、preview/export同一関数を変えない。SIGKILL／write failure／stderr flood／exit 0不正output／既存final保持を負例にする。

### GAP-27 GPU health taxonomy

wgpu errorを文字列化する前に分類し、recoverable frame failure、device lost／OOM等のfatal、再生成可能性を決める。poisonをpanicへ戻さずtyped fatalにする。callback ownerは`GpuCtx`一つのまま、UI toolkitやpluginへcallback slotを公開しない。visual thresholdやgoldenをGPU errorに合わせて変更しない。

## 7. 復活させないもの

- historical `[x]`をcommit／test逆リンクなしで現行完成証拠にすること。
- 部分mp4 path／`frames_written`を公開成果物契約へ戻すこと。
- finish時stderr読取だけをcontinuous drainと呼ぶこと。
- `finish(self)`後write用の到達不能error variantを追加すること。
- uncaptured GPU errorを全種一過性として無条件復帰すること。
- 最終`RenderedFrame`を中間pool aliasへ戻すこと。
- 単発`render_frame`をexport loopの正規入口へ戻すこと。
- G1〜G8未到達を理由にM1 exit demoの成立履歴を改変すること。

## 8. 固定歴史出典とcoverage

初版`43e7ab17`と最終追補`0ec3ad11`を全文で読み、中間3版の全差分（checkbox、二回の名称変更）を確認した。処分した5 unique blob（29,700 bytes）の完全SHAは`evidence/historical-value-recovery/disposition-receipts/05a-r1-export-gpu-safety.tsv`を正本とする。cutoff総数1,797のうち処分済みは342、未処分は1,455である。
