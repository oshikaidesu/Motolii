# M3着手前決定 — 操作の意味を固定し、見た目の実値は測って決める

日付: 2026-07-16

ステータス: **設計決定**。M3 G0-2/G0-3/G0-4/G0-7を閉じる。G0-6は導出手順だけを固定してreference screenの人間審判待ち、G0-8は意味だけを固定してM4-K1a実測待ちとする。

関連: [M3仕様](../specs/M3-ui-integration.md)、[UI操作言語](../ui-interaction-language.md)、[UI視覚言語](../ui-visual-language.md)、[UI境界汚染の予防](2026-07-14-m3-ui-boundary-prevention.md)、[プラグインUIモデル](../plugin-ui-model.md)

## 1. 決定の切り分け

M3前に全ての色、寸法、性能値を推測で埋めることを「仕様確定」と呼ばない。先に固定するのは、後から変えるとDocument意味、公開契約、操作互換性を壊す事項である。目視または実測なしに正当化できない値は、測定の入力、手順、審判者、採択条件だけを固定する。

| Gate | 判定 | M3前に固定するもの | 後続証拠を待つもの |
|---|---|---|---|
| G0-2 | **採用・完了** | inputの意味、状態の持ち場と寿命、keymap保存、v1 a11y保証/非保証 | OS別IME実機結果 |
| G0-3 | **2026-07-21再評価中 / 2026-07-22軸分離** | `NodeDesc`自動panel fallbackと比較前の自由UI非公開は維持。公開kit、sandbox、互換、配布はG0-3 / GAP-13で判定し、G0-9は製品surface証拠だけを渡す | [軸分離決定](2026-07-22-m3-surface-extension-axis-separation.md)、[React / WebView再選定](2026-07-21-m3-react-webview-runtime-reconsideration.md) |
| G0-4 | **採用・完了** | 計測fixture、環境記録、操作列、集計、CI比較方法 | 製品fps/latency/memory絶対閾値 |
| G0-6 | **手順確定・審判待ち** | token role、生成方式、reference screen、自動/人間審判 | 色、spacing、icon、motion、寸法の実値 |
| G0-7 | **採用・完了** | 共通操作文法、component契約、入口間conformance | 各機能のfixture実装 |
| G0-8 | **意味確定・実測待ち** | 設定の所有、縮退の順序、固定設定の不変条件 | preset予算、安全余白、hysteresis実値 |

G0-6とG0-8はM3全体を止める門ではない。依存するU0e/U0fだけを止め、U0a等の独立レーンは進める。

## 2. G0-2: inputとUI状態の意味

### 2.1 inputは物理キーでなくCommandを正本にする

