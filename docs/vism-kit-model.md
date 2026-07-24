# Vism / Kitモデル — 小さな表現を完成した用途へ組む

作成日: 2026-07-17

状態: **設計原則決定／公開schema・拡張子・runtime未決**。`Core / Vism / Kit / Project`の責任分離、Vism同士を直接参照させず型付き入出力をKitが接続する原則、v1のKitをmaterialize型とする方向を定める。2026-07-24にBPM／拍リズムの製品所有者をVism providerへ決定した。`BeatMap`、`TimeGuide`、`KitDefinition`等の型名は説明用の仮名であり、M2 Document、plugin公開API、package形式の実装許可ではない。

関連正本: [Vismコンセプト](vism-package-concept.md)、[小さなコアと探索可能な拡張](extensible-core-model.md)、[プラグイン作者向け規約](plugin-authoring.md)、[Vism実装計画](reviews/2026-07-17-vism-implementation-plan.md)

## 1. 結論

```text
Core     = 文法
Vism     = 語彙
Kit      = 接続済みの文章／用途セット
Project  = 実際の作品
```

- **Core**は時間、型付き入出力、接続、identity、保存、Undo、資源、失敗を管理する。
- **Vism**は一つの小さな映像表現またはproviderを実装する。
- **Kit**は複数Vism、接続、初期値、素材要求を目的単位へまとめる。
- **Project**はKitの展開結果を通常のObject、Effect、Data接続として所有する。

Vism AがVism BのIDを直接要求しない。Vismは必要な**型**を宣言し、Kitが具体的なproviderを選ぶ。

```text
禁止:
  Particle Vism → org.example.beat-analyzer.vism

採用方向:
  Particle Vism ← BeatEvents
                    ↑
                 Kitがproviderを選択
```

Kitは依存を隠す巨大pluginではない。依存と接続をVismの外へ持ち上げ、Hostが導入前に検査できる宣言層である。

本書の**Kit**は利用者向けのVism構成概念である。Rustの`motolii-testkit`（テスト支援）やUI accessibilityで使うAccessKitとは無関係で、コード上の型名・crate名は未決とする。

## 2. なぜKitが要るか

小さなVismだけを許すと、利用者が毎回provider、consumer、接続、初期値を組み立てることになる。逆に「音に合わせて粒子を動かす」一式を一つのVismへ詰めると、音声解析、Beat解釈、Particle、UI、更新責任が再び巨大pluginへ集まる。

Kitはこの二択を避ける。

| 層 | 再利用の単位 | 変更理由 | 欠落時 |
|---|---|---|---|
| Vism | Beat解析、Particle、Glow等の小さな表現 | 計算、parameter、表現意味の更新 | 該当表現だけunavailable |
| Kit | Music Reactive、Lyrics Starter等の用途 | provider選択、接続、初期値、素材構成の更新 | 展開前に依存不足を診断 |
| Project | 一つの作品内の具体instance | 制作者の編集 | 原本を保持し無関係編集を許可 |

Kitにより、初心者は完成した用途を一つ追加でき、上級者は展開されたVismと接続を開いて組み替えられる。SimpleとAdvancedを別の仕組みにしない。

## 3. Vismは実装名でなく型を要求する

Vismが宣言できる依存を少なくとも次へ分ける。

| 依存 | 例 | 原則 |
|---|---|---|
| **型付きdata input** | BeatEvents、DataTrack、State sample、texture | 第一選択。providerの実装を知らない |
| **Host execution capability** | GPU texture処理、Host所有Simulation、Asset解決 | Core／互換Hostがlifecycleと資源を管理 |
| **Asset requirement** | 音声、font、SVG、glTF | 内容hashと型で宣言し、暗黙filesystem参照にしない |
| **Authoring convenience** | BPM Grid UI、MIDI import、解析panel | Vismの評価意味から分離し、Host標準投影またはToolにする |

たとえばParticle Vismは「BPM Grid機能」を呼ばない。`BeatEvents`を入力として受ける。供給元は固定BPM、音声解析、MIDI、手動tap、fork固有live inputのいずれでもよい。

新しいHost capabilityを追加する前に、既存の型付きinputで表せないかを審判する。表せるものをambientなHost APIへしない。これはUnixのpipeと同じで、producerとconsumerを互いの実装から切り離す。

