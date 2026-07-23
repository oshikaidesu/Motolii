# 小さなコアと探索可能な拡張

作成日: 2026-07-17

状態: **設計原則**。利用者と開発者の学習曲線、UI、編集系pluginの責任境界をまとめる。本書は未凍結の公開API、Document variant、custom plugin UIの実装許可ではない。具体的な契約追加はM2/M3の解凍手続きと反対側レビューを先に行う。

関連正本: [コンセプト](concept.md)、[操作単純化モデル](interaction-simplicity-model.md)、[UI操作言語](ui-interaction-language.md)、[ジェネラティブユーザー境界](generative-user-boundary.md)、[プラグイン作者向け規約](plugin-authoring.md)

本書の拡張原則を、ProjectとHostから持ち運べるユーザー向け配布単位へ投影した長期コンセプトは[Vism](vism-package-concept.md)を参照する。小さなVism同士を直接依存させず、型付きinputをKitがproviderへ接続してProjectへmaterializeする責任分離は[Vism / Kitモデル](vism-kit-model.md)を正本とする。

## 1. 結論

Motoliiは、少数の型付き意味をHostの小さなコアに置き、Direct / Tool / Advanced / pluginの全入口をそこへ正規化する。Linux/UNIX思想から借りるのは外観やcommand lineではなく、**一つの責任を持つ小さな要素、明示した入出力、合成可能性、交換可能な操作面**である。

```text
Direct / Tool / Advanced / plugin UI
                  ↓
       Domain Intent / typed input
                  ↓
       Host command or evaluation
                  ↓
              Document
```

小さいことは機能が少ないことではない。Hostが作品の持続性に必要な意味だけを厳格に所有し、表現の組合せと専門的な操作面を開くことである。

ここでいうミニマリズムは、空の本体へ必要機能を各自で寄せ集めさせることでも、UIを疎に見せることでもない。**必要十分な表現力を保ちながら、使わない能力の常駐負荷と、使う能力の導入・更新・再現に伴う管理負荷を最小にすること**である。pluginで能力を追加できても、導入手順、version、欠落時の挙動、作品共有が利用者の記憶に依存するなら、Hostは小さく見えるだけで制作環境は小さくない。Hostの型付き契約、互換診断、lifecycle、共通UIと検証境界によって、拡張後も「待たない・迷わない・抱えない」を保つ。これはv1へmarketplaceや新しい配布契約を追加する決定ではない。

> **意味は厳格に、表現は自由に。壊れ方は封じるが、暴れ方は封じない。**

極端な値、逆転、発散、画面外、奇妙なeasing、通常想定しない組合せは正当な表現である。拒否するのは、名前検索、隠れcontroller、宣言されない状態、全体を評価不能にする循環、復元不能なmutation等、因果と回復を失わせる仕組みである。

## 2. 学習曲線は利用者と開発者で同じ形にする

### 2.1 UIは実行可能な最初のドキュメント

初心者の最初の成果に外部manualを要求しない。学習は次の循環で進む。

```text
触る → 結果が見える → 戻せる → 別の値を試す → 関係を理解する
```

標準UIは、操作前に抽象概念を暗記させず、必要になった瞬間だけ次の情報を出す。

- 何を操作しているか。
- 今つながる、または適用できる対象は何か。
- 確定すると何が起きるか。
- なぜ拒否されたか、どう回復できるか。
- もっと理解したい時に、現在の対象と状態を引き継いでどの資料へ進めるか。

Helpやdocsは基本操作を成立させる前提ではない。操作によって生じた疑問を一般化し、応用へ進むための第二層とする。汎用のHelpトップへ飛ばすのではなく、選択中の意味、接続、error、評価結果に対応する節へ文脈付きで到達させる。

### 2.2 壊れやすさは学習を単一用途へ縮退させる

ハック的なscriptやrigは、少し値を変えただけで壊れ、原因と復元方法が見えないことがある。その時ユーザーは仕組みを探索せず、動いた一つの手順だけを儀式として暗記する。

```text
少し触る → 壊れる → 原因が見えない → 戻せない → 正解手順だけ暗記する
```

