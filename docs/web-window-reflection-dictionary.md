# Web窓への製品反映辞書

日付: 2026-08-14  
状態: **決定**

## 目的

MotoliiのUIをWebで先に詰める時に、Webだけの実装、nativeだけの実装、意味の二重管理を作らないための辞書。
Web窓とnative窓は同じ製品coreとsnapshotを読む二つのprojectionであり、一方を他方へ移植しない。
各surfaceと操作は、実装前に必ず次の反映種別へ分類する。

## 中心原則: Web実行はprojection adapter最小化の証拠

ある利用者成果がWeb窓で次の一周を完了できるなら、その成果の製品意味、状態機械、通常操作はOS固有実装から分離できている。

`Discover -> Target -> Preview -> Commit/Cancel -> Inspect -> Undo`

この時各window adapterへ残してよいのは、同じtyped contractへ接続するOS／browser／GPU／codec／storage capabilityだけである。
Web窓またはdesktop窓を成立させる時にlayout、shortcut、Undo意味、parameter state、Rerun sceneを再実装する必要があるなら、共有routeはまだ原本化できていない。

ただし次を混同しない。

- browser APIで実行できることは、desktop製品でもbrowser実装を使い続ける義務ではない。
- Webで見えたことは、Rust coreのWasm compile、実Document接続、native実機、Preview／Export同値の証拠ではない。
- native高速化は実装substrateの交換であり、製品意味やUI仕様の分岐ではない。

## 反映種別

| 反映種別 | 意味 | 実装規則 |
|---|---|---|
| `SHARED_SOURCE` | Webで直接実行可能な共有製品sourceを原本にし、Web／macOS／Windowsが同じcomponentと状態投影を読む | 一つのwindowやfixtureを原本扱いせず、見た目・配置・通常操作をplatform別に複製しない |
| `SHARED_RUST_CORE` | 同じRust実装をnative libraryとWasmの両方で実行する | TypeScriptへ意味を写し直さず、同じ型・command・oracleを使う |
| `PLATFORM_ADAPTER` | 意味は共有するが、OS／browser／device eventの取得だけが異なる | adapterは正規入力またはtyped resultへ変換し、製品意味を持たない |
| `HOST_CONTRACT` | UIから要求できるが、可否・commit・永続意味はRust Hostが所有する | HostはnativeでもWasmでもよい。React stateで成功を先取りせずterminal resultを投影する |
| `RERUN_STANDARD` | 公式Rerun Web Viewerと固定Rerun subsystemの標準機構で運ぶ | scene／view／camera／pickingをMotoliiで再実装しない |
| `WASM_REBUILD` | native Rerun forkのRust拡張をWebでも使うには同じforkのWasm buildが必要 | 公式npm Viewerに拡張が入っていると仮定しない |
| `PLATFORM_CAPABILITY` | browser、OS、GPU、codec、window system等の実行環境能力 | Web／desktopのどちらにも模造の製品意味を作らず、能力不足をtypedに表示する |
| `WEB_PREVIEW_ONLY` | UI検討用の一時入力・fixture | Document、Undo、作品state、製品完了の証拠にしない |
| `FORBIDDEN_DUPLICATION` | 第二Document、第二writer、第二renderer等の禁止領域 | Web／nativeのどちらにも追加しない |

## 証拠状態

反映種別は共有ownerとprojection方法、証拠状態は現在どこまで成立したかを表す。両者を一つの「対応済み」に潰さない。

