# M4: キャッシュ層と解析の拡充

ステータス: **ドラフト**(凍結ゲートで確定)

## 目的(退治する落とし穴)

B-5(時系列依存とキャッシュ)、B-6(OpenCV依存)、F-2(キャッシュの並行契約 — Natronがデッドロックした正確なポイント)。

## 方針

- キャッシュキー = ノードID × 時間区間 × 入力パラメータハッシュ **× Quality/出力記述子(2026-07-10追記)**。時系列解析ノードは**区間単位**のキャッシュエントリ(フレーム単位ダーティフラグでは表現しない)
  - Draft/Finalは同一関数の引数違い(B-4)だが出力ピクセルは異なるため、**Qualityと出力FrameDesc(解像度スケール・精度)はキーの一部**。Draftキャッシュ(fp16・半解像度)とFinalキャッシュは別エントリであり、混ざるとB-4検証が嘘をつく。ただしQuality非依存な成果物(解析DataTrack、シミュレーションのStateTrack — [simulation-model.md](../simulation-model.md)§3.3)はキーにQualityを含めず、両品質から共有する
- **キャッシュの前提(2026-07-10)**: 全レイヤー/ノードは最終的に2D合成パイプライン上の出力。`render(t)`が**前フレームの内部状態に依存しない**純関数であること([concept.md](../concept.md)の横断決定)。逐次依存シミュレーションは**レンダ経路には存在しない** — 逐次状態はレンダ外のベイク境界のStateTrackとして確定し、レンダはそれを読むだけ([simulation-model.md](../simulation-model.md))。外部ベイク済みアニメは**入力アセット+パラメータ**としてキーに含め、再生は`t`の純関数として扱う
- **キャッシュキーの完全性原則(2026-07-10追記、キャッシュ制御とAPIの関係)**: キーは「ノード出力に影響し得る全入力」の完全な列挙でなければならない。したがって**プラグインAPIに入力の口を1つ開けるたび、その口のキーへの寄与を同時に定義する**(時間窓→キーの時間区間が点から窓へ / コライダー参照→参照レシピハッシュを算入 / 未定義ならその口は開けない)。凍結ゲートでのAPI審査項目に含める
- **キャッシュはプラグインAPIに露出しない**: キャッシュの配置・追い出し・無効化はホストの専権事項で、プラグインからのキャッシュヒント/自前キャッシュは受けない(隠れ状態=純関数契約違反)。「どのノードを優先的にキャッシュするか」のコスト判断は作者申告ではなく**ホストの実測(レンダ時間の計測フック、K1に含む)**で行う — 申告は嘘をつくが計測は嘘をつかない(LLM量産プラグイン前提では特に)
- **キャッシュは意味論に影響しない**: 全成果物がレシピ(Document)から決定論的に再生成可能なので、任意のエントリをいつ落としても正しさは不変(遅くなるだけ)。この保証が成り立つのは決定論契約(B-4/F-12)のおかげであり、逆にこの保証があるからLRU追い出し・予算管理・並行契約(F-2)を「性能の問題」として単純に扱える
- **区間キャッシュのクライアントは3種を想定してキーを設計する(2026-07-10追記、F-12)**: (1)解析DataTrack(最終フェーズ)、(2)グループ仮出力=ベイク(K7)、(3)**シミュレーションのStateTrack**(チェックポイント列。[simulation-model.md](../simulation-model.md)§3.3。実装はv1.x=K1a〜K1c/K7後だが、K1bのキー/無効化設計が「チェックポイントからの部分再計算」を表現できることを確認する)
- RAM/VRAM別のメモリ予算とLRU追い出しをキャッシュ層の初期設計に含める(4K RGBA=33MB/枚を常に意識)。階層の役割分担・読み戻し例外(確定出力の非同期コピーアウト)・VRAM予算の自前管理と逼迫時の退避はしご・ディスク階層は[memory-model.md](../memory-model.md)のP1〜P4に従う(2026-07-09)
- **並行契約(F-2、K1の設計に含める)**: (1) 読み手はエントリのハンドル(参照カウント)を取得して使う。追い出しは「LRU選定→参照ゼロになってから解放」の遅延方式で、**使用中エントリを無効化しない**。(2) キャッシュの内部ロックは1段のみ(ロック保持中に別のロックやGPU同期を取らない)。(3) 無効化(パラメータ変更)はwriterのコマンド適用起点でエポック/世代番号を進め、古い世代のスナップショットで走行中のレンダは自分の世代のエントリを読み切ってよい。(4) バックグラウンドのプロキシ/解析ジョブ(K4/K5)はスナップショット読み+結果メッセージ返しのみで、キャッシュ/ドキュメントへ直接書かない(M2-D8の所有権モデルに従う)
- インポート時パイプライン: プロキシ生成(低解像度)+ VFR→CFR正規化
- **解析(色解析・オプティカルフロー・トラッキング)は最終フェーズへ移動した(2026-07-09決定、[roadmap「解析駆動」](../pitfalls-and-roadmap.md)参照)。** M4のスコープはキャッシュ/プロキシ/デコーダプール/ベイクまで。区間キャッシュの並行契約とキー設計(K1b)は解析の“口”として残すが、解析プロデューサ実装(旧K3/K5)は最終フェーズ
- ノードグラフの領域伝播(RoI)設計はNatronの考え方を参考(references.md、GPLのため設計参考のみ)
- **領域契約はK0で先行固定する**: OpenFX型の論理的出力範囲(RoD)と要求領域(RoI)を分離し、`Finite / Infinite / Unknown`を持つ。`Unknown`は空でなく最適化不能を表し、全入力RoDまたはHost安全上限へ保守的fallbackする。実texture bounds、Document永続値、GPU alpha readbackと混同しない。正本は[既知技術による処分決定](../reviews/2026-07-14-motion-foundation-known-tech-disposition.md)