したがって「壊れない」はクラッシュしないことだけを意味しない。

- 予測可能な不正操作はCommit前のpreflightで局所的に拒否する。
- 拒否はexpected/actual、原因、回復候補を型付きで返す。
- 操作中はTransient previewとし、CancelはDocument変更ゼロにする。
- 一つの意図は一つのUndoで戻す。
- 画面外や極端値の対象も、Fit / Reveal / Reset / Undoから回収できる。
- 一部の欠落、循環、plugin errorで無関係なDocument領域を操作不能にしない。
- 自動修復で別の意味へ黙って変えず、修復候補を提示してユーザーが確定する。

失敗も学習UIである。人間向け文言は「できません」ではなく、製品の意味を説明する。外部検索は原因と次の一手の代用品にしない。

### 2.3 開発者にも崖を作らない

開発者はHost全体、任意mutation API、独自panel shellを理解してから拡張を書くのではなく、最小の型付き境界から始める。

- 一つの目的を一つの種別へ置く。
- 既存のDomain Intent、command、parameter、入力型を合成する。
- HostがUndo、single writer、cache、preview/export、欠落診断、UI componentを受け持つ。
- 必要な自由度が増えた時だけ、宣言する責任を一段増やす。
- 参照実装と機械判定可能なtestで境界適合を確認する。

利用者のSimple / Advancedと同様、初心者plugin作者用の別世界を作らない。最小例から高度な時間依存や独自編集面へ進んでも、入力、出力、責任、失敗の同じ語彙を使う。

### 2.4 利用者と開発者を固定身分にしない

利用者と開発者は別製品、別community、別の意味体系に属する二種類の人ではない。同じ人が場面ごとに、既存表現を使う人、値を調整する人、接続を組む人、recipeを共有する人、Vismをforkする人、codeやcomponentを書く人、他者の成果を保守する人になる。

```text
Use → Tune → Compose → Inspect → Fork → Author → Publish → Reuse
```

この階段の各段で、対象identity、型付きparameter、入力、Preview、診断、versionを捨てて別toolへ移り直させない。codeを書かない利用者へauthoringの複雑さを常設せず、進みたい人にだけ現在の文脈を保った次の段を見せる。

多数の作者が並行して表現を増やせることは、Motoliiの機能供給戦略である。ただし、空の本体を渡して利用者に基礎機能の穴埋めをさせる意味ではない。Hostは§3の共通責任と標準制作体験を持ち、作者は表現固有の発明へ集中する。詳細は[Creator / Developer連続体](reviews/2026-07-22-creator-developer-continuum-decision.md)を正本とする。

## 3. Hostが小さくても手放さない責任

小さなコアは薄いコアではない。次は表現pluginやUI pluginへ投棄しない。

- Documentの型、stable ID、ownership、参照、version、migration。
- single writer、D2 command、macro、Undo/Redo、journal。
- 時刻、型付きparameter、評価順、循環拒否、scope。
- Preview/Export同一評価、cache/invalidation、resource lifecycle。
- 正準座標、色変換、Quality、GPU境界。
- selection、focus、Target、Preview、Commit/Cancel、error、accessibilityの共通component。
- plugin欠落時の保持、診断、fallbackと再導入時の復元。

pluginが所有してよいのは、専門的な**表現の計算**または同じHost意味へ到達する**操作面**である。作品全体の整合性と回復可能性はHostが所有する。

## 4. ピクセル以外を操作するpluginの責任寿命

「位置や大きさを扱うか」では境界を決めない。pluginの責任が確定後も残るか、時刻や入力変更で再評価が必要か、独自構造が作品の正本かで分類する。

