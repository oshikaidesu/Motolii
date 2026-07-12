# プラグイン生態系(発見・導入・再現) — v2設計

作成日: 2026-07-12  
状態: **設計草案**(実装はヒーロー誕生後/v2。[concept.md](concept.md)「プラグインファーストの範囲」と整合)  
関連: F-9 / G-2 / G-3・エコシステム死、[plugin-authoring.md](plugin-authoring.md)、backlog V2-1 / V2-8

この文書は、AviUtl/AE圏で起きている **「地図が無い」「置き場が非直感」「口伝えだけ」** への構造的回答である。  
**motolii が売らない・配らない・審査しない**前提で、「存在する」「入れられる」「作品で揃えられる」だけを担う。  
**コミュニティ成長の主回路は「使用セットの書き出し → 他人がインポート」**である(§5.9)。DL数ランキングではない。

> **規律:** 各節は必ず参考元を明示する。独自発明を既定にしない。未決は「未決」と書き、もっともらしいデフォルトで埋めない。

---

## 1. 問題定義

| 痛点 | 現場の典型 | motolii での扱い |
|---|---|---|
| **地図が無い** | プラグインは個人サイト・Discord・Xの口伝え。横断検索不能 | 分散索引(tap)の購読・マージ表示 |
| **導入が非直感** | AE/AviUtl: 正しいフォルダへ手コピー+再起動 | D&D / 追加ボタン。置き場をユーザーに教えない |
| **再現不能** | 「あの人と同じセット」が口伝え | 使用セットの書き出し / インポート(`plugins.lock.toml` · kit) |
| **コミュニティが育たない** | おすすめが口伝えか中央ランキングのみ | **セット共有が伝播単位**(§5.9)。記事はセットへのリンクを貼る |
| **おすすめの責任** | 中央ランキングや公式キュレーションが枯れる/偏る | 記事は note 等の外部。索引は `links` だけ持つ |
| **作者が人質になる** | 中央 bot / 不透明な内部状態。壊れたら LLM でも直せない | テキスト正本 + CLI `verify`/`repair`/`doctor` |

**やらないこと(明示):**

- 中央 `index.json` 一枚の恒久運用(AviUtl2 カタログ型の正本集中)
- ダウンロード数・人気順・トレンド(要ホスト + telemetry)
- 決済・バイナリ CDN・審査制マーケット
- **Civitai Manager 系の自己満足 UI**(§5)

**ダウンロード数を正義にしない理由:** 累積DL・人気順は「既に知られているもの」をさらに押し上げる。**ゲームチェンジャーが現れた瞬間に過去の指標になる** — 新しい尖りは母数が無いので一覧の下に沈み、口伝えか外部記事が無ければ発見されない。地図の役割は存在と語彙(id/tags/category/updated)であり、人気の再生産ではない。キュレーションの時間軸は note / Awesome 側に置く。

### 1.1 粒と全部入りのダブリ(不安への回答)

魚眼(粒)と VHS(全部入り)がカタログ上に並ぶと、**同じ歪みが二重に見える**不安は正当。これはバグではなく **F-8(意図単位)と拡張の粒の緊張**そのもの。

| 層 | 役割 | 例 |
|---|---|---|
| **look(意図)** | ユーザーが選ぶ完成形。パラメータは少ない | VHS、シネマティック、歌詞テロップ一式 |
| **primitive(粒)** | 組み合わせ部品。パワーユーザー・作者・kit 向け | 魚眼、色収差、スキャンライン単体 |

**方針(採用):**

1. **両方を禁止しない。** 中央で「どちらか一方だけ」と審査しない(ホストレスと矛盾する)。
2. **ユーザー向けの第一級は look。** ファーストパーティと authoring 規約は既存どおり意図単位([concept](concept.md) F-8 / [plugin-authoring](plugin-authoring.md) §4)。原子の組み立てをユーザーに強いない。
3. **ダブリの解消は「 dedupe サーバ」ではなく役割分担 + kit。**
   - 見た目が不可分でチューニング済み → **1 プラグイン(look)** として出してよい(VHS を魚眼+ノイズ+…に分解してユーザーに組ませない)。
   - 部品として意味がある → **primitive** として出し、全部入りは **kit**(§5.9)で「魚眼+走査線+…」と渡す道を優先する(中身の再実装を避ける)。
