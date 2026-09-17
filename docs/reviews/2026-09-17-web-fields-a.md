# Web の欄 a — Clip 辺・Split・Loop(2026-09-17)

[Web の写し](2026-09-17-web-clone-gallery.md) の「要る欄」のうち、台本を短くする 3 つを足した。芯は利用者の原則: **ソフトが持つのは意味でなく道具、欄の名前は Web の語、値は時刻の純関数、Web の意味(既定込み)をそのまま写す、離散の切替の後に連続の収まりが置ける**。

## 1. Clip 辺 — `clip-path: inset(top right bottom left round r)`

- **意図**: 帯で開く・拭き取る・角から拭く・道の残像・面が四隅へ開く(s2・s3・s5・c7・c10)。今までは 1 辺を動かすのに **鍵 3 本**(箱の Height + 箱の Position + 中身の Position)で「遠い辺を留める」形にしていた。
- **札**: 並べる Group の欄 `Clip Top / Clip Right / Clip Bottom / Clip Left`(CSS の inset の順、px、既定 0、辺ごとに鍵が打てる、負も可 = CSS)と `Clip Radius`(= inset の `round`、既定 0)。
- **先例**: CSS `clip-path: inset()`(CSS Masking 1 §6 basic-shape)。`inset()` の `round` は `border-radius` と別の値なので、Overflow Clip の角(Border Radius)と Clip Radius は別の欄。clip-path は**要素ごと**に掛かる(自分の背景も切る)、overflow は箱の外の子孫だけ。
- **借りた物**: 既にあった Overflow Clip の切り方(`layout.rs clip_box` → `resolve.rs clipped_masks` が祖先の箱を Intersect の mask で子孫へ渡す)。**足した物**: `clip_inset`(4 辺で削った箱、Clip Radius)、`clipped_masks` が自分自身の inset も切る。描く側は触っていない(mask は既存)。
- **絵**: s2・s3・s5・c7・c10 の evidence は差し替え前と同じ絵(見比べて確認)。s3 は Web の通り「上の画を右から削る」形に戻せた(Width の鍵で要った回避 = 詰まった所 2・3 が消えた)。c10 は箱の Background が inset で切れる(CSS の通り)。
- **test**: `clip_inset_cuts_the_box_from_each_edge_like_css_inset`(`inset(10% 0 0 0)` の 400×300 → 上 30 px が消える、CSS の辺の順、辺が向かいを越えたら空、Overflow Clip と inset は 2 つの切り)。
- **仕様が開けている所(決めていない)**: (1) `%` の単位 — 値の仕組みに % が無いので px だけ(台本は箱の寸法を知っているので写せた)。(2) inset は Display の Group だけ(箱の大きさは並べる箱から取る、Display None の Group・文字・画の層には効かない)。(3) `clip-path` の他の形(circle / ellipse / polygon / path)は無し(s5・c10 の polygon は辺の enum で足りた)。(4) 3D の層への切り口(既存の Overflow と同じ制限)。

## 2. Split — 文字を字・語・行の単位に(GSAP SplitText / CSS `sibling-index()`)