| 分類 | 責任寿命 | 保存されるもの | plugin欠落時 | 例 |
|---|---|---|---|---|
| **Authoring Tool** | Commitまで | Host標準のtyped command結果 | 完全に通常編集できる | 整列、円形配置、keyの時間移動、one-shot生成 |
| **Behavior / Driver** | Document内で継続 | 型付き入力、出力、scope、評価順、version | 関係と欠落を識別し、無関係部分は編集可能 | Follow、音量→Scale、Field→Position |
| **Generator / Structured Recipe** | 独自recipeが存在する間 | version付きrecipe、宣言入力、materialize/live区分 | recipeと最後の診断可能な結果を保持。完全再編集は再導入後 | node graph、procedural layout、専門的generator |
| **Render / Simulation** | 評価またはHost管理bake中 | NodeDesc、parameter、入力、必要ならStateTrack | Host既定のpass-through/placeholder/保持規則 | Filter、Composite、LayerSource、Simulation |

この分類名は設計上の責任を表す。現在のv1で公開済みtrait一覧を意味しない。新しい`BehaviorPlugin`、`ToolPlugin`、custom UI APIを本表から直接実装してはならない。

### 4.1 Authoring Toolは自由な`&mut Document`を受け取らない

一回限りの編集拡張は、read-only snapshotと選択・入力を読み、型付きcommand batchまたはDomain Intentを返す。Hostは全体をpreflightし、一つのmacroとしてsingle writerへCommitする。

```text
immutable snapshot + typed selection/input
                    ↓
              Authoring Tool
                    ↓
       typed command batch / intent
                    ↓ preflight
          1 macro commit or no change
```

部分適用、途中の失敗、UI event列の保存、layer名/property path検索、隠れhelper生成を許さない。確定後にplugin固有runtimeが不要なら、Documentへplugin依存を残さない。

### 4.2 Behaviorは「続く意味」を宣言する

時刻、入力、参照先の変更後も効果が続くものをcommand macroで偽装しない。最低限、入力型、出力型、所有scope、評価順、時間依存、循環/欠落/削除時挙動、複製規則、cache keyへの寄与を宣言する。

標準機能で同じ結果を手作業できることは、BehaviorをHostへ焼く理由にも、pluginを拒否する理由にもならない。判定するのは結果の再現可否ではなく、継続する意味と責任の所在である。

### 4.3 Generatorは独自構造を隠さない

node graph等の専門的編集面はpluginとして提供できる候補だが、自由なscript panelを第二のアプリケーション基盤として認めることとは異なる。

- pluginはrecipe、入力、出力、version、依存、Materialize/Live/Bakeを宣言する。
- 標準UIから、結果、由来、依存、異常、plugin依存の有無を確認できる。
- custom UIだけに存在する保存parameterを作らない。Host標準Inspectorから全保存値を検査できるfallbackを維持する。
- node editorを開かなくても通常の制作が成立する。ノードの複雑さを製品の高度さとして常設しない。
- plugin欠落をDocument load失敗や全体編集不能にしない。

## 5. Script Panel化を防ぐcapability境界

AE型のscript panelが複雑化する原因は、UIの自由さそのものより、panelがDocument探索、mutation、名前接続、独自controller、独自Undo、独自状態を同時に所有できることである。Motoliiは「任意panel API」を一つ開けず、必要能力を分離して貸す。

| capability | pluginができること | Hostが保持すること |
|---|---|---|
| Read snapshot | 型付きIDと公開意味を読む | 内部表現、mutable ownership |
| Propose edit | Intent / typed command batchを提案 | preflight、single writer、Undo、journal |
| Connect | 公開parameterと期待型を宣言 | target picker、型検査、循環拒否、rename/delete |
| Preview | Transientな候補を返す | 描画寿命、Cancel、品質縮退、Stage統合 |
| Evaluate | 宣言入力から結果を返す | scheduling、cache、resource、Preview/Export |
| Present custom UI | 専門的な編集投影を提供する | shell、focus、scale、theme、error、標準fallback |

capabilityを組み合わせても、pluginにDocument正本、UI正本、cache正本を渡さない。必要能力が既存capabilityに収まらない時は、裏口を追加せず新しい責任を一つだけ宣言する契約として審判する。

## 6. PluginからHostへの昇格

user拡張は不足するコア概念を観測するprobeである。人気や販売数だけで標準化せず、次の順で責任を引き取る。

