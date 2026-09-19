# 本気の道具を共同体にした物 — 道具の中に何を仕込んだか(2026-09-19)

利用者の問い「どのソフトが、本職の道具を共同体に変えたか。宣伝でなく、道具の中に見える製品判断(UI・ファイル形式・投稿・リミックス・既定値)は何か」。一次資料(公式 docs・社長が訊く・開発者 blog・CACM 論文・終了告知)から。引用は 25 語以内、URL 付き。読めなかった資料はそう書いた(社長が訊く 日本語版 vol5 は Shift_JIS で取得失敗 → 英語版 Iwata Asks を使用)。

## 要約 5 行

1. 共通する芯は **「投稿ボタンが道具の中にある」+「共有の単位 = 1 ファイル(ソース込み)」** — Scratch の Share アイコン、うごメモの本体投稿、PICO-8 の `.p8.png`、Figma の Duplicate。宣伝で共同体を作った例は無い。
2. **リミックスは系譜と一体** — Scratch は remix に自動で元へのリンク、Dreams は Genealogy、ニコニコはコンテンツツリー(親作品)。系譜が無いと「盗まれた」で荒れる(Scratch 初期の実例)。
3. **制約が身元になる** — うごメモ 3 色・DSi 限定、PICO-8 128×128・16 色・8192 token、Scratch は「天井を上げない」。制約は絵の様式を揃え、見ただけで「あの道具の作品」と分かる。
4. **本職の道具(AviUtl・MMD・Blender・Ableton)は投稿機能を持たず、ファイル形式 + 置き場(plugins フォルダ、pmx/vmd、Extensions、Packs)で共同体を作った。配布側はニコニコ/BBS が担った** — 道具は器、意味は外。
5. 失敗はほぼ **「場が道具の外にあり、場が消えると作品が消える」**(うごメモはてな 2013、Dreams 2023)と **「素材の権利が道具の外で未整理」**(MMD)。Motolii への含意は「rrd = 共有単位、系譜を文書に、投稿は道具から」の 3 つの問いとして末尾に。

## 事例の表

