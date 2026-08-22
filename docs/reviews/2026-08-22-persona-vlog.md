# ペルソナ「実写Vlog編集者」— 最短の欠落洗い出し(2026-08-22)

状態: **調査**(製品コード変更なし)
起点: 利用者の危機感「このままだと空中分解しかけそう。足りてない機能を最短で洗い出す」。
台帳(`next/reference/normal-map.tsv`)を上から潰す運転ではなく、**ワークフローは直列なので
最初に壊れる所が最短の答え**という仮説の下、実写素材を大量に取り込み・間引き・繋ぐ編集者の
10工程を実コード(`next/`、2026-08-22時点で`main`をマージ済み)に対して1歩ずつ「到達可能か」で
判定した。推測は禁止し、各判定は識別子(struct/fn/enum/message名とfile:line)で示す。

---

## 結論(先出し)

**最初に致命的に詰まる所 = 工程7「テロップを入れる」。** Text レイヤーを作成する UI 入口が
**存在しない**(`CreateKind` enum に `Text` variant が無い・`Message::AddLayer` は Solid 固定・
`motolii-menubar` の "New Layer" も同じ)。store 側の型(`LayerSource::Text`/`TextDocument`)と
Inspector の編集 UI(色・スタイル)は実装済みなのに、**新規作成の入口だけが空席** ——
コード自身のコメントがこれを「見送り」と明記している(`next/ui/motolii-browser-pane/src/lib.rs:147-156`)。
工程1〜6は全て「詰まるが迂回できる」なので完走できるが、工程7は迂回不可(外部ツール経由の
画像テロップ焼き込みしかない)。工程8(色)・工程10(SNS縦横比)も同格の空洞だが、直列の
最初の壁ではない。

---

## 工程表(10段)

### 1. 素材をまとめて取り込む(複数選択・フォルダごと)

**通る(一部)**。
- 複数選択: `FileDialogs::pick_import_paths` が `rfd::AsyncFileDialog::new().pick_files()` を呼ぶ
  (`next/shell/motolii-shell/src/file_dialogs.rs:142-157`)。Video/Image/Audio 拡張子フィルタ付き。
  結果は `Message::AdmitPaths(Vec<PathBuf>)` → `Shell::admit` (`next/shell/motolii-shell/src/lib.rs:1258,1478,1929`)。
- OS ドラッグ&ドロップ: `iced::window::Event::FileDropped(path)` → `Message::DropReceived`
  (`next/shell/motolii-shell/src/lib.rs:1184-1192`)。複数ファイルの同時ドロップは iced が1イベント/1ファイルで
  発火するため複数回の `DropReceived` として届き、結局同じ `admit` に合流する。
- **詰まる(フォルダ単位の取り込み)**: `pick_folder` 系 API・`read_dir` を使った再帰取り込みの経路は
  **無い**(`admit`/`file_dialogs.rs` 全体を grep しても folder import 相当のコードは0件)。フォルダを
  ドラッグしても iced の `FileDropped` はファイルパス1本を運ぶ想定で、ディレクトリを展開する処理も無い。
  → **迂回可能**: OS のファイルダイアログで100本まとめて複数選択すれば同じ結果になる(フォルダ単位の
  「同じ場所に固めておいた」効率は失うが、量そのものには耐える)。

**副次的な発見(工程1と4が同じ操作に融合している)**: `admit` は取り込んだ各 path を
**即座にタイムラインへも配置する**(`Intent::AddLayer`+`Intent::SetMeta{ timing: LayerTiming::place(self.session.playhead, ...) }`、
`next/shell/motolii-shell/src/lib.rs:1970-1989`)。「ライブラリにだけ入れる」独立した取り込みは無い。
しかも複数ファイルをまとめて取り込むと、ループ内の `start` は**全ファイル共通で `self.session.playhead`**
(`next/shell/motolii-shell/src/lib.rs:1936`)——つまり N 本まとめて取り込むと N 本が**同じ時刻に重なって**
置かれる(AE のレイヤー取り込みと同型)。連番のクリップ列にはならない。工程4で詳述。

### 2. 素材を見分ける(サムネイル・並べ替え・検索・お気に入り/タグ)

