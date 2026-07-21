# M3着手前決定 — 操作の意味を固定し、見た目の実値は測って決める

日付: 2026-07-16

ステータス: **設計決定**。M3 G0-2/G0-3/G0-4/G0-7を閉じる。G0-6は導出手順だけを固定してreference screenの人間審判待ち、G0-8は意味だけを固定してM4-K1a実測待ちとする。

関連: [M3仕様](../specs/M3-ui-integration.md)、[UI操作言語](../ui-interaction-language.md)、[UI視覚言語](../ui-visual-language.md)、[UI境界汚染の予防](2026-07-14-m3-ui-boundary-prevention.md)、[プラグインUIモデル](../plugin-ui-model.md)

## 1. 決定の切り分け

M3前に全ての色、寸法、性能値を推測で埋めることを「仕様確定」と呼ばない。先に固定するのは、後から変えるとDocument意味、公開契約、操作互換性を壊す事項である。目視または実測なしに正当化できない値は、測定の入力、手順、審判者、採択条件だけを固定する。

| Gate | 判定 | M3前に固定するもの | 後続証拠を待つもの |
|---|---|---|---|
| G0-2 | **採用・完了** | inputの意味、状態の持ち場と寿命、keymap保存、v1 a11y保証/非保証 | OS別IME実機結果 |
| G0-3 | **2026-07-21再評価中** | `NodeDesc`自動panel fallbackと比較前の自由UI非公開は維持。Host/コミュニティ同一UI kit、sandbox、互換、配布をG0-9へ送る | [React / WebView再選定](2026-07-21-m3-react-webview-runtime-reconsideration.md) |
| G0-4 | **採用・完了** | 計測fixture、環境記録、操作列、集計、CI比較方法 | 製品fps/latency/memory絶対閾値 |
| G0-6 | **手順確定・審判待ち** | token role、生成方式、reference screen、自動/人間審判 | 色、spacing、icon、motion、寸法の実値 |
| G0-7 | **採用・完了** | 共通操作文法、component契約、入口間conformance | 各機能のfixture実装 |
| G0-8 | **意味確定・実測待ち** | 設定の所有、縮退の順序、固定設定の不変条件 | preset予算、安全余白、hysteresis実値 |

G0-6とG0-8はM3全体を止める門ではない。依存するU0e/U0fだけを止め、U0a等の独立レーンは進める。

## 2. G0-2: inputとUI状態の意味

### 2.1 inputは物理キーでなくCommandを正本にする

- `CommandId`は安定した意味IDとし、key、mouse button、modifier、device名を含めない。
- input routerは少なくとも`Press / Release / Click / DragStart / DragUpdate / DragEnd / Cancel`を区別する。clickをpressの別名にしない。
- 物理入力はkeymapで`CommandId`または型付きgesture intentへ変換し、その先のdomain intentにegui/eframe/winit型を出さない。
- IME preedit中の文字入力、候補確定、Enter/Escapeをshortcutより先に処理する。OS差をshortcut special-caseで隠さない。
- pointer capture loss、window focus loss、Escapeは未commit gestureをCancelし、Document変更ゼロにする。

### 2.2 状態の持ち場と寿命

| 状態 | 持ち場 | 寿命 | Undo/Document |
|---|---|---|---|
| layer、clip、parameter、接続、camera等の作品意味 | Document | project保存と同じ | D2 command、Undo対象 |
| keymap delta、UI scale、theme、reduce motion、resource policy | User settings | user単位、projectをまたいで保存 | 対象外。Document/journalへ入れない |
| panel開閉・幅、Timeline density等の作業配置 | Workspace profile | user単位。壊れた場合に既定へ全reset可能 | 対象外。projectの作品意味にしない |
| Stage View pan/zoom/fit、Timeline scroll/zoom、選択中panel等 | Project session | project identity単位のbest-effort cache。欠落・破損時は安全な既定へ戻せる | 対象外。export/evalへ寄与しない |
| hover、focus、drag preview、connection picking、popup、IME preedit | Transient | event/session内だけ | 保存しない。Cancel時変更ゼロ |

Workspace profileとProject sessionはDocument schema、journal、plugin公開契約ではない。U0bでは分類とdomain型だけを作り、永続化形式を発明しない。保存実装を行う時はversion、未知field原本保全、reset、破損fallbackを別タスクの完了条件にする。

### 2.3 keymap保存

