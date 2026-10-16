# 韓国語圏の AE スクリプト・パネル（2026-10-17 調べ）

読み取りだけ。サインインなし。ブラウザ1タブ（Bing 検索経由）と WebFetch。
Naver（ブログ・カフェ・検索）は道具側で全部閉じていたので、Naver 内の記事は題名と抜粋しか読めていない。

## 1. 呼び名（検索候補・記事題名・動画題名から）

- ソフト名: 애프터이펙트 / 에프터이펙트（つづり違い、両方多い）/ 애펙・에펙（略）/ AE
- 道具の総称: 스크립트, 플러그인, 확장 프로그램（이지플로우の店）, 확장 플러그인, 스크립트 패널, 스크립트 플러그인（両方を混ぜて呼ぶ）, 프리셋, 툴
- 推し言葉: 추천, 필수, 무조건 소장해야 할, 꿀 플러그인, 꿀팁, 끝판왕, 역대급, 치트 키, 작업속도 200%, 유료급 퀄리티, BEST 20, TOP3
- 配り言葉: 무료 배포, 한시적 무료배포, 직접 제작한, 소스 공유, 무료소스
- 韓国特有: 한글 타자 효과 / 한국어 타이핑 효과, 자음 모음 분리, 자소 분리, 한글 입력 오류, 한글판 / 영문판, 한글 깨짐
- 2026年の新語: AI 에펙 스크립트, (Claude・Manus で) 직접 만든 스크립트, 자동화
- 道具名の韓国名: 이지플로우, 이지에딧, 이지에프엑스, 한글타자, 디반장스크립트

## 2. まとめ記事での登場回数

数えた一覧（韓国語の記事のみ、本文まで読めたもの）:
A = markppark.tistory.com/113（BEST 20）、B = editcopilot.tistory.com/8、C = inmare.github.io（無料プラグイン集、35件）、D = rgy0409.tistory.com の AE 記事一覧、E = Shutterstock 韓国語版（翻訳記事、抜粋のみ）。
「動画」= Bing に出た韓国語動画の題名・説明。

| 道具 | 回数 | どこで |
|---|---|---|
| Saber（Video Copilot） | 5 | A B C D E |
| FX Console（効果を検索窓で呼ぶ） | 4 + 動画 | A B C E、動画「설치 전과 후로 달라집니다」 |
| Displacer Pro | 4 | A B C E |
| AEJuice 無料品（GIF 等） | 4 | A B C E |
| Duik（Bassel / Angela） | 3 + dcinside | A B C |
| ButtCapper（線の端を丸く） | 3 | A C D |
| Orb | 3 | B C E |
| Quick Chromatic Aberration | 3 | B C E |
| Decompose Text（文字を1字ずつ分ける） | 2 + 動画 | A D |
| Animation Composer | 2 + 動画 | B D |
| Ease Copy | 2 | A C |
| アンカーポイント移動系 | 2 | A D |
| Dojo Glitch / GridRig / Color Vibrance / LIMBER | 各2 | B E |
| Motion Tools（Classic / Pro） | 2 | C、Naver editor_monkey |
| F's Plugins（日本製） | 1 + 動画 | C |
| Motion Bro / BadFX / Real Glow / 3D Stroke | 動画のみ | |
| 1回だけ | | Shape Repeater Baker, RD Composition Setter, Bendy Nulls, True Comp Duplicator, Flow, LoopMaker, Deep Glow, Repeater, Thicc Stroke, BaraMoji（日本製）, LayerNode, Utility Box, Randomizer ほか |

一言: 韓国のまとめ記事に並ぶのは、ほぼ英語圏の定番の無料品。効果を足すもの（光・ズレ・グリッチ）と、手間を省く小物（検索・アンカー・線端・イーズ）が半々。
韓国製の道具はまとめ記事にほとんど出てこない。韓国製は YouTube 動画と個人ブログで配られている。

## 3. 韓国の作り手の道具

