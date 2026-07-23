# メモリモデル: VRAM/RAM/ディスクの役割分担

作成日: 2026-07-09

2026-07-23歴史監査: cutoff全6版を[Unit 5L回収](reviews/2026-07-23-historical-memory-model-lineage-recovery.md)で処分した。VRAM作業セットとRAM／disk容量階層の分離、Host所有台帳、admission前hard cap、容量／期限制御の分離は維持する。ただし固定2枚中間target、約400MB、40 layerでも1GB未満は直列graphの歴史的floorであり製品保証ではない。ResourceLedger、cache copy-out、K7 group bake、K8全曲Draft coverageは未実装である。

GPU駆動(VRAM常駐)というコンセプトは、「VRAMは増設できない。AEならRAM増設という逃げ道があるのに、自ら捨てているのでは」という疑念を構造的に何度も呼び込む。この疑念への答えを各ドキュメントに分散させず、**容量・メモリ階層の議論はこのファイルに一元化する**。他のドキュメント(concept / performance-model / 各仕様)はここへのリンクだけを置く。これは北極星(モーショングラフィックが主役)を容量談義で侵食しないための封じ込めでもある。

## 結論

**GPU駆動は自縄自縛ではない。VRAMに縛るのは「作業セット」だけで、容量を食う実体(キャッシュ・ベイク・先読み)はRAM/ディスク階層に置く。**

| 階層 | 置くもの | 容量の性質 |
|---|---|---|
| VRAM | 作業セット（現在frameのsource texture＋liveness-aware中間target pool）＋render済みcacheのhot層 | 直列graphの2 target／約400MB級はfloor。branch、先読み、staging、format、cacheに応じて増えるためlayer非依存や1GB未満を保証しない |
| RAM | デコード済み/プロキシフレーム、解析結果(DataTrack)、確定出力のフレームキャッシュ(P1の非同期コピーアウトで充填) | マシン依存。増やせばキャッシュ容量に直接効く |
| ディスク(NVMe) | ベイク(グループ仮出力)、プロキシ、合成済み全曲Draftを含む長尺キャッシュ | ユーザー指定hard budgetまで。1080p 8bit 30fpsの再生に必要な250MB/sはNVMe(数GB/s)で余裕 |

AEとの対比: AEはRAM=作業セット+preview cacheの一体運用（だからRAM増設が効く）。うちはVRAM=作業セット、RAM／disk=cache（容量はここで稼ぐ）。**「RAM増設が効かない設計」ではなく、「RAMの役割が作業セットからcache階層へ変わる設計」である。** 禁止しているのは作業セットをVRAMから降ろすこと（CPU合成への回帰）だけである。VRAM作業セットを定数・小と断定せず、Host台帳とhard budgetで実測・制御する。

## ポリシー(他ドキュメントを拘束する決定、2026-07-09)

### P1. 「読み戻し禁止」の正確な範囲

[performance-model.md](performance-model.md)のGPU原則2が禁止するのは**評価チェーン途中での同期読み戻し**(エフェクト間でGPU→CPU→GPUと往復する、フレームごとにmapで待つ等のストール)。以下は例外として明示的に許可する:

- 書き出し(従来通り)
- ゴールデンテスト(従来通り)
- **確定した出力(最終フレーム/ベイク対象グループの出力)の非同期コピーアウト**によるRAM/ディスクキャッシュへの充填。非同期・パイプライン化されている限り評価chainをストールさせない。**(2026-07-13訂正、2026-07-23現行再確認)実証済みなのはダウンロードバッファ再利用まで**で、現行exportは各フレームで`map_async`完了を同期待ちする直列ループ(render→download待ち→encode)。複数本のbounded staging ringは候補であり、本数・優先度・方式はGAP-29の原因分離ベンチと採択判断前に決めない。exportの最終frame copyとK1c/K7aのcache copy-outは帯域／Queue競合を別々に測る([Unit 5C回収](reviews/2026-07-23-historical-wgpu-readback-cold-compile-lineage-recovery.md))