| 状態 | 必要証拠 |
|---|---|
| `WEB_RENDER_PROVEN` | 共有sourceをWeb rendererが直接読み、縮約copyなしで対象surfaceを描画した |
| `WEB_INTERACTION_PROVEN` | pointer／keyboard／focusを含む対象操作がWeb上で完了し、Cancel／失敗も見える |
| `WASM_CORE_PROVEN` | 対象Rust ownerをWasm buildし、browserで同じsemantic fixtureが通った |
| `REAL_HOST_ROUNDTRIP_PROVEN` | fixtureでなく実Host snapshotとtyped intent／terminal resultが一周した |
| `ADAPTER_DEFINED` | 対象window adapterの入力、出力、lifecycle、失敗、非目標が閉じた |
| `WINDOW_ROUTE_PROVEN` | 同じ共有source／Rust coreを対象windowが読み、実環境で同じ操作が完了した |
| `SEMANTIC_PARITY_PROVEN` | 同じ入力列に対するDocument／Undo／diagnosticがWebとnativeで一致した |
| `PERFORMANCE_GATE_PROVEN` | hot path、GPU、codec、memoryのnamed budgetを対象platformで通した |

`WEB_RENDER_PROVEN`だけで`WINDOW_ROUTE_PROVEN`や製品完成へ繰り上げない。一方、Webで一周したUI意味をdesktopで再設計し直さない。

## projection adapter予算

Web／macOS／Windows adapterが所有してよい責任は次の四種に限る。

| seam | 所有してよいもの | 所有してはいけないもの |
|---|---|---|
| renderer mount | RN/Fabric component、DOM mount、window／surface lifecycle、寸法、DPI | panel構成、visual token、製品state |
| input normalization | OS／DOM／RN raw event、pointer capture、IME、reserved trigger | shortcut表、CommandId意味、gesture成功の先取り |
| capability I/O | file、clipboard、dialog、menu、permission、notification、storage byte I/O | Document schema、journal意味、asset identity |
| accelerated surface | Rerun、wgpu、codec、zero-copy texture、frame scheduling | 第二scene、第二Timeline、別Preview／Export意味 |

adapterは次の条件をすべて満たす。

1. 入出力が既存typed contractへ閉じる。
2. platform固有型をDocument、plugin、Vism公開契約へ漏らさない。
3. layout、色、label、shortcut、Undo規則を持たない。
4. 失敗をtyped terminal resultとして返し、UI成功を推測しない。
5. lifecycleとresource解放を除き、作品stateを保持しない。
6. Webとnativeで同じsemantic fixtureを再利用する。

## surface／能力辞書

