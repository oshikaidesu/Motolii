# Timeline 転写ギャップ台帳 — モック×実装の全要素差分(2026-08-21)

読むだけの実測。判断・裁定はしない(製品コード・モック・正典への変更は一切していない)。
起点: 利用者の実窓較正「Timeline のどの要素もチグハグに見える。一から UI を考えたようには
見えない」— レーン積み上げの各要素が別々の時代の正典を転写した結果ではないか、という仮説の
白黒つけ。

## 読んだもの

1. 視覚正本(転写元第1号): `next/reference/mocks/timeline-semantics.html`
2. `docs/ui-spatial-score.md`(S0〜S6、裁定163/164/167/168)
3. `docs/decision-index.md` 裁定165〜168 行
4. 実装: `next/ui/motolii-timeline-pane/src/{canvas,lane_bar,key_rows,projection,hit}.rs`、
   `next/ui/motolii-tokens-rs/src/lib.rs`(+`tokens/dimensions.json`)、
   `next/shell/motolii-shell/src/lib.rs` の `transport`/`status_band`、
   `next/ui/motolii-settings-pane/src/chrome.rs`(`button_style` 共有元)
5. 実窓実測画像: `motolii-window.png`(2184×1728、Retina 2x と推定)。以下の画素値は
   すべてこの画像からの実測(`python3 -c "from PIL import Image; ..."` で座標抽出)

分母表記は全て「対象値/分母」の形。比率原則(裁定165(1))・余白梯子(裁定167、分母=行高)・
文字余白 em 族(裁定168、分母=文字寸)に従う。

---

## 台帳