この例外が無いと、合成結果のキャッシュ(AEのRAMプレビュー相当)がVRAM予算だけに縛られ、「自縄自縛」が現実化する。逆にこの例外を「チェーン途中の読み戻し」へ拡大解釈した瞬間、AEのPCIeバウンス地獄が再発する — 境界は**「レンダが完了した成果物を流し出すだけ」かどうか**。

### P2. ターゲットHWのメモリモデル前提

- **設計基準は分離VRAM(dGPU)とする。** v1の動作保証はmacOS(開発主機)のみだが、**Windows対応は将来ターゲットとして確定**(2026-07-09)しており、Windowsの主流はVRAM固定(8〜24GB)のdGPU。「ユニファイドメモリだから容量は気にしなくてよい」という前提を設計に焼き込むと、Windows移植がBlender/Resolve型の後付け改修になる
- 開発主機はApple M4 / ユニファイドメモリ16GB。VRAM=RAM共有でPCIe転送は無いが、GPUが使える実効上限はOS推奨値で全メモリの2/3〜3/4程度、そこからOS・他アプリを引くと**キャッシュに使える枠は数GB** — 奇しくも中級dGPUと同程度の予算感であり、P3の予算・退避はしごは開発初日から実地で鍛えられる
- 帯域の差(ユニファイドは機種差大 vs dGPU 400〜1000GB/s)は、Draft品質(1/2解像度で帯域1/4)で吸収する設計が既にある(performance-model)

### P3. VRAM予算は自前で持つ(ドライバに任せない)

- wgpuには**ポータブルで信頼できる空きVRAM API**が無い(2026-07-13、wgpu 29時点へ表現を更新)。`Device::generate_allocator_report()`でwgpu管理下の割当量は取れ、`MemoryBudgetThresholds`でD3D12/一部Vulkanの予算閾値は設定できるが、allocator reportはbackend依存で`None`になり得、`MemoryBudgetThresholds`も対応backendが限られる。Metalを含む全環境の空きVRAM・レジデンシを統一的には取れない。**allocator reportは診断補助に使い、正本は自前台帳**([反対側レビュー](reviews/2026-07-13-wgpu-challenges-counter-review.md) B-2)。超過時の挙動はOS依存: WindowsはWDDMがシステムRAMへ自動ページング(クラッシュはしないが1桁遅くなる見えない崖)、Vulkan系はallocation失敗やdevice lost。**どのOSでもドライバの自動ページングに頼らない**
- キャッシュ層(M4 K1a〜K1d)はVRAM予算を設定値として自前管理し、超過する前に追い出す。正本はtexture/buffer descriptorから見積もる**Motolii自身の割当台帳**で、`AllocatorReport`との差は診断にだけ使う。対応backendでは`MemoryBudgetThresholds`を追加の安全柵にするが、未対応backendでも同じ挙動になることを必須とする
- これは採択済み契約であって現行実装の説明ではない。`PipelineCache`、render target pool、wgpuのbudget threshold設定をResourceLedger／全owner accounting／admission成立と数えない
- 予算はソフト目標でなく**新規割当のadmission前に強制するhard cap**とし、作業中の一時割当とdevice復旧用の安全余白を予算外へ予約する。使用中で追い出せないresourceだけで要求を満たせない場合は、OS paging/OOMへ進まず要求元・要求量・現予算を含む型付きエラーへ縮退する
- ユーザー設定は`Auto`を既定とし、詳細設定でMotoliiが使ってよいVRAM/RAM/ディスクの絶対上限を指定できる。これは作品の意味でなく**User settings**であり、Document・journal・plugin APIへ入れない。`Auto`の具体値と「省メモリ/性能優先」preset値は基準機計測後に固定し、実装者がGPU名や総RAMだけから場当たりに決めない
- Apple Silicon/iGPU等の共有メモリでは、VRAM台帳とRAMキャッシュを別々に上限まで使わせず、両者の合算上限も持つ。dGPUでも他アプリ・surface・driver内部割当の余白をゼロにしない
- 逼迫時の退避はしご(この順で発動):
  1. レンダ済みVRAMキャッシュをRAM/ディスク階層へ降格(P1のコピーアウト)
  2. デコード先読み深度の削減
  3. Draft解像度の段階降格 1/2→1/4(performance-model既定の仕組みに接続)
  4. 解像度固定中、または1/4でも作業セットが入らなければ、新規preview/background jobを型付き拒否し、既存の最後の正常frameと編集操作を維持する。FPS低下は1フレームの必要容量を減らさないため、この容量退避はしごへ含めない

