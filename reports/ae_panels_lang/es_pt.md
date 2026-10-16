# AE のスクリプト・パネル: スペイン語圏とポルトガル語圏(とくにブラジル)

調べた日: 2026-10-17。ウェブだけ、読むだけ。サインインなし、ダウンロードなし。所要 約25分。

## まとめ(先に)

- 一番大きい「現地で作られた道具」はブラジルの **Brazu**。2021年創業(サンパウロ州バルエリ)、5つの道具を1つのパネルにまとめて R$549 の買い切り。利用者は1万人超とのこと。
- 現地の作り手が何度も作り直している問題のトップは **「プロジェクトパネルの散らかり」**。フォルダを自動で作って中身を振り分けるスクリプトが、別々の5人から出ている。
- 次に多いのが **文字アニメーション**、**字幕**、**素材の収集と軽量化**、**拡張機能のインストール**。
- 嫌われている点として名前が挙がったのは、AE の文字アニメーター(範囲セレクター)、グラフエディター、ドル建てで英語だけの海外製プラグイン。
- パネルの絵(スクリーンショット)は見られなかった。テキストしか読めない取り方だったため。パネルの見た目は、文章に書かれていた分だけ載せている。

## 1. 言葉のリスト(検索候補・題名・機能名から)

ポルトガル語(ブラジル)
- scripts after effects / scripts para after effects / scripts after effects grátis / download / mac
- melhores scripts de After Effects / scripts essenciais / scripts (muito) úteis / scripts mais insanos
- plugins after effects grátis / plugins indispensáveis / plugins extremamente úteis
- extensão after effects / instalar extensão / adicionar extensão
- painel (ScriptUI) / painel autoral / ferramentas / workflow / acelerar seu workflow
- pack motion after effects / motion bro pack / pack de textos animados
- legendas no After Effects / animação de texto / organizar pastas
- (候補に出た道具名)horse(= Mister Horse の Animation Composer)、deep glow、flow、motion bro、worley

スペイン語
- scripts para After Effects / scripts after effects gratis / instalar scripts
- mejores scripts / imprescindibles / que no te pueden faltar / para trabajar más rápido
- plugins after effects gratis / plugins after effects texto / caballo(= Mister Horse)/ saber / particular
- extensiones after effects / extensiones gratis / instalar extensiones
- herramientas / flujo de trabajo / organizar proyectos / panel
- (アルゼンチン口調)"Usá Motion Tools y aumentá tu velocidad"

傾向: どちらの言語でも、検索候補の最初は「無料」と「インストールのしかた」。道具の名前は英語のまま使われる。

## 2. よく挙がる道具の回数(現地語で読めた情報源だけ)

数えた情報源(12): ① Yogo Motion(pt) ② Alura・AVMakers の講座(pt) ③ Motion Design Brasil フォーラム(pt、検索結果の抜粋だけ) ④ Filmora BR(pt) ⑤ Crehana「5 plugins」(es) ⑥ Crehana のほかの記事(es) ⑦ Domestika の Zenzuke 講座の説明(es) ⑧ AEdicas(pt) ⑨ André Sarti(pt、2014) ⑩ Vidmob(pt、2022) ⑪ 2DeadFrog の文字アニメーション記事(pt) ⑫ aleRENDER(es、アルゼンチン)。検索候補(P)は別枠にした。

| 道具 | 回数 | 出てきた情報源 | 何をするか |
|---|---|---|---|
| Animation Composer(Mister Horse) | 5 + P | ④⑤⑥⑦⑪ + 候補 "horse/caballo" | 無料のプリセット置き場 |
| Duik | 5 | ①②④⑦⑫ | キャラクターのリグ、無料 |
| Decompose Text / TextExploder | 3 | ③⑪⑫ | 文字を1字ずつのレイヤーに分ける |
| Motion Tools | 2 + α | ③(+ブラジルの代理販売、アルゼンチンの YouTube) | イージングとアンカーのボタン群 |
| FX Console | 2 | ③⑧ | 効果をすばやく呼び出す |
| Ease and Wizz | 2 | ⑨⑫ | イージングのエクスプレッション |
| Chameleon(Mamoworld) | 2 | ⑨⑫ | 配色パレットを適用 |
| RubberHose | 2 | ③⑫ | ゴムホース状の腕のリグ |
| Motion Bro | 1 + P | ⑥ + 候補 | プリセットパック |
| GIF 書き出し(GifGun / GifExporter) | 2 | ⑧⑩ | AE だけでは GIF を書き出せない |
| AEJuice | 1 | ⑪ | プリセットパック |
| Newton(+同じ作者の Paravent) | 2 | ⑧⑫ | 2D の物理 |
| Overlord, Kbar, Bodymovin/Lottie | 各1 | ③ | Illustrator→AE / ボタン列 / Lottie 書き出し |
| Plexus, Lenscare, AfterCodecs | 各1 | ⑤ | 線と点 / 被写界深度 / Windows で ProRes |
| Squash & Stretch, Type Builder, typeGEMs ほか文字用15種 | 各1 | ⑪ | 文字アニメーション |
| Deep Glow, Saber, Particular, Flow | 候補のみ | P | 光、パーティクル、イージング |
| **「プロジェクト整理」スクリプト(現地製)** | **5件** | 下の表 | 決まったフォルダを作って中身を振り分ける |

