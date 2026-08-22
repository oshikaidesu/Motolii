# 自前実装の棚卸し — `owns:` 32件を「なぜ持っているのか」で検分

日付: 2026-08-23 / 状態: **棚卸し完了**。write-set は本ファイルのみ。コード・
`next/check.sh`・`next/reference/` は1文字も触っていない(2レーン走行中の柵を遵守)。

前提: `git merge main` 実行済み(差分0、already up to date)。

## 0. 根拠と方法

`next/DECISIONS.md` **裁定215**(2026-08-22、利用者裁定「コンセプトすら委託している。
それ自体が根幹」): 既定は「借りる」。`//! owns:` を宣言してよいのは
**(a) それを強制する意見を名指しできる時**、**(b) 上流に相当物が無いことを実際に
探して確かめた時**だけ。どちらも書けないなら借りる。

`grep -rn --include='*.rs' "^//! owns:" next/` = **32件**(自分で数え直した。発注書の
見積もりと一致)。対照として `wraps:`/`borrows:`/`delegates:` を同様に数えると
**17件**(発注書の見積もり「17件前後」と一致)。

判定は発注書の4分類。**(d) が本発注の中心** — 「今すぐ捨てろ」ではなく「立証が
足りない」という記録。

## 1. 棚卸し表

| # | モジュール | 何を持っていると宣言しているか | なぜ持つのか(現在の記述) | 判定 | 根拠 |
|---|---|---|---|---|---|
| 1 | `core/motolii-core/src/lib.rs` | 有理数フレーム時刻(`RationalTime`)・frame記述 | rerun の `TimeInt` は `i64 + TimeType` で 30000/1001 のような有理 fps を正確に持てない、と具体的な型の限界を名指し | (a)/(b) | 「映像編集はここを落とせない」(ドメイン要件)+ 上流型の具体的限界を記述 |
| 2 | `core/motolii-eval/src/lib.rs` | keyframe補間(Hold/Linear/cubic-bezier)・区間分割 | rerun の latest-at は **step補間のみ**——`next/probes/r0-store-edit` の実測(R0-3)で確認済みと明記 | (b) | 主張が実測(probe)で裏取りされている、数少ない例 |
| 3 | `core/motolii-store/src/lib.rs` | Documentの意味(layer同一性・素材指紋・comp時刻解決) | 当初 `wraps: re_entity_db::EntityDb` と名乗っていたが**敵対的レビュー(2026-08-20)で指摘され訂正**——`fingerprint.rs`/`resolve`/`ResolvedLayer` は汎用entity storeが持ちようのないドメイン意味だと明記 | (b) | marker訂正の経緯そのものが「調べた形跡」。ただし発見のきっかけは自己申告でなく外部レビュー |
| 4 | `core/motolii-testkit/src/lib.rs` | 「外部ツール無し環境はskip、CIは落とす」の試験方針 | 上流にこの方針は無い(製品判断)。旧8,106行から使う分だけ移植、`golden/README.md` 等に規約の由来を明記 | (c) | testkit相当、発注書の指示どおり別扱い |
| 5 | `engine/motolii-audio/src/lib.rs` | 音声の意味核(決定論的mix・program組立・クロック・実デバイス再生) | KNOWN.md「音声」節: rodio/kira は cpal のような生callbackタイムスタンプを露出しないため**クロック所有契約(D4/D5)を守れない**と却下理由を明記 | (a) | 上流候補(rodio/kira)を名指しで却下——音声7件のうち最も強い(a) |
| 6 | `engine/motolii-audio/src/clock.rs` | 音声クロック(供給済みサンプル数→タイムライン時刻) | D4/D5契約+旧`motolii-transport`/`motolii-audio`から移植。`PlaybackClock`自体は「新規の口」と自己申告した上で中身は移植済みの演算のみと明記 | (a) | 新規部分と移植部分を明確に切り分けている |
| 7 | `engine/motolii-audio/src/device.rs` | cpal出力(実デバイス経路) | D4契約「ここだけがハードウェアに触る」。中身は旧crateからの型読み替えのみ | (a)、ただし境界寄り | 実質は cpal の薄いラッパー(フォーマット交渉ロジックのみ自前)——`wraps:` に近い。誤分類の可能性を §3 で指摘 |
| 8 | `engine/motolii-audio/src/producer.rs` | mixプロデューサスレッド | KNOWN.md「audio callback内でallocしない」という明記された規律から、mixをバックグラウンドスレッドへ追い出す必然性を説明 | (a) | 規律は名指し、seek対応は新規実装と自己申告 |
| 9 | `engine/motolii-audio/src/ring.rs` | 音声コールバック側の充填ロジック(SPSC本体は`rtrb`) | 旧367行の自前SPSCは**KNOWN.md 2026-08-20判定済みで「再発明」**と明記——上流`rtrb`へ譲り、`rtrb`に無いフレーム境界の充填/無音補填/カウンタ記録だけ残したと具体的 | (b) | 「上流にあるものは譲った」実例。旧実装を潔く切った記録が残っている |
| 10 | `engine/motolii-audio/src/session.rs` | 実デバイス再生セッション(device+producer+clock束ね) | 発注書「旧PlaybackSessionの形を移植——スクラッチ禁止」を直接引用 | (a) | 意見は発注書の明示指示そのもの |
| 11 | `engine/motolii-audio/src/time_map.rs` | クリップローカル時刻→ソース時刻の写像 | `motolii_store::LayerTiming`/`Speed`は**裁定63でcompフレーム単位の整数写像**と決まっており、mixが要る「48kHzサンプルグリッド上の有理数写像」を表せないと具体的に確認 | (b) | 裁定番号つきで「上流(store)に無い」ことを実証した最良の例の1つ |
| 12 | `engine/motolii-engine/src/shape.rs` | `Vec<ShapeNode>`→`render_tree`→RGBA8の橋渡し | `Engine::texture_for`のShape枝が常に`(None,[0,0])`だった「繋がっていない穴」を塞ぐグルー。canvas centered/top-left origin の選択理由を裁定173 H4文脈で説明 | (b)寄り | 汎用レンダラがDocumentの型を知りようがない、という自明ではあるが具体的な橋渡し。opinion名指しは弱め |
| 13 | `engine/motolii-engine/src/text.rs` | `TextDocument`→輪郭→RGBA8 | 裁定190/BL4を引用し、`ResolvedLayer::id`追加で塞がった具体的な穴を説明 | (a)/(b) | 決定番号つきで経緯が追える |
| 14 | `engine/motolii-media/src/lib.rs` | フレーム正確seek付きdecode/probe/encode/mux | **裁定24**: 上流`re_video`(既に依存グラフに在る)を**4点の具体理由**(MP4限定・再生指向API・全メモリ展開・encode/mux無し)で名指しして却下、「4点が変わったら再裁定する」まで明記 | (b) | 32件中**最も強い(b)の実例**。候補を名指しし、具体理由4つ、再検討条件まで書いている |
| 15 | `engine/motolii-media/src/waveform.rs` | timeline波形のpeaks抽出(min/max bucketing) | decode経路は`FrameReader`を再利用と明記されているが、**peak抽出アルゴリズム自体を他crateと比較した形跡が無い** | **(d)** | 小さいユーティリティだが「持つ理由」の記述が無い。既存waveform/peaks crateを探した痕跡なし |
| 16 | `engine/motolii-vector/src/lib.rs` | パス演算子7種(trim-path/repeater/rounded-corners/pucker-bloat/zig-zag/offset-path/twist)+星形パス源 | 「持っている2D crateは無い」と断言するが、**具体的な候補名(kurbo/lyon等)を検討した記述が無い**。移植の正当性(裁定10=再実装より移植優先)は書けているが、それは「旧実装をコピーしてよい理由」であって「自前で持つべき理由」の代わりにはならない | **(d)** | 発注書が名指しした2件のうちの1件。詳細は §2 |
| 17 | `engine/motolii-vector/src/text.rs` | 字形→輪郭(cosmic-text→swash→3次ベジエ) | `next/probes/r6-text-shaping`の実証(裁定190)をほぼそのまま昇格。cosmic-text/swashを使い、自前分は「次数上げ(quad→cubic)」の変換だけと明記 | (b) | probeで実際に検証済み。所有範囲を最小限に絞っている模範例 |
| 18 | `probes/r0-store-edit/src/lib.rs` | R0 probe測定器具 | 「rerun store をDocumentの実体にできるか」を測る一点物 | (c) | probe |
| 19 | `probes/r0-store-edit/tests/r0.rs` | R0合否判定 | 「編集ソフトとして成立するか」の審判は上流に無い | (c) | probe |
| 20 | `probes/r1-frame-throughput/src/lib.rs` | 40層破綻しない、の実測器具 | 合否を測る物は上流に無い | (c) | probe |
| 21 | `probes/r2-view-projection/src/lib.rs` | front=投影のみ、が毎フレーム成立するかの実測 | 同上 | (c) | probe |
| 22 | `probes/r3-pointcloud/src/lib.rs` | PLYをre_rendererの点群として撮れるかの実測 | rerun本体のPLY loaderはviewer層(裁定3で引かない)にしか無い、と明記 | (c) | probe |
| 23 | `probes/r4-widget-timeline/src/lib.rs` | widget timelineの性能実測 | TL-arch survey のEVIDENCE_GAPを埋める器具 | (c) | probe |
| 24 | `probes/r6-text-shaping/src/lib.rs` | R6 probe測定器具(文字列→輪郭の1本道実証) | 依存増分ゼロ(iced forkが既にcosmic-text 0.19を引く)の実証込み | (c) | probe |
| 25 | `shell/motolii-shell/src/auto_save.rs` | 一定間隔で`()`を発行する購読口 | **`next/reference/KNOWN.md`に「`iced::time::every`はこのworkspaceでは使えない」と明記** — forkの`iced_futures`がtokio/smol featureを有効化していないと具体的理由まで記述 | (b) | 上流API(iced)の具体的な機能欠落を検証済み |
| 26 | `shell/motolii-shell/src/transport.rs` | 実時間再生(Play/Pause/seekの状態遷移) | 発注書「旧PlaybackSessionの形を移植」+「デバイス抽象はフェイクで」を両立する薄い口、と自己申告。実体は`motolii_audio::PlaybackSession`への委譲 | (a) | 意見は発注書の明示指示。実装は薄い(ほぼ委譲) |
| 27 | `ui/motolii-export-pane/src/lib.rs` | Exportダイアログの投影view | 「意味は`motolii-export`が持ち、このcrateは読み取り投影しか持たない」と明記。map(B09)由来の行だけを採用し「飾り禁止」の先例(SET+)を引用 | (a)、境界寄り | §3で指摘: 記述内容が事実上「wraps」の形に近い。marker文言は本発注では直さない |
| 28 | `ui/motolii-keymap/src/lib.rs` | (キー+修飾キー+文脈)→動詞idの対応表・解決器 | 「icedはキーイベントを配るだけで、割当を意味づける層はアプリ側の責務」と上流の役割範囲を明記。`timeline-grammar.md`拘束6を引用 | (a)/(b) | GUIツールキットの一般的な役割分担を正確に指摘した上で拘束6と紐付け |
| 29 | `ui/motolii-shell-state/src/focus.rs` | パネルのフォーカス/巡回状態 | `layout.rs`と同じ「Session水準のpane状態」を`PaneKind`だけで疎結合にする設計。循環回避のleaf化が理由 | (a) | 循環依存回避という構造上の必然性(裁定160切片6の survey を引用) |
| 30 | `ui/motolii-shell-state/src/lib.rs` | shell横断の共有状態型(Session/KeySelector/KeySelectionOp) | pane crateとassemblerの共通の親——循環を避けるためのleaf、と明記。**裁定160切片6**と切片7を具体的に引用 | (a) | 発注書の例示にある「循環回避」パターンそのもの。commit hashまで添えている |
| 31 | `ui/motolii-tokens-rs/src/lib.rs` | デザイントークンの読み口(寸法JSON+DTCG色) | 「icedのThemeは色/境界/影しか持てず寸法をTheme化できない」と上流の構造的限界を明記。裁定117(デザイン値の外出し)を引用 | (a)/(b) | 上流(iced::Theme)の型を実際に検討した跡がある |
| 32 | `ui/motolii-verbs/src/lib.rs` | 動詞の正本(id・ラベル・shortcut・入口集合・map行id) | **裁定195(S6=Ableton可視性原理)**が実測した違反パターンを、`static`初期化子のconst評価でコンパイルエラーにする、と明記 | (a) | 32件中最も明確に「意見が強制している」例。名指しされた意見(S6)がコードの構造(const fn強制)に直結している |

