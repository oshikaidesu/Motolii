# 表現別の台帳(3)— 実サイトと Codrops の 20 表現を、仕組み・時計・篩で読む

2026-09-17 調査。入口は SANKOU!(sankoudesign.com、動きのタグから国内の実サイトへ)と Codrops(ソース付きの見本)。単位はサイトではなく **表現(見た目の 1 現象)**。抜いたのは CSS の宣言と JS の関数名、時計(時間・スクロール・hover・登場)、篩 3 つ(意味を貸してこないか・貼れるか・時刻の純関数か)、Motolii の欄で書けるか。[台帳 1](2026-09-14-css-ledger.md)・[台帳 2](2026-09-17-css-ledger-2.md) の続き。

利用者(2026-09-17)「意味を再発明しなくても、テクニカルなサイトは CSS と JS で全部やっている。その語彙に習えばよかった」。

---

# SANKOU! 経由の実サイト 10 表現 — CSS/JS を読んで抽出(2026-09-17)

入口: https://sankoudesign.com の絞り込み(ダイナミックな動き / パララックス / ホバー / スライダー / Canvas·WebGL / タイポグラフィ)。掲載サイトの HTML を取り、リンク先の CSS/JS を直接読んだ。台帳 1(2026-09-14)・台帳 2(2026-09-17)に既にある法は「台帳」欄で名指しする。単位は**表現**(現象)であって、サイトではない。

篩: **a** 意味を持たない道具か / **b** 写真・文字・他人のシェーダに同じ法で掛かるか / **c** 時刻の純関数か(状態機械なし)。○ △ ×。

## 表