注意: 現地語の記事の多くは英語圏の上位リストを訳しただけ。英語圏と違うのは「無料」「インストール」をとても重く見る点と、現地製の整理スクリプトが多い点。

## 3. 現地で作られた道具

ブラジル(ポルトガル語)

| 名前 | 作者 | 解決すること | 価格 | リンク |
|---|---|---|---|---|
| **Brazu**(Copilot / Tools / Keyframe / Parallax / Presets) | Brazu(2021年創業、SP州バルエリ、Adobe パートナー) | AI で指示から動きを作る / 50種以上の動きをワンクリック / **グラフエディターを使わずに動きをなめらかにする** / 3D の視差 / 編集できるプリセット | R$549 買い切り(3回払い可) | brazu.io |
| Simple Text | 2DeadFrog(作者は "Jack") | AE の文字アニメーターの代わりに、シェイプレイヤーのコントローラーで文字を動かす。遅れ方は 通常/逆/ランダム | US$29 | 2deadfrog.com |
| Legendas(字幕)スクリプト | 2DeadFrog | AE で字幕を作る | 無料、または好きな金額 | 2deadfrog.gumroad.com |
| Melhores Scripts #1 | Guia MD Motion(Amadeus) | 他人の無料スクリプトをまとめたパック(作ったのではなく選んだもの) | 無料 | guiamdmotion.gumroad.com |
| XFlow | Aldox Motion(Hotmart で販売、国は未確認) | 整理 / 未使用を消す / ファイル収集 / 長さ・サイズ・fps 一括 / キャッシュ消去 | 不明 | hotmart.com |
| Itaú-Mosys | brunojorri(GitHub) | 銀行(Itaú)の社内パネル。Dropbox の「Motion System」素材庫に直結 | 社内用 | github.com/brunojorri |
| fast-comp-collect | brunojorri | コンポを、それだけで完結する小さな複製にする(CEP) | 公開 | 同上 |
| ae-silence-cutter-cep | luigypedro22 | AE の中で無音部分を自動でカット | 公開 | GitHub |
| af-painel-buni | bunihayashi | 編集の組み立てを自動にする ScriptUI パネル | 公開 | GitHub |
| ae-organizator / initial-folders-template | andrepires / jonataswilliam | 決まったフォルダの自動作成と振り分け | 公開 | GitHub |
| script-launcher-modules | iocarry | スクリプトランチャーの公開モジュール(pt) | 公開 | GitHub |
| Editor Black Belt Tools PRO | jhonatangtw | Premiere と AE 共通の CEP 道具箱 | 不明 | GitHub |
| (注文制作)| Zago(Fiverr)、Gabriel Carreira(TabNews) | 頼まれた仕事の自動化。Carreira はフリーランス案件が多すぎて作り始めた | 依頼制 | — |

スペイン語圏

| 名前 | 作者 | 解決すること | 価格 | リンク |
|---|---|---|---|---|
| OrganizarProyecto.jsx v2.0 | Gastón Bruno(スペイン、.es) | COMPS / PRECOMPS / VIDEO / IMAGENES / AUDIO / SOLIDOS / OTROS を作って振り分ける。名前が PRE_ で始まるものはプリコンプ扱い。作るフォルダの選択と名前変更ができ、Ctrl+Z で元に戻せる | 無料(受け取りにメールアドレスが必要) | gastonbruno.es |
| Platzi-Composer-Pro | DanielGutierrezB | 教育会社 Platzi(コロンビア創業)の社内向け CEP。動きと作業効率の道具 | 社内用 | GitHub |
| simbaqueva の小さな JSX 群 | simbaqueva(es) | 電光掲示板の流れる文字 / ライトリーク / Trim Paths で順に見せる / 写真スライダー / 強いグリッチ / PNG の連番からストップモーション | 公開 | GitHub |
| Cache-Pro | Animateoo | 動作を軽くする道具を1つにまとめたもの | 公開 | GitHub |
| Viegas Installer | viegasestudio | ZXP/CCX 拡張のインストールと更新 | 公開 | GitHub |
| Optimization Tools | Fede(fedeespana、国は未確認) | 決まった作業の繰り返しを自動化 | 不明 | gumroad |
| (YouTube)| Harold Silva | コピーして貼るだけのモーション用エクスプレッション | 無料 | YouTube |
| Eje in Motion(10種) | 国は未確認(名前の "eje" はスペイン語で「軸」) | Power Shapes(無料)、QuickCaption(.srt 字幕、無料)、Power Loop(エクスプレッションなしでループ、無料)、Snake Paths(1本の線に複数の色、無料)、Quick Animator(US$20)、Motion Lines(US$10)、Easy Taper(US$10)、SoundBox(効果音90種以上、US$20) | 一部無料 | ejeinmotion.com |

