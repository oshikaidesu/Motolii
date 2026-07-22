# Historical-only D2 / selection / Timeline lineageの価値回収（Unit 2C、2026-07-23）

状態: **決定**（歴史文書8 blobの処分、4契約を採択済み・未実装follow-upとして再採択）

対象: Rectangle Place、Position Add Key、Place product core、single-primary selection、headless Timelineのhistorical-only 5 path。

関連: [全歴史coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)、[M2仕様](../specs/M2-document-model.md)、[M3仕様](../specs/M3-ui-integration.md)、[historical React / WebView回収](2026-07-23-historical-react-webview-lineage-recovery.md)

## 1. 結論

5 path / 8 blobを、初版全文と版間diffで処分した。4契約はいずれもdocs-only決定まで到達していたが、現在のlineageとコードから消えている。

| 契約 | 現行コード事実 | 処分 |
|---|---|---|
| U2b-2 Place core | `DocumentEditRuntime`は`DeleteTargetedItems`だけをqueueし、Place private入口・dedupe・receiptが無い | **D0を再採択 / 未実装** |
| U4b-0 Position Add Key | `Command`にAdd Key variantがなく、stable-ID reservationはEffect lifecycle 6 variantのまま。`allocate_keyframe_id`は帯域外でlive counterを進める | **durable Route Dを再採択 / 未実装** |
| U2h-1 primary selection | publish型は`PublishedDocument { revision, snapshot }`だけで、Host selection、projection generation、reconcileが無い | **primary-only契約を再採択 / 未実装** |
| U3a-1 headless Timeline | `motolii-timeline` crateもDocument→bar/key layout/hit-testも無い。spikeだけが固定track/f32モデルを持つ | **headless契約を再採択 / 未実装** |

これらをM2/M3が完了済みだったと書き換えない。D2基盤、U2b-1、既存100k capacity spikeの成立は保持し、その上に独立follow-upを追加する。旧VS-0の「今晩優先」例外や発注順は再発効しない。現在のSelected U series運用は維持するが、論理依存としてU3a-1 headlessをG0-9から外し、U3a-2 windowed rendererだけをG0-9待ちに戻す。

## 2. 個別処分

| 歴史path / blob | 分類 | 判定 | 現在の回収先 |
|---|---|---|---|
| `2026-07-21-m3-place-rectangle-d2-contract.md` / `1afb62c9`,`1340181d` | **現行規範へ再採択 + 負例** | Place専用private D0、accepted drop時だけID×1 / AddTrackItem×1 / macro×1、正準drop、defaults、selection/Timeline同一ID、durability分離を再採択。初版のnested Group compatibleはproduct coreでtop-levelだけへ縮小済み | 本書§3、[Rectangle D2 options](2026-07-21-m3-rectangle-drop-d2-contract-options.md)、[M3 U2b](../specs/M3-ui-integration.md) |
| `2026-07-21-m3-position-add-key-d2-contract.md` / `87b3f955`,`9152ad5e`,`96dd7c52` | **現行規範へ再採択 + 負例** | explicit Add Position Key、same-time no-op、durable reservation付きforward/inverse、curve-preserving insert、off-key暗黙編集拒否を再採択。旧Rust DTOの`Box`、solver閾値、sample数は実装形候補でありpublic APIとして即時復活させない | 本書§4、[M2 D2 follow-up](../specs/M2-document-model.md)、[M3 U4b](../specs/M3-ui-integration.md) |
| `2026-07-22-m3-u2b-2-core-product-contract.md` / `bd6bc065` | **現行規範へ再採択 + 停止線** | mandatory single composition、top-level compatible selection、first Track top fallback、private transport dedupe、appearance/durability非完了をPlace契約の製品縮小として採択 | 本書§3 |
| `2026-07-22-m3-u2h-1-single-primary-selection-contract.md` / `059c0bb2` | **現行規範へ再採択 + 停止線** | Host Transient primary-only、recursive existence oracle、atomic Document+selection envelope、revision/generation分離、Undo dangling clear / Redo非復元を再採択。additive/range/marquee/AXはU2h-2へ残す | 本書§5、[M3 U2h](../specs/M3-ui-integration.md) |
| `2026-07-22-m3-u3a-1-headless-timeline-contract.md` / `3a99cbd5` | **現行規範へ再採択 + 負例** | top-level Clip/Position keyのread-only投影、RationalTime、deterministic first-fit、typed hit-testを再採択。fixed lane/f32 spike、Group意味、renderer、色/pxは持ち込まない。G0-9はU3a-2だけ | 本書§6、[M3 U3a](../specs/M3-ui-integration.md) |