| 対象 | 種別 | Web原本／共通部分 | platform側またはHost側 | 受入条件 |
|---|---|---|---|---|
| shell、panel配置、resize、tab、scroll | `SHARED_SOURCE` | React component、layout state、token | native window寸法だけadapter | 同じcomponentでWeb／nativeの配置と操作が一致 |
| Browser、Inspector、Timeline chrome | `SHARED_SOURCE` | 表示、検索、filter、階層、parameter行、drag UX | 実データsnapshotとintentはHost | fixtureを消しても同じcomponentが実snapshotを描く |
| theme、色、spacing、icon、focus表示 | `SHARED_SOURCE` | tokenとcomponent state | OS contrast／font fallbackだけadapter | platform別token copyがない |
| accessibility | `SHARED_SOURCE` + `PLATFORM_ADAPTER` | role、name、state、focus順 | native accessibility bridge | keyboardだけでも同じ操作へ到達 |
| command palette／shortcut設定UI | `SHARED_SOURCE` + `SHARED_RUST_CORE` | 検索、表示、編集、競合表示 | resolverは同じRustをnative／Wasmで実行し、保存先だけadapter | 同じ`CommandId`とversion付きkeymap JSONを読む |
| keyboard raw event | `PLATFORM_ADAPTER` | 下記の正規化辞書 | DOM／RN／OS event取得 | raw keyを製品commandへ直結しない |
| focus／IME | `PLATFORM_ADAPTER` | focus owner表示 | `ImeGateState`とcomposition event | preedit中shortcut抑止、focus loss時安全Cancel |
| pointer／drag／capture | `PLATFORM_ADAPTER` | Preview→Commit／Cancelの操作状態 | pointer capture、touch、pen、DPI | capture lossが`SafetyInterrupt`になる |
| context menu | `SHARED_SOURCE` + `PLATFORM_ADAPTER` | action集合とenabled理由 | browser／native menu surface | action意味と並びをOS別に分岐しない |
| clipboard | `HOST_CONTRACT` + `PLATFORM_ADAPTER` | Copy／Cut／Paste command入口 | OS clipboard、payload検証、再写像 | clipboard失敗を成功表示しない |
| file picker、folder登録、外部drop | `HOST_CONTRACT` + `PLATFORM_ADAPTER` | entry UI、進捗、拒否表示 | permission、bookmark、path、stream | browser Fileを永続pathとみなさない |
| menu bar、global shortcut、multi-window | `PLATFORM_CAPABILITY` + `PLATFORM_ADAPTER` | command／panel identity | browser／AppKit／Windows window system | 環境固有入口も同じ`CommandId`へ戻る |
| Timeline描画・直接操作 | `SHARED_SOURCE` | 実`WireTimelineProjection`から生成する共有scene/model、視覚、typed hit／gesture | snapshot、commit、accelerated surface | 固定`ROWS`なしで同じ行／key／playhead契約を読み、実`set_time`／edit terminalとDocument parityが通る |
| Stage標準spatial表示 | `RERUN_STANDARD` | Rerun Web Viewer mount | Rust側Rerun projection／gRPC／RRD | Rerun内部機構の再実装ゼロ |
| Rerun custom visualizer／Vism filter | `WASM_REBUILD` または `HOST_CONTRACT` | 標準componentへlowerできる結果だけ共有 | 同じforkをWasm化、またはRustで評価して標準Rerun入力へstream | 公式npm版とcustom forkの差を明示 |
| media decode、hardware codec、zero-copy texture | `PLATFORM_CAPABILITY` | 状態・診断表示のみ共有 | WebCodecs／Wasm codec／FFmpeg／OS codec／wgpu texture | capability差をPreview／Export意味の差と称さない |
| Document、selection identity、playhead authority | `SHARED_RUST_CORE` + `HOST_CONTRACT` | revision付きsnapshotの表示 | 同じRust Host／Documentをnative libraryまたはWasmで実行 | React local stateを第二authorityにしない |
| Undo／Redoとin-memory history | `SHARED_RUST_CORE` + `HOST_CONTRACT` | command入口とterminal result | 同じwriter／inverse／history実装をnative／Wasmで実行 | 同じcommand列が同じDocumentへ戻る |
| journal／save／open | `SHARED_RUST_CORE` + `PLATFORM_ADAPTER` | codec、version、replay、atomicity判定 | native filesystemまたはWeb OPFS／IndexedDBへのbyte I/Oだけadapter | 同じjournal bytesとreplay oracle。保存先差を意味差にしない |
| Preview／Export評価 | `SHARED_RUST_CORE` + `PLATFORM_CAPABILITY` | evaluation graphとQuality意味は共有可能 | codec、GPU surface、長時間job、file出力はplatform capability | 簡易previewを別platformのExport成功と称さない |
| GPU budget、cache、ResourceLedger | `PLATFORM_CAPABILITY` + `HOST_CONTRACT` | pressure／refusalの表示 | Rust resource owner | UIがresource成功を推測しない |
| 第二Document／第二writer／別Preview renderer | `FORBIDDEN_DUPLICATION` | なし | なし | 存在しないこと |

## 現行Motolii surfaceの具体判定