### 3.1 それでも残る依存

依存そのものは消えない。Kitが次を明示する。

- 必要なVism identityと互換version。
- 各entryの型付きinput／output対応。
- 必須接続と任意接続。
- 初期parameterと利用者へ公開するcontrol。
- 必要assetとprovider候補。
- 非対応時に展開を止める条件。

Vism package内部のlibrary依存と、作品意味を構成するVism間依存を混ぜない。前者はbuild／供給網、後者はKit／Project graphの責任である。

## 4. Kitの責任

概念上のKitは次を持てる。

```text
kit identity + version
required Vism constraints
typed connections
initial parameters
exposed controls
declared assets
optional provider choices
preview / example / diagnostics
```

具体field、container、拡張子は未決である。`.vism`へ同梱する、別形式にする、package metadataの一種にする、いずれもまだ決めない。

Kitが持たないもの:

- 任意Document mutation code。
- 独自Timeline、独自Undo、独自window shell。
- plugin内部の隠れ接続。
- Host名やlayer名を検索する参照。
- 展開後のProjectを無断で書き換える更新処理。
- cache、Bake結果、Workspace配置。

実行codeが必要ならVismである。KitはHostが検査・展開できる宣言に留める。

### 4.1 Preset／Recipeとの境界

- **Preset**は原則として一つの既存能力のparameter初期値であり、新しいprovider依存や実行identityを持たない。
- **Recipe**は現在のdocsで生成規則や再現手順を広く指す一般語として残る。
- **Kit**は複数Vismの要求、型付き接続、初期値、素材要求を一つの利用目的へまとめる正式な責任層である。

単一Vismの設定だけならPreset、複数Vismの依存と接続を再現するならKitを第一候補とする。名称だけを変えて巨大templateをKitと呼ばない。

## 5. v1 Kitはmaterializeする

最初のKitはProjectへ常駐するruntimeにしない。

これは目標契約であり、現行`DocumentWriter::apply_command`はcommandを逐次適用するだけで、batch全体のpreflight／rollbackを提供していない。M3-U9aまたは独立に採択された同等境界が成立する前に、同一`GestureId`へ複数commandを積むだけで「失敗時変更ゼロ」を公約しない。

```text
Kitを選ぶ
  → 必要Vism／asset／型をpreflight
  → Project snapshotに対して展開案を作る
  → 全体成功時だけ1 macro commit
  → Vism instanceと接続を通常編集
```

不変条件:

1. 1回のKit追加は1 Undo。
2. 欠落、型不一致、循環、resource超過、Cancel、stale snapshotではDocument変更ゼロ。
3. 展開後はKit runtimeがなくても通常のProject意味が残る。
4. Kit更新で既存Projectを自動変更しない。
5. 展開されたVismのidentity、version、payloadをProjectが通常規則で保持する。

linked Kit、Kit更新の追従、共有Kit Definitionは将来の別審判とする。最初から「templateを直すと全作品が変わる」責任を持ち込まない。

## 6. BPM Gridを分解した例

現在「BPMに合わせる」と一語で呼んでいる機能には、少なくとも四つの責任がある。

```text
BPM Rhythm Vism（Beat Map provider）
  固定BPM／tempo map／解析／MIDI → BeatEvents

Beat Motion Vism
  BeatEvents → Position / Scale等の値

Beat Grid projection
  TimeGuide列 → Timeline表示／snap（Host）

Music Reactive Kit
  provider、motion、particle、guideを接続
```

Coreは「キックで跳ねる」「サビ頭で弾ける」を知らない。Coreが知るのは時刻、型、接続、評価順、循環拒否、保存、Undo、欠落、Timelineへの汎用guide投影だけである。

BPM／拍リズムはCoreの特殊機能ではなく、型付きリズムdataを供給するVism providerが所有する。Coreは時刻、型、接続、評価順を持ち、TimelineのBeat Gridはそのdataを読むHost projectionである。

BPM Rhythm Vismへ入力できる候補:

- Projectの固定BPM／tempo map。
- 手入力またはtapでmaterializeしたtempo／meter。

同じrhythm出力を供給できる別provider:

- 音声解析Vism。
- MIDI由来provider。
- fork固有のlive tempo provider。

Beat MotionやParticleはproviderを知らず、同じ`BeatEvents`を受け取る。

`BeatEvents`、`TimeGuide`は説明用の仮名である。現行`DataTrack<Value>`で十分か、event／label／meterを持つ別のtyped domainが必要かをfixtureで比較する前に公開型を追加しない。

## 7. Motoliiとforkの関係

Vismの憲法上の可搬先は、任意の映像ソフトではなく**Motoliiの公開契約を継ぐ互換Host／fork群**である。他製品adapterは可能性として残すが、Vism完了条件にはしない。

```text
Base capability      全互換forkが意味を維持する小さな契約
Optional capability  Simulation、Tracking等の明示要求
Fork capability      名前空間付きの実験能力
```

fork固有Vismを禁止しない。ただし、fork固有能力へ依存する前に型付きdata inputとKit provider差替えで表現できないかを審判する。

```text
標準Kit: BeatEvents provider = Static Beat Map
fork Kit: BeatEvents provider = example-fork.live-midi
```

consumer Vismを変えずにproviderだけを差し替えられるのが望ましい。fork固有能力は名前空間、version、非互換理由を宣言し、未対応Hostは原本を保持して評価を拒否する。成功したfork能力は利用例と複数fixtureを得てから追加的に上流へ昇格できる。

## 8. 現在のコードはどこまで来ているか

Vismという名称より先に、実行核の多くは実装されている。

| 必要な機構 | 現在の実装 | 判定 |
|---|---|---|
| 表現traitと静的登録 | `motolii-plugin`: `PluginKind`、各trait、`PluginRegistry` | pre-Vism kernel実装済み |
| 自己記述parameter | `NodeDesc`、`ParamDef`、検証 | 実装済み |
| 型付き時系列 | `DataTrack`、`DataTrackId`、`ParamSource::Data`、`ParamDriverPlugin` | 値pipeの基礎実装済み |
| Project内recipe | plugin ID、version、params、unknown `extra` | 実装済み |
| 共有Effect lifecycle | Effect Definition／Use | 実装済み |
| 欠落・未来版 | 開く=保持+警告、export=型付き拒否 | 実装済み |
| Host所有state | SimulationPlugin／StateTrack | 設計済み、実コード待ち |
| Kit | Recipe／presetの断片的記述のみ | 製品概念として未定義だった |
| 汎用typed port | `DataTrack<Value>`中心 | structured event／domainは未決 |
| 動的Vism配布 | なし | v2・未決 |

現在の継ぎ目も明確である。

- `PluginId(pub &'static str)`と`&'static dyn Plugin`は静的registryの実装事情である。
- `motolii-doc`はfirst-party plugin ID／kind／versionを既知表としてミラーしており、fork追加をruntimeには解決できない。
- `Document.bpm`は単一の有理BPMを特別なトップレベルfieldとして持つ。
- `DataTrack<Value>`は連続値には強いが、Beat event、bar、label、meter等のstructured出力をまだ表現しない。

したがって「Vismをゼロから作る」のではない。既存plugin／Document／DataTrackをpre-Vism実装として反証し、固定された継ぎ目だけを追加的に解く。

### 8.1 Vism候補の判定はトリアージである

「利用者が名前で探すか」「独立した入出力／parameterがあるか」「独立したversion／欠落lifecycleに意味があるか」は、Vism候補を探すための質問であり、admission testではない。Clear Filterも形式上は入出力とparameterを持ち、SineもCore primitiveとprovider Vismの両方に解釈できるため、この三問だけでは処分できない。

Opacity／Sineを最初に選ぶ理由はVism認定済みだからではなく、現存するFilter／ParamDriverの最小実装として公開境界を反証しやすいからである。一Vism一entryか、複数entryか、fixture専用かという処分はKit比較とidentity fixtureの前に固定しない。

## 9. 現行BPMからBPM Rhythm Vismへどう移るか

`Document.bpm`を直ちに削除、汎用化、Vism参照へmigrationしない。M2で保存済みの意味を、今回の整理だけで変更してはならない。