## タスク分割(粗案)

### 操作単純化モデルへの割当

M4は[操作単純化モデル](../interaction-simplicity-model.md)の**再計算の予測可能性**を担当する。Direct/Tool/Advancedという入口差はcache keyへ入れず、出力を変えるDocument意味だけを入れる。K1bのキー網羅性変異テストとK2の無効化伝播へ代表操作コーパスの変異を追加し、実装時点で存在するplugin ID/version/content hash、DataTrack、target、scopeを追跡する。M5-P2Dでdepth policyが入った時、PP-Gate通過後にModifierが入った時は、それぞれ追加・削除・並べ替えを同じ変異集合へ追加する。proxy/Bake/解析jobの完了を編集の前提にせず、Purge/Refresh/再起動を通常の修復手順にしない。

`K1`は既存文書から参照されるキャッシュ基盤のumbrella IDで、実装PRの単位ではない。実装はK1a〜K1dへ分け、`M4-K1`という既存参照はこの4件のうち利用機能に必要な範囲を指す。同様に`K7`はグループ仮出力、`K8`は全曲Draftキャッシュのumbrella IDであり、実装PRは各枝番へ分ける。

| ID | 内容 | 依存 | 完了条件(概要) |
|---|---|---|---|
| K0 | [#167](https://github.com/oshikaidesu/Motolii/issues/167) **SpatialExtent / RoD / RoI契約spike+凍結判定**: `Finite(Aabb) / Infinite / Unknown`、output extent、要求出力から各入力regionへの逆伝播、Host clamp、transparent-black範囲外を既存render graph上で固定。最初は`Unknown`既定で正しさを保つ。Document schemaと最適化は触らない | 凍結ゲート解凍手続き, M2-D3 | (1)Blurが半径分入力regionを拡張 (2)transformが必要入力を逆写像 (3)無限generatorを有限Final/Stage要求へclamp (4)Unknown経路が全域評価とpixel一致 (5)過小Finiteを注入したconformanceが全域評価との差を検出して宣言を拒否 (6)未検証pluginはUnknown (7)正準座標のみ (8)同期readbackなし (9)preview/exportが同じ領域関数を使用 |
| K1a | **ResourceLedger+hard budget基盤**: texture/buffer/cache/prefetch/stagingをowner・階層・resident/pinned別に事前見積りし、VRAM/RAM/ディスクと共有メモリ合算のhard capをadmission前に強制。wgpu `AllocatorReport`は診断照合、`MemoryBudgetThresholds`は対応backendの追加安全柵に限定 | K0, 凍結ゲート | (1)小さな注入予算で割当合計がcapを越えない (2)mip/sample/format/alignmentを含む見積り境界値 (3)allocator report=`None`でも同じ判定 (4)共有メモリ合算cap (5)全ownerの生存量が解放後ゼロ (6)拒否診断にowner/要求量/使用量/予算 (7)Document/plugin公開契約に予算・backend型なし |
| K1b | **キャッシュ同一性+並行store**: 完全cache key、区間/Quality/RoD・RoI寄与、LRU metadata、計測フック、参照カウントhandle+遅延解放+単段lock | K1a, M2-D8 | キー網羅性変異と再計算範囲の単体試験。Unknown fallbackと最適化経路がpixel一致。使用中entryが無効化/追い出しされず、読み手・無効化・evict並行でdeadlockしないloomまたは反復stress test |
| K1c | **階層admission+退避**: K1a予算とK1b storeを接続し、VRAM→RAM→disk降格、同期evict、disk watermark、全pin時の型付き拒否、確定出力だけの非同期copy-outを実装 | K1a, K1b | (1)VRAM/RAM/disk各capでLRU降格順が再現 (2)使用中entryを待たず別候補を選ぶ (3)全pin時にOS OOMへ進まず拒否 (4)数千frameで平衡水位へ収束 (5)壊れ/欠落cacheはmissへ縮退しFinal bit一致 (6)評価chain途中の同期readbackなし |
| K1d | **PreviewPressureController**: 容量逼迫とrender deadline超過を別信号で扱い、P3/P3aの退避はしご、decode先読み削減、Draft 1/2→1/4、固定解像度時の拒否、runtime-only pressure reasonを統合 | K1c, K4 | (1)容量注入ではframe dropでなく退避が先 (2)deadline超過だけではVRAM evictionしない (3)自動解像度時だけ1/4へ遷移 (4)固定時はscale不変 (5)状態遷移にhysteresisがあり境界で往復しない (6)Document/Final frame不変 (7)理由/scale/予算値をSlint非依存snapshotで観測可能 |
| K2 | motolii-render統合: パラメータ変更・型付きtarget参照・Shared Effect Definitionの無効化伝播 | K1b, M2-D3, M2-D3e | 「1パラメータ変更→影響ノードのみ再計算」をプロファイルで確認。LookAt/Follow/Parent targetのtransform変更は参照元へ伝播し、無関係nodeは維持。Shared Effectのdefinition param変更は全useだけを無効化し、use順変更は対象layer以外を維持。target renameは無効化せず、削除/差替えはtyped diagnostic。全cache purgeで代用しない |
| ~~K3~~ | **最終フェーズへ移動**: 時系列解析ノードの区間キャッシュ + 部分再解析(解析駆動とセットで実装) | — | — |
| K4 | インポートパイプライン(プロキシ生成、CFR正規化、バックグラウンド実行) | M2-D1 | VFRスマホ素材でフレームズレなし、4K素材でプロキシスクラブが破綻しない |
| ~~K5~~ | **最終フェーズへ移動(2026-07-09決定)**: 解析プラグイン(色解析→必要ならオプティカルフロー)。roadmap「解析駆動」参照 | — | — |
| K6 | SVGインポート(usvg)→ベクターソースノード(Vello描画)。コンセプト決定によりコア機能 | M1-T7(Vello採用) | 代表的なSVG(パス・グループ・塗り/線)のゴールデンイメージテスト、アセットとしてタイムライン配置可能 |
| K7a | **グループ仮出力の成果物境界+atomic commit**: 選択groupの子合成直後・group stack適用前を、指定時間区間/Quality/FrameDescで確定出力として非同期copy-outし、temp→検証→rename後だけK1 storeへ登録する。Documentにはベイク状態・path・解像度・尺を保存しない | K1b, K1c, M2-D3 | (1)同じsnapshot/keyの再ベイクはpixel同一 (2)group stackだけの変更で子ベイクkey不変 (3)途中失敗/フレーム欠落/破損成果物をcatalogへ登録しない (4)評価chain途中の同期readbackなし (5)再起動後に検証済み成果物だけ再利用 (6)cache削除後も通常評価で同じFinalを再生成 |
| K7b | **区間無効化+世代再利用**: group内部のDocument変異から影響ノード/時間区間を導出し、その区間だけ新世代を要求する。無関係区間と走行中snapshotの旧世代handleは維持する。宣言済み時間窓は前後へ拡張し、未知依存は全区間無効化へ保守的に倒す | K7a, K2 | (1)30〜35秒だけのkey変更でその区間だけmissし他区間はhit (2)group外・group stackだけの変更で子ベイク維持 (3)Shared Effect/target参照変更が依存useだけへ伝播 (4)時間窓ぶん無効区間が拡張 (5)Unknownは過小無効化せず全区間へfallback (6)編集・再生・evict並行で旧世代読取が完走しdeadlockなし |
| K7c | **ベイク再生置換+再フリーズ**: 有効なgroupベイク区間では内部graphを評価せず成果物をsourceとして使い、miss/破損区間だけ通常評価へ戻す。手動freeze/unfreezeはキャッシュ利用policyでありDocument意味を変えない | K7a, K7b | (1)40-layer fixtureを複数groupへ分けてベイク後、再生時の内部node評価回数が0 (2)ベイク有無でpreview/export pixel同一 (3)部分編集後は無効区間だけ再計算して再利用 (4)unfreezeしてもDocument/Undo/serialize不変 (5)欠落・破損cacheで停止せず通常評価へ縮退 |
| K8a | **全曲Draft coverage planner**: Composition全尺を区間coverageとして管理し、foreground優先度を`再生に必要な次frame > 未被覆区間のDraft穴埋め > 現在位置周辺の高品質化 > 全曲の品質向上`に固定する。要求は最新再生位置で更新し、background jobは編集/UIを待たせない | K1b, K1c, K1d, M2-D3 | (1)固定操作列で優先順位が再現 (2)seek後は古い先読みより新位置の次frameが先 (3)再生停止中に全尺Draft coverageが単調増加 (4)編集で影響区間だけ未被覆へ戻る (5)全pin/低予算でもeditor操作をblockせず型付き理由を返す (6)planner状態はTransientでDocument/journalへ入らない |
| K8b | **全曲Draftディスクキャッシュ+通し再生E2E**: 合成済みComposition Draftをレイヤー数非依存の1系列としてディスクへ置き、K7成果物を入力として再利用する。再生時は音声/Transportの作品時刻を正本にし、Draft hit、通常render、最新frame表示の順で追従する。Final書き出しへDraft成果物を混入しない | K7c, K8a, M2-D5 | (1)1080p/30fps/5分・40動画layerの容量accounting fixtureで、100GB disk budget内に全曲1/2 Draftと現在位置周辺10秒以上のFinal windowが収まり全尺coverage完了 (2)実データを100GB生成せずfake/sparse storeでhard capとevictionを検証 (3)曲頭→曲末の連続再生でaudio/Transport時刻不変、映像遅延時も作品時刻を遅らせない (4)ベイク済みgroup内部の再評価0 (5)1区間編集後も無関係な全曲coverageを保持 (6)Final frameはcache有無でbit一致 |

並列レーン: K0はM2-D3後に独立実施できるが、M3-U1fの透過Stageをblockしない。K0→K1a→K1b→K1cと、K4は並行。K1c+K4→K1d。D3e+K1b→K2。K6は独立(UI接続はM3-U6と調整)。K1b+K1c+D3→K7a、K7a+K2→K7b→K7c。K1d+D3→K8a、K7c+K8a+D5→K8b。K3/K5は最終フェーズへ移動。

## 実装ガード(先行ツールの失敗・ユーザー不満クロスチェック 2026-07-11)

「キャッシュを消せ」が万能対処になっている出荷ツール群(AE/Premiere/Resolve)の苦情と、ビルドシステムのキャッシュ汚染事例(Bazel/ccache)を調査し、既存方針(完全性原則・並行契約・予算)に無いガードを抽出した。

1. **プロダクト目標「Purgeボタン不要」+キー網羅性の変異テスト**: 業界最大手3製品すべてで「キャッシュ削除」がサポートの第一手であり続けているのは、キー網羅性の失敗が事後修正不能な負債になる証拠。完全性原則(方針)を機械検証に落とす: 「出力ピクセルを変え得る変異(パラメータ/入力ファイル/フォント/プラグイン版/色設定)を列挙し、それぞれでキャッシュキーが必ず変わる」プロパティテストをK1b完了条件に追加。手動全消去UIはデバッグメニューにのみ置き、使われたらそれ自体をキー漏れバグのシグナルとして扱う
2. **同一性判定にmtime・パス・時計の単調性を使わない**: Premiereは (a) 同名上書きファイルで古い映像を表示し続け(キーが実質パス+タイムスタンプ)、(b) NASとの時計ズレで「開くたび全再conform」の無限ループを起こした(公式KBが時刻同期を案内する事態)。`source_id`は内容指紋(先頭/末尾チャンクhash+サイズ)とし、パスは指紋への別名にすぎない扱いに。冪等性テスト「同一入力への解析/プロキシジョブの2回目はno-op」を、mtimeを±数分ずらしたモックFSで回す(K4)
3. **キーに環境saltを含める**: Bazelは環境(glibc版)がキーに入らずABI非互換オブジェクトを混入させた。フレームキャッシュのキーにはレンダラ版・キャッシュフォーマット版・プラグインの内容ハッシュ(mtimeでなく内容 — ccacheの`compilercheck=content`の教訓)・ffmpegビルド識別(同一ファイルでもビルド差でデコード結果が変わり得る)を算入する。フォーマット変更時はグローバルsaltのバンプ1行で全無効化できる構造に(K1b)
4. **ジョブ成果物は検証を通ってからアトミックにコミット**: KdenliveはVAAPI初期化失敗で生まれた数百バイトの壊れたプロキシを「成功」として登録し再生を壊した。Bazelにも「exit 0だがmalformedな出力がキャッシュを汚染し全消費者に波及」の実例(#4276)。プロキシ/ベイク/解析の成果物はtempに書き→デコード可能・フレーム数/duration一致を検証→renameでコミット。壊れファイルは絶対にカタログへ登録せず、フルレゾへ透明フォールバック。「2回目の起動で再生成ジョブが0件」を冪等性の回帰テストに(K4/K7)
5. **キャッシュは正しさに対して常に透明(異常系も含めて)**: Resolveには「レンダーバーが青(キャッシュ済み)なのにMedia Offlineで再生停止」という、エントリの存在と有効性の混同によるユーザー可視エラーがある。エントリの喪失・破損・読み取り失敗は**missと同一の挙動(再計算)に必ず縮退**し、ユーザー可視のエラー状態を作らない。テスト: レンダ中にキャッシュファイルをランダム破壊/削除しても出力がbit一致(K1b/K1c)。方針「キャッシュは意味論に影響しない」の異常系版
6. **ディスクキャッシュも同じハード予算下に置く**: Premiereのmedia cacheは外付けSSDに設定してもCドライブを数十GB食い潰し、「running low on space」ポップアップが常態化(予算が後付けオプトイン+時間ベースGCで、書き込み時のadmission controlが無い)。ディスク階層([memory-model.md](../memory-model.md) P4)にもRAM/VRAMと同格の必須予算+LRU+書き込みadmission制を適用し、既定の置き場所はシステムドライブを避け、空き容量の絶対下限(watermark)で書き込み停止。テスト: 予算1GBで長時間スクラブしてもディレクトリサイズが予算+εを超えない(K1c/K7)
7. **プロキシ=デコード置換のみ。解釈はプロキシに焼き込まず、生成キーにも入れない**: Premiereの「プロキシと本編で色が違う」は、色管理・LUT・interpretがプロキシ生成時点で焼き込まれる(または落ちる)ことが根因。解釈パラメータは「キーに入れると解釈変更のたび再生成地獄/焼き込むと不一致」の両落ちで、唯一の正解は**プロキシ/フルレゾ共通のグラフ側で再生時に評価する**こと(色変換一元化=絶対規律2のプロキシ版)。ゴールデン: 同一フレームのプロキシ経路/フルレゾ経路の出力をΔE閾値で比較(K4)
8. **プロキシは`source_id`に紐づく内部生成物(ユーザー非可視)**: Premiereの外部プロキシ手動アタッチは「音声チャンネル数不一致で拒否」等、対応関係の検証責任をユーザーに転嫁して苦情源になった。インポート時自動生成(既定方針)を貫き、対応検証(フレーム数/タイムコード一致)は取り込みジョブ内で行う(K4)
9. **予算はソフト目標でなくハード上限**: AE 2022のメモリリークでは、ユーザーが「Nフレームごとに自動purge」「他アプリ用予約RAMを増やしてAEの上限を絞る」という予算強制の手動代行で自衛していた。割当が予算超過なら同期evict、evict不能(全pin済み)なら該当ジョブを失敗させ、OSのOOM killerに到達させない。CIに平衡テスト: 数千フレーム連続レンダでRSS/VRAMが平衡水位に収束すること(単調増加=リーク検出)。デバッグビルドでは参照カウントハンドルに生成backtraceを記録し、終了時に生存ハンドルをダンプ(K1a/K1c)
10. **時間窓を持つノードはメモリ見積りを事前宣言→admission control**: Resolveの「GPU Memory is Full」はtemporal系(NR等)がウィンドウ分のフレームを暗黙に常駐させることが主因の一つで、エラーに要求元・要求量が出ないためユーザーが対処不能だった。TemporalFootprint(凍結ゲート項目18)の宣言値から「ウィンドウ長×フレームサイズ」をスケジューリング前に予算照会し、入らなければ実行前に縮退(プロキシ/タイル化)。割当失敗の診断は必ず「ノード+要求量+現予算」を含める(K1a/K1c/K2)
11. **バックグラウンドジョブはエディタを人質に取らない**: Premiereの「Media Pending」「Preparing Audio」ハングは、index/conform/波形生成の完了が表示の前提になっているため、ジョブ失速=編集不能になる構造。不変条件「エディタ操作は解析/プロキシジョブの完了を決して待たない」(成果物が欠けていれば空表示+進捗インジケータ)+ジョブに進捗ハートビート+watchdog(N秒進捗なしで再起動/縮退)。テスト: ジョブを人工的に無限スリープさせてもタイムライン操作のp99レイテンシが閾値内(K4。M2-D8の所有権モデルが前提)
12. **細粒度キャンセルポイント+再オープン後の成果物再利用**: FCPXは背景レンダとユーザー操作の綱引き(粗粒度preempt: 走り出した重い単位を途中で譲れない)と「ライブラリを開くたび全再レンダ」(不安定キー)の両側で不満を生んだ。prefetch/ベイクジョブはタイル/数フレーム単位の協調的キャンセルポイントを持ち、ユーザー入力で数ms以内に譲るSLOを計測。永続キー(ノード×区間×ハッシュ)で再オープン後も再利用し、「開くたび全再計算ゼロ」を回帰テストに(K1b/K7)
13. **インポート時のfps誤ラベル検出(VFR正規化と別軸)**: NTSC系(30000/1001)を整数30/60として扱うと2時間で7.2秒ずれる(長尺収録の定番苦情)。K4のCFR正規化に加え、コンテナ申告fpsと実測PTS間隔平均を照合し、1001系との0.1%差を検出したら警告+実測値採用。テスト: 2時間相当の合成タイムラインで「音声サンプル総数 == duration×sample_rate」の厳密一致(K4)
14. **容量逼迫と再生期限超過を混同しない**: VRAM不足にframe dropを当てても1枚の必要容量は減らず、逆にshaderが遅いだけなのにcacheを捨てると再計算で悪化する。K1dは容量注入とdeadline遅延注入を別fixtureにし、前者だけがP3退避、後者だけがP3aの最新frame表示へ進むことを固定する。解像度固定時はscaleを黙って変えず、project fps/audio clock/Final出力を一切変更しない
15. **MVの通し確認を局所LRUへ還元しない**: VFXショット型の「再生head近傍だけを最高品質化」では、曲頭から曲末まで展開・密度・反復を確認するたびcache missへ戻る。K8は全尺Draft coverageを一級状態として持ち、局所Finalより未被覆Draftを先に埋める。全曲Draftは合成済みComposition 1系列で、40 layer分の展開済みframeを複製保存しない。disk budget不足時も既存coverageを無差別purgeせず、品質向上候補・古い局所Final・再生成の安いentryから退避する
16. **freezeを永続意味や全尺再ベイクへ変えない**: freeze/unfreezeはcache利用policyであり、Documentのgroup構造・Undo・serializeを変更しない。内部編集時は依存区間だけを無効化し、時間窓/Unknown以外で全尺を捨てない。group stackは既定ベイク点より後なので、stack parameter調整だけで高価な子合成を再ベイクしない。K7b/K7cは部分再計算回数とcache hit区間を自動審判にし、見た目だけの確認で完了にしない

出典: community.adobe.com(Purge文化/同名上書きstale/conformループ/Media Pending/AE 2022メモリリーク) / helpx.adobe.com(media cache KB) / forum.blackmagicdesign.com・hetzbiz.cloud(Resolveキャッシュ/Media Offline) / KDE bugs(Kdenlive壊れプロキシ) / bazelbuild/bazel#4276 / ccache.dev/manual(compilercheck・sloppiness明示主義) / discussions.apple.com(FCPX背景レンダ綱引き) / creativecow.net(プロキシ色違い/GPU memory full)

## 未決事項

- 解析結果(DataTrack)のディスク永続化をv1に含めるか(ベイク/プロキシの**ディスクキャッシュ自体はv1に含めると決定** — [memory-model.md](../memory-model.md) P4、2026-07-09)
- (最終フェーズで判断)K5の実装手段(wgpu compute自前 / OpenCV / ONNXモデル)と、色解析だけに留めるかオプティカルフローまで含めるか
