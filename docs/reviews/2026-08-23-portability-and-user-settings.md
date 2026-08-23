# User Settings 層と可搬性の設計案(F-1、2026-08-23)

対象: `next/reference/procedures/P3-motion-and-carry.md` 後半「別の日に開く・別マシンで開く・人に渡す」の
【穴】意味が無い 17件、および C-1(保存と復帰)が着地時に自己申告した未着手項目
(`next/shell/motolii-shell/src/document_io.rs:202-212` のコメント、`next/reference/axis/A06-restore.tsv`)。

**コードは1行も書いていない。** 設計案のみ。実装するかどうか・どう割るかは末尾「6. 順序と切り方」、
利用者裁定が要る論点は「7. 利用者裁定が要る論点」、裁定候補は末尾に列挙(採番は総監督)。

---

## 0. 現在地(実装済みの物を先に確定する)

`git merge main` 後に調査。C-1(`773fea08` 波C、`90ac5d03` Cmd+S 実キー配線)は**この文書の発注時点より進んでいた**
— 発注書が前提にした「平の Save が無い」「未保存●が無い」「×ボタンに確認が無い」は**既に塞がれている**
(`next/shell/motolii-shell/src/document_io.rs` 全体、`next/shell/motolii-shell/src/lib.rs` の `title()`/`WindowCloseRequested` 経路)。

C-1 が**残した**物(このレーンの起点):

> `document_io.rs:202-212` のコメントを直接引用:
> 「A06『置き場所が未設計』の答え: **Document には入れない**(裁定46/107、発注書の第一候補どおり)。
> `next/` に `dirs`/`ProjectDirs` 等の User Settings 層がまだ無い(KNOWN.md 実測 grep 0件)ので、
> 新しい依存を足す判断はこのレーンの裁量を超える — 代わりに OS 標準のユーザー設定ディレクトリを
> `std::env` だけで組み、1行(path 文字列)だけを書く最小の sidecar にした。**中身は path だけ**で
> Session(選択/playhead/pane レイアウト)は一切入れていない — 置き場(User Settings 層)自体を
> 新設する設計判断が要るため、このレーンでは着手せず未着手のまま残す」

つまり C-1 は**この文書が答えるべき問いをそのまま名指しして手前で止めた**。今 `next/` にある物:

| 物 | 場所 | 状態 |
|---|---|---|
| `last_project.txt` sidecar(直近1件の project path のみ) | `document_io.rs:219-273`、`~/Library/Application Support/Motolii/last_project.txt` 等 | **在る**(C-1)。path 文字列1行だけ、`#[derive(Serialize)]` 型ではなく素の `std::fs::write`/`read_to_string` |
| `Session`(playhead/選択/Timeline キー選択/折り畳み) | `next/ui/motolii-shell-state/src/lib.rs:31-73` | **在る**(型は完成)。doc が明言:「Document には乗らない」。**永続化経路が無い** |
| `WorkspaceBook<K>`(名前付きレイアウト、save_as/switch_to/delete/names) | `next/ui/motolii-shell-state/src/layout.rs:406-446` | **在る**(型は完成・serde round-trip 試験も緑)。**shell が import すらしていない**(`A06-restore.tsv` 実測、`grep -n "WorkspaceBook" shell/motolii-shell/src/lib.rs` = 0件) |
| pane 分割木の実働側(`pane_layout::Layout`) | `next/shell/motolii-shell/src/pane_layout.rs:191-260` | **在る**が Serialize/Deserialize 無し。起動のたび既定値で作り直す |
| `ui_scale` | `next/ui/motolii-tokens-rs/src/lib.rs:930-975` | debug ビルドのみ `tokens/dimensions.json`(複数 worktree 共有の正本)へ書き戻す。**release は no-op**(個人設定ファイルではない) |
| `auto_save`(Document 本体の自動保存) | `next/core/motolii-store/src/persist.rs:187-224` | **在る**。project 隣の `<name> auto-save/` へ世代保存。**User Settings ではない**(Document のコピー) |
| `dirs`/`directories`/`ProjectDirs` 等の crate | — | **無い**。`next/Cargo.lock` を `dirs`/`directories`/`dirs-sys`/`dirs-next` で grep = 0件(直接・推移的とも)。`rfd`(ファイルダイアログ、既存依存)も引いていない |

---

## 1. Document に入れる/入れないの判定式

### 1.1 先例の抽出

先例は2つ、**同じ論理を独立に2回書いている**:

