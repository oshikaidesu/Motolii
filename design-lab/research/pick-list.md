# 選び表(pick-list)

2026-10-02。Widgetbook の 48 部品(批評 3 巡目で 41 が RELEASE、残り 7 は本巡で直りを実機確認して RELEASE)の一覧です。

## 1. これは何で、どう使うか

- 目的: あなたが「これを採る」「ここは違う」と決めるための一枚。部品ごとに、手癖との関係・契約の 1 行・触る場所・画面・残った未決を並べた。
- 使い方: lab_book(Widgetbook)を開き、左の Component 名を開く。最初の `Parts @200%` は部品の見本帳、2 つ目からが実際の panel の use case。右の Knobs(Panel width / Items / Trigger 等)を動かすと状態が変わる。panel の中は実際に触れる(hover・click・キー)。画像は `shots/` の相対 path。
- 手癖の分類は experience-first.md の台帳 C に従う: 手癖のまま(既定にする)/ 手癖の上の追加(求めた時だけ出る)/ 手癖から外れる(外す手癖と得る物が明記されている案)。
- 判定は 3 巡目の独立批評 + 本巡の再撮影(2026-10-02)。FIXED は本巡で実機確認済み。
- OPEN = 色相(accent / tab / thumbnail / stage 軸の青)などあなた専権の未決。欠陥ではない。

## 2. 部品表(48 行)

