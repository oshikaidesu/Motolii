# 関数群の契約 — 合成とホットリロードを通常の開発単位にする

- 状態: **決定／実装・検収完了**(2026-09-05)。名前のある関数群と代表経路を実装し、意味・実GPUの試験、後から開いた別窓の連続更新、source復元とowner保持を確認した。既存の型・保持入口の契約内で目標を達成した。
- 優先条件: 状態とGPU資源を保った反復。形式化した関数群を差し替え単位とし、内部を原子と合成へ分解できること。
- 適用先: [PR #479 採用計画](2026-09-04-pr-479-adoption.md)。利用者の達成まで継続する指示に従い施工を再開済み。実装・試験・実窓を別々に記録する。
- 開発ループの最終目標とCLI内部に基づく補正: [ホットリロード目標](2026-09-04-dioxus-reload-goal.md)。workspace対応はライブラリ説明とCLI実装を区別する。
- グループの入力・出力・依存・副作用・配置の正本: [function-contracts.json](../../motolii/reference/function-contracts.json)。本書は意味・法則・採用根拠・検収を持つ。
- doc/render/ui/vism/tests の5家、Documentの編集状態、Intent経由の書き込み、上流再利用、固定dxの運転を継ぐ。[現行憲法](../../motolii/AGENTS.md)

## 1. 確定する構造

数学の原子を既存の `std` / `kurbo` / `glam` / doc評価へ借り、意味を守るブロックをその上に組む。ブロックを名前のあるRust moduleとして切り、内部の関数を合成する。Dioxusは表示・入力の接続、実行側は状態保持と副作用を担う。

```text
原子・既存の評価 ──→ Lens / Read / Control / Verb
                           ↓
                Gesture + Compose → 編集計画
                           ↓
            Shellが Preview / Commit / Cancel を実行

Read → Paint / View → 表示
Table → 型付きの操作・部品を選択 → View / Compose
```

- `atom` は数値・区間・写像・近傍など。`copy_body`、素材のトリム制約、ID採番は編集の意味なので `lens/verb` に置く。
- `lens/read/verb/control/gesture/compose` は、入力に現れない時計・選択・乱数・I/O・globalを読まない。戻り値は値・計画・理由であり、Documentを書かない。
- `paint` の `&mut Scene` は明示した出力バッファへの書き込みであり、厳密な純関数とは呼ばない。描画の準備とGPU資源の作成・登録・submitを分離する。
- `table` は型付き関数・descriptor・bindingの供給。`view` はRSXと入力通知。作品の意味や永続値を所有しない。
- `shell` は役割であり、全機能を一つの巨大関数へ集めない。Host、Session、widget adapter、render側の資源ownerが各々の副作用を実行する。
- `compose` は関数の合成を指す。作品設定の既存 `ui/composition.rs` と区別する。
- グループ数は完全性の定理ではない。分割理由は変更を局所化する責任と接続契約。初期moduleと依存方向をmanifestに固定し、変更は同じ契約を更新する。

## 2. ブロックの接続契約

共通portは既存の `LayerId`、`PropertyId`、`RationalTime`、`Value`、`Intent` 等と、その型付きの組で表す。共通型の数に上限を設けず、単位や対象の違いを一つの文字列や無型の値へ押し込めない。

| 接続 | 必須条件 |
|---|---|
| 数学原子 | 有効域、単位、座標系、端点の開閉、丸め、overflow/NaN/ゼロ除算の扱いを宣言。法則はその有効域で成立させる |
| 読み取り | baselineのrevisionと時刻を入力する。表示のtransientと永続sourceのどちらを読むかを宣言する |
| 属性の編集参照(lens) | addressだけでなく、値の型・既定値の持ち主・source・編集規則・失敗条件を持つ。`PropertyId`だけを数学的Lensとは数えない |
| 単一対象の意味関数 | 同じbaselineとpayloadから、同じ順序付きIntentブロック／no-op／理由付き拒否を返す |
| 選択全体の意味関数 | Group、複製、並び替え、共有source等を全体で計画。ID写像・対象閉包・依存順序を一箇所で決める |
| 合成 | portの型と前提条件が接続できること。相互依存する編集は意味を合成してからIntent化する |
| 描画準備 | 同じ入力から同じ出力命令。途中失敗・hotpatch再試行で半端な描画を再利用／二重追加しない |
| 外界への要求 | 保存・読込・音声・GPU等は実行要求と結果の入力に分ける。外界そのものを純関数だとは扱わない |

状態を持つ対話も `step(固定baseline, 対話state, 正規化input) -> (次state, request)` として純関数化する。stateの**保持**はShell、stateの**型定義**は既存doc型またはUIの型宣言専用contractに置く。純粋な群からShell型をimportする循環を作らない。

`&Document` / `&StoreView` のread-only利用は許す。既存 `Document::place` のようなIntent生成をUIに複製しない。キャッシュ以外の観測可能な副作用や、内部に隠れた可変入力がある呼び出しは純粋な群から分離する。

**値とsourceは別の意味である。** absent/default、Constant、Track、Slot、Link、modulatorを区別する。数値が等しいだけでno-opにはしない。既存キーの補間、駆動や共有の維持／切断、キーを追加する時刻は属性のownerが定める。未定のsourceへはそのcontrolを接続せず、暗黙に `SetConstant` へ変換しない。

## 3. 合成の規則

1. 通常の関数呼び出し、標準iterator、`Result`、既存のデータ型で合成する。独自VM、グラフ実行機、文字列dispatch、plugin ABIを前提にしない。
2. `map/lift` で独立した対象を扱い、`fold` で前の結果を必要とする変更を扱う。区別を操作の契約に宣言する。
3. 書き込み先の単位は実際の永続単位。同じVec2のX/Y、同じtrackの異なるkey、同じeffects列への追加は重なる。独立に作った `SetTrack` / `SetEffects` を連結して後勝ちにしない。
4. 同じ完全値を逐次上書きすること自体が仕様の箇所だけ、順序を明示する。汎用の「最後の値だけ残す」をIntent全体へ適用しない。
5. 部分拒否は独立操作の準備段階で決め、理由と対象を返す。依存関係のあるGroup/複製等は同じ拒否単位にする。予期しないStoreErrorを任意の局所拒否へ読み替えない。
6. IDや参照を新設する計画は選択全体で一意性を確定。同じsnapshotから各層が同じnext-IDを取る方式にしない。

## 4. Preview / Commit / Cancel

操作開始時に、基準revision・時刻・対象IDと順番・元値とsource・座標写像・関数世代を固定する。修飾キーについては、開始時固定か途中変更を許すかを操作ごとに宣言する。巨大Documentの複製を要求せず、必要なread snapshotを取る。

計画は概念上、基準情報、順序付き編集、preview対象、結果identity、事前に除外した対象と理由を含む。これは必要な情報の契約であり、別の永続schemaを導入する指示ではない。

- **PreviewとCommitは同じ最後の計画を使う。** releaseで最新の選択・時計・制約を読み直して別の編集を生成しない。
- Previewは同じDocument評価へ届く必要がある。帯だけ動く等の局所表示を、Stage/Exportまでの同一性と取り違えない。即時コマンドはPreviewなしを明記できる。
- 1操作は `apply_all` 等、Documentの一つのtransaction。成功は1 Undo、no-op・取消・失敗は0 Undo。失敗時は**Redoも含めた編集履歴**を保持する。
- Commit直前に基準revisionを確認。不一致の既定は取消と理由通知。自動で最新状態へ書き込まず、rebaseを許す操作だけ明示的に再準備・再表示する。
- Cancelはその操作が所有するtransientだけを消し、無関係なpreviewを消さない。同じpropertyへの同時previewはownerが調停し、片方の取消が他方を消す状態を作らない。
- 同一性は固定時刻・同じ表示条件で評価する。関数の同じ型、同じPID、同じ見た目一枚だけを根拠にしない。

## 5. ホットリロード契約

**頻繁に変える関数と組み合わせは、実際に差し替わるコンパイル経路へ置く。** binary tipと、CLIが捕捉したworkspace libraryの再compile/link経路を区別して検収する。doc/renderの既存評価・永続規則は所属を維持する。「下位crateだから対象外」という当初の規則は、同版CLI内部のworkspace replayを確認したため撤回する。詳細は[開発目標の内部調査](2026-09-04-dioxus-reload-goal.md)。

各関数群はmanifestへ、実際のcompilation unit、差し替えの入口、保持するstateのowner、到達を観測した証拠を記録する。実装値と代表証拠を記録し、全関数の個別検証へ拡張解釈しない。

| 変更 | 決定する扱い |
|---|---|
| RSX / CSS / asset | Dioxusの対応するreloadへ。入力ownerやgeometryを破棄する構造変更は通常のdispose/cancelへ |
| 検収したtip/workspace内の関数本体・control/commandの組み合わせ | 既存 `call` / `HotFn` を通して差し替える。古いvtable内の直接呼び出しをhotな入口とは数えない |
| 保持中の型layout・enum/closure capture・portの非互換変更 | 関数本体だけのpatchとして通さない。明示した再生成／移行、必要なら範囲を限定したrebuild。無条件の状態保持は約束しない |
| 下位crateの関数本体 | CLIのcaptured rustc/rlib依存閉包が対応する範囲はthin patch候補。対応しない普通の関数変更はhot経路の未閉鎖点であり、恒常的なbuild例外にはしない |

Rust patchの受理時は、安定したHost入口で対象build/PIDを確認し、**進行中の未確定操作を取消してから**適用する。永続Document・Undo/Redo・選択・再生状態・Device/Queue・無関係な資源を保つ。取消は暗黙commitにしない。新しいstepへ古い途中stateを渡す互換契約は初版では採らない。

一つのprocess-global patchの適用と、全窓のtemplate/dirty更新を分ける。nativeのtrait-object境界ではcallee自体を型付き `HotFn::current(...).call(args)` へ渡す。raw関数pointerを返すclosureだけをhotにしても、そのcalleeの更新は保証されない。関数世代はHostの受理順を使い、失敗・stale・unwind後に副作用を無条件再実行しない。

DocumentとGPUの長寿命ownerを、差し替えで再生成されるDioxus componentの寿命に巻き込まない。型が変わらない場合も、実際のowner identityを観測する。接続が未成立ならHotFn/Blitzの既存入口を調整し、独自loaderへ進まない。

workspace内のstaticを唯一の状態ownerにしない。catalogのArcはHostが保持し、各root VDOM contextから新しいapp/detachedが最初のconsumerより前にbindする。受理済みRust変更ではDioxus root ErrorContextをclearして表示をretryできるが、一般のエラーを成功に変換しない。Velloの変更済みoverride textureはatlasへ再通知し、Stageは入力が同じpaintで既存frameを再利用する。renderer-localな登録と実GPU資源のidentityを分けて検収する。

## 6. 一次資料と採用範囲

出典・版・該当箇所はmanifestの `sources` に固定する。以下は転用の判断であり、原資料がMotoliiの設計を証明したという意味ではない。

- **[Parnas (1972)、pp.1054–1058](https://wstomv.win.tue.nl/edu/2ip30/references/criteria_for_modularization.pdf)**: 変更される設計判断をmodule内へ隠す分割を採用。処理順や関数の短さだけでmoduleを決めない。原子数・開発速度の倍率は導かない。
- **[Hughes (1989)、§2–4・6](https://www.classes.cs.uchicago.edu/archive/2010/spring/22300-1/papers/whyfp.pdf)**: 合成手段によって独立した部分を再利用する考えを採用。Rustの関数・iterator・型付き入力へ写す。参照したPDFは著者の修整本文。遅延評価やHaskellの実装を一括導入しない。
- **[Fosterほか (2007)、Definitions 3.1–3.3・Composition](https://www.cis.upenn.edu/~bcpierce/papers/lenses-toplas-final.pdf)**: LensのGetPut `put(get(c),c)=c` とPutGet `get(put(a,c))=a` は定義域と情報保持を伴う。PutPutは同論文では必須でない。Intent生成をLensと呼ぶ場合も、適用後の状態の同値関係まで定義して法則を調べる。丸めた表示値の再書込やSlotからConstantへの変換は反例になり得る。
- **[Elm公式Architecture](https://guide.elm-lang.org/architecture/)・[HTTPのupdate](https://guide.elm-lang.org/effects/http)**: Model/View/Update、`Msg -> Model -> (Model, Cmd Msg)` と結果messageを借りる。**[elm/core 1.0.5 Cmd.batch](https://github.com/elm/core/blob/1.0.5/src/Platform/Cmd.elm#L52-L62)は結果の順序を保証しない**ため、Documentの順序付きtransactionやUndoの根拠にはしない。
- **[Subsecond 0.7.10対応commit](https://github.com/DioxusLabs/dioxus/blob/57d6794ad60b949e5bd8aa282f6f8c3dc97a365e/packages/subsecond/subsecond/src/lib.rs)**、[Dioxusの適用実装](https://github.com/DioxusLabs/dioxus/blob/57d6794ad60b949e5bd8aa282f6f8c3dc97a365e/packages/devtools/src/lib.rs#L19-L56)、[Blitzのwidget入力実装](https://github.com/DioxusLabs/blitz/blob/64eb27853aa2672486b7edf825fb044be78c9db3/packages/blitz-dom/src/events/mod.rs#L145-L159): local registryの `.cargo_vcs_info.json` と固定checkoutの本文を読み、公開された同じcommitも確認した。tip、layout、vtable、適用先の制限を契約へ反映する。

独立した3つの読取レビューで、hotpatch到達、編集計画の合成、一次資料の帰属を照合した。指摘された「関数抽出だけでhot」「Cmd.batchでatomic」「任意Intentの後勝ち」「数学原子だけで複製」を採らず、本書の条件へ修正した。これは設計レビューであり、動作検証ではない。

## 7. 実装された境界と検収範囲

| 境界 | 実装と確認 | 証拠の範囲 |
|---|---|---|
| binary tipとworkspace | `main.rs`がUIを直接compile。road rlib取得を修正し、UI/doc/render本体とCSSの混在Thinを再生中のnativeへ反映 | 既存ABIの代表変更。全関数の個別hotpatchではない |
| Widgetと保持状態 | Host MountStore、StageState/TimelineState、typed HotFn、mount-local bindings/ResourceIdを接続。Engine/Textureを保持 | 有効なlayoutとrendererごとの接続。非互換型の自動移行は含まない |
| transactionとsource | `place_checked`、preview owner、StoreViewのTiming/Track/Text投影、失敗時Redo保持。Doc7と関連UI50が合格 | 数値source、選択合成、checked placement等の代表操作 |
| 関数群と合成 | 10群moduleと既存Shell、placement、共通contract/property_editを実callerへ接続。未使用の間隔配置課題が合格し、新moduleをlive追加 | 原子数の完全性や全既存関数の再配置は主張しない |
| catalogと描画 | 保持Arcを最初のconsumer前にbind。新Gain data追加・編集・Undo、不正blurのlast-goodと復元、8bit/float連鎖をnative確認 | 実GPU5件は別途exit0。Stage cached paintとVello atlas再通知は実窓で確認 |
| CLIとnative callback | TLS初期化、混在通知、callback寿命を修正。Rust受理後のErrorContext retryで同PID復帰 | 原因記録付きbaselineと、後続通常変更を区別する |

共通ファイルの所有者を一人に固定した後に、衝突しない群へ分担する。型定義、登録入口、意味の規則を各laneで重複させない。

## 8. 設計を反証する検収

1. 原子: `about(p,id)`、有効域内での区間split/joinやshift逆変換、componentのread/write等を、借りた定義の法則で測る。浮動小数点の誤差・境界条件も借りた表現から定める。
2. 代表操作: 数値入力/スクラブ、Stage変形、Timeline Move/Trim、Split/Duplicateを同じ契約へ通す。複数選択・取消・1 Undoの実装を入口ごとに増やさない。
3. 未使用課題: **選択した帯を操作前のTimeline順で一定間隔に配置する**計画を試験用に組む。先頭位置を固定し、source-inを保持し、各層のkeyを同じ差分で追随させる。対象間に制約があるため、不可対象があれば全体を準備段階で拒否する。この課題は製品コマンドの追加指示ではない。
4. 合成の判定: 未使用課題の追加が、選択全体の意味関数と既存Moveブロックの組み合わせで閉じる。Shell/Control/Gestureへ操作名の特例が要れば、契約を再設計する。
5. hotpatch: 群の関数本体・descriptorを変更し、実際の挙動が変わる。main/別窓、Undo/Redo・選択・再生・GPU資源の保持、drag中patchの取消、focus lossを観測する。時間は[build-loop budget](../../motolii/reference/build-loop-budget.tsv)の改善目標として測り、最新の利用者指示に従って必要なbuildを強制中断しない。

現在の判定は **実装済み／代表試験とnative経路は合格／最終2窓確認待ち**。Doc7・UI50・render GPU5はexit0。最終runのCLI42238/app43349はDocumentとGPUを保持し、epoch1で再生中の混在変更、epochs15/16でcanonical catalogと不正shaderのlast-goodを確認した。新Gainと異format連鎖も実窓へ届いた。最後のdoc label/Gain既定値のsource復元後の両窓確認はrootが確定する。[採用計画](2026-09-04-pr-479-adoption.md)とmanifestに範囲を残し、全関数個別検証や常時5秒以内の保証にはしない。

設計確定時はJSON構文・群ID・依存の非循環・契約欄と局所リンクを確認した。その時点の全体 `check-docs.sh` は範囲外のreview未登録とunbound variableで未完走であり、リポジトリ全体の文書合格へは繰り上げない。