1. `Asset::status: AssetStatus`(`next/core/motolii-store/src/asset.rs:109-113`)の `#[serde(skip)]` 理由:
   > 「いま参照できるか」の環境の事実。**保存しない**。新規 `Asset` は常に `Unchecked` から始まり、
   > `resolve_status` を呼んだ側だけが更新する
2. `Session`(`next/ui/motolii-shell-state/src/lib.rs:31`)の doc:
   > front だけが持つ状態。**Document の写しは1つも入れないこと**(裁定46/107)

両者に共通する軸は1本: **「同じ Document を別の環境で開いた時、この値は環境ごとに変わって当然か」**。
`AssetStatus` は「このマシンでこのパスが今読めるか」= 変わって当然。`Session.playhead` は
「この人が今どこを見ているか」= 別の人・別の起動では変わって当然。

### 1.2 判定式(2軸)

先例1本だけでは Session の全フィールドと User Settings 候補(pane レイアウト・MRU・キーマップ上書き・
テーマ・UI scale)を割り切れない(`AssetStatus` は「今の環境の事実」だが、キーマップ上書きは
「今の環境の事実」ではなく「この利用者の恒常的な好み」であり、質が違う)。そこで軸を2本に分解する:

**軸A — 意味の正本か、環境/利用者の付帯情報か**
「これが無いと Lottie 書き出し(=作品の意味そのもの)が変わるか?」
- 変わる → Document 側の候補(そのうえで軸Bへ)
- 変わらない → Document に**入れてはいけない**(裁定46/107 の理由そのもの)

**軸B — 別マシン・別利用者へ持ち越すべきか**
「この project ファイルを他人に渡した時、この値も一緒に運ばれてほしいか?」
- 運ばれてほしい(Comp 設定・layer・keyframe・mask・effect パラメータ・track) → Document
- 運ばれてほしくない、または運んでも無意味(このマシンの絶対パス・このユーザーの playhead 位置・
  このマシンの画面サイズに合わせた pane 比率) → User Settings 側

**判定表**:

| 軸A(意味を変えるか) | 軸B(他人へ運びたいか) | 判定 |
|---|---|---|
| 変わる | 運びたい | **Document**(layer/keyframe/mask/effect/track) |
| 変わらない | 運びたくない | **User Settings**(playhead・選択・pane レイアウト・MRU・キーマップ上書き・テーマ・UI scale・last_project) |
| 変わらない | 運びたい(相手にも同じ見え方をしてほしい) | **どちらでもない第3の箱**(後述 1.4) |
| 変わる | 運びたくない | 出現しない想定(意味を変える情報を「他人には見せない」ケースは無い — もしあれば個別裁定) |

軸Aと軸Bが一致しないマスがある(3列目)ことに注意。**「レンダリング結果は変えないが、
他人にも同じ体験をしてほしい」情報**が実在する — 次項。

### 1.4 第3の箱: 「作品に添える推奨」

`WorkspaceBook`(名前付きワークスペース)が典型例。ワークスペースの並びは Lottie 出力の画素を
1つも変えない(軸A=変わらない)が、「この project はこのレイアウトで開くと作業しやすい」という
**作者からその project を受け取る人への推奨**は運ばれてほしい場合がある(Resolve の Page 切替・
Premiere のワークスペースプリセットは共有可能な named preset として製品機能を持つ)。

Motolii は**この箱をまだ持っていない**。今回の設計では**新設しない**(軸4「発明しない」・
GOALS.md の除外リストに抵触はしないが、実装対象を増やす判断は書かない)。
`WorkspaceBook<K>` は**個人のプリセット**(User Settings 側)として扱うのが妥当 — 後述 3.3。

### 1.3 実例への適用(P3/A06 が挙げた全項目)

| 項目 | 軸A | 軸B | 判定 | 根拠 |
|---|---|---|---|---|
| Comp 設定・layer・keyframe・mask・effect・track | 変わる | 運びたい | Document | 既に該当(persist.rs) |
| `AssetStatus`(素材が今読めるか) | 変わらない | 運びたくない | User Settings(でも保存すらしない=Unchecked起点) | 既存判断(`#[serde(skip)]`)。**「今日の答え」ではなく既に在る先例** |
| playhead・選択・Timeline キー選択・折り畳み | 変わらない | 運びたくない | User Settings | `Session` doc の既存判断どおり |
| pane レイアウト(実働の分割木・比率・開閉) | 変わらない | 運びたくない(画面サイズが違えば無意味) | User Settings | 新規判断 |
| `WorkspaceBook`(名前付きレイアウト) | 変わらない | 中間(1.4 参照) | User Settings(個人プリセット扱い) | 新規判断 |
| MRU(最近使ったファイル) | 変わらない | 運びたくない(他人のファイル履歴) | User Settings | 新規判断 |
| キーマップの上書き | 変わらない | 運びたくない(個人の好み) | User Settings | 新規判断 |
| テーマ(Dark/Light) | 変わらない | 運びたくない | User Settings | 新規判断 |
| UI scale | 変わらない | 運びたくない(画面の物理サイズに依存) | User Settings | 新規判断。現行 `tokens/dimensions.json` は誤った置き場(後述 3.4) |
| `last_project.txt`(直近1件の project path) | 変わらない | 運びたくない | User Settings | 既存(C-1) |
| `path_absolute`(素材の絶対パス) | 変わる(無いと素材が解決できない) | **運びたいが、他マシンでは無意味になりうる** | **Document に残す**(現状維持) | 1.5 参照 — 軸Bで割り切れない例外 |