- `CommandId`は安定した意味IDとし、key、mouse button、modifier、device名を含めない。
- 組み込み`CommandId`の文字列表現は`motolii.`で始め、その後に1個以上の`.`区切りの意味segmentを置く。各segmentは小文字ASCII英字で始まり、以後は小文字ASCII英数字または`_`だけを使う。空segment、空白、大文字、先頭・末尾`.`、連続`.`を拒否する。例は`motolii.edit.delete_targeted_items`。表示名、翻訳文言、物理入力、画面入口からIDを生成せず、それらが変わってもIDを変えない。
- この文字列表現はkeymap・JSON codecを定めない。U0c-1は検証済みIDとmetadata registryだけを作り、U0dが同じIDを永続keyとして利用する。adapter内の一時kindは`CommandId`へ昇格しない。
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
- U0dのtoolkit非依存binding語彙は次の閉集合とする。`KeyToken`はレイアウト解決後の論理ASCII英字/数字と、`Space / Enter / Escape / Delete / Backspace / Tab / ArrowUp / ArrowDown / ArrowLeft / ArrowRight / Home / End / PageUp / PageDown`。英字tokenは小文字の単一表現とし、大文字入力は`Shift`+同tokenで表す。OS scancodeやwinit/egui keyを正本にしない。`Modifier`は`Primary / Control / Meta / Alt / Shift`の集合で、列挙順へsortし重複を除く。1 Gesture内で`Primary`と`Control`または`Meta`を併記した場合は型付き拒否する。`Primary`はresolverへ注入したplatform command modifier(`Control`または`Meta`)へ競合判定前に展開し、`Control`/`Meta`は明示割当用として区別する。`PointerButton`は`Primary / Secondary / Middle / Auxiliary1 / Auxiliary2`。F-keyや記号等を追加する場合は局所文字列で迂回せず、この閉集合の仕様改訂を先に行う。
- `Gesture`は`Keyboard { key, modifiers, phase }`、`ModifierPointer { button, modifiers, phase }`、`KeyToggle { key, modifiers }`の3種だけとする。`Keyboard.phase`は`Press / Release`、`ModifierPointer.phase`は`Press / Release / Click / DragStart / DragEnd`だけを許し、`DragUpdate`は開始済みgestureのTransient継続、`Cancel`は安全入力としてkeymap対象にしない。`KeyToggle`は同じ割当からPress/Releaseの2つの有効triggerへ展開し、resolverはhold状態を持たない。Context/scope、OS名、表示文字列、px、device IDをGestureへ入れない。U0d-1のbinding targetは登録済み`CommandId`だけとし、並立する第二のgesture intent正本を作らない。
- deltaの対象同一性は上記正規化後の`Gesture`完全一致で決める。実行時の競合は、`Primary`をplatform command modifierへ置換し、`KeyToggle`をPress/Releaseへ展開した後の`EffectiveTrigger`(keyboardまたはpointer、token/button、modifier集合、phase)完全一致で決める。異なるGestureでも同じEffectiveTriggerが異なる`CommandId`へ写れば競合であり、後勝ち・先勝ちにせず型付き診断を出してそのtriggerを実行mapへ載せない。同じCommandへ複数EffectiveTriggerを割り当てることは競合ではない。
- 全bindingは追加、置換、複数割当、無効化ができる。Space再生、Delete、Undo/Redo、tool切替、snap、Relative Move、接続開始、Cancel等を「標準だから」という理由で固定しない。
- user deltaは順序依存にしない。`Add { gesture, command }`はbaseに無いGestureを追加し、`Replace { gesture, command }`と`Disable { gesture }`はexact一致するbase Gestureだけを対象にする。同一Gestureへ複数delta、存在しないbaseへのReplace/Disable、base GestureへのAddは型付き診断とし、暗黙適用しない。複数割当は異なるGestureのAddで表す。
- builtin baseはversionごとに不変とし、user設定には追加、置換、無効化のdeltaだけを保存する。
- 保存形式はversionとsourceを持つJSONとし、migrationは冪等とする。専用設定UIが未完成でも、documented JSONの読込、検証、書出しによって全bindingを変更できることをv1 fallbackとして保証する。
- 未知`CommandId`と移行前原本を失わない。未知項目は実行しないが、読込→保存で黙って削除しない。
- conflictは後勝ち等で暗黙解決せず、競合するbindingとcommandをユーザーへ示す。
- platform入力はcommand modifier(`Control`または`Meta`)と予約`EffectiveTrigger`集合を持ち、resolverからOS APIを呼ばない。予約triggerは`UnavailableOnPlatform`型付き診断としてbinding単位で示し、固定例外へ差し替えない。IME preeditはU0c-2のruntime gateで扱い、静的予約へ混ぜない。pointer capture loss等の安全eventはshortcutではなくCancel intentを発生させる入力境界として残す。
- U0d-1でdeltaが未登録`CommandId`を参照した場合は型付き診断として実行mapへ載せない。読込原本と未知field/未知IDを失わない責任はU0d-2のcodecが持ち、U0d-1へopaque永続payloadを入れない。
- U0d-2のwire shape、version/source、未知入力処分、原本保全、migration固定点、limits、error境界は[U0d-2 keymap JSON codec契約](2026-07-20-m3-keymap-codec-contract.md)を正本とする。初版currentはv1で、存在しない旧形式をv0として発明しない。

U0c/U0dのconformanceは、shortcutを持つ全登録commandを列挙し、(1)安定`CommandId`がある (2)既定bindingを無効化できる (3)別bindingで同じintentを発行できる (4)機能crateにraw key/modifier分岐がない、を検査する。

