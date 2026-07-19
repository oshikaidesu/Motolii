# リリックモーション: Text Sequence / Materialize 比較台帳(2026-07-19)

日付: 2026-07-19
状態: **比較中**。本書は設計決定でも実装許可でもない。2026-07-19のGrok反対側レビューは完了し、判定`REVISE MEMO BEFORE FIXTURE`への主担当補正を§7まで反映した。fixture比較を経るまで、いかなる項目も設計根拠・公約にしない([レビュー規律6点](README.md))。

## 0. 経緯と目的

Plugin Browser検討中の`Type Pulse`(文字ごと登場エフェクト)を起点に、ボカロMV型リリックモーションの要求(文字の直接配置・個別タイミング・後編集)を2026-07-19のエージェント対話(Codex/Fable)で往復整理した。本書はその結論の固定ではなく、**確度が高い整理**と**比較中の選択肢**を分離して保存し、AM実機観察・反対側レビューを経て、次の審判(fixture比較)への入力にする。

併読正本:

- [text-model.md](../text-model.md)(ドラフト) — selector + properties のアニメーター骨格
- [extensible-core-model.md](../extensible-core-model.md)(設計原則) — §4 Authoring Tool責任寿命、§7 個体4段、§8 外殻/payload分離
- [モーション4ツール監査](2026-07-14-motion-tools-praise-diy-gap-audit.md) G2/G4、[既知技術処分決定](2026-07-14-motion-foundation-known-tech-disposition.md)(Element Domain=P0I spike、Materialize詳細未固定)
- [反復再発明監査](2026-07-14-repeated-wheel-standardization-audit.md) R1(Timing Rail)/R3(Text addressability)
- [Relative Move / Duplicator決定](2026-07-15-relative-scope-duplicator-decision.md)(D2 macro、seed規約)
- [ui-score-model.md](../ui-score-model.md)(設計決定) — 譜面=投影、Laneを所有者にしない。ParticleのDepth投影をMaterialize後へ限定する既決文と、§2.7の高数量domainへの一般化が衝突する(§2.7参照)

### 0.1 守る製品命題(中立な比較軸ではない)

本比較の中心は文字分解方式の選定でも、高機能なText Animatorの追加でもない。**Textの本文・組版・一括評価を保ったまま、addressable characterをStage上の直接操作とTimeline上の評価済み時間へ同時に露出できるか**を審判する。Materializeはこの体験を代替せず、Text外の所有・接続・Effect・生存期間が必要な場合の出口である。

次の4条件はLive Text仮説を採る場合の製品命題であり、全方式を同格に並べる中立な比較軸ではない。今後の実装案は、少なくとも次の4条件を同時に満たさなければ**同等な製品案**として扱わない。一方、条件を満たさない破壊的Splitも、既存方式との作業量・保持意味・性能を比較する基準/負例としてfixtureから除外しない。

| 不変条件 | 失ってはならない意味 |
|---|---|
| **LIVE TEXT** | 本文、font、style、字間、改行、行組を後から変更できる |
| **DIRECT CHARACTERS** | Materializeせず、Stage上で文字候補を選択・移動・回転・拡縮できる |
| **VISIBLE TIME** | 制御値だけでなく、各文字候補が実際にいつ動くかをTimelineで読める |
| **OPTIONAL DETACH** | Object化は別Effect、外部接続、別所有、独立生存期間等が必要になった時だけ明示する |

短い製品命題は **「Textを壊さず、文字を触り、文字の時間を読む」** とする。名称、保存形式、公開API、v1の値集合をこの命題から直接決めない。

## 1. 新しい証拠は「方式」ではなく「優先度」

リリック用途の市販ツールが独立に複数成立している。

