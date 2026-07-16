# 先例調査: すぐに受け入れられたUI(2026-07-16)

ステータス: **仮説メモ**(受容側の先例集 — **設計根拠ではない**。同日のレビュー3巡を反映: ①「全事例が既存ユーザー基盤/界隈への着地であり不確定変数が多すぎる」→第一部の結論を格下げ(「参照クラスの偏り」節)、②「政治でなく根本的なUX/UIの話」→**第二部(操作パターン単位の収斂語彙とUX原理)を追補**、③「Abletonの答えは操作の直感性。MotoliiはAEのカウンターとしてそうありたい」→**第三部(後発の勝ち筋の分解)を追補**。文末の改訂記録)。M3は[基盤再締結ゲート](2026-07-15-m2-foundation-reclosure-gate.md)発効中の製品実装停止であり、本文書はタスク・完了条件・契約を一切変更しない。転移候補は「M3入場PRの再翻訳時に個別採択する仮説」として台帳化するのみ。

運用規律: [成功先例調査](2026-07-12-success-prior-art.md)と同じ基準 — 出典URLの無い逸話を設計根拠にしない。加えて本調査固有の制約として、**取得環境のプロキシ制限(HTTP 403)により原文全文を照合できなかった出典が多数ある**。等級とは別に「原文未照合(スニペット経由)」を都度明記し、設計根拠へ昇格させる前の原文照合を必須とする。

## これは何か

M3の[実装ガード11項](../specs/M3-ui-integration.md)は全て**失敗回避**(出荷済みエディタの苦情・死因)から抽出した。本メモはその対 — 「出た直後にユーザーへ受け入れられたUI」を集め、受容がどのUI決定に帰属されているかを証拠の強さつきで読む試み。

**三部構成(2026-07-16 同日追補)**: 第一部=プロダクト単位の受容事例(市場での受容 — 交絡が大きく、用途は界隈の期待チェックリストまで)。**第二部=操作パターン単位の受容**(業界横断で収斂した操作語彙と、それを支えるUX原理の一次資料)。操作単位の収斂は「どの製品が市場で成功したか」に依存しない受容証拠 — 数十年・数十ツールが同じ操作を再採用し続けている事実そのものが「ユーザーが訓練済みでゼロ学習」の証拠になる — ため、**M3設計への転移はこちらが本線**。**第三部=後発の勝ち筋**(Abletonを先例に「どの操作も直感的」を検証可能な語彙へ分解し、AEの間接性との対比表にする)。

**前提の注意(全事例共通)**: 「受け入れられた」は UI 単独の効果ではない。無料・流通チャネル・時代要因(リモートワーク、サブスク反発、TikTok流入)との交絡が全事例にあり、本文で都度申告する。**UI単独の因果はどの事例でも分離できていない** — 最もクリーンなBlender 2.80(価格ゼロ不変のままUI刷新直後に企業支援が集中)ですらEeveeレンダラとの交絡が残る。

**最大の交絡(レビュー指摘で昇格)**: 全事例が**既存のユーザー基盤または既存界隈への着地**である。即時性の強い証拠を持つ事例ほどこの交絡が強く、「即時受容」はUI設計の現象ではなく**待機需要の現象**である可能性が高い。詳細は「参照クラスの偏り」節。本メモの結論として使えるのは「対象界隈が受容/拒絶を語るときにどのUI決定を挙げるか」という**語彙の台帳(期待チェックリスト)まで**であり、「このUI決定をすれば受け入れられる」ではない。

## 証拠等級

- **A** = 公式一次資料・学術・大手報道の具体的数値/事実
- **B** = 開発元・当事者の自己申告(公式ブログ・プレス)
- **C** = フォーラム・レビュー・コミュニティの定性評価
- **未確認** = 出典に到達できない/二次情報のみ。設計判断に使わない

## 要約表

| 事例 | 受容の最良証拠 | 受容が帰属されたUI決定 | 主な交絡 |
|---|---|---|---|
| AviUtl2 (2025-07) | 公開翌朝Xトレンド1位・大手一斉報道 [A] | 64bit化・軽さ継承・シングルウィンドウ統合・既存操作の継承・SDK同時公開 | 8年待望の後継という文脈 |
| VOICEVOX (2021-08) | 配布インフラ飽和→窓の杜が代理配布 [A] | 「丁度よい」デフォルト+深い調整の二層、キャラ選択UI | 無料・商用可・キャラIP |
| CapCut (2020〜) | 2022年世界DL4位・3.57億DL [A] | テンプレート駆動(学習曲線の除去)、ワンタップ書き出し | TikTok流入(分離不能) |
| FCPX (2011)【両面】 | 拒絶署名3,700筆 [A] / 後年200万→250万ユーザー [A] | マグネティックタイムライン: 新規層は受容・既存プロ層は即時拒絶 | 回復に約6年 |
| Blender 2.80 (2019) | 直後にEpic $1.2M・Ubisoft・NVIDIA支援 [A] | 左クリック選択デフォルト化(業界標準への一点降伏)、ワークスペースタブ | Eevee同時搭載 |
| VS Code (2015) | SO調査 34.9%(2018)→50.7%(2019)で首位 [A] | 軽量・高速起動、拡張エコシステム | 無料・MS・OSS |
| Figma (2016) | UX Tools調査 7%(2017)→66%(2020) [A] | URL共有・マルチプレイヤー(ファイル管理の消滅) | 無料枠・コロナ禍・Sketch=Mac専用 |
| Flow (2016, AEプラグイン) | 定量なし(業界レビュー遍在) [C] | 正規化1本カーブ+CSS cubic-bezier相乗り+プリセット視覚ライブラリ | 採用規模の一次資料なし |
| LosslessCut (2016) | GitHub 42.1k stars(2026-07実測) [A] | 単機能特化(開く→イン/アウト→書き出しのみ) | ユーティリティであり NLE ではない |
| YMM4 | 界隈の定番扱い+個人作としては異例の報道頻度 [C] | テキスト主導(セリフ入力=編集)、1本で完結、VOICEVOX連携を数日で追加 | 定量一次資料なし |
| Resolve Cut page (2019)【非受容の対照】 | 既存ユーザー層はほぼ無視 [C] | 「同一製品内の別ページ」でも既存層には届かない | — |

## 各事例の詳細

### 1. AviUtl ExEdit2(2025-07-07公開 → 2026-07-07正式版)

対象層がMotoliiと最も重なる事例。KENくん氏が約6年ぶりに公開した後継で、本体と拡張編集を統合しゼロから再構築。