U0d-3では「shortcutを持つ」を`builtin_command_registry()`の全commandへ広げ、
製品既定キーの有無にかかわらず例外ゼロを審判する。各commandへ一意な合成base Gestureと
別の合成Gestureを割り当て、baseを`Disable`した旧`EffectiveTrigger`が実行mapから消え、
別Gestureの`Add`をresolveしたtriggerが同じ`CommandId`を返すことをregistry全量で検査する。
そのIDを`ImeGateState::Inactive`の`InputRouter::route(NormalizedInput::Command { .. })`
へ渡し、registry metadataと同じ`DomainIntent`を持つ`RouterOutput::Intent`が出ることを
必須とし、registryの直読みだけを合格にしない。`CancelInFlightGesture`を含む全commandで
同じ経路を通すため、各審判前に`NormalizedInput::Phase(InputPhase::DragStart)`をrouteして
in-flight状態を作る。Cancelだけを除外したりrouter契約を弱めたりしない。
これはresolverのconformance fixtureであり、Reduce MotionやReset Workspace等の
製品出荷用既定Gestureを決めない。製品builtin base内容とpresetは別の製品内容決定であり、
U0d-3で推測して追加しない。

U0d-3のraw input監査は次の機械境界に限定する。

- 対象はworkspace memberの`crates/*/src/**/*.rs`と`plugins/*/src/**/*.rs`にある
  全product source。`tests/`、`examples/`、build scriptは対象外だが、`src/`内の
  `#[cfg(test)]` moduleは対象に含め、member・source file・CommandIdのallowlistを置かない。
- 全対象fileをRust parserでAST化し、parse失敗も監査失敗にする。line comment、入れ子の
  block comment、通常/raw/byte/raw-byte string、char literal内はAST上の値なので照合せず、
  lifetimeをcharと誤認しない。attribute、macro invocation、`macro_rules!` definitionの
  token treeもcodeとしてpathとidentifierを再帰監査し、AST visitorの外へ逃がさない。
- ASTの識別子境界とpath segment列で
  `egui::Key / egui::Modifiers / egui::PointerButton / egui::Event /
  egui::InputState / egui::RawInput / winit::keyboard / KeyCode / PhysicalKey / NamedKey /
  ModifiersState / MouseButton / KeyEvent / ElementState / RawKeyEvent /
  winit::event::KeyEvent / winit::event::ElementState / winit::event::RawKeyEvent /
  winit::event::WindowEvent / winit::event::DeviceEvent / WindowEvent::KeyboardInput /
  WindowEvent::ModifiersChanged / DeviceEvent::Key /
  .key_pressed( / .key_released( / .key_down(`を拒否する。toolkitを直接依存できる
  `motolii-ui`のproduct sourceでは、型推論で`InputState` pathを隠す
  `egui::Context`と`egui::Ui`の`.input(`/`.input_mut(`を、receiver名によらず
  method identifierで拒否する。
  `use egui::{Key, Modifiers as EguiModifiers}`のようなuse treeとaliasも元pathへ展開して
  拒否し、method callは空白の有無によらずmethod identifierで判定する。
  `motolii-ui::keymap::Modifiers`等の正規化済みdomain型名は拒否対象ではない。
- scannerはuse treeの各leafを元pathで照合する。`egui`/`winit`を起点とするrename、
  crate/module alias、`extern crate ... as ...`、globは宣言自体を監査失敗にし、
  raw APIをaliasの後ろへ隠せないようにする。raw inputと無関係なaliasはfile全域へ
  flattenせず、別module/block scopeの同名aliasやcycleをraw input違反にしない。
- 監査自身の負例はbrace import、alias、完全修飾path、method callの空白差を拒否し、
  `winit::event::KeyEvent`のuse alias、macro invocation/definition内の禁止pathも拒否する。
  `ctx.input(|i| i.modifiers.ctrl || !i.keys_down.is_empty())`、
  `ui.input(|i| !i.events.is_empty())`、
  `egui::{InputState, RawInput}`、`winit::event::{DeviceEvent, RawKeyEvent}`も拒否する。
  通常/raw/byte/raw-byte string、char、line comment、入れ子block comment内の
  禁止語を無視することを固定する。