- **製品内の全shortcutをユーザーが選べることを不変条件にする。** keyboard、modifier+pointer、key toggle等、shortcutとして提示する入口は全て安定`CommandId`または型付きgesture intentを持ち、機能側で物理key/modifierを直接比較しない。
- 全bindingは追加、置換、複数割当、無効化ができる。Space再生、Delete、Undo/Redo、tool切替、snap、Relative Move、接続開始、Cancel等を「標準だから」という理由で固定しない。
- builtin baseはversionごとに不変とし、user設定には追加、置換、無効化のdeltaだけを保存する。
- 保存形式はversionとsourceを持つJSONとし、migrationは冪等とする。専用設定UIが未完成でも、documented JSONの読込、検証、書出しによって全bindingを変更できることをv1 fallbackとして保証する。
- 未知`CommandId`と移行前原本を失わない。未知項目は実行しないが、読込→保存で黙って削除しない。
- conflictは後勝ち等で暗黙解決せず、競合するbindingとcommandをユーザーへ示す。
- OS予約shortcutやIME preedit中のkey等、Hostが捕捉できない組合せは固定例外へ差し替えず、`UnavailableOnPlatform`等の型付き診断としてbinding単位で示す。pointer capture loss等の安全eventはshortcutではなくCancel intentを発生させる入力境界として残す。

U0c/U0dのconformanceは、shortcutを持つ全登録commandを列挙し、(1)安定`CommandId`がある (2)既定bindingを無効化できる (3)別bindingで同じintentを発行できる (4)機能crateにraw key/modifier分岐がない、を検査する。

### 2.4 v1 accessibilityの保証と非保証

v1で保証する:

- 標準panel/controlのkeyboard focus、可視focus、accessible name、enabled/disabled/error状態。
- menu、Browser、Inspector、settings、transport、保存dialogの主要操作をkeyboardで完結できる。
- Timelineの選択、移動、trim、keyframeの主要編集に、pointerだけに依存しないCommand経路を持つ。
- text/UI scale、contrast、色だけに依存しない状態、reduce motion、日本語IMEを審判する。
- custom描画面の選択対象と操作結果をInspector/TimelineのHost標準componentから検査できる。

v1で保証しない:

- Stage上の自由な空間dragと、全Timeline geometryをscreen readerだけで同等に操作できること。
- 全OS・全screen reader組合せへの適合宣言。
- pluginが独自accessibility treeや独自入力方式を追加すること。

非保証を理由に作品意味をpointer専用UIへ閉じ込めない。空間操作の結果はHost標準Inspectorから数値編集・検査できることをfallbackとする。

## 3. G0-3: plugin UIは「表現を開き、操作文法を閉じる」

> **2026-07-21追記**: 本節は`NodeDesc` fallbackと自由UIを無審査公開しない停止線として保持する。Host/コミュニティ同一kitを長期原則としたため、「自由UIをv1以降も開かない」採否はG0-9で再評価中である。

### 3.1 能力コーパスの判定

| UI用途 | 必要能力 | 判定 |
|---|---|---|
| scalar、bool、enum、color、asset | 既存ValueType→Host control | v1自動生成panelで扱う |
| point、angle、emitter位置 | Host parameter control+宣言的Stage gizmo | 語彙候補。公開struct解凍と順/逆写像仕様後に型ごとに追加 |
| easing/Flow、色調curve | semantic curve型+Host curve editor | v1.x候補。M3だけでValueTypeを発明しない |
| gradient | semantic gradient型+Host editor | v1.x候補。保存・補間・cache意味を先に決める |
| scope/histogram等の可視化 | Host所有の読取専用visualization surface | 保存値を専用UIへ閉じない条件で将来候補 |
| particle emitter編集 | 宣言的gizmo+Host panel | 既存語彙で不足を測り、型ごとに追加 |
| Optical Flares級の専用データモデル編集 | 専用object graph editor | v1 third-party UI契約にしない |
| Element 3D/Particular Designer級のミニアプリ | 独自scene/runtime/editor | v1非目標。必要時に別境界として再審査 |

判定語は**縮小採用**とする。`plugin-ui-model.md`の「Hostが解釈を所有する」原則を採り、能力が未証明な自由描画口は延期する。

### 3.2 v1公開契約

- 全保存parameterは`NodeDesc`から生成するHost標準panelだけで編集・検査できる。
- plugin所有のtoolkit/native UI codeと、plugin所有wgpu UI textureはv1公開契約から削除する。
- file/asset/target picker、keyframe、DataTrack、error、Undo、focus、scale、themeはHost componentが所有する。
- plugin固有の高度用途は、既存variant、宣言語彙の追加、専用semantic ValueTypeの順で解く。自由UIを先に開けない。
- `ParamDef`/`ValueType`/Documentへ語彙を追加する場合は、それぞれの解凍、migration、互換、意味論goldenを先に通す。

