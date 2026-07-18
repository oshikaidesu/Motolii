# 個体性・介入・上限・縮退・遊びの先例翻訳(2026-07-17)

ステータス: **調査第二陣(既知部分の検索とMotolii翻訳案)**。[小さなコアと探索可能な拡張](../extensible-core-model.md) §7(個体性)と§9(遊びの段階)が開いた問題群について、「既知の要素の組み換えで埋まる部分」を一次資料で確認し、Motoliiの語彙へ翻訳する。[レビュー規律](README.md)に従い、本書の結論をそのまま設計根拠にしない。反対側レビュー未実施のため、全項目は**仮説と整合する事例**に留める。採用時は判定語(採用/縮小/延期/棄却)を併記し、公開契約・Document schemaへの反映は各ゲート(M5-P0I spike、PP-Gate、解凍手続き)を通す。

**再決定しないもの(既決の正本)**: `InstanceId != index`・明示seed・PCG32([2026-07-15決定](2026-07-15-relative-scope-duplicator-decision.md))、Cavalry型Duplicator/Context/Behaviour(同)、Element Domainを畳まない([2026-07-14処分](2026-07-14-motion-foundation-known-tech-disposition.md))、SimulationPlugin+StateTrackとStateTrackのQuality非依存([simulation-model.md](../simulation-model.md))。本書はこれらへ補強出典を足すだけで、意味を変更しない。

出典の等級: 無印=公式仕様・公式マニュアル・本人論文/本人記事。[参考]=第三者媒体・コミュニティ解説(事実認定に使わない)。[実機観察]=ユーザーによる当該版の操作確認で、公開仕様の代用や機能不在の悉皆証明には使わない。

## 1. 目的と方法

extensible-core-model §9は「既知の要素を組み換えるだけでは足りない」段階への移行を宣言した。その仮説を無検証で信じないため、本書は逆側から攻める: **各問題を既知の解で埋められるだけ埋め、埋まらなかった残りを名指しする**。借りるのは意味構造(何を正本にし、何を宣言し、何を禁じたか)だけで、UI・schema・型名は借りない。

## 2. 要約表

| 領域 | 既知で埋まる意味 | 代表先例 | 既知で埋まらない残り |
|---|---|---|---|
| 四段の存在論(§3) | 各段は独立に実在し、段間の移行は明示操作 | Blender GN、C4D、USD、Niagara | 四段を一つの利用者向け文法として提示した製品は未確認 |
| アドレス可能性(§4) | IDはopt-in capabilityで、常時コストにしない | Niagara Persistent ID、Blender `id` | (既決の補強のみ) |
| 選択≠Object化(§5) | 選択は個体の外に立つタグ/集合/述語として保存 | MoGraph Selection tag、Houdini group | 再生成される動的identityへの選択の保存と欠落表示。AM実機では描画個体の直接操作まで到達せず |
| 宣言的介入(§6) | 介入=ソルバが解釈する宣言データ。Exclude系は集合側保存の直接先例あり | Vellum pin/stopped、POP Kill、USD inactive/invisibleIds | Pin/Impulse型の「値の介入」を集合側へ保存する形、参照待ち/再接続 |
| 集合所有の状態と出口(§7) | 状態はソルバ/ネットワーク所有、個体はID、event出口はID前提 | Houdini DOP、Maya nucleus、ECS、Niagara Events | 個体別overrideの保存境界(既決どおり未決のまま) |
| 上限非焼き込み(§8) | 構造へ焼いた上限は後継フォーマット丸ごとの代価を払う | GIF→PNG、FAT→exFAT、MIDI 2.0、Shapefile→FGDB | 資源上限の文書化と意味非汚染の両立は運用課題 |
| Preview縮退(§9) | 縮退は表示側の別管理値+状態の可視化+不能なら素通し警告 | Blender Simplify、AE Fast Previews、Nuke proxy | 縮退可能軸を型付き契約にした先例は未確認 |
| 遊びの観察と停止線(§10) | 観察指標と「早期固定の害」の学術語彙が存在する | Resnick、Wright、Cognitive Dimensions、Dreams、LBP | 楽しさの判定そのものは移植不能。Motolii固有の観察が必要 |

## 3. 四段の存在論(描画→アドレス可能→状態→Document実体)

**問い**: §7の四段(描画個体/アドレス可能な個体/状態を持つ個体/Document実体)は発明か、それとも既知の分離の整理か。

**既知の解**: 四段のそれぞれと「段間の明示移行」は、成熟実装に独立して存在する。

