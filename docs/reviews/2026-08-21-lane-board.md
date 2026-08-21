# レーンボード — 2026-08-21(このセッションの走行状態の正本)

日付: 2026-08-21 / 状態: **終了時点の最終版**(引き継ぎ= 2026-08-21-session-handoff-ui-completion.md)
運転手順: レーンの完了・発注のたびにこの表を更新する。TaskList はセッション死で消えるため、この文書が正本。

## 走行中(未返却)

| レーン | 種別 | 場所 | 中身 |
|---|---|---|---|
| ρ: レイヤー差し色 第一波 | 実装(cargo・sonnet) | lane-shell | 利用者裁定「色が足りない」— `label_color: Option<u8>`(**index 保存**= AE 同型)+生成時 `id%12` 決定論割当+Timeline bar 塗り+**パレット候補 A/B/C(意図設計・同一 fixture)比較 PNG**。変更 UI は後続波 |

## 完了・main 着地済み(実装)

| レーン | 結果 |
|---|---|
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

## 次の議題

- ~~カメラレイヤー~~ → **利用者委任(2026-08-21、裁定156)**: 観測/レンダリング分離は既知実装で意味を組める — 縫い目調査レーンから通常回収へ
- **ギズモの拡張**: 現行既決=スクラッチ(2026-08-20。plan B= transform-gizmo+自前カメラ行列)— カメラ回収の後段が自然