| ID と名前 | 手癖の分類 | 契約(1 行) | 触る場所(Component / use case / knob) | 画像 | 判定 | OPEN |
|---|---|---|---|---|---|---|
| B3 Find | 手癖のまま | 常設の検索欄。1 字ごとに更新、複数語は AND、Esc で消去、2 回目で欄を出る | Browser B3 Find / B3-a Find unified query / knob Query, Place | shots/B3-find.jpg | RELEASE | なし |
| B2 Tag bands | 手癖のまま | タグ群は AND で効き、帯が件数を言い、Clear で全解除(Ableton 型) | Browser B2 Tag bands / B2-a Ableton type / knob Add mode | shots/B2-tag-bands.jpg | RELEASE | なし |
| B4 Results band | 手癖の上の追加 | 結果帯が今の絞りの数を言い、0 でも出口を示し、Clear は 1 click | Browser B4 Results band / n filters and Clear / knob Filters on, Search text | shots/B4-results-band.jpg | RELEASE | なし |
| B3-b Tab menu | 手癖の上の追加 | Tab でポインタ位置に小検索、Enter で選択に適用、Esc は何も書かない | Browser B3 Tab menu / type-ahead at the cursor / knob Target | shots/B3-tab-menu.jpg | RELEASE | 今回 FIXED: family 帯に chevron と末尾の fade |
| B3-c Prefix | 手癖の上の追加 | 先頭 1 文字(# > @)で領域を切替。印を打たない人は何も失わない | Browser B3 Prefix / marks pick the area / knob Query | shots/B3-prefix.jpg | RELEASE | なし |
| B3-d Empty query | 手癖の上の追加 | 空欄は 最近 → 選択に合う物 → 残り、各行が出ている理由を言う | Browser B3 Empty query / recent, then what fits | shots/B3-empty-query.jpg | RELEASE | なし |
| B2-b Chips | 手癖の上の追加 | `kind:effect` と打つと chip になり、chip 数 = 絞りの数 | Browser B2 Chips / sentences become chips | shots/B2-chips.jpg | RELEASE | なし |
| B2-c Sentence heading | 手癖から外れる(外す: Ableton の件数+Clear 帯。得る: 何に絞られているか 1 文で読める) | 見出しが全絞りを 1 文で言い、語を押すとその絞りだけ外れる | Browser B2 Sentence / the filter, said in words | shots/B2-sentence.jpg | RELEASE | あなたの採否(下記 3-4)。見出しが上の tab/switch と重なる |
| B2-d Saved filters | 手癖の上の追加 | 保存した絞りは件数付きの置き場になり、消しても素材は消えない | Browser B2 Saved filters / a filter becomes a place | shots/B2-saved-filters.jpg | RELEASE | Delete を hover だけで出すか(下記 3-6) |
| B10-a Collections | 手癖のまま | 数字 1-9 で collection に入れ、0 で全部外す。色は使わず名前 | Browser B10 Collections / number keys 1 to 9 | shots/B10-collections.jpg | RELEASE | なし |
| B10-b Star and recent | 手癖のまま | 星は 1 種(F)。Recent / Frequent は使用から自動 | Browser B10 Star and recent / F stars, the rest fills itself | shots/B10-star-recent.jpg | RELEASE | なし |
| B10-c Used stack | 手癖の上の追加 | 使った物が先頭に積まれ、同じ物は前へ、12 超で古い物が落ち、何が落ちたか言う | Browser B10 Used stack / newest on top / knob Shelf size | shots/B10-used-stack.jpg | RELEASE | なし |
| B12-b Similar | 手癖の上の追加 | Y か Similar で近さ順に並べ替え、1 つの外せる絞りとして出る | Browser B12 Similar / more like this | shots/B12-similar.jpg | RELEASE | 前提 OR-B1(下記 3-7) |
| B5-a Moving tiles | 手癖から外れる(外す: 静止サムネ。得る: 見た瞬間に動きが分かる) | 見えている tile だけが動き、周期は共通、reduce-motion で 1 コマ止め | Browser B5 Moving tiles / every tile screens itself / knob Tiles, Reduce motion | shots/B5-moving-tiles.jpg | RELEASE | 絵の色相(下記 3-3) |
| B5-b/B6 Hover and scrub | 手癖のまま | hover でその tile だけ再生、x 位置が時刻、見ただけでは何も書かない | Browser B5 Hover and scrub / tiles, rows, long clips | shots/B5-hover-scrub.jpg | RELEASE | なし |
| B5-c Own frame | 手癖の上の追加 | 矢印で自分のコマに候補を重ね、Enter まで書かない。Esc・Cmd-Z で戻る | Browser B5 Own frame / peek with the arrows / knob Trigger | shots/B5-own-frame.jpg | RELEASE (今回 FIXED) | なし |
| B5-d Preview surface | 手癖のまま | 固定の大きい面が選択に追随、Space で再生、Hide で高さを返す | Browser B5 Preview surface / a fixed big surface / knob Sample frame | shots/B5-preview-surface.jpg | RELEASE | なし |
| B7-a Click | 手癖のまま | 単 click = 選ぶ、dbl-click / Enter = 使う、drag = 置く。全棚で同じ | Browser B7 Click / B7-a Click selects, double-click uses | shots/B7-click.jpg | RELEASE | なし(B7-b/c は反例・不採用案として同居) |
| B9-a Swap | 手癖のまま | Q で Browser を effect に結び、↑↓ で着せ替え、Enter 確定、Esc で戻る(Ableton) | Browser B9 Swap / B9-a Hot-swap: link mode | shots/B9-swap.jpg | RELEASE | なし |
| B12-a Library | 手癖のまま | 一覧とサムネは同じ物の 2 面。選択と件数は保たれる | Browser B12 Library / Thumbnails, many items / knob Items | shots/B12-library.jpg | RELEASE | なし |
| B13 Context | 手癖の上の追加 | 選択に合わない物は隠すか淡くし、理由を 1 語、押しても何も書かない | Browser B13 Context / B13-a Misfits are not shown | shots/B13-context.jpg | RELEASE | なし |
| B10 Favourites | 手癖のまま | 数字 collection + 星 + 自動 Recent を 1 つの棚で | Browser B10 Favourites / B10-a Numbered collections | shots/B10-favourites.jpg | RELEASE | なし |
| B9-b〜d Cards A-B | 手癖の上の追加(b は手癖: card への drop 置換) | card へ drop で置換(Alt で新規)、↑↓ で同 family、候補と今を 2 枠で比べ Keep で 1 回書く | Browser B9 Cards A-B / B9-b Drop on an effect card | shots/B9-cards-ab.jpg | RELEASE | A/B は下記 4 |
| B8 Drop | 手癖のまま(a。b/c は外れる案として同居) | layer 上でも drop は追加、Alt で置換、結果は先に outline で見える(AE) | Browser B8 Drop / B8-a Add by default, Alt replaces / knob Alt held | shots/B8-drop.jpg | RELEASE | なし |
| B11 Mine | 手癖のまま(a)+ 追加(b,c) | 棚へ drop = 保存(元は不変)、同梱 preset は触ると複製、履歴 snapshot | Browser B11 Mine / B11-a Carry onto Mine to save | shots/B11-mine.jpg | RELEASE | 共有範囲 OR-B2(下記 3-8)。snapshot は下記 4 |
| B12-c Similar Axis | 手癖から外れる(c。外す: 過去の Explore 地図。得る: 規模での 1 軸絞り)。b/d は追加 | 軸を 1 つ選び、帯の drag で範囲に絞る(400 件) | Browser B12 Similar Axis Live / B12-c One-axis band | shots/B12-similar-axis.jpg | RELEASE (今回 FIXED) | 前提 OR-B1 |
| I2 Number | 手癖のまま | 値面が 3 役: 動かすと scrub、静止 click で入力、dbl-click で既定 | Inspector I2 Number / I2-b Value face / knob Effect groups | shots/I2-number.jpg | RELEASE | なし |
| I2 Grammars | 手癖のまま(a)+ 追加(c,d) | label を掴んで scrub、数字 click で入力。保持中に上下で刻みが変わる、hover で ± | Inspector I2 Grammars / I2-a Label grab / I2-c Ladder while held | shots/I2-grammars.jpg | RELEASE | なし |
| I3 Space | 手癖のまま(a)+ 追加(b,c) | Stage が手、Inspector は数値 1 行ずつ。panel 内の pad・dial は同じ値の別の面 | Inspector I3 Space / I3-a Stage leads / I3-b Pad, dial and box | shots/I3-space.jpg | RELEASE | OR-I1(下記 3-9) |
| I4 Key mark | 手癖のまま | 行ごとの stopwatch が 輪郭/半/塗り で状態を言い、click で key、key へ jump | Inspector I4 Key mark / I4-a Key mark on every row | shots/I4-key-mark.jpg | RELEASE (今回 FIXED: 1180.5) | なし |
| I4 Animate | 手癖のまま | Animate 切替(OFF は値だけ、ON は最初の編集で key)。スイッチ+縁+diamond+件数 | Inspector I4 Animate / I4-b Animate off, Animate on | shots/I4-animate.jpg | RELEASE (今回 FIXED) | 初期値 OR-I2(下記 3-10) |
| I5 Curve | 手癖のまま | 名前付き preset + handle。hover と ↑↓ は覗くだけ、click か Enter で 2 key に書く | Inspector I5 Curve / I5-a Named presets + handles | shots/I5-curve.jpg | RELEASE | なし |
| I5 Words | 手癖の上の追加 | 動きを言葉(10 種)で選び、tile は自分で 1 秒動く | Inspector I5 Words / I5-c Motion words / I5-d Copy ease | shots/I5-words.jpg | RELEASE | なし |
| I6 Tone | 手癖から外れる(a: 外す: 無印の AE。得る: 一目で差が分かる)+ 手癖(b: U/UU 型の差分だけ表示) | label の濃さで「既定との差」を言い、差だけに絞れる | Inspector I6 Tone / I6-a Label tone / I6-b Show only what changed | shots/I6-tone.jpg | RELEASE | a の採否は同じ「外れる」枠 |
| I6 Reset | 手癖のまま(c: group reset)+ 追加(d) | group 見出しの Reset はそのグループだけ。key・driven は残し、1 undo | Inspector I6 Reset / I6-c Reset in the group header | shots/I6-reset.jpg | RELEASE (今回 FIXED) | なし |
| I7 Fold | 手癖のまま | 主要な値を先に、長い残りを件数付きで折る | Inspector I7 Fold / I7-a Advanced fold | shots/I7-fold.jpg | RELEASE | なし |
| I7 Find | 手癖のまま | 上端の欄で打つごとに行が絞られ、不一致は戻り道つきで言う | Inspector I7 Find / I7-b Find and narrow | shots/I7-find.jpg | RELEASE | なし |
| I7 Pin Recent | 追加(c 星)+ 外れる(d 触った順。手が覚えられず不採用寄り) | 星を付けた行が全 layer で上に固定される | Inspector I7 Pin Recent / I7-c Pin with a star | shots/I7-pin-recent.jpg | RELEASE | d は台帳で不採用寄り |
| X Controls | 手癖のまま(X3,X4)+ 追加(X2) | 数値の drag・Shift・Alt・矢印・wheel は全ての数で同じ意味、hover=覗く click=残す Esc=戻す | Inspector X Controls / X4 One grammar for numbers | shots/X-controls.jpg | RELEASE | なし(dim の Undo は低コントラスト、未検証) |
| X States | 手癖のまま(X6,X7)+ 追加(X5) | 行き止まりは題と動く語だけ。編集の対象を値の前に帯で言う | Inspector X States / X5 Who an edit will change | shots/X-states.jpg | RELEASE (今回 FIXED: Parts の overflow なし) | なし |
| I8 Stack | 手癖のまま | effect を card で積み、header は fold のみ、並べ替えは専用 grip | Inspector I8 Stack / I8-a Card stack | shots/I8-stack.jpg | RELEASE | なし(I8-b chain は retired) |
| I11 Clipboard | 手癖のまま | 値・ease・色・key・effect の一側面だけ copy/paste、持っている物を言う | Inspector I11 Clipboard / I11-a Clipboard of one aspect | shots/I11-clipboard.jpg | RELEASE | なし |
| I11 Eyedrop Shelf | 手癖の上の追加 | 他 layer の値を hover で読み、click で残し、Esc で離れる。側面は棚に保存 | Inspector I11 Eyedrop Shelf / I11-c Eyedropper / knob Start | shots/I11-eyedrop-shelf.jpg | RELEASE (今回 FIXED: Cancel 1 行) | なし |
| I9 Whip | 手癖のまま | 値から相手の値へ線を引いて結ぶ(AE の pick-whip)。型の合う相手だけ灯る | Inspector I9 Whip / I9-a Pick-whip grab and tie | shots/I9-whip.jpg | RELEASE | なし |
| I9 Lasso | 手癖の上の追加(既決の入口) | 小さい地図で layer を囲み範囲を決めて 1 対 N で従わせる | Inspector I9 Lasso / I9-b Lasso one to many | shots/I9-lasso.jpg | RELEASE | なし |
| I9 Macro Inline | 手癖の上の追加 | 取っ手を 1 つ作り、値ごとの範囲で結ぶ。行の下に 1 行の relation を出す | Inspector I9 Macro Inline / I9-c Macro handle / I9-d One-line relation | shots/I9-macro-inline.jpg | RELEASE | なし |
| I12 Mixed | 手癖のまま | 複数選択で違う値は「混在」、scrub は各自へ相対、入力は全員へ絶対 | Inspector I12 Mixed / I12-a Mixed values | shots/I12-mixed.jpg | RELEASE | なし |
| I12 Stagger Grab | 手癖の上の追加 | ずらしの step を値の隣に出し、Falloff 図、1 property を全員から掴む | Inspector I12 Stagger Grab / I12-b Stagger beside the value | shots/I12-stagger-grab.jpg | RELEASE | bar の色(下記 3-5) |

(上の 48 行 = 部品数。B5 Hover and scrub は B5-b と B6 を 1 行にしている。)

## 3. あなたが選ぶ項目(本当に同じ価値の問い)

私の傾きは、判断の順(利用者の便利さ → 実制作 → 先例 → Motolii 内の一貫)で付けた。色相は先例でも決まらないので「傾き」は弱い。

### 3-1. accent の色相(ペリウィンクル #6982D1 付近)
- 選択肢: ① 今のまま ② 別の 1 色 ③ グレーだけ(open-decisions E)。
- 見る画像: shots/I4-animate.jpg(Animate ON のスイッチ・縁・diamond が accent)、shots/B3-find.jpg(選択中 tab の下線)、shots/B5-own-frame.jpg(PREVIEW の破線)。
- 傾き: ① のまま。理由(一貫): 選択・Animate・PREVIEW が既に同じ 1 色で通っており、先例も accent 1 色が Linear・Framer・Warp。③ は Animate ON とのオフの差が灰同士になり、暗所で読みにくい。色相そのものの好みはあなたの専権。

### 3-2. tab の色相
- 選択肢: ① accent と同じ ② 別の色 ③ 無彩色(g20 の丸い面+白字のみ)。
- 見る画像: shots/B3-find.jpg、shots/B3-tab-menu.jpg、shots/B2-chips.jpg。
- 傾き: ③ + 下線だけ accent。理由(一貫): open-decisions I の選択の見せ方(g20 面+1px accent 下線)に揃えると、色を足さずに済む。

### 3-3. thumbnail(動く tile の絵)の色相
- 選択肢: ① 無彩色(今) ② 素材の実色を出す ③ 種類ごとの淡色。
- 見る画像: shots/B5-moving-tiles.jpg、shots/B12-library.jpg、shots/B5-hover-scrub.jpg。
- 傾き: ② 実素材では実色。理由(実制作): 実際の素材 thumbnail は元の色を持ち、grey のままでは色を選ぶ作業(look 選び)が成り立たない。サンプル絵の段階は grey でよい。

### 3-4. B2 Sentence heading(文章見出し)を採るか
- 選択肢: ① 採る(Ableton の件数+Clear 帯を置き換える) ② 採らない(帯+chip のまま) ③ 帯の下に 1 行だけ併置。
- 見る画像: shots/B2-sentence.jpg と shots/B2-chips.jpg、shots/B4-results-band.jpg を並べる。
- 傾き: ② 採らない(options-browser D-11 と同じ)。理由(手癖・先例): Ableton の帯と chip で同じ状態が既に言えており、Sentence は上の tab/switch を言い直すだけ。外す手癖に見合う得が薄い。ただし「何に絞られているか」を一目で読みたい人には ③ が妥当。

### 3-5. I12 Stagger の bar の色
- 選択肢: ① accent ② stage 軸の青 ③ 無彩色(白〜灰)。
- 見る画像: shots/I12-stagger-grab.jpg。
- 傾き: ③ 無彩色の濃淡 + 選択中だけ accent。理由(一貫): 色は relation 家族の印という既決(D-8、基準 4)があり、グラフの棒に独立した色を足さない。

### 3-6. Saved filters の Delete を hover のときだけ出すか
- 選択肢: ① hover のときだけ出す ② 常時出す(淡く) ③ 長押し・右 click の menu に入れる。
- 見る画像: shots/B2-saved-filters.jpg(Parts の Places and buttons も見る)。
- 傾き: ① hover のみ + focus 中も出す。理由(実制作・先例): 保存済み絞りの削除は稀で、常時の × は一覧を騒がせる。Finder・Ableton の保存検索も hover で出す。ただし hover の無いキーボード操作では focus 時に出す。「素材は消えない」を契約で言い切れているので事故も軽い。

### 3-7. OR-B1 素材の規模(1 案件が数十か数千か)
- 選択肢: ① 数十(B12-a で足りる) ② 数千(B12-b 似た物・B12-c 軸帯を作り込む)。
- 見る画像: shots/B12-library.jpg(knob Items 5,000)、shots/B12-similar.jpg、shots/B12-similar-axis.jpg。
- 傾き: 先例では決まらない。② なら B12-b/c を本実装まで進める。作りながら試せるよう、B12-a を既定のまま B12-b は追加として残す(手癖を壊さない)。

### 3-8. OR-B2 保存物の共有棚(チーム)
- 選択肢: ① 個人棚のみ ② チームで共有する棚が要る。
- 見る画像: shots/B11-mine.jpg。
- 傾き: ① 個人棚のみから始める。理由(実制作): D-7 の「利用者の共有棚が既定」は個人の話。チーム共有は同期・衝突の別問題で、先に価値が見えない。

### 3-9. OR-I1 空間の手触りを panel 内の instrument に持つか、Stage の gizmo に任せるか
- 選択肢: ① Stage の gizmo+数値行(I3-a) ② panel 内の pad/dial(I3-b) ③ 両方(a が既定、b は展開で出す I3-c)。
- 見る画像: shots/I3-space.jpg(Parts と I3-a / I3-b)。
- 傾き: ③。理由(手癖): AE・Figma・Blender の手癖は Stage の gizmo で、これが既定。panel 内の instrument は Stage を常に見ない場面(単独の Inspector)のための追加。あなたが Stage を常に見て作るなら ① だけでよい。

### 3-10. OR-I2 Animate(auto-key)の初期値
- 選択肢: ① OFF(AE。意図しない key を避ける) ② ON(他の多く)。
- 見る画像: shots/I4-animate.jpg(OFF と ON の両方を use case で見る)。
- 傾き: ① OFF。理由(手癖・先例): あなたの手癖は AE の Animate 型で、OFF なら「触ったら key が増えた」事故が無い。ON の道具の人は 1 click で切り替えられ、状態が帯に出る。

(OR-B1, OR-B2, OR-I1, OR-I2 は上の 3-7 〜 3-10。)

## 4. 既存の道具のどれも解いていない問題(Motolii が違う所)

| 問題 | どの道具も解けていない所 | 探っている部品 |
|---|---|---|
| 1 側面だけのコピー(ease だけ・色だけ・effect 群だけ) | AE/Blender は丸ごと copy。部分は property 単位か手作業 | I11 Clipboard、I5 Words(I5-d Copy ease)、I11 Eyedrop Shelf(I11-d 棚) |
| A/B と snapshot(候補と今の絵、履歴の名前付き保存) | NLE の A/B は別機能、undo 履歴は名前を持てない。確定まで履歴に残らない試写も無い | B9 Cards A-B(B9-d A/B)、B5 Own frame、B9 Swap、B11 Mine(B11-c History snapshots) |
| アプリ全体で 1 つの文法(数値・click・Esc・「確定していない」) | 道具ごと・面ごとに drag の意味が割れる | X Controls(X4)、I2 Number / Grammars、B7 Click、X States(X6/X7) |
| 面を dock した時に panel が自分の声で話す(幅 176-282 でも同じ契約、行き止まりは理由+次) | 幅が変わると語や機能が落ちる、空状態は無言 | X States(X7 Docked widths)、B3 Find(narrow dock)、I2 Grammars(dock 220)、X5 の対象帯 |
| 選択に合わない物を隠さず理由を言う(使えない物の扱い) | 多くは灰にして終わり、理由が無い | B13 Context、B3-d Empty query、B9 Swap(Link が disabled の理由) |
| 多数に値を連動させる時、間違った相手に結ばない | pick-whip は相手が全て見えず誤結線しやすい | I9 Whip(型の合う相手だけ灯る)、I9 Lasso、I9 Macro Inline、I12 Stagger Grab |

## 5. 色(2026-10-02 追加: 「色がなくて見づらい」への対応)

- 触る場所: Widgetbook の右パネル「Addons」タブ → Colour(Grey / Palette A / Palette B / A+色覚シミュレーション 3 型)。全パネルが切り替わる。初期値は Palette B。
- 調査: research/color-ud.md(CUDO・Okabe-Ito・WCAG、色差と色覚シミュレーションの計算)、判定: research/critique-colour.md、research/critique-colour-r2.md。画像: shots/colour-*.jpg。
- 判定: Palette B = おすすめ(ブロックする欠陥なし)。Palette A = 条件つきで可(キーの黄が強い。変更とモードが D 型で近い=形で区別)。Grey = 今(見づらい)。
- あなたが選ぶ項目:
  1. 配色 A / B / Grey(私の傾き: B。ユーザーの便利さ=変更済みが一目で見つかり、落ち着いた密な見た目が保たれる。制作=状態の読み取りが速い。先例=Blender の黄=キー・緑=アニメ・紫=ドライバー。一貫性=役割 10 個を 1 か所の Role で管理)。
  2. 「変更済み」の色: 空色(A/B 今)か、オレンジ(Blender の「変更あり・キー未設定」流)か。先例がないので専権。
  3. DESIGN.md の色の規律の更新(下)。
- 残った小さな欠陥: B2 のチップの反応が 1px の下線だけで弱い、B5/B2 の選択バーが白のまま、B2 の「0 件」の空状態に「!」がない。
- DESIGN.md の更新案(決めるのは利用者): 「色は関係の 6 色とアクセント 1 つだけ」→「色は ①関係の 6 色(バー・線) ②状態の役割色(変更・キー・リンク・選択・モード・成功・警告・エラー)を Role で 1 か所管理。小さな印・1px の縁・下線・変更値の文字だけに使い、大きな塗りや 2px 超の線には使わない。1 行に役割色は 2 つまで。色は唯一の伝達手段にしない(形の手がかりを必ず併用)。エラーは必ず記号付き。」

## 6. 現象のミニチュアと世界(2026-10-02 追加: 数値を現象に変える GUI のトーナメント)

- 触る場所: Widgetbook の Component 'Phenomenon A〜E'(パラメータ群のミニチュア 30)と 'World A〜E'(OP-1 流の、Effect/Relation ごとの小さな世界 29)。World の knob 'Driven'(Off/Wiggle/Pulse)で変調を見る。Words addon=Hidden で文字なしテスト。
- 判定: research/critique-pheno.md、critique-worlds.md、tournament-pheno-worlds.md。勝者の画像: shots/win-*.jpg。
- 勝者(概念ごと): ミニチュアが勝ち=Scatter A1、Stagger A2、Echo E4、Warp C3、Blur B5、Shadow B3、Wiggle D3、Time remap D5、Opacity C6。世界が勝ち=Falloff A1、Glow C2、Noise C5、Easing E1、Spring E2、Loop E5、Mask feather D5、Stroke D4、Glass D1、Repeat B3、Fill D3。Camera shake(D6)と Camera(E6)は別概念で両方残す。
- 一番おもちゃらしい 10: Echo(彗星)、Shadow と太陽、Opacity(猫の窓)、Noise(等高線)、Glass、Repeat(六角格子)、Glow、Easing(足跡)、Variations、Spin(独楽)。
- 落とす/直す: Grid(ミニチュア)は落とす寄り、World の Stagger(ドラッグが歩く線だけ)・Graph/Link(中央の取っ手が死んでいる)・Blur(箱の 22% だけ)は直す、World Noise は折れ線グラフに見えるので落とす(Noise は Fractal Noise C5 が勝ち)。
- 未確認: Words Hidden は再判定していない。Driven=Pulse は 6 世界だけ。5 つの勝者(Spring E2、Mask feather D5、Easing E1、Repeat B3、Opacity C6)は再撮影していない。
- あなたが選ぶ項目: 遊び心の量(どのミニチュア/世界を採るか、複数の概念で重なる物はどちらか)。色の割当(A 青・B 緑・C 白・D オレンジ)をトークンにするか。