## 2. 特に丁寧に見た2件

### `engine/motolii-audio/`(owns 7件)

**判定: 意見が在る。単なる自前実装ではない。**

7件すべてに共通する forcing opinion は KNOWN.md「音声(2026-08-20解析済み)」の
**D4/D5契約**——「クロック所有権(生callbackタイムスタンプ)を持てるのはcpalだけで、
rodio/kiraのような高レベルcrateはこの契約を守れない」。この1点が、なぜ
producer/device/clock/session/ringを自前で持つ必要があるかを一貫して説明している。

加えて個別の借用判断が具体的:
- `ring.rs` — 旧367行の自前SPSCは「KNOWN.md 2026-08-20判定済みで再発明」と**明記の上で
  実際に`rtrb`へ切り替えた**。残したのは`rtrb`に無いフレーム境界充填ロジックのみ。
- `time_map.rs` — store側の型(`LayerTiming`/`Speed`)が裁定63で整数comp-frame単位と
  決まっており、mixが要る有理数サンプルグリッド写像を**表現できないことを確認した上で**
  この crate 自身に置いている。

一方 `device.rs` はやや境界寄り——中身はcpalの薄いラッパー(フォーマット交渉ロジックの
みが自前)で、`wraps:`に近い性質を持つ。ただし「ハードウェアに触るのはここだけ」という
D4契約上の**意図的な集約点**でもあるため、`owns:`のままでも不自然ではない(§3参照)。