| # | 要素 | モックの宣言(比率・余白段・文字・token) | 実装の実測(定数 file:line+画像所見) | 判定 | 根拠裁定 |
|---|---|---|---|---|---|
| 1 | rail 幅 | `.grid{grid-template-columns:150px 1fr}` — 固定 150px(裁定147 出典は別 mock `ui-scale-and-z.html .thead`) | `Dimensions::timeline_lane_bar_width = 150.0`(`motolii-tokens-rs/src/lib.rs:100,124-126`、JSON同値)。`TimelinePane::rail_width()`(`lib.rs:203-205`)がそのまま返す | **適合** | 裁定147 |
| 2 | rail スウォッチ(色チップ)寸法 | `.rail`内 `<span style="width:8px;height:8px;border-radius:2px;background:hsl(...)">` — 8×8px、行高26に対し 8/26=0.308(裁定167 上段ラダーに合致) | `swatch_size = dims.spacing_m`(=8px、`lane_bar.rs:166`)。8×8で寸法は一致するが、比率の分母が違う: 8/row_height(20)=**0.40**(裁定167 上段0.30から外れる) | **逸脱**(寸法px一致は偶然 — spacing_m はrow_height比例で選ばれた値ではない) | 裁定167 |
| 3 | rail スウォッチ 色 | mock の色チップは **bar と同じ `hsl(i*30,32%,62%)`**(`timeline-semantics.html:114,119` — 同じ式を2箇所で呼ぶ、スウォッチ=bar色のプレビュー) | `colors.way_timeline`(Timeline全体アクセント、単色固定)を**全行で使い回す**(`lane_bar.rs:164-172`、コード注記:「Document に色ラベルが無いので発明しない」)。画像実測(`rail_zoom.png`): 全行のスウォッチが同一の鮭色(≈`way_timeline`)で、実際の bar 色(オリーブ・緑・水色・青・紫等、実窓で確認)と**無関係** | **逸脱**(S5c 二重発話禁止の逆——本来同じ情報〈層の色〉を2箇所〈スウォッチ・bar〉が主張すべきところ、スウォッチ側だけ主張を失っている) | S5c、裁定165(2) |
| 4 | rail スウォッチ 角丸 | `border-radius:2px`(2/8=0.25 of スウォッチ寸) | `canvas::Frame::fill_rectangle`(角丸引数なし、`lane_bar.rs:168-172`)= 直角 | **逸脱** | 裁定165(1)(形の転写) |
| 5 | rail 名前(文字) | `.rail{padding:0 8px}` 内にテキスト、隣接 `.g` チップへ**あふれない**前提(mock はテキストと chip の間に gap:4px を明示) | `lane_bar.rs:181-188` `frame.fill_text` — **切詰め・幅チェックなし**。画像実測(`rail_zoom.png`)で「メインボーカル映像M」「波形ビジュアライザM」「リリックモーション背景S」「グリッチトランジション...MS...」が M/S/L チップへ素通しで衝突(実窓実測 2026-08-21 深夜、裁定168 が既に記録・レーン φ 発注済みだが**未着手のままこの実窓にも再現**) | **逸脱**(裁定168 不衝突文法の絶対規則違反、既知・未修理) | 裁定168 |
| 6 | rail M/S/L glyph 寸法・様式 | `.rail .g{width:12px;height:12px;background:#4a4a4a;border-radius:2px;font-size:8px}` — **塗り潰しチップ**、12/26=0.462(行比) | `inspector_glyph_width`(=18px、**Inspector 側 mock `ui-scale-and-z.html --hit` からの流用** — Timeline 転写元とは別 mock)を rail の glyph 幅に使い回す(`lane_bar.rs:14-15,53`)。高さ= `row_height - spacing_xs`=18px。18/20=**0.90**(行比、mockの2倍近い)。様式も**枠線ボックス**(`canvas::Stroke`、`lane_bar.rs:205-211`)で mock の塗り潰しチップと違う | **逸脱**(2つの異なる mock 世代からの借用が同一要素に同居 — チグハグ仮説の直接証拠) | 裁定165(1)、S4(段借用の柵) |
| 7 | 行 ゼブラ(奇数行 wash) | `.row.z .field{background:rgba(255,255,255,.05)}`(field のみ、rail は受けない) | `colors.timeline_row_zebra = rgba(255,255,255,.05)`(`motolii-tokens-rs/src/lib.rs:380`、mock値そのまま)。`canvas.rs:85-102` は `rail_width` から描画開始(railを受けない)。画像実測: `y=982` 行で baseline 61→71(理論値 61+(255-61)*0.05≈71.7、一致)、`y=946` 行(偶数)は 61 のまま(zebra無し) | **適合**(画素実測で確認) | 裁定148(2) |
| 8 | 行 境界線(hairline) | `.row{border-bottom:1px solid rgba(0,0,0,.35)}` | `colors.border_hairline_weak = rgba(0,0,0,.35)`(`lib.rs:378`、mock値そのまま)。`draw_hairline`(`canvas.rs:171-178`)が全幅(rail込み)で描画 | **適合** | 裁定142 |
| 9 | ruler 高さ | `.ruler{height:22px}` — row(26)と**別値**、22/26=0.846 | `ruler_height() = dims.row_height`(=20px、`lib.rs:196-198`)。コード注記:「第1波は測定済みの行高をそのまま流用する(独自の寸法を発明しない)」— mock の別値を転写せず row と等値化 | **逸脱**(mock は ruler≠row を明示しているのに実装は同値。「流用」自体が別世代の判断) | 裁定165(1) |
| 10 | ruler 目盛り(小/大)の長さ比 | `.tick{height:5px}` / `.tick.major{height:11px}`(ruler高22に対し 0.227/0.5) | 小=`height-spacing_s`(4px、20分の0.20)/大=`height-spacing_m`(8px、20分の0.40)(`canvas.rs:247-251`)。近似だが厳密な比率規定の裁定は無い | **EVIDENCE_GAP**(tick 長そのものの比率原則は裁定に無い — 数値は近いが一致を主張できない) | ― |
| 11 | ruler 数字ラベル | mock JS(`timeline-semantics.html:88-96`)は目盛り div を打つだけで**ラベルを描かない**(意味層注記にも数字表示の言及なし) | `draw_ruler_ticks`(`canvas.rs:262-270`)は大目盛にフレーム番号を `fill_text` で描く。コード注記:「NON-GOALS『ルーラーへの timecode 文字』は新設しないという意味なので、既存のプレーンなフレーム番号表示はそのまま残す」— **旧世代の意匠を意図的に保持** | **余剰**(モックに無い要素。ただしコード自身が「新規追加ではなく旧踏襲」と明記 — 無自覚な混入ではないが、転写元(裁定165(2))を正本とする原則からは逸脱) | 裁定165(2) |
| 12 | セル比率(小目盛間隔/行高) | 合格モック実測 13.5/26 ≈ **0.52**(裁定165(1) の直接出典) | `TARGET_CELL_RATIO = 0.52`(`projection.rs:112`)。`tick_steps` がラダーの中から比率最近傍を選ぶ実装で、独立算出テスト付き(`projection.rs:250-283`) | **適合**(比率原則の模範実装 — このファイルだけは転写が完結している) | 裁定165(1) |
| 13 | 時間場 縦線(全目盛) — 周波数分担 | `bands()`第2ループ: 小目盛ごとに1本、色 `rgba(0,0,0, major?0.30:0.18)` | `colors.timeline_grid_minor = rgba(0,0,0,.18)` / `timeline_grid_major = rgba(0,0,0,.30)`(mock値そのまま、`motolii-tokens-rs/src/lib.rs:401-403`)。画像実測: 帯「オン」区間(baseline 61)で線 50(61×0.82≈50、α0.18と整合)、周期は約14px(minor tick 間隔と一致) | **適合**(画素実測で確認 — 裁定165(4)「帯=面/線=細」の対立疑いは**白**、両立実装できている) | 裁定165(4) |
| 14 | 時間帯(大目盛周期の帯) | `bands()`第1ループ: 大目盛2周期ごとに `rgba(255,255,255,.035)` の帯 | `colors.timeline_time_band = rgba(255,255,255,.035)`(mock値そのまま)。画像実測: x=600→674で baseline 61→54へ切替(理論値: panel地54に対し白wash.035適用で54+(255-54)*0.035≈61、一致) | **適合**(画素実測で確認) | 裁定148(1) |
| 15 | bar 縦 inset | `.bar{top:4px;height:18px}` in row26 → inset=4/26=**0.154**(裁定167 中段ラダー) | `row_top+spacing_xs`(=2px)始点、高さ`row_height-spacing_s`(=16px)。結果 top/bottom inset とも2px。2/20=**0.10**(裁定167 中段0.15からもラダー下段0.075からも外れる中間値 — 裁定167 が明示的に禁じる「段の中間値」) | **逸脱** | 裁定167(段の中間値禁止) |
| 16 | bar 角丸 | `.bar{border-radius:2px}` | `canvas::Frame::fill_rectangle`(角丸引数なし、`canvas.rs:154-161`)= 直角 | **逸脱** | 裁定165(1) |
| 17 | bar 差し色・状態優先順位 | 意味層注記:「差し色=label_color index(AE同型)。dragging=ACCENT・hidden=mutedが優先」 | `canvas.rs:142-153` が dragging>hidden>label_color>way_timelineの順で完全一致実装(コード注記でR1実測踏襲を明記) | **適合** | S 注記(意味層) |
| 18 | playhead | `.play{width:1.5px;background:action-active}` + 注記「S5b: pane内の最大コントラストはヒーロー内にのみ」 | `hairline*1.5`・`colors.action_active`(`canvas.rs:190-199`) | **適合** | S5b、S 注記 |
| 19 | キー菱形 寸法 | `.dia{width:8px;height:8px}rotate(45deg)` | `KEY_DIAMOND_SIZE=8.0`描画・`KEY_HIT=12.0`当たり(`key_rows.rs:37-38`) | **適合**(寸法) | 裁定151 |
| 20 | キー菱形 色 | mock は全キー同色 `#c8c8c8`(選択状態の描き分け無し・簡易サンプル) | 選択=`action_active`・非選択=`way_timeline`(`key_rows.rs:131-136`)。mockはこの区別自体を持たないため直接比較不可 | **EVIDENCE_GAP**(mock に選択状態の色仕様が無い — 実装側の判断そのものは他S注記〈dragging=ACCENT等〉と整合的だが、この意味層モックからは検証不能) | ― |
| 21 | property 行(キー行)高さ | mock 対応なし(timeline-semantics.html に property 行の別高さ指定なし、mockの`.keys{height:18px}`はクリップ内キー行の**別実装**でproperty行そのものとは形が違う) | `timeline_param_row_height=16.67`(egui版 `ROW_H=24/PROP_H=20` 比を row_height(20) へ適用、`dimensions.json:58-59`) — **Timeline mock 系列ではなく egui 版正典から借用**(裁定150 型の踏襲) | **EVIDENCE_GAP**(転写元 mock に対応値が無いため合否判定不能。ただし借用元が第3の canon〈egui〉である点は裁定165(2)の「モック=転写元」原則の対象外領域として記録) | 裁定150、裁定165(2) |
| 22 | マーカー | 意味層モックに描画例なし(S0凡例にも言及なし) | `pane.markers`をruler帯へ`way_timeline`色・hairline×2で描画(`canvas.rs:64-76`) | **EVIDENCE_GAP**(視覚正本が無いため合否判定不能) | ― |
| 23 | transport(Play/frame/slider) | mock に対応要素なし。`docs/ui-spatial-score.md` S5 表は「header/transport = pane ではなく世界の縁、重み低の物の家」と**別身分を明記** | `transport()`(`shell/lib.rs:2468-2503`)。Play ボタンは `chrome::button_style`(角丸0、`chrome.rs:53-63`)で他 header ボタンと同一意匠(適合的に**低重み扱いされていない** — Undo/Redoと同じ強さの枠線ボタン)。**slider は `iced::widget::slider` を無 `.style()` のまま使用**(`shell/lib.rs:2495-2497`)— iced既定の太いトラック+丸い大玉つまみ(画像実測 `transport_zoom.png` で確認)。Timeline全体を支配する「flat・hairline・角丸ゼロ」の意匠語彙から完全に外れた**別世代(iced ネイティブ)の部品**がそのまま露出 | **余剰+逸脱の複合**(mockに無い要素=身分としては予期される余剰だが、その中身がtoken非経由のネイティブ意匠という別の逸脱を抱える) | S5、裁定142(raw値直書き禁止の精神) |
| 24 | transport frame カウンタ 色 | mock 該当なし | `text(...).color(colors.action_active)`(`shell/lib.rs:2492-2494`)— playheadと同じACCENT色をpaneの外(世界の縁)で使用 | **EVIDENCE_GAP**(S5bは「pane内の最大コントラスト」の規定でtransportはpaneでないため文言上は対象外。ただし精神的には要 裁定) | S5b(適用範囲要裁定) |
| 25 | status 帯 | Timeline mock に対応要素なし(S5表で Stage にのみ「下縁=状態帯」と明記、Timelineの欄には無い) | `status_band()`(`shell/lib.rs:3087-3115`)は Shell 全体共通の帯、caption_text+text_muted(低weight)で実装 — S4 の低重み文法自体には適合 | **EVIDENCE_GAP**(Timeline固有の要素ではなくShell全体chrome。判定対象外として記録) | ― |