- Texton(After Effects): テキストを1文字ずつレイヤー化し、入退場モーションを一括生成 — [BOOTH 7838882](https://booth.pm/ja/items/7838882)(等級: 市場=製品ページ)
- Aulymo(After Effects): 既存の複数レイヤーへ位置・時間差・ランダム・入退場を一括適用 — [BOOTH 6403113](https://booth.pm/ja/items/6403113)(等級: 市場)
- AE系: [TextExploder](https://aescripts.com/textexploder/)・[DecomposeText](https://aescripts.com/decomposetext/)等の文字分解スクリプト反復(R3で登録済み)
- AviUtl標準文化: 「文字毎に個別オブジェクト」+ さつき氏TA系スクリプトでの文字アンカー直接ドラッグ(等級: 市場・コミュニティ)

これらが証明するのは「Materialize(レイヤー分解)方式が正解」ではない。R3が指摘済みのとおり、分解の流行は**弱いアドレス可能性への回避策**を含む。証明されたのは次の1点である。

> リリックMVでは、順次アニメーションだけでなく、分解・直接配置・個別タイミング・後編集までが、有料ツールが複数成立するほど高頻度である。**G4 Materializeを「後段」へ置いた優先度判断は再審理に値する。**

さらに、AE第三者ツールのLayer分解は需要への回答であると同時に、Host拡張面の制約を受けた回答だった可能性が高い。AE scriptingは`TextDocument`、文字range、Layer property、keyframe等を操作できるが、第三者panelがAE本体のStage hit-test、Text内部の編集handle、Timelineの評価結果laneそのものを新設できる公開面は確認できていない。したがって市場の分解ツールを「利用者が独立Layerを本質的に望んだ証拠」とだけ読まず、**本当はText内部の個体を触りたかったが、公開されたLayer / Property / Keyframeへ正規化せざるを得なかった痕跡**という解釈も比較へ残す。ただしAPI不在から作者の意図までは証明できないため、方式決定の根拠にはしない。

## 2. 確度が高い整理(ただし決定ではない)

### 2.1 Text Sequence = text-model animatorのUI翻訳

「文字ごと登場」というSequence評価自体は新しい評価機構ではなく、タイポライターの一般化であり、[text-model.md](../text-model.md) §2の`animators: [{selector, properties}]`が該当する。

| UI概念 | text-model既存構造 |
|---|---|
| WHAT: 状態差(どこから来る・回る・潰れる・透明) | `properties: {position, rotation, scale, opacity, 軸Δ}`(各ParamSource) |
| WHERE: Characters / Words / Lines / Range | `selector: {unit: クラスタ\|単語\|行, start, end}` |
| WHEN: Order / Offset / Duration / Stagger / Ease | `selector.offset/shape/seed` + ParamSourceのkey・AM式easing |
| タイポライター | weightが0/1(矩形shape)+Opacityのみの最小ケース |
| カラオケワイプ | `selector.offset`の時間駆動(text-model §2に明記済み) |

ここで新しいselector評価器を発明しない。既存評価器契約(motolii-eval)のままSequenceを表現できるかを先に審判する。単位は書記素クラスタ(P6クラスタ対応表)。UI名称は「Text Animator」より範囲の狭い`Text Sequence`系が候補(名称は未決)。

ただし、Live Splitのaddressable cluster、疎な手動介入の所有、RESULTからの逆編集は既存text-modelに無い新しいDocument/UI意味の候補である。**Sequence評価が既存骨格に収まることと、Live Split全体が新機構なしであることを同一視しない**。後者は§3.2/§3.3の恒久面審判を要する。

### 2.2 Ghost Pose: 登場前状態のStage直接編集

パネル入力(`Direction: Bottom / Distance: 0.35`)だけではプリセット生成器で終わる。登場前状態のghostをStageへ表示し、直接drag・回転・縮小で「状態差」を作らせる(G8直接操作)。先に固定すべき規則:

- **書き込み先は選択中Animator単体**。stack(並び順逐次+加算合成、text-model §2.2)の中でどれがACTIVEかを常時表示する
- ghostは選択Animator単体の絵ではなく、**前後のAnimatorを含む最終合成結果**として描く(「ghostで見た位置」と「確定後の位置」の乖離を作らない)
- dragから選択Animatorの差分へ**逆変換できる場合のみ**直接編集可能。不能なら理由付きでInspector編集へ戻す(G1「Canvas操作可能なものは逆変換を宣言」の適用例)
- プリセットは完成品ではなく編集開始地点(適用直後からghostを掴める)

### 2.3 Materialize(Split Text)はText外の所有が必要な時の明示的な出口

Text Sequenceの規則で配る差だけでなく、文字候補ごとの手配置、個別Timing、Position channelも、まずLive Textのまま成立する案を審判する。**個別軌跡や個別Easingが必要という理由だけで直ちにMaterializeへ送る案は、§0.1の製品命題を満たさない縮退案**である。Materialize候補は、別Effect stack、外部接続、別Group、独立した所有・生存期間、Textでないpath/shape化など、Text外の意味が必要になった場合に限る。

契約形はG4(明示コマンド・1 Undo・保持宣言・隠れコピー禁止)とR3の条件4点(組版位置保持・隠れ全文コピー禁止・既定でGroupを増やさない※比較中・1コマンド=1履歴)に従う。Illustratorのアウトライン化が「凍結の明示」のユーザー既知先例。

実装面はextensible-core §4.1(snapshot → typed command batch → preflight → 1 macro)が既にあり、新しいプラグイン分類・公開traitは不要。v1はfirst-party限定(P0I前の公開trait禁止の停止条件に従う)。

### 2.4 Railの名称分離

「Timing Rail」をR1の一回性Host Toolへ予約し、Text Sequence側は別名にする。

```text
Timing Rail(R1)
  Transient → D2 macro → 普通の時刻値を焼く。DocumentにRailを保存しない

Sequence Timing Lane(Text Sequence内)
  Persistent projection → Stagger/Overrideパラメータの常時投影 → 評価時刻
  (Staggerを2f→3fへ変えると非override文字がライブで動く)
```

操作文法(ノード・範囲・Distribute・Reverse・Randomize+seed)は共有してよいが、所有と確定結果は別(R1「操作文法だけ共有し、同じ永続型へ抽象化しない」の適用)。UI表記は`CHARACTER TIMING`/`SEQUENCE TIMING`等。「Stagger」単独名称を避け、対象を明記する: `Distribute Object Starts` / `Distribute Selected Keys` / `Character Interval`。

### 2.5 Materialize / Map / Order / Stagger / Fold の解像度

5つを同列の基盤プリミティブにしない。

| 候補 | 扱い |
|---|---|
| Order / Stagger | R1のとおりTextに閉じないHost共通候補(市場証拠がレイヤー汎用) |
| Materialize | G4契約形の共通化。identity規則はdomain別(§4参照) |
| Map | 各Tool内部の型付きfan-outパターン。汎用property-path APIにしない(文字列式への回帰を防ぐ) |
| Fold | Group(所有)とHost所有instance列の性質。独立した擬似所有概念を新設しない(R2) |

### 2.6 制御値と評価結果を同じ時間面へ投影する

タイポライター型の機能が理解しにくい原因は、Timelineに`Progress 0→100`等の**生成規則の制御keyだけ**が見え、実際に各文字が登場する評価済み時刻がEffect内部へ隠れることにある。Text Sequenceは入力と結果を二段で表示する候補とする。

```text
CONTROL
Progress     ◆────────────────◆

RESULT
Characters  夜◆──を◆──走◆──る◆
```

下段は4つの独立Objectや保存keyを意味しない。`Order + Interval + Duration + Overrides`を現在の本文へ評価した結果の投影であり、本文・順序・Interval変更時にライブ更新する。文字nodeの選択はStage上の同じ文字候補と同期し、node dragやspan端dragを許す場合は、生成規則を黙って絶対key群へ置換せず、採用済みの明示overrideへ逆変換できる時だけ編集する。

逆変換不能時の閉集合は`(a) 理由付きで操作を拒否する`または`(b) Inspectorで書き込み先を明示選択して再試行する`までとする。絶対key群への暗黙Bake、Timeline固有の保存状態、UI-only overrideを第三の出口にしない。

この原則はG6 Evaluation VisibilityのText上の具体例である。Duplicator instance、音声連動、particle emission等にも同型の必要があり得るが、本書から共通Timeline schemaや全domain共通laneを作らない。まずText fixtureで「制御値だけ」「結果だけ」「二段併記」を比較する。

### 2.7 上位仮説: 評価結果をObject化せず編集面へ戻す

Textで現れた製品仮説は、`Split Text`固有の改善より広い。

```text
Semantic Source
Text / Cloner / Generator / Analysis
            ↓ evaluate
Addressable Evaluated Elements
character / instance / point / event
            ↓ project
Stage / Timeline / Inspector
            ↓ user intent
Typed Intervention
Pin / Offset / Retiming / Exclude / Restyle
            ↓
Sourceへ戻って再評価
```

従来の「編集可能なもの=Document Object」という一致を外し、**編集可能であることと、Document Objectであることを分離する**。Documentへ保存する候補は生Objectの山ではなく、Semantic Source、domain固有identity規則、型付きの疎な介入である。Timelineも保存Objectを縦に並べる場所だけでなく、手続きが生成した内部時間を評価済みの譜面として開く場所になる。

この一般化自体は[extensible-core-model.md](../extensible-core-model.md) §7のアドレス可能な個体と宣言的介入に既に含まれる。今回新しく製品面へ現れたのは、次の結合である。

| 責任 | Textでの最初の実証候補 |
|---|---|
| 意味の正本 | Live Text |
| 空間上の投影と逆操作 | Character handles |
| 時間上の評価結果と逆操作 | Character score / Sequence Timing Lane |
| 完全独立の出口 | Detach |

共通化する候補はidentity、選択、空間投影、時間投影、逆変換可否、Detach可否という**外殻capability**であり、文字、clone、beat、particleを単一Element意味へ畳まない。identity寿命、介入payload、合成、欠落時挙動はdomain側に残す。[extensible-core-model.md](../extensible-core-model.md)は既にHost管理面へ時間を含め、§8.1で個体の存在時刻、§8.3で`time_scope`、§9.1で時間上の介入を候補にしている。一方、§8.2の能力候補(Queryable / Addressable / Selectable / Intervenable / Materializable / Stateful)には、**評価された個体別時間を譜面へ投影し、dragを生成規則または疎な介入へ逆変換する能力**が明示されていない。本仮説の新しい1軸は時間そのものではなく、この`temporal projection + inverse edit`である。

進める順序も固定する。まずTextで4不変条件と本文編集後の和解を成立させ、次にClonerで同じ外殻が成立するかを反証し、その両方で残った共通部だけをHost capability候補へ昇格する。Text fixture前に全domain共通schema、公開`Element` trait、汎用property path、共通Timeline laneを作らない。

**既決との衝突範囲(要審判)**: [ui-score-model.md](../ui-score-model.md)(2026-07-17設計決定)§3は、Depth RailへParticleを投影する文脈で「Particle個体をmarker化せず、Emitter/生成元の安定IDだけを選択・keying対象にする」「**個体編集はMaterialize後の通常Objectへ限定する**」と定めている。これはTextClusterを扱うLive TextまでMaterialize限定にした全domain決定ではないため、Text fixture自体は直ちに同書を上書きしない。一方、本仮説をCloner / Particle等の高数量domainへ昇格する段階では後半と正面衝突する。黙って一般化せず、次のいずれかで審判する。

1. Text fixtureは現行決定と両立するdomain固有prototypeとして進め、Cloner反証ではParticle決定を維持する
2. Cloner反証まで通った後、同文を高数量domainの規模則へ縮小し、「評価個体の投影・逆操作は明示展開・選択scopeのopt-inに限る(全個体の常時lane/marker化は引き続き拒否)」とui-score-modelを正式改訂する

いずれの場合も「個体数に比例してmarker・lane・Document項目が増えない」(同書回帰審判14)は不変条件として保持する。Live Splitが保存するのは疎な介入のみであり、覆すのは編集可能性のMaterialize限定であって規模則ではない。

### 2.8 UI仮説: 親で全体のルールを決め、子で個別に直す

利用者へaddressable element、selector、intervention等の内部語彙を要求しない。Textでは、親の`IN MOTION`を演出規則、展開した文字候補を評価結果と個別差分の入口として見せる。

```text
▼ T「夜を走る」
    IN MOTION       From / Scatter / Order / Interval / Duration / Ease / Seed
    夜              generated timing + sparse overrides
    を              generated timing + sparse overrides
    走              generated timing + sparse overrides
    る              generated timing + sparse overrides
```

子行は独立Objectではない。親の規則から導出したbarを表示し、dragは採用済みの型付き差分へ逆変換する。候補は`Start Offset`、`Duration Scale`、Text-local transform差分等であり、親のInterval、Order、Duration等を変更しても差分を維持する。`Reset to Sequence`は該当する疎な差分を削除し、親の自動値へ戻す。絶対key群へ黙って焼かない。

ただし、UI上の親子順と評価順を混同しない。文字別Font、Font Size、Color等はTextのstyle rangeであり、motion overrideではない。評価は次の順になる。

```text
本文 + style range(Font / Font Size / Color)
        ↓ shape・改行・行組・addressable cluster候補
親のIN MOTION規則
        ↓ generated pose / timing
子文字のmotion差分(Position / Visual Scale / Rotation / Timing)
        ↓ final glyph projection
```

- `Font Size`は組版を変え、後続文字位置とcluster対応へ影響し得る
- `Visual Scale`は組版後のglyph transformで、advanceと改行を変えない
- style変更でshape / cluster対応が変わった場合も、別文字へ介入を黙って移さず§3.2の和解へ送る
- 表示/非表示は「advanceを保ってglyphだけ隠す」「本文から除いて再行組する」で意味が異なるため、単一の曖昧なtoggleとして採用しない。前者はOpacity / glyph visibility、後者は本文編集として区別する

`Font Size`(組版)と`Visual Scale`(見た目)の分離は、[text-model.md](../text-model.md) §4が積み残した未決「『見かけのサイズ』(アニメ)と『組版のサイズ』(スタイル)の区別のUI文言反映(未決、M3)」への最初の回答候補である。採用判定時はtext-model §4の未決欄へ反映する。

なお「子のmotion差分」1箱には評価上の非対称が隠れている。Position / Visual Scale / Rotation差分は生成済みposeへの**出力側の加算**だが、`Start Offset` / `Duration Scale`は親規則の**評価入力**(そのclusterに対するselector時間)を書き換える。text-model §2.2はanimatorの複数枚stack(並び順逐次+加算)を許すため、timing差分が全stackへ共通に効くのか、対象Sequence 1枚だけに効くのかは未決である。§2.2「Ghost Poseは選択中Animator単体へ書く」との整合もここで問われる。

**STOP**: timing差分の対象scopeと書き込み先を宣言していないprototypeでは、RESULT node/spanの編集を有効にしない。fixture 14では`対象Sequence単体`と`全stack共通`を別prototype意味として明示し、同じ保存値を両義に解釈しない。最終合成ghostから選択Animatorへの逆変換が一意でない時も、§2.6の閉集合へ戻す。

この節はUIと評価順の比較仮説であり、`IN MOTION`という名称、control一覧、override fieldを決定しない。

## 3. 比較中の選択肢

### 3.1 同等候補と比較基準を分ける

- **製品仮説 / Live Text + Optional Detach**: Text内部のaddressable clusterをStage/Timelineへ投影し、通常制作は4製品命題を保つ。Text外の所有・Effect・接続・生存期間が必要な時だけ明示Detachする(未採用)
- **比較基準 / Destructive Split**: 文字を独立Objectへ変換し本文を凍結する既存方式。4製品命題を満たさないため同等候補ではないが、作業手数、保持意味、描画、Timeline密度のbaseline/負例として比較する
- **縮退基準 / Rule-only Text Animator**: 規則的なSequenceだけをText内に保ち、個別調整はSplitへ送る。既存text-modelで閉じる最小案だが、DIRECT CHARACTERSとVISIBLE TIMEの要求をどこまで失うか比較する

従来のA/B/C表記は三方式が同格に見えるため廃止する。DetachはLive Textと別の第三方式ではなく、その製品仮説に含まれる明示出口として扱う。

### 3.2 文字別Timing Override(恒久スキーマ候補 — 未採用)

「走だけ早く」等の要求に対し、Sequence内へ疎な差分(`Start Offset` / `Duration Scale`のみ、絶対時刻を保存しない)を持つ案が出たが、**本対話で初めて出た恒久スキーマ候補であり採用しない**。本文編集後の和解規則が本体で、少なくとも4案を比較する。

1. Text編集正本へ安定した書記素単位IDを持つ(§3.2.1)。shaping出力のcluster IDを保存する案とは分け、Document恒久面の追加として要審判
2. 元index+前後文脈を保存し、文字列diffで和解する(アンカー誤接続の検証が必要)
3. Source Text変更時に全Overrideを無効化し再確認させる(最も単純・最も失う)
4. 個別Timingへ踏み込む時点でMaterializeする(override store自体を持たない。ただし§0.1を満たさない縮退比較であり、同等案ではない)

`Needs Review`案も自明ではない: 保存状態として持つのか、元文字列hashと現在文字列からの導出診断にするのかで恒久面が変わる。共通の停止線: 挿入・削除周辺のoverrideを**別文字へ黙って転用しない**(extensible-core §2.2「自動修復で黙って変えず候補を提示」)。

**最初のfixture gate**は保存方式の採択ではなく、4候補を非永続prototypeで比較し、反復文字・cluster再形成・style変更後の**黙った誤接続0件**を満たせるか反証することである。誤接続を避けるため`Needs Review`や明示Resetへ落とすことは失敗ではなく、別文字へ自動転用した時だけ即失格とする。このgate通過前に安定書記素ID、override store、Needs Review状態をDocumentへ追加しない。

当初案はv1の値集合を`{Start Offset, Duration Scale}`へ限定し、文字別Position等をMaterializeへ送る防火壁だった。§3.3のLive Split案は、本文を生かしたまま直接配置する中核要求のため、この防火壁を`local Position / Scale / Rotation`まで広げる提案である。どちらも未採用で、fixture前にDocument fieldを足さない。

#### 3.2.1 文字数変更に対するidentity候補

Live Textを本気で保持する候補として、Textの編集正本を単なるStringだけでなく、編集をまたいで継承される書記素単位ID列として扱う案が出た。

```text
夜  id:A
を  id:B
走  id:C
る  id:D

「夜を走る」→「夜道を走る」

夜  id:A  override保持
道  id:E  新規・親SequenceのAuto値
を  id:B  override保持
走  id:C  override保持
る  id:D  override保持
```

候補規則:

- 挿入は新ID、変更されていない前後の書記素はIDを保持する
- 削除は同じText edit commandで対象IDと介入を除き、Undoで本文・ID・介入を一緒に戻す。別の同字へ介入を移さない
- 置換は既定で旧ID削除+新ID作成。旧文字に介入がある場合だけ、明示的な引継ぎ候補を提示できるが自動継承しない
- 全文paste等で対応不能なら、保持/対応不明/削除の件数をCommit前に示し、確認、Reset、Cancelへ分ける
- Positionは`layout_position(target) + text_local_offset`候補とし、本文・font・字間・改行後も絶対Stage座標へ取り残さない

ただし、この案で保存するidentityは**編集正本の書記素**であり、shape結果のcluster/glyphではない。合字、結合文字、正規化、font fallback、ルビでは複数書記素IDが一つのshaped clusterへ写る、または対応が再形成され得る。個別transformのtargetをID単体/ID集合/clusterのどれにするか、合字を保ったまま個体編集できない場合の拒否/分解、IME composition中とCommit後のID発行は未決である。ID列を入れただけでP6 cluster対応を解決済みにしない。

現行[text-model.md](../text-model.md) §2は`content: 文字列`候補であるため、ID-bearing contentの採用は同書とDocument意味の正式改訂を要する。非永続prototypeでは「編集操作列が既知の時にIDを継承する案」と「String diffで後から和解する案」を比較し、前者が有力でも§4.1 gate前に恒久化しない。

#### 3.2.2 Auto / Offset / Pinned timing

文字数変更後のTimingには少なくとも3状態があり、同じ「override」に畳まない。

| 状態候補 | 保存する意味 | 親のOrder / Interval変更 |
|---|---|---|
| **Auto** | 保存差分なし。親Sequenceから導出 | 再計算する |
| **Offset** | 親から導出した時刻への相対差分 | 新しい自動時刻へ同じ差分を加える |
| **Pinned** | Text/Clip先頭からの絶対ローカル時刻 | 音合わせを優先し、その時刻を維持する |

ユーザーがTimeline上でdragした時にOffset/Pinnedのどちらへなるかは未決であり、両者は親変更時の結果が異なる。曲全体の絶対時刻は保存せずText/Clipローカル時刻とし、Text ObjectをTimeline上で移動すれば一緒に移る。新規文字はAutoで開始する。「前後のAuto/Pinned時刻の間へ置く」はPinnedが順序外や同時刻にあると一意でないため既定規則にせず、まず親SequenceのAuto評価へ参加させる。

### 3.3 Live Split / Detach(恒久スキーマ候補 — 未採用)

AE式Splitの失敗は分解操作そのものではなく、見た目を直接編集可能にするために元のText意味、本文編集、font変更、行組、一括glyph描画を失う点にある。Live Splitは、UI上だけ文字レイヤーのように展開し、Document上は1つのTextのまま保つ候補である。

これは新しいアーキテクチャではなく、[extensible-core-model.md](../extensible-core-model.md) §7「アドレス可能な個体」(個体をObjectとして保存しない)+§7.1の宣言的介入(`Pin(instance_id, …)`と同型)+§8.3外殻/payload分離の、Textによる最初の製品具体化である。「別文字へ黙って付け替えない」も§7.1が一般形で規定済み。P0I spikeの比較対象には最初からTextClusterが含まれる([既知技術処分決定](2026-07-14-motion-foundation-known-tech-disposition.md) §5)ため、Live Splitはこのspikeへ粒に先行するText側の製品動機を与える。

```text
▼ T  夜を走る          1 TEXT · 4 EDITABLE CHARACTERS
    夜
    を
    走
    る
```

- 通常のTimelineはText 1行。明示展開時だけ文字別の評価済みlaneを出す
- Stageでは各文字候補を直接選択・drag・scale・rotateできる
- 本文、font、style、行組、Text全体transform、まとめた`draw_glyphs`経路を保持する
- Manual character layout / Manual character timing / Text Sequenceを同じText内で合成する
- UI上の子行は独立Object、所有、Effect stack、個別生存期間を持つとは限らない

保存候補はレイアウト後の各文字候補へ適用する疎なlocal transform/timing差分だが、対象identity、値集合、合成順、anchor、保存形式は未決である。Position等は再行組後の基準位置へ加えるText local正準差分でなければ本文・font変更に追従できないが、この原則から具体fieldを作らない。

未決の軸がもう1つある: **overrideの時間可変性**。少なくとも2案を比較する。

1. Live Splitのoverrideを静的な配置と時刻補正に限り、文字ごとにPosition等を時間変化させる時点でDetachする
2. addressable clusterごとのPosition / Scale / Rotationにも既存ParamSourceを許し、Text内部の個別animation channelとして編集する。Detach境界は時間可変性でなく、Text外の所有・Effect stack・接続・生存期間に置く

後者はLive Splitの「レイヤーのように触れる」を素直に満たすが、identity、channel数、Timeline密度、本文編集後の和解を重くする。前者は恒久面を狭くする一方、1文字だけ軌跡を直したい通常のリリック作業で早すぎるDetachを強いる可能性がある。fixture前にどちらも採用しない。

また[text-model.md](../text-model.md) §1の二層は、時間可変/静的ではなく、スタイル=リフローする属性 / アニメーター=`glyph_transform`・opacityでリフローしない属性の分離である。したがって静的layout overrideは評価経路上は後者へ収まり得て、直ちに第三の評価層を新設するとは限らない。新しいのは、Selectorで配る規則とは別にaddressable clusterへ疎な手動介入を所有・保存する意味である。採用時はtext-modelへtarget identity・合成順・UI上の区別を追記する必要があるが、二層表を三層へ改訂するか、非リフロー層内の介入として整理するかは未決とする。

本当のObject化は明示Detachとして最後にだけ行う。最初の成立基準は**Text全体をDetachして完全独立させる**ことであり、候補条件は次のとおり。

- 文字ごとに別Effect stackを持たせる
- 別Groupへ移す、mask/inputとして外部接続する
- Textでなくpath/shapeとして加工する
- Text本体と異なる生存期間・所有・参照を持たせる

DetachはG4の明示Materializeで、Preview/Cancel、保持内容と失う意味の事前表示、1 Undo、確定後の独立を満たす。元Textとのlive同期や自動再Detachは作らない。

最大の未決は、本文編集後のoverride和解である。例:`夜を走る`→`夜道を走る`では新しい`道`を既定値で追加し、既存候補を可能な限り保持したいが、重複文字やcluster再形成時に別文字へ黙って移さない。§3.2の和解4候補、`Needs Review`の所有、P0I fixtureを先に審判する。

部分DetachはOPTIONAL DETACH成立の必須条件に数えない別実験である。選択文字を元Textから削除すれば後続の行組が詰まり、残せば独立Objectと二重描画になる。少なくとも「元のadvance/cluster slotを予約してglyphだけ抑止する」「文字を削除して再行組する」「Text全体をDetachする」を比較し、本文・font変更時の追従、元Textと独立Objectの関係、欠落参照、Undoを決める。slot予約はTextが配置穴を所有し続けるため完全独立ではなく、Detachの充足例として扱わない。なおslot予約案には、行内でadvanceを占有する非文字要素というDTPの同型先例(インライン/アンカー付きオブジェクト。例: [InDesign Anchored Objects](https://helpx.adobe.com/indesign/using/anchored-objects.html))があり、「glyph抑止」ではなく「clusterをanchored placeholderへ置換する」と定義し直せる。この場合、font変更時にplaceholder advanceを元値で凍結するかem比例で追従させるかが派生未決になる。

### 3.4 その他の比較中

- Split時のGroup既定(R3「既定でGroupを増やさない」は比較仮説。Timeline折り畳み要件との緊張あり)
- Character LayoutのLive Text内部要素版とDestructive Split Object版
- 順序モード(内外・ランダム等)の`selector.shape/seed`への写像語彙
- Browserでの提示: `Text Sequence`(Effects系)と`Split Text`(TOOL · CREATES OBJECTS表示)の分離、[Browser分類の未統一](../decision-index.md)への合流
- Materialize後のglyph様Object N個のdraw再バッチ可否(1 Text Objectの`draw_glyphs`一括との性能差) — 性能fixture行き

### 3.5 DuplicateはIndependent(既決のTextへの適用)

MVで同じ歌詞やモーションを繰り返す場合も、既定Duplicate/Copy-PasteでDefinition/Instanceのlive共有を作らない。[2026-07-13決定](2026-07-13-decision-pack-adoption.md) A8はDocument操作対象のIDを複製時に新規採番し、複製サブツリー内参照だけを新IDへ再写像すると決めている。Textにもこの一般則を適用する。

- Duplicate / Copy-PasteしたTextは新しいText IDと、採用されるなら新しい書記素IDを持つ
- 本文、Style、Sequence、文字別介入は現在値から複製し、複製内のtargetだけ新IDへ再写像する
- 複製後は独立し、片方の本文・Style・Sequence変更を他方へ反映しない
- 文字TimingはText/Clip先頭からのローカル時刻なので、Timeline上の別位置へ複製しても内部譜面ごと移る
- 再利用の入口はまずStyle preset / Motion preset候補とし、Text本体のlive共有を作らない

`Linked Repeat` / Phrase Definition-Instanceは初期範囲から棄却ではなく延期する。既存のGroup Definition/Use需要札([AEレイヤー処分](2026-07-16-ae-layer-system-disposition.md) §5)とLinked clones延期([反復再発明監査](2026-07-14-repeated-wheel-standardization-audit.md)「Materialize / Linked clones」表)と同じく、実需、リンク表示、局所override、unlink、欠落、複製規則を再審理するまで追加しない。少なくとも初期リリック機能の前提にはしない。

## 4. 比較fixture

1. かな・漢字・結合文字・混在styleを含む20〜30クラスタの歌詞
2. 文字を非直線的に手配置
3. 上下左右・交互・ランダムの入退場+時間差(seed固定)
4. 作業後に歌詞中央へ2文字挿入 → **Undo一回性・組版位置保持・手配置/override生存率**を測定
5. 同一文字列断片の反復: `夜へ 夜へ` → `夜へ 深い夜へ`(LCS・内容アンカーの誤接続最悪ケース)
6. 結合文字・同じ漢字の連続・ルビ付き範囲・合字の有効/無効切替・フォント変更によるshaped cluster変化
7. 1曲分を想定した数百〜千Objectで、Timeline・journal・評価・描画を測定
8. Live Split中に本文・font・字間・改行幅を変更し、local transform/timing差分が別文字へ誤接続せず、組版基準位置へ追従するか
9. `CONTROL`のみ / `RESULT`のみ / 二段併記で、指定文字の登場時刻発見・個別調整・自動規則へのResetを比較する
10. Text全体をDetachし、見た目・時間・styleの保持、Text意味とlive同期が切れること、1 Undo、確定後の完全独立を確認する
11. 1文字だけを部分Detachし、slot予約 / 削除再行組 / 全体Detachで、後続文字位置、本文・font変更、二重描画、欠落時表示を比較する
12. Live Split中の1文字だけへPosition keyを追加し、Text内部channel / 即Detachで、作業手数、Timeline密度、本文編集後の保持、通常のリリック手直しを比較する
13. Cloner反証では高数量(1,000 instance以上)を含め、明示展開・選択したinstanceだけが時間面へ投影され、個体数に比例したlane・marker・Document項目が生じないことを確認する([ui-score-model.md](../ui-score-model.md)回帰審判14と同一条件)
14. `走`へStart / Duration / Position差分を入れた後に親のInterval / Order / Durationを変更し、差分維持と`Reset to Sequence`を確認する。続けてFont Size変更とVisual Scale変更を比較し、前者だけが再行組し、いずれも別clusterへoverrideを誤接続しないことを確認する。さらにanimator 2枚(例: Rise+Bounce)の構成で、`走`のtiming差分がどの規則評価へ効くか(全stack共通 / 対象Sequence単体)を両案で記録する
15. 同じTextをDuplicateして内部targetを新IDへ再写像し、複製後に片方の本文・Style・Sequenceを変えて他方が不変であることを確認する。Auto / Offset / Pinnedの3候補で親Interval変更とText全体移動を行い、前者では各定義どおり変化し、全状態がTextローカル時刻として一緒に移ることを確認する

### 4.1 fixtureの順序と合否

fixtureは次のgate順に行う。先のgateが失格でも、後段UIを都合のよい仮identityで採用判定しない。

1. **Identity/Reconcile gate**: fixture 4/5/6/8/15。黙った別clusterへの誤接続は`0件`。和解不能を`Needs Review`/Resetへ送ることは許すが、同じ入力と編集列から判定が決定論的に一致すること。挿入/削除/置換/全文pasteに加え、IME Commit、Unicode正規化、合字、fallback、ルビ、Duplicate後のID再写像を含む
2. **Evaluation ownership gate**: fixture 12/14。各prototypeはtiming scope、書き込み先、合成順を開始前に宣言する。逆変換不能時にDocument変更`0`、絶対keyへの暗黙Bake`0`、UI-only保存`0`
3. **Projection/UI gate**: fixture 2/3/9。collapsed時はText 1行、展開行は明示したText/selection scopeだけ。UIのbar packingや開閉でDocument snapshotが変化しない
4. **Detach gate**: fixture 10を先に審判し、全体Detachが1 macro / 1 Undo、Undo後snapshotが操作前と一致、確定後に元Text変更の影響`0`。fixture 11の部分Detachは別実験で、失敗しても全体Detachの成立を否定しない
5. **Scale/engine gate**: fixture 7/12/13。Live TextのDocument Object増加は文字数に対して`0`(Text 1 Objectのまま)。Clonerの表示行/markerは明示selection/scope以下で、instance総数だけを増やしてもDocument項目数と表示行数が増えない。各sample時刻の評価済みtransform/timingはPreview/Exportで同じ意味になり、Quality差以外の不一致`0`

全gestureはCancel時Document変更`0`、確定時1 commandまたは1 macro / 1 Undo、Undo後の意味snapshot一致を共通条件とする。時間、identity、journal、cache invalidation、Preview/Export一致はUI mockだけでは合否を出さず、純関数fixtureまたは計測可能なprototypeで審判する。性能値は基準機と予算が未決のため採否閾値を捏造せず、同一入力でLive Text / Destructive Split / Rule-only baselineを比較記録する。

## 5. AM実機観察(2026-07-19、ユーザー確認)

AMには、今回比較している文字分解・文字別配置・文字別タイミングを一括生成または横断編集する機能は無く、利用者が手作業で組む。AEのScriptUIやAviUtlのスクリプトに相当する拡張面も無いため、Texton/Aulymo/TA系のような不足補修を第三者が追加する経路も無い。

この観察はLive Text / Destructive Split / Rule-onlyのいずれかを支持する証拠ではない。UX北極星にもこの用途の解答は存在せず、手作業負担が残っているという反面事例である。MotoliiがAMの操作感を参照しても、この欠落まで模倣しない。AM利用者の具体的な手作業手順と、本文変更後に何を作り直すかは未記録のため、本観察から永続overrideやMaterialize方式を決めない。

## 6. 停止線(本書で変更しないもの)

- M2凍結面・公開plugin trait・Document field: 追加しない(P0I spike・解凍手続き前)
- Identity/Reconcile gate通過前に安定書記素ID、override store、Needs Review状態をDocumentへ追加しない
- Textの編集正本をID-bearing contentへ変えず、Auto/Offset/Pinned fieldも追加しない。採用にはtext-modelとM2 Document意味の正式改訂を要する
- TextとClonerの両gate通過前に共通Element schema、公開`Element`/capability trait、汎用property path、全domain共通Timeline laneを追加しない
- Authoring Toolの第三者公開: しない(v1 first-party限定)
- Authoring Toolが読めるのはDocument snapshotと純関数評価結果のみ。GPU readback・実時刻・一時UI状態を読まない(選択順は操作開始時の入力列としてのみ)
- seedは明示`user_seed`をD2 commandでDocumentへ(2026-07-15決定)
- 本書のUI名称・表・fixtureは審判前の仮名・仮案であり、モック・仕様へ公約として転記しない

## 7. 反対側レビューの処分(2026-07-19)

Grok 4.5へ本書と併読正本を入力したread-only反対側レビューは`REVISE MEMO BEFORE FIXTURE`と判定した。repository変更は行わせていない。レビュー原文は[evidence/grok-lyric-counter-review](evidence/grok-lyric-counter-review/README.md)へmodel指定、実施日、入力文書、prompt要旨とともに全文取込済み。主担当は次を採用して本書へ反映した。

- 4条件を中立比較軸でなくLive Textの製品命題と明記し、Destructive Splitを同等候補からbaselineへ移す
- §2.1の「既存機構」をSequence評価へ限定し、Live Splitの所有意味は新規候補と訂正する
- timing差分のstack scope、逆変換不能時、部分DetachをSTOP/別実験として分離する
- identityを最初の非永続fixture gateとし、定量的な合否を§4.1へ置く
- 共通Host capabilityと恒久面の停止線を再掲する

一方、「identity契約が未決ならfixture自体を止める」という強い判定は採らない。止める対象は恒久schema・採用・一般化であり、和解4候補を非永続prototypeで反証するfixtureはidentity契約を選ぶために必要である。P0/P1という外部reviewerの重要度ラベルは設計決定へ転記せず、上記の論点単位で処分した。

## 8. Gate 1基本編集subfixture(2026-07-19、部分合格)

[lyric-identity-reconcile](../spikes/lyric-identity-reconcile/README.md)を、製品コード・Document schema・公開APIへ触れないHTML/JavaScriptの非永続prototypeとして作成した。これはIdentity/Reconcile gate全体の完了ではなく、単純な挿入・削除・置換・保守的全文変更・Independent Duplicateだけの最初のsubfixtureである。

初期状態:

```text
夜 g1  AUTO
を g2  AUTO
走 g3  PINNED 4f / Y -36 / Visual Scale 140%
る g4  AUTO
```

`夜`の後へ`道`を挿入した結果:

```text
夜 g1  AUTO 0f
道 g5  AUTO 2f  ← 新ID。親seed=1842由来のRandom In
を g2  AUTO 4f
走 g3  PINNED 4f / Y -36 / Visual Scale 140%  ← 同じIDと介入を保持
る g4  AUTO 8f
```

観測:

- `道`は新ID・overrideなしで親Sequenceへ参加し、同じseed+同じIDならRandom In poseが決定的に一致した
- 既存の`走`はID、Pinned local time、Text-local Y offset、Visual Scaleを保持した
- `走→飛`では`飛`へ新IDを割り当て、旧`走`の介入を自動継承せずNeeds Reviewへ隔離した
- 全文変更で対応不能な旧`走`の介入も別文字へ移さずNeeds Reviewへ隔離した
- `夜へ 夜へ→夜へ 深い夜へ`の単一挿入では、操作上明確な後半suffix IDと介入を保持した
- Independent Duplicateは全IDを再採番し、overrideを値として複製した後は相互に影響しなかった

自動検査:

```text
node --test docs/spikes/lyric-identity-reconcile/reconcile.test.mjs
7 tests / 7 pass
```

ブラウザでも挿入、置換、全文変更を操作し、Stage ghost、Character Score、Identity表、Needs Reviewが純関数結果と一致することを確認した。途中で置換後の上部説明だけ旧`走`を表示するUI分岐漏れを発見・修正し、再操作で一致を確認した。

**未実施のためGate 1完了としない**:

- IME composition/Commit、Unicode正規化
- 結合文字、合字ON/OFF、font fallback、ルビ、style range境界
- 複数の最適diffが存在する反復文字編集
- 実Document command/journal/Undo、save/reload、migration
- shaped clusterと書記素ID集合のtarget規則

### 8.1 Random InとCharacter Collisionの次段仮説

ユーザー観察として、入口モーションは多くの場合Random Inで十分で、価値の中心は文字ごとの最終配置・大きさ・直接修正にあり、さらに文字同士の衝突へ広げると既存ノード合成ソフトでは届きにくい表現になり得る、という仮説が出た。市場全体の利用比率を測った証拠ではないため、Random Inを唯一の既定へ決定しない。

評価境界は既存[simulation-model.md](../simulation-model.md)のはしごに従う。

- 衝突なしのRandom Inは`seed + character identity + t`で決まるLevel 0の閉形式純関数候補
- 文字同士/他Shapeとの衝突、押し合い、落下を有効にした時は前状態依存なのでLevel 3 `Simulation + StateTrack`候補。AnimatorやFilterへ隠れ状態を持たせない
- 相互作用する文字群は一つのText由来Simulation領域として扱い、文字ごとのDocument ObjectやTimeline rowへ自動展開しない
- body identityはGate 1の書記素/cluster target審判へ依存する。本文編集後に物理介入を別文字へ移さない
- seed、固定dt、collider recipe、初期poseを決定入力とし、Preview/Exportで同じStateTrackを読む

本subfixtureにはphysics solver、StateTrack、Bake、Collider fieldを追加しない。Gate 1完了後に、2〜8文字の単純円/箱近似で「衝突なしLevel 0 / 衝突ありLevel 3」を別prototypeとして比較する。simulation-modelの実装時期・解凍gateを本書から前倒ししない。

## 9. Vism分界仮説(2026-07-19、比較中)

リリック機能の最終形はText Animator表現を第三者Vismへ開くことにあるが、Text Animator全体をVismへ追い出さない。分界は2層ではなく**3層**で仮説化する — 「基盤=Host」と一括りにすると、[text-model.md](../text-model.md) §3/F-6分界(行組=プラグイン段)を黙ってHost coreへ移すことになるためである。

```text
Host core(公開境界の管理面)
├─ identity外殻: 保持・参照・欠落・Undo・誤接続防止の規律(Gate 1)
├─ 介入の保存・Undo・欠落診断
├─ Stage hit-test・選択・Character Score/Stageへの投影
├─ animator合成順・評価scheduling
└─ itemize / shape / draw_glyphs(text-model §3の段1-2/6)

First-party Text plugin(公開境界上の参照実装 — extensible-core §8.6ドッグフード)
├─ 行組(F-6分界のまま。Host coreへ移さない)
├─ TextCluster domainの定義と、書記素→TextCluster写像・identity継承候補
│  (extensible-core §8.1。候補の採否はGate 1 fixtureが審判 — identityはHost単独の発明ではなく共同境界)
└─ 標準Sequence(Random Entrance等)

Third-party Vism(表現)
├─ 選択重み・登場順・間隔・seed
├─ 始点pose・散らばり・easing・軌跡
└─ 将来: 物理・Field・音反応(§8.1のsimulation-modelはしご準拠)
```

- **専用`TextAnimatorPlugin` APIを作らない**(extensible-core §8.1「表現名のHost APIを増やさない」)。既決なのは意味方向 — Cavalry型Behaviour構造の縮小採用、純関数、`InstanceId != index`、明示seed([Duplicator決定](2026-07-15-relative-scope-duplicator-decision.md) §5)。一方、Effector評価形(`Effector(instance, t, params) -> InstanceDelta`、`InstanceContext`)自体は[既知技術処分決定](2026-07-14-motion-foundation-known-tech-disposition.md) §5の**spike仮説でP0I比較中**であり、Cloner/Effectorの製品採用・公開`EffectorPlugin` traitは未決。Text Animator Vismは、専用APIを増やさずこの評価形へ合流させる**第一候補**であり、P0I spikeが同型性の審判になる。
- **Host共通overrideの合成機構(仮説)**: `Start Offset` / `Duration Scale`は、Hostが該当elementのlocal timeを`(local_time - start_offset) / duration_scale`様のアフィン変換で書き換えてからEffectorを呼ぶ**入力側local time remap候補**(原点、clamp、逆再生、`scale <= 0`、複数Animatorへの作用scopeは未決)。Position / Visual Scale / Rotation差分はEffector出力への後段加算(出力側)。`Host入力補正 → 第三者Effector → Host出力補正`の形により、第三者animatorはoverrideの存在を知らず純関数のままで、§2.8の入力/出力非対称はplugin境界で機構的に解消する。逆変換を宣言しない第三者animatorではGhost直接編集が§2.6の閉集合へ落ちるが、**Host override層による直接操作の最低床は任意のanimator上で成立する**。
- **texture-only化の禁止方向**: Text Animatorを「最終textureを返すVism」にすると内部時間と文字が再び隠れ、§1の「AE分解ツール=Host拡張面制約の痕跡」仮説をMotolii自身の拡張面で再生産する。現行LayerSource loweringは最終textureのみを返すため、**texture出力とは別に、評価済み個体をHostへ提示する公開能力が必要**になる。それをLayerSource loweringの拡張として実現するか、並列のevaluated-domain capabilityとして独立宣言するか(`Render capability → texture` / `Evaluated-domain能力 → query / identity / bounds / delta` の分離)は**P0I後に決める**。描画責任と個体問い合わせ責任を同じinterfaceへ結合するか自体が未決であり、現段階ではどちらも候補に留める。
- **開放時期は既存停止線に従う**([vism-package-concept §11](../vism-package-concept.md)、本書§6)。順序: first-party Live Text+標準Entranceを内部APIなしで作る → identity・直接操作・Character ScoreをHost能力として確立 → 同じ境界を第三者Vismへ開く。

この境界を育てる価値は標準Animatorの品数ではなく、**Motolii本体が想像していない文字表現を第三者が正規ルートで発明できる余白**にある(extensible-core §1「意味は厳格に、表現は自由に」の最初の実証領域)。想定される将来表現は、それぞれ異なる応力を境界へかける。固定の返却項目一覧ではなく能力宣言型のAPIにすべき理由は、この表の多様さ自体である。

| 将来表現の例 | 境界へかかる応力 | 既存アンカー |
|---|---|---|
| モーラ・アクセント核・音素・歌声・感情で動く | domain固有の読み取り属性(言語・解析注釈)の宣言 | extensible-core §8(解析→生成が生む個体は前提)、[vism-kit-model](../vism-kit-model.md)の型付きinput |
| 字形の面積・重心から運動を決める | shaping結果由来のglyph幾何をdomain属性として供給(GPU readback禁止のまま) | 本書§6停止線、P6クラスタ対応表 |
| 文字同士の衝突・**結合・分裂** | L3 Simulation+StateTrack(§8.1)。**結合・分裂はelement集合とidentityを実行時に変える唯一の新応力** — 親identityからのhash合成が候補 | [simulation-model](../simulation-model.md)、nested Duplicatorのhash合成規約 |
| 映像中の人物・輪郭を避けて配置 | Text APIの拡張ではなく、解析providerの型付きinputをKitが接続する合成 | [vism-kit-model](../vism-kit-model.md) |
| 部首・ストロークへ潜って動かす | 下位domainの提示(cluster→stroke)。domain入れ子 | nested InstanceIdのhash合成と同型 |
| 単語=群れ、文字=個体 | 複数粒度のcontext同時提示(group参照をcontextへ含める候補) | text-model selector unit(word/line)と同根 |
| 3D飛来・螺旋・文字別押し出し・ビルボード・剛体整列 | base pose/deltaの3D化を`Transform3D / Geometry3D / Material / StatefulSimulation`等の**追加的能力**として宣言(全TextClusterへmesh/physicsを義務づけない。非対応Text pluginは診断・拒否し、黙って2Dへ潰さない — extensible-core §8.7) | [統一Stage/Camera決定](2026-07-14-unified-stage-camera-design.md)、AEのper-character 3D([Adobe: Animating text](https://helpx.adobe.com/after-effects/using/animating-text.html) — 専用Animatorを別発明せず同じanimatorへ3D propertyを足した先例)、§8.1 simulationはしご |

3D能力には「追加channel」では済まない契約接点が2つある。(1) [text-model.md](../text-model.md) §2のproperties(`position: Vec2`)とP6 `glyph_transform`経路は2D前提であり、Transform3DはM5-P6契約とproperty集合の正式拡張を要する。(2) 文字が個別depthを持つ時、Text内部のローカル3D(文字同士のみ前後し、Textは1合成順位)と、シーン中の他Objectとの個別深度ソート(1-Object合成と`draw_glyphs`一括を壊す)は**別の製品**であり、明示的に選ぶ(黙って一方を他方に見せない)。どちらもM5の解凍手続き対象で、本書から前倒ししない。一方、**Character Scoreのidentity・時間編集・入力側override機構は次元数に依存せず再利用できる** — 文字identity、個別選択、local time remap、`Start Offset` / `Duration Scale`、Timing Scoreの行と時間表示、overrideの保存・Undo・欠落規律はそのまま継承する。対して空間overrideの3D化 — `Position Vec2`のXYZ型拡張、Rotation Zから3軸回転への意味、3D Scaleの合成規則、親Text transformとの座標順序、Score/Inspectorへの追加channel投影、Stage操作・hit-test・bounds — はTransform3D能力とP6契約の正式拡張を要し、「追加設計なし」ではない。分担は次のとおり。

```text
既存のまま継承
  Identity + Local Time + Timing Score

M5で追加審判
  Transform3D delta + Spatial Score channels

さらに別の製品判断
  Local 3D / Scene-participating 3D
```

これにより「仕組みの骨格は共有できる」と「3Dはchannel追加だけでは済まない」が矛盾せず両立する。

本節は分界の仮説登録であり、公開trait・`.vism` loader・LayerSource改訂・第三者開放の実装許可ではない。