| 道具 | 年 | 何が本職向けか | 道具内の 1 つの仕掛け | 共有の単位 | 意図した制約 | 何が壊れたか |
|---|---|---|---|---|---|---|
| うごくメモ帳 / うごメモはてな | 2008–2013 | 手描きコマ撮り(パラパラ) | 本体から投稿・DL・星・他人の作品を落として編集(spin-off)。「Lock」で編集禁止を作者が選ぶ | `.ppm`(フレーム + ADPCM 音) | 3 色、DSi 限定、はてなの人力監視 + 通報 | 3DS 版に伴い 2013-05-31 終了。作品は Archive/Sudomemo の有志が保存 |
| Ableton Live + Note + Learning Synths | 2001– / Note 2022 | DAW・Push・Max for Live | Note → Cloud → Live の Browser に Set が現れる。Learning Synths の「Export = M4L synth を Live Set に」 | Live Set(.als) + Pack、M4L device | Cloud は 8 Set まで、同一 account | 共同体は外部(maxforlive.com、Gumroad)。道具内は片方向の受け渡しのみ |
| AviUtl + 拡張編集 | 1997– / 拡張編集 2008 | 本職級のフィルタ・出力 | `plugins` フォルダに dll を置くだけ(.auf/.aui/.auo/.auc)。拡張編集は Lua スクリプト(.anm/.obj)配布で表現が増える | `.aup` + 使ったプラグイン一式 | 32bit・AVI 中心。本体は最小 | 配布は作者個人サイト + ニコニコ。プラグイン依存で `.aup` は単体で開けない。2019 停止 → 2025 ExEdit2 でゼロから |
| MikuMikuDance | 2008– | 3D 振り付け・レンダ | モデル(pmd/pmx)・モーション(vmd)・ステージ(x)を別ファイルで読む → 部品の交換が自然 | `.pmx` / `.vmd` / `.pmm` | 樋口氏はユーザーモデルに一切関与しない立場 | 権利は素材ごとの規約任せ。MMD杯はマイリスト工作・転載で「MMD杯問題」 |
| PICO-8 | 2015– | 完全なゲーム開発環境(コード・絵・音) | カートが PNG(コード込み)。SPLORE で BBS のカートを本体内で見て `load #id` → ESC でコードが開く | `.p8.png` | 128×128、16 色、8192 token(「意図的に厳しい」) | ほぼ無い。ただし共有は BBS(web)側に依存 |
| Scratch | 2007– | (本職向けではない)プログラミング環境 | 画面上部の Share アイコンで即投稿。Remix で元へのリンクを自動付与、派生一覧 | `.sb3`(zip、ソース込み) | 天井を上げない(低い床・広い壁) | 初期「盗まれた」論争 → 系譜表示で解決 |
| Figma Community | 2019 beta / 2020 | 本職 UI デザイン | 生きたファイルを公開、Duplicate で自分の Drafts に写す(履歴・コメントは切れる) | Figma file(CC BY 4.0) | 複製は完全に別物、更新は再複製 | 複製制限が効かない報告、テイクダウン運用は事後(6 か月以内の異議) |
| Blender | 1994– / Extensions 2024 | 本職 3D | 2006 Elephants Dream = 制作ファイル 7GB を CC で公開。2024 Preferences › Get Extensions で本体内から導入 | `.blend` + Extension | GPL 限定、既定でネットに繋がない | 商用は Blender Market 等に分離。open movie は道具の宣伝であり共同体機構ではない |
| Dreams | 2020–2023 | ゲームエンジン級 | Public にすると誰でも remix。Genealogy と creation credits で元作者は常に分かる | Dreamiverse 内の element/scene(外へ出ない) | PS4 限定、外部書き出し無し | 2023-09-01 live support 終了。作品は PS 内に残るが形式は閉じたまま |
| Roblox Studio / Minecraft | 2006– / 2010– | ゲーム制作 | Creator Store(2024 改称)で asset・plugin を Studio 内から挿入。Minecraft は mod フォルダ | asset / jar mod | 2018 off-sale 検索停止(盗用対策) | 盗用と無断再配布の常態化 |
| CapCut / Canva | 2020 年代 | (大衆向け) | 「Use template」= 穴だけ差し替え | template | 構造は固定、素材だけ自分 | 意味は template 作者の物。作る側は増えない |
| ニコニコ動画(配布側) | 2007– | — | 投稿時にコンテンツツリーの親作品を登録、「オリジナル作品表明」 | 動画 + 親作品 ID | — | 道具から直接投稿は無い(AviUtl → 手動アップ) |

## 仕掛けの詳細(引用付き)

### うごくメモ帳 — 本体で描く・本体で投稿・本体で見る・落として編集

- 投稿・DL・星が全部 DSi 側: 「Through the Nintendo DSi client, users were able to download Flipnotes to their DSi, upload their own Flipnotes, and add stars」 — https://en.wikipedia.org/wiki/Flipnote_Studio
- 派生は「落として編集」: 「Users could also "spin off" another user's Flipnote, by downloading it and editing it.」 — 同上
- 作者が選ぶ禁止: 「If you wish to prevent other people from altering the Flipnote after you've sent it, select Lock.」 — https://en-americas-support.nintendo.com/app/answers/detail/a_id/3996/
- 監視は待たせない設計: 近藤(はてな)「something that would not have to be monitored 24 hours a day, something that wouldn't make users wait for approval」 — https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Nintendo-DSi/Volume-7-Flipnote-Studio-Creation/3-Keeping-It-Safe/3-Keeping-It-Safe-1049372.html
- 通報率: 近藤「About one in ten thousand would report works that they thought weren't appropriate」 — 同上。星を減点なし(plus-minus 案は「マイナスには理由が要る」と退けた: 「if you gave someone a minus, you would need to give them some kind of reason」同上)
- 時間差公開: 「once a certain amount of time has passed, it's deemed acceptable for a broader audience and can be viewed on a Nintendo DSi」 — 同上(web で先に晒し、DSi へは遅れて出す = 子どもの画面を守る)
- 形式: PPM = 「animation frame bitmaps with simple I-frame and P-frame video compression, and ADPCM-encoded audio tracks」 — https://en.wikipedia.org/wiki/Flipnote_Studio。有志の保存: https://archive.org/details/flipnotes_hatena2009-2013

