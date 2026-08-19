# UI トンマナ統一 campaign — 目標更新と視覚デルタ(第5区切り)

日付: 2026-08-19
状態: **決定(利用者裁定の記録)+ 実測デルタ + campaign 計画**

## 利用者裁定(2026-08-19、原文要旨)

> 目標はアップデートし、Ableton のようなトンマナで、一目で情報を多く受け取れる UI にすること。
> 普通に使えるソフトにする、というのも忘れずに。
> 今の UI は正直まだまだトンマナがなく、野暮ったくてダサい。

これは実窓スクリーンショット(`/tmp/ui-final.png`、main=`b0c6e36a` 時点)を見た上での
**実機裁定**である([ux-authority-order] 実機裁定 > 品質バー > 文法地図 > 旧仮説)。

## 位置づけ — 新しい視覚言語は発明しない

トンマナの正本は既に存在し、十分に厚い:

- [docs/ui-visual-language.md](../ui-visual-language.md) — 面・色・文字・icon の規約
  (フラット・細罫線・低彩度の地に意味色・token 外 raw color 禁止・余白でなく罫線と明度差)
- [docs/CANON.md](../CANON.md) — 面ごとの視覚正本
  (Browser/Inspector = `docs/mocks-ui/public/*-library.html`+`.css`、Timeline = egui 実装、
  token = `ui/motolii-tokens/sources/motolii-dark.json`)

**モック正本自体は既に Ableton 的トンマナを満たしている**(browser/inspector/timeline-reference.png
を本セッションで再確認)。野暮ったさの出所は正本の欠如ではなく、
**iced shell が正本を通らずに機能先行で建てられた部分**である。
したがって本 campaign は (a) 正本への適合、(b) 正本が無い chrome への正本新設、の2種だけを行う。

## 実測デルタ(絵の突き合わせ、`/tmp/ui-final.png` vs 正本)

| # | 出所 | 現状 | 目標(正本根拠) |
|---|---|---|---|
| D1 | **最下帯**(プロジェクト名 + Undo/Redo/Export) | ベージュ塗りの浮いたボタン。正本のどこにも無い即席意匠 | フラット暗面 + 1px 罫線のボタン(ghost)。Export だけ accent 縁。プロジェクト名は muted 文字。timeline モック最下帯の status 文法に揃える |
| D2 | **ショートカット legend 帯**(`Space=play / pause L=loop …` 全文常設) | フルテキスト1行が常時最大輝度で並ぶ。ノイズ | timeline モック最下帯の文法: 左=選択/文脈(`Selected Group: Title scene · 3 children`)、右=muted の短ヒント(`Wheel: pan time · …`)。**文脈連動**(hover/選択中の対象に応じたヒント)は ui-visual-language の「右下 status 領域」規約そのもの |
| D3 | **transport 帯**(`⏮ ▶ 0:00:00 2 rows view 0.00-16.00s grid 2`) | デバッグ調の内部状態文字列が利用者向け文字と同格で並ぶ | timeline モックの header 文法: 左=title+操作、右=Snap/Fit 系 + **amber の tabular timecode**。`2 rows`/`view 0.00-16.00s`/`grid 2` は muted 化 or 撤去(内部状態は status 帯へ) |
| D4 | **クリップ/行の色**(サーモン・カーキ・緑) | 彩度も明度も揃っていない仮 hue。[meaning-freeze-is-not-design-freeze] の「仮 hue を継承しない」違反状態 | ui-visual-language「安定 Object ID → theme palette slot の決定的導出」。モックの低彩度スレート/セージ/ローズ系(browser-reference の thumbnail 群、timeline-reference の帯)と同族の palette を token 化 |
| D5 | **Browser カード** | 詰まって縦潰れ。thumbnail の letterbox 無し。タブ(Media/Effects/Create/Panels)相当の骨格が痩せている | `browser-library.css` の計算値(器具 `motolii-css-metrics` で機械抽出)に追随。16:9 contain + neutral letterbox は正本規約 |
| D6 | **Inspector 下端切れ**(Opacity 行) | スクロール無しで切れる | scroll 導入。既知残差(第4区切り引き継ぎに記録済み) |
| D7 | **panel header 文法の不統一** | Inspector=accent 縦棒+title、Browser=title のみ、Timeline=transport が header を兼ねる | 3面とも同一文法: accent marker + title + 右端 muted 文脈 label(モック3枚が全て同型) |
| D8 | **Stage 余白** | オリーブ褐色のグラデ様の面 | neutral dark の letterbox(ui-visual-language: 装飾 gradient 禁止・余白は neutral) |

## campaign 計画(レーン分割)

原則: [structure-over-supervision] — oracle・柵・UX 合否の3本以外は型で縛る。
発注は capsule(CONTEXT/TARGET/ALLOWLIST/正負 oracle/RETURN、「決める許可」を書かない)。
全レーン `model: sonnet`、worktree 隔離、`CARGO_TARGET_DIR=/private/tmp/motolii-lane-target`、
同時 cargo 2本まで。**視覚受入条件(--screenshot 120+ frames の絵)と器具照合を必ず入れる**
([capsule-gaps-are-the-defect-source])。