### 1.5 例外: 絶対パスは「Document に入れてはいけない情報」ではない

`Asset.path_absolute` は一見「このマシン固有の事実」に見えるが、**軸Aで変わる**(無いと素材の
解決手段がゼロになる。`path_project_relative` は project フォルダ配下の素材にしか使えない —
`Asset::resolve_status` の解決順序、`next/core/motolii-store/src/asset.rs:229-274`、絶対 → 相対 → 失敗)。
これは「User Settings 行きにできない Document 側の情報」であり、**可搬性の問題は「入れる/入れない」
の判定式では解けない** — 2章(可搬性の設計)で別に扱う。

---

## 2. User Settings 層の形式と置き場

### 2.1 探索範囲(裁定215「既定は借りる」に従い、まず借りられる物を探した)

1. `next/Cargo.lock` を `dirs`/`directories`/`dirs-sys`/`dirs-next` で grep → **0件**(直接依存にも、
   既存依存(`rfd` 0.15.4 等)の推移依存にも無い)。KNOWN.md の既存記述と一致(A06-restore.tsv の
   `#sweep` 行が同じ結果を先に記録済み)。
2. `next/` 内に自前の User Settings 層(`UserSettings`/`AppSettings`/`Preferences` 型)が無いか
   → `grep -rn "UserSettings\|AppSettings\|struct Preferences"` 相当を A06-restore.tsv が既に実測(0件)。
3. `auto_save.rs`(`next/shell/motolii-shell/src/auto_save.rs`)・`document_io.rs` の sidecar 実装
   → 借りられる**作法**(OS 別ディレクトリの決め方・ベストエフォート書き込み)はここに在る。
   借りられる**型**(構造化データの読み書き)は無い(sidecar は path 文字列1行専用)。
4. `tokens`(`next/ui/motolii-tokens-rs`)の `write_ui_scale_to_path`/`save_ui_scale`
   → 借りられない。対象が個人設定ではなく複数 worktree 共有の正本 JSON(`tokens/dimensions.json`)
   なので構造からして別物(3.4 で後述)。

**結論: crate としては新規追加(`dirs`/`directories` のどちらか)が要る。作法(OS別ディレクトリの
決め方・ベストエフォート・存在確認込み読み込み)は C-1 の sidecar から畳んで持ち越せる。**

### 2.2 crate を足すか、C-1 の手組みを流用するか(利用者裁定が要る)

C-1 が書いた `user_settings_dir()`(`document_io.rs:219-241`)は `directories::ProjectDirs::from(...)`
が返す `config_dir()` と**実質同じ3分岐**(macOS: `~/Library/Application Support/<name>`、
Windows: `%APPDATA%\<name>`、それ以外の unix: `$XDG_CONFIG_HOME` or `~/.config/<name>`)を
`std::env` だけで再実装したもの。40行に満たない小さな重複だが、これから載せる物が増える
(Session・pane レイアウト・MRU・キーマップ上書き・テーマ・UI scale)につれ、
- 保存先ディレクトリの決め方
- 読み込み失敗時のフォールバック
- (将来 Windows/Linux 実機検証が要る点)

を自前で保守し続けることになる。`dirs`/`directories` は Rust エコシステムでこの領域の事実上標準
(cargo・rustup 自身が `home`/類似 crate を使う。`directories` は `dirs` の後継 fork で XDG Base
Directory 仕様への準拠が明示的)。**新規依存の追加は裁定215「既定は借りる」の対象そのもの**であり、
このレーン(F-1)の裁量では決め切らない(C-1 が同じ理由で決め切らなかったのと同じ境界)。