## 3. U2b-2 Place product core再採択

### 3.1 到達意味

```text
Host Transient Place drag
  → start / preview / cancel / stale / duplicate: semantic write 0
  → accepted terminal dropだけ、同じ同期call stackで
       writer snapshotのLayerIdTable clone上に候補IDを作る
       → fresh live-nextをtyped比較
       →既存Command::AddTrackItem 1件をapply_macro 1回
  → success/failureともterminal、自動retryなし
```

- 公開Place planner、公開raw ID mint、新Command variant、公開transaction lifecycleを作らない。
- live `reserve_layer_id` / `allocate_layer_id`をdrag前に呼ばない。clone上の候補とfresh writerのnextを比較し、既存table entryの黙認をfresh createと数えない。
- planとapplyの間にyield、別edit、prepared request再queueを挟まない。
- 成功は新`LayerId`をprivate receiptとして返し、同じHost turnでterminal化、U2h-1 selection reconcile、atomic publishへ渡す。
- failureはDocument、layer counter、history、revision、selection publishを呼出前と一致させる。

### 3.2 product Placeの閉じた意味

| 項目 | 再採択する値 |
|---|---|
| composition | 現行の単一mandatory `composition`。active-composition APIを追加しない |
| compatible selection | 同じcompositionのtop-level `TrackItem`だけ。Group、video Asset、Vector、LayerSource pluginはcompatible。audio-only、nested child、削除済みはincompatible |
| insertion | compatibleなら同じTrackの`selected_index + 1`。無ければ既存`tracks`列順先頭の`items.len()`。Track 0はtyped reject、Track新設なし |
| source | `ClipSource::Vector` / `VectorRecipe` / `StandardShape::Rect` |
| local center / size | `[0,0]` / `[0.2,0.2]` |
| Transform position | final dropのcanonical Y-up座標 |
| name | `Rectangle` |
| start / duration | playhead / `composition.duration - playhead`。remainingが1 frame未満またはoverflowならreject、clampしない |
| appearance | 未決。fill/stroke/colorを仮決めせず、D3/GPU Preview完成を主張しない |

これらはPlace GUI commandの意味であり、schema defaultではない。Document fieldへ`serde(default)`で焼かない。

### 3.3 transportとdurabilityの分離

dedupe keyはHost Transientの`(webview_instance_epoch, drag_ordinal, event_sequence, layout_epoch)`相当とし、一active drag、bounded terminal detail、高水位でeviction後の再適用も拒否する。exact wireはWebView Host contract側で再固定する。drag IDやepochをD2、Document、journalへ保存しない。

既存`AddTrackItem` journal互換は利用できるが、`apply_macro`成功後の`commit_edit`失敗順序は別契約である。U2b-2だけでcrash durability、appearance、四面E2Eを合格にしない。

## 4. U4b-0 durable Add Position Key再採択

### 4.1 semantic contract

- 作成入口はexplicit **Add Position Key @ playhead**だけ。global Auto Keyを暗黙追加しない。
- `Const(Vec2)`は同じ値の1 keyへ変換し、outgoingはLinear。
- animated trackのoff-key追加はplayhead評価値を1 keyとして挿入する。同一時刻に既存keyがあれば採番・編集・journal 0で既存`KeyframeId`を返す。
- on-key Position編集は既存keyのvalueだけを変更し、ID/time/interpを保持する。Auto Key無しのoff-key Position編集はtyped rejectし、Add Keyをrecoveryとして示す。
- 作成成功はfresh `KeyframeId`×1、durable command×1、revision+1、Undo+1。Undo後counterは巻き戻さず、Redoは同じIDを復元する。