| 名前 | 作り手 | 何を解くか | 無料/有料 | 場所 |
|---|---|---|---|---|
| 이지플로우 Easy Flow | 모셔니스트 코리아（会社） | キーフレームを打たずに、クリックだけで動きを付ける。8方向の移動・拡大・XYZ回転・不透明度、組み合わせ、よく使う動きのプリセット化、クリックで動きの長さを変える、他の属性ともつなぐ | 有料 49,000원（定価 59,000）、オンラインでライセンス確認、CC2019以降、購入評価 35件 4.8 | motionist.kr/?idx=91 |
| 이지에딧 EZ EDIT / 이지에프엑스 EZ FX | 同上 | 中身は未読（店の一覧で AE 用と確認） | 有料 49,000원 / 39,000원 | motionist.kr/?idx=73, 71 |
| 한글타자 | Pindang2（個人） | 韓国語のタイピング演出。字を子音・母音に分けて、1つずつ打たれるキーフレームを作る。間隔と速さの揺れも指定できる | 商用ライセンス、値段は書いていない、ベータ | scripts.pindang2.com |
| Additional Transform / Mirror Positions / Auto Textbox / JType | Pindang2 | 説明なし（JType は日本語のタイピング演出、Web） | 不明 | scripts.pindang2.com |
| Text to separated solids（한글 자음모음 분리） | 향햐리향（STUDIO MIMID / HYENA 名義も） | 文字を子音・母音ごとの平面＋マスクに分けて、パーツごとに動かせるようにする。ㅇㅁㅂ のような閉じた字は塗りつぶされ、手で直す必要あり | 期間限定の無料（あとで有料予定） | hyanghya.tistory.com/4、rgy0409.tistory.com/5300 |
| DBJ 디반장 스크립트 | 도비의정석（YouTube） | 韓国語タイピング演出を一発で（動画題名より） | 不明 | YouTube「애펙 한국어타이핑 효과 원샷원킬」 |
| Korean Typewriter Ver.2 | 不明（Behance） | 韓国語タイプライター | 不明 | Behance（未到達） |
| 円柱・らせんスクリプト | 디자이너 덕디 DeokDi（YouTube の現役モーションデザイナー） | レイヤーを好きな数に切って3D空間で円柱・らせんに並べる（AE のレイヤーは曲げられない） | 無料 | YouTube 2025-09、Naver MyBox で配布 |
| プロジェクト整理スクリプト | 同上 | 大きくなったプロジェクトの素材・コンプを整理。Manus AI で作り直した版 | 配布（値段の記載なし） | deokdi.tistory.com/33 |
| SRT からライブチャット欄・バラエティ字幕を作る道具 | 同上 | SRT 字幕から、配信のチャット欄風の画面と強調字幕を自動で作る。ブラウザで動き、汎用の LLM で一度だけ JSON を作って AE に読み込む | 無料（再配布・改変配布は禁止） | deokdi.tistory.com/34 |
| SRT 字幕自動化（「字幕作業10倍速」） | 同上 | AE には字幕機能がないので SRT から字幕レイヤーを作る | 動画のみ確認 | YouTube |
| before & after スクリプト / サイズ違い書き出しスクリプト | AI각효과（YouTube、広告の後処理の現役） | 前後比較を一発で作る。縦横比違いの版を一度に作る | 無料 | YouTube 2026-07 |
| Un-Precompose | 프로에펙러 Pro AE（YouTube） | プリコンポーズを解く（AE には「戻す」がない） | 無料 | YouTube 2026-05 |
| XdobeFixSwitch / XdobeFixGUI | mijien0179（GitHub） | 韓国語版 AE でプリセット名の韓国語が文字化けするのを直す。プリセット名を韓国語と英語で切り替える | 無料 | github.com/mijien0179 |
| after-effects-scripts-korean | devuterian（GitHub） | Kyle Martinez のスクリプト集の韓国語訳（自作ではない） | 無料 | github.com/devuterian/after-effects-scripts-korean |

傾向:
- いちばん多いのは**韓国語の文字の問題**（タイピング演出、子音・母音分解）。同じ問題を少なくとも4組が別々に解いている。
- 2025〜26年は、現役デザイナーの YouTuber が **AI に書かせた一つの用途だけのスクリプト**を動画と一緒に無料で配る形が増えた（덕디、AI각효과、프로에펙러）。配り先は Naver MyBox、作品ファイルは marpple で販売。
- 会社として売っているのは 모셔니스트 코리아 くらい（AE 用3本、Premiere 用2本）。

## 4. 「AE にない」と言われていること