- **意図**: 語ごと・字ごとに時間差で立ち上がる・晴れる・跳ねる・起きる(s1・c1・c2・c3)。今までは台本が `split(" ")` / `[...line]` で **1 語 1 層・1 字 1 層**を手で作り、Hug の行に並べて行の Stagger でずらしていた(空白は Opacity 0 の升)。
- **札**: 文字の層の欄 `Split`(`None | Chars | Words | Lines`、既定 None = SplitText の `type`)。単位の順番は**その文字の層の** `Stagger / Stagger From / From End`(箱の子と同じ 3 欄、同じ法 = 全体の幅)。読む順 = 組んだ順(行の上から、行の中は左から)。空白は単位にならない(SplitText と同じ)。
- **先例**: GSAP SplitText(`type: "chars,words,lines"`、各 char は自分の中心で変形)、CSS `sibling-index()`(兄弟の番号を値に)、AE の Text Animator の Based On(Characters / Words / Lines — 名前の元)。
- **借りた物**: 順番の札 `layer_time`(箱の子の時刻をずらす)を `schedule_shift(holder, i, n, base)` に括り出し、単位にも同じ法を掛ける。Repeater の「時刻のずれた配置は、その時刻の姿を取り直す」(`resolve_with_solo(layer, at)`)と写しの `copy` 番号、mask の Intersect(Overflow Clip と同じ道)。行の物差し `ShapedText.lines`(字の x・元の byte)は 文字組み lane の物。**足した物**: `text_frame::split_boxes`(単位の箱、素材座標)、`layout::text_units`、`resolve::push_split`(単位ごとに層全体をずれた時刻で解き、箱で切り、変形の中心を箱の中心へ共役で移す)、`names::TEXT_SPLIT`。**書類に子の層は作らない**(時刻の純関数、文字組みの法は段落全体に掛かったまま)。
- **絵**: s1・c1・c2・c3 の evidence は差し替え前と同じ絵(s1 の 1/3 コマで「2026」が「TOKYO」より遅れて上がる = 語ごとの時計)。
- **test**: `split_words_give_each_word_its_own_time_under_the_texts_stagger`(3 語 + Stagger 0.2 で 2 語目は 0.1、3 語目は 0.2 遅れて鍵を読む、単位は読む順で自分の箱、Chars は空白を数えない、1 行の Lines は分けない)。
- **仕様が開けている所(決めていない)**: (1) **Stagger の意味は既存の「全体の幅」**(GSAP `stagger: {amount}`)のまま — 依頼の test 文「3 語 + Stagger 0.1 で 2 語目が 0.1」は GSAP `stagger: 0.1`(1 つずつ)の読みで、Motolii では 3 語目が 0.1。1 つずつの欄(`Stagger Each`)を足すかは裁定待ち。(2) 単位の箱 = 字の送り幅 × 行の箱の縦 — 送りをはみ出す字形(斜体の f、装飾)は隣の単位の側で切れる(SplitText は overflow visible)。(3) 写しは**層の鍵・効果・変換**をずれた時刻で読むが、**文字の中身と字の style の鍵は描く側がコマの時刻で読む**(単位ごとに違う中身は c9 の scramble の欄)。(4) Split の文字に Repeater・Motion Blur は掛からない(`push_split` が先に積む)。(5) Transition(FLIP)・Arrive は単位を見ない(Arrive は語彙に無い)。(6) 行の Split は折り返しの行(`Lines`)だけで、`\n` の段落も行として数える。

## 3. Loop — `animation-iteration-count: infinite` + `animation-direction`

- **意図**: 流れ続ける帯・漂い続ける札(s7・s10)。今までは周期ごとに鍵を打ち直す(s10 は 1.8 s ごとに反転の鍵、s7 は折り返しの鍵を隣り合う 2 コマの間に置く)。
- **札**: 全ての層の欄 `Loop Duration`(秒、既定 0 = 繰り返さない)と `Loop Direction`(`Normal | Reverse | Alternate | Alternate Reverse` = CSS `animation-direction` の 4 値、既定 Normal)。層の時刻 t を、順番の札(Stagger)でずらした後に周期で畳んでから鍵・効果を読む: normal = t mod D、reverse = D − (t mod D)、alternate = 奇数回目が逆向き、alternate-reverse はその逆。時刻の純関数。
- **先例**: CSS Animations `animation-iteration-count: infinite` + `animation-direction`、Remotion `<Loop durationInFrames>`、AE の `loopOut("cycle"|"pingpong")`。
- **借りた物**: 順番の札の口(`view.rs value_at` が層の時刻をずらす 1 か所)。**足した物**: `layout::looped_time`、`is_loop_row`(Loop の欄自身は畳まずに読む)。
- **絵**: s7・s10 の evidence は差し替え前と同じ絵。
- **test**: `loop_folds_the_layers_time_like_css_animation_direction`(鍵 0 → 1 s、D = 1: t = 2.5 → 0.5、Alternate の t = 1.5 → 0.5 / 1.2 → 0.8、Reverse は毎回逆、Loop の欄自身は畳まない)。
- **仕様が開けている所(決めていない)**: (1) **周期の原点はコンポの 0 秒**(CSS は要素の animation の開始 = delay 後、Remotion は `<Loop>` の置かれた frame)。層の in 点や最初の鍵を原点にするかは裁定待ち。(2) `animation-iteration-count` の**有限回**(3 回で止まる)と `animation-fill-mode` は無い(infinite だけ)。(3) 畳むのは鍵・効果の欄(`value_at` を通る物)で、**文字の中身(ContentTrack)と物理の集まりの時刻(`blocks.rs` の `layer_time`)は畳まない**。(4) 子の層は親の Loop を継がない(CSS も要素ごと)— 箱ごと繰り返すなら箱の鍵に Loop を置く。(5) 依頼の名は `Normal | Alternate` だったが、CSS の enum を丸ごと(Reverse・Alternate Reverse を含めて)写した。