→ **裁定候補として提示**(末尾)。ただし**どちらを選んでも設計の他の部分(2.3以降)は変わらない**
(`ProjectDirs::config_dir()` も C-1 の `user_settings_dir()` も同じディレクトリを指す設計にできる)。

### 2.3 形式: 単一 JSON、パスは1本

- ファイル名: `user_settings.json`(sidecar と同じディレクトリ、`last_project.txt` と並置)
- 中身は1つの `struct UserSettings` を `serde_json` で読み書き(project ファイル自体が JSON 系ではなく
  `.rrd` バイナリなので、他形式との統一は不要 — テキストで人が中身を見られる方を User Settings では優先)
- **1ファイルにまとめる理由**: sidecar(path 1行)方式をフィールドの数だけ増やすと、書き込みの
  アトミック性(複数ファイルへの分割書き込みは片方だけ失敗しうる)を個別に扱う必要が出る。
  1ファイルなら `persist.rs::save_atomic` と同じ「一時ファイルへ書いてから rename」の作法を
  1箇所だけに適用すればよい(persist.rs に既存の atomic 書き込みパターンを流用できる — 借りる)

### 2.4 スキーマと後方互換

`Document`(`.rrd`)の doc 自身が「上流の rev を上げると古い project が読めなくなりうる」と自認する
(`persist.rs:1-13`)。User Settings は Document と違って**壊れても致命的ではない**(読めなければ
既定値へフォールバックするだけで、作品は無傷)という非対称性があるため、Document より緩い規律でよい:

- `schema_version: u32` フィールドを先頭に持つ
- 読み込み失敗(パース不能・version 不明)は**エラーにせず既定値へフォールバック**(C-1 の
  `read_last_project_path` が「存在確認込み・失敗時は `None`」とした先例と同じ規律を拡張)
- フィールド追加は additive のみ(`#[serde(default)]`)。削除する時だけ `schema_version` を上げて
  未知の旧フィールドは無視(`serde(deny_unknown_fields)` を**付けない** — これは Document の
  `.rrd` と違い、後方だけでなく**前方互換**(新しいバージョンの Motolii で書いた設定ファイルを
  古いバージョンで開いても壊れない)も緩く担保したいため)

### 2.5 内容(何を載せるか)

1.3 の判定表で「User Settings」判定になった全項目 + C-1 の `last_project` を1本の型へ畳む:

```
UserSettings {
  schema_version: u32,
  last_project_path: Option<PathBuf>,       // C-1 sidecar を畳む(3.1)
  recent_projects: Vec<PathBuf>,             // MRU、新規(上限 N、先頭=最新)
  workspace_layouts: WorkspaceBook<PaneLayout>, // WorkspaceBook を畳む(3.3)
  active_pane_layout: Option<PaneLayout>,    // 「今の」実働レイアウト(pane_layout::Layoutのserde化)
  keymap_overrides: ...,                     // 現状 next/ に無い(キーマップ上書き機構自体が未実装)
  theme: Option<ThemeChoice>,                 // 現状 next/ に無い(既定Dark確定はtokens側、上書き機構は未確認)
  ui_scale: Option<f32>,                      // tokens から移す(3.4)
  window_size: Option<(f32, f32)>,            // 任意、pane レイアウトと同根
}
```

`Session`(playhead・選択・Timeline キー選択)は**この型に含めない** — 1.3 の判定は User Settings 側だが、
「再起動しても前回の再生位置に戻る」は**GOALS M11 が明示的に求めていない**(P3 手順96 は「利用者は
期待する」と書いたが、GOALS.md M11 の文言は「再起動で続きが開く」= project が開くこと止まりで、
playhead 位置までは明言していない)。載せる設計自体は上の型に1フィールド足すだけで対応できるので、
**やるかやらないかは利用者裁定**(末尾7)。

---

## 3. 既に在る物との関係 — 畳めるか

**畳める。新規に作るのではなく、以下の3本を1つの `UserSettings` 型へ収容する設計にする。**

### 3.1 C-1 の sidecar(`last_project.txt`)

`write_last_project_path`/`read_last_project_path`(`document_io.rs:254-273`)を
`UserSettings.last_project_path` フィールドへ差し替える。**動作は変えない**
(存在確認込み読み込み・ベストエフォート書き込みの規律は継承)。差分は「専用ファイル1本」→
「共通ファイルの1フィールド」だけ。MRU(`recent_projects`)を足すなら、保存のたび
`last_project_path` を `recent_projects` の先頭へ push する1行が増えるだけで自然に合流する
(P3 手順93「最近使ったファイル一覧」の穴はここで塞がる)。

### 3.2 `Session`(`next/ui/motolii-shell-state/src/lib.rs`)