結論: 音声7件は「借り先(cpal)はあるのに自前が厚い」という発注書の懸念を検分した結果、
**厚みの理由が名指しされた契約(D4/D5)とrtrbへの実際の切替実績で裏付けられている**。
(d)判定に落ちる物は無かった。

### `engine/motolii-vector/`(パス演算子7種)

**判定: 意味がLottie由来で自前実装が必要なのは当然。ただし実装の借り先を探した形跡は無い。**

`lib.rs`の記述は「上流のラスタライザはパスを塗ることしか知らない」「(これらの演算子を)
持っている2D crateは無い」と断言するが、**具体的な候補名を検討した記述が一切無い**。
`裁定10`(移植は再実装より優先)は「旧`pathgeom.rs`をコピーしてよい理由」であって、
「旧`pathgeom.rs`を書いた時点でその内容を自前実装すべきだった理由」を示す物ではない
——2つは別の問いである。

実際に workspace の `Cargo.lock` を確認すると、**kurbo(3バージョン)・lyon一式
(lyon/lyon_algorithms/lyon_geom/lyon_path/lyon_tessellation)は既に依存グラフに
transitiveに存在する**(next/Cargo.lockで確認)。これらを検討した記述は
`motolii-vector`にもdocs配下にも無い(`grep -rn kurbo\|lyon`はCargo.lockと
`reference/lottie-coverage.tsv`のみヒット、実装検討の文脈では0件)。