### Scratch — Share アイコンが UI の一等地、remix に自動で系譜

- 「The concept of sharing is built right into the Scratch user interface, with a prominent Share menu and icon」 — Resnick et al., CACM 2009, https://web.media.mit.edu/~mres/scratch/scratch-cacm.pdf
- 「when someone remixes a project, the website automatically adds a link back to the original project, so that the original author gets credit」 — 同上
- 荒れた経緯: 「some Scratchers were upset when their projects were remixed, complaining that others were "stealing" their projects」 — 同上
- 「More than 15% of the projects on the website are remixes」 — 同上
- 方針: 「we plan to keep our primary focus on lowering the floor and widening the walls, not raising the ceiling」 — 同上
- 現行 UI: 「Once shared, the project will display a little notice reading "Thanks to [creator of the original project]"」 — https://en.scratch-wiki.info/wiki/Remix

### PICO-8 — カートが絵、絵がソース

- 「Using .p8.png filename extension will write the cartridge in a special image format that looks like a cartridge.」 — https://www.lexaloffle.com/dl/docs/pico-8_manual.html
- 「SPLORE is a built-in utility for browsing and organising both local and bbs (online) cartridges.」 — 同上
- 「Filenames that start with '#' are taken to be a BBS cart id, that is immediately downloaded and run.」 — 同上。読んだ後 ESC でコードが開く = 「見る」と「開く」が同じ動作
- 制約の宣言: 「The harsh limitations of PICO-8 are carefully chosen to be fun to work with, encourage small but expressive designs」 — https://www.lexaloffle.com/pico-8.php(製品頁。manual 内の位置は未確認)

### Figma — 生きたファイルを公開、複製は完全な別物

- 「a public space where you can now publish live design files that anyone in the world can inspect, remix, and learn from」 — 2019-10-22, https://www.figma.com/blog/introducing-figma-community/
- 「This duplicate is an entirely new file, so you won't be able to access any version history, comments, or permissions」 — https://help.figma.com/hc/en-us/articles/360038510873-Duplicate-Community-files
- 800 → 1000 ファイル(2020-02): https://www.figma.com/blog/config-2020-new-feature-announcements/
- 系譜は無い(複製で切れる)。代わりに license(CC BY 4.0)で「元を書け」を人に委ねる

### Dreams — 公開 = 誰でも remix、系譜は自動

- 「A Public version can be remixed, stamped, edited and generally made into something else by anyone.」 — https://docs.indreams.me/en/create/releasing/releasing-your-dreams
- 「The original creator of a Public creation is always known, via Genealogy and creation credits.」 — 同上
- 段階: Private / Playable(作者だけ remix 可)/ Public — 同上。作者が段階を選ぶ = うごメモの Lock と同じ形
- 系譜は web にも出る: https://indreams.me/element/ofWLsrrVVfk/genealogy/remixes

### Blender — 制作ファイルを全部出す、拡張は本体の中から

- Elephants Dream(2006): 制作ファイル約 7GB を CC で配布 — https://orange.blender.org/
- 「The goal of this platform is to make it easy for Blender users to find and share their add-ons and themes」「The platform only offers GNU GPL compliant software」 — https://extensions.blender.org/about/
- 本体統合の理由: 「to integrate the platform in Blender itself, providing a better user experience」。ただし「won't connect to the Internet by default」 — https://code.blender.org/2024/05/extensions-platform-beta-release/

### Ableton — 学びの場から Live へ「持ち帰る」片方向

- Learning Synths: 「Export – turns your creation into a Max for Live synth contained in a Live Set」 — https://www.ableton.com/en/blog/new-in-learning-synths-export-to-live-record-your-creations-and-more/
- Note → Live: Cloud で Note Set が Live の Browser に現れる、8 Set まで — https://help.ableton.com/hc/en-us/articles/6242153675676-Setting-Up-Ableton-Cloud(本文は 403 で取得失敗、検索要約のみ)
- 共有の場は Ableton の外(maxforlive.com・Packs 販売)。道具内に「投稿」は無い