```text
user plugin / recipe
        ↓ 反復需要と失敗を観測
validated preset / first-party plugin
        ↓ 意味と審判が安定
Host Tool
        ↓ 複数用途の基礎責任だと判明
typed primitive
```

昇格時に見るもの:

1. 複数作者が同じ目的を独立に再実装しているか。
2. 隠れhelper、名前検索、独自Undo等、Host不在では安全に実現できないか。
3. 一文で説明できる小さな意味へ分解できるか。複数の意味が重なるなら先に分ける。
4. 追加的schema、migration、欠落時挙動、意味論goldenを定義できるか。
5. 標準化後も、より奇妙な組合せをpluginで作る余地を奪わないか。

「UIだけ標準化し、裏でNull、expression、slider、scriptを生成する」は昇格ではない。Hostが意味とlifecycleを引き取った時だけコア化と呼ぶ。

## 7. Documentを増やさず、個体性を開く

パーティクル、Duplicator、文字単位処理、ブラシ散布、群れ、タイル、パス上の点、メッシュ要素、トラッキング点、解析領域、生成された字幕単語等では、画面上の要素数とDocument上のObject数を同一視しない。少なくとも次の四段を分離する。個体の種類をコアへ列挙しない受け口は§8で扱う。

| 段 | 存在するもの | できること | 正本 |
|---|---|---|---|
| **描画個体** | 評価時にだけ現れる要素 | 描画される | 原型、生成規則、時刻、seed |
| **アドレス可能な個体** | 安定したidentityを持つ評価要素 | hit test、選択、参照、問い合わせ | identityの生成規則。個体をObjectとして保存しない |
| **状態を持つ個体** | 速度、接触、寿命等を持つ評価要素 | 物理、近傍相互作用、履歴を伴う変化 | Simulation recipeとHost管理StateTrack |
| **Document実体** | 独立したShape/Object | 通常編集、個別key、永続所有 | Document。明示Materialize時だけ増える |

たとえば一つのShapeを原型として一万個の粒を描いても、Documentへ一万個のShapeを生成しない。

```text
Shape / Prototype
        ↓ typed reference
Generator / Distribution
        ↓ evaluate(t, seed)
addressable instance set
        ↓ optional Simulation
Render / Query / Selection
        ↓ explicit Materialize only
Document Objects
```

`instance set`は、全要素をCPUメモリへ並べた`Vec`を意味しない。概念上の生成空間であり、GPU生成、可視範囲だけの評価、空間問い合わせ、chunk処理、選択地点付近だけのidentity解決を許す。描画、選択、物理を可能にするために、全個体をTimeline rowやDocument Objectへ展開してはならない。

### 7.1 identityはindexではない

個体への選択、乱数、motion sample、物理的介入を配列順へ結び付けると、個数変更や並べ替えで意味が別の個体へ移る。既決定どおり、`stable identity != index`とし、index/countは順序表現にだけ使う。identityの具体的な生成・継承規則はgenerator domainごとに異なり得るため、一つの公開schemaへ先に固定しない。

選択した個体への操作も、選択しただけでObject化しない。たとえば次は、個体そのものではなく安定identityへの宣言的介入である。

```text
Pin(instance_id, position)
Impulse(instance_id, at_time)
Exclude(instance_id)
```

生成規則の変更で対象identityが一時的に存在しなくなっても、別個体へ黙って付け替えない。参照待ちまたは欠落として識別し、再出現時の再接続、介入の削除、明示Materializeを回復路にする。介入の保存形式、identity寿命、個体選択UIは未決であり、この原則だけからDocument fieldや公開traitを追加しない。保存形式の分解方向(共通外殻とplugin固有payload)は§8.3に置く。

### 7.2 物理は個体Objectではなく集合の責任

2D物理で粒ごとの位置、速度、衝突を扱っても、各個体へ隠れた状態を持たせない。Instance Set全体を一つのSimulationとして扱い、初期条件、固定step、seed、相互作用規則を正本とし、状態列はHost管理のStateTrackへ置く。レンダはその時刻の結果を読む。詳細は[時間軸の自由度モデル](simulation-model.md)に従う。