4. **発見語彙で層を分ける(任意タグ規約)。** 例: `tags` に `look` / `primitive`(小文字・既存 tags 規則)。一覧の既定フィルタは look 寄りでもよいが、消さない。
5. **関連の宣言はリンクのみ(任意)。** `motolii.toml` に `related = ["vendor.filter.fisheye"]` 程度。自動統合や排他はしない(未決を埋めない)。

**やらない:** 中央が「この VHS は魚眼を内包しているから魚眼を一覧から落とす」— 作者の意図も kit の選択も壊す。

**作者への一文:** 同じ見た目を「単体プラグイン」と「全部入りの中の隠し機能」の両方にメンテし続けない。部品にするなら primitive を公開して kit で組むか、look 一つに閉じるかを選ぶ。

**他ソフトが粒ノードに寄る理由(理解の固定):** ユーザー同士が同じ歪み・同じ色処理を**毎回べた書き再発明**するのを止めるため(合成グラフで再利用)。車輪の再発明コストが、意味的ダブリより痛い、という判断。

**誤読の禁止 — 粒推奨 ≠ GLSL/WGSL フィルタ(look)の排除:**

| 誤読 | 実際 |
|---|---|
| 「ノードで組め」= 一枚シェーダのエフェクトは二級 | **否。** look(VHS 等)の一発 Filter / 一枚 WGSL は F-8 の第一級。ユーザーに露出する単位そのもの |
| 粒に寄る = モノリシックシェーダ禁止 | **否。** 不可分な見た目は一枚に閉じてよい(§1.1-3) |
| Cavalry/ノード勢の逆張り(F-8) = 部品を作るな | **否。** 部品は作者・kit・パワーユーザー用。**組み立てをユーザーに強制しない**だけ |

motolii の合成は **エフェクトスタック**(AE 的)が主で、Cavalry 型の原子ノードグラフをユーザーに強いない。スタック上の1段が「魚眼」でも「VHS(中で複数パス)」でも、どちらも正当な Filter。GLSL/WGSL を書くこと自体は歓迎(契約は wgpu/WGSL、F-9)。排除しているのは「技術者向けプリミティブ組み立てを唯一の正規ルートにする」ことだけ。

### 1.2 似た VHS が何個も生まれる問題 — 他コミュニティは「回避」していない

**運営(ホスト作者)が似たプラグインを間引く仕事ではない。** ホストレス設計と両立しないし、審査は枯れる。

他エコシステムも **意味的な類似(似た VHS が N 個)** はほぼ放置し、別の層で耐えている。