- **描画個体≠実体**: Blender公式はinstancesを「同一ジオメトリの効率的な複製」とし、[Realize Instances](https://docs.blender.org/manual/en/latest/modeling/geometry_nodes/instances/realize_instances.html)を「インスタンスを実ジオメトリへ変える」明示ノードとして定義する。個別編集可能になる代わりに「多数インスタンスでは性能が大きく悪化する」制約まで文書化されている(=段の移行はコスト付きの明示操作)。C4Dも実体化を[Current State to Object](https://help.maxon.net/c4d/s22/us/html/5667.html)という別コマンドにし、元オブジェクトを変更しない。
- **アドレス可能な個体**: [USD PointInstancer](https://openusd.org/24.08/api/class_usd_geom_point_instancer.html)は「配列indexがフレーム間で別個体に再利用される」問題を明記した上で、任意時刻の個体同定用に時変`int64 ids[]`を置く。個体はprimにならないままIDで参照・介入できる(§6)。
- **状態を持つ個体**: Houdini公式はDOPのobjectを「データの容れ物」とし、状態(sim data)の計算と添付をソルバの責務にする([Understanding Houdini dynamics](https://www.sidefx.com/docs/houdini/dyno/about.html))。
- **実体化時のidentity合成**: Realize Instancesは実体化時にインスタンス側idと内部ジオメトリのpoint idを**合成して重複を防ぐ**と明記。既決のnested Duplicator「親子InstanceIdのhash合成」と同型の解が独立に存在する。

**Motoliiへの翻訳**:

| 段 | 先例の対応物 | Motoliiの既存語彙 |
|---|---|---|
| 描画個体 | GN instances、GPU instance列 | Duplicator/Generatorの評価結果(既決: Timeline row非生成) |
| アドレス可能 | USD `ids[]`、Niagara Persistent ID | `InstanceId`(必要時解決、§4) |
| 状態を持つ | DOP sim data、nucleus | SimulationPlugin+StateTrack(既決) |
| Document実体 | Realize Instances、Current State to Object | 明示Materialize(一方向・1 Undo・コスト明示) |

Materializeの利用者向け意味はRealize型に揃えるのが素直: 明示操作、一方向、実体化後は通常編集、性能コストを隠さない。identity合成規則は既決のhash合成がそのまま該当する。

**埋まらない残り**: 各段は全ソフトに散在するが、**四段を一続きの利用者向け文法(いま自分がどの段の何を触っているかの提示、段を上がる操作の一貫語彙)として設計した製品は今回確認できなかった**。Blender/Houdiniでは段の区別はデータモデル知識として暗黙に要求される(Indexの不安定性を利用者がマニュアルで学ぶ)。ここはMotoliiが自分で設計する部分であり、extensible-core §10の審判「Object、評価個体、規則、状態のどれを操作していると理解したか」が対応する検証枠になる。

## 4. アドレス可能性はopt-in capability(既決の補強)

`InstanceId != index`は既決なので、新規に確認できた補強出典だけ足す。

- Houdini公式はポイント番号と`id`を明確に分離し、POPでは「**削除で番号が変わるため、常に`@ptnum`ではなく`@id`を使え**」と指示する([POP attributes](https://www.sidefx.com/docs/houdini/dopparticles/attributes.html))。`id`は「1回のシミュレーションを通して不変のunique id」。
- Blender公式は[Index Node](https://docs.blender.org/manual/en/latest/modeling/geometry_nodes/geometry/read/input_index.html)に「indexは生成アルゴリズムの内部都合で決まり、入力変更やBlenderのバージョン更新で変わり得る」と明文の警告を置き、安定参照には`id` attributeを使わせる([Distribute Points on Faces](https://docs.blender.org/manual/en/latest/modeling/geometry_nodes/point/distribute_points_on_faces.html)は生成時にstable IDを自動付与し「変形・密度変更後も残存点で値が一貫する」と明記)。
- **Niagaraの[Persistent ID](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/VersionedNiagaraEmitterData?application_version=5.3)はopt-in**で、公式説明が「フレーム間で不変の安定識別子(Particles.ID)を作る。**少量のメモリと性能コストを伴う**」と、コストと引き換えの能力であることを明記する。§7.3の「アドレス可能性は全個体への常時コストではなく、必要時に解決できるcapability」はこの構造と一致する。
- Rust実装側の定石: [slotmap](https://docs.rs/slotmap/latest/slotmap/)は(value, version)組で「スロットが再利用されても古いキーは恒久に無効」を保証し、[BevyのEntity](https://docs.rs/bevy_ecs/latest/bevy_ecs/entity/struct.Entity.html)はindex+generationで同一indexの再利用と旧参照を区別する。「消えた個体のIDを別個体へ黙って付け替えない」(§7.1)は、generational identityとして実装語彙が揃っている(解説: [Catherine West RustConf 2018](https://kyren.github.io/2018/09/14/rustconf-talk.html)、本人ブログ)。

## 5. 選択してもObject化しない

**問い**: 評価個体を選択・参照するとき、個体をDocument Objectにせずに済む保存形は何か。

**既知の解**: 選択を「個体の外に立つデータ」として保存する形が2系統ある。

1. **タグ/集合**: C4Dの[MoGraph Selection](https://help.maxon.net/c4d/en-us/Content/html/TOOL_MGSELECT.html)は、クローン個体をviewportで直接選択し、結果を**generatorに付くMoGraph Selection tagとして保存**する。クローンはObject Managerに現れず、Effectorの適用対象の限定に使う。選択状態は個体上の色付きキューブで常時可視化される。Houdiniの[group](https://www.sidefx.com/docs/houdini/model/groups.html)も「名前付きのpoint/face集合」をジオメトリに保持し、後続ノードの適用範囲指定に使う(順序付き/無順序の2種)。
2. **述語**: MoGraphの[Effector](https://help.maxon.net/c4d/en-us/Content/html/7443.html)+[Fields](https://help.maxon.net/c4d/r25/en-us/Content/html/58091.html)は、対象を列挙せず「空間的条件が返す強度」で個体群を選ぶ。列挙(tag)と述語(Field)は併用でき、役割が違う。

**Motoliiへの翻訳**: 既決の`Selector(element_context, t) -> weight`が述語側、`InstanceId`集合が列挙側に対応する。UI選択(transient)はDocumentへ入れず、**選択が永続化するのは介入(§6)が保存される時だけ**、その形式はID集合(狙った個体)か述語(条件に合う個体)のどちらか — この二形は先例で役割が分かれており、単一形式へ畳む理由がない。選択の常時可視化(MoGraphの色キューブ)は「可視の因果」(ui-interaction-language)と同じ路線。

**Alight Motion実機観察(2026-07-17、ユーザー確認)**: パーティクル/リピート系の表現は確認できたが、いずれも描画結果としての個体であり、評価個体を個別に選択、固定、除外する操作面は確認できなかった。物理状態や個体への物理介入も考慮されていなかった。この観察はAM全版・全機能に対する不在証明ではないが、少なくとも通常導線では「描画個体→アドレス可能な個体」への段上げが提示されていない実例として扱う。

**埋まらない残り**: 先例の弱点が3つ、Motoliiの要求と食い違う。(a) Houdini groupは「ポイント削除時に自動でgroupからも除去する」= **欠落を黙って消す**側の挙動で、§7.1の「参照待ちまたは欠落として識別する」はこれより強い要求。(b) MoGraph Selectionの保存がクローンの安定identity基盤か配列順基盤か(クローン数変更で選択がズレるか)は一次資料で確認できなかった — 反対側レビューでの確認事項(§12)。(c) AM実機観察では、個体を描画する親しみやすいUIと、個体を直接操作するUIの接続自体が無かった。高度なDCCに存在する集合/IDの仕組みを、その専門語彙を要求せず直接操作へ落とす部分は引き続き未確認である。

## 6. Pin / Impulse / Exclude — 宣言的介入の保存

**問い**: `Pin(instance_id, position)` / `Impulse(instance_id, at_time)` / `Exclude(instance_id)`のような介入を、個体をObject化せずに保存する先例はあるか。

**既知の解**:

- **介入=ソルバが解釈する宣言データ**(Houdini)。Vellumの固定は[per-point attribute](https://www.sidefx.com/docs/houdini/vellum/vellumattributes.html)で表現される: `pintoanimation`(位置をターゲットへ追従)、`stopped`(ビットフラグで積分停止)。削除は[POP Kill](https://www.sidefx.com/docs/houdini/nodes/dop/popkill.html)だが、その実体は「`dead` attributeへ1を書く」ことで、実削除はソルバ最終段のReapingが行う。解除専用の[POP Awaken](https://www.sidefx.com/docs/houdini/nodes/dop/popawaken.html)(`stopped`をリセット)まで用意されている。**介入の表現・解釈・実行時点がすべて分離されている**。
- **可逆性は保存形式で決まる**(Vellum)。[Vellum Constraints](https://www.sidefx.com/docs/houdini/nodes/sop/vellumconstraints.html)のPinには3形があり、公式が使い分けを明記する: Permanent(`mass`を0へ**上書き** — 元値が保存されず後から解除できない)、Stopped(`stopped`=1 — `mass`不変なので後から解除可能)、Soft(長さ0のdistance constraint)。**「解除する予定のpinはmass上書きでなくstoppedで」という公式運用**は、破壊的上書きと可逆な宣言の差がユーザー体験に直結する証拠。
- **介入を集合側へ保存する直接先例**(USD)。[PointInstancer](https://openusd.org/24.08/api/class_usd_geom_point_instancer.html)は個体をprim化せずにIDで介入する機構を2形持つ: `inactiveIds`(list-editable **metadata**、全時刻一様の無効化、`DeactivateId`等のAPI)と`invisibleIds`(**時変attribute**、時刻ごとのID列で可視性をアニメーション可能)。恒久の介入と時間依存の介入を別形式で持つ区別自体が先例にある。
- **介入をcomponent集合参照の独立ノードにする形**(Maya)。[nCloth Transform constraint](https://help.autodesk.com/cloudhelp/2024/ENU/Maya-CharEffEnvBuild/files/GUID-D1A60AD2-7879-472A-805D-72BE3D956C9A.htm)は選択頂点集合から`dynamicConstraint`ノードを作り、nucleusソルバへ接続する。介入が個体の属性ではなく**個体群を参照する第一級の宣言**として立つ。
- **反面事例**(AEエコシステム)。[Newton](https://www.motionboutique.com/files/newton4/)は公式ガイドが「Newtonはeffectではない」と明記する別アプリで、物理個体の単位はAEレイヤー(=事前のDocument実体化が前提)、結果は終了時にkeyframeへ変換して返す。以後、生きた物理状態も個体への介入手段もAE内に残らない。[Trapcode Particular](https://help.maxon.net/rg/en-us/Content/html/01-Trapcode-Particular.html)の制御粒度はemitter/particle group/[system](https://help.maxon.net/rg/en-us/Content/html/13-Trapcode-Particular-about-multiple-systems.html)/[global](https://help.maxon.net/rg/en-us/Content/html/48-Trapcode-Particular-global-controls.html)までで、放出後の特定個体を選択・編集する機能は公式マニュアルの制御面に存在しない(否定の悉皆証明は不可、§13)。「個体に触るには全部レイヤー化するか、一切触れないか」の二極が、§7が埋めようとしている穴の実在を示す。

**Motoliiへの翻訳**:

- 介入は「個体の属性を上書きする」のではなく「**Documentに宣言を追加し、Generator/Simulationが評価時に解釈する**」。Vellumの3形が示す通り、可逆性は保存形式の帰結なので、介入=宣言の追加/削除(=Undo対応、1介入1 D2 command)にすればPermanent型(元値喪失)の落とし穴を構造的に回避できる。
- `Exclude`はUSDの2形(全時刻一様/時変)がそのまま意味の叩き台になる。`Pin`はVellumの`pintoanimation`/Stopped、`Impulse`はPOPの力系が意味の先例。
- Simulationへの介入は「追加の境界条件」なので、StateTrackの無効化は既決の意味論(影響時刻以降のみ再シム、[simulation-model §3.4](../simulation-model.md))がそのまま適用できる。

**埋まらない残り**: (a) Houdini/Vellumの介入は**ジオメトリストリーム=個体側データが正本**であり、「個体を保存せず介入だけをDocumentへ保存し、評価のたびに再生成された個体へ再結合する」というMotoliiの逆転構造の直接先例は、Exclude系(USD)を除いて薄い。特に**Pin/Impulse型の「値を伴う介入」を集合側に保存した先例は未確認**。(b) 生成規則の変更で対象identityが消えた時の「参照待ち/欠落表示と再接続」は、先例が沈黙削除(Houdini group)か全時刻マスク(USD)で、§7.1の要求水準の先例が無い。ここはplugin欠落時の「保持+診断+再導入時復元」というMotolii既存パターンの転用が最短で、外部先例より内部一貫性で決める領域。(c) `Impulse(instance_id, at_time)`のような**時刻付きone-shot介入の永続保存**はゲームエンジンでは実行時API(保存されない)であり、これも先例が薄い。

## 7. 物理状態の集合所有と、個体からの出口

**問い**: 「物理状態は個体Objectではなく集合単位のSimulationが所有する」(§7.2)は先例と整合するか。個体から値/eventを取り出す口はどう作られてきたか。

**既知の解**:

- **状態の所有者はソルバ/ネットワーク**。Houdini公式: DOPのobjectはデータの容れ物で、ソルバがdataを読み・添付しながら挙動を計算し、[DOP Networkのcacheは「シミュレーションの状態全体」を自動保存する](https://www.sidefx.com/docs/houdini/nodes/dop/file.html)。Mayaの[nucleus](https://help.autodesk.com/cloudhelp/2022/ENU/Maya-SimulationEffects/files/GUID-8BB10228-74E2-49A6-864C-03110C7FBB45.htm)も「system内の全Nucleusオブジェクトのsimulation dataをソルバが計算する」集中ソルバ。ECSでは[Entityは一意な識別子にすぎず](https://docs.unity3d.com/Packages/com.unity.entities@1.2/manual/concepts-intro.html)、データはcomponent、処理はsystemが持つ([Bevyも同構造](https://docs.rs/bevy_ecs/latest/bevy_ecs/))。simulation-model.mdのSimulationPlugin+StateTrack(Host所有)は、この収束点のプラグイン契約化であり、新規要素ではない。
- **個体eventの出口はidentityを前提とする**。Niagaraの[Events/Event Handler](https://dev.epicgames.com/documentation/unreal-engine/events-and-event-handlers-in-niagara-effects-for-unreal-engine)は公式に「eventsを使うにはRequires Persistent IDsを有効化せよ」と指定する。「衝突した個体だけ別の見た目へ渡す」(§7.2)型の表現は、**アドレス可能性(§4)→event→受け手**という依存順で作られてきた。
- Sleep/Wakeの自動化: [POP Solver](https://www.sidefx.com/docs/houdini/nodes/dop/popsolver.html)は低速度が続いた個体へ自動で`stopped`=1を立て(経過は`deactivation_time`に蓄積)、介入用と同じattributeを最適化にも使う — 宣言の語彙が介入と最適化で共有できる例。

**Motoliiへの翻訳**: 集合所有は既決(StateTrack)で埋まっている。「個体から集計値やeventを取り出す境界」(§7.2で未決)は、先例上は (a) sim→DataTrack(集計値の時系列 — 既にsimulation-model §3.7で一方向出力として決定済み)と (b) **event stream(個体ID+時刻+種別の列)→Selector/受け手**の2口に分かれる。(b)を将来設計する時は、NiagaraがPersistent IDを前提条件にした依存関係(ID無しのevent口を作らない)だけ拾えばよく、schemaを今固定する必要はない。双方向結合がシムグラフ=v2の席であることも既決のまま動かさない。

**埋まらない残り**: 個体別overrideの保存(§7.2の未決)は、先例でも「介入(§6)の一種」以上の収束が見えない。未決のまま保つ現状の判断と矛盾する先例は見つからなかった。

## 8. 数の天井を意味論へ焼かない

**問い**: 「instance数などの固定上限をDocumentや公開契約へ焼かない」(§7.3)は、どの失敗と成功に支えられるか。

**既知の解(失敗側)**: 構造へ焼いた上限は、後継フォーマットの発明という最大級の代価で償還されてきた。

- [GIF89a](https://www.w3.org/Graphics/GIF/spec-gif89a.txt)はColor Tableサイズを3bitフィールドで持ち最大256色。[PNG(RFC 2083)](https://www.rfc-editor.org/rfc/rfc2083)は「GIFの置き換え」を開発動機に明記して生まれた。
- FAT16のボリューム上限(MS-DOSで2GB、[Microsoft公式KB](https://learn.microsoft.com/en-us/troubleshoot/windows-client/backup-and-storage/fat-hpfs-and-ntfs-file-systems))→ FAT32が導入されたが、**FAT32自身が最大ファイル4GiBという新しい上限を焼き**([公式比較表](https://learn.microsoft.com/en-us/windows/win32/fileio/filesystem-functionality-comparison))、[exFAT仕様](https://learn.microsoft.com/en-us/windows/win32/fileio/exfat-specification)が§1.1で「ファイルサイズを64bitで記述する」ことを設計目標に掲げてようやく解消した。**「上限を広げる」対応は次の上限を作るだけ**、という点で§7.3の「上限を置かない」側を支持する史実。
- MIDI 1.0の7bit値域(0-127)は、[MIDI Association自身が](https://midi.org/the-state-of-midi-2-0-high-resolution-performance-and-the-rise-of-profiles-update-feb-2026)「現代音源の内部精度と不整合(段付き)」と説明し、MIDI 2.0でvelocity 16bit/コントローラ32bitへ拡張+新旧の決定的変換を仕様化する規模の工事になった。
- Shapefileは各ファイル2GB・フィールド名10文字・NULL非対応が[Esri公式ドキュメントに制限として列挙され](https://desktop.arcgis.com/en/arcmap/latest/manage-data/shapefiles/geoprocessing-considerations-for-shapefile-output.htm)、後継File Geodatabaseは[既定1TB/フィールド名64文字](https://desktop.arcgis.com/en/arcmap/latest/manage-data/administer-file-gdbs/file-geodatabase-size-and-name-limits.htm)へ拡張された。

**既知の解(成功側)**:

- [glTF 2.0仕様](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html)はnode/mesh等の**個数上限を規範として定義しない**。実装依存の限界は認めつつ、仕様が置くのは「バッファ2^53バイト超をSHOULD NOT」(JSONパーサ精度由来)という助言まで。「コアは表現量の上限を定義しない。Hostは有限資源内の評価戦略を選ぶ」(§7.3)と同じ分担。
- After Effectsの30,000×30,000pxは上限を置いた側だが、[公式ヘルプ](https://helpx.adobe.com/after-effects/using/composition-basics.html)が**上限値・メモリ根拠(8-bpcで約3.5GB)・環境依存性をセットで文書化**しており、「資源上限は黙らせず根拠ごと識別可能にする」参考型。

**Motoliiへの翻訳**: §7.3の不変条件は先例の帰納として支持される。実装水準では「schemaのcount値域を狭い整数型にしない」「`max_instances`定数をDocument/公開契約に置かない」「pluginへ全件`Vec`必須の契約を書かない」が対応する(既にレビュー観点「instance数や全件materialize等、現在の実装都合を公開意味へ焼いていないか」がある)。資源不足を「識別可能な品質縮退または型付き評価失敗」にする方針は、AE型の根拠つき文書化と§9のNuke型素通し警告が既知の形。

**埋まらない残り**: 上限を置かない規範と、実機で必ず存在する資源限界の**見せ方**(いつ・どの語彙で縮退/失敗を提示するか)は先例が慣習止まりで、Motoliiの型付きエラー文化に合わせて自作する部分。M3E-2の性能ハーネスやmemory-modelの疑念台帳が受け皿。

## 9. Preview縮退と作品の意味の分離

**問い**: 「Previewは密度・解像度・サンプル・更新頻度を縮退できるが、Documentの個数や意味を書き換えない」(§7.3)の先例と反例。

**既知の解(支持側)**:

- Blenderの[Simplify](https://docs.blender.org/manual/en/latest/render/cycles/render_settings/simplify.html)は**Viewport節とRender節を分離**し、同じ項目をビューポート用と最終レンダー用の独立値として持つ。Child Particlesは「全child particleの一部だけを表示する」割合指定 — シーンデータ(パーティクル設定)の書き換えとしては記述されない、表示側の密度縮退。
- AEの[Fast Previews / Adaptive Resolution](https://helpx.adobe.com/after-effects/using/previewing.html)は操作中だけダウンサンプルし、**縮退中はComposition view隅にモード名を表示**(状態の可視化)。同ページはRegion of Interestが「既定ではファイル出力に影響しない」こと、Preview解像度がコンポジション設定を上書きするだけで書き換えないことも明記 — preview系の値が保存意味へ漏れない構造。
- 縮退できない処理の扱い(Nuke): 公式は[proxy mode](https://learn.foundry.com/nuke/11.1/content/getting_started/managing_scripts/proxy_mode.html)の結果を「同一(または少なくとも非常に近い)合成」としか表現せず**完全一致を保証しない**。さらに解像度依存のF_ReGrainは「proxy解像度でのグレイン操作は信頼できない」ため、**proxyでは動作せず警告して画像を素通しする**([公式](https://learn.foundry.com/nuke/12.1/content/furnacecore/proxy_resolutions.html))。劣化した結果を黙って出すより、識別可能な非適用を選ぶ — §7.3「資源不足は黙った削減ではなく識別可能な縮退または型付き失敗」と同じ判断が公式に文書化されている。
- 時間軸の縮退禁止: [Fix Your Timestep(Glenn Fiedler本人記事)](https://gafferongames.com/post/fix_your_timestep/)は、dtを描画レートへ追従させると結果がフレームレート依存になり非決定になることを示し、固定dt+描画分離+補間を解として確立した。simulation-modelの固定step・可変dt禁止・StateTrack Quality非依存は、この定石のDCC翻訳として既決。

**既知の解(反例側)**: Unreal Niagaraの[scalability](https://dev.epicgames.com/documentation/unreal-engine/scalability-and-best-practices-for-niagara)は品質レベルとプラットフォームで**spawn数のスケールダウン、インスタンスのcull、emitter/system自体の無効化**を行うと公式に明記([Spawn Count Scale、Max Distance、Cull Reaction等の設定名まで公式KBにある](https://dev.epicgames.com/community/learning/knowledge-base/LJnb/unreal-engine-niagara-scalability-effect-types))。ゲームのフレーム予算では正当な設計だが、「品質設定で作品の中身が変わる」側の実例であり、MotoliiがPreviewへ持ち込んではならない構造の名指しに使える。

**Motoliiへの翻訳**: 縮退可能軸と不可侵軸の対応表が先例から直接引ける。

| 軸 | Preview縮退 | 根拠先例 |
|---|---|---|
| 解像度・テクスチャサイズ | 可(表示側の別管理値) | Blender Simplify、AE Adaptive Resolution |
| 描画密度(個体の描画間引き) | 可。ただし縮退中の可視化必須 | Blender Child Particles %、AEモード表示 |
| 更新頻度・サンプル数 | 可(Quality既決) | AE Fast Previews |
| sim step・seed・軌道 | **不可**(StateTrack Quality非依存、既決) | Fix Your Timestep、反例=Niagara |
| Documentの個数・意味 | **不可** | 反例=Niagara Spawn Count Scale |
| 縮退不能な処理 | 黙って近似せず、識別可能に非適用/失敗 | Nuke F_ReGrain |

**埋まらない残り**: 先例の縮退は製品ごとの慣習(設定パネルの節分け)であり、**「このpluginはどの軸なら意味を変えずに縮退できるか」を型付き契約(NodeDesc等)として宣言させた例は未確認**。Motoliiが契約化するなら自作になるが、これは§7.3の「具体方式は性能fixtureとM5-P0I spikeを経て選ぶ」の範囲内で、今決めない。

## 10. 遊びを生む段階の観察と、早期固定しない停止線

**問い**: §9の「楽しさを先例の多数決でなく探索行動で観察する」「demo一つから万能node graph/汎用expressionを固定しない」に、方法論の先例はあるか。

**既知の解(観察の方法論)**:

- Resnick & Silverman ([IDC 2005](https://web.media.mit.edu/~mres/papers/IDC-2005.pdf))は構築キットを「特定の活動の集合ではなく探索すべき空間」とみなし、**成果物の多様性(diversity of outcomes)を成功指標**とする — 作品が似通えば失敗、1作で終わっても失敗。さらに「作り手が想像しなかった使途」への驚きを設計目標として明記する。low floor / high ceilingへ**wide walls**を追加したのも同論文系([CACM 2009](https://web.media.mit.edu/~mres/scratch/cacm-scratch-09.pdf)はlow floor/high ceilingをPapertへ帰属)。§9.2の「説明されていない組合せを自発的に試すか」はdiversity-of-outcomes型の指標に翻訳できる。
- [Designing for Tinkerability (Resnick & Rosenbaum 2013)](https://web.media.mit.edu/~mres/papers/designing-for-tinkerability.pdf)はtinkerabilityの設計原則を**immediate feedback / fluid experimentation / open exploration**の3点に定式化する。extensible-core §2.1の学習循環(触る→結果が見える→戻せる→別の値を試す)と同型。
- Will Wrightは設計対象を「プレイヤーが後に探索するpossibility space」と定式化し([Accelerating Change 2004講演、公式説明文のアーカイブ](https://web.archive.org/web/20130724222658/http://itc.conversationsnetwork.org/shows/detail376.html)、[GDC 2003 Dynamics for Designers](https://www.gdcvault.com/play/1019938/Dynamics-for))、toyはgameよりopen-endedで多様な遊び方を生むと述べる(書籍インタビュー)。「穴を閉じる=機能リスト」から「遊びを生む=可能性空間」への§9の転換は、この語彙で説明できる。
- 観察が製品を変えた一次事例: Media Molecule公式ブログは、**専用logic部品が無い初代LittleBigPlanetでプレイヤーが物理部品の組合せから8-bit計算機を作った事例**に言及し、LBP2のadvanced logic circuitsでの再現デモを掲載する([公式ブログ](https://www.mediamolecule.com/blog/article/emails_from_the_molecule_can_you_make_a_calculator_with_lbp2_logic))。ユーザーの想定外工作が観測され、後継で第一級機能になった時系列は、extensible-core §6の昇格パイプライン(user工作→観測→Host引き取り)の遊び版そのもの(因果の公式明言は無い、§13)。任天堂側でも、World 1-1の「最初のGoombaとキノコで教える仕掛け」が当初設計でなく開発中のtrial and errorから生まれたことを宮本茂が[Iwata Asks](https://iwataasks.nintendo.com/interviews/wii/nsmb/0/3/)で明言している — 「触って分かる」は事前設計でなく観察と反復の産物。
- Dreamsの現行形: 公式ユーザーガイド上、ロジック構築手段は[gadget](https://docs.indreams.me/en/create/resources/edit-mode-guide/assembly/gadgets)(Sensors/Logic/Movers等7カテゴリ+wire+Microchip)のみで、テキストscripting面は存在しない。汎用言語を開かずに部品意味論で統制した現行例(排除理由の公式明言は未特定、§13)。開発過程はGDC 2021公式セッションの通りuser researchで反復([GDC Vault](https://www.gdcvault.com/play/1026982/UX-Summit-Expanding-the-Dreamiverse))。

**既知の解(早期固定の害の語彙)**:

- [Cognitive Dimensions of Notations (Green & Petre 1996)](https://web.engr.oregonstate.edu/~burnett/CS589and584/CS589-papers/CogDimsPaper.pdf)は「早期固定の害」を**premature commitment**(情報が揃う前に決定を強制されるか)として名指しし、viscosity(1変更の労力)が高い環境では「壊滅的になりうる」と分析する。**progressive evaluation**(未完成でも実行してfeedbackを得られるか)も同枠組みの次元。§9.3の停止線は「Motolii自身の設計プロセスに対するpremature commitment回避」、§2.1の学習循環は利用者へのprogressive evaluation提供、と翻訳できる。同論文はLabVIEWが「決定の先送り」と「変更の容易化」でpremature commitmentを部分解決した経緯も記録しており、対策の型(先送り+低viscosity)まで先例がある。
- 汎用expressionを早期に開いた側の保守コスト(AE): 公式ヘルプが、expressionがレイヤー名/プロパティ名に依存し**rename時の自動更新は「複雑な場合には失敗し、ユーザーが自分で修正する」**こと([Expression errors](https://helpx.adobe.com/after-effects/using/troubleshooting-expressions.html))、エンジン移行(ExtendScript→JavaScript, [ECMAScript 3→2018](https://helpx.adobe.com/after-effects/using/legacy-and-extend-script-engine.html))で**既存expressionの書き直しが必要になり得る**ことを明記する。名前ベース参照と汎用言語エンジンをDocument意味へ焼いた帰結が、公式文書に保守作業として現れている。
- 統制した側: Houdini [VEX](https://www.sidefx.com/docs/houdini/vex/lang.html)は型付きで「contextが利用可能な関数・変数を規定」し、[wrangle snippet](https://www.sidefx.com/docs/houdini/vex/snippets.html)は流入ジオメトリのattributeへ`@`束縛される(名前文字列でシーンを横断しない)。Cavalryは[sceneを触れる`api`をJavaScript Editor専用に隔離し、composition内のJavaScript Layersには計算用`ctx`だけを渡し](https://docs.cavalry.scenegroup.co/tech-info/scripting/api-module/)、参照値は[UIで型を選んで明示input化させる](https://docs.cavalry.scenegroup.co/nodes/general/javascript-layers/)。「言語を開くなら、能力を分離し参照を型付き入力に限る」— extensible-core §5のcapability分離と同じ構造が独立に2実装ある。

**Motoliiへの翻訳**: §9.2の記録項目は、次の既存語彙で観察設計へ落とせる(採用は運用文書側で判断):

| §9.2の記録項目 | 対応する既知の観察語彙 |
|---|---|
| 最初に触った対象と次の操作 | immediate feedback / fluid experimentation (tinkerability) |
| 嬉しい/不快な意外性 | 作り手が想像しなかった使途への驚き(IDC 2005)、emergence (Wright) |
| 壊れる恐れと回復路 | viscosity、progressive evaluation (CD) |
| どの段(§3)を操作していると理解したか | abstraction gradient (CD) |
| 意味を変えない品質縮退 | §9(内部規律) |
| 別の遊びへの合成可能性 | wide walls、diversity of outcomes |

停止線(§9.3)は「demoの成功→汎用機構の固定」を禁じる点でpremature commitment管理と同型であり、LabVIEW型の対策(決定の先送り+変更を安くする)とDreams型の対策(汎用言語を開かず部品意味論で統制)の2系統が既知。Motoliiは前者を凍結ゲート/解凍手続きとして既に制度化しており、後者はcapability分離(§5)として明文化済み — つまり停止線の実装機構は新設不要。

**埋まらない残り**: 観察の**語彙と指標**は移植できるが、**「Motoliiで何が楽しいか」の判定そのものは移植できない**(Resnickの指標は教育文脈、WrightとLBPはゲーム文脈由来。転移条件の検討が規律2の対象)。§9.1の候補リストをprobeとして実際に観察する作業はMotolii固有で、ここが「既知の組み換えでは埋まらない」核心。

## 11. どこが本当に未知か(組み換えで埋まらない残り)

発端の仮説「これらの問題は既知の要素の組み換えだけでは単純にできない」への回答。**機構レベルは予想より広く既知で埋まる**。identity、選択の外部保存、宣言的介入、集合所有の状態、上限非焼き込み、縮退の分離は、いずれも複数の成熟実装で意味が収束しており、Motoliiの原則はその収束点の再記述に近い(=§7の不変条件は先例に支えられ、独自リスクは低い)。

埋まらない残りは次の4点に絞られる。

1. **介入の正本の逆転**(§6): 先例は個体側データ(ジオメトリ属性)が正本。個体を保存しないまま「介入だけをDocumentへ保存し、毎評価で動的identityへ再結合し、欠落を参照待ちとして扱う」構造は、Exclude系のUSD 2形を除き直接先例が無い。特にPin/Impulse型の値介入と、時刻付きone-shot介入の永続保存。
2. **四段を一つの利用者文法にすること**(§3): 段は全部先例にあるが、段の区別を専門知識でなくUIの言語として提示した製品は未確認。AM実機観察も描画個体で止まり、個別選択・固定・除外・物理介入へ進む導線は確認できなかった。
3. **縮退可能軸の契約化**(§9): 先例は慣習止まり。型付き宣言にするかは性能fixture後の判断。
4. **遊びの判定**(§10): 観察の語彙・指標・停止線の型は既知だが、判定はMotolii上の観察でしか得られない。

つまり「単純にできない」のは機構の発明ではなく、**(1)の保存契約の設計と、(2)(4)の体験設計・観察運用**である。1は反例試験(identity再生成をまたぐ介入の追従fixture)を作れる種類の問題で、P0I spikeの検証対象に足せる。2と4はprototypeと観察の対象で、先に公開型を固定しない現行方針が正しい帰結になる。

## 12. 反対側レビューへの依頼と実機確認結果

反対側レビュー(規律2)で特に検証してほしい点:

1. **転移条件**: Resnick/Wright/LBPの観察指標は教育・ゲーム文脈由来。MVツールの利用者(納期のある制作者を含む)へ同じ指標を使ってよいか。
2. **帰属**: LBP計算機→LBP2 logic回路の因果は公式ブログの時系列のみで、Media Molecule自身の因果明言は未確認。昇格パイプラインの例証として使う場合は「整合する事例」止まりにすること。
3. **反例探索**: 「介入を集合側へ保存する」構造の失敗例(参照が腐って使い物にならなくなった製品)を探すこと。本書は成功側の収束だけを集めており、規律3の通り反例未探索。
4. **より小さい対策**: §6の「参照待ち/欠落」はplugin欠落の既存パターン転用で足りるか、介入専用の新しい診断語彙が必要か。
5. **一次資料の穴**: MoGraph Selectionの保存基盤(identityか配列順か)。クローン数変更で選択がズレるなら、§5の先例は「反面事例」側へ移る。

Alight Motionの実機確認は2026-07-17にユーザーが実施した。結果は§5の通り、パーティクル/リピート系は描画上の個体に留まり、評価個体の個別選択・固定・除外、物理状態、物理介入は通常導線で確認できなかった。したがって現時点ではAMを§5〜§6の直接先例にせず、**親しみやすい描画UIだけでは個体性の遊びへ到達しない反面観察**として置く。将来の版・隠れた機能・公式資料で反例が見つかれば再判定する。

## 13. 主要な未確認事項(推測で埋めていない点)

- Houdini POPの`id`採番アルゴリズムの詳細(単調カウンタか等)は公式記述を発見できず。
- Newtonがbakeする具体プロパティ名(position/rotation等)の公式明示は未発見。
- Particularの「個体介入機能が無い」ことは公式マニュアルの制御面の消極的確認であり、悉皆証明は不可能。
- AEの30,000px上限の導入/引き上げバージョン履歴は一次資料未確認。
- MIDI 2.0仕様書本体(M2-115等)は会員限定のため、数値はMIDI Association公式記事で確認。
- Dreamsが「テキストscriptingを意図的に排した理由」の公式明言は未特定(ガイド上の不在は確認済み)。
- AE新expressionエンジンの「V8」という固有名はAdobe公式ページでは未確認(「ECMAScript 2018ベース」まで)。
- Blender Simplifyについて「元データを変更しない」という明文はマニュアルに無く、Viewport/Render節の分離構造からの帰結として読んでいる。
- USDの個数上限の扱いは未調査(glTFで代替確認)。
- Alight Motionの確認はユーザーによる実機観察であり、版番号・全エフェクトの悉皆確認・公式仕様による機能不在の証明ではない。