| surface／成果 | Web窓へ反映する共有範囲 | platformへ残る最小接続 | 現在の証拠gap |
|---|---|---|---|
| top shell／transport | 全layout、button、shortcut表示、状態投影 | playback／export intentとterminal result | Web render済み。全shortcut roundtripは未実測 |
| Browser | Media／Effects／Create／Palettes、検索、階層、表示形式、drag UX | file permission、thumbnail／catalog source、apply intent | Web render済み。実catalogとdrop commitは未接続 |
| Inspector | parameter、animation diamond、context action、色、effect順序UX | real parameter snapshot、D2 intent | Web render済み。全parameter型の実Host roundtripは未完了 |
| Timeline | 行、clip、key、playhead、選択、drag preview、key tools | snapshot、commit、native accelerated leaf | Web Skia render済み。直接操作と実Document parityは未完了 |
| Stage chrome | Fit、zoom表示、overlay、tool affordance | Rerun surface、GPU texture、picking、D2 terminal | shared shell済み。実Rerun Web Viewer接続は辞書後に再施工 |
| Rerun標準viewer | spatial view、camera、picking、standard visualizer | Document→Rerun projection、resource admission | 公式Web Viewerは既知実装。Motolii実入力のlocal roundtrip未完了 |
| Rerun custom Vism | UI、parameter、diagnostic、標準結果表示 | Rust評価、標準component lowering、必要時fork Wasm | native forkと公式npmの差分compile未実証 |
| keymap | 設定UI、label、focus、競合表示 | raw event normalizationだけ | `motolii-input`へ共有ownerを抽出し、`motolii-web-input`のWasm build、Undo／Redo CommandId、IME抑止、SafetyCancel parityまで実証済み。DOM接続と実Host roundtripは未完了 |
| Document／Undo | UI、履歴表示、command入口 | storage byte I/O以外なし。Rust coreをWasm実行 | native Rust実装済み。対象crateのWasm compile未実証 |
| save／open | UI、進捗、recovery、diagnostic | native FSまたはOPFS／IndexedDB byte adapter | journal意味は既存。Web storage adapter未実装 |
| Preview | UI、time、Quality選択、結果表示 | GPU／codec capability | 同じevaluationのWasm／WebGPU成立性は未実証 |
| Export | UI、queue、progress、cancel、terminal result | codec、長時間job、出力file | Web／macOS／Windowsごとのcapability profileとproduct gateが未実証。あるwindowの成功を別windowの証拠にしない |

## 第一共有基盤: input／Undo

### 現行code fact

- `ui/motolii-rn/src/host.ts`はmacOS key codeの一部をTypeScriptへ複製し、DOMのphysical `code`、Release、IME composition、editable focus、reserved shortcutを一つの製品入力へ正規化していない。
- `ui/motolii-rn/native-renderer/src/host_bridge.rs`は入力ごとにkeymapとuser deltaを解決し、toolkit非依存の`InputRouter`を通さないため、IME抑止と`SafetyInterrupt`がWeb／desktop共通routeになっていない。
- 既存の`state_ownership`、`domain_intent`、`command_registry`、`keymap`、`keymap_codec`、`input_router`はtoolkit非依存であり、第二実装を作らずplatform-neutral ownerへ抽出できる。
- `motolii-doc::DocumentWriter`はsnapshot、revision、Undo／Redoを既に所有する。React／TypeScriptへDocumentやinverse commandを写さない。

### 閉じた第一契約

| field | 内容 |
|---|---|
| `OUTCOME` | Web窓とdesktop窓の`Primary+Z`／`Primary+Shift+Z`が同じRust resolverを通り、同じ`DocumentWriter`のUndo／Redo結果を返す |
| `SEMANTIC_OWNER` | 既存Rust input modulesと`motolii-doc::DocumentWriter` |
| `WEB_EXECUTION` | 正規化済みkey／modifier／phase／IME／platformをWasm境界へ渡し、Rust側で`CommandId`を解決する |
| `PLATFORM_CAPABILITY` | DOM／RN／OS raw event取得とcomposition／focus／capture lifecycleだけ |
| `ADAPTER_INPUT` | physical key、modifier、phase、repeat、composition、editable focus、platform command modifier |
| `ADAPTER_OUTPUT` | Rust resolverが返す`CommandId`、抑止理由、typed terminal result |
| `LIFECYCLE` | key press／release、composition start／end、focus loss、capture loss |
| `FAILURE` | unknown key、reserved trigger、IME-owned、unknown command、undo unavailable |
| `STATE_HELD` | input routerのgesture／IME lifecycleだけ。作品stateは`NONE` |
| `PARITY_ORACLE` | 同じ入力列ごとのDocument serialized bytes、revision、`can_undo`、`can_redo`、diagnostic reason |
| `PERFORMANCE_GATE` | `NONE`。入力ごとのkeymap／user preference再読を解消した後にhot-path budgetを計測する |
| `RETIREMENT` | TypeScript／native bridge内の部分shortcut表とCommandId直結 |

