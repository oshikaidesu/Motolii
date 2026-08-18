# 体験の段差台帳(friction ledger)

- 制定: 2026-08-12(利用者裁定「まだある細かい部分の体験の段差を全て撤廃する」)
- 位置づけ: [品質バー](ui-quality-bar.md)違反の**未修正在庫**。空であることが定常状態 — 新しい段差は発見次第ここへ入れ、orderで焼却する。18本の独立review・3ハンター・gesture嵐で指摘済みだが未修正のP2級を初期在庫とする

## 在庫(2026-08-12制定時点)

### 掃討wave A(interaction) — **焼却済み 2026-08-13**

F1(playhead直接掴み)/F2(カーソル言語: trim=resizeLR・clip=open/closedHand・key=pointingHand・drag中はhit外でも維持・mouseUp後再計算)/F3(Undo/Redo文脈disabled、wire `history`)/F4(ruler目盛の絶対位相)/F5(exact-on-key行のgesture実信号凍結+凍結identity commit)/F6(空Timeline一行ガイド)/F7(key hit半径5.6px視覚一致+境界test)/F8(`(+N)` 実件数、`truncated_total` saturating集計) — order 19+fix19で全焼却。PNG sha `43ec101c` 不変。

### 掃討wave B(performance) — **焼却済み 2026-08-13**

F9(毎tick最大128KiB JSONパース → `motolii_rn_host_projection_stamp` 軽量stamp FFIで変化時のみfull読み。stampはtick冒頭1回読みで判定と保存に同値を使いTOCTOUなし、失敗時はstamp破棄で次tick必ず再読)/F10(registry mutexを「取り出し/書き戻し」2区間へ分割、graph構築とGPU submitはlock外)/F11(mount時warm-up先払い、telemetry `warmup_us=… ok=…`) — order 20+fix20で全焼却。実機実測(30秒run全体max、初回可視フレーム込み): 初回スパイクStage 489ms/Timeline 175ms → **Stage 20.7ms/Timeline 1.3ms**(50ms超ゼロ=B6充足)、warm-up自体はStage 29.6ms/Timeline 11.7ms(ok=1)。

### 掃討wave C(vism source write) — **焼却済み 2026-08-13**

F12(source paramが表示のみ) — `SetProperty` + `ScalarPropertyId::SourceParam` が既存 `ClipSource::Plugin.params` キーへ任意 `DocParam` を書く。RN Inspector の第一consumerは f64 Const。Color/Vec は同じ command。vism の型天井は置かない。

### 台帳残(grain/campaign従属 — 対応先が決まっているもの)

| # | 段差 | 行き先 |
|---|---|---|
| F13 | 17件目以降のlayer/65個目のkey/9個目のeffectがUIから触れない(capの沈黙は(+)で緩和済みだが操作不能は残る) | cap設計grain |
| F14 | DOC status labelの開発者臭(`DOC r42`) | Q0 inventory掃除(Sol) |
| F15 | keydragが隣接clip境界のkeyでtrimに先勝ちされる | 判定順の再考(小粒) |
| F16 | 複数timeline間のCAS上書き / BigInt精度(i64>2^53) | known limit(multi-window/将来) |
| F17 | Timeline drag中にEscapeを押しても、window focus/pointer captureを失っても、掴んだ状態が解けない(gesture stateが`Some`のまま残る) | `TimelinePointerPhase`にCancel位相を通す |
| F18 | `enqueue_timeline_intent` の失敗が全call siteで `let _ =` により捨てられている(Document到達失敗が無音) | enqueue失敗の可視化 |

### F17/F18の観測(2026-08-15、`timeline_egui.rs` 851行時点)

Document到達経路そのものは存在し、型付きの門を通っている。`app.rs:607 handle_timeline_pointer` → `TimelineMoveGesture` / `TimelineTrimGesture` / `EguiKeyDrag` → `timeline_intent_adapter::TimelineIntent::{MoveClip, TrimClip, MovePositionKey}` → `enqueue_timeline_intent` → `DocumentEditQueue`。**「clip trim / key moveが門を通っていない」という初出の記述は誤りだったので上表を訂正した。**

実際の穴は2つ。

- **F17(Cancel位相の不在)**: `timeline_egui::TimelinePointerPhase` は `Down` / `Drag` / `Up` の3値で、Cancelが無い。`motolii-input` 側には `InputPhase::Cancel`、`SafetyInterrupt::{PointerCaptureLost, WindowFocusLost}`、`DomainIntent::CancelInFlightGesture` が揃っているが、egui Timelineはこれを一切発火しない。`TimelineCommand::Escape` は `timeline_egui` で収集されるものの、`app.rs:486` のCommand matchが拾うのは Undo / Redo / Duplicate だけで `_ => None` に落ちる。結果、drag中のEscapeもfocus喪失も `self.timeline_move` / `timeline_trim` / `timeline_key_drag` を解除しない。
- **F18(enqueue失敗の無音)**: `enqueue_timeline_intent` は `Result<(), TimelineIntentAdapterError>` を返すが、`app.rs` の全call site(502、627、705、718、732)が `let _ =` で捨てている。Documentに届かなかった事実がどこにも出ない。これが「気づきづらい」の実体。

**入力層の重複**: `timeline_egui.rs` を `InputPhase` / `InputRouter` でgrepすると0件で、dragは生eguiで処理されている(`interact_pointer_pos()` 432、`!pressed && response.dragged()` 489、`egui::Event::PointerMoved` 511)。489行は `InputPhase` が既に持つpress/drag弁別の手書き再実装。`TimelinePointerPhase` は `InputPhase` の部分再定義で、落ちたのがCancel — F17はこの重複の帰結。

**構造的原因**: 境界が強制されていない(生eguiの`response.dragged()`が常に手の届く所にある)ため、局所で完結する実装が最も安い経路になる。issue単位のdispatchはこれを増幅する — 受注側にとって、別crateの位相enumに合流させるより当該ファイル内で3値を定義する方が合理的なため。落ちるのは「その操作に無くても動くもの」= Cancelとエラー伝播で、どちらも正常系では観測されない。

> **パス訂正(2026-08-19)**: 上のF17/F18観測が指す`timeline_egui.rs`は2026-08-16に削除され、現在の実装は`crates/motolii-ui/src/timeline_editor/`(9,059行)。観測内容(Cancel位相の不在・enqueue失敗の無音)自体がこの新しい実装でも再発していないかは未確認 — F17/F18は上表の「台帳残」から未消化のまま。

## 規則

- 段差の追加は誰でも(発見者が)行う。**削除はorderの着地のみ**
- 「仕様どおり」は撤廃を免除しない — 体験として段差なら在庫に入る