### P3a. 容量逼迫と再生期限超過を別の制御ループにする

VRAM/RAM予算超過は**容量**、GPU演算・帯域不足で次の表示時刻に間に合わない状態は**throughput/latency**の問題である。同じ「重い」という見た目でも対策を混ぜない。

- 容量制御はP3の割当台帳とadmissionで判定し、キャッシュ降格・先読み削減・許可時だけ解像度降格を行う。表示FPSを落としても1枚のtexture容量は変わらない
- 再生期限制御はGPU frame timeとrender queue latencyで判定する。既定Draft値(低sample、最終質感skip)を使った上で、解像度自動時は1/2→1/4へ段階降格し、それでも間に合わなければ**表示する中間frameを捨てて最新時刻だけを要求**する
- 解像度固定時は勝手に縮小せず、Draft固有のeffect品質まで適用した後、表示frameを間引く。project fps、`RationalTime`、audio/Transport主クロックは変更せず、30fps素材が遅く再生されるのではなくpreview表示だけが15/10fps相当へ落ちる
- 自動降格・frame drop・固定設定による拒否は観測可能にし、理由・現在の解像度scale・実表示fps・予算使用量をHUDへ出す。自動状態はTransient、ユーザー選択はUser settingsで、どちらもDocumentへ保存しない
- Final書き出しは同じ`render_frame(t, Quality)`を全frameに対して完走させ、表示frame dropや自動Draft降格を持ち込まない。リアルタイムに間に合わない場合は時間を掛けて正しく書き出す

### P4. ディスクキャッシュはv1に含める(ベイク/プロキシ/全曲Draft用)

3〜5分MVの全曲キャッシュはどのマシンのRAMにも入らない(1080p 8bitで約15GB/分、5分≈75GB。fp16なら倍)。プリコンポ代替の**ベイク(仮出力、M4 K7)が「全曲スクラブに耐える」にはディスク階層が前提**。プロキシ(K4)は元からディスク成果物であり、同じ置き場・同じ無効化(世代)管理に相乗りさせる。解析結果(DataTrack)の永続化は引き続き未決(M4)。

MVでは現在位置の数秒だけでなく、曲頭から曲末まで展開・密度・反復を確認する通し再生が主要操作になる。したがってディスク階層は局所LRUの退避先だけでなく、**合成済みCompositionの全曲Draft coverage**を保持する。全曲Draftはレイヤー数に依存しない1系列であり、素材40本を非圧縮展開して複製保存する意味ではない。優先度は「再生に必要な次frame → 未被覆区間のDraft穴埋め → 現在位置周辺の高品質化 → 全曲の品質向上」とし、音声/Transportの作品時刻をcache生成待ちで遅らせない。

容量基準の審判として、1080p/30fps/5分の1/2解像度fp16 Draftは非圧縮上限でも約37GBである。100GBのdisk budgetは全曲Draft 1系列と現在位置周辺10秒以上のFinal windowを同時に収められる基準fixtureに使う。ただし100GBを製品既定値やDocument意味にはせず、User settingsのhard budgetと実形式のaccountingからadmissionする。CIは100GBの実ファイルを生成せずfake/sparse storeで検証する。