- **受容の証拠 [A]**: 公開翌朝の2025-07-08に「Aviutl2」がXトレンド1位([XenoSpectrum](https://xenospectrum.com/aviutl2-updated-for-the-first-time-in-6-years/)、[ニコニコニュース公式X](https://x.com/nico_nico_news/status/1942366791956865122))。[GAME Watch](https://game.watch.impress.co.jp/docs/news/2029406.html)・[GIGAZINE](https://gigazine.net/gsc_news/en/20250708-aviutl-exedit2-beta1/)・[電ファミ](https://news.denfaminicogamer.jp/news/250708f)・[AUTOMATON](https://automaton-media.com/articles/columnjp/aviutl-20250709-348554/)が即日〜数日で一斉報道。約1年・50超のβを経て正式版化([窓の杜](https://forest.watch.impress.co.jp/docs/news/2123628.html)、原文未照合)
- **帰属されたUI決定**: (a)64bit化(積年の一点不満の解消。関連ハッシュタグがトレンド2位)、(b)軽さの継承 — ZIP約2.2MBという配布サイズ自体が話題化、(c)マルチウィンドウ→シングルウィンドウ統合 —「フィルターの個別設定ウインドウなどもメインウインドウに統合されて行方不明にならなくて良い」([個人レビュー](https://blackbird-blog.com/aviutl2-release-review) [C])、(d)既存操作体系の継承+ショートカットカスタマイズ([解説サイト](https://vip-jikkyo.net/aviutl2-tutorial) [C])、(e)プラグインSDK同時公開=コミュニティの共同開発者化
- **反証・留保**: 旧プラグイン/スクリプトと非互換のため資産を持つヘビーユーザーほど移行が遅い [C]。純正UIの配色への不満から有志の[ダークモード化MOD](https://github.com/hebiiro/al2_jd)が登場。DL数の公式数値は未確認

### 2. VOICEVOX(2021-08-01公開)

- **受容の証拠 [A]**: 公開直後にGoogle Driveの配布上限へ到達し続け、**窓の杜が作者の許可を得て一時的に代理配布**する異例の措置([窓の杜](https://forest.watch.impress.co.jp/docs/news/1341517.html))。[PC Watch](https://pc.watch.impress.co.jp/docs/news/1341708.html)・[GIGAZINE](https://gigazine.net/gsc_news/en/20210802-voicevox/)・[DTMステーション](https://www.dtmstation.com/archives/41014.html)が数日で報道。ずんだもんが2022年「ネット流行語100」入賞、行政動画での使用を開発者が報告([X](https://x.com/hiho_karuta/status/1584910652346630144) [B])
- **帰属されたUI決定**: 窓の杜(第三者)の記事タイトルが核心 —「素人でも手軽に満足のいく品質が得られる**丁度よさが魅力**」。デフォルトで十分な品質を返し、アクセント・イントネーションの深い調整は求める人だけが触る二層構造。キャラクター(立ち絵つき話者)選択UI
- **反証・留保**: 無料・商用可のライセンス設計とキャラIPの交絡が大きい。累計DL数の一次数値は未確認。約3.26GBの配布サイズは配布設計としては失敗でもある(飽和はその裏返し)

### 3. CapCut(国際版2020-04〜)

- **受容の証拠 [A]**: 2022年世界DL第4位・3.57億DL(TikTok/Instagram/WhatsAppに次ぐ。[Forbes/Apptopia](https://www.forbes.com/sites/johnkoetsier/2023/01/04/top-10-most-downloaded-apps-of-2022-facebook-down-spotify-up-tiktok-stable-capcut-keeps-growing/))。累計14億DL等の大きい数字は集計サイト由来 [B〜C]
- **帰属されたUI決定**: テンプレート駆動編集(完成プロジェクトに素材を差し替えるだけ=学習曲線そのものの除去)、縦型ネイティブ、自動キャプション、ビートシンク、TikTokへのワンタップ書き出し。帰属は主に第三者のマーケティング分析
- **反証・留保**: **TikTokという巨大流入源とUIの効果が分離不能**。2024→2025年はDL前年比-18%。プロ向け機能の深さは評価されていない

### 4. FCPX マグネティックタイムライン(2011)【受容と拒絶の両面】

M3実装ガード5(革新は必ずオプトイン)の根拠事例を、受容側からも読み直す。

- **拒絶の証拠 [A]**: 発売直後に旧版復活署名が3,700筆超([Variety](https://variety.com/2011/digital/news/final-cut-pro-update-draws-backlash-1118039277/)、原文未照合)。プロ層は「改良されたiMovie」と見なした([AppleInsider回顧](https://appleinsider.com/articles/25/12/19/inside-final-cut-pro----apples-superb-video-editing-suite-and-a-huge-mistake))
- **受容の証拠 [A]**: Apple公式発表で2017年に200万ユーザー、2018年に250万超。「100万→200万は最初の100万よりはるかに速かった」([9to5Mac](https://9to5mac.com/2017/04/26/final-cut-pro-x-sales/)、[AppleInsider](https://appleinsider.com/articles/17/04/26/final-cut-pro-x-now-has-over-2-million-users-apple-says))。新規層・YouTuber層が担い手(間接証拠)
- **読み方**: 「プロ層に拒絶され新規層に受容された」仮説は概ね支持されるが、ユーザー数がFCP7ピーク回復に約6年([Creative COW](https://creativecow.net/forums/thread/apple-took-6-years-to-recover-the-number-of-fcp-us/) [C])。**「すぐに」の反例として最重要** — 訓練された既存層の拒絶は即時、新規層の受容は数年単位

### 5. DaVinci Resolve Cut page(2019)【非受容の対照】

- 同一製品内の「別ページ」として追加された革新タイムラインですら、既存Edit pageユーザーには「大規模ユーザーグループの投票では圧倒的多数がCut pageをスキップ」([公式フォーラム](https://forum.blackmagicdesign.com/viewtopic.php?f=21&t=202756) [C])。転向表明([個人ブログ](https://frankglencairn.wordpress.com/2020/06/18/finecut-why-you-should-give-the-davinci-resolve-cut-page-a-try-even-as-a-seasoned-editor/) [C])は少数
- **読み方**: オプトインにしても、既存層は新UIを「拒絶」ではなく「無視」する。新UIの受容判定は新規層で測るしかない

### 6. Blender 2.80(2019-07-30)

- **受容の証拠 [A]**: リリース同月にEpic Gamesが$1.2MのMegaGrant([公式プレス](https://www.blender.org/press/epic-games-supports-blender-foundation-with-1-2-million-epic-megagrant/))、Ubisoftが開発基金参加+主要DCC移行を発表([公式プレス](https://www.blender.org/press/ubisoft-joins-blender-development-fund/))、10月にNVIDIAがPatron参加([CG Channel](https://www.cgchannel.com/2019/10/nvidia-backs-blender-development/) [B])
- **帰属されたUI決定**: 左クリック選択のデフォルト化([リリースノート](https://developer.blender.org/docs/release_notes/2.80/ui/))=「最大の新規ユーザー障壁」の一点解消(帰属はコミュニティ・第三者が広く主張、[CG Cookie](https://cgcookie.com/posts/left-click-or-right-click) [B〜C])。タスク別ワークスペースタブ、Industry Compatibleキーマップ
- **反証・留保**: 古参の右クリック擁護論争は継続 [C]。Eevee・Grease Pencilとの交絡。価格ゼロが不変のまま採用が急伸した点で「UI+機能刷新の効果」を示す比較的クリーンな事例だが、UI単独ではない

### 7. VS Code / Figma(調査データで受容が測れる2例)

- **VS Code [A]**: [SO Developer Survey](https://survey.stackoverflow.co/2019)で2018年34.9%→2019年50.7%、登場3年で首位。帰属は「軽量で速い・拡張・統合ターミナル」(第三者)。交絡: 無料・MS・OSS・TypeScriptブーム
- **Figma [A/B]**: [UX Tools調査](https://uxtools.co/survey/2020/)で主要UIツール利用が7%(2017)→37%(2019)→66%(2020)。帰属は「URL共有とマルチプレイヤーでファイル管理が消えた」。**公開当初2年は「ブラウザでプロツールは無理」と懐疑された** — 即時受容ではない。交絡: 無料枠・Sketch=Mac専用・コロナ禍
- **読み方**: 「即時受容」の実態は多くの場合2〜4年。1日〜1ヶ月で測れたのはAviUtl2(トレンド)、VOICEVOX(配布飽和)、Canva(初月15万人 [B])程度

### 8. Flow(2016、aescripts製AEイージングプラグイン)

M3のU4(区間イージングポップアップ)が参照する当のUI。

- **受容の証拠 [C]止まり**: 販売数は非公開で定量なし。[Lesterbanks](https://lesterbanks.com/2016/09/flow-will-change-work-aes-graph-editor/)・School of Motion・Motion Array等、主要MoGraph教育サイトが軒並みレビューし業界認知は広い
- **帰属されたUI決定**: (a)速度グラフ/値グラフの二重概念を捨て**正規化された1本のカーブ**へ統一、(b)CSS `cubic-bezier` という**既存メンタルモデルへの相乗り**(cubic-bezier.com互換値のコピペ対応)、(c)プリセット25種の視覚ライブラリ+ワンクリック適用、(d)カーブのユーザーライブラリ保存
- **反証・留保**: 「AE標準グラフエディタより受け入れられた」の直接証拠は定性のみ。精密制御では標準エディタへ戻るとの指摘あり

### 9. LosslessCut / YMM4 / Canva(「最初の成果までの操作数」の3例)

- **LosslessCut [A実測]**: [GitHub 42.1k stars](https://github.com/mifi/lossless-cut)(2026-07-16実測)。タイムライン・トラック・エフェクトを捨て「開く→イン/アウト→書き出し」に特化
- **YMM4 [C]**: 「セリフのテキスト入力→音声・字幕・口パク・タイミングをタイムラインへ自動生成」というドメイン特化ワークフロー。開発者自身が「編集作業の簡略化のため」と動機を自称([X](https://x.com/manju_summoner/status/1806180615446053235) [B])。VOICEVOX公開の数日後に自動連携を追加(エコシステム即応)。定量一次資料なし
- **Canva [B]**: 初月15万ユーザー、初年度75万(自己申告由来)。テンプレート起点で「ドラッグ&ドロップできれば作れる」まで学習曲線を引き下げ

### 10. その他(簡潔)

- **Procreate**: 2013年Apple Design Award、2018年iPadベストセラー [B/報道]。ADAは「デザインが理由」の第三者帰属として強い形式だが、買い切り$10とiPad Pro普及の交絡が大
- **Ableton Live セッションビュー(2001)**: 「エレクトロニック系に即座に人気」は回顧記事 [B/C] のみ。クリップ起動UIが後年Logic等に模倣された事実が間接証拠
- **Clip Studio Paint**: プロ漫画家の使用率76.1%→95.7%(業界調査 [B]、原文未照合)。Adobeサブスク移行への反発+安価買い切りの交絡が大。即時ではなく数年がかりの置換
- **Rive State Machine**: 採用企業側(Duolingo)が独立に技術的理由を挙げる点で帰属は比較的強い [B]。ただしB2B中心の漸進的採用
- **Alight Motion(UX北極星)**: **日本の若年層MAD界隈での受容を裏付けるA/B級証拠は今回発見できず**(ストアDL数区分1億超はあるが原文未照合)。北極星の受容根拠が定性止まりである点は要追撃

## 横断の型(仮説 — 「成功の法則」ではない。**界隈への着地事例で語られた語彙の観察**であり、受容の生成条件ではない)

1. **既存メンタルモデルへの相乗り**: Flow×CSS cubic-bezier、Blender×業界標準クリック、AviUtl2×旧操作体系の継承。ゼロから覚えさせない
2. **積年の一点不満の解消が最速で効く**: 64bit化(AviUtl2)、右クリック選択(Blender)、ウィンドウ行方不明(AviUtl2統合)。トレンド入り級の即時反応は「何年も待たれた不満の解消」で起きている
3. **「丁度よい」デフォルト+深い調整の二層**(VOICEVOX): 初心者は1操作で成果、上級者だけが深部を触る
4. **最初の成果までの操作数を1に近づける**: CapCutテンプレ、YMM4セリフ入力、LosslessCut単機能、Canvaテンプレ
5. **インストール障壁ゼロ・起動即答**: ZIP解凍即起動・配布2.2MB(AviUtl2)、起動1秒未満の言説(AviUtl [C])
6. **新規層と既存層は別の審判**: 革新UIは新規層に受容されても、既存層は即時拒絶(FCPX)か無視(Cut page)する。既存層の回復は年単位。M3実装ガード5(オプトイン原則)と整合し、加えて「新UIの受容判定は新規層で測る」という測定側の含意を足す
7. **エコシステムの共同開発者化**: SDK同時公開(AviUtl2)、隣接ツールとの連携即応(YMM4×VOICEVOX)
8. **交絡の常在**: 全事例に無料/価格/流通の交絡。受容事例を「UIが良かったから」と単因で読まない

## M3への転移候補(仮説台帳 — 採択はM3入場PRの再翻訳時に個別判断)

現行方針を**変えない**。既定方針の裏付けになったもの、具体化の候補になったものを分けて記す。

### 既定方針の裏付け(変更不要、出典が増えただけ)

- **U4 区間イージングポップアップ(Flow/AM式)**: Flowの受容帰属(正規化1本カーブ・cubic-bezier相乗り・プリセット視覚ライブラリ)は、concept.mdの2026-07-09決定(AEグラフエディタを作らない)と一致
- **実装ガード5(革新はオプトイン)**: FCPX両面+Cut page無視の対で補強。既存層の拒絶は即時、受容は年単位
- **U1 固定分割レイアウト**: AviUtl2のシングルウィンドウ統合が「ウィンドウ行方不明の解消」として受容された。フローティング多窓に戻さない方針と整合
- **実装ガード10(起動時間・アイドルメモリの数値目標)**: AviUtl2でも軽さ(配布2.2MB)自体が話題化=この層の軽さ感度の再確認
- **プラグインファースト**: SDK同時公開が「コミュニティと共に開発する姿勢」として好意的に受容された(AviUtl2)

### 具体化の候補(U4/U1の完了条件へ焼くには個別採択が必要)

1. **U4**: イージングプリセットは一覧テキストではなく**カーブ形状の視覚ライブラリ**として出す。cubic-bezier 4値の**文字列コピペ互換**(cubic-bezier.com/Flow/CSS表記)を入出力に持つ。ユーザー定義カーブの保存
2. **実装ガード10**: 数値目標をU1実測で決める際の参照点に「配布サイズ」も加える(ZIP解凍即起動の成立性はSlint/wgpu依存で異なるため、目標値ではなく計測項目として)
3. **U4 パラメータパネル**: VOICEVOX「丁度よさ」型 — 参照プラグインの`ParamDef` defaultは「触らずに見栄えのする値」であることをレビュー観点に置く(plugin-authoringのdefault必須規約に意味の観点を足す提案)
4. **U5/U6/U7 横断**: YMM4型の「ドメイン特化の最短経路」をMVに翻訳すると「**楽曲をセット→BPM入力→ビートグリッドにスナップして置くだけで音に合う**」がそれに当たる。新規タスクは切らず、U6→U7の接続体験(楽曲セットからスナップ配置までの操作数)を統合レビューの観点とする
5. **測定側の含意(型6)**: M3のUI受け入れ確認を既存AEユーザーだけで行わない。AM/CapCut系のモバイル編集経験層(=新規層)を含める

## 第二部: 操作パターン単位の受容 — 収斂した操作語彙とUX原理(2026-07-16 追補)

第一部への批判(交絡・参照クラスの偏り)を受けて、受容の単位を**プロダクトから操作パターンへ**下げる。ここでの「すぐに受け入れられるUI」の操作的定義:

> **業界横断で同一の操作へ収斂している語彙は、対象ユーザーが既に訓練済みであり、新規に学ぶものが無い(=即受容)。**

これは根拠のある定義である: 学習時間は「新規に学ぶ手続き(プロダクションルール)数」に線形比例し、既知システムと共有される手続きの学習コストはほぼゼロ(Bovair, Kieras & Polson 1990、後述)。逆に、収斂していない領域でどの慣習を選んでも誰かの期待を裏切る。したがってM3の設計方針はシンプルに書ける: **収斂語彙はそのまま借用し、独自設計(と選択の自由)は非収斂領域に限定する**。実装ガード5「既定は業界標準の操作」の「業界標準」の中身を、以下で具体的に列挙する。

照合状態の注記: 各操作の裏付けは公式マニュアル・公式ヘルプURL(恒久文書)。support.alightmotion.com とBlackmagicの公式マニュアルPDFは直接フェッチ403のためスニペット照合(その旨明記)。

### 2-A. 収斂語彙の台帳(ゼロ学習で借用できる操作)

| 操作 | 収斂の内容と裏付け | M3該当 |
|---|---|---|
| **Space=再生/停止** | 実質全ツール共通([Premiere公式ショートカット表](https://helpx.adobe.com/premiere/desktop/get-started/keyboard-shortcuts/default-keyboard-shortcuts.html)、[FCP公式](https://support.apple.com/guide/final-cut-pro/keyboard-shortcuts-ver90ba5929/mac)) | U5 |
| **JKLシャトル** | J=逆再生/K=停止/L=順再生/連打で倍速、**K+J/Lで半速**という細部までFCP/Resolveで一致(FCP公式同上、Resolveはマニュアルに独立節 — 公式PDF直接照合は403)。モバイル系には無い=デスクトップ専門家語彙 | U5 |
| **クリップ端ドラッグ=トリム** | [Premiere公式](https://helpx.adobe.com/premiere-pro/using/trimming-clips.html)、[FCP公式](https://support.apple.com/guide/final-cut-pro/extend-or-shorten-clips-ver9847ec25/mac)。※トリム後の隙間処理は非収斂(2-B) | U3 |
| **磁石アイコン=スナップ+単キートグル(ドラッグ中も切替可)** | [FCP: Nキー・長押しで一時切替](https://support.apple.com/guide/final-cut-pro/snap-to-items-in-the-timeline-ver9f7888dc3/mac)、[Premiere: Sキー・磁石アイコン](https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/snap-clips.html)。図像と「単キー+一時切替」は収斂、キー自体は非収斂(2-B) | U3/U7 |
| **再生ヘッド位置で分割+Shiftで全トラック** | 構造が収斂: [Premiere Ctrl+K / Ctrl+Shift+K](https://helpx.adobe.com/premiere/desktop/get-started/keyboard-shortcuts/default-keyboard-shortcuts.html)、[FCP Cmd+B / Shift+Cmd+B](https://support.apple.com/guide/final-cut-pro/cut-clips-in-two-ver4e30479/mac)。キーはB陣営(Apple/BMD/CapCut)とK陣営(Adobe)に分裂(2-B) | U3 |
| **キーフレーム=菱形(ダイヤモンド)図像** | [FCP「白いダイヤモンドで表示」](https://support.apple.com/guide/final-cut-pro/add-video-effect-keyframes-ver8e3f20ea/mac)、[Blender Dope Sheet](https://docs.blender.org/manual/en/2.83/editors/dope_sheet/introduction.html)、[Adobe KF図解](https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/about-keyframes.html)。図像としてほぼ100%収斂 | U4 |
| **1個目のKFだけ明示的、以降は値変更で自動キー** | [AE公式](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animation-basics/animation-basics.html)。AM/FCP/Resolveも同挙動(AM公式ヘルプは403、裏付け中) | U4 |
| **正規化2ハンドルのcubic-bezierイージング+ease系命名** | [CSS easing-function(MDN)](https://developer.mozilla.org/en-US/docs/Web/CSS/easing-function)で規格化。[Flow](https://aescripts.com/flow/)はこのモデルとAEを橋渡しし**cubic-bezier値のコピペ**対応。[AM公式のカーブエディタ](https://support.alightmotion.com/hc/en-us/articles/10536934703889-Animation-Easing-Curves)(403・スニペット照合)も同型。**「Ease In=ゆっくり始まる」の語義は完全に統一** — 独自命名を避けるべき領域 | U4 |
| **scrubbable number(数値の横ドラッグ増減)** | 4ツール以上で収斂、用語も「scrub」で統一: [AE(hot text)](https://helpx.adobe.com/be_en/after-effects/using/layer-properties.html)、[Figma](https://help.figma.com/hc/en-us/articles/360039956914)、[Blender(縦ドラッグで複数フィールド一括)](https://docs.blender.org/manual/en/latest/interface/controls/buttons/fields.html)、[Cavalry](https://docs.cavalry.scenegroup.co/user-interface/menus/window-menu/attribute-editor/control-rows/control-rows-interaction/) | U4 |
| **Shift=比率固定(完全収斂)、8ハンドル構造、Shift回転=角度スナップ** | [Figma](https://help.figma.com/hc/en-us/articles/360040451453-Scale-layers-while-maintaining-proportions)、[PowerPoint](https://support.microsoft.com/en-us/office/graphics-visuals/change-the-size-of-a-picture-shape-text-box-or-wordart)、[Canva(Shift回転=15°)](https://www.canva.com/help/flip-and-rotate/)。※「中心基準スケール」の修飾キーは非収斂(2-B) | U1/U3 |
| **ダブルクリック=一段深く入る、Esc=一段出る** | [Illustrator分離モード](https://helpx.adobe.com/illustrator/desktop/manage-objects/select-objects/isolate-objects.html)、[Figma deep select](https://help.figma.com/hc/en-us/articles/360040449873-Select-layers-and-objects)。グループ内部編集の共通文法 | U8 |
| **OSからのドラッグ&ドロップ読み込み** | [FCP(タイムラインへ直接、編集種別つき)](https://support.apple.com/guide/final-cut-pro/drag-clips-to-the-timeline-ver4e30143/mac)、[Premiere](https://helpx.adobe.com/premiere-pro/how-to/import-file-directly.html)。※Premiereは実装差で公式がMedia Browser経由を推奨 — 「D&Dは必ず動く」を完了条件で保証する価値がある | U6 |
| **Cmd/Ctrl+Z、右クリックメニュー(モバイルは長押し)** | 普遍。ただし**「無制限Undo」は業界標準ではない**([AEの既定Undo段数は32](https://helpx.adobe.com/after-effects/using/preferences.html)) — M2ジャーナル設計で深いUndoを持つMotoliiには差別化点でもある | U2 |

### 2-B. 非収斂領域の台帳(独自設計の自由と責任がある帯)

どれを選んでも誰かの慣習を裏切る領域。方針: **主要ターゲット(AviUtl/AM出身層)の慣習を第一参照にして選び、実装ガード5の「初日からのショートカットカスタマイズ」を保険にする**。

| 領域 | 分岐の実態 |
|---|---|
| 分割キー | B(Blade: Apple/BMD/CapCut)vs K(Adobe)。新しめのツールはB系 |
| スナップキー | N(FCP/Resolve)vs S(Premiere) |
| タイムラインズーム | Cmd+=/-、=/-、Ctrl+ホイール等ツールごとにバラバラ。ピンチはタッチ系のみ収斂 |
| トリム後のリップル挙動 | FCP=常時リップル、Premiere/Resolve=隙間残置+別ツール、CapCut/AM=自動クローズ寄り。**ツール哲学の分岐点**(ガード5のオプトイン原則の適用先) |
| KF有効化の入口 | **ストップウォッチはAdobe方言**([AE](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animation-basics/animation-basics.html)/[Premiere](https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/add-keyframes.html)のみ)。FCP/Resolve/AMは**菱形ボタンで直接キー追加**([FCP](https://support.apple.com/guide/final-cut-pro/add-video-effect-keyframes-ver8e3f20ea/mac))— 非Adobe系はこちらに収斂 |
| パラメータのリセット | Blender=右クリック→デフォルトへ、Adobe=エフェクト単位Reset、Figma=無し。統一パターン不在 |
| リネーム | ダブルクリック(Figma等Web系)vs Enter(AE)vs F2(Blender) |
| グループ化の意味 | Ctrl+Gの図像は共通だが、映像系では意味自体が分岐(AE=プリコンポ、NLE=ネスト)。MotoliiはAM式グループ意味論を決定済み(concept.md)なので、借りるのはキー図像のみ |

### 2-C. UX原理の一次資料台帳(なぜその操作は即受容されるのか)

| 原理 | 原典 | 核心 | 効くM3面 |
|---|---|---|---|
| 直接操作の3要件 | Shneiderman (1983) "Direct Manipulation," *IEEE Computer* 16(8) [DOI](https://dl.acm.org/doi/10.1109/MC.1983.1654471) | "Continuous representation of the object of interest / Physical actions... instead of complex syntax / **Rapid incremental reversible operations** whose impact ... is immediately visible" | キャンバス・タイムライン・KFは数値入力より直接ドラッグを優先(U1/U3/U4) |
| 実行/評価の隔たり | Hutchins, Hollan & Norman (1985) *HCI* 1(4) [PDF](https://worrydream.com/refs/Hutchins_1985_-_Direct_Manipulation_Interfaces.pdf) | 操作と結果表示の距離が「直接感」を決める | 同上 |
| 応答時間0.1秒の限界 | Miller (1968) [DOI](https://dl.acm.org/doi/10.1145/1476589.1476628) / Nielsen (1993) [3限界](https://www.nngroup.com/articles/response-times-3-important-limits/) | "0.1 second is about the limit for having the user feel that the system is reacting instantaneously" | スクラブ・値変更は100ms以内に**何かを**見せる。Draft降格(低忠実でも即時)はこの原理の実装(U5・ガード8) |
| スクラブの遅延対策(実証) | Matejka, Grossman & Fitzmaurice, "Swift" (CHI 2012) [DOI](https://dl.acm.org/doi/10.1145/2207676.2207766) / "Swifter" (CHI 2013) [DOI](https://dl.acm.org/doi/10.1145/2470654.2466149) | スクラブ中の低解像度即時表示が遅延の悪影響を打ち消す。サムネイル格子でシーン探索が最大48%改善(著者報告) | M2 Transport+適応解像度+「最新要求のみ処理」(ガード8)の**直接の学術的裏付け** |
| Jakobの法則 | Nielsen (2000) ["End of Web Design"](https://www.nngroup.com/articles/end-of-web-design/) | "Users spend most of their time on other sites... users prefer your site to work the same way as all the other sites they already know" | 収斂語彙の借用そのもの(2-A) |
| 一貫性と標準 / 認識>想起 | Nielsen (1994) [10ヒューリスティック](https://www.nngroup.com/articles/ten-usability-heuristics/) | #4: プラットフォーム・業界慣習への追従。#6: "The user should not have to remember information..." | プリセットは名前でなく**カーブ形状のサムネイル**で見せる(U4) |
| 段階的開示 | Nielsen (2006) [Progressive Disclosure](https://www.nngroup.com/articles/progressive-disclosure/) | "defers advanced or rarely used features to a secondary screen, making applications easier to learn and less error-prone" | 第一部のVOICEVOX「丁度よさ」の原理名。NodeDescパネルは賢いデフォルト+詳細展開の二層(U4) |
| Fittsの法則 | Fitts (1954) *J. Exp. Psychol.* 47(6) [PDF](http://www2.psychology.uiowa.edu/faculty/mordkoff/InfoProc/pdfs/Fitts%201954.pdf) | 到達時間は距離と目標幅の関数 | 菱形KF・ベジェハンドル・トリムハンドルのヒットエリアを見た目より広く(U3/U4) |
| Hickの法則 | Hick (1952) [DOI](https://journals.sagepub.com/doi/10.1080/17470215208416600) | 選択時間は選択肢数の対数に比例 | イージングプリセット一覧はカテゴリ分割・少数先出し(U4) |
| 学習転移の定量 | Bovair, Kieras & Polson (1990) *HCI* 5 [PDF](https://web.eecs.umich.edu/~kieras/docs/Procedural_knowledge/BovairKierasPolson1990.pdf)、Card, Moran & Newell (1980) [KLM](https://dl.acm.org/doi/10.1145/358886.358895) | 学習時間は**新規に学ぶプロダクションルール数に線形比例**。既知と共有する手続きのコストはほぼゼロ | 「収斂語彙=ゼロ学習」の実証根拠。本部の定義そのもの |

照合状態: nngroup.com はプロキシ403のため、引用は複数の独立ソースのスニペットで逐語照合(URLは原典恒久URL)。Shneiderman 1983本文はACM購読壁(書誌・要旨確認)。設計根拠へ昇格させる前に原文再確認が必要。

### 2-D. M3への転移(操作レベルの仮説台帳 — 採択はM3入場PRの再翻訳時)

1. **U3(タイムライン)**: 2-Aの収斂語彙(端ドラッグトリム/磁石+単キー+一時切替/分割+Shift全トラック/Space・JKL)を受け入れ確認のチェックリストにする。非収斂のキー割当(分割・スナップ・ズーム)はAviUtl2の既定を第一参照に選び、初日カスタマイズ(ガード5)を保険に
2. **U4(キーフレーム)**: 菱形図像を採用。**ストップウォッチは採用しない**(Adobe方言。非Adobe系の「菱形ボタン直接追加+以降自動キー」に乗る)。イージングは既定方針(Flow/AM式)が2-Aの収斂と完全整合 — 具体化: 正規化2ハンドル+ease系命名+形状サムネイルのプリセット+cubic-bezier 4値文字列のコピペ入出力
3. **U4(パラメータパネル)**: NodeDesc自動生成行の数値は**全てscrubbable**(スライダー併記の有無に関わらず)。リセットは非収斂領域なので独自に決めてよい(Blender式右クリックを候補)
4. **U5(トランスポート)**: Swift/Swifterを、ガード8(最新要求のみ+観測可能)とDraft降格の学術的裏付けとして出典に追加。「100ms以内に低忠実でも何かを見せる」を受け入れ観点の言葉にする
5. **U6(アセットブラウザ)**: OSからのD&Dが「必ず動く」ことをU6完了条件が既に含む(現行どおり)。Premiereの実装差はD&Dを軽視しない反面教師
6. **U8(グループ)**: 「ダブルクリックで一段入る/Escで出る」をグループ内部選択の文法に借用

## 第三部: 後発の勝ち筋 — 「どの操作も直感的」(Ableton→Motolii、2026-07-16 追補)

方針の明文化(2026-07-16 ユーザー明言): **MotoliiはAEのカウンターとして「どの操作も直感的」を目標にする**。Abletonが後発DAWなのに広まった答えは「どの操作も直感的だった」ことにある、という仮説が出発点。本節はその仮説を一次資料で検証し(結論: **留保つきで成立**)、「直感的」を検証可能な語彙に分解する。

照合状態の注記: 本節の出典は全てプロキシ403のため検索スニペット照合(フレーズ完全一致で確認したものを明記)。設計根拠への昇格前に原文照合が必要。

### 3-A. Abletonは実際に何をやったか(一次資料)

**設計意図(創業者自身)**:

> "We try to make it more like an instrument and less like a tape machine." — Gerhard Behles([Tape Op](https://tapeop.com/interviews/73/gerhard-behles-dave-hill)、フレーズ完全一致で確認)

> "What we wanted to develop was a new type of sequencer, optimized for live performance, based on our own experiences." — Robert Henke([MusicRadar](https://www.musicradar.com/news/ableton-live-origins-robert-henke))

初版マニュアル自身がLiveを「sequencing instrument」と自称([SOS 2002レビュー](https://www.soundonsound.com/reviews/ableton-live)経由)。

**操作レベルの具体(公式マニュアルで裏付く)**:

| 操作 | 内容 | 出典 |
|---|---|---|
| 止まらない音 | "You can play MIDI and audio loops of different lengths in any combination, **without ever stopping the music**" | [公式 What is Live?](https://www.ableton.com/en/live/what-is-live/) |
| クリップ発射+グローバルクオンタイズ | いつ押しても音楽的に正しいタイミングで発射される(失敗できない操作) | [Launching Clips](https://www.ableton.com/en/manual/launching-clips/) |
| D&D→自動テンポ同期 | 任意のサンプルをドロップするだけで選択テンポに同期(ワープ) | [Audio Clips, Tempo, and Warping](https://www.ableton.com/en/manual/audio-clips-tempo-and-warping/) |
| 読み込む前に試聴(prehear) | ブラウザでサンプル/プリセットを配置前に試聴、メイン出力を止めずに | [Working with the Browser](https://www.ableton.com/en/live-manual/12/working-with-the-browser/) |
| シングルウィンドウ・非モーダル | 「Liveにはウィンドウが1つしかない」、ツール持ち替え不要 | [Tape Op Live 4レビュー](https://tapeop.com/reviews/gear/46/live-4)、[Live Concepts](https://www.ableton.com/en/manual/live-concepts/) |
| 全パラメータが即オートメーション可 | "Practically all mixer and device controls in Live can be automated, **including the song tempo**" | [Automation](https://www.ableton.com/en/manual/automation-and-editing-envelopes/) |

**第三者の帰属**: 最初期(2002年)のSOSレビューが既に "an **intuitive** graphical front end allowing for a fair amount of improvised rearrangement" と評し、20年後のMusicRadarは "Live's greatest strength is that it feels like **the DAW itself is an instrument**"([2024](https://www.musicradar.com/news/8-things-ableton-live))。対比される既存パラダイムは、マルチトラックテープレコーダー+ミキシングデスクのメタファー(録音→編集→ミックスの直線工程。[ARP Journal](https://www.arpjournal.com/asarpwp/beyond-skeuomorphism-the-evolution-of-music-production-software-user-interface-metaphors-2/))。

### 3-B. 正確な読み(留保込み — ここが引用の誠実さの境界)

「即座に誰にでも直感的だった」は**単純化**である:

- セッションビューは今でも初見の混乱要因(「Session View is what confuses people when they first open Ableton」[pushpatterns](https://www.pushpatterns.com/blog/is-ableton-hard-to-learn)、公式フォーラムにも学習曲線スレッド複数)
- SOS自身が「登場時、他のループ系アプリが既にあり、その重要性はすぐには明白でなかった」と回顧([25 Products That Changed Recording](https://www.soundonsound.com/reviews/25-products-changed-recording))
- 普及にはベルリンのシーン人脈・EDM/フェス文化・ワープの技術優位・安定性の交絡(Henke自身は普及要因を直感性より「ステージへの明確な経路」で語る — [Vice](https://www.vice.com/en/article/ableton-live-history-interview-founders-berhard-behles-robert-henke/))

正確な型はこうなる: **既存メタファー(テープ/スタジオ)を捨て、特定ユースケース(演奏)に最適化した即時性(immediacy)**。「直感的」の実体は見た目の分かりやすさではなく、**「操作→音」のフィードバックループが決して切れない**こと — Shneiderman第3要件(即時可視・漸進・可逆)の徹底である。そしてパラダイム転換は独自の学習コストを伴った。Motoliiはこのコストを、第二部の収斂語彙借用+実装ガード5(革新はオプトイン)で抑えられる — Abletonが持っていなかった保険。

### 3-C. 「直感的」の検証可能な分解

「どの操作も直感的」を、原理台帳(2-C)の語彙でM3の受け入れ観点に翻訳する:

1. **対象が常に見えている**(認識>想起): 操作対象(クリップ・キーフレーム・カーブ・パラメータ)が画面上の実体としてあり、名前や式で参照させない
2. **文法でなく物理動作**(直接操作): モーダルダイアログ・別画面・テキスト入力・モード切替を経由せず、見えているものを掴む
3. **結果が100ms以内に見え、可逆**: 操作→プレビュー反映のループが切れない。切れそうな時は忠実度を落として即時性を守る(Draft降格)

**要件3は性能の関数である。** AEが直感的になれない根因は、機能の不足ではなく「操作→結果」のループが描画待ちで切れること([ae-pain-points](../ae-pain-points.md) A章: リアルタイムプレビュー無し・マイクロフリーズ)。MotoliiのVRAM常駐・f(t)純関数・Draft降格([performance-model](../performance-model.md))は「軽さ」の話であると同時に**直感性の前提条件**であり、Abletonにおけるリアルタイム音声エンジンと同じ位置にある。アーキテクチャの絶対規律がそのままUXテーゼになる。

### 3-D. AEの間接性 vs Motoliiの対応(カウンターの棚卸し)

[ae-pain-points](../ae-pain-points.md)の痛点を「どの直感性要件が切れているか」で読み直すと、Motoliiの既存決定はほぼ全てこの表に沿っている:

| AEの間接性 | 切れている要件 | Motoliiの対応(決定済み) |
|---|---|---|
| 変更→RAM Preview待ち | 3(即時) | 100ms以内にDraftで見せる+適応降格(U5・ガード8、Swift/Swifterが実証) |
| グラフエディタ=別画面で接線を操作 | 2(間接) | 区間を選択→**その場**ポップアップ(U4、Flow/AM式) |
| 式=JS文字列で他レイヤーを参照 | 1・2(文法) | ターゲットを**クリックする**型付きリンク(GAP-8、concept.md) |
| 物理風の動き=valueAtTime式シミュ | 2(文法) | 補間型の**選択肢**(Interp::Bounce/Elastic — AM実証) |
| プリコンポ=別タイムラインへ移動 | 1(文脈喪失) | その場グループ+項目エンベロープ(concept.md) |
| Undo=メモリ不安・段数制限 | 3(可逆) | コマンド差分ジャーナル(M2-D2、状態を複製しない) |
| プラグインUIごとの独自画面 | 1・2 | NodeDesc自動生成の統一パネル(U4) |

**Abletonの「止まらない音」のMV版**: 編集操作(配置・移動・トリム・値変更)をしても**再生が止まらない**こと。M2 Transport決定(音声クロック常時主+ドロップ+適応解像度)は構造としてこれを許すので、U5の受け入れ観点の言葉にできる。

### 3-E. 転移仮説(採択はM3入場PRの再翻訳時)

1. **「どの操作も直感的」の受け入れ観点化**: 「主要編集操作(配置/移動/トリム/キー打ち/イージング編集/パラメータ変更)は、モーダルダイアログ・別画面・テキスト入力を経由せず、結果が100ms以内にプレビューへ現れる」をM3横断の受け入れ観点として明文化する
2. **「再生を止めない編集」**をU5の観点候補に追加する(編集中も音声クロックが走り続ける)
3. **アセットのprehear**: U6のアセットブラウザに「配置前プレビュー」(動画サムネイルのホバースクラブ/楽曲の試聴)の席を検討する(Abletonブラウザの型)
4. 直感性の主張は文言でなくアーキテクチャで担保されていることを、M3ドキュメントからperformance-modelへ相互参照する

## 参照クラスの偏り(2026-07-16 レビュー指摘で追加 — **第一部**の最も重要な限界)

指摘: 「どれも以前からユーザーがいたソフトで、界隈にも属している。不確定の変数が多すぎる」。事例を先行基盤で分類し直すと、指摘のとおり構造的な偏りがある:

| 事例 | 先行して存在したもの | 即時性の実態 |
|---|---|---|
| AviUtl2 | AviUtl界隈(2008年拡張編集以来)+作者KENくんの正統性 | 即時(翌朝トレンド)=**待機需要の解放** |
| VOICEVOX | ゆっくり/音声合成文化圏・ニコニコ界隈+キャラIP | 即時(配布飽和)=同上 |
| CapCut | TikTokの流通と投稿動機 | 即時〜数ヶ月=流通の現象 |
| Blender 2.80 | 既存Blenderユーザー・OSSコミュニティ | 即時(支援集中)=既存不満の解消 |
| VS Code | Microsoft・既存開発者エコシステム | 3年 |
| FCPX | 既存Final Cutプロ層(=拒絶の主体) | 拒絶が即時、受容は6年 |
| Flow | AEユーザー(プラグインは定義上ホスト界隈に着地) | 不明(定量なし) |
| YMM4 / Clip Studio | YMM1〜3/ゆっくり界隈、ComicStudio資産 | 年単位の置換 |
| Figma / Ableton / Procreate / Canva | **既存界隈を持たない新規参入に近い** | **2〜4年、または証拠がマーケ交絡で弱い** |
| LosslessCut | OSS(界隈なし) | 緩慢な蓄積(stars)。NLEではない |

**帰結**: 「ゼロ界隈から、UI設計の力で、即時(日〜週)に受け入れられた」事例は本調査に**1件も無い**。即時性の強い証拠(トレンド入り・配布飽和)は全て待機需要を持つ界隈への着地で発生しており、界隈を持たない新規参入は例外なく年単位か証拠薄弱。したがって:

1. 横断の型1〜7は「界隈への着地に成功した事例で語られた語彙」の観察であり、受容の生成条件ではない
2. **Motoliiへの読み替え**: Motoliiは「界隈(AviUtl層/AM層)は存在するが、作者の正統性・既存資産・流通を持たない新規参入」。参照枠はAviUtl2型の即時受容ではなく、**Figma/Ableton型(新規層から年単位)が現実的な期待値**。AviUtl2型の即時性を計画の前提にしない
3. 本メモの用途は「対象界隈の期待チェックリスト」(何を受容/拒絶の理由として語る界隈か)に限定する

## 選択バイアス・限界の申告

- **参照クラスの偏り**: 上節のとおり。本メモ最大の限界
- **生存バイアス**: 受容された事例だけを集めた。同型のUIで受容されなかった反例(例: Premiere Rushのテンプレ駆動は CapCut と同型だがDLシェア9%)の探索は未実施
- **「即時」の定義が未統一**: トレンド入り(1日)〜調査首位(3〜4年)まで幅がある。真に即時(1ヶ月以内)で測れたのはAviUtl2・VOICEVOX・Canvaのみ — そしてその3件全てに待機需要または強いマーケ交絡がある
- **UI単独因果は全事例で分離不能**: 交絡(無料・流通・時代・先行基盤)を各事例に明記した。本メモから言えるのは「受容がどのUI決定に**帰属されたか**」までであり「そのUI決定が受容を**生んだ**」ではない
- **原文未照合の出典**: プロキシ制限により窓の杜・Wikipedia・variety.com・aescripts等の多数が403で、検索スニペット経由の確認に留まる(本文に都度明記)。**設計根拠へ昇格させる項目は原文照合を先に行う**

## 追撃調査項目

| 項目 | 目的 |
|---|---|
| **ゼロ界隈からの即時受容が存在するかの探索** | 参照クラスの偏りの検証。見つからなければ「即時受容=待機需要の現象」仮説が強まり、UI設計への期待値を年単位に固定する |
| Alight Motionの受容定量(ストア統計・Z世代調査) | UX北極星の受容根拠がC級のままなのを解消 |
| 反例探索(同型UIで非受容: Premiere Rush等) | 型1〜7が生存バイアスでないかの検証 |
| 窓の杜・ニコニコ公式ランキング等の原文照合 | 未照合A級出典の確定 |
| Flowの採用規模の代理指標(チュートリアル言及数等) | U4方針の参照事例の証拠強化 |
| 第二部の403出典の原文照合(AMヘルプ・Resolveマニュアル・nngroup.com・Shneiderman 1983本文) | 収斂語彙・UX原理を設計根拠へ昇格させる前提 |
| AviUtl(拡張編集)自身の操作語彙と2-A収斂語彙の差分表 | 非収斂領域のキー割当を「AviUtl2既定を第一参照」で決めるための材料 |

## 改訂記録

- 2026-07-16(同日・1回目): レビュー指摘「全事例が既存ユーザー/界隈持ちで不確定変数が多すぎる」を反映 — ①「参照クラスの偏り」節を追加し先行基盤で全事例を再分類 ②「ゼロ界隈からの即時受容は本調査に1件も無い」を明記 ③本メモの用途を「対象界隈の期待チェックリスト」へ格下げ ④Motoliiの参照枠をAviUtl2型(即時)ではなくFigma/Ableton型(年単位)と明記 ⑤横断の型を「語られた語彙の観察」へ弱化
- 2026-07-16(同日・2回目): レビュー指摘「市場の話ではなく根本的なUX/UIの話」を反映 — 第二部を追補。①受容の単位を操作パターンへ下げ「業界収斂した語彙=訓練済み=ゼロ学習」と操作的に定義(Bovair/Kieras/Polsonの学習転移研究を根拠に) ②収斂語彙の台帳(2-A)と非収斂領域の台帳(2-B)を公式マニュアルURLつきで作成 ③UX原理の一次資料台帳(2-C)を追加 ④操作レベルのM3転移仮説(2-D)を追加。発見: ストップウォッチはAdobe方言/「無制限Undo」は業界標準ではない/分割キーはB陣営とK陣営に分裂
- 2026-07-16(同日・3回目): ユーザー方針「Abletonの答えは操作の直感性。MotoliiはAEのカウンターとしてそうありたい」を受けて第三部を追補。①Abletonの設計意図("more like an instrument and less like a tape machine")と操作具体(止まらない音・クオンタイズ発射・D&D即同期・prehear・非モーダル単一窓)を一次資料で裏付け ②「即座に誰にでも直感的」は単純化と申告(セッションビューの学習コスト・交絡) — 正確な型は「既存メタファーを捨て特定ユースケースに最適化した即時性」 ③「直感的」をShneiderman 3要件+100msへ分解し、要件3が性能の関数=performance-modelが直感性の前提条件であることを明文化 ④AEの間接性vs Motolii対応表(ae-pain-points接続)と転移仮説4件(受け入れ観点化・再生を止めない編集・prehear・相互参照)を追加
