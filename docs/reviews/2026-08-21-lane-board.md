# レーンボード — 2026-08-21(このセッションの走行状態の正本)

日付: 2026-08-21 / 状態: **運転中**(セッション終了時に最終版へ更新して引き継ぎに畳む)
運転手順: レーンの完了・発注のたびにこの表を更新する。TaskList はセッション死で消えるため、この文書が正本。

## 走行中(未返却)

| レーン | 種別 | 場所 | 中身 |
|---|---|---|---|
| T2: clip move/trim | 実装(cargo) | lane-shell | 正典 §2 — transient+SetTiming 1発・スナップ7px/Cmd トグル・Esc/RMB キャンセル |
| T3: キー行+菱形 | 実装(cargo) | lane-shell2(新設) | params 行キー持ちのみ既定・菱形 8×8/hit 12×12・選択3種(key_rows.rs 自己完結) |

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
| (supervisor 直) 色 token 追随 fix / 市松レーン回収 / 引き継ぎ123コミット着地 | main 前提の整地 |

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

## 走行中 campaign: UI 完成 =「普通」の地図(裁定154)

型 = Lottie 地図の再現: 語彙源4種(メニュー木/ショートカット/パネル/環境設定)を4製品から機械列挙 → supervisor が同義束ね+頻度=普通度で単一 tsv 台帳へ → 判定3値(採用済/採用予定/不採用・理由)→ **未判定0 が完成条件**。「採用予定」の列が UI 完成の残作業リスト=次の発注キューになる。

## 待機キュー(順序)

1. **Timeline 第2波 第1束**: レーンバー実体+move/trim(正典+裁定147/148/151 が発注仕様。egui 階層構造メモあり)
2. **Timeline 第2波 続き**: キー行+菱形 → ループ/locator → ナビゲータ・メニュー・keymap 層
3. **engine の effect 消費+内蔵 vism 第1号(推奨: Glow — M5 proof 済みで移植元がある)**: store の effect stack(実装済み)を engine が読んで GPU pass を回す消費側。~~codex 枠~~ → **Codex 解約(2026-08-21)につき通常の Claude レーンで**(裁定152)

## 利用者の目待ち(貯め — 裁定151 で操作意味論は自走確定に変更。残るのは見た目の実窓合否のみ)

- 実窓での線化(第1弾+第2弾の明暗リズム強度)+値セル余白+市松の手触り
- ~~正典の候~~ → 裁定151 で人口多数決により一括確定済み(§8.2)

## 次の議題

- ~~カメラレイヤー~~ → **利用者委任(2026-08-21、裁定156)**: 観測/レンダリング分離は既知実装で意味を組める — 縫い目調査レーンから通常回収へ
- **ギズモの拡張**: 現行既決=スクラッチ(2026-08-20。plan B= transform-gizmo+自前カメラ行列)— カメラ回収の後段が自然