抽出先は入力意味だけを所有する`motolii-input`とする。`motolii-ui`全体をWasm化せず、同じinput crateを互換re-exportしてdesktop窓も読む。Web窓の入力bindingを置く場合もresolver／routerだけに限定する。

Web窓の製品Hostは新設しない。既存v1 `dispatchIntent`／`readSnapshot` wireを実装するadapterとして、既存Host engineの再利用または既存Hostへのtyped transportへ接続する。`DocumentWriter`だけを直接包む`create`／`undo`／`redo` APIや汎用`Command` JSONは、既存Hostのadmission、projection generation、diagnostic、journal write blockを迂回する第二Hostになるため禁止する。

最初のinput Wasm依存treeへ`motolii-ui`、`eframe`、`egui-winit`、`skia-safe`、`winit`、`wry`、`motolii-audio`、`motolii-export`、`motolii-transport`、`cpal`が入った場合は失格とする。

2026-08-14時点で第一契約は`WASM_CORE_PROVEN`まで成立した。`motolii-input`をdesktop側`motolii-ui`が互換re-exportし、input-only `motolii-web-input`は`wasm32-unknown-unknown` release buildを通過した。Wasm依存treeに上記禁止依存はなく、native／Web相当の同一入力列でUndo／Redo CommandId、IME抑止、pointer capture lossのSafetyCancelが一致した。これは`REAL_HOST_ROUNDTRIP_PROVEN`ではない。

### 後続Host接続gate

- Web adapterも既存v1 `dispatchIntent`／`readSnapshot` wire、projection generation、typed diagnostic、write-block reasonを維持する。
- Browser／Inspectorは実catalog、selection、parameter、effect順序snapshotとPreview→Commit／Cancel→Undoを同じcomponentで一周する。
- Timelineは固定`ROWS`とlocal playhead-only previewを退役し、実projection、terminal result、Document bytes／revision parityを通す。
- Stageはcamera／pickingをRerun、transform gesture lifecycleを共有入力core、Preview／Commit／CancelをD2 Hostへ閉じ、capture lossを`SafetyInterrupt`へ送る。
- save／openはjournal codec／recovery policyとplatform byte I/Oの分離をcompileで示すまで、Web用`ProjectSession`を再実装しない。

## 「Web窓へ反映できる」の判定手順

各成果は次の順で判定する。

1. **意味owner**: React表示、Rust core、Rerun、platform capabilityのどこが既存ownerか。
2. **Web実行**: copyを作らず、同じsource／Rust Wasm／Rerun Viewerで実行できるか。
3. **real roundtrip**: fixtureでなく実snapshot、intent、terminal result、UndoまでWeb窓で一周するか。
4. **adapter残余**: 各windowで共有できなかった責任が上記projection adapter四種だけか。
5. **parity oracle**: 同じ入力列、Document bytes、diagnostic、visible resultを比較できるか。
6. **retirement**: Web原本と重複するnative presentationを退役できるか。

4で製品意味が残る場合はadapterを広げず、Web routeまたは共有Rust coreの不足へ戻す。

## 施工時のseam manifest