### 4.2 durable Route D

帯域外`allocate_keyframe_id` + `SetProperty`は、pre-edit snapshotからjournal replayした時にcounterを進めないため製品経路から棄却する。forward/inverseのAdd Position Key専用Commandが次を自己完結payloadとして持つ形を再採択する。

- target `LayerId`。`Transform2D.position`専用で、汎用property pathやEffectParamへ広げない。
- 完全な`old_value` / `new_value` `DocParam`。
- `added_key_id`。
- ちょうど1 IDの`StableIdReservation [before, after)`。

apply / replayがlive target、old value、導入ID集合、reservationを再検証し、初回だけcounterをcommitする。Undoはold valueへ戻してcounterを戻さず、Redoは同じpayload/IDを復元する。既存v1/v2 `SetProperty`へ必須fieldを足さず、新Command variantを既存journal versionで未知variantとして型付き拒否可能な追加形にする。format/min-reader変更が必要と判明したら実装を止め、M2永続形式改訂へ戻す。

prepare結果は`Edit { owned command, key_id } | AlreadyPresent { key_id }`相当のruntime-only閉集合にし、no-opを空macroや仮Commandで表さない。ただし`Box<Command>`、公開可視性、exact type名はcurrentコードでsize/API監査後に固定する。

### 4.3 curve-preserving insertion

既存区間内へkeyを挿入しても評価曲線を変えない。

- Holdは左右Hold、Linearは左右Linear。
- Bezierはtimeline progressに対応するcubic parameterでde Casteljau分割し、左右を各部分区間の0..1へ正規化する。
- equal Position endpointsは同値区間としてHoldへ正規化できる。
- 非有限、無効x control、正規化不能なdegenerate spanはtyped rejectし、Linear近似やclampへ黙って落とさない。
- solverは`motolii-eval`の既存`cubic_bezier_ease`と数値規則を共有し、`motolii-doc`へ第二solver/EPSを複製しない。

旧exact contractのRust関数名、`1e-12`、257 sample、`1e-6` toleranceは再入場時の候補oracleとして保持するが、現行public API/数値契約にはまだしない。実装発注前に現在のsolverと反対側レビューで固定する。

## 5. U2h-1 Host Transient primary selection再採択

### 5.1 ownerと閉集合

Host event-loop runtimeが`DocumentEditRuntime`の隣でtoolkit非依存の`primary: Option<LayerId>`だけを所有する。DocumentWriter、Document/serde、journal、Undo、workspace、Project sessionへ保存しない。keyboard focus、hover、panel、window、keyframe focusとも分ける。

操作はvalid `ReplacePrimary(id)`と`ClearPrimary`だけ。同じID、already-noneはno-op / no publish、unknownまたはtable-only IDはtyped rejectとする。妥当性はrecursive `DocumentWriter::find_envelope`相当だけで判定し、`LayerIdTable::contains`単独を使わない。

### 5.2 atomic publish

Documentとselectionを別channelで同期せず、次の意味を一つのprivate envelopeでatomic publishする。

```text
document_revision
Arc<Document>
primary selection
projection_generation
```

selection-only変更はDocument revision、serialize、history、journalを変えず、projection generationだけを進める。same-ID/no-op/rejectはgenerationも進めない。

D2 Apply / Undo / Redo成功後、publish前に必ずprimaryを現Documentへreconcileする。danglingならclear、validなら保持、Redoでselectionを暗黙復元しない。Place成功時はreceiptの新IDを同じHost turnでreplaceしてからDocumentと一回だけpublishする。Stage、Timeline、Inspector、KEYS/LAYERS、Easing triggerは同じenvelopeのread-only projectionだけを持つ。

U2h-1はsingle-primaryだけでU3a-1に依存しない。additive / range / marquee / bounded AX projectionはU2h-2とし、U3a-1以後へ残す。