これにより、一粒を弾く、特定個体を固定する、粒同士を衝突させる、衝突した個体だけ別の見た目へ渡す、といった個体性を、Object大量生成やFilter内の隠れ状態なしで表現できる。個体から集計値やeventを取り出す境界、双方向結合、個体別overrideの保存は未決の探索対象である。

### 7.3 数の天井を意味論へ焼かない

有限資源は認めるが、現在のCPU、GPU、UI実装の都合を作品の上限にしない。

- Document schemaや公開契約へ、instance数、選択可能数、物理個体数の任意な固定上限を置かない。
- pluginへ「全個体を一括配列で返す」ことを必須にせず、Hostが時刻、範囲、品質、chunk、問い合わせを要求できる方向を保つ。
- Previewは密度、解像度、サンプル、更新頻度を縮退できるが、Documentの個数や意味を書き換えない。
- Exportは同じ評価意味をchunk化、stream、bakeして完遂する。Preview/Exportで別の作品にしない。
- 資源不足はDocument破損や黙った削減ではなく、識別可能な品質縮退または型付き評価失敗にする。
- アドレス可能性は全個体への常時コストではなく、必要時に解決できるcapabilityとして検証する。

> **コアは表現量の上限を定義しない。Hostは有限資源内の評価戦略を選ぶ。資源不足は作品の意味を変更しない。**

ここで決めるのはRust APIではなく不変条件である。streaming iterator、GPU indirect draw、spatial query等の具体方式は、性能fixtureとM5-P0I spikeを経て選ぶ。

## 8. 表現の種類をコアへ列挙しない

§7の四段と介入は、パーティクル固有の問題ではない。文字glyph、パス上の点、群れの個体、ブラシ跡、メッシュ要素、トラッキング点、解析領域、生成された字幕単語 — 「評価時に多数現れ、選択や介入の対象になり得るが、Documentへ並べたくないもの」は今後も増える。特にトラッキング点、解析領域、字幕単語は「解析→生成」という本製品の長期路線が生み続ける個体であり、種類の増加は例外ではなく前提である。その全種類を事前に列挙することはできない。

§1で借りると宣言したLinux思想の核心も、未来の機能を予測したことではない。**未来の用途を知らないまま、所有権、識別、入出力、寿命を扱える境界を用意した**ことである。そのLinuxですら既存境界だけでGPUを吸収できず、DRM/KMSを後から追加した。したがって目標を「最初のコアを永久に変更しないこと」に置かない。

> **未知の能力を、既存コアを破壊せず追加できること。**

### 8.1 domainはpluginが定義し、Hostは外殻だけを知る

`ParticlePlugin`、`TextElementPlugin`のように表現名でHost APIを増やす方式は、新しい表現のたびに未来予測とコア改訂を要求する。個体が何であるか(domain)はpluginの定義に置き、Hostは中身を解釈しない共通の外殻だけを扱う。

```text
Particle / Text glyph / Path point / Tracked feature /
Generated clone / Physics body / Future unknown
                  ↓
         plugin-defined domain
                  ↓
     Host-visible common envelope
```

| Hostが知るもの | pluginが決めるもの |
|---|---|
| どのplugin・生成元に属するか | 個体が粒、文字、点、領域のどれであるか |
| 安定identityがあるか | identityの生成・継承規則(§7.1) |
| いつ存在するか | 生成・消滅の条件 |
| どこに見え、問い合わせできるか | boundsやhit testの計算 |
| 選択・参照できるか | 個体固有の属性 |
| 介入を保存できるか | Pin、Impulse等の具体的な意味 |
| 欠落、version、Undo、診断(§3) | 評価と描画 |
| 資源、Quality、cache | GPU上の内部表現 |

これは「Element Domainを単一schemaへ畳まない」既決([2026-07-14処分](reviews/2026-07-14-motion-foundation-known-tech-disposition.md))の上書きではない。畳まないのは個体の意味(payload)であり、共通化するのは所有、identity、時間、失敗という管理面だけである。

### 8.2 能力を一つの巨大interfaceへまとめない