作り手ではなく、教える側・選ぶ側: Gabriel Suchowolski(microbians、スペイン、Domestika のエクスプレッション講座)、Zenzuke(Carlos Albarrán、Domestika、講座にスクリプトの章あり)、aleRENDER(アルゼンチンの紹介ブログ)、AEdicas(Adriano Dorow Darosi、ブラジル)。

除外したもの: GitHub の「adobe-after-effects-mk20」などは中身が Vegas Pro の宣伝文で、マルウェアの誘い込みに見えるので外した。

## 4. AE に足りないと言われていること

1. **プロジェクトパネルが散らかる** → 整理スクリプトが5件(ブラジル3、スペイン1、Hotmart 1)。いちばん多く作り直されている。
2. **文字アニメーターが分かりにくい**(2DeadFrog の記事より): 範囲セレクターと Start/End で決まるイージングが分からない。数値スライダーばかり。文字の通り道を曲線で描けない。文言を変えると作り直しになる。文字ごとに違う動きを混ぜるのが大変。ランダムさが出しにくい。文字・単語・行を分けて操りにくい。
3. **グラフエディターが難しい**: Brazu Keyframe は「グラフを使わずになめらかにする」を売りにしている。
4. **海外製はドル建てで英語だけ**: Brazu は「ポルトガル語のスクリプト」から始まり、現地通貨で買える値段を掲げている。
5. **字幕と音声**: 字幕スクリプト(2DeadFrog、QuickCaption)と無音カット。もともと AE は字幕を入れる道具ではない。
6. **納品と軽量化**: ファイル収集、未使用の削除、キャッシュ消去、コンポを単体で切り出す(XFlow、fast-comp-collect)。
7. **書き出しの穴**: GIF が出せない(GifGun / GifExporter)。Windows で ProRes が出せない(QuickTime 廃止。AfterCodecs で補う)。
8. **インストールが難しい**: 検索候補の上位が「instalar」。インストーラー(Viegas)まで作られている。
9. **会社のブランド素材と直結したい**: Itaú と Platzi が社内パネルを自前で作っている。
10. **VFX の作業に繰り返しが多い**(TabNews のコメントより)。

## 5. パネルの見た目(文章に書かれていた分だけ)

- Simple Text: 文字レイヤーを選んで「Simple Text」ボタンを1回押し、遅れ方を3つから選ぶだけ。
- OrganizarProyecto: フォルダごとのチェックと名前の入力欄。押すのは1回。
- XFlow: 5つの機能ボタン(整理・削除・収集・設定・キャッシュ)。
- Brazu: 5つの道具を1つのパネルに収める。指示を書く欄(Copilot)、50種以上のワンクリックの動き、グラフの代わりになる調整。
- Eje Quick Animator: **向きを選ぶ9つの矢印**(3×3の並び)と、入りと出のカーブ。文字を使わない操作の例として Motolii の参考になる。
- Easy Taper: スライダーで線の太さを先細りにする。
- 全体に、「1回押せば終わる」ボタンを並べたパネルが中心。Ableton のように結果の絵で語る操作は見つからなかった(ただし画像は見ていない)。

## 6. 読めた情報源 / 読めなかった情報源

読めた: brazu.io、adobevideopartner.com/partners/brazu、2deadfrog.com(文字アニメーション記事、Simple Text)、2DeadFrog・Guia MD・fedeespana の Gumroad(説明文の冒頭だけ)、gastonbruno.es、aedicas.com/category/scripts、andresarti.com.br、vidblog.vidmob.com、alerenderscripts.blogspot.com、ejeinmotion.com、hotmart(XFlow)、renderu.com/pt(AFPCollection。作者 Anthony Possobon の国は未確認)、tabnews.com.br(Gabriel Carreira)、crehana.com(plugins 記事)、GitHub の公開検索 API、Google の検索候補(pt-BR / es)、YouTube の題名(検索結果だけ)。

読めなかった:
- motiondesignbr.com.br: 証明書切れのうえ、アカウント停止中。検索結果の抜粋だけ使った。
- yogomotion.com.br: ドメインが見つからない。抜粋だけ使った。
- mxcursos.com: 403。layerlemonade.com: 404。
- aescripts.com の作者ページ: 403。作者の国籍は確かめられなかった(Rodrigo Domínguez などスペイン語の名前はあるが、確認できていない)。
- DuckDuckGo: ボット確認が出たので、突破せずに止めた。Bing: ボットには結果を返さなかった。
- Gumroad と Hotmart の本文(JavaScript で描かれていて、価格と画面が読めない)、YouTube の動画の中身、Domestika の講座の中身(ログインが必要)、Instagram。
- **パネルのスクリーンショットは1枚も見ていない。**