**畳まない、または部分的に畳む**(利用者裁定待ち、末尾7)。`Session` 自体は Document 型ではなく
front 専用の実行時状態としてそのまま残す。もし「前回の playhead/選択を復元する」を採るなら、
`Shell::boot` が起動時に `UserSettings` から読んだ値で `Session::default()` を上書きする1本の
経路を足すだけ(`perform_open` が明示的に `Session::default()` している箇所、`lib.rs:1918`、
を条件分岐に変える)。**`Session` の型定義自体に `Serialize`/`Deserialize` を足す必要はない**
(`UserSettings` 側に対応フィールドを持たせ、boot 時に詰め替える形にすれば `Session` の
「Document の写しは1つも入れない」という現行の性質を壊さずに済む)。

### 3.3 `WorkspaceBook<K>`(`next/ui/motolii-shell-state/src/layout.rs`)

**畳める。ただし2段階が要る**:

1. **型は既に完成している**(`save_as`/`switch_to`/`delete`/`names`、serde round-trip 試験も緑)。
   `UserSettings.workspace_layouts: WorkspaceBook<PaneLayout>` として**そのまま**埋め込める
   — 新しい型を発明する必要がない。
2. **shell 側の結線が要る**(A06-restore.tsv が指摘した「型はあるが shell が import すらしていない」
   穴はそのまま残っている)。`pane_layout::Layout` に Serialize/Deserialize を足す(現状 3.4 参照の
   通り無い)のがこの結線の前提条件になる — **`WorkspaceBook` を User Settings へ載せる作業は
   `pane_layout::Layout` の serde 化を内包する**。

### 3.4 `pane_layout::Layout`(実働の分割木) と `ui_scale`(`tokens`)

- `pane_layout::Layout`: Serialize/Deserialize を足し、`UserSettings.active_pane_layout` へ載せる。
  `WorkspaceBook<K>` の型パラメータ `K` にもこの型を使えば、「名前付き保存」と「今の実働状態」が
  同じ型を共有できる(2つの別型を用意しない)。
- `ui_scale`: 現行の書き戻し先(`tokens/dimensions.json`)は**複数 worktree・複数開発者が共有する
  正本 JSON**であり、個人の User Settings ではない(`write_ui_scale_to_path` doc の明言どおり)。
  **この2つを混同しない** — `tokens/dimensions.json` の debug-only 書き戻しは開発時のホット
  リロード機構(裁定117)として現状維持、**利用者が実行時に変える UI scale**は別に
  `UserSettings.ui_scale` を新設し、そちらを個人設定として持つ。今は release ビルドで
  UI scale 変更が保存されない(no-op)穴を、ここで初めて塞げる。

### 3.5 `auto_save`(Document のコピー保存)

**畳まない。** これは Document 本体の複製であって User Settings ではない(1.3 の判定表どおり
軸A=変わる・軸B=運びたい寄り、Document 側の機構)。ただし**クラッシュ復帰の検出**
(`document_io.rs:283-` `recoverable_autosave`)は「このマシンで今読める世代ファイルが在るか」
という**環境の事実**の判定であり、判定結果自体を User Settings へ書く必要はない
(判定は起動のたび `auto_save_dir` を見て毎回やり直せば済む — 保存する意味が無い)。

---

## 5. 可搬性の設計

### 5.1 P3 の17件が指す4つの穴と、Motolii が採る作法

先例を裁定150(既存ソフトで確立した意味を既定とする)に従って調べた。4製品の作法をまとめる:

| 製品 | 機能名 | 何をするか | 出典 |
|---|---|---|---|
| After Effects | Collect Files(File > Dependencies > Collect Files) | project + 参照する footage/画像/音声 を1フォルダへコピー。**フォントは集めない**(レポートには列挙される) | [aejuice.com 解説](https://aejuice.com/blog/how-to-package-after-effects-file/)、[Adobe コミュニティ](https://community.adobe.com/t5/after-effects-ideas/collect-files-fonts/idc-p/13321037)(2026年時点で存在するフォント非対応への改善要望 — 未実装のまま) |
| Premiere Pro | Project Manager(Collect Files and Copy to New Location / Consolidate and Transcode) | 使用中の素材だけを1箇所へコピー。Consolidate は未使用クリップを除外・トランスコードも選べる | [Adobe 公式](https://helpx.adobe.com/premiere/desktop/organize-media/create-projects/consolidate-and-archive-projects.html) |
| DaVinci Resolve | Media Management(File > Media Management) | 使用中メディアを Copy/Move/Transcode。Timeline/bin 単位で対象を絞れる | [steakunderwater.com Resolve 18 手引き](https://www.steakunderwater.com/VFXPedia/__man/Resolve18-6/DaVinciResolve18_Manual_files/part1141.htm) |
| Blender | Pack Resources(File > External Data > Pack All into .blend / Unpack All Into Files) | 外部ファイルを **.blend 単体に埋め込む**(コピーではなく同梱)。動画等一部の重いデータは埋め込めない | [Blender 5.2 Manual](https://docs.blender.org/manual/en/latest/files/blend/packed_data.html) |
| Figma | Missing font ダイアログ | フォントが無いレイヤーへサイドバーにアイコンで気づかせ、モーダルで代替フォントへの一括置換を促す | [Figma Help Center](https://help.figma.com/hc/en-us/articles/360039956994-Missing-font-alert-in-Figma-Design)、[LogRocket 解説](https://blog.logrocket.com/ux-design/handle-figmas-missing-fonts-warning/) |

**共通する作法**(4製品全部が独立に採る形なので、これを Motolii の既定にする — 裁定150 の趣旨どおり):

1. **「集める」対象は project が参照する外部ファイルのみ**(未使用アセットは対象外にできる選択肢を持つ、
   Premiere/Resolve の Consolidate 相当)
2. **フォントは別枠**。AE も「集めない」を明示的に選んでおり、Blender の Pack は動画等一部データを
   除外する前例がある — **「全部を1つに埋め込む」を無条件の目標にしない**(素材はコピー/移動、
   フォントは「無いことを知らせる」までを最低ラインにする、これは Figma の作法)
3. **欠落の見せ方は「まとめる機能」とは別のUI**(Figma はサイドバーの常時可視アイコン+
   モーダルでの一括置換、AE/Premiere/Resolve は「集めた結果のレポート」でしか欠落を示さない)。
   Motolii は Ableton可視性原理(ユーザーメモリ)と一致させ、**常時可視の帯**(status 帯、
   D-3 が `resolve_status` を結線した先と同じ経路)を欠落表示の第一候補にする

### 5.2 4つの穴への対応案

#### (a) 絶対パスが別マシンで解決できない(P3 手順108)

- **現状**: `path_absolute` はコピー元マシンの文字列のまま、`canonicalize` は呼んでいない
  (grep 0件、`asset.rs`/`motolii-media` 双方)。`resolve_status` の解決順序(絶対→相対→失敗、
  本日 A-3/D-3 が結線済み)は既に「相対パスがあれば別マシンでも解決を試みる」設計になっている。
- **不足**: (1) `path_absolute` しか持たない素材(project フォルダ外に置かれた素材)は
  そもそも相対パスを持てないので解決不能 — **これは機構の穴ではなく素材配置の問題**、
  Collect Files 相当(5.1 の共通作法1)が無いと利用者が自衛できない。
  (2) 解決に失敗した状態(`AssetStatus::Missing`)を**利用者に見せる口**が無い(1.3 の
  `AssetStatus` は判定式としては正しく User Settings 側=保存しない設計だが、
  「表示する」は別問題 — D-3 が結線したのは「Replace 操作時」限定で、常時可視ではない)。
  → 5.1(3) の**常時可視の帯**を新設する設計判断が要る(利用者裁定候補)。

#### (b) フォント埋め込みが無い(P3 手順109-110・114)

- 5.1 の作法1〜2 に従い、**「埋め込み」ではなく「気づかせる」を最低ラインにする**
  (Figma 型)。`find_family`(`next/ui/motolii-font-catalog/src/lib.rs:132`)が `None` を返す
  ケースを**そのタイミングで一度**通知する経路を足す(常時ポーリングは不要 — text レイヤーを
  開いた/選んだ時点でのチェックで足りる、Figma もサイドバーアイコンは静的表示)。
  **フォントファイルそのものの埋め込み**(AE ですら「集めない」を選ぶ難所)は今回のスコープから外す
  設計判断を提案する(利用者裁定候補)。

#### (c) プロジェクトを人に渡す手段が無い(P3 手順112-113)

- Collect Files/Project Manager/Media Management に共通する最小形:
  「project が参照する外部素材だけを1フォルダへコピーし、project 内の `path_absolute` を
  コピー先の相対パスへ書き換えた**複製**を作る」。**project ファイル自体を書き換えない**
  (元の project は無傷のまま、複製の方だけ path を書き換える — Save a Copy と同じ「複製」の
  身分を踏襲すれば実装が既存の `perform_save_a_copy` 経路に近い形で乗る)。
- **GOALS.md 除外リストとの衝突確認**: 「要らないもの」節(`next/GOALS.md:80-93`)を全項目照合した。
  trim family・プリコンポ/Nest・ノードグラフ UI・JS式/グラフエディタ・IK/リグ・marketplace/SDK・
  第二 runtime・rerun viewer/egui shell のいずれとも**無関係**(Collect Files 相当はファイル
  I/O とパス書き換えのみで、除外リストが指す「機能の複雑化」領域に触れない)。**衝突なし**。

#### (d) 欠落を利用者に見せる口が無い(P3 手順101-102・normal-map id1393)

- `Find Missing Footage`(id1393、`採用予定`)は台帳に在るが実装ゼロ。5.1(3) の**常時可視の帯**
  (status 帯)が先に要る — 「探し直す」コマンド単体を先に作っても、**そもそも欠けていることに
  気づく手段が無い**なら意味が無い(裁定 194/216 と同じ「入口より状態の可視化が先」のパターン)。
  実装順序は 6章で扱う。

### 5.3 整合性確認(P3 手順116)

4製品とも「渡した後の整合性をアプリ側が機械的に確認する」機能は見当たらなかった(探索範囲: 上記5製品の
公式ドキュメント/フォーラム記事の grep 相当。チェックサム・バージョン一致確認に言及する記述は0件)。
**先例が無い機能**なので、Motolii が単独で発明することになる — 優先度は低いと判断し、
今回の設計案には含めない(P3 手順116 は「意味が無い」のまま残す。利用者裁定は不要、単に後回し)。

---

## 6. 実装の順序と write-set

### 6.1 順序(依存関係)

```
1. UserSettings 型 + 読み書き(次のディレクトリ/atomic書き込みは C-1 sidecar から畳む)
   └─ 2.2 の crate 裁定(dirs/directoriesを足すか)が前提
2. C-1 sidecar → UserSettings.last_project_path への移行(動作不変のリファクタ)
3a. pane_layout::Layout に Serialize/Deserialize
3b. WorkspaceBook<PaneLayout> を UserSettings へ結線(shell の import)
4. Session の一部(利用者裁定7で決まった範囲)を boot 時に UserSettings から復元
5. status 帯へ「素材/フォント欠落」の常時可視表示(5.1(3)・5.2(a)(d)の前提)
6. Find Missing Footage / Find Missing Fonts の実コマンド(5 の後、5が無いと無意味)
7. Collect Files 相当(独立、1〜6と依存関係なし)
```

### 6.2 write-set 案(幹の軸で切る、今日の方法に倣う)

| レーン | 対象 | write-set(触るファイル) | 他レーンとの関係 |
|---|---|---|---|
| **G-1** | `UserSettings` 型の新設+読み書き+sidecar 統合(手順1-2) | 新規 crate または `next/shell/motolii-shell/src/document_io.rs` 拡張、`next/Cargo.toml`(依存追加) | 他の全レーンの前提。**最初に単独で終わらせる**(直列) |
| **G-2** | `pane_layout::Layout` serde 化 + `WorkspaceBook` 結線(手順3) | `next/shell/motolii-shell/src/pane_layout.rs`、`next/ui/motolii-shell-state/src/layout.rs`、`lib.rs` の import 部のみ | G-1 完了後。G-3/G-4 と write-set が交わらない(pane_layout.rs 以外は触らない) |
| **G-3** | Session 復元(手順4、利用者裁定7の範囲による) | `next/shell/motolii-shell/src/lib.rs`(boot 経路のみ) | G-1 完了後。G-2 と同時に走らせても `lib.rs` の同じ関数(`boot`)を触る可能性があるため**要調整**(G-2 は import 追加のみに留めれば衝突回避できる) |
| **G-4** | 欠落の常時可視表示(手順5) | `next/ui/motolii-inspector-pane` or status 帯を持つ pane、`next/shell/motolii-shell/src/lib.rs`(status 更新箇所) | G-1〜G-3 と独立。並列可 |
| **G-5** | Find Missing Footage/Fonts 実装(手順6) | G-4 が触った status 帯の消費側 | G-4 完了後 |
| **G-6** | Collect Files 相当(手順7) | 新規モジュール(例 `next/shell/motolii-shell/src/collect.rs`)+ menu.rs への1エントリ | 完全独立。**いつでも並列可能**(G-1〜G-5 のどの状態にも依存しない) |

G-1 が唯一の直列ボトルネック(全員が `UserSettings` 型を触るため)。G-1 の対象を「型定義+ファイル
I/O のみ」に絞れば数十行規模で終わり、G-2〜G-6 は G-1 完了後に4レーン並列できる(G-6 だけは
G-1 の完了すら待たずに始められる)。

---

## 7. 利用者裁定が要る論点(勝手に決めない)

1. **`dirs`/`directories` crate を新規依存として足すか**、C-1 の手組み(`std::env` 直書き)を
   拡張し続けるか(2.2)。**推奨**: 足す(理由: エコシステム標準・保守コスト・XDG準拠の正確性)。
   ただし新規依存の追加はこのレーンの裁量を超える(C-1 と同じ境界)。
2. **前回の playhead/選択(`Session`)まで復元するか**、project を開くところまでで止めるか(2.5・3.2)。
   GOALS M11 の文言は「project が開くこと」までしか明言していない。P3 手順96 の「利用者は期待する」は
   調査者の推測であり確定要件ではない。
3. **「まとめて渡す」(Collect Files 相当)を実装対象に含めるか**、また含める場合
   **フォント埋め込みまで踏み込むか、通知止まりにするか**(5.2(b)(c))。AE ですら埋め込みを避けている
   難所であり、スコープを広げる前に利用者の要否確認が要る。
4. **`WorkspaceBook`(名前付きワークスペース)を今回の User Settings 層へ含めて結線するか**、
   それとも型だけ残して見送るか(3.3)。GOALS.md には対応する明記が無く、normal-map id1517 は
   「戻らない」判定のまま(A06-restore.tsv)。

---

## 裁定候補一覧(採番は総監督)

- **候補X**: User Settings 層の置き場は OS 標準のユーザー設定ディレクトリ、形式は単一 JSON
  (`user_settings.json`)、`schema_version` を持たせ additive-only で進化させる(2.3-2.4)
- **候補Y**: `dirs` または `directories` crate を新規依存として採用し、C-1 の手組み
  `user_settings_dir()` を置き換える(2.2、利用者裁定1と対）
- **候補Z**: 可搬性(Collect Files 相当)の作法は AE/Premiere/Resolve/Blender/Figma の共通項
  ——「使用中の外部素材だけをコピー、フォントは埋め込まず通知止まり、欠落は常時可視の帯で示す」
  ——を Motolii の既定とする(5.1)
- **候補W**: 素材/フォントの欠落表示は「操作した瞬間だけ」ではなく status 帯の**常時可視**を
  第一候補にする(Ableton可視性原理の踏襲、5.1(3)・5.2(a)(d))
- **候補V**: `WorkspaceBook<K>` の型パラメータと `pane_layout::Layout` を同一化し、
  「今の実働レイアウト」と「名前付き保存」で型を分けない(3.4)

---

## RETURN 要約

1. **Document に入れる/入れないの判定式**: 軸A(Lottie 出力の意味を変えるか)× 軸B(他人へ運びたいか)の
   2軸(§1.2)。既存の2先例(`AssetStatus` の `#[serde(skip)]`・`Session` の「Document には乗らない」)
   は同じ論理の独立な2回の発見だったと確認。絶対パスは軸Bで割り切れない例外として2章へ分離。
2. **形式・置き場**: OS 標準ユーザー設定ディレクトリ配下の単一 JSON。`dirs`/`directories` crate は
   ワークスペース内に**皆無**(直接・推移的とも grep 0件)——C-1 の手組みを畳んで crate へ乗り換えるか
   は利用者裁定(§2、探索範囲は §2.1 に明記)。
3. **可搬性の作法**: AE Collect Files(フォント非対応)/Premiere Project Manager/Resolve Media
   Management/Blender Pack Resources/Figma missing-font ダイアログの共通項——「使用素材のみコピー・
   フォントは通知止まり・欠落は常時可視」を既定として提案(§5.1、出典つき)。
4. **畳めるか**: `WorkspaceBook`・`pane_layout::Layout`・C-1 sidecar は**畳める**(型は既存、
   `UserSettings` 型へ収容する設計で新規発明ゼロ)。`auto_save` は畳まない(Document のコピーで
   User Settings ではない、§3.5)。`Session` は型を変えず boot 時の詰め替えだけで対応可能(§3.2)。
5. **順序と並列**: G-1(UserSettings 型、直列・単独)→ G-2/G-3/G-4 並列 → G-5 → G-6 はいつでも独立
   (§6)。
6. **利用者裁定が要る論点**: crate 追加の可否・Session 復元範囲・Collect Files のスコープ
   (フォント埋め込みまで含むか)・`WorkspaceBook` を今回含めるか(§7、4点)。
7. **裁定候補**: 5件(§末尾、採番は総監督)。