結論: 意味側(Lottie互換のパス演算子集合)を自前で持つのは当然だが、**実装側の借り先
検討が欠けている**という点で厳密には(d)。詳細な調査は本発注の範囲外(§4参照)。

## 3. `owns:`/`wraps:` 境界の観察(逸脱ではなく記録)

`core/motolii-store/src/lib.rs`が自ら記す通り、marker は**crateの根しか見ない**規律
(`check.sh`)なので、`wraps:`を名乗ったcrateの中に`owns:`相当の実装が混ざると規律が
空振りする——逆に`owns:`を名乗ったcrateの大半が実質wrapsでも、量としては数えられて
しまう。今回の棚卸しで気になった2件を記録するに留める(**marker文言は直していない**):

- `engine/motolii-audio/src/device.rs` — 中身はcpalの薄いラッパー。D4契約の集約点という
  理由はあるが、`wraps: cpal`寄りの性質。
- `ui/motolii-export-pane/src/lib.rs` — 自己申告が「意味は`motolii-export`、この crate は
  読み取り投影しか持たない」——文言そのものが他の`wraps:` crate(例:
  `motolii-timeline-pane`「書き込みはIntent経由のみ、Documentの写しを持たない」)と
  ほぼ同型。

どちらも「捨てる/直す」判断はしていない——気づいた事実の記録のみ。