- U1a-2以前の製品sourceではtoolkit raw inputを読むadapterの許可fileはゼロだった。
  U1a-2では`crates/motolii-ui/src/layout_runtime_adapter.rs`だけを、focus中separatorの
  Arrow / Home / Escapeをprivate layout actionへ正規化する局所adapterとして許可する。
  入力はeguiのfocus中separator response、raw key event、`ImeGateState`、継続resize状態。
  安全中断だけは`egui::Event::PointerGone`と`egui::Event::WindowFocused(false)`も入力し、
  それ以外のraw pointer/window eventを読まない。出力はprivateなresize/reset action、
  または既存のtoolkit非依存`SafetyInterrupt::{PointerCaptureLost, WindowFocusLost}`だけで、
  `CommandId`、`DomainIntent`、D2 commandへ接続しない。raw安全eventは一度だけ
  `SafetyInterrupt`へ正規化し、同じ値を
  `InputRouter::route(NormalizedInput::SafetyInterrupt(..))`とprivate layout reducerへ
  配送する。routerはglobal gesture、layout reducerは継続resizeだけをcancelし、layout
  reducerはrouterの`DomainIntent`を入力しない。`ImeGateState::PreeditActive`中はArrow / Home /
  Escapeをlayout actionへ変換せずIMEを優先する。pointer capture lossはkeyとは別の
  `SafetyInterrupt`として、window focus lossと同様に継続resizeをcancelするが、preedit中の
  EscapeだけでなくIME非active時のEscapeもSafetyInterruptへは読み替えず、private cancel
  actionに限定する。pointer dragとdouble clickは標準
  `egui::Response`から読む。Viewのrestore/resetは標準menu/buttonのkeyboard activationを
  使い、独自raw key分岐を持たない。
- 上記単一file以外はkey/modifier variantだけでなく`WindowEvent`/`DeviceEvent`等の
  toolkit event面全体を引き続きゼロとし、resize/close等を理由に暗黙のevent adapterを
  作らない。許可file内でもlayout以外のkey、modifier、text、IME event、上記2種以外の
  raw pointer/window event、device eventを読まず、alias/helper/macroで許可範囲を
  広げない。監査は許可fileのpathだけで全禁止APIを免除せず、許可する識別子と入力種別の
  閉集合も検査する。
  将来toolkit eventを`NormalizedInput`へ変換するprivate adapterが必要になった時は、
  adapterの入力、出力、IME優先、SafetyInterrupt、許可fileを仕様改訂で先に固定し、
  許可file外ゼロの負例、`CommandId`/keymap resolverを迂回した`DomainIntent`直接発行の
  禁止、toolkit型の公開API非流出も同じ改訂で審判化する。機能側やresolverへ例外を広げない。
- 上記AST監査を主審判とし、字句・path・use-treeの規則を満たさないscannerでは
  U0d-3を完了にしない。既存
  `check-ui-toolkit-deps.sh`の「`motolii-ui`外にtoolkit直接依存なし」、
  公開型走査は補助審判であり、AST監査の代替にしない。

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

> **2026-07-21追記、2026-07-22訂正**: 本節は`NodeDesc` fallbackと自由UIを無審査公開しない停止線として保持する。公開kit、sandbox、権限、互換、配布はG0-3 / GAP-13で再評価する。G0-9は標準製品surfaceのplatform証拠を提供するが、その合格だけで自由UIを公開しない。

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

[UI操作言語](../ui-interaction-language.md)を採用し、
`Discover → Target → [Preview] → Commit → Inspect`と
`Target / Preview → Cancel`を6状態の共通状態遷移とする。Cancelは変更ゼロで待機へ戻る。
UndoはこのTransient状態機械の外にある通常のD2 commandであり、Direct / Tool / Advancedは
同じDomain Intentへ正規化し、入口差をDocumentへ保存しない。

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