グループ仮出力はM4 K7a〜K7c、全曲Draft coverageはK8a/K8bへ分ける。freeze/unfreezeはcache利用policyでありDocumentへ保存せず、内部編集では依存する時間区間だけを無効化する。完成に近いgroupほど再計算から外れ、その成果物を全曲Draft生成が再利用することで、作品が固まるほど通し再生を軽くする。

P4はM4の製品要件であり、現行codeにdisk store、K7 freeze、K8 coverage、100GB accounting fixtureはない。37GB／100GBは非圧縮上限と審判fixtureの歴史的試算で、実format、既定budget、性能保証ではない。

## 試算の読み方(performance-model §7 への注記)

§7の40レイヤー試算は検算済みで、算術はすべて正しい(2026-07-09検算: ソース40×8.3MB≈330MB、合成帯域1.7GB/フレーム=50GB/s@30fps、YUVアップロード3.7GB/s、いずれも再計算一致)。ただし**下限(floor)として読む**:

- **未計上: decoder先読み・YUV staging。** 先読み4 frame×40本をYUVで持つ歴史概算は約500MBだが、decode pool自体が未成立で、branch target、alignment、surface、cache等も別勘定になる。「合計1GB未満」の保証にはしない
- **未計上: エフェクトスタックの帯域。** §7の〜50GB/sは合成のみ。1エフェクトパス=fp16の読み+書き33MB/枚なので、平均2エフェクト/レイヤーなら計〜130GB/s(dGPU 400GB/sの3割、Draft 1/2で1/4)。結論(成立)は変わらないが「1割」ではなくなる
- **M4キャッシュのVRAM占有は別勘定。** キャッシュは予算いっぱいまで使うのが仕事であり、P3の予算と退避はしごで管理する

「支配コストはデコード」という§7の結論は覆らない。

## 疑念台帳

「GPU駆動で本当に良いのか」系の疑念は再検討せず、まずここを読む。新しい疑念が出たら行を追記する。

| 日付 | 疑念 | 結論 |
|---|---|---|
| 2026-07-09 | VRAMは増設できず、RAM増設の逃げ道を自ら捨てているのでは(自縄自縛疑念) | VRAM作業セットとRAM／disk容量階層の責任分離で解消。作業セット量は固定2枚・約400MBへ凍結せず、livenessとHost台帳で制御する。P1〜P4を決定し、CPU合成へは戻さない |
| 2026-07-16 | wgpuなら空きVRAM取得と退避が標準化されているのでは / 重い時は解像度・FPSをどう落とすか | portableな空きVRAM正本は無いため自前台帳+hard capを正本とし、backend APIは補助に限定。容量逼迫はP3退避、期限超過はP3aのDraft降格+最新frame表示で分離。ユーザー予算・解像度固定はUser settings |
| 2026-07-16 | MVは局所previewだけでなく全曲を通して流れを見る。100GB程度で通し再生を守れるか | 合成済み1/2 Draftは5分1080p30 fp16でも約37GBでレイヤー数非依存。100GB fixture内で全曲Draft+局所Finalを優先保持し、K7 freeze成果物をK8が再利用する。素材全本の非圧縮展開は保持しない |

## 関連リンク

- [performance-model.md](performance-model.md) — 帯域が支配コストである物理、§7の40レイヤー試算、GPU原則(P1が精密化)
- [specs/M4-cache-and-analysis.md](specs/M4-cache-and-analysis.md) — キャッシュ層の実装仕様(K1a〜K1dの台帳/並行契約/予算/逼迫制御、K4プロキシ、K7 group freeze、K8全曲Draft coverage)
- [pitfalls-and-roadmap.md](pitfalls-and-roadmap.md) — B-5(キャッシュのメモリ予算)、E(ターゲットOS。Windows将来対応)
- [concept.md](concept.md) — 仮出力(ベイク)=プリコンポ代替の決定