長期の製品所有は決定した。**固定BPMを含むBPM／拍リズムはBPM Rhythm Vismが供給する**。現行`Document.bpm`はその反対を示すCore正本ではなく、旧Projectとの互換を保ったままBPM Rhythm Vism相当へ値を渡すpre-Vism入力源として扱う。

未決なのは所有者ではなく移行と公開形である。

1. `DataTrack<Value>`で十分か、beat／bar／meterを持つtyped rhythm domainが必要か。
2. Beat Gridを汎用Time Guide projectionへどう畳むか。
3. 旧Projectを追加migrationなしでBPM Rhythm Vism相当へ投影できるか。
4. 現行`Document.bpm`を将来どの仕様改訂とmigrationで縮退／撤去するか。
5. 手入力BPM、tempo map、tap、MIDI、解析providerを同じ出力型へどう接続するか。

処分にはGR-PV、M2仕様改訂、migration、旧reader、意味論golden、反対側レビューが必要である。UIから先に`BeatMap`やKit fieldをDocumentへ足さない。

## 10. Identityと依存の層

最低でも次を混ぜない。

| identity | 所有者 | 役割 |
|---|---|---|
| Vism package identity | 配布系 | 作者、version、由来、導入、更新 |
| Vism entry identity | 表現契約 | Hostが評価する入口 |
| Kit identity | 構成作者／配布系 | Vism要求、接続、初期値の版 |
| Project instance identity | Document | Undo、複製、参照、欠落復元 |
| Artifact／署名identity | build／trust系 | 同じsource／版から得た実体の由来 |

Kit identityをProject instance identityとして流用しない。materialize後の各instanceはProjectが採番する。Vism package identityをentry IDやartifact hashへ流用しない。

## 11. 開発が単純になる理由

目標境界が成立すれば、機能要求をコア改造ではなく、三つの独立した発注へ分けられる。

```text
1. provider Vism: 入力Aから型Bを作る
2. consumer Vism: 型Bから映像／値Cを作る
3. Kit: providerとconsumerを接続して初期値を与える
```

各VismはHost全体、Timeline UI、他Vismの内部を理解しなくてよい。LLMへも「この型を受け、この型を返し、公開契約とconformanceを通す」という小さな発注ができる。

ただし現行`ParamDriverPlugin`は入力portを持たず、`DataTrackId`から既存parameterを駆動する経路だけが実装済みである。「consumer Vism」は将来境界の説明語であり、現行APIで実装可能とはみなさない。方式決定は[Vism実装計画 VSM-B2](reviews/2026-07-17-vism-implementation-plan.md)と[Vism-ready反対側レビュー採否](reviews/2026-07-17-vism-ready-counter-review-disposition.md)に従う。

試験も分かれる。

- Vism単体: 決定論、純関数、GPU、resource、typed error。
- 接続: 型一致、循環拒否、欠落provider、cache dependency。
- Kit展開: preflight、1 Undo、Cancel／失敗変更ゼロ、展開後runtime不要。
- Project lifecycle: save/reload、未知保持、再導入復元、strict export。

失敗が局所化し、作者と利用者の両方が原因を理解できる。

## 12. 停止線

次を仕様決定前に実装しない。

- `KitDefinition`等の恒久Document schema。
- Kit専用runtime、独自graph editor、custom panel。
- `.kit`等の拡張子またはKit container。
- Vism IDを直接埋め込むconsumer plugin API。
- BPM専用の新しいplugin kind。
- `BeatMap`、`BeatEvents`、`TimeGuide`を証拠なしにuniversal型へすること。
- Kit更新による既存Projectの自動追従。
- fork capabilityの名前空間／互換規則を決めずに独自Host APIを公開すること。
- 現行`Document.bpm`の意味変更や削除。

fixtureは段階を飛ばさない。VSM-A7では「現行固定BPM → DataTrack値列 → 既存parameter結線」だけを扱う。package／entry／Kit／Project instance／artifactのidentity fixture（VSM-B0）と成果物境界（VSM-B1）の後、VSM-B2で初めて「provider → consumer → materialize Kit」の方式を比較する。これでpixel Filterだけでは見えなかったtyped provider、非画素出力、Timeline投影、fork差替え、Kit展開を、現行APIへ架空のconsumerを足さず反証する。