| # | 表現 | 出典 | 仕組み(見た宣言・関数) | 時計 | 篩 a/b/c | Motolii |
|---|---|---|---|---|---|---|
| 1 | **文字が下からマスクで立ち上がる**(語ごとに時間差) | https://toshiyukihashimoto.jp/ | 親 `.a-up{overflow:hidden;height:var(--up-h)}`、子 `.a-up__in{opacity:0;translate:0 100%}` → `[data-shown="1"]` で `translate:0 0`。分割は GSAP `SplitText(r,{type:"words",wordsClass:"a-up__in"})`、遅れは `data-stagger` | 画面に入った時(IntersectionObserver)= 層の in 点 | ○ / ○(箱なら何でも。画も同じ) / ○(in 点からの経過) | 既にある: Clip(箱の overflow)+ Position の登場 + Stagger(台帳 E `sibling-index`)。「行の箱の高さ = マスク」は `text-box-trim`(台帳 2 D)が入ると自然 |
| 2 | **見出しが上下の帯で開く**(inset クリップ) | https://toshiyukihashimoto.jp/ | `clip-path:inset(100% 0% 0% 0%)` → `inset(0% 0% 0% 0%)`、`transition:clip-path .4s var(--ease-io)`。モーダルも `gsap.fromTo(modal,{clipPath:"inset(100% 0 0 0)"},{clipPath:"inset(0% 0 0 0)"})` | 出現 / hover / クリック(全部 = キーの時刻) | ○ / ○ / ○ | 台帳 1 F `clip-path`「係が違う(マスク)」。inset の 4 辺に数値の軸があれば済 |
| 3 | **画像が左から拭き取られて次へ入れ替わる**(KV の wipe) | https://companycoc.com/ | JS: `Ke="inset(0 0% 0 0)"` → `go="inset(0 100% 0 0)"`、`style.transition=\`clip-path ${D}ms ease\``、class `is-kv-wiping`。前の画の右辺だけ縮み下の次の画が見える | 時間(自動送り `setTimeout`)+ クリック | ○ / ○(画・動画・文字) / ○ | 2 と同じ inset の 1 辺。「重ねて上を削る」= 合成の順と Clip で済、**遷移(Transition)の型として 1 つにできる** |
| 4 | **下線が右へ消えて左から引き直される**(hover の線) | https://recruit.toyox.co.jp/ | `.c-text-line{background:linear-gradient(#091E2D,#091E2D);background-size:0% 1px}`、`@keyframes textLine{0%{background-size:100% 1px;background-position:right bottom}30%{background-size:0 1px…}}` と `textLineTo`(left から 100% へ) | hover(`@media(any-hover:hover) .u-hover:hover`) | ○ / △(文字の行に付く塗り。箱の下辺なら何でも) / ○ | 無。**線の長さ(0→100%)と原点(左/右)** が 1 つの軸。台帳 1 A `flex-grow` の比・Fill と同じ「長さの軸」、原点は transform-origin の語彙 |
| 5 | **文字が四角の角から拭き出される**(clip-path の polygon を 4 方向) | https://recruit.toyox.co.jp/ | `@keyframes textClip-left{0%{clip-path:polygon(100% 0,100% 0,100% 100%,100% 100%)}20%{…(0 0,100% 0,100% 100%,0 100%)}}`、`-right / -top / -bottom` の 4 種 | 時間(登場)| ○ / ○ / ○ | 2 と同じ Clip。polygon の頂点補間は台帳 2 `shape()`「相談」。4 方向の名前は **原点の enum** だけで、法は 1 つ |
| 6 | **線が描かれてから文字が出る**(stroke の描画) | https://recruit.toyox.co.jp/ | `@keyframes line-stroke{to{stroke-dashoffset:0}}`(SVG の `stroke-dasharray` = 全長を先に置く) | 時間(登場) | ○ / △(線を持つ形だけ。文字の輪郭も線なら可) / ○ | 一部: 台帳 1 F `offset-path`「Repeater の Along Path」とは別。**Trim Path(線の 0→100%)** は AE にもあり Motolii に無ければ線の Stroke に「始点・終点」の軸を足す |
| 7 | **横に流れる帯**(marquee、2 本を半周ずらす) | https://recruit.toyox.co.jp/、https://musu-sauna.com/ | toyox: `@keyframes loop{0%{translateX(100%)}to{translateX(-100%)}}`、`animation:loop 120s -60s linear infinite`(負の delay で位相を半分ずらす)。musu: `.cm-h__infoWeather-track{width:max-content;animation:weather-scroll 10s linear infinite}`、`to{transform:translate(-50%)}` | 時間 | ○ / ○ / ○(`mod(t, 周期)`) | 台帳 2 G `mod()`「取る」+ 台帳 1 A Flex の `width:max-content`(Hug)。**負の delay = 位相** は Stagger の符号で表せるか要確認 |
| 8 | **カーソルに遅れて追う画**(hover で出るサムネが lerp で追従) | https://companycoc.com/ | JS: `u.targetX=(d.clientX-m)*sr`、`l.currentX+=(l.targetX-l.currentX)*ar`、`el.style.transform=\`translate3d(...)\``、`requestAnimationFrame` | pointer(mousemove)| ○ / ○ / ×(**lerp は前フレームの記憶** = 1 次の遅れ。目標が時刻の関数なら畳み込みで純関数に写せるが、Web はそうしていない) | 台帳に無い。物理の法(9/13 の「留め具」)の「ばね・遅れ」で書けるか相談。時計は pointer だが動画では「別の物の位置」を読む口(台帳 2 の時間の法 3) |
| 9 | **スクロールで画が暗くなる**(sticky の上に黒を被せ、変数 1 つで濃さ) | https://musu-sauna.com/ | CSS: `.p-tsunagaru__mvImg{--scrub:0;position:sticky}`、`:before{background-color:#00000080;opacity:var(--scrub)}`。JS: `gsap.fromTo(n,{"--scrub":0},{"--scrub":1,ease:"none",scrollTrigger:{…scrub:!0}})` | scroll(= s∈[0,1]) | ○ / ○(何を被せても) / ○(**scroll = 時間**の典型) | 台帳 1 F `@property`「1 つの変数に鍵 → 全体」と台帳 2 の時間の法 3(スカラー駆動)。Motolii では s = 時刻で済、被せは合成の係 |
| 10 | **回り続ける円環に物が乗る / 上下に漂う** | https://mamamamamamama.com/、https://recruit.toyox.co.jp/ | mama: `animation:home-objects-spin 360s linear infinite`、`@keyframes home-objects-spin{to{rotate(360deg)}}`(子の画は逆回転せず一緒に回る)。toyox: `@keyframes floating-y{0%{translateY(-10%)}100%{translateY(10%)}}`(alternate)、`circle-rotate` | 時間 | ○ / ○ / ○(`sin`, `mod`) | 済: 台帳 2 G 三角関数「取る」+ 親の Rotation。ふわふわは Wiggle でなく `sin(t)` の純関数 |

