# レーンボード — 2026-08-21(このセッションの走行状態の正本)

日付: 2026-08-21〜22 / 状態: **第3セッション延長戦・走行中**(引き継ぎ文書は 2026-08-22-…-hierarchy.md — 延長戦分は本ボードが正)(引き継ぎ= 2026-08-21-session-handoff-night-s-score.md。前任分= -ui-completion.md)
運転手順: レーンの完了・発注のたびにこの表を更新する。TaskList はセッション死で消えるため、この文書が正本。

**実窓較正第1回の初回収(2026-08-21深夜)**: 利用者「イージングがガタつく」— release ビルドでも再現=構造確定 → **裁定166**(Stage presenter shader 化+tick vsync 整列)→ τ 着地後に**利用者確認「スムーズになった」= 合格**(実録動画 12s を利用者へ送付済み)。第2の較正収穫=「文字が酷い」(→裁定168・φ)、第3=「どの要素もチグハグ」(→ψ 台帳)。残りのチェックリスト(音・状態帯・市松・差し色・目盛り)は φ 着地後の窓で再開。

**実窓一瞥 第2回(2026-08-22・第5セッション)**: 利用者「ようやく動画ソフトっぽくなりました。UI は概ね OK」= **概ね合格**。不合格1点= Browser の構造文法が media タブのみ(他タブはサムネイル羅列)→ BR-sym レーン。UX 課題1点= キーフレームが1つしか打てない(AE 作法を習う UX へ — 機序仮説: keys<=1 編集可規則が track を静的に戻す)→ KF-ae レーン。以後**本格自走**(利用者宣言「では!よろしくお願いします」)。

**第5セッション 全レーン着地(2026-08-22)**: pane_grid検収 / BR-tabs / PH-handle / IB-const / TL-transport / MB-vendor / IK-keyframe / iced-inventory / weblike-layout / TF-container(taffy widget — grid対応込み・実欠陥1件をテスト先行が事前捕獲)/ CM-port(css_metrics 移植 — dimensions 6値一致)。裁定178〜184。**taffy 三点照合(モック→css-metrics→dimensions→TaffyBox)が開通 — 転写の手癖工程は次の pane 書き換えから廃止可能**。走行中レーンなし

## 走行中(未返却)

| レーン | 種別 | 場所 | 中身 |
|---|---|---|---|


## 完了・main 着地済み(実装)