**通る(部分)**。
- サムネイル: `next/ui/motolii-browser-pane/src/lib.rs`・`model.rs` にサムネイル描画あり。
- 並べ替え: `SortKey`(Name/AddedDate/Kind)+`sorted()` (`next/ui/motolii-browser-pane/src/model.rs:693-705`)。
- 検索: `state.query()`(`next/ui/motolii-browser-pane/src/state.rs:137,184-185`、部分一致・大小無視)。
- **詰まる(お気に入り/タグ)**: `tag`/`Tag`/`favorite` で見つかるのは effects/create カタログの
  `PreviewTag`(プリセットの分類)だけで、**ユーザー素材に対する星付け・タグ付けは無い**
  (`core/motolii-store/src/slot.rs` の `tag` は無関係な直列化属性)。
  → **迂回可能**: ファイル名で運用(リネームは可能かは未確認だが、少なくとも検索で名前は引ける)。
- **詰まる(ライブラリでの複数選択)**: `BrowserState.selected: Option<CardKey>`
  (`next/ui/motolii-browser-pane/src/state.rs:136,180`)——**単一選択のみ**。100本から
  まとめて選ぶ操作が構造的に無い。
  → **迂回可能だが量に耐えない**: 1本ずつ選んで取り込む/配置するしかない。100本規模では
  ここが体感速度の主因になる(ペルソナの核心「回転の速さ」を最も削る箇所)。

### 3. 素材を下見する(Browser でプレビュー再生・イン/アウト点を打つ)

**詰まる(Source Monitor 相当が無い)**。
- `next/ui/motolii-browser-pane/src` 全体を Play/Playback/scrub で grep して**0件**。
  素材カードの click は `Message::SelectCard` のみ(`next/ui/motolii-browser-pane/src/lib.rs:1322`)、
  double-click は create タブの `CreateFromCard` 専用(`next/ui/motolii-browser-pane/src/lib.rs:1572`)——
  media タブのカードに二度押しの意味は無い。
- 台帳側にも「Source Monitor」が採用予定のまま未消化(`next/reference/normal-map.tsv:1034`
  `Source Monitor / 素材クリッププレビューパネル / 採用予定 / B25`)。
- イン/アウト点(work area の mark in/out)はタイムライン上の**コンポジション**に対してのみ存在する
  (`next/ui/motolii-timeline-pane/src/work_area.rs:279,291` の `mark_in_keeps_a_usable_out_and_otherwise_opens_to_the_end` 等)——
  素材そのものへの in/out ではない。
  → **迂回可能**: 先にタイムラインへ置いてから、trim(端8px、`next/ui/motolii-timeline-pane/src/clip_gesture.rs:91,108`)
  と再生(工程9)で下見する。順序を「置いてから選別」に変えれば完走できる。

### 4. タイムラインへ並べる(順に置く・隙間を詰める)

**詰まる(自動の順送り配置が無い)**。
- 工程1で確認した通り、`admit` は複数ファイルを**全て同じ開始時刻**に置く
  (`next/shell/motolii-shell/src/lib.rs:1936-1989`)。Premiere/Resolve/CapCut 的な
  「インサート/上書きで自動的に前クリップの直後へ続く」経路は無い。
- 手動配置は可能: clip の move ドラッグは吸着(スナップ)対象に「他 clip の start/end」を持つ
  (`next/reference/timeline-grammar.md:60` 「スナップ対象= 0秒・終端・playhead・ループ両端・他clipのstart/end・全キー時刻」)。
  → **迂回可能**: 置かれた各クリップを手でドラッグし、直前クリップの終端へスナップさせて
  1本ずつ並べ直す。100本なら100回のドラッグが要る。

### 5. 不要部分を切って捨てる(トリム・Split・削除・詰め直し=リップル)