補: WebGL の物(読めた範囲)— **aratanagara.com** は自前 WebGL2(`shredder.js`: 「カール(渦)ノイズ…1 粒ごとに鋭利な多角形の破片」、`uzumaki.js`: 「24枚のカードとして3D対数螺旋へ展開」)で、CSS では書けない粒子・折り紙。ただし写真は「テクスチャ 1 枚」(`setTextureFromUrl`)= **b の篩を満たす**(何の画にも掛かる)。**twotone.jp** は `<canvas class="voronoi-canvas">` のボロノイ hero(JS は Astro のバンドルで読めず、DOM と `scroll-snap-type:y mandatory` から推定)。**eightdesign.co.jp / andmade.jp** は Next.js のチャンクで読めなかった(未記載)。musu-sauna・toyox・hashimoto・coc は全部 **GSAP + Lenis(慣性スクロール)** で、`lerp` は Lenis 側にもある(`lerp:u=!l&&.1`)。

## 見立て

1. **1 つの法「クリップの辺・頂点に数値の軸」**に 1・2・3・5 が畳まれる。上から出る(1)・帯で開く(2)・拭き取り遷移(3)・4 方向(5)は全部 `inset()`/`polygon()` の**原点と長さ**の違いだけ。Motolii のマスクに「辺の %」と「原点の enum」があれば 4 表現が 1 つの欄
2. **「長さ 0→100% と原点」**は 4(下線)・6(stroke の描画)にも同じ形で現れる。塗りの幅(background-size)と線の dash は素材が違うだけで法は同じ = 篩 b が通る
3. **scroll = 時間**: 9 は変数 1 つ(`--scrub`)を scroll で 0→1 にしているだけ。7・10 も `mod(t)`/`sin(t)` の純関数で、Web は `animation: … linear infinite` と負の delay(位相)で書く。動画では時刻に置き換えて終わり(台帳 2 時間の法 3)
4. **状態が漏れる唯一の物は 8(lerp の追従)**。前フレームを持つので純関数でない。物理の「留め具(ばね)」へ寄せるか、目標(t) の畳み込みで書き直すかは相談。台帳に無い
5. WebGL 物(aratanagara の粒子・折り紙、twotone のボロノイ)は表現が完成形に寄る(篩 a が △)が、**画を 1 枚のテクスチャとして受ける**点は Motolii の「他人のシェーダに貼れる」と一致。棚の 1 枚として足す候補、法ではない

## Sources

- https://sankoudesign.com/category/dynamic-move/ ・ /category/parallax/ ・ /category/hover/ ・ /category/slide/ ・ /category/canvas-webgl-threejs/ ・ /category/typography/
- https://toshiyukihashimoto.jp/ — `/assets/css/style.BviHdqiV.css`、`/assets/js/Dg_lTjbO.js`(GSAP・SplitText・Lenis を含む)
- https://companycoc.com/ — `/wp/wp-content/themes/coc-theme/assets/js/main.js`(バンドル済だが関数は読める)
- https://recruit.toyox.co.jp/ — `/assets/css/style.css`、`/assets/js/main.js`(GSAP・Swiper)
- https://musu-sauna.com/ — `/assets/css/style.css`、`/assets/js/main.js`(GSAP ScrollTrigger・Lenis・Splide)
- https://mamamamamamama.com/ — `/_astro/index@_@astro.DZBwCu2z.css`
- https://aratanagara.com/ — `/wp/wp-content/themes/aratanagara-brut/js/shredder.js`、`uzumaki.js`、`function.js`(WebGL2、コメントは日本語)
- https://twotone.jp/ — HTML の `<canvas class="voronoi-canvas">`、`/_astro/index.D2Au_qr-.css`
- 台帳: docs/reviews/2026-09-14-css-ledger.md、docs/reviews/2026-09-17-css-ledger-2.md