### AviUtl・MMD — 器だけ置く、意味は外で育つ

- AviUtl: プラグインは「.dll を改名した物」を `plugins` に置くだけ(.auf/.aui/.auo/.auc/.aul) — https://ja.wikipedia.org/wiki/AviUtl。2008 拡張編集、2011 Lua で「自由度が大きく広がった」。ニコニコ「リスペクトされた作品」2021 で 1 位、親作品登録 約 21 万 — https://dic.nicovideo.jp/a/aviutl
- MMD: 2008-02-24 VPVP で無償公開。モデル・モーション・ステージが別ファイル — https://dic.nicovideo.jp/a/mikumikudance。「『MMDは素材のフリー公開が当然』とかいう勘違いはしないように」 — 同上(素材の権利は素材作者の規約)
- ニコニコ側の系譜: コンテンツツリー(2011-12、親作品・子作品、「オリジナル作品表明」) — https://dic.nicovideo.jp/a/コンテンツツリー、投稿時登録 https://blog.nicovideo.jp/niconews/159854.html

### 2020 年代の大衆形 — template

- CapCut「Use template」= 穴に素材を入れるだけ — https://www.capcut.com/resource/tiktok-template。作る側の意味は template 作者に固定される。Motolii の「意味は作る人の物」とは逆向き(意味を借りる型)

## 失敗

| 事例 | 何が起きたか | 教訓(道具の側) |
|---|---|---|
| うごメモはてな | 2013-05-31 終了(3DS 版へ)。作品は「YouTube やはてなフォトライフへ保存可」と告知 — https://ugomemohatena.hatenastaff.com/entry/close | 場がサーバに、作品形式は閉じたまま → 有志の .ppm 保存が唯一の遺産。**ファイルが道具の外で読める事**が寿命 |
| Dreams | 2023-09-01 live support 終了。「Dreams and all of its content will remain available on PlayStation」 — https://docs.indreams.me/en/whats-happening/news/dreams-live-support | 書き出し不可 → 系譜も作品も PS の中だけ。Content Usage Terms(2023-07)で外での利用を許したが形式は出ない |
| Figma Community | 複製制限が効かない報告(forum)、削除は事後審査 — https://help.figma.com/hc/en-us/articles/21328699835543 | 「公開 = 誰でも複製」を後から絞ると矛盾が出る。段階(Dreams の Private/Playable/Public)を先に置く方が壊れない |
| MMD | 素材の権利は素材ごと、MMD杯は転載・マイリスト工作で「MMD杯問題」 — https://dic.nicovideo.jp/a/mmd杯問題 | 道具が系譜を持たないと、外の場で毎回もめる |
| AviUtl | `.aup` はプラグイン一式が無いと開けない、作者停止で 6 年凍結 | 共有単位が「ファイル + 環境」だと再現性が人に依存 |
| Roblox | 2018 off-sale 検索停止(盗用) | 市場を先に作ると系譜より盗用対策が先に来る |

## Motolii への問い(決めない、証拠を並べる)