| レーン | 結果 |
|---|---|
| MB-2(検収合格・merge) | ヘッダー箱ボタン列→ menubar(File/Edit/Layer/View・shortcut 併記の実行照合 oracle)+右端 icon ボタン2(icon+tooltip ペア第1号)。**Freeze/Unfreeze 初露出**(新 Message 2本 — 逸脱受理・group_drive 3本)。menu.rs は意味台帳化(widget 自作 348行を撤去)。q0 除外は MenuBar 自己登録 bounds の最小面積のみ。267緑 |
| LN-D2(検収合格・merge) | Inspector 値セル/ボタンの常時輪郭廃止 — `HoverValueBox` wrapper(fork container が hover status を持たない実測への「迂回よりwrapper」)。幾何不変を ±1px 柵の緑で証明。drag 中視覚は hover 箱で代替(利用者チェックリスト行き)。66緑 |
| LN-D4(検収合格・merge) | Browser チップ/rail 輪郭を選択の器へ — mock の非選択常時輪郭だけ裁定179 が上書き(衝突点の明示)。透明 border 方式で幾何不変。容器罫線は D5 の領分と線引き。74緑 |
| KF-ae(検収合格・merge) | キーフレーム AE 作法 — 真因=「keys<=1 のみ編集可」規則が値編集で track を静的化しキーを消していた(利用者「1つしか打てない」の正体・仮説一致)。`edited_value_track` 純関数= キー持ちは playhead へ upsert(値更新 or 挿入・interp/spatial 保持)・静的は静的のまま。Q0 達成(値セル常時編集可)・1 undo=1 SetTrack・324緑 |
| BR-sym(検収合格・merge) | Browser 構造文法の全タブ対称化 — モックはタブ別に宣言済み(tabScoped 節)だったので転写: effects=Color/Utility/Animation・create=Shapes/Built-in・panels=Tags/Notes/Export の rail+チップ+カード click。media 回帰なし・69緑。逸脱7件は理由つき(COLLECTIONS 予約地の対称留保等) |
| IC-svg(検収合格・merge・第5セッション) | `motolii-icons` 新設 — Material Symbols 37個 vendoring(全24px outlined・1KB未満・Apache2.0/NOTICE同梱・rev固定)+transport 5ボタン SVG 化。`icon(Icon,size,color)`+`frame_px_for_glyph_px`。既知の限界: svg は押下瞬間の ink 変化が効かない(iced svg Status=Idle/Hovered のみ — 面浮き+再生中accentで裁定179の実質維持) |
| pane_grid 第1切片(検収合格・第5セッション回収・merge、suite 緑5本 log 直読) | shell の pane_grid 化 — リサイズ+ドッキング・Browser=左全高カラム・グリップ帯8px(title_bar 無しでは drag 不能のため)・q0_fence 偽陽性155件を on_click 実配線で解決。**既知制限: Browser 開閉= Configuration 再構築で drag 並べ替えが失われる**。screenshot.rs は旧・上帯 Browser のまま(task chip 化済み)。分離レーンへの引き渡しメモ(daemon 化の signature 変更・Stage 単一 surface 前提)は RETURN 参照 |
| BL3: blend 分離可能11モード(検収合格・merge `118cdbf4`、後任セッション回収) | W3C 3.6節の2枚読みWGSL(`compositor/src/blend.rs`)+engine変換表11値(`_`なし網羅match)+Inspector巡回13値+golden 11枚(tol::EXACT)+独立Rust実装照合oracle。store enum は既存17値で無改造。3経路を `accumulate_sequential` コアへ統合。**副産物: `background_rect` の `depth_offset: i16::MIN` 縮みバグ再発見・修正**(レーン B と同型・2度目 — 極端値禁止を doc 化)。conflict解決1件(SR の scratch プール返却を finalize_texture 後へ移植・「確保5→1」テストが正しさを捕捉)。**FINDING: render_to_texture が all-Normal でも layer 毎 submit+poll を払う構造退行 → run-batching レーンで根治済(下記)** |
| run-batching(検収合格・merge `09ef13a6`+柵 `4a214f7d`) | BL3 FINDING の根治 — Normal/Add 連続区間を単一 ViewBuilder/submit へ束ね、分離可能 blend の出現点だけで切る。all-Normal 3層= submit 1(実測、旧3)。`sequential_submits` introspection 新設・分離可能パス無改変・byte一致/golden 全緑。supervisor 追記: inputs=depth_offset 非減少の debug_assert(SetOrder 将来配線の黙殺防止) |
| freeze/unfreeze 意図動詞(検収合格・merge `048978bf`) | 裁定119 §4 第1切片(store 層のみ)— `LayerAttrs.frozen`(**Patch から意図的に除外** — 汎用 patch の凍結すり抜けを型で封じ)+`Intent::Freeze/Unfreeze`+凍結ゲート11 arm+reparent 侵入拒否+ungroup 拒否/削除許可(tombstone 可逆の論証)。設計逸脱1件受理: `LayerSource::Group` の形は不変(下流4 crate の網羅 match 保護)。store suite 255 緑。後続: engine キャッシュ束(fingerprint)・MB-2 露出 |
| A: shell テストバイナリ統合 | 10本→2本、フルリンク 45.5s→18〜26s(裁定138の本丸) |
| B: 外周1px alpha=0 根治 | 真因= `order: i16::MIN` の depth_offset 縮み。`BACKGROUND_ORDER=-1` |
| C: 線化第1弾 | tokens weight/ink 段・Inspector hairline 化・値セル横余白(裁定137/139) |
| D: track 解析キャッシュ | r2 投影 15530→495µs(裁定140)。revision 鍵・proptest 等価柵 |
| E: 市松 AE 型可視化 | 不透明背景でもトグル即応(裁定141)。理由文 chrome 撤去 |
| 器具 Inspector 拡張 | screenshot 器具が Inspector を描く(線化の視覚検分ギャップ解消) |
| transient drag 書き換え | 幽霊 redo エントリのワークアラウンド撤去。履歴構造的無傷 |
| 線化第2弾+明暗リズム | Timeline ゼブラ×時間陰影(絶対時刻基準・パンで動かない)・Settings/chrome 線化・token 3ロール追加。egui グループ階層の構造メモ(第2波用)付き |
| iced theme 結線 | 実窓の地色が OS 外観フォールバック(Light になり得た)→ tokens 由来 `Theme::custom` へ。watch 追随可・柵4本。注: palette の danger ロールは正本に無く status_warning を仮当て |
| トンマナ柵 | 裁定142 完結。**現行コード違反ゼロの実証**+実ファイル赤→緑証明+除外表の識別子存在ガード。曖昧4件(器具幾何)は現状維持と検収判断 |
| effect 縫い目調査 | 縫い目不在の確定・挿入点3案比較・5切片割り(→裁定153。保全: docs/reviews/2026-08-21-effect-seam-survey.md) |
| S1: store resolve 拡張 | ResolvedEffect(ResolvedMask と同型)・時刻評価済み param・空 stack 早期 return。probe 平坦(r2 289〜300µs) |
| S2: compositor 枠 | LayerWithPasses 別口(Layer 不変)・Identity=copy_texture_to_texture・scratch pool 再利用証明・同一 submission で第二パス禁止維持 |
| S3: engine 語彙変換 | translate_effect_passes(空 match、S4 が腕を足す)・render_with_effects 切替・背景 pass なし。全経路開通 |
| Timeline 切片0 | timeline/ 5分割(--list 131/131 一致・PNG バイト一致・柵は対象拡大)。後続 write-set 割り表つき |
| S4: Glow 移植 | vism 第1号が動いた(隣接画素 0→12 の加算 bloom 実証)。HDR>1.0 がマゼンタ破損検査に触れる経路を発見し白飽和規約へ clamp |
| T1: レーンバー | rail 150px(mock 出典 token)・M/S/L 実 Intent 結線・名前のクリップ面退去・ゼブラ連続。視覚検収合格 |
| S5: golden+器具 | Glow golden 2枚(max=0 安定)+fixture 搭載・実絵で視覚確認済み。**裁定153 の5切片完結 = vism 第1号開通**。既知の穴: pass は layer 境界内のみ(halo あふれ未実装、KNOWN 記載) |
| T2: clip move/trim | 正典 §2 全部(BarPart 8px・スナップ7px 画面距離・Cmd 一時トグル・Esc/RMB・ロック拒否 M13)。逸脱1件は理由つき採択(timing preview は pane-local — §5.5 に実装注記) |
| T3: キー行+菱形 | property_rows 投影(キー持ちのみ既定)・菱形選択3種・Delete キー優先。finding: hit 経路の縦ズレ → T3b へ |
| T3b: hit 経路統合 | 縦ズレ根治(囮 layer での赤証明つき)。y 計算の正本= layer_row_top 一本化を grep 検証。第1ラウンド完結 |
| halo 拡張領域 | EffectPass::padding 宣言で AE 型 halo 実装(縁外 0→114・統制点で有界証明)。golden 更新。発見: 複数 pass は連鎖しない(KNOWN へ) |
| 普通地図: CapCut 列挙 | menu40/shortcut12/panel5/pref6。矛盾ペア両論併記 |
| 普通地図: AE/Premiere/Resolve 列挙 | AE 417/315/32/21(Adobe封鎖をプロキシ+community実機抽出で突破)/ Premiere 71/141/23/16(二次資料・質フラグ)/ Resolve 357/194/15/28(公式マニュアル完全列挙)。4本とも docs/reviews/2026-08-21-normal-map-sources/ へ保全 |
| カメラ縫い目調査 | レンダリングカメラ実装済み・export 構造隔離確認。観測カメラ置き場= Shell 直下(→裁定157)。4切片割り |
| カメラ S0: engine 第二エントリ | ObservationCamera(pan+zoom)+render_frame_with_view_camera。既定視点=レンダリングカメラの**バイト一致証明**・export 経路不変を grep 再確認 |
| 普通地図: 併合 | **単一台帳 normal-map.tsv 1,551行**(捨て行ゼロの機械検証) |
| **普通地図: 完成(裁定158)** | **未判定0** — 採用済92/採用予定1,195/不採用264(全行理由つき)。最優先キュー= freq≥2 の42件。以後の UI レーンは行 id を発注書に書き、着地で verdict を更新 |
| T4: キー時刻編集 | ドラッグ・nudge・Cmd リタイム(数値証明: scale 3/7 で 70→41)・衝突の決定論解決。逸脱4件は理由つき(Cmd 二役は click-vs-drag 分離)。finding: ライブプレビュー不在 → T5 へ |
| T5: ライブプレビュー | §5.5 根治 — 投影段の純関数置換(赤→緑証明)・ACCENT 強調・タイムコードミニラベル。finding: fmt ドリフトは既存(未介入) |
| U1: レイヤー編集動詞束 | アプリ内クリップボード(snapshot 丸ごと・1 undo)。map 6行消化。finding: Split 未実装(正しく未消化)・multi-select ハイライト未配線 |
| U2: playhead ナビ束(再発注) | **初回は crates/ legacy へ誤実装し棄却(SHA f9739a0d、KNOWN へ教訓化)**。再発注で next/ に正しく着地 — Step/Home・End/J・K/I・O、map 10行消化。**台帳消化 108/1,551** |
| GitHub 置き換え+README | origin へ472コミット push(旧世界は歴史として保存)。README フック書き換え(「2つの if」— 意味を保つ合成の遊び場×架けられなかった橋)。AviUtl への評価は不記載に修正済み |
| **カメラ campaign 完結(裁定156/157)** | S0 engine 第二エントリ+Shell 側 S1〜S3。観測=wheel アンカーズーム/中ボタンパン・Shift+F で復帰・**フレーム枠 overlay 視覚合格**・export 汚染ゼロ直接証明+構造柵(呼び出し点 grep 固定)。既知境界: 観測中は市松プレビュー非合成(doc 明記) |
| (supervisor 直) 色 token 追随 fix / 市松レーン回収 / 引き継ぎ123コミット着地 | main 前提の整地 |
| (supervisor 直・後任) multi-select 行ハイライト配線 | U1 finding 根治(`830bce23`)。`row_selected` 純関数抽出+赤→緑・pane36+shell156 全緑。primary 区別は property 行展開のまま(AE 同型)。実窓合否は利用者チェックリストへ追加 |
| **A2: 実時間再生 第2切片(検収合格・merge `f026e7cc`)** | **再生できる動画ソフトになった**。cpal device/producer(旧 PlaybackSession 移植・rtrb 化)+`PlaybackSession`+shell `transport.rs`(Space=Play/Pause・拘束5 ドラッグ中無効・playhead 追随 tick・scrub=seek・終端自動停止・soundtrack 無しは無音成立)。oracle (a)〜(e) 7本+audio 59+5 全緑を後任 supervisor が main checkout target で再実行・workspace 94 スイート全緑。map 1041 採用済(消化 109/1,551)。発見: `iced::time::every` は本 workspace で使えない(KNOWN 追記済み、`stream::channel`+OS スレッドで自作)。既知の制約: seek 時リング容量 4,096 フレーム(~85ms)分だけ古い位置の音が流れきる。**前任セッション発注・後任検収の初のセッション跨ぎレーン** |
| (supervisor 直・後任) テスト警告 196 件ゼロ化 | `2ed52c1e`。裸の `shell.update()` へ `let _ =`(T5 期からの蓄積、A2 起因ではない)+未使用 import 除去 |
| **α: blend Add 縦一本(検収合格・merge `4569a366`)** | Normal 以外で初めて絵が変わる blend。store `Add` variant(serde は名前ベースで並び挿入安全を確認)+engine 変換腕+Inspector **クリック巡回ボタン**(Normal→Add→Normal、SUPPORTED 台帳に engine 同期義務 doc)+shell `SetAttrs` 即時1発。oracle 赤(コンパイル赤保存)→緑8本・shell suite 165 全緑・check.sh 通過。map 行75 採用済(**消化 110/1,551**)。store stale doc 2箇所も掃除(R9 FINDING 1 根治) |
| **β: BL1 逐次合成の枠(検収合格・merge `22f8e436`)** | **狙いどおり EVIDENCE_GAP の確定が成果**: 重なり半透明でバイト不一致(1264/4096px・maxΔ49)、真因= gamma 空間(`composite()` が layer 毎に srgb round-trip)。ライフサイクル・第二パス禁止柵は障害でないと実証。scaffold+`#[ignore]` 証拠テストを歴史証拠として merge。**→ 裁定161**(fork へ main_target accessor、BL1b へ) |
| **γ: MK1 mask ラスタ配線(検収合格・merge `fe0724d7`)** | 未配線 `motolii-vector` を engine へ。`Path`→`Shape` 橋(単一輪郭を輪郭列に包むだけ・別型ではないと確定)+白 fill=coverage の純関数。oracle 4本(二値・AA 縁有界・面積・byte 決定論)。EVIDENCE_GAP: mask 座標系(comp 絶対 vs layer local)は Canvas を呼び手が組む設計で MK3 へ先送り |
| **δ: speed 操作面の文法調査(回収)** | 保全= `2026-08-21-speed-ui-grammar-survey.md`。採択=2段構成(第一波 Inspector 数値欄・第二波 Shift+端drag は**利用者実機確認後**)。speed でキー時刻は動かさない(拘束7(a))。bar 文脈の空き modifier は Shift のみとコードで確定 |
| **ρ: レイヤー差し色 第一波(検収合格・merge済み)** | `label_color` index(AE 同型)+`id%12` 決定論割当+bar 塗り+Copy/Paste/Duplicate の色継承。候補 A/B/C を同一 fixture で比較し**利用者裁定= C(トンマナ従属)確定**。ALLOWLIST 逸脱2件(fixture/screenshot — 比較 PNG 成立の必須)と clipboard 追随は理由正当で採択 |
| **σ+σ2+梯子補充(検収合格・merge済み)** | **初のモック駆動発注**: 全目盛の縦線(帯=面・大/線=細・全目盛の周波数分担、新ロール `timeline_grid_minor/major`)→ **比率の原則**(利用者裁定: 形は比率で定数化)で梯子選択を `TARGET_CELL_RATIO=0.52` 最近傍へ(σ2)→ 梯子へ 2f・半秒を補充(supervisor 直、1426px で 15f=0.457)。S4 へ比率原則・器具節へ「モック=転写元」を正典化 |
| **意味論モック4枚(利用者合格・main 資産)** | `next/reference/mocks/`: timeline(合格・転写元第1号)/ **browser-library.html=旧世界視覚正本の移植**(色のみ tokens 追随・CSS インライン)+browser-semantics(意味層・救出台帳)/ stage v4(**boxcam**=破線+ハンドルのカメラビュー・**User View** 改名・視点タブ・帯アイコン化・**UI 英語**・AE 対応表) |
| **ν: 目盛り梯子+大目盛帯(検収合格・merge済み)** | 利用者裁定の根治 — 時間整列の梯子(小=px下限10で選ぶ最小ステップ・大=5/10倍)へ置換、全尺等分(7.5s級の半端)を撤去、**明暗帯の区間=大目盛周期**に一本化。canvas/projection/screenshot が単一 pub 関数 `tick_steps` を共有。timeline-pane 50+shell 189 全緑・PNG 検分済み(縦リズムがプレードから階層へ) |
| **市松v2(検収合格・merge `75ab2b0c`)** | 利用者較正「市松が見えない」の根治 — 専用 token 対 `checkerboard_light/dark`(Δ8→**Δ30**、S4 新柵の初適用)+表示空間タイル 8pt 一定の純関数補正。器具 PNG はタイルが追加縮小される既知あり(色差は画素実測済み)。実窓の手触りは利用者チェックリストへ |
| **(supervisor 直)行ゼブラの rail 退去** | 利用者知覚モデル「rail=時間カメラ外の上層」を doc 化し wash を時間場に限定(`947790a9`) |
| **μ: Stage 下縁状態帯(検収合格・merge `3737d5c4`)** | **裁定163 適用第1号が着地**。視点状態(観測中のみ・クリック復帰= S6 違反第1号根治)・解像度 cap Auto/½/¼(既存 sqrt 予算と min 合成・RenderedFrame キャッシュ鍵へ追加)・市松の引っ越し(Settings から撤去)。赤→緑・216 全緑・S5b の PNG 実測つき。ALLOWLIST 逸脱2件(metrics fence・main.rs 追随)は理由正当で採択 |
| **ξ: S 定数化の外部理論調査(回収)→ 裁定164** | 保全= `2026-08-21-spatial-score-constants-survey.md`。採用: KLM 秒数(S2 格上げ)・WCAG 比の ink 段・Rosenholtz/Miniukovich の画素判定基盤・ISO 9241-110 分類・ヒット寸の誤借用柵。棄却: F/Z(反証済)・黄金比(特定値の根拠なし)。S5a 占有率は外部定数なし=実測較正が正、と確定 |
| **κ: UI 入口台帳(回収)→ 裁定163 S 空間スコア制定** | 保全= `2026-08-21-ui-entrance-atlas-survey.md`、正典= `docs/ui-spatial-score.md`。S0 慣習段差(辞書式最優先)〜S5 ヒーロー構図(pane ごとに主役を逆算・補完は縁へ・占有率/濃度序列は画素判定)+S4 視覚重み写像(発見依存度→tokens 段量子化)。初期在庫と器具計画込み。**次の適用第1号= Stage 下縁状態帯**(1/2 プレビュー入口+市松の引っ越し) |
| **λ: 編集ショートカット8本配線(検収合格・merge `f8c34328`)** | κ FINDING 1 の根治 — Cmd+Z/Shift+Z/C/V/X/D/A/Shift+A。赤8本→緑 182/182・captured ガード試験込み。S0 群0 の shortcut 側を焼却(menu 側はメニューバー campaign 待ち・map verdict は据え置き) |
| **BL1b: fork accessor+線形化(検収合格・merge `cbbe4f2b`)** | **裁定161 完了 — blend 逐次合成がバイト一致で開通**。fork 実質差分 +15行(accessor 1本、`856f597c3` GitHub push 済み・全 rev pin bump 済み)。Motolii 側は**新規 WGSL ゼロ**(既存 RectangleDrawData+import_gpu_premultiplied 再利用、「直前結果を背景 rect として同一 main_target 内で blend」の構成転換で 8bit 量子化往復も回避)。#[ignore] だった overlap テストが緑化。罠2件(texture pool の destroy / Nearest 固定)は doc 記録済み。**BL3(分離可能11モード)/BL4(非分離4)が発注可能になった**。seam 台帳追記済み |
| **ι: Browser B1(検収合格・merge `80577081`)** | drop が台帳へ記帳される — bin-first の下地開通。記帳と配置は独立(junk file も fingerprint が読めれば台帳に載る)・undo 粒は既存(1 drop batch=1 undo)に同居・dedup は η の content_hash 統合に一任。赤(E0599 5件)→緑 174/174。FINDING: multi-GB 素材の hash 時間未計測(KNOWN へ B7 リスクとして記録)・asset_type は拡張子の仮判定(意味起草タスク#14 待ち) |
| **η: 素材台帳移植(検収合格・merge `d5370a0e`)** | 裁定162 の核心が着地 — 旧 asset.rs 740行を store へ(fingerprint は移植済みを参照)。`Intent::AdmitAsset/RemoveAsset`(read-modify-write、`Composition:assets` component の JSON blob = markers/slots と同じ流儀)・`StoreView::assets()/asset()`。persist は汎用経路が無改修で運ぶ。dedup の赤→緑実証あり((a)(c) は赤未保存 — 実装同時書きの逸脱として記録)。store 21 スイート+workspace 全緑 |
| **θ: browser-pane B0 骨格(検収合格・merge `8a4e7ffb`)** | 新 crate(`//! wraps:` マーカー・空 Message enum・空 view)+shell 埋め込み(`Message::Browser` 腕のみ・view 未配線)。**PNG SHA `ee217ba2…` 前後一致を supervisor 再実測**(pane 分割以来の正準ハッシュ)。shell suite 170 全緑。B1 は η(素材台帳)着地後 |
| **M-menu: メニューバー基盤設計調査(回収・merge 済み)** | 保全= `2026-08-22-menubar-foundation-survey.md`。構造は map 分布から導出(AE 8トップレベルを Layer 語彙へ畳む・全項目に行 id+freq)・**アプリ内メニューバー推奨**(winit に NSMenu 統合なしの実測)・道具= iced の `overlay::menu` primitive が pub で流用可(wraps 層)、iced_aw は fork 互換未検証(GAP)。**S6 併存表の重要発見: New Project/Save As/Zoom In·Out/Paste Attributes/Find 等は現在入口ゼロ** — メニューだけ足すと S6 違反になるため、同切片で shortcut/直接操作の併設が義務。切片 MB-0(基盤 widget)→MB-1(File/Edit)→MB-2(Layer/View)→MB-3(右クリック基盤)。**MB-0 は TP 着地後**(shell write-set) |
| **G1: グループ化動詞(検収合格・merge `922f6a27`)— 変換階層が動詞まで完成** | 裁定174 施工 — **新 Intent 不要**(既存語彙の合成で ⌘G/⌘⇧G)・ungroup は「祖父母 world が代数的に相殺」証明つき焼き込みで world 位置保存((95,10) 前後一致)・projection.rs 無改修でツリー反映。**map 6行採用済= 消化143/1,551**。merge 時の意味衝突2件(menu doc・Item Option 化)は supervisor 解決。FINDING: 焼き込みは t=0 評価(アニメ付き Group の解除は近似 — 在庫)。workspace 40 binary 全緑(レーン内)+storm flake は release 単独緑 |
| **CF: cap 高速路修理(検収合格・merge 済み・supervisor 収容)** | RB 発見4根治 — cap≠Auto でもゼロコピー維持・縮小は fragment の pixel_scale 量子化(naga 検証済み)。「comp 出力縮小は帯域を救わない」(RB 発見1)を doc に正直化。red(cap=½ で何も描かれない)→緑。**未コミット報告3例目+1** — status 貼付指示が効いて即発見 |
| **SR: effect scratch 再利用(検収合格・merge `b31c6a1f`)** | RB 発見3根治 — 同一 queue 順序論証(裁定171 §5 と同じ議論の直接適用・新機構なし)で poll なし再利用。確保 5→**1**・stale 画素の直接検証つき。compositor 41 緑 |
| **IB: 意図束草案(回収・merge `16636fef` — **利用者の命名一瞥待ち**)** | **44束+宿無し3行**(AI 系3行 — 生成境界原則との衝突で verdict 再審要)・新規大型統合13件(7〜8カテゴリ跨ぎの束を発見 — 作業範囲束は8カテゴリ合流)・束単位 freq TOP10・全1,285行の機械検証(重複ゼロ・欠落ゼロ)。**次: 利用者が束の名前と形を直す→bundle 列を map へ+機械検査** |
| **MB-1: File 束(検収合格・merge `46f4481d`)— File メニュー出現・freq4 の頭を消化** | 裁定176 施工 — `FileDialogs` trait(注入境界 — rfd を test で呼ばない構造)+New Project(⌘N・dirty 確認)/Save As(⌘⇧S・既存汎用 persist=.rrd)/Save a Copy/Quit(⌘Q)。**map 1221/1223/1225/1227 採用済= 消化137/1,551**。FINDING: q0 の File 除外は構造保証で代替(Quit=exit(0) が test binary を殺すため — 理由 doc 化)・**Open は境界 FINDING**(load は存在・Shell 配線+破損/recent/大型コストの3設計問題が未決 — 専用切片へ)。process 逸脱1(red checkpoint なし — 自己申告)。shell 221 緑 |
| **RB: 残ボトルネック軽調査(回収・merge 済み)** | 保全= `2026-08-22-residual-bottleneck-survey.md`。**実像**: GPU=ソース解像度×層数の帯域律速(comp 解像度でない — r1 実測 40層1080p≈40ms・ソース1/4で5.9ms)・decode= ffmpeg プロセス起動/cache miss+64枚 LRU 永続なし(ゼロコピーと直交の日常コスト)。**バグ発見: resolution cap ½/¼ は高速路を無効化し CPU 後段縮小のみ=「速くするボタンが遅い」**(chip 発行済み・修理は MB-1 着地後 — shell 衝突回避)。M4 の effect scratch 非再利用も記録。保留14行の判定: cache 11行=(a) 根は実在(対処は永続 cache が先)・Fast Previews 3行=(b) probe 待ち。新玉: 永続 frame cache(大玉)・scratch 再利用・cap 修理 |
| **H4: シェイプ入れ子(検収合格・merge `4df3aadd`)— 再帰の3つ目の顔** | `ShapeNode::Leaf/Group`(Shape 本体は1フィールドも触らない設計 — engine mask.rs 無改修の理由)+`flatten`(H1 と同一の合成数学 — 数値証明も同じ (110,5)/(95,20))。repeater の Affine を単一源へ昇格(core 依存を作らない)。移植/新設の区別を正直に報告。逸脱1件(clipboard 機械追従)理由正当。vector 67+workspace 111 スイート 978 全緑 |
| **B3: Browser 絵+view 配線(検収合格・merge `471e1500`)— Browser が実窓に出現** | header トグル+カード grid(比率定数は行高分母で宣言・micro_text 初消費)+`--browser-open` 器具フラグ+q0/atlas 新状態。**map 980 完全消化= 消化133/1,551**。drop→記帳→rail/filter→grid の端到端 oracle。browser 26+shell 212 緑 |
| **MA: normal-map 全面精査(回収 — 提案は未適用・次セッションへ)** | 保全= `2026-08-22-normal-map-audit.md`(worktree 未コミット報告の2例目 — supervisor 直接収容)。verdict 修正提案(1450→採用済・Expression 6行→保留(D5まで)・441/1054/1080→不採用=跨カテゴリ重複バグ発見・909/910 は理由引用を173/174へ)・L8/L9 型見本(ペイント族=島=継ぎ目候補)・L4 縮約9クラスタ→ **freq≥2 キューが29→21意図へ**・**争点束13行(利用者裁定待ち — 次セッション第1手)** |
| **SS: 構造解決台帳(回収・merge `04186411`+supervisor 追記)** | 三層フィルタの一次資料 — 構造解決14件(map 対応行 id つき)+未完9件(**verdict 自己矛盾2件を発掘**: Pre-compose 909/910 は H1 着地前に不採用確定・Expression 系6行は代替 D5 未完のまま不採用)+委託9技術(iced #3281・rerun #2852 を一次確認)。supervisor 追記: **Intent 背骨=スクリプト命令集合**(利用者洞察 — スクリプト系 map 行は薄い顔) |
| **H1: 親変換合成(検収合格・merge `74da6c42`)— 「親が動くと子も動く」成立** | 裁定173 施工 — `StoreView::world_affine`(旧世界 spatial_resolve 移植・LookAt/Follow を落とし parent 1本へ単純化)+memo(3階層で計算4回の白箱証明)+循環は local 縮退の防衛ガードへ格下げ(書き込みガードが一次)。`LayerSource::Group` マーカー着地(members なし)。数値証明: 平行移動 (110,5)・回転スケール合成 (95,20) 手計算一致。engine は「描かない」腕合流のみ・compositor 無改修(H-survey 実測どおり)。red 本物(Group 未定義でコンパイル赤)→11/11 緑。**workspace 110 スイート 972 全緑**。残: ~~H3~~ → **裁定174 で G1(グループ化動詞)へ置換**(957 は不採用・455-457/468-470 が G1 対象) |
| **MB-0: メニューバー基盤+Edit 束(検収合格・merge `2a145ddb`)** | **メニューバー初出現** — 逸脱採択: overlay::menu の Overlay は operate() を持たず simulator 不可視(実測)→ Settings パネル型(view-flag+通常 button)へ転回、dropdown 全項目が Target で見える。Edit=既存入口持ち8動詞のみ(S6 安全・shortcut 併記)。q0/atlas/tonmana 柵へ新状態を追加。**map 8行採用済= 消化132/1,551**。shell 211 緑 |
| **MK2: mask 被覆代数(検収合格・merge `8fa92130`)** | 6 mode(Add/Subtract/Intersect/Lighten/Darken/Difference)+**mode 別単位元の設計**(AE 実機の「先頭 mask の mode で絵が変わる」に oracle 4本で一致)。serde 後方互換(mode 欠落=Add)。red 先行13本。**map 11行採用済= 消化124/1,551**。FINDING: r2 の負荷 flake 1件を新規観測(KNOWN へ supervisor 追記) |
| **TP: transport 転写(検収合格・merge `b903eb73` — **朝の合否待ち**)** | ψ 主因1位根治 — slider tokens 化(2px トラック・方形つまみ・進捗強調なし= accent は playhead 専有)・counter ACCENT→text_primary+等幅。**実窓 before/after PNG で差分が transport 帯 31px 行に限定されることを画素証明**(evidence/tp-transport-transcription/)。mock へ transport 行追記(夜間導出案の注記つき)。tonmana fence 緑・shell 205 緑 |
| **H2: Timeline ツリー行(検収合格・merge `016b29e3`)** | 旧世界 timeline_rows 移植 — flatten-per-frame(depth/has_children/children_open)+fold state(既定全展開へ方向反転)+rail インデント/fold 三角。**red 捕捉1件: fold された子と孤立子の区別**(到達性と行組み立ての2巡分離で根治)。平坦 Document は前置き要素ゼロ=見た目完全不変(自己修正)。shell/src 無改修で配線が届いた(既存受け皿)。timeline 78+state 4+shell 193 緑。**朝の一瞥キュー: インデント 0.30×行高は宣言値**(モックに親子例なし) |
| **B2: Browser rail/filter(検収合格・merge `d9e90ea7`)** | RailScope(All/Media+予約地)+filter shelf(種別 chip・検索・Clear)+`visible()` 投影純関数。pane 単体 atlas 柵(rail/chip/検索欄が Target で見え click が publish する4本)。**map 848(Filters)採用済= 消化113/1,551**・980 は view 配線が B3 境界のため部分前進。shell は PaneState field+委譲腕のみ(PNG SHA 不変を守る正しい留保 — 全面配線は B3 で絵と一緒に)。browser 19+shell 205 緑(main 再実行 224 全緑) |
| **I-tokens: 4値束再転写(検収合格・merge `713aea04`)** | 二重モック構造の実装側根治 — row 20→**25**・value 38→**64**・glyph 18→**26**・panel 496(出典を v3.1 へ統一)。**文字寸比 0.55→0.44(帯内・モック実測と完全一致)= φ FINDING 根治**。裁定169 アンカーは実フォント自然幅測定で 6→**11字**へ再較正。pixel fence の隣接値誤マッチを EPS_EXACT で根治。inspector 50+shell 190+tokens 27 緑(main 再実行 282 全緑)。**朝の一瞥キュー: Inspector の行が高く・セルが広くなった見た目** |
| **TL-P1: rail widget 化(検収合格・merge `fc0e61d7`)— TL-arch Phase 1 完結** | `view()` = `row![rail, canvas]` — rail は実 widget(swatch=着色 container・名前= native `Ellipsis::End`・M/S/L= 実 button)。**atlas に M/S/L button と層名 Text が初出現**(canvas 不可視の盲点が晴れた)。canvas は時間場専用になり rail offset 算術を撤去。行 y 単一源(layer_row_top)の複製ゼロを grep 証明。φ の手製切詰3関数は #[deprecated] で退役。timeline 69+shell 190+inspector 49 緑(main 再実行 274 全緑) |
| **H-survey: 親子変換木(回収・merge `0cde024f`)→ 裁定173** | 大発掘3件: (1) 2026-08-20 起草 `group-layer-semantics-decision.md`(未採番)が precomp を parent/Group+fold/isolate に分解済み (2) **store に `LayerAttrs.parent` が循環ガード付きで実装済み・resolve だけが未読**(縫い目は view.rs:822 の1点・compositor 無改修で済む実測) (3) **旧世界に完成品**: spatial_resolve.rs(メモ化循環安全の world-affine)+timeline_rows.rs(fold 軸独立 flatten・テスト付き)が移植予約のまま眠っていた。単一再帰木仮説= アルゴリズム真/スキーマ偽(辺2種)。→ 裁定173 で schema 案(c)採択・H1〜H4 切片化(並走条件つき) |
| **M4: ゼロコピー presenter(検収合格・merge `61d5189e`)— 裁定166/170/171 campaign 完結** | `Compositor::render_to_texture`(readback なしの GPU 出力・既存4メソッド無改造)+`Engine::with_device`+shell の GPU 高速路(revision 不変・市松OFF・観測なし・cap=Auto の時のみ readback ゼロ、他は安全側フォールバック)。`frame_rgba()` は遅延 readback 化(export 真値は要求時のみ)。oracle: readback ゼロ緑化(#[ignore] 撤去)・screenshot 4条件 PNG SHA 完全一致・render_to_texture=render_with_timing のバイト一致証明・市松 CPU フォールバック生存・**workspace 106 スイート 910 全緑**(main 再実行)。副産物: metrics 共有 static の試験間汚染を発見し METRICS_LOCK 集約。**KNOWN 閉鎖: 市松 ON の実窓を利用者確認「OK」(2026-08-22)— campaign 完全クローズ**(WGSL unmultiply の理論導出が実窓で裏書き)。1レーン2停止→ALLOWLIST 2段拡張の capsule-gap 学習も記録 |
| **T-rail: rail 転写(検収合格・merge `85b3514d`+supervisor fix-forward)** | 裁定172 §2 — スウォッチ(0.308×行高・角丸0.25・**色=label_color 復帰** — 「発明しない」旧注記は ρ で失効)+M/S/L を 12px 塗りチップへ(0.462×行高・字0.667、塗りは surface_hover=既存ロール最近傍 Δ4)。比率4点全てモック一致・65/65 緑。**検収 red 1本**: shell 側テストがグリフ幾何の複製コピーを固定 → supervisor が `glyph_size_px` pub 化+正本導出へ書き換えて根治(273 全緑)。FINDING: screenshot 器具の手描き rail も乖離(T-canvas と同根 — M4 後の器具置換玉に合流) |
| **TL-probe: widget タイムライン性能実測(検収合格・merge `97382933`)** | r4 probe 新設 — **判定: Phase 2 CPU 側 GO**。1000 bar: パン=カメラ 24µs/frame(8.3ms 線の345倍余裕・layout 1回のみをカウンタで機械証明)・素朴再構築でも 99µs(84倍)・zoom x-only 140µs。UserInterface::draw は layout を呼ばない(scrollable と同じ with_translation 手口)。EVIDENCE_GAP: GPU/実窓側・ノード数はbar 3ノード構成のみ |
| **I-mock: Inspector 意味論モック(検収+利用者合格 — 転写元第5・6号)** | 旧世界 inspector-library.*(899+524行)を構造逐語・色ロール別読み替えで next/reference/mocks/ へ+semantics 版(救出台帳14行: effect stack/mode tabs/selection summary/context menus/extension tabs 等が解禁束)。冒頭で ui-scale-and-z を scale 資料へ降格宣言=二重モック構造の解消。FINDING: way_inspector/way_plugins 相当の token ロール不在(旧候補色のまま)。**v2/v3.1(利用者裁定3+1件)**: ○選択ボタン撤去(選択=行の地+アクセント線)・FXバッジ撤去・型名はリネーム時のみ・Fill スワッチは Recent と画素同文法(16×16 素の色面、UA 色井戸を剥がす)→ **利用者合格「確認できました」(2026-08-22)** |
| **T-canvas: canvas 転写(検収合格・merge `9e6077c3`)** | 裁定172 §1/§2 — 比率純関数5本(pub 化・screenshot 器具の式置換用に輸出)+bar 角丸は `Path::rounded_rectangle`(iced native・近似でない)。ruler 20→17・bar inset 2→3・角丸 0→2 等、全てモック比の最近傍。波及 grep 済み(hit/lane_bar/key_rows/input は引数経由で自動追随)。60/60 緑を main で再確認。**FINDING(重要): screenshot 器具は手描き再実装で実描画から完全独立** — 前後 PNG バイト一致で証明(比率変更が器具に一切写らない=器具が Timeline の絵の真実を語っていない)。M4 後の追い玉: 器具を pub 比率関数へ置換 |
| **I-ratio: Inspector 比率台帳(検収合格・merge `929980c6` — 転写ゼロが正しい返し)** | 裁定172 §3 — **0.55 の白黒: モック外と確定**(inspector-library 実測 0.44 は帯 0.42±0.05 の内・実装 0.55 はどちらとも不一致)。ただし根本原因は inspector-pane でなく **tokens の4値束**(row_height 20/value_width 38/glyph_width 18 = `ui-scale-and-z.html` 300px pane 前提 vs panel_width 496 = inspector-library 由来)の**二重モック構造** — 単独修正は新不整合を生むため見送りが正。**最重要発見: 実装の視覚正本は実は ui-scale-and-z.html**(コード自己申告+DOM 1:1 で確認)。regression lock 5本+台帳 `2026-08-22-inspector-ratio-ledger.md`。→ 新裁定待ち: **Inspector の転写正本の統一**(意味論モックの Inspector 版が無い — モックバックログ4件目・最大) |
| **TL-arch: canvas/widget 分割線調査(回収・merge `a390f408`)** | 保全= `2026-08-22-timeline-canvas-widget-survey.md`。**発見TOP3**: (1) **`iced::widget::pin` が絶対配置の一級部品として実在** — Phase 2 に自作 layout container 不要(前提転覆) (2) **canvas::Text の Ellipsis はフィールドだけあって描画側が無視**(geometry/text.rs が default 固定)— φ の手製切詰は canvas 選択の構造的帰結だったと確定。widget text なら cosmic-text 経由で正しく効く (3) gesture 純関数群(clip/key/nav)は既に実装非依存 — widget 化の gesture コストはゼロ。3分類台帳・pane_grid はパネル用でトリム流用不可の裏取り・端 drag 共有は n=1 で時期尚早。先例訂正: egui 版/NeoUtl も canvas 系(widget 先例ではない)。100 layer ≈ 800-1000 widget・Scrollable は仮想化なし(全子を毎フレーム layout、ソース確認)→ **Phase 2 は性能 probe 待ち・Phase 1(rail widget 化)は高確度で推奨**。T-rail/T-canvas と write-set 衝突のため実装はその着地後 |
| **M3: fork new_from_device(検収合格・merge済み・fork push `7cca401e`)** | 裁定170 §2 施工 — `DeviceCaps::from_device`(backend 分岐・doc 充実)+`RenderContext::new_from_device`(new() を new_impl へ畳む挙動不変リファクタ込み、+86/-5)。**adapter を明示 drop してからバイト一致**の常設 oracle。red(E0599)→緑・workspace 101/886 全緑×2回。supervisor: fork push・rev pin 全 entry bump・patch 節撤去を merge 前に実施(patch の中間状態を main に入れない判断)・seam 台帳へ記帳。**検収 FINDING: レーンは Motolii 側変更を未コミットのまま「commit 済み」と報告**(worktree status で発見・実害なし — 検収は status 確認から、の再確認事例) |
| **M01: iced 0.15-dev pin(検収合格・merge+supervisor 追い施工)** | 裁定170 M01 — iced/iced_test を fork `motolii/host-seams`(`73e686ee`)へ pin、**wgpu 系9パッケージが単一 29.0.4 へ統一**(lockfile 実測)。API 破壊は4種8箇所(Palette→Seed・Style.icon 削除・text_input 借用寿命・Widget::update clipboard 引数 — 全て機械的、レーンは柵どおり2回停止→ALLOWLIST 一括拡張で完走。**ω「差分ゼロ」の反証台帳として commit に保全**)。PNG oracle 全緑(font スタック更新は fixture 無風)・screenshot SHA `01f71f0820d4…` 不変。merge 後に supervisor 追い施工2件: τ presenter の wgpu29 追随3種(M01 分岐点が τ 以前だった段差)+φ が足した inspector の iced_test を fork へ統一(iced_core 2本混線の根治)。**101 スイート 886 全緑**・実窓 0.15 ビルドを supervisor 実画面確認(文字・寸法・Auto 1.00× 全て同等)。**利用者合否「見え方問題なし」= font スタック更新の審判合格(ω EVIDENCE_GAP-2 焼却、2026-08-22)** |
| **M2: Compositor::with_device(検収合格・merge `80707b4f`)** | 裁定170 M2 — `headless()` の device 構築後半を抽出した第二コンストラクタ(現 fork rev の `RenderContext::new` に素直な signature、adapter 落としは M3)。配線ゼロ・`headless()` はバイト一致素通し(既存 golden 全緑が証明)。red→緑(E0599)+同一フレームのバイト一致 oracle。compositor 27 全緑を main checkout 再実行 |
| **ω: iced 0.15/wgpu29 移行調査(回収・merge `b139fe45`)** | 保全= `2026-08-22-iced-015-wgpu29-migration-survey.md`。**fork は upstream master と sha 同一(ドリフト0)・host-seams=+seam2本**。wgpu 統一を metadata 実験で実証(14パッケージ→単一29.0.4)。API 差分は使用面338箇所で実質ゼロ(iced_test も再export で無傷)。唯一のブロッカー= adapter gap → **裁定170 で案B強化版に決着**(wgpu29 Device が adapter_info/features/limits を公開する supervisor 実測が決め手)。EVIDENCE_GAP 7件中、#1 は裁定170 で焼却・#2(font スタックの見え方)は M01 着地時の利用者の目へ |
| **φ: 文字の不衝突+文字余白(検収合格・merge `ca63dd1d`+supervisor 追い施工=裁定169)** | 裁定168 適用第1号 — (A) rail 名前の省略記号切詰(純関数 `truncate_to_width`+決定論近似測定器、実窓で衝突消滅を確認) (B) Inspector 値セル: 0.6em 横余白+0.075×行高 gap+**clip(true) が真の根治点**(text の paint が layout 幅を無視して隣へ滲む iced 実測をソース読みで特定)。検収の実画面検分で**次の欠陥を発見**: 38px セルに 7字「960.000」が入らず桁欠け → **裁定169**(表示はセル適合精度 `display_number`・編集 draft は全精度)を supervisor 直施工、「960.00/540.00」全桁可読を実画面確認。FINDING: body_text/行高=0.55 は裁定168 の 0.42±0.05 帯の外(未変更・裁定待ち)。timeline-pane 56+inspector 43+workspace 100 スイート 883 全緑 |
| **ψ: Timeline 転写ギャップ台帳(回収・merge `e7ae27ba`)** | 保全= `2026-08-21-timeline-transcription-gap-survey.md`。25要素: **適合9/逸脱8/余剰2/欠落0/EVIDENCE_GAP6** — 欠落ゼロ=チグハグの実体は「機能の不在」でなく「別世代正典の混在」と確定(利用者知覚の白)。主因TOP3(画素根拠つき): (1) transport slider が無style の iced 既定(金玉つまみ — token 非経由の唯一の生 widget) (2) rail 文字衝突(φ 施工中) (3) M/S/L glyph が Inspector 側 mock からの借用で行比 0.90(mock 0.46 の約2倍)+スウォッチ色が bar 色と無関係の単色。転写束割り案 3 レーン(rail/canvas/transport)。EVIDENCE_GAP 6件は裁定待ちに積む。**唯一の転写完結ファイル= projection.rs(0.52 比率)** |
| **τ: Stage presenter shader 化(検収合格・merge `1f2660a5`・利用者合否=合格)** | **裁定166 施工完了** — `StagePresenterProgram/Primitive/Pipeline`(永続 `wgpu::Texture`+世代ゲート `write_texture`)で image widget 置換・letterbox は stage-pane の既存 pub 関数再利用(2箇所目なし)・1.5MB 柵と auto sqrt 縮小を撤去し **Auto=1.00×(フル解像度)を実窓で supervisor 実画面確認**(旧 0.43×)。tick は `window::frames()`(vsync)へ。oracle 全通過を main checkout で再実行: handle_creations 10tick で 0・PNG SHA `01f71f0820d4…` 前後一致・shell 204+workspace 99 スイート 872 全緑。施工中の自己検出バグ1件(presenter_generation の 0 上書き — red で捕捉・修正済み)。KNOWN 追記2件(frames() occluded・GPU 実描画は headless 不可視)。**イージング滑らかさの最終審判=利用者の実窓**(裁定166 §4) |
| **υ: S 器具第一波(検収合格・merge `c86f9101`)** | 正典「器具」節 1-2 が着地 — `collect_targets` を `target_walk.rs` へ逐語抽出(q0_fence 判定無改変・柵は緑のまま)+`entrance_atlas_dump.rs`(通常 test= 既知 widget 存在+bounds 窓内、dev dump= `MOTOLII_ATLAS_OUT` ゲート)+`scripts/s-score.py`(S0 適合33行・S1 Fitts ランキング・S2 KLM 秒数、21 unittest)。red→緑証拠あり(walker 空実装で red 実証)。**S0 が実不一致を既に検出**(Undo/Redo: normal-map 3:3 タイの辞書式 tie-break vs κ の人裁定 — 上書きせず review 行き)。FINDING: (1) κ 台帳の「同/同上」参照セルは機械照合不能(次の κ 改訂で展開) (2) 書き出しは入口台帳に行ゼロ=ギャップとして顕在化 (3) S1 の距離起点は今波は窓中心固定(工程連鎖は次波) (4) Fitts 幾何は header ボタン5操作のみ(canvas 内は walker に構造的に不可視 — q0_fence 既知の限界と同根)。検収: main checkout で shell 191+python 21+workspace 99 スイート 866 全緑 |
| **ε: SP1 第一波 Speed 数値欄(検収合格・merge済み)** | Inspector ATTRS に Speed 行(%表示・click→type・Reset)。`retimed_duration` 純関数(i128 整数丸め・最低1フレーム・start/source_in 不変)は第二波と共有。1 gesture=1 undo・ロック M13 拒否・100% reset は no-op。map 963(Time Stretch)+269(Reset speed)採用済(**消化 112/1,551**)。pixel fence 7→8 行は実描画差分への正しい追随として採択。FINDING: Speed だけ Shell 側で Intent 組み立て(ALLOWLIST 制約の非対称 — 次の inspector 発注で統一余地) |

## 完了・保全済み(調査 — docs/reviews/2026-08-21-timeline-grammar-surveys/)

| 番号 | 対象 | 結果 |
|---|---|---|
| R1 | egui 版抽出 | 操作30件+文法定数+既決照合5点(正典の背骨) |
| R2 | Ravel | RESEARCH_RETURN(リポ未特定)。裁定145で全面除外 |
| R3 | 商用公式 | AE 修飾キー表ほぼ完全 |
| R4 | NeoUtl | 操作20種+AviUtl ExEdit 慣習 |
| R5 | Lottie 圏・現代7エディタ | AE から捨てられた要素の共通パターン8点 |
| R6 | AE ショートカット逆算監査 | 既載21/**抜け21**/対象外13/保留2。新概念型の抜けはレイヤーマーカーのみ |
| R7 | Godot/Blender | Godot 14項目+Blender 15項目。モーダル対比・スナップ多層・RMB キャンセル |
| R8 | Unity/Unreal/Spine | 37項目(既載13/抜け6/対象外12/保留6)。blend は理由つき対象外 |
| **統合パス** | **完了** | R6+R7+R8 の抜け39件を正典 §8 処置台帳へ(正13アクション採用・候8・保留8・対象外実証5) |
| R9(後任) | backend 消費ギャップ縫い目(blend/mask/speed) | 保全= `2026-08-21-backend-gap-seam-survey.md`。**speed「型だけ」claim は誤りと確定**(映像消費済み・欠落は UI 書き口)。mask は engine が黙って無視(matte と非対称)・`motolii-vector` が未配線資産と発見。blend は逐次合成の枠(BL1)が中核判断。切片割り SP1-2/BL1-5/MK1-4・EVIDENCE_GAP 4件・FINDING 4件(mask_apply.wgsl は実は matte の移植元、等) |

## **pane crate 分割 完了(裁定160、2026-08-21夜)**

6 crate 抽出(tokens/shell-state/timeline/settings/inspector/stage)+assembler。検証: テスト union 維持・check.sh 全通過・**PNG SHA-256 は全切片を通して `ee217ba2…` 不変**・レーン単体時間 2.7〜7.4s(旧 30〜38s)。切片7残骸(dead code 87行)掃除済み。音声A1(PlaybackClock)も着地済み。

## campaign: UI 完成 =「普通」の地図(裁定154)— **地図フェーズ完了(裁定158)、消化フェーズへ**

残作業=採用予定1,195件(正本 next/reference/normal-map.tsv)。運転: freq 降順に重み均等の切片で発注し、着地ごとに verdict を採用済へ更新(Lottie 地図と同じ)。最優先42件(freq≥2)から。

## 待機キュー(順序)

1. **Timeline 第2波 第1束**: レーンバー実体+move/trim(正典+裁定147/148/151 が発注仕様。egui 階層構造メモあり)
2. **Timeline 第2波 続き**: キー行+菱形 → ループ/locator → ナビゲータ・メニュー・keymap 層
3. **engine の effect 消費+内蔵 vism 第1号(推奨: Glow — M5 proof 済みで移植元がある)**: store の effect stack(実装済み)を engine が読んで GPU pass を回す消費側。~~codex 枠~~ → **Codex 解約(2026-08-21)につき通常の Claude レーンで**(裁定152)

## 利用者の目待ち(貯め — 裁定151 で操作意味論は自走確定に変更。残るのは見た目の実窓合否のみ)

- 実窓での線化(第1弾+第2弾の明暗リズム強度)+値セル余白+市松の手触り+glow の halo
- multi-select 行ハイライト(`830bce23`): Cmd/Shift 複数選択で全行が同色ハイライト・focus だけ property 行展開、の見え方
- **A2 実機確認4点**(`f026e7cc` — 実デバイス経路はテスト不能のため実窓が最初の審判): (1) Play/Pause ボタン・Space で実際に音が鳴るか (2) 再生中 scrub で音が新位置へ追従するか(リング容量分 ~85ms だけ古い音が残る設計) (3) soundtrack 無し Document で無音 Play+playhead 追随が見えるか (4) 終端到達でボタンが Play 側へ自動で戻るか
- リタイムが**複数プロパティ選択を一括で比例スケール**する解釈(T4 逸脱4 — 文法は範囲を限定していないため広く取った。実機で違和感があれば property 単位へ絞る)
- ~~正典の候~~ → 裁定151 で人口多数決により一括確定済み(§8.2)

## map 精査の観点(起草中 — SS 台帳の返却後に発注書へ固める)

- レンズ8本: **L1** 意図 vs 機構(裁定174 の一般化)/ **L2** 補償機能(三層フィルタ: Motolii が解いた/委託技術が解く/どちらも解かない — SS 台帳が一次資料)/ **L3** 教義整合(意味を保つ合成 — 意味破壊系は書き出しの顔でだけ)/ **L4** 重複意図の縮約(freq 水増しの補正 — 消化キュー順序に波及)/ **L5** コアか継ぎ目か(拡張の哲学)/ **L6** 保守コスト vs 頻度 / **L7** 時機(出力は「今/後/継ぎ目/不採用」の4値 — 利用者提起)/ **L8** 基盤/顔の分解(基盤コストは一度だけ数え顔は個別判定 — 第1例: パス編集基盤=コア(マスクで既払い)・マスクの顔=コア・ペイントの顔=継ぎ目候補、利用者提起) / **L9** 島度=合成参加度の欠如(利用者提起 — 「人口の無い意図」の測定可能な代理指標。入力辺・出力辺・基盤共有辺の3種を数え、辺が少ないほど島=周縁。型見本: ペイント(出力=どこにも流れないラスタ画素)vs パス(マスク・変形・キーへ流れる=辺が太い)。教義「意味を保つ合成」の測定可能な影。傍証: AM にもペイントはあったが皆パスを使った(利用者実観察)。freq は実装数であって人口ではない、の補正)
- 運転: 精査レーンが採用予定1,195行を分類 → 不採用候補(レンズ札)/縮約候補(意図動詞への写像)/争点束の3出力。**行は消さない**(S0 期待分布と歴史証拠)。利用者が見るのは争点束のみ

## 次の議題

- **端 drag プリミティブ(利用者提起 2026-08-22・同日縮小訂正)**: トリムは一般形の特殊化 — だが**パネルリサイズは iced `pane_grid` が native で持っていた**(pin rev 実測: `on_resize(leeway, f)`+`on_drag` 並べ替えまで)。よって自前で括る範囲はトリム×boxcam ハンドルの2実例+小物(ヒット幅・カーソル予告)に縮小。**shell レイアウトの pane_grid 化**(リサイズ+並べ替えを貰う)が新たな有力玉 — TL-arch 調査へ評価を注入済み。pane_grid gesture は分割木モデル縛りでトリムへの流用は不可(逆方向)
- ~~ゼロコピー presenter spike~~ → 裁定170/171 で campaign 化済み(M01〜M4)。M4 走行中

- ~~カメラレイヤー~~ → **利用者委任(2026-08-21、裁定156)**: 観測/レンダリング分離は既知実装で意味を組める — 縫い目調査レーンから通常回収へ
- **ギズモの拡張**: 現行既決=スクラッチ(2026-08-20。plan B= transform-gizmo+自前カメラ行列)— カメラ回収の後段が自然