1. 韓国語のタイピング演出ができない。AE の文字アニメーターは1字（音節ブロック）単位で、子音・母音で組み立てる打ち方にならない。「영어는 되는데, 한글은 안되는게 참 많습니다」（Naver sukkeychung、抜粋）。
2. 韓国語の字を子音・母音のパーツに分けて動かせない。
3. 文字入力で韓国語の入力が壊れる。消すと2字消える、改行すると2回入る。何年も直らないという声が Adobe 公式掲示板にある（25.5版、2025-10）。
4. 韓国語版と英語版の違い。プリセット名が文字化けする。式やチュートリアルが英語版前提なので、表示言語を英語に切り替える手順の記事・動画が多い。
5. 字幕機能がない（Premiere にはある）。SRT から字幕を作る道具が出てくる理由。
6. 簡単な動きでもキーフレームが面倒（Easy Flow の売り文句は「키프레임 없이」）。
7. プリコンポーズを戻せない。プロジェクトを整理する機能がない。
8. 縦横比違いの版を作り直すのが手作業（RD Composition Setter、サイズ違い書き出しスクリプト、コンプ解像度スクリプト）。
9. レイヤーを曲げられない（円柱・らせん）。
10. 標準効果の物足りなさ（光る効果、ずらし、3D、色、リピーターを1つずつ動かせない、線の端）。まとめ記事が暗に言っている。
11. 効果パネルが重く、探しにくい（FX Console が人気の理由）。
12. 入れるまでの手間。「ファイルの書き込みとネットワークへのアクセスを許可」の設定、ZXP の署名で失敗する、オンラインでのライセンス確認。

## 5. パネルの見た目

- **Easy Flow**（画像を見た）: ビューアの下に置く**横一列の細長い帯**。暗い色の四角いボタンが並び、記号はミントグリーン（8方向の矢印＝移動、四隅に向かう矢印＝拡大、丸い矢印＝回転、だんだん薄くなる横線＝不透明度）。追加は紫の小さな「＋」バッジ。文字はほとんどない。**記号そのものが動きの結果を表している**。
- **한글타자**（文の説明から）: ScriptUI の小さな窓。文字の入力欄、キーフレーム間隔の数値、速さの揺れ 0〜100%、「만들기」ボタン、進み具合のバー。文字の多いフォーム。
- **子音・母音分解**: 「Generate!」ボタン1つと顔文字、作者名、版だけ。最小限。
- **덕디の SRT 道具**: AE のパネルではなくブラウザで動く。結果は JSON で AE に読み込む。
- **FX Console**: 検索窓が浮かぶ形（文字で選ぶ）。
- **Motion Tools Pro**: ボタンの並びを自分で変えられる格子（Naver editor_monkey の紹介）。
- 店や動画の見せ方は「before / after」と結果の動画が中心で、パネル自体の画面写真は少ない。曲線エディタ、色見本、効果の縮小見本が並ぶ表は、韓国製では見つからなかった。

## 6. 読めた所

- https://github.com/devuterian/after-effects-scripts-korean
- https://github.com/search?q=애프터이펙트&type=repositories（mijien0179 の2件）
- https://deokdi.tistory.com 、/33 、/34
- https://hyanghya.tistory.com/4
- https://rgy0409.tistory.com/category/Programs/AfterEffect 、/5300
- https://scripts.pindang2.com と 한글타자 のページ
- https://markppark.tistory.com/113
- https://editcopilot.tistory.com/8
- https://inmare.github.io/2025/09/05/After-Effects-무료-플러그인-모음/
- https://motionist.kr 、/?idx=91（商品画像5枚を見た）
- Bing 韓国語検索 約15回（題名・抜粋・動画題名）
- YouTube 検索（このタブでは2件しか表示されず、ほぼ使えなかった）

## 7. 読めなかった所

- Naver ブログ・カフェ・検索・知識iN: 道具側で全部閉じていた。題名と抜粋だけ（「무조건 소장해야 할 필수 무료 스크립트/플러그인」dhsh11、「[모션그래픽학원] TOP3」motionisart529、sukkeychung のタイピング記事、editor_monkey の Motion Tools Pro、Utility Box）。
- motionlab.co.kr（403）、Behance（403）、dcinside（URL を取れず）、Clien（検索の途中でページ移動が拒否された）
- 크몽・클래스101: AE スクリプトの出品は見つからず（크몽はマクロ一般の受注のみ）
- motionist の EZ EDIT / EZ FX の中身、Pindang2 の Additional Transform の中身、DBJ 디반장 の中身
- 推奨キーワードの自動候補（Bing の候補 API は入力語をそのまま返すだけだった）