**通る(トリム・Split・削除)/ 意図的に不採用(リップル)**。
- トリム: `trimmed_in_start`/`trimmed_out_end`(`next/ui/motolii-timeline-pane/src/clip_gesture.rs:91,108`)。
- Split: `Message::SplitAtPlayhead` → `split_at_playhead`(`next/ui/motolii-timeline-pane/src/write.rs:200-203,753`、Cmd+K)。
- 削除: 専用の「Delete」入口は**キーフレーム限定**——Backspace/Delete キーは
  `timeline_pane::Message::DeleteSelectedKeys` にしか配線されていない
  (`next/shell/motolii-shell/src/lib.rs:5432-5439`、`next/ui/motolii-keymap/src/defaults.rs:31-43`)。
  クリップ(layer)を消す唯一の経路は Edit メニュー「Cut」(Cmd+X)= Copy+`Intent::RemoveLayer`
  (`next/shell/motolii-shell/src/menu.rs:104`、`next/shell/motolii-shell/src/lib.rs:463-465,1767-1784`)。
  → **通るが迂回**: 「削除」ではなく「切り取り」を使うことになる(クリップボードを汚す・意味的に妙)が、
  実害はない。
- **リップル削除(詰めて削除)は意図的不採用**: `timeline-grammar.md` 拘束1
  「trim family(ripple/roll/slip/slide/insert/overwrite/lift/extract/sync lock)は採らない」
  (`next/reference/timeline-grammar.md:8,10`)。台帳側も明記(`next/reference/normal-map.tsv:169`
  `Ripple Delete / リップル削除(詰めて削除) / 不採用 / 理由: trim family(ripple)不採用 — 拘束1`、
  同様に id 197/209/210/211/276/277/278/279、marker側 id 741 も裁定175で「ripple編集不採用による
  空洞化」と自認)。
  → **迂回可能**: クリップを Cut/Delete した後、後続クリップ群を選択(行クリック Shift=範囲、
  `next/reference/timeline-grammar.md:79`)して一括ドラッグし、直前クリップの終端へスナップさせれば
  隙間を閉じられる。100本規模では手間が線形に増えるが、**編集は成立する**(不成立ではない)。

### 6. 音を整える(音量・フェード・BGM とナレーションのバランス)

**通る**。
- Inspector の AUDIO section が Level/Pan/Fade In/Fade Out の4行を持つ
  (`next/ui/motolii-inspector-pane/src/audio.rs:3-4,30,42-43`)。レイヤー単位なので、
  BGM レイヤーと音声レイヤーを別々に Level 調整すればバランスは取れる。
- 実再生は本物の音声デバイスへ出力する: `transport::open_real_playback` が
  `motolii_audio::AudioProgram::from_view` を組んで `PlaybackSession::open_default` を開く
  (`next/shell/motolii-shell/src/transport.rs:92-113`)。Space キーで `toggle_playback`
  (`next/shell/motolii-shell/src/lib.rs:2609-2627`、`Named::Space` 配線は`lib.rs:5665`)。

### 7. テロップを入れる(短いテキストを何枚も)— **最初の致命的な詰まり**

**詰まる、迂回不可(アプリ内)**。
- store 側の型は実装済み: `LayerSource::Text`(`next/core/motolii-store/src/lib.rs` 内、
  `next/shell/motolii-shell/src/lib.rs:613,4486,4498` が resolved 側で `LayerSource::Text` を扱う)。
  Inspector にも `text.rs`/`color.rs`(テキストの fill/stroke 色編集、`next/ui/motolii-inspector-pane/src/color.rs`)
  が存在する。
- しかし**新規作成の入口が無い**:
  - Browser の create タブが作れる種類は `CreateKind::{Rectangle, Ellipse, Solid, Null}` の4つのみ
    (`next/ui/motolii-browser-pane/src/model.rs:341-350`)、Text は無い。
  - メニュー「New Layer」(`next/shell/motolii-shell/src/menu.rs:168`)は `Message::AddLayer` に固定配線され、
    中身は常に `LayerSource::Solid{ rgba:[80,160,220,255], ... }`
    (`next/shell/motolii-shell/src/lib.rs:1409-1424`)。Text 版の分岐は無い。
  - コード自身がこの空席を認めている: 「テキストレイヤー作成の map 行(id 1284「New text layer」)は
    …verdict も既に『採用済』(store 側 LayerSource::Text/TextDocument 実装済みを指す)——
    この切片の EXACT TARGET の範囲外……CreateKind の新 variant 追加は
    motolii-shell::create_from_card の match 網羅性を壊す cross-crate 変更になるため、見送り」
    (`next/ui/motolii-browser-pane/src/lib.rs:147-156`)。