Hostがpluginへ問うのは「この評価結果には何ができるか」の宣言である。

```text
Evaluated Domain
  capabilities:
    - Queryable        範囲・時刻・条件の問い合わせに答える
    - Addressable      安定identityを解決する(§7.1)
    - Selectable       hit test、選択候補を返す
    - Intervenable     保存された介入を解釈する
    - Materializable   Document実体化を提案する
    - Stateful         Simulation状態を持つ(§7.2)
```

この名前と型を今固定するのではない。固定するのは分解の原則である。

- 描画だけのpluginへ、stable IDや物理状態のコストを強制しない。アドレス可能性が必要時のopt-inであること(§7.3)の一般形。
- 能力間に暗黙の含意を作らない。SelectableだからMaterialize必須、AddressableだからStateful、とはしない。
- 能力ごとに互換性を独立判定できる粒度を保ち、新能力の追加は既存能力の解釈を変えない(§8.7)。versionをどこへ、どの形式で持つかは未決とする。
- §5の表が「pluginがHostへ何をできるか」を分離したのと同じ手つきで、「評価結果がユーザーへ何を許すか」を分離する。

### 8.3 共通外殻とplugin固有payloadを分ける

介入(§7.1)や個体参照の保存は、Hostが管理する共通部分と、pluginだけが意味を知る型付きpayloadへ分かれる。次は保存形の方向を示す叩き台であり、fieldの確定ではない。

```text
Host envelope:
  plugin_id / source_id      どの生成元の話か
  element_identity           どの個体への話か
  time_scope                 いつの話か
  payload_version            payloadの版
  (欠落状態、診断、Undo単位はHostの管理面)

Plugin payload(例):
  Pin { target, strength }
  Impulse { vector }
  FoldGlyph { ... }
  CustomFutureAction { ... }
```

Hostは未知payloadの意味を推測せず、そのまま保持する。pluginが欠けてもDocumentは開き、無関係な部分は編集でき、再導入時に介入が復元される — §3で宣言済みの欠落時責任を、介入と個体参照へも適用する。未知の未来を受けるうえで最も強い性質はこれである。

ただしpayloadへ自由なDocument mutationを入れない。plugin固有にできるのは表現の意味だけであり、保存、Undo、参照解決、version管理はHostに残る(§4.1、§5)。

### 8.4 データ構造ではなく要求を契約にする

§7.3の「全個体の一括配列を必須にしない」を契約の水準で言い直す。

```text
悪い固定:
  pluginが全instanceの配列を返す(列挙の契約化)

開いた境界:
  Hostが query / time / range / quality を要求する
  pluginは結果、またはGPU常駐のhandleを返す
```

前者は将来のGPU生成、巨大集合、遅延評価、空間インデックス、stream評価、分散処理を縛る。後者は実装方式を差し替えても契約が生き残る。具体方式を性能fixtureとM5-P0I spikeで選ぶ既決(§7.3)は変わらない。

### 8.5 動詞はpluginが増やし、文法はHostが守る

Hostが操作の全種類を定義し、pluginが計算だけを埋める構造では、遊びがHostの想像力を越えない。選択中の個体へ何ができるかは、pluginが型付きの操作候補として宣言する。

```text
selected element
    ↓ pluginが型付きactionを提案
[Pin] [Kick] [Exclude] [Connect] [Materialize]
    ↓
Host標準の Preview / Commit / Cancel / Undo / 診断
```

- 操作の意味(動詞)はpluginが増やせる。対象の提示、Transient preview、Commit、Cancel、1 Undo、失敗診断は§2.2と[UI操作言語](ui-interaction-language.md)の共通文法を通す。
- 実行の正規化先は§4の既存分類で足りる。one-shotはAuthoring Tool型のtyped command batch(§4.1)、続く意味は介入(§7.1)またはBehavior(§4.2)。
- **自由なscript panelは許さず、未知の動詞は許す。**§5のcapability分離を、選択対象からの操作面へ言い直したものである。

これにより「選択対象から次の一手を発見できるか」(§2.1、§10)の供給源が、Hostの固定リストからpluginの宣言へ広がる。