共有coreをWeb窓またはnative窓へ接続する変更は、実装前に次を一行ずつ埋める。空欄を推測で実装しない。

| field | 記載内容 |
|---|---|
| `OUTCOME` | 利用者がWeb／nativeの両方で完了する一つの成果 |
| `SEMANTIC_OWNER` | shared React、shared Rust core、Rerunの既存owner |
| `WEB_EXECUTION` | 同じsourceで完了するWeb routeと現行証拠状態 |
| `PLATFORM_CAPABILITY` | 対象windowに不足する具体的browser／OS／GPU capability。一つ以上列挙するだけでは広すぎる |
| `ADAPTER_INPUT` | raw platform eventまたはshared coreから来るtyped request |
| `ADAPTER_OUTPUT` | normalized input、bytes、texture、typed terminal resultのいずれか |
| `LIFECYCLE` | mount／open／resize／focus／cancel／close／resource release |
| `FAILURE` | permission denial、device loss、capture loss等のtyped failure |
| `STATE_HELD` | adapterが保持するlifecycle state。作品stateは`NONE`でなければならない |
| `PARITY_ORACLE` | Webとnativeへ同じ入力を与えた時に比較するDocument／diagnostic／visible result |
| `PERFORMANCE_GATE` | native capabilityを選ぶ理由になる実測budget。不要なら`NONE` |
| `RETIREMENT` | 接続後に削除する重複presentation、fixture、旧adapter |

### adapter失格条件

次のどれかがあれば、薄いprojection接続ではなく第二実装なので接続を止める。

- adapterが`CommandId`を自作・改名・別keyへfallbackする。
- adapterがselection、playhead、parameter、effect順、Undo可否を所有する。
- platform component内に共有原本と別のlabel、色、layout、hit areaがある。
- 一つのwindowだけで成功し、shared componentへtyped terminal resultを返さない。
- Webとnativeで異なるfixture／oracleを使い、意味差をplatform差として隠す。
- 性能の実測なしにWeb routeを捨て、native専用presentationへ置換する。
- platform高速化のためDocument／plugin／Vism契約へOS型やGPU handleを露出する。

### adapterの大きさを測る観点

行数ではなく責任で測る。許される責任はraw event変換、capability呼出し、lifecycle、resource解放、typed failureの五つだけ。
六つ目の責任が必要なら、既存adapterへ足さず独立した意味ownerまたは共有coreのgapとして再分類する。

## Web窓とnative窓でsubstrateを共有するかの別判定

Webで可能でも、desktopで同じ実装を採用するかは性能・権限・配布で別に決める。

| 能力 | Webでの成立候補 | desktopで交換し得る理由 | 交換後も共有するもの |
|---|---|---|---|
| storage | OPFS／IndexedDB | project file、atomic replace、外部volume | journal bytes、version、replay意味 |
| clipboard | Clipboard API | rich payload、permission、menu連携 | Copy／Paste CommandIdとpayload contract |
| file input | File System Access API／drop | bookmark、registered folder、監視 | asset admissionとdiagnostic |
| video | WebCodecs／Wasm codec | format coverage、hardware path、export | timeline time、Quality、failure contract |
| GPU | WebGPU | surface共有、zero-copy、resource budget | canonical scene input、color、Preview意味 |
| Rerun | Web Viewer Wasm | custom fork、native wgpu integration | entity／time projection、selection mapping |

この交換可能性こそがWeb原本の価値であり、Web版を捨てる理由ではない。

## shortcut正規化辞書

shortcutは `physical event -> Gesture -> EffectiveTrigger -> CommandId -> DomainIntent／HostKind`
の一方向へ流す。`Gesture`以後は同じRust resolverをnative／Wasmで実行し、Web専用shortcut tableを作らない。

### key