- **迂回不可の理由**: 工程2〜6は「手間が増えるが完走できる」代替経路が残っていたが、テロップは
  作成そのものの入口が無いため、アプリ内には代替操作が存在しない。外部の画像/動画編集ツールで
  テロップを画像として焼き込み、Media として import する(`LayerSource::Media` は画像も動画も
  同じ variant で通す設計、`next/core/motolii-store/src/lib.rs:188-196` のコメント参照)ことは
  構造的に可能だが、これは「Motolii の外でテロップを作る」ことであり、ペルソナが求める
  「短いテキストを何枚も素早く打つ」高速反復には使えない。

### 8. 色を軽く整える

**詰まる(調整系エフェクトが無い、迂回可能=スキップ)**。
- `next/engine/motolii-compositor/src/effects/` は `glow.rs` のみ。Inspector 側の `effects.rs` が
  持つのは Glow(ブルーム)の param 群だけ(`next/ui/motolii-inspector-pane/src/effects.rs:59-101`)。
  Brightness/Contrast/Saturation/Exposure/LUT/Curves のいずれも grep 0件。
- 新規追加された `color.rs`(2026-08-22 マージで追加、`next/ui/motolii-inspector-pane/src/color.rs`)は
  **テキストの文字色編集専用**(`text_style_color`/`set_text_style_color_channel`/`commit_text_style_color`
  など、`TextDocumentStyle` にしか触れない)——クリップの色調整とは無関係。
  → **迂回可能**: 色を諦めても書き出し自体は完了する(必須工程ではない)。

### 9. プレビュー(通しで見る・音付き)

**通る**。工程6で確認した `open_real_playback`(`next/shell/motolii-shell/src/transport.rs:97-113`)が
映像・音声を同時に再生する。映像側は vsync 駆動(`iced::window::frames()`、裁定166、
`next/shell/motolii-shell/src/transport.rs:115-129`)で、Space 1つで通し再生ができる。

### 10. 書き出し(1本・SNS 用の縦横比があるか)

**通る(基本の1本)/ 詰まる、迂回不可(縦横比の選択)**。
- 基本の書き出し: `CONTAINER_CODEC_LABEL = "MP4 / H.264"` の1択(`next/ui/motolii-export-pane/src/lib.rs:132`)、
  解像度は `composition.width/height` をそのまま表示(`next/ui/motolii-export-pane/src/lib.rs:364`)——
  1本書き出す基本機能自体は成立する。
- **SNS 用アスペクト比は無い**: 書き出し解像度はコンポジション寸法に完全従属していて、
  export 側に独立の crop/reformat やアスペクト比プリセットは無い(`export-pane/src/lib.rs` に
  aspect/resolution 選択 UI は0件)。さらに遡ると、**コンポジション自体の width/height を
  変更する UI が存在しない**——`NewProjectConfirmed` は `reset_document()` → `Self::default_document()`
  を呼ぶだけで(`next/shell/motolii-shell/src/lib.rs:1639-1647`)、新規プロジェクト時に寸法を選ぶ
  ダイアログは無く、`SetCompDimensions` 相当の Intent も grep 0件。縦(9:16)・正方形(1:1)の
  コンポジションを作る手段がプロジェクトの生成時点から存在しない。
  → **迂回不可**: 16:9 固定でしか書き出せない。ペルソナが「SNS用の縦動画」を作ろうとした瞬間、
  プロジェクト作成の最初の一歩から詰まる(工程10相当だが、実際には工程開始前の制約でもある)。

---

## 最初に致命的に詰まる所

**工程7「テロップを入れる」**。工程1〜6は全て遅い・不格好な迂回はあっても最終的に
「動画1本が完成する」ところまで到達できる。工程7だけはアプリ内に代替操作が一切なく、
Text レイヤーの新規作成入口が構造的に空席(`CreateKind` に variant が無い・唯一の
「New Layer」メニューは Solid 固定)。実写Vlogにテロップ抜きで公開する編集者は稀であり、
ここが「回転の速さ」を主張する製品として最初に致命傷になる工程である。

## 迂回可能な詰まり(重い順)