### 8.6 ファーストパーティへ特権を与えない

標準搭載パーティクル([simulation-model §8](simulation-model.md))を含むfirst-party実装は、公開境界だけで書く(ドッグフード方針)。内部APIを使った瞬間、第三者は同じ遊びへ到達できなくなり、§6の昇格観測も歪む。公開境界で作れない部分が見つかった時は、その表現専用の裏口を足すのではなく、「どの共通能力が欠けているか」を特定し、能力一つの追加として審判する。

first-partyは単なる同梱機能ではなく、次のcreator-authorが模倣、分解、fork、検証できる**実行可能な手本**である。source、scaffold、最小fixture、負例、conformance testを揃え、「標準品だから可能」を残さない。この到達可能性が、利用者から作者へ続く学習曲線の最終段を実物で証明する。

### 8.7 進化の手続き自体をコアの一部にする

未来を予測できない以上、価値は変更を避けることではなく、破壊せず追加できることにある。

- capabilityは互換性を個別に判定できる。versionの保存位置と表現形式は各契約の審判まで固定しない。
- 未知のcapability宣言を黙って適用も削除もせず、unsupportedとして保持・診断しながらDocumentを開ける。
- 未知payloadを丸めず、削らず、推測せず保持する(§8.3)。
- plugin欠落は局所化する(§3)。
- 未知capabilityまたはpayloadが最終結果に必要なら、似た絵へfallbackせずexportを型付きで拒否する。保存・再読込できることと、再現可能にexportできることを混同しない。
- 新しい能力は既存能力の解釈を変更せず、追加だけを行う。既存作品の意味を変更しない([7審判](concept.md#設計と実装の審判)の作品の持続性)。
- 複数pluginで需要が収束してからHostへ昇格する(§6)。昇格前に模範実装と機械判定できるfixtureで反証する。

まとめると、コアは機能集合ではなく次の小さな憲法である。

> **pluginは未知の名詞と動詞を発明できる。Hostはそのidentity、時間、依存、寿命、Undo、資源、失敗を管理する。新しい能力は追加できるが、既存作品の意味を変更しない。**

これならMotoliiは「GPUの次に来るもの」を予測する必要がない。未来の表現を理解していなくても、それがどう存在し、どう要求され、どう保存され、どう失敗するかを受け止められる。

本節は`EvaluatedDomain`、capability enum、envelope schemaの実装許可ではない。名前はすべて説明用の仮名であり、公開契約への反映は§6の昇格と該当ゲート(M5-P0I spike、PP-Gate、解凍手続き)を通す。

## 9. 穴を閉じる設計から、遊びを生む設計へ

既知ソフトの不満、先例、失敗史の調査は、欠落責任や危険な自由を見つけるのに強い。しかし、それだけを最適化するとMotoliiは「既存DCCの穴が閉じたソフト」で止まる。ここからの問いは、既知機能を何個再現したかではなく、**小さな要素を触った時に、ユーザー自身が予想外の関係を発見できるか**である。

これは既知データから正解を採択する段階ではない。次の仮説を、遊べるprototypeと観察で育てる。

### 9.1 楽しさを生む候補

- 一つのShapeを粒、群れ、文字、ブラシ先、コライダーとして再利用できる。
- 画面上の個体を直接つかみ、弾き、固定し、その操作が時間上の介入として残る。
- 音、映像解析、距離、衝突eventを、別の見た目や動きへ型付きで渡せる。
- 同じinstance setを、点、原型Shape、軌跡、線、面等で描き替えられる。
- Group、Text、Path、Particleを同一schemaへ潰さず、SelectorやField等の小さな関係だけ共有できる。
- 失敗、極端値、欠落参照からUndo、Reset、差し替え、Materializeで戻れ、怖がらず試せる。
- 最初は直接操作だけで成功し、必要になった瞬間に規則、identity、Bake等の深い面へ進める。

これらは製品機能の採用リストではない。「組み合わせた時に新しい遊びが発生するか」を試すprobeである。既知ソフトのUIや用語へ似せることより、ユーザーが説明なしに触り始め、次の実験を自分で思いつくかを観察する。

### 9.2 prototypeで記録するもの

未知領域では「便利だった」だけを結論にしない。各prototypeは少なくとも次を記録する。

1. ユーザーが最初に触った対象と、次に試した操作。
2. 予想した結果と、嬉しい意外性／不快な意外性。
3. どこで壊れることを恐れ、どの回復路で再び試せたか。
4. Object、評価個体、規則、状態のどれを操作していると理解したか。
5. 表現量を増やした時、意味を変えずに品質縮退できたか。
6. 一つの用途専用機能だったか、別の遊びへ自然に合成できたか。

成功の兆候は、手順を正しく再現できたことだけではない。説明されていない組合せを自発的に試し、失敗後も別案を続け、作り手が想定しなかったが契約上は正当な表現へ到達することである。

### 9.3 先に固定しないもの

- `InstanceSet`、`Intervention`、`EvaluatedDomain`、capability名(Queryable等)、envelope fieldを現時点の公開型名として固定しない。
- 楽しいdemo一つから万能node graph、汎用expression、自由script panelを導入しない。
- prototypeの内部実装を、そのままDocument正本やplugin契約に昇格しない。
- 初期端末で重かったことを理由に個数上限を保存意味へ入れない。
- 既知ソフトに同名機能がないことを、不採用理由にも採用理由にもしない。

先例調査は引き続き失敗と既知の契約を確認するために使う。§7と§9の既知部分を一次資料で確認しMotolii語彙へ翻訳した調査は[先例翻訳(2026-07-17)](reviews/2026-07-17-extensible-core-prior-art-translation.md)にある(調査であり、本書の原則の設計根拠にはしない)。しかし楽しさの採否は、先例の多数決ではなく、Motolii上での探索行動、合成可能性、回復可能性から判断する。

## 10. 実装前の審判

新しい標準機能、編集plugin、custom UI、Document意味を提案する時は次を確認する。

### 利用者

- 外部manualなしで最初の成功へ到達できるか。
- 選択対象から次の一手を発見できるか。
- 極端な値と想定外の組合せを試せるか。
- 失敗が局所的、可視、可逆で、原因と回復方法が分かるか。
- SimpleからAdvancedへ移っても別の概念体系を覚え直さないか。
- docsが現在の対象と状態から文脈付きで開くか。
- 評価個体を触るためにDocument Objectへ展開する必要がないか。
- 失敗後も別の組合せを試し続けられるか。
- 説明された一用途を越え、自発的な遊びへ進めるか。

### 開発者

- 責任寿命をAuthoring Tool / Behavior / Generator / Renderのどれかで説明できるか。
- 一つの境界が一つの責任を持つか。
- 任意Document mutationなしで実装できるか。
- Host標準command、component、diagnostic、cache、resource lifecycleを再利用するか。
- plugin欠落、取消、Undo、複製、削除、rename、循環、Preview/Exportを試験できるか。
- 同じ需要が収束した時、preset、Tool、primitiveへ追加的に昇格できるか。
- instance数や全件materialize等、現在の実装都合を公開意味へ焼いていないか。
- 新しい表現のために、表現名のHost API(`XxxPlugin`)を増やしていないか。既存能力の合成、または能力一つの追加へ分解したか(§8)。
- 未知のcapability宣言と未知payloadを保持したままsave/reloadでき、plugin再導入で復元されるか。最終結果に必要な能力が欠けるexportは、黙ったfallbackでなく型付き拒否になるか(§8.3、§8.7)。
- first-party実装が公開境界だけで書かれているか。不足があれば裏口ではなく、欠けた共通能力として特定したか(§8.6)。
- 利用者が現在の対象と意味を保ったままTune / Compose / Inspect / Fork / Authorへ進めるか。作者になるために別の製品モデルを覚え直させていないか(§2.4)。

どれかが未決なら、custom panelや汎用APIで先に包まない。非永続prototype、既存typed command、first-party実験の最小範囲へ戻し、意味が安定してから公開境界を増やす。