| コミュニティ | 似たものが増えたとき実際にやっていること | やっていないこと |
|---|---|---|
| **ReaPack** | 作者/リポジトリ単位で分ける。Browse で絞る。obsolete は「索引から消えたパッケージ」の衝突処理([User guide](https://reapack.com/user-guide)) | 「似たスクリプトを公式が1つに統合」しない |
| **npm / crates.io** | スコープ(`@author/…`)。検索・README。使う側が選ぶ | 同機能パッケージの dedupe |
| **VS Code Marketplace** | publisher.id。推奨リスト・ワークスペース推奨 | 類似拡張の統合削除(むしろ publisher 移行で二重が残る事例あり) |
| **AviUtl / AE** | フォルダ分け・作者サイト・口伝え。導入済みだけ見る | 中央が類似エフェクトを潰す |
| **ComfyUI** | custom node 乱立を前提。workflow 共有が選択装置 | ノードの意味的ユニーク制約 |

つまり回避策の本体は **「世界を一意にする」ではなく「ユーザーの視界を狭く保つ」**。

motolii での耐性(採用方向):

1. **視認性(UI)** — 探すの既定は簡潔な一覧。作者名・`id`・tags を一目で区別。似た display_name でも `vendor.*` が違うことが見える(ReaPack / npm 同型)
2. **ユーザー棚(フォルダ / コレクション)** — 導入済み・お気に入り・自分で名付けたフォルダにだけ日常の選択を閉じる。全カタログは「地図を開いたとき」だけ。AE/AviUtl の「自分の Plugin フォルダ」感覚を、パス暗記なしで再現する
3. **kit(§5.9)** — 「どの VHS を使うか」は個人のセット共有で伝播。運営のおすすめ棚にしない
4. **外部記事** — note が「今はこの3つ」と書く。時間軸は外に置く(§1 DL数否定と同根)

**やらない:** 類似度スコアで一覧から自動除外、公式「正規 VHS」認定、似たものをマージする運営キュー。

```
全tapの海(似たVHSが並んでよい)
        │ 探す = たまに開く地図
        ▼
ユーザー棚(フォルダ/お気に入り/導入済み)  ← 日常の選択はここだけ
        │ 渡す = kit
        ▼
他ユーザーの棚
```

棚のスキーマ(フォルダ名の保存先・プロジェクト紐付けかグローバルか)は未決(§10)。UI 要件としては「導入済み / お気に入り / ユーザーフォルダ」の三段があれば足りる見込み。

### 1.3 界隈のガラパゴスと kit(AviUtl の記憶)

必須プラグインが界隈ごとに違う(歌詞勢・VHS勢・スクリプト勢…)のは **避けられないし、消す対象でもない**。共通セットを運営が決めるとホストレスと矛盾し、尖りも死ぬ。

痛いのはガラパゴス自体ではなく、AviUtl で覚えのある **「何を入れれば同じ絵になるか分からない / 手置きが地獄」** の方。

| 現象 | 扱い |
|---|---|
| 界隈ごとに必須が違う | **許容**(ユーザー棚・外部記事・作者別 kit) |
| 新人・友達が同じ環境に入れない | **kit / lock で改善**(§5.9)。「この界隈の入口」ファイルを渡す |
| プロジェクトを開いたら足りない | 警告 → **揃える**(未知 id パススルー + Sync) |

```
界隈A kit ──┐
界隈B kit ──┼─→ ドロップして揃える → その人の棚が一時的に同じになる
作品の lock ─┘
```

運営は「公式必須セット」を持たない。界隈のガラパゴスは **kit がコピー可能な入口になる**ことで初めて健全になる。

### 1.4 なぜ他ソフトは kit 相当を厚く作らないか / 地図と人気の切り分け

#### 他ソフト側

| ソフト | 近いもの | 薄い理由(推定) |
|---|---|---|
| AviUtl / AE | ほぼ無し | プラグイン=フォルダへ手置きが前提。再現単位が「ファイル集合」のまま公式化されなかった |
| ReaPack | repo 単位の一括 | 「作者リポジトリ」が既にセット。ユーザー自作の横断 kit は第一級にしていない |
| VS Code | `extensions.json` | **ある**が、マーケットの主役は個別拡張の検索・人気 |
| Homebrew | Brewfile | **ある**が、CLI 文化。GUI 地図の主役ではない |
| npm | package.json | プロジェクト単位。エフェクト見た目の共有とは別問題 |
| ComfyUI | workflow JSON | 作品ごと。プラグイン集合の「界隈入口」としては弱い |

共通して、**ストアのKPIが「個別パッケージのDL/インストール」**だと、セット共有は売上・計測の邪魔になりやすい。motolii は売らない・DL数正義を採らないので、kit を主回路にできる(§5.9)。

#### kit が地図に載らない危険

**ある。** 地図(Layer B tap)のピンは package。kit は伝播単位(Layer C / ユーザー生成)なので、既定では一覧に並ばない。結果:

- 新人は「探す」ばかり見て、界隈 kit に辿り着けない
- kit だけが Discord に流れ、地図と分断する

**緩和(採用方向・運営はホストしない):**

1. package / tap の `links` に kit URL を載せる(作者・キュレーターが任意)
2. **kit 専用の個人 tap**(Awesome 型): エントリが kit を指す。地図に「セット」が並ぶのはこの経由
3. UI: 探すに「パッケージ | セット」切替は任意。セットはローカルに取り込んだ kit + `links` で知った URL
4. プロジェクト警告の「揃える」が **lock 経由の強制入口**(地図を通らなくても再現できる)

kit を中央ギャラリー化しない(§5.9)。載せるなら分散のまま。

#### 「地図に人気順が必要」への答え

体感としての **「今なにが生きているか」** は必要。  
採らないのは **ホスト集計の累積DL/トレンド**(§1・AviUtl2 telemetry 反面教師)。ゲームチェンジャーで陳腐化し、サーバーが要る。

| 信号 | 置く場所 | 備考 |
|---|---|---|
| 名前 / 更新日 / タグ | **地図(アプリ内)** | 既定ソート。ホスト不要 |
| 編集部の「今月これ」 | **外部**(note) または **個人 tap の並び** | 人気の代理を編集が担う。時間軸あり |
| GitHub star 等 | **外部・任意表示** | 公式ランキングにしない(以前合意) |
| 「何人の kit に入ったか」 | 中央集計は **やらない** | やるならユーザーローカル(自分が取り込んだ kit 内での共起)まで |

**結論:** 人気の熱は **外部(と編集タップ)に託す**。地図本体は存在・語彙・updated。人気順をアプリの既定ソートに戻さない。  
「託す」= motolii が note をホストするのではなく、`links` と個人 tap と kit 伝播で外の熱を指差せるようにする、という意味。

#### 既存例(ある / 近い / 無い)

kit・地図・人気の切り分けはゼロから発明ではない。部品ごとに前例がある。

| 欲しいもの | 既存例 | 何が近いか | 足りない点 |
|---|---|---|---|
| **使用セットを渡して揃える** | [Homebrew Bundle](https://github.com/Homebrew/homebrew-bundle)(`Brewfile`) | 一式ファイル → 一括導入 | CLI。GUI 地図の主役ではない |
| 同上 | VS Code **Profiles** / `extensions.json` / Profile Sync 系拡張 | 拡張一覧の書き出し→取り込み | マーケット個別人気が主発見路のまま |
| 同上 | Unity `Packages/manifest.json` | プロジェクトが依存を固定 | エフェクト界隈の「入口 kit」UI ではない |
| **界隈のセットが地図に載る** | **Steam Workshop Collections** | 個別アイテムの海とは別に「コレクション」が第一級。URL 一つで購読 | 中央ホスト + コレクション自体に購読数 |
| 同上(薄い) | ReaPack の **repository 単位** / repo リストの import | 「この作者一式」は repo URL で渡せる | ユーザー横断の自作 kit は第一級でない |
| **人気・熱をカタログ外に置く** | GitHub **Awesome リスト** + star | 編集リストが発見の熱。レジストリ本体は別 | インストールと直結しない |
| 同上 | note / ブログの「おすすめ○選」 | 時間軸つきキュレーション | アプリ外 |
| **作品に紐づく再現** | ComfyUI workflow、AE プロジェクト+手置き | 開くと足りないものが分かる | プラグイン手置きが地獄(AviUtl 記憶) |
| **素材の集め** | AE Collect Files / Blender Gather Resources | 「依存を一袋に」 | プラグイン環境セットではない |

**総合すると:**

- 「セット共有」単体 → **Brewfile / VS Code Profile が最も近い**(motolii kit の直接参考)
- 「セットが地図に並ぶ」→ **Steam Collection が最も近い**(ただし中央ホスト。うちは個人 tap + `links` でホストレス近似)
- 「人気は地図の累積DLにしない」→ **Awesome + 外部記事**が前例。Steam は逆にプラットフォーム内人気が強い

「全部入りの公式機能としてクリエイティブ系ホストが揃えている」例は薄い。だから AviUtl/AE で困った、が残る。motolii はそこの穴を kit で埋める側。

---

## 2. 参考元マップ(一次)

| 領域 | 採る | 採らない | 出典 |
|---|---|---|---|
| 購読・同期・索引形式 | ReaPack の repository URL / Sync / index | 中央ストア | [Index Format](https://codeberg.org/cfillion/reapack/wiki/Index-Format), [reapack-index](https://github.com/cfillion/reapack-index) |
| 索引の自動生成 | repo 走査 → index 生成(作者がローカルで回せる) | bot 専用の秘匿パイプライン | [reapack-repository-template](https://github.com/cfillion/reapack-repository-template) |
| プロジェクト固定 | Cargo.lock / npm lockfile 思想 | 「導入済み」だけが真実 | 既存 workspace 慣習 |
| **セット共有(コミュニティ)** | Homebrew Bundle / VS Code extensions 推奨 / Brewfile 的な「一式ファイル」 | 中央のおすすめ棚 | [Homebrew Bundle](https://github.com/Homebrew/homebrew-bundle)、VS Code `extensions.json` |
| パッケージ宣言 | `motolii.toml` ≒ package.json + ReaPack package 要素 | — | npm / ReaPack |
| 外部キュレーション | Awesome リスト + ReaPack `metadata/link` | アプリ内おすすめ枠 | GitHub Awesome 慣習 |
| 種別・検索語彙 | 既存 `NodeDesc` / `PluginKind` | 自由文字列の別語彙 | [plugin-authoring.md](plugin-authoring.md) §2 |
| オープン配布 | ソース + ホスト側ビルド(F-9) | ベンダー/OS固有バイナリ必須 | concept F-9 |
| インストール検出 | コンテンツハッシュ再計算 | 不透明な内部ハッシュ DB | AviUtl2 カタログの XXH3 **思想のみ** |
| git 導入の配管 | ComfyUI Manager の「node を git で揃える」 | その Gradio UI | ComfyUI Manager(設計参考) |
| 中央カタログ | — | 単一正本・人気順・telemetry | AviUtl2 `aviutl2-catalog-data`(反面教師) |
| **UI 見た目** | ReaPack の3画面 + 自前アセットブラウザの簡潔さ | **Civitai Manager / Browser+ のカード祭り** | §5 |

---

## 3. 三層モデル

```
Layer C  外部キュレーション(note / Awesome / YouTube)
         motolii は URL を開くだけ
Layer B  Tap(索引レポジトリ) … 複数を購読・マージ
Layer A  Package Repo(作者の実体) … ソース / メタ / 商用リンク
         ▲
         │ プロジェクト固定
    plugins.lock.toml
```

用語:

| 用語 | 意味 | 参考 |
|---|---|---|
| **tap** | 購読可能な索引1本(`tap.toml` の URL) | ReaPack repository / Homebrew tap |
| **package** | 発見単位。`id` は `NodeDesc.id` と同一 | ReaPack package |
| **lock** | プロジェクトが要求する一式の固定 | Cargo.lock |
| **kit** | ユーザーが名前を付けて書き出した使用セット(プロジェクト非依存でも可)。中身の正本は lock と同型 | Homebrew Bundle / VS Code extensions 推奨リスト |
| **pack** | ローカル袋(`.motoliipack`)。商用やオフライン用 | —(中身スキーマは未決) |

---

## 4. スキーマ草案

### 4.1 `motolii.toml`(Package Repo・Layer A)

参考: npm `package.json` + ReaPack package メタ。  
ランタイムの真実は Rust の `NodeDesc`。`motolii.toml` は発見・導入用の鏡。乖離は CI/`motolii plugin verify` で赤。

```toml
[package]
id = "vendor.filter.glow"     # NodeDesc.id
param_version = 1             # NodeDesc.version
display_name = "Glow"
kind = "filter"               # PluginKind
category = "Color"
tags = ["glow", "bloom"]

[authors]
name = "Vendor"
url = "https://github.com/vendor"

[source]
repo = "https://github.com/vendor/motolii-glow"
directory = "crates/glow"     # 省略可

[distribution.open]
build = "cargo build -p glow --release"

[distribution.commercial]
purchase_url = "https://booth.pm/ja/items/xxxx"
# バイナリ URL は載せない

[[links]]
rel = "article"               # ReaPack link の拡張(website/donation/screenshot に article/video)
url = "https://note.com/..."
title = "紹介記事"
```

### 4.2 `tap.toml`(Layer B)

参考: ReaPack `index` 要素(1 URL = 1 index)。形式は TOML(手編集・Rust 親和)。

```toml
tap_version = 1
name = "motolii-community/jp-filters"
about = "非公式フィルタ集"
homepage = "https://github.com/motolii-community/jp-filters"

[[packages]]
id = "vendor.filter.glow"
display_name = "Glow"
kind = "filter"
category = "Color"
tags = ["glow"]
updated = "2026-07-01T00:00:00Z"

[packages.source]
type = "git"
url = "https://github.com/vendor/motolii-glow"
rev = "a1b2c3d"

[packages.distribution]
lane = "open"                 # open | commercial | dual
```

マージ規則:

- 同一 `id` が複数 tap → **ユーザーが設定した tap 優先順**で解決。衝突は警告表示。
- ソート軸: `display_name` / `updated` / `category` / `kind` のみ。**人気順なし**。

### 4.3 `plugins.lock.toml`

参考: Cargo.lock。Document(プロジェクト JSON)には入れない(既存「文書とキャッシュ分離」)。サイドカー。

```toml
lock_version = 1

[[plugins]]
id = "vendor.filter.glow"
kind = "filter"
param_version = 1

[plugins.source]
type = "git"
url = "https://github.com/vendor/motolii-glow"
rev = "a1b2c3d4e5f6..."

[[plugins]]
id = "vendor.filter.premium_blur"
[plugins.source]
type = "local_pack"
path = "~/.motolii/packs/premium_blur.motoliipack"
```

### 4.4 導入状態 `installed.toml`(テキスト正本)

参考: Civitai Helper の「ハッシュで照合」**思想のみ**。記録形式は TOML。不透明 DB を唯一の真実にしない(§7)。

```toml
[[plugins]]
id = "vendor.filter.glow"
content_hash = "xxh3-128:a1b2c3..."
hash_scope = "artifact"       # artifact | source_tree | pack_manifest
path = "~/.motolii/plugins/vendor.filter.glow/"
source = { type = "git", url = "...", rev = "..." }
recorded_at = "2026-07-12T05:00:00Z"
```

---

## 5. ユーザー UX

### 5.1 三動詞だけ(第一画面に他を置かない)

| 動詞 | ユーザー行為 | 裏 |
|---|---|---|
| **探す** | 購読済み tap のマージ一覧を絞る | fetch index |
| **入れる** | D&D または詳細の「追加」 | Sync / pack 展開 / Booth へ |
| **揃える** | lock / kit をドロップ / プロジェクト警告から | lock と installed の diff |
| **渡す** | 使用セットを書き出して共有 | kit / lock ファイル生成(§5.9) |

### 5.2 画面骨格(参考: ReaPack)

| ReaPack | motolii |
|---|---|
| Manage repositories | リポジトリ(tap URL・優先順) |
| Browse packages | 探す(一覧+詳細) |
| Synchronize | 同期(差分表示。ユーザー向けは「更新」) |

既存のアセットブラウザ(concept)と並べ、**プラグインは素材と同列の「取得できるもの」**としてサイドバーに置く。ネイティブ巨大マーケット UI は必須ではない。

### 5.3 UI 参考 — 採る / 採らない

**採る(簡潔・目的単一):**

- ReaPack: 一覧・詳細・repo 管理の分離。装飾より機能
- 自前アセットブラウザと同じ視覚言語(Slint)。カード祭りを別デザインシステムで始めない
- Alight Motion 的な「少ない画面で完結」(北極星との整合)

**採らない(反面教師):**

- **Civitai Manager / Civitai Browser+ 系 UI** — カードグリッド・ダウンロード数・プレビュー肥大・自己満足の「マネージャ感」。発見の主役をビジュアルの量に置くと、索引の本質(存在・ID・再現)が埋もれる
- AviUtl2 カタログの**人気順デフォルト** — ホスト前提の発見軸
- AE/AviUtl の「フォルダパスを覚えさせる」導入

Civitai 系から借りてよいのは **「アプリ内で探して入れる」「ハッシュで導入済み判定」** という課題意識まで。**見た目と情報設計は借りない。**

### 5.4 一覧の情報設計

**一次(視線):** `display_name` / `kind`+`category` / `tags`(少数) / レーン(オープン|有料) / `updated`(任意)

**二次(詳細・折りたたみ):** `id`(コピー可) / tap 名 / source URL・rev / `links` / 依存(将来)

**出さない:** popularity / trend / DL 数 / 「公式おすすめ」順位  
(§1: DL数正義はゲームチェンジャー到来で陳腐化する)

### 5.5 Import surfaces(D&D 第一級)

AE/AviUtl の手コピーは反面教師。ドロップ可能な袋を標準化する。

| ドロップ | 動作 |
|---|---|
| `.motoliipack` | 検証 → 展開 → installed / lock 更新 |
| `plugins.lock.toml` / `.motolii-kit` | 「N件を揃える」確認 → Sync |
| `.motolii-tap` / `tap.toml` | 購読追加 |
| `motolii.toml` を含むフォルダ | ローカル package 登録 |

置き場は `~/.motolii/...` に集約。ユーザーにパス暗記を要求しない。

### 5.6 Progressive disclosure(簡略と透明)

ユーザー向けに手順を隠すことと、作者から構造を隠すことは別。

| OK | NG |
|---|---|
| デフォルト配置を覚えなくてよい | 配置先・rev・ログを作者から隠す |
| 完了トーストは短い | 失敗をサイレント |
| 詳細は折りたたみ | `verify`/`repair` が無いブラックボックス |

作者面(設定またはプラグイン「開発者」): 配置フォルダを開く / 直近ログ / CLI 導線 / authoring docs リンク。

### 5.7 プロジェクト文脈

未知プラグイン `id` はロード失敗にしない(警告+パススルー、F-9 / plugin-authoring)。  
警告から「揃える」へ誘導し、編集中にマーケットを開かせない。

### 5.8 商用レーン

探す → 詳細 → Booth 等へ → ユーザーが `.motoliipack` をドロップ。  
決済・DRM は外部。索引にはメタと `purchase_url` のみ。

### 5.9 使用セットの共有 = コミュニティの主回路

**仮説(採用):** ユーザーが自分の使用プラグインをまとめて書き出し、他者がそれをインポートできるとコミュニティが発展する。  
口伝えの終着点がファイルになり、note / Discord / X は **「この kit をドロップして」** で足りる。中央のおすすめ棚や DL 数ランキングより、**伝播単位としてセットの方が強い**(尖った新作も誰かの kit に入れば届く)。

```
[作者A] 作る → tap に載る(存在)
[ユーザーB] 使う → 「このセットを書き出す」→ foo.motolii-kit / plugins.lock.toml
[ユーザーC] ドロップして揃える → 同じ一式が入る
[キュレーター] note に kit へのリンク / gist / GitHub raw
```

| 操作 | 入力 | 出力 |
|---|---|---|
| **書き出し(プロジェクト)** | 現在の Document が参照する plugin id | 隣の `plugins.lock.toml`(rev 固定) |
| **書き出し(キット)** | 導入済みから選択、または「このプロジェクトの一式」 | `name.motolii-kit`(中身は lock 同型 + 任意の title/about/links) |
| **インポート** | lock / kit ドロップ | 不足分だけ Sync / 商用は購入案内 |

参考:

| 参考 | 対応 |
|---|---|
| Homebrew `Brewfile` + `brew bundle` | 一式ファイルを共有して揃える |
| VS Code `.vscode/extensions.json` | 「推奨拡張」をリポジトリに同梱 |
| npm の `package.json` dependencies | プロジェクトに依存を書く |
| ComfyUI の workflow 共有 | 作品単位の再現(うちはプラグイン集合に限定。Document 本体は別) |

**UI 要件(簡潔):**

- プラグイン画面に **「セットを書き出す」** / **「セットを取り込む」**(D&D と同パイプライン)
- 書き出し前に一覧プレビュー(id / lane)。商用が含まれる場合は「要購入」を明示
- 取り込後は verify。失敗は doctor へ(§7)

**やらない:** セットの中央ギャラリーやいいね数。kit の置き場はユーザーの gist / GitHub / 記事添付でよい(ホストレス維持)。

### 5.10 用語の使い分け(lock と kit)

| | lock | kit |
|---|---|---|
| 目的 | このプロジェクトの再現 | 人に渡す・おすすめ一式 |
| 置き場 | プロジェクト隣(サイドカー) | 任意(ダウンロード・添付) |
| スキーマ | `plugins.lock.toml` | 同型 + 任意メタ(`title` / `about` / `links`) |
| 必須度 | プロジェクト保存時に更新してよい | 明示的な書き出し操作 |

kit のファイル拡張子・メタ欄の確定は未決(§10)。中身の plugin 列は lock と共通パーサにする。

---

## 6. 導入パイプライン

### オープン(F-9 第一候補)

```
tap エントリ → git fetch(rev) → build → 登録 → installed.toml 更新
```

AviUtl2 の `download→extract→copy` DSL は、ビルド中心に簡略化した形で参考にする。

### 商用

```
メタ表示 → 外部購入 → local_pack ドロップ → lock に type=local_pack
```

### インストール済み検出

ディスク上の成果物を **毎回再計算可能なハッシュ**で照合(§7)。UI の「使用中」は verify 結果の表示にすぎない。

---

## 7. 見通し(Observability) — ハッシュ腐敗への予防

動機: Civitai Manager 等で内部ハッシュが壊れたとき、LLM に頼んでも直らない経験がある。原因は **正本が不透明・再計算不能・二重管理**。

### 7.1 レイヤ

```
[正本] lock / motolii.toml / tap.toml     … 手編集・git 可
  ↓ resolve
[記録] installed.toml                    … repair で再生成可
  ↓ verify(ディスクから再ハッシュ)
[表示] UI
  ↓
[揮発] cache / ビルド tmp                 … いつ消してもよい
```

### 7.2 CLI(人間と LLM が同じ手順)

| コマンド | 役割 |
|---|---|
| `motolii plugin verify` | 存在 + ハッシュ再計算 + lock 照合 |
| `motolii plugin hash PATH` | 単体再計算(`--explain` で算法・scope) |
| `motolii plugin repair` | cache 破棄 + installed を再スキャンして書き直し |
| `motolii plugin doctor --json` | 診断パック(lock/installed/verify/ログ末尾/スキーマ版) |

破損時は **明示的 repair**。サイレント自動修復を既定にしない。  
ハッシュ不一致で消してよいのは cache と installed の該当行まで。lock とソースは触らない。

### 7.3 ハッシュ契約

- 記録は `算法プレフィックス:hex`(例: `xxh3-128:...`)
- `hash_scope` 必須
- 算法・対象範囲は docs と `hash --explain` で一致すること

### 7.4 一文

> 導入状態の正本はテキスト。不透明な内部 DB を唯一の真実にしない。キャッシュ破損は破棄で回復可能であること。

---

## 8. 作者ワークフロー

参考: reapack-index を作者が自分で回せる点。

1. `motolii.toml` + 実装(`NodeDesc` 一致)
2. `motolii plugin verify`(ローカル)
3. オープン: git push。tap メンテナがエントリ追加 or 作者自前 tap
4. 商用: Booth 配布 + 索引にメタのみ。pack 形式は未決(§10)
5. 索引生成を bot 専用にしない(テンプレ CI は可)

---

## 9. v1 / v2 切り分け

| 段階 | スコープ |
|---|---|
| **v1(今)** | `NodeDesc` の category/tags/kind。静的リンク。配布 UI なし。本設計のスキーマだけ固定可 |
| **v2-alpha** | tap 購読 + 探す(存在表示) + D&D の一部 |
| **v2-beta** | git Sync + lock + build + verify/repair/doctor |
| **v2** | 商用 pack + 動的ロード(V2-1)は別軸で評価 |

実装着手は concept どおりヒーロー後。本ドキュメントの改訂はスキーマ未決(§10)を潰す形で行う。

---

## 10. 未決事項

仕様書改訂で潰すまで実装に入らない。

1. tap 正本 URL の慣例(`raw.githubusercontent.com` 固定か Pages か) — ReaPack は raw git URL
2. `.motoliipack` の中身(zip+manifest のみか、署名か)
3. tap / pack の任意署名(minisign 等)。既定は「URL 選択が信頼境界」(ReaPack 同型)か
4. コミュニティ tap のガバナンス(個人 Awesome 型で足りるか)
6. `motolii.toml`↔`NodeDesc` 乖離検証を作者 CI テンプレに含める範囲
7. プラットフォーム付き成果物が要る段階の `target` フィールド(v1 は開発機固定と整合)
8. **kit のファイル形式**: 拡張子(`.motolii-kit` vs lock 兼用)、任意メタ欄、単一ファイルか zip か
9. **look / primitive タグ規約を必須にするか任意か**、および `related` フィールドのスキーマ化時期
10. **ユーザー棚**: フォルダ/お気に入りの保存場所(グローバル vs プロジェクト)、エクスポートに含めるか(kit との関係)

---

## 11. 既存決定との接続

| 既存 | 本設計 |
|---|---|
| F-9 ソース第一・ベンダーAPI禁止 | オープンレーン = git+build |
| G-2 種別レジストリ | tap/`motolii.toml` の `kind` = `PluginKind` |
| 未知 ID パススルー | プロジェクト警告 → 揃える |
| 文書とキャッシュ分離 | lock / installed は Document 外 |
| 配布は v2 | 本ドキュメントは設計のみ |
| plugin-authoring `id`/`category`/`tags` | 発見語彙の単一ソース |

---

## 12. 改訂履歴

| 日付 | 内容 |
|---|---|
| 2026-07-12 | 初版。分散 tap・lock・D&D・progressive disclosure・observability。UI は ReaPack 基調、Civitai Manager UI は明示的に不採用 |
| 2026-07-12 | DL数正義の否定根拠を追記(ゲームチェンジャー到来で累積指標が陳腐化する) |
| 2026-07-12 | 使用セット書き出し/インポートをコミュニティ主回路として §5.9 に格上げ(kit / Brewfile 参考) |
| 2026-07-12 | §1.1 粒(primitive)と全部入り(look)のダブリ方針: 禁止せず役割分担+kit。中央dedupeしない |
| 2026-07-12 | §1.2 類似プラグイン乱立は他コミュニティも回避せず、ユーザー棚+視認性+kitで耐える。運営は間引かない |
| 2026-07-12 | §1.1追記: 粒ノードは再発明防止が動機。look/WGSL一発Filterの排除ではない(F-8との誤読防止) |
| 2026-07-12 | §1.3 界隈ガラパゴスは消さず、kit/lockを入口にする(AviUtlの手置き地獄の対案) |
| 2026-07-12 | §1.4 他ソフトがkit薄い理由・kitと地図の分断リスク・人気の熱は外部/編集tapへ(累積DLは地図に載しない) |