1. **クリップの詰め直し(リップル)不在**(工程5、拘束1で意図的不採用) — 手動選択+ドラッグ+
   スナップで代替可能。100本規模では手間が線形に増えるが編集は成立する。
2. **ライブラリの複数選択が無い**(工程2、`Option<CardKey>`) — 1本ずつ選ぶしかなく、
   量に対する耐性が最も削られる箇所。
3. **取り込みが順送り配置にならない**(工程4、全ファイル同一開始時刻) — 手動ドラッグで
   並べ直せば解決するが、「並べる」が最初から二度手間になる。
4. **フォルダ単位の取り込みが無い**(工程1) — 複数選択ダイアログで代替可能。
5. **Source Monitor 相当が無い**(工程3) — 先に置いてから下見する順序に変えれば代替可能。
6. **色調整エフェクトが無い**(工程8) — 必須工程ではないためスキップで完走できる。
7. **削除の直接キーが無くCutを流用**(工程5) — 実害は小さいが意味的に不自然。

## 迂回不可の詰まり(致命度が高い順)

1. **テロップ(Text レイヤー新規作成)** — 工程7。外部ツール経由の画像焼き込みでしか回避できない。
2. **SNS用アスペクト比(縦・正方形コンポジション)** — 工程10、実際にはプロジェクト作成時点の制約。
   16:9 固定でしか書き出せない。

## この1本が通るための最小実装(順序つき)

1. **Text レイヤーの新規作成入口を1本通す**(最優先・工程7の解消)。
   `LayerSource::Text`/`TextDocument` は実装済みなので、`CreateKind::Text` variant の追加+
   `motolii-shell::create_from_card` の match 網羅+デフォルト `TextDocument` 生成のみで済む
   (コード自身が経路を明示済み、`next/ui/motolii-browser-pane/src/lib.rs:147-156` 参照)。
   店じまいコスト最小の一手。
2. **Browser の複数選択**(工程2/4の効率を同時に底上げ)。`selected: Option<CardKey>` を
   集合へ拡張し、選択済み全件をまとめて `admit` 相当の配置経路へ渡す。これだけで
   「100本から選ぶ」「まとめて置く」の両方が楽になる。
3. **取り込み時の順送り配置**(工程4)。`admit` のループで `start` を毎回
   `self.session.playhead` 固定ではなく、直前に置いた layer の終端へ前進させる
   (`LayerTiming::place` の呼び出し引数を変えるだけで済む可能性が高い、
   `next/shell/motolii-shell/src/lib.rs:1936,1984-1989`)。
4. **コンポジション寸法の選択 UI**(工程10)。New Project 時か Settings pane に
   width/height(または 16:9/9:16/1:1 プリセット)を選ばせる Intent を1本追加する。
   Export 側は既に comp 寸法をそのまま使っているので(`export-pane/src/lib.rs:364`)、
   この1本で書き出しの縦横比問題も同時に解ける。
5. **リップル(隙間閉じ)の再検討は最後でよい**。拘束1は利用者裁定であり動かすべきでは
   ないが、「選択クリップ群を1コマンドで前へ詰める」ような**move の合成操作**(既存の
   move+snap を束ねるだけ、trim family の新設ではない)なら拘束1と衝突せずに工程5の
   手間を減らせる余地がある——ただし今回の調査範囲外の設計判断なので提案に留める。

---

## 逸脱

- 発注は「各判定は grep で識別子を示す」だったため、実行時の挙動確認(実機ビルド/画面操作)は
  行っていない。全判定は静的なコード読解(struct/fn/enum/message 名の追跡)によるもの。
  `cargo build`/`cargo test` は実行していない(読むだけの調査という規律のため)。
- 工程3「イン/アウト点を打つ」について、素材そのものへの in/out 概念が無いことは確認したが、
  「タイムラインに置いた後の trim が実質的に in/out 点として機能するか」は grammar 文書の記述
  (`timeline-grammar.md`)に基づく推定であり、実機操作では確認していない。
- `next/reference/normal-map.tsv`(1552行)は全件を読んでおらず、本調査で言及した行のみを
  grep で抽出した。台帳全体の整合性監査は範囲外。