将来の例外は拒否しない。ただし「任意コードを埋め込めること」ではなく、座標空間、入力、出力、保存、Undo、失敗、a11yを宣言できる新語彙として審査する。

## 4. G0-4: 性能測定プロトコル

全測定はcommit、release/debug、OS、CPU、GPU、RAM/VRAM、display refresh、DPI/UI scale、window/viewport、fixture hashを記録する。raw結果を保存し、要約値だけを残さない。

| 項目 | 固定手順 |
|---|---|
| warm-up | shader/pipeline生成と初回asset decodeを含む前処理を完了し、同じ操作列を1回捨て走行する |
| 測定 | 操作列を30秒以上または100反復以上。短い方だけで終了しない |
| 集計 | frame/input/render latencyのp50/p95、最大値、deadline miss、dropped/stale generation、CPU/GPU/RAM/VRAMを記録 |
| fixture | empty、通常project、clips 1,000+keys 100,000、Effect Use 500、Stage枠外objectを版管理 |
| 操作列 | idle、window resize/UI scale、Stage pan/zoom、seek/scrub、再生、Timeline scroll/zoom、parameter drag、connection routing |
| CI | 固定runnerでは同一fixtureの基準比で退行を検出。hardware非指定の60fpsを合否にしない |

最初のU1c/U3a/U1f計測後、絶対的なfps、input latency、起動時間、memory上限を独立仕様改訂で採択する。基準機を持たない環境は正しさfixtureだけを実行し、性能合否を偽装しない。

## 5. G0-6: 見た目はUXの投影として導出する

意味role、component state、contrast基準、token単一正本、reference screen、人間審判は[UI視覚言語](../ui-visual-language.md)で固定済みである。次の順番を変えない。

1. 代表操作と状態matrixを置く。
2. 5つのreference screenを同一fixtureで作る。
3. 意味roleだけで構成し、色・spacing・icon・motion候補をtokenへ集約する。
4. contrast、CVD、UI scale、focus、raw literalを機械審判する。
5. 人が通常作業密度、5秒識別、既存componentとの馴染みを目視する。
6. 通った実値だけをG0-6決定記録として凍結する。

したがって、実装者が先に「完成theme」を発明することも、見た目を無期限に未決として局所値を増やすことも禁止する。U0eはreference screenと人間審判が揃うまで完了にしない。

ポップな配色、過剰な丸み、派手なmotion、ケレン味のある専用演出を、親しみやすさや高機能感の代理指標にしない。彩度とmotionを落としても対象・状態・因果・戻し方が成立することをG0-6の前提審判とする。意味を速く伝える色、ghost、仮線、warningは残し、注意だけを奪う演出を削る。

## 6. G0-7: 操作文法を共通部品の契約にする

[UI操作言語](../ui-interaction-language.md)を採用し、`Discover → Target → Preview → Commit / Cancel → Inspect → Undo`を共通状態遷移とする。Direct / Tool / Advancedは同じDomain Intentへ正規化し、入口差をDocumentへ保存しない。

新機能は次を先に提出する:

- Domain Intent、target型、scope、永続物、1 gestureのUndo単位。
- preview/hover/focus/error/Cancel/capture lossを含むcomponent state matrix。
- Simpleに残すsemantic badgeとAdvancedで追加表示する由来/評価順。
- 入口違いのserialize同値、Cancel変更ゼロ、拒否理由のfixture。

共通componentから漏れた局所picker、独自popup、独自hover/focus/error、plugin専用保存値は未完成として扱う。

## 7. G0-8: resource値はM4の事実から決める

`Auto / 省メモリ / 性能優先 / Custom`の名称と意味、User settings所有、Document/Final不変、固定解像度ではscaleを変えないこと、容量pressureとdeadline missを分離することは固定する。

各presetのVRAM/RAM/disk値、安全余白、hysteresis、縮退開始/復帰点はG0-4手順とM4-K1aの実測後に決める。backendが報告する「空きVRAM」を正本にせず、HostのResourceLedgerと実際のallocation失敗を根拠にする。

## 8. M3開始時の停止条件

- UI都合でDocument field、px/DPI、egui/eframe/winit型、event列を保存したくなったら停止する。
- plugin custom UI、WidgetHint、新ValueType、workspace永続形式を「ついで」に公開したくなったら停止する。
- 配色/spacing/iconの局所literal、測定前のfps/memory公約、基準機不明の性能合否を追加したくなったら停止する。
- 既存共通componentで表せない場合は、機能内で作らずcomponent契約の拡張として先に審査する。

この決定によりM3全体の設計判断待ちは解消する。残る待ちは、対象タスクに明記されたM2/M4依存、G0-6の目視、G0-8の実測であり、実装者が埋める未決事項ではない。