## 4. § 借りられるかもしれない物

(d)と判定した2件について。**思い当たる候補のみ挙げる。無理に探していない**
(詳細調査は別発注)。

- **`engine/motolii-vector/src/lib.rs`(パス演算子7種)** — `kurbo`(弧長計算・
  オフセット曲線を持つ)・`lyon_algorithms`(弧長ウォーク=trim-pathの基礎になりうる)
  はどちらも既にCargo.lockのdependency graphに(他crate経由で)存在しており、**新規
  依存の追加なしで検討できる**候補。ただしrepeater(アフィン冪コピー)/pucker-bloat/
  zig-zag/rounded-corners(fillet置換)のような Lottie/AE 固有の modifier 一式を
  1つのcrateがまとめて持っている見込みは薄く、**部分的な借用(弧長・オフセットだけ)**
  に留まる可能性が高い。
- **`engine/motolii-media/src/waveform.rs`(peaks抽出)** — 明確な候補は思い当たらない
  (`symphonia`はデコードのみでpeak抽出はアプリ側の仕事、専用のwaveform crateは
  エコシステム上マイナー)。**無し**として記録する。

## RETURN

- commit hash: `a0a4c88a`(作業開始時点のHEAD、merge mainは差分ゼロで変化なし)
- `owns:` 総数: **32件**(発注書の見積もりと一致)。対照の`wraps:`/`borrows:`/`delegates:`は
  **17件**(見積もりと一致)
- 判定内訳: **(a) 14件 / (b) 8件 / (c) 8件 / (d) 2件**
  - (a): motolii-core/lib.rs, audio/lib.rs, audio/clock.rs, audio/device.rs,
    audio/producer.rs, audio/session.rs, engine/text.rs, shell/transport.rs,
    export-pane/lib.rs, keymap/lib.rs, shell-state/focus.rs, shell-state/lib.rs,
    tokens-rs/lib.rs, verbs/lib.rs(複数は(a)/(b)混在、表の判定列参照)
  - (b): motolii-eval/lib.rs, motolii-store/lib.rs, audio/ring.rs, audio/time_map.rs,
    engine/shape.rs, media/lib.rs(**最強例**), vector/text.rs, shell/auto_save.rs
  - (c): motolii-testkit/lib.rs + probes 7件(r0 lib/tests, r1, r2, r3, r4, r6)
  - **(d) 2件**: `engine/motolii-media/src/waveform.rs`(peak抽出、借り先候補「無し」)、
    `engine/motolii-vector/src/lib.rs`(パス演算子7種、借り先候補=kurbo/lyon一式・
    部分借用の見込み)
- 音声(`engine/motolii-audio/`)の判定: **意見が在る**。D4/D5契約(cpalだけがクロック
  所有に足る)が7件全体を一貫して説明し、`ring.rs`は実際にrtrbへ切替済み、
  `time_map.rs`は裁定63で store 側に代替が無いことを具体的に確認済み。ただし
  `device.rs`は`wraps: cpal`に近い性質を持つ境界事例として記録した(§3)。
- ベクタ(`engine/motolii-vector/`)の判定: 意味(Lottie準拠のパス演算子集合)を持つのは
  当然だが、**実装の借り先(kurbo/lyon)を検討した形跡が無い** — (d)。両方とも既に
  Cargo.lockのdependency graphに在るため、新規依存追加なしで検討可能(§4)。
- § 借りられるかもしれない物: kurbo / lyon_algorithms(部分借用見込み、vector向け)、
  waveform peaks抽出は候補「無し」
- 逸脱: 無し。write-setは本ファイルのみ、コード・`check.sh`・`next/reference/`は不読
  ではなくgrep/Read専用で確認しただけで編集していない。