| Web／RN event | `KeyToken` |
|---|---|
| `KeyA`〜`KeyZ` | lowercase `AsciiKey('a'..'z')` |
| `Digit0`〜`Digit9` | `AsciiKey('0'..'9')` |
| `Space` | `Space` |
| `Enter` | `Enter` |
| `Escape` | `Escape` |
| `Delete` | `Delete` |
| `Backspace` | `Backspace` |
| `Tab` | `Tab` |
| `ArrowUp/Down/Left/Right` | 対応するArrow token |
| `Home/End/PageUp/PageDown` | 対応するnavigation token |
| locale文字、未定義key | shortcutへ変換せずtext input／unavailable diagnostic |

### modifierとphase

| event | 正規値 |
|---|---|
| `metaKey` | `Modifier::Meta` |
| `ctrlKey` | `Modifier::Control` |
| `altKey` | `Modifier::Alt` |
| `shiftKey` | `Modifier::Shift` |
| key down | `InputPhase::Press` |
| key up | `InputPhase::Release` |
| macOSの`Primary` | `Meta`へ展開 |
| Windows／Linuxの`Primary` | `Control`へ展開 |
| pointer down／up／click | `Press`／`Release`／`Click` |
| drag | `DragStart`／`DragUpdate`／`DragEnd` |
| `Escape`、capture loss、window focus loss | `Cancel`または`SafetyInterrupt` |

`event.key`の表示文字ではなく、既定shortcutは位置が安定する`event.code`をkey tokenへ写す。
IME preedit中は既存`ImeGateState::PreeditActive`へ渡し、shortcutを発火しない。
browser／OS予約組合せは別keyへ黙って置換せず、既存`UnavailableOnPlatform` diagnosticとして表示する。

## 現行builtinの共有対象

現行`product_builtin_keymap()`と`host_kind_rows()`を唯一の既定表とする。少なくとも次をWebでも同じresolverから読む。

- Primary+Z: Undo
- Primary+Shift+Z: Redo
- Escape: in-flight gestureのCancel
- Delete／Backspace: targeted delete
- Space: playback toggle
- J／K／L: shuttle reverse／stop／forward
- I／O: trim in／out
- Primary+D／K／C／X／V／A: duplicate／split／copy／cut／paste／select all
- navigation、solo、mute等の既存HostKind

表示ラベルはmacOSで`Cmd`、Windows／Linuxで`Ctrl`へ変えてよいが、保存契約は`Primary`のままにする。

## Rerun境界

公式`@rerun-io/web-viewer`は標準Rerun ViewerのWeb実行であり、StageのUI検討と標準component表示に使える。
一方、native-rendererが固定forkへ加えたcustom visualizer、filter、renderer変更はnpm packageへ自動では入らない。

現行`ui/motolii-rn/native-renderer/Cargo.toml`の実固定は`oshikaidesu/rerun`の`8c6865acb9770d7f0ea50319fc985d9c7ceac055`、lock上のRerun crate版は`0.35.0-alpha.1+dev`である。
旧資料や画面labelの`954bf95`を現行binary fingerprintとして使わない。Web Viewer、RRD／gRPC producer、native Viewerの互換判定は実manifest／lockから得る同じversionとcommitで行う。

処分は二つだけとする。

1. Vism／Rust側で評価し、Rerun標準archetype／componentへlowerしてWeb ViewerへRRDまたはgRPCでstreamする。
2. 標準入力へlowerできないRerun拡張だけ、固定forkのViewerを同じfeature setでWasm buildする。

JavaScriptで同じfilterを再実装する第三routeは作らない。

## 禁止

- Webとnativeへ別々のshortcut表、panel tree、parameter stateを置く
- Rust Document／Undo／keymap resolverをTypeScriptへ再実装する
- browser予約shortcutを別操作へ黙ってfallbackする
- fixtureの成功をDocument commit、Undo、Preview／Export成功として表示する
- 公式Rerun npm版にMotolii forkの拡張が含まれると扱う
- native能力がないWeb previewのために第二renderer、第二cache、第二writerを作る