| レーン | 中身 | 受入 |
|---|---|---|
| A(先行・機能) | **拒否理由表示のテスト固定**。発注前調査(2026-08-19)で、引き継ぎの「`take_rejections` 呼び出し0件」は救出 merge `ab4ff63e` で**大半が失効済み**と判明: intent 経路(`intent.rs:786,809`)が transcript へ流し、iced の `status_band`(`view.rs:230`)が描画、ロック中ドラッグは理由送出+`NotAllowed` カーソルまで実装済み。残る穴は「絵に出ることを固定するテストが1本も無い」こと。レーンは全拒否パスの踏破テスト+無報告パスの修復に縮小 | 落ちるテスト先行 + ロック中 clip ドラッグで理由が絵に出ることを simulator で assert |
| B | shell chrome 統一(D1+D2+D3+D7): status 帯の新設、legend 撤去、transport 整理、header 文法統一 | 絵 + 新 status 帯のテスト。**A と B は status 帯で合流する**ため、A が先に status 帯の口を定義する |
| C | Browser/Inspector 忠実度(D5+D6): css-metrics 器具の計算値へ追随 | oracle(器具値 pin)+ 絵 |
| D | Timeline/Stage 色(D4+D8): palette slot 決定的導出の token 化と適用 | 絵 + slot 導出の決定性テスト(同一 Document 再起動で同色) |

**token の扱い**: D4 の palette slot は `motolii-dark.json`(DTCG 正本)へ追加し生成経路を通す。
コード内 raw literal を増やさない(ui-visual-language「token の格納形式」)。

## 非目標

- 視覚言語正本・モック正本の改訂(適合が先。正本自体への不満が絵で出たら別粒で裁定)
- ライトテーマの同時実装(同格提供の方針は不変だが、本 campaign は dark の適合まで)
- G0-6 の全審判の前倒し(contrast 実測などは campaign 後の区切りで)

## 追記 — 第2波: 密度と配置の利用者裁定(2026-08-19 午後)

第1波(A〜D)着地の絵を見た利用者の追加裁定:

1. **「タイムラインから下がかなりどデカい」— skia や egui を参考に圧縮する**。実測比較:
   iced は行より上に transport 30 + overview 22 + ruler 36 = **88px** 積んでいる
   (egui 正本は 84px だが HEAD_H 34 に操作を同居、skia 死蔵は 22+18=**40px**)。
   行高は iced/egui/css とも 24(skia 最小 20、2026-08-08 決定「行高は固定・最小20px」の最小側)。
   rail は iced 234 vs egui/css 196。M/S/L ボタンは iced 22×18 vs モック css `.ms` **16×16**
2. **Undo / Export は下帯でなくヘッダに出す**(RN chrome 正本の titlebar 構成とも一致)
3. **ログ(状況報告)は下に出す** — 下帯は「左=最新の報告 / 右=近道ヒント」の薄い status 帯だけにする

対応レーン: E(ヘッダ新設+下帯薄化)、F(Timeline 縦密度: D9 表の値へ)。
oracle の pin(RAIL_W / TRANSPORT_H / ROW_H)は**根拠ごと更新**する
(RAIL_W 196 は css と真の一致へ戻る。ROW_H 20 は css 24 との既知乖離として本裁定を根拠に pin)。

## 追記 — 第3波: 正本改定の裁定(2026-08-19 夕)

第2波(E: ヘッダ+下帯 / F: Timeline 縦密度)着地後の利用者裁定:

> ここから先は正本改定を土台とし、Ableton 系の余白を作らないフラット UI を調査する。

含意:
1. **モック正本(`docs/mocks-ui/public/*-library.html`+css)の改定が第3波以降の土台**。
   これまでの「正本適合のみ」から一段進み、正本そのものを Ableton 実測へ寄せる
2. 方向 = **余白を作らないフラット UI**: 分離は罫線(1px)と明度差で作り、padding/gap を
   詰める。ui-visual-language の既存規約「余白は装飾や高級感の手段にしない・領域の分離は
   罫線と明度差で行う」の徹底形
3. 文字クラスの一段下げ(BODY 13 → Ableton 主文字帯 11px 級)も正本改定に含めて設計する
4. 手順: 調査(Ableton フラット文法の画素実測+現行モック余白の器具実測)→ 正本改定の
   設計 → モック css 改定 → 器具(css-metrics)で再抽出 → iced 追随+oracle 更新

## 追記 — campaign のゴール裁定(2026-08-19 夜)

利用者裁定:

> **評価軸の定数化を行えるようになるまでをゴールとする**。それが行えれば随時確認しなくても
> 済む。Motolii も一面に出る情報量はできるだけ多く保つ(一目で構造が理解できない UI は
> いけない)。

含意と設計:
1. ゴール = トンマナ・密度の評価軸を**機械検査(トンマナ oracle 群)へ落とす**こと。
   絵の目視確認は「新しい意匠判断の時だけ」へ縮退させる
2. Ableton の「多情報なのに見やすい」の正体は**分散の小ささ**(文字1バンド・行格子・
   chrome無彩色でデータにだけ色・分離は1px罫線と微小明度差・部品文法の反復)。
   分散は測れる — 定数化の対象は: type scale 集合 / 行高の種類数 / surface ladder 段数と
   輝度差範囲 / border 幅と role 数 / spacing scale 集合と pane 間 gap=0 / raw hex 0件 fence /
   コントラスト 4.5:1・3:1 / 角丸0(識別記号のみ例外)
3. 検査の型は既存資産の延長: `css_metrics_oracle`(器具値 pin)+ `intent_gateway_fence`
   (ソース走査 fence)。新機構は発明しない
4. **限界の明記**: ゲシュタルト(一目で構造が分かるか)自体は定数化しない — Q0 型の
   人間審判(5秒テスト)に残す。定数化するのは「構造理解を壊す既知要因の混入検知」まで
5. 情報量は**減らさない**: 密度を保ったまま分散を殺す方向で正本を改定する

## 記録

- 起点: 第4区切り引き継ぎ [2026-08-19-session-handoff-timeline-port-and-instruments.md](2026-08-19-session-handoff-timeline-port-and-instruments.md)
- 本文書と同 commit で decision-index / reviews README へ行を追加する