---

# Codrops 2024–2026 の表現 10 — Web が軽々と使う動きを、法の単位で

2026-09-17 調査。github.com/codrops の repo(created 2024-04 → 2026-06)の js/css を raw で読んだ。篩は台帳 2 と同じ: **a** 意味を貸してこない道具か / **b** 写真・文字・他人のシェーダに同じ法で掛かるか / **c** 時刻の純関数か(状態機械なし)。○ △ ×。

| # | 表現 | 出典(demo / repo) | 仕組み(CSS / JS) | 時計 | 篩 a/b/c | Motolii |
|---|---|---|---|---|---|---|
| 1 | 文字がぼけて暗い所から 1 字ずつ晴れる | [ScrollBlurTypography](https://tympanus.net/Development/ScrollBlurTypography/) / [repo](https://github.com/codrops/ScrollBlurTypography) `js/effect-1/blurScrollEffect.js` | SplitType で chars → `gsap.fromTo(chars, {filter:'blur(10px) brightness(0%)'}, …)`、`stagger: 0.05`、`scrollTrigger: {scrub: true, start:'top bottom-=15%'}` | scroll(= スカラー) | ○ / ○(filter は何にでも) / ○ | 一部: Stagger(番号比例)+ 効果の blur。**進みを scroll でなく時刻**にすれば既存で書ける。台帳 E「scroll timeline」= 口 1 つ |
| 2 | 語の下に蛍光ペンが左から伸びる | [OnScrollTextHighlight](https://tympanus.net/Development/OnScrollTextHighlight/) / [repo](https://github.com/codrops/OnScrollTextHighlight) `js/effect-5/highlightEffect.js`, `css/base.css` | `.hx-5::after { background: var(--color-bg-highlight) }` を `gsap.to(el, {'--after-scale': 1, ease:'expo'})`(型付き変数で scaleX)。字は `stagger: pos => 0.1+0.05*pos`。別案 `.hx__select { mix-blend-mode: plus-lighter }` | scroll(onEnter で発火 → 時間) | ○ / △(文字の箱に付く帯、帯自体は形なので何にでも) / ○ | 一部: 語の箱に子の矩形 + Scale + 台帳 G `@property`。**「語の箱」を Anchor にする**= 台帳 B anchor positioning の候補 |
| 3 | 文字が奥から起き上がる(rotationX + z) | 同上 `js/effect-1/highlightEffect.js` | `gsap.fromTo(chars, {opacity:0, z:300, rotationX:-45}, {…, stagger: 0.04})` | scroll → 時間 | ○ / ○ / ○ | 済: 文字の Each + 3D の Rotate/Z(2.5D 札)。行に perspective が要る = 台帳外(箱の `perspective` 欄) |
| 4 | 格子の絵が中央から散って・寄って組み替わる(9 種の formation) | [OnScrollLayoutFormations](https://tympanus.net/codrops/2024/09/18/exploration-of-on-scroll-layout-formations/) / [repo](https://github.com/codrops/OnScrollLayoutFormations) `js/index.js`, `css/base.css` | `display: grid; grid-template-columns: repeat(8,1fr)`、`gsap.timeline({scrollTrigger:{pin, scrub: 0.2, end:'+=200%'}})` で `x,y,z,scale,rotateX/Y,skewX,filter:brightness`。`stagger: {from:'center' / 'edges' / 'random', grid:[4,9]}`。`calculateInitialTransform()` = 画面中心からの距離で初期変形 | scroll(scrub = スカラー) | ○ / ○ / ○ | 一部: Grid 済、Stagger の `from: center/edges/random` は台帳 E の候補。**中心からの距離で変形の量を決める**= 台帳 G `abs()/sign()` の候補 |
| 5 | 1 つの物が scroll に合わせて次の枠へ滑る(waypoint) | [OneElementScroll](https://tympanus.net/Development/OneElementScroll/) / [repo](https://github.com/codrops/OneElementScroll) `js/index.js` | `Flip.getState(stepElement)` を枠ごとに撮り `tl.add(Flip.fit(oneElement, state), '+=0.5')`、`scrollTrigger: {scrub: true, start:'clamp(center center)'}` | scroll | ○ / ○(fit は矩形の写像) / ○(枠の並びは全部先に解ける) | 済: 状態を持たない Transition(間合いの法 4・5)。scroll を時刻に読み替えるだけ |
| 6 | 格子の列が中央から遠いほど遅れて追う(弾性) | [ElasticGridScroll](https://tympanus.net/Tutorials/ElasticGridScroll/) / [repo](https://github.com/codrops/ElasticGridScroll) `js/demo1/index.js` | ScrollSmoother `smoother.effects(element, { speed: 1, lag })`、`lag = baseLag + distance * lagScale`(中央からの列の距離) | scroll + 遅れ(lag = 1 次遅れの追従) | ○ / ○ / △(lag は前フレームの記憶 = 物理の追従。時刻の純関数にするなら「遅れ = 時刻のずらし」で近似) | 一部: 並べる容器の Stagger が「距離で遅れを配る」提案(9/15)そのもの。バネの追従は physics(留め具)の係 |
| 7 | 画が枠の複製を残して道を通り、次の場所へ入れ替わる | [RepeatingImageTransition](https://tympanus.net/Development/RepeatingImageTransition/) / [repo](https://github.com/codrops/RepeatingImageTransition) `js/index.js` | 始点と終点の矩形を `lerp(startCenter.x, endCenter.x, t)` で N 段に刻んだ mover(clone)を `delay = index * config.stepInterval` で出し入れ。面は `clip-path: inset(100% 0% 0% 0%)` → `inset(0% 0% 0% 0%)`。sine/wobble で道を曲げる | click → 時間 | ○ / ○(画でも文字でも「矩形の道」) / ○ | 無: **矩形 A→B の道を N 段に離散化した残像**。Repeater(Along Path)+ Transition の合成に近いが「始点と終点の箱から道を作る」欄が無い。台帳 F `offset-path` + 台帳 E FLIP の間 |
| 8 | 見出しが墨のように滲んで・歪んでから結像する(SVG filter) | [OnScrollSVGFilterText](https://tympanus.net/Development/OnScrollSVGFilterText) / [repo](https://github.com/codrops/OnScrollSVGFilterText) `js/filter2.js`, `index.html` | `<filter id="goo-2">` = `feGaussianBlur` + `feColorMatrix` + `feComposite atop` + `feTurbulence` + `feDisplacementMap`。gsap が数値 obj を tween し `onUpdate` で `feBlur.setAttribute('stdDeviation', …)`、`feDisplacementMap.setAttribute('scale', …)`(20→0、100→0) | scroll(onEnter) → 時間 | ○ / ○(filter graph は何にでも) / ○ | **shader 印**。一部: blur は効果にある。gooey(blur → contrast の色行列)と displacement は無。台帳 F `filter` の行に「filter graph の 3 原語」を足す候補 |
| 9 | 文字が記号にばらけて 3 回明滅し元へ戻る(scramble) | [LineTextHoverAnimations](https://tympanus.net/Development/LineTextHoverAnimations/) / [repo](https://github.com/codrops/LineTextHoverAnimations) `js/effect-1/text-animator.js`;同型は ScrollTextMotion の `ScrambleTextPlugin` | SplitType `'words, chars'`、字ごとに `gsap.to(char, {duration: 0.03, repeat: 3, innerHTML: () => lettersAndSymbols[random]})`、`delay: (position+1)*0.07`、`reset()` で `char.innerHTML = originalChars[index]` | hover → 時間 | ○ / △(文字だけ。字形の差し替えは文字の法) / ○(乱数の種を固定すれば) | 無: **字の中身(glyph)を時刻の関数で差し替える**欄。Each + 台帳 G `random()`(種)+ `round(t, step)` で書ける。文字の Text Animator に「文字列の置換」の軸 |
| 10 | 面が 1 点から四隅へ開き、閉じる時は別の ease で戻る | [EaseReverseClipMenu](https://tympanus.net/Development/EaseReverseClipMenu/) / [repo](https://github.com/codrops/EaseReverseClipMenu) `js/index.js` | `clip-path: polygon(50% 50%, 50% 50%, 50% 50%, 50% 50%)` → `polygon(0% 0%, 100% 0%, 100% 100%, 0% 100%)`、`ease: 'expo'`。閉じは `easeReverse: 'elastic.out(0.3)'`(GSAP 3.13)。周りの物は中心からの距離で stagger し 600px 外へ | click → 時間 | ○ / ○(clip は何にでも) / △(easeReverse は「途中で反転」= 状態。動画では往復のキーで純関数) | 係が違う: マスクの形の補間(台帳 F `clip-path`)。**戻りの ease を別に持つ**は無 → キーの「往」「復」で書けるので欄は要らない |

補: [Staggered3DGridAnimations](https://tympanus.net/Development/Staggered3DGridAnimations/) は 4(格子)+ 3(rotateX 70・z 300)+ 1(blur→晴れ)の合成、`delayFactor = |columnIndex − middle| * 0.2` で 6 と同じ「中央からの距離」。[ScrollTextMotion](https://tympanus.net/Development/ScrollMotion/) は 5(Flip + scrub)+ 9(Scramble)の合成。[ContextAwareLogoAnimationScroll](https://github.com/codrops/ContextAwareLogoAnimationScroll) は onEnter/onLeave のイベント駆動で、動画では層の in/out 点に落ちる。

## 見立て

- **法は 3 つに畳める**: (i) 分ける + 番号で時刻をずらす(1・3・4・9 = split + stagger、`from: center/edges/random` と「中央からの距離」が同じ数 = 台帳 G `abs()`)、(ii) 箱 A → 箱 B の写像(5・7 = FLIP、7 はそれを N 段に刻んだ残像)、(iii) 面の切り抜き・filter の量に 1 本の数(2・8・10 = `--var` / `stdDeviation` / `clip-path` の補間)。
- **scroll = 時刻**: 1・4・5 は `scrub` で進みがスカラー s、そのまま t に読み替えて済(台帳 2 の結論 3)。2・3・8・9・10 は onEnter/hover/click で発火してから時間 → 動画では層の in 点 = 純関数。
- **6 だけ遅れが記憶**(ScrollSmoother の lag = 1 次遅れ)。時刻のずらしで近似するか、物理の留め具に寄せるかの相談。
- **無い欄は 2 つ**: 7「始点と終点の箱から道と残像を作る」、9「glyph の中身を時刻の関数で置換」。8 は shader 印(gooey・displacement を効果の棚へ)。
- 10 の easeReverse は Web の「途中で反転する UI」の都合。動画では往復のキーで消える(取らない)。

## Sources

- https://api.github.com/orgs/codrops/repos?sort=created (2024-04 以降 18 repo を列挙)
- https://github.com/codrops/ScrollBlurTypography (js/effect-1/blurScrollEffect.js) · 記事 https://tympanus.net/codrops/?p=76992
- https://github.com/codrops/OnScrollTextHighlight (js/effect-1, effect-5, css/base.css) · https://tympanus.net/codrops/?p=76961
- https://github.com/codrops/OnScrollLayoutFormations (js/index.js, css/base.css)
- https://github.com/codrops/OneElementScroll (js/index.js) · https://tympanus.net/codrops/?p=82884
- https://github.com/codrops/ElasticGridScroll (js/demo1/index.js) · https://tympanus.net/codrops/?p=94826
- https://github.com/codrops/RepeatingImageTransition (js/index.js) · https://tympanus.net/codrops/?p=92571
- https://github.com/codrops/OnScrollSVGFilterText (js/filter2.js, index.html) · https://tympanus.net/codrops/?p=80141
- https://github.com/codrops/LineTextHoverAnimations (js/effect-1/text-animator.js) · https://tympanus.net/codrops/?p=78645
- https://github.com/codrops/EaseReverseClipMenu (js/index.js) · https://tympanus.net/codrops/?p=114731
- https://github.com/codrops/Staggered3DGridAnimations · https://github.com/codrops/ScrollTextMotion · https://github.com/codrops/ContextAwareLogoAnimationScroll

---