---

## チグハグ知覚の主因 TOP3(画像根拠つき)

1. **transport slider が別世代の意匠語彙(#23)** — Timeline全体は canvas 手描きの hairline・直角・低彩度アクセントで統一されているのに、`Play`ボタンのすぐ右の scrub slider だけ iced 既定の太いトラック+丸い金色の大玉つまみになっている(`transport_zoom.png` で直接確認)。token を一切経由しない生の iced widget デフォルトが、フラット設計言語のド真ん中に置かれている — 「一から考えたように見えない」の最も分かりやすい物的証拠。
2. **rail 名前 × M/S/L の文字衝突(#5)** — `rail_zoom.png` で「メインボーカル映像M」「波形ビジュアライザM」「リリックモーション背景S」「グリッチトランジション」の4行が実際に文字とチップが重なって読めている。裁定168 は2026-08-21深夜時点で既に検出・レーンφ発注済みの**既知**違反だが、この実窓実測でも未修理のまま再現しており、「詰め切られていない」という第一印象に直結している。
3. **rail M/S/L の二重借用(#6)とスウォッチ色の断絶(#3)** — glyph 幅は Inspector 側の別 mock(`ui-scale-and-z.html`)から、セル比率は Timeline 側の新 mock(`timeline-semantics.html`)から、という**2つの異なる世代の正典が同じ行の中に同居**している(glyphは行高の0.90を占め、mockの0.46の約2倍で妙に大きい)。さらにスウォッチ色が bar の実際の色と無関係に単色固定なので、同じ行の中で「色の主張」が2箇所に分裂している(S5c 二重発話禁止の逆側の違反 — 本来揃えるべき情報が割れている)。この2つが合わさって、rail 列だけ密度・色・寸法の基準がクリップ面と噛み合っていない印象を作っている。

## EVIDENCE_GAP 一覧(裁定不在、勝手に裁定しない)

- #10 ルーラー目盛り長さの比率規定(裁定なし、数値は近似)
- #20 キー菱形の選択色規定(mockが状態分岐を持たない)
- #21 property行(キー行)高さの出典 mock 不在(egui版からの借用、裁定165(2)対象外領域)
- #22 マーカーの視覚正本不在
- #24 transport frame カウンタの ACCENT 色使用可否(S5bの適用範囲=pane限定と読めるが、transportのような「世界の縁」への拡大解釈は未裁定)
- #25 status帯のTimeline側の身分そのもの(Shell全体chromeか、Timeline縁の一部か — S5表に記載なし)

## 判定集計

- 適合: 9件(#1,7,8,12,13,14,17,18,19)
- 逸脱: 8件(#2,4,5,6,9,15,16,23)
- 余剰: 2件(#11,23〈複合〉)
- 欠落: 0件(mock にあり実装に完全に無い要素は今回の網羅範囲内では検出せず — チグハグの原因は「無い」ことではなく「別世代の値・様式が混在している」こと)
- EVIDENCE_GAP: 6件(#10,20,21,22,24,25)

(#23 は「余剰」と「逸脱」の複合のため両方の集計に一度ずつ計上。合計は表の行数と厳密に1:1ではない)

## 転写レーン束の割り案(重み均等・write-set 互いに素)

| レーン | 対象ファイル(write-set) | 含む項目 | 行数目安 | 判断の重さ | 領域数 |
|---|---|---|---|---|---|
| α: rail 転写 | `next/ui/motolii-timeline-pane/src/lane_bar.rs` のみ | #2,3,4,5,6(スウォッチ色源・radius・glyph寸法/様式・名前切詰め) | 223行(全体) | 中(意匠選択が複数点: glyphを塗り潰しチップへ変えるか・スウォッチ色をbar色に連動させるか) | 1(rail) |
| β: bar/ruler 形状転写 | `next/ui/motolii-timeline-pane/src/canvas.rs`(+ 必要なら `motolii-tokens-rs` へ ruler 専用高さトークン新設) | #9,15,16(ruler高さ独立化・bar inset梯子適用・bar角丸) | 376行(全体、変更点は局所) | 中(ruler高さは新トークン新設の裁定が要る — 裁定167ラダーへ厳密適合させるか現状維持かの判断) | 1(canvas)〜2(トークン新設時) |
| γ: transport 意匠整合 | `next/shell/motolii-shell/src/lib.rs`(`transport`関数)+ 必要なら `motolii-settings-pane/src/chrome.rs`(slider共通style新設) | #23,24(sliderのtoken化・frameカウンタ色の裁定) | 局所(~40行)+chrome.rs追加 | 中〜高(iced slider の custom style実装+S5b適用範囲の裁定待ち) | 1〜2 |

3レーンの write-set は完全に素(lane_bar.rs / canvas.rs / shell::lib.rs+chrome.rs)。#10,20,21,22,25 の EVIDENCE_GAP は裁定文書側の追記のみで済むため、コードレーンには含めない(裁定が出てから該当レーンへ差し込む)。

## RETURN 用メモ

- push はしていない。commit のみ worktree 内。
- 製品コード・モック・正典ファイルへの変更は一切なし(このファイルの新規作成のみ)。