## 6. U3a-1 headless Timeline再採択

### 6.1 G0-9から分ける境界

U3aを次へ分ける。

- **U3a-1**: toolkit/renderer非依存のDocument projection、packing/layout、viewport cull、hit-test。G0-9非依存。
- **U3a-2**: windowed native renderer、direct wgpu/Vello比較、WebView同居、input/present/platform受入。G0-9依存。

これはU3a-1を現在のSelected U seriesで直ちに次発注へする決定ではない。現行直列運用を保ったまま、論理依存と実装責任を分ける。

### 6.2 最小投影

- 入力は`&Document`とcaller注入metrics/viewportだけ。Document、revision、history、selectionを変更しない。
- top-level Clip barは既存`LayerId`、半開`[start,start+duration)`を投影する。
- Position Constはkey 0、Keyframesは既存`LayerId + KeyframeId + exact RationalTime`を投影する。Data / Vec2Axes / LookAt / Followはtyped unsupportedで、X/Yへ黙って分割しない。
- Group bar / children hull / 展開は意味未決として非目標。黙ってskipしてTimeline全体完成と書かない。
- bar/keyを第二Document objectや別ID空間にしない。

### 6.3 packingとhit-test

固定Track laneやspikeの`track: u32`を使わない。barを`(start,end,LayerId)`でsortし、半開区間が重ならない最初のbandへfirst-fitする。接する区間は同band、重なる区間は別band。packing後にviewport cullし、cullでidentityやband結果を変えない。

時間正本は`RationalTime`、最終座標だけ`f64`。band height、diamond half-extent等はcaller注入で、色、font、固定px、DPIをheadless契約へ入れない。hit priorityはManhattan diamond key > bar > none。同優先のkeyは`LayerId, KeyframeId`、barは`LayerId`で決定的にtie-breakする。invalid duration、overflow、NaN/Inf metrics、invalid viewportはtyped rejectし、部分layoutを成功返却しない。

既存1k clip / 100k key実測はcapacity/rendering foundationの証拠だけに使う。Document projection、stable ID、RationalTime、hit-test、D2を証明せず、U3a-1入場時の100k再実測も要求しない。正しさは小さな決定的Document fixtureで審判する。

## 7. 共通STOP線

- 4契約を一つの実装発注へ束ねない。Place、Add Key、selection、headless Timelineを各1境界に分ける。
- historical docs-only決定をコード完了、journal durability、Preview画素完成、四面E2E完成と書かない。
- public raw allocation、`from_raw`、`peek_next`製品入口、汎用planner、公開transaction lifecycleを足さない。
- selection、packing band、drag ID、layout epoch、px/DPIをDocument/journal/plugin契約へ保存しない。
- Add Keyのdurable Command追加で既存SetProperty wireの意味を変えない。`serde(default)`でreservation/IDを捏造しない。
- Group insertion/layout、Track新設、multi-composition、Auto Key、X/Y分離、appearanceを隣接境界として発明しない。
- spikeのf32/fixed-track/wgpu modelをheadless product crateへportしない。
- test期待値、golden、lint allow/ignore、special-caseで契約を迂回しない。

## 8. 固定歴史出典

| lineage | 読み方 |
|---|---|
| PlaceRectangle | 初版`1afb62c9`を全文、product-core接続版`1340181d`をdiffで確認 |
| Position Add Key | 初版`87b3f955`を全文、implementation closure版`9152ad5e`、boxed DTO版`96dd7c52`までdiffで確認 |
| Place product core | `git cat-file -p bd6bc065ae031d5693b7eec82404878f12955c79` |
| primary selection | `git cat-file -p 059c0bb212858e7d170f8be6ad45ff953238a925` |
| headless Timeline | `git cat-file -p 3a99cbd5ace2c03ef2a275694b13937b98728df4` |

これら8 blobは本書でDISPOSITIONEDとする。旧文書の実装許可file、task順、exact Rust signature、数値閾値を現行発注書として直接使わず、本書と現在のM2/M3仕様からclosed orderを作り直す。