1. **共有の単位は rrd か** — PICO-8 の `.p8.png`(絵の中にソース)、Scratch の `.sb3`(ソース込み)が最も強かった。rrd は rerun の公開形式で外でも読める(うごメモ・Dreams の逆)。問い: rrd 1 本に「編集の意味(Document)+ 札(WGSL)+ 素材参照」が全部入るか。入らないなら AviUtl 型(ファイル + 環境)になる。
2. **札(WGSL の文字)を単位にするか** — Ableton の Learning Synths Export(M4L device 1 個)、Blender の Extension、AviUtl の .anm が「部品の共有」。問い: 札 1 枚を rrd から切り出して別の rrd に落とせるか。MMD の pmx/vmd 分離は「部品の交換が自然」を生んだ。
3. **系譜を文書に持つか** — Scratch(自動リンク)と Dreams(Genealogy)は系譜が道具/場の側にあり、Figma は複製で切った。問い: Document に「親 rrd の id」を 1 つ持つだけで、ニコニコのコンテンツツリーに親作品を自動登録できるか(投稿時の親作品 API はある: https://blog.nicovideo.jp/niconews/159854.html)。
4. **投稿は道具から出すか** — Scratch の Share アイコン、うごメモの本体投稿が共同体の起点。本職の道具(Live・Blender・AviUtl)は持たなかったが、Blender は 2024 に「本体の中で Get Extensions」へ寄せた(既定はオフライン)。問い: Motolii の窓に「投稿」を置くなら宛先は何か(ニコニコ? 自前?)。うごメモの失敗は宛先が 1 つで消えた事。
5. **作者が選ぶ段階** — うごメモ Lock、Dreams Private/Playable/Public。問い: rrd に「remix 可否」の旗を 1 bit 持つか。持たないと Figma 型の後付け制限で矛盾する。
6. **制約を身元にするか** — 3 色・128×128・「天井を上げない」。Motolii は「天井を作らない」(記憶)と逆。問い: 制約でなく **既定値**(色 4〜6・ease 1 本、質感の参照 3 つ)で「Motolii の絵」と分かる様式を作れるか。
7. **意味を借りる型を入れるか** — CapCut template は意味が作者に固定。Motolii の「意味は作る人の物」と衝突。問い: template を入れるなら Scratch の「See inside」型(開けば全部見える・改変できる)に限るか。

## Sources

- Iwata Asks Vol.7 Flipnote Studio (EN): https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Nintendo-DSi/Volume-7-Flipnote-Studio-Creation/3-Keeping-It-Safe/3-Keeping-It-Safe-1049372.html(iwataasks.nintendo.com は証明書エラー、日本語 vol5 は文字化けで未読)
- Flipnote Studio (Wikipedia): https://en.wikipedia.org/wiki/Flipnote_Studio / Lock: https://en-americas-support.nintendo.com/app/answers/detail/a_id/3996/ / 終了: https://ugomemohatena.hatenastaff.com/entry/close, https://www.nintendo.co.jp/ds/dsiware/kguj/announcement/index.html / 保存: https://archive.org/details/flipnotes_hatena2009-2013
- Scratch: Resnick et al. "Scratch: Programming for All" CACM 2009 https://web.media.mit.edu/~mres/scratch/scratch-cacm.pdf / https://en.scratch-wiki.info/wiki/Remix
- PICO-8: https://www.lexaloffle.com/dl/docs/pico-8_manual.html / https://www.lexaloffle.com/pico-8.php
- Figma: https://www.figma.com/blog/introducing-figma-community/ / https://www.figma.com/blog/config-2020-new-feature-announcements/ / https://help.figma.com/hc/en-us/articles/360038510873-Duplicate-Community-files / https://help.figma.com/hc/en-us/articles/21328699835543
- Dreams: https://docs.indreams.me/en/create/releasing/releasing-your-dreams / https://docs.indreams.me/en/whats-happening/news/dreams-live-support / https://www.gdcvault.com/play/1026982/UX-Summit-Expanding-the-Dreamiverse(未視聴)
- Blender: https://orange.blender.org/ / https://extensions.blender.org/about/ / https://code.blender.org/2024/05/extensions-platform-beta-release/
- Ableton: https://www.ableton.com/en/blog/new-in-learning-synths-export-to-live-record-your-creations-and-more/ / https://help.ableton.com/hc/en-us/articles/6242153675676-Setting-Up-Ableton-Cloud(403)
- AviUtl: https://ja.wikipedia.org/wiki/AviUtl / https://dic.nicovideo.jp/a/aviutl
- MMD: https://dic.nicovideo.jp/a/mikumikudance / https://dic.nicovideo.jp/a/mmd杯問題
- ニコニコ コンテンツツリー: https://dic.nicovideo.jp/a/コンテンツツリー / https://blog.nicovideo.jp/niconews/159854.html
- Roblox Creator Store: https://roblox.fandom.com/wiki/Creator_Store / https://devforum.roblox.com/t/introducing-the-new-creator-store/2805975 / Minecraft modding: https://en.wikipedia.org/wiki/Minecraft_modding
- CapCut templates: https://www.capcut.com/resource/tiktok-template
