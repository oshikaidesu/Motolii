# プラグイン生態系(発見・導入・再現) — v2設計

作成日: 2026-07-12  
状態: **設計草案**(実装はヒーロー誕生後/v2。[concept.md](concept.md)「プラグインファーストの範囲」と整合)  
関連: F-9 / G-2 / G-3・エコシステム死、[plugin-authoring.md](plugin-authoring.md)、backlog V2-1 / V2-8

この文書は、AviUtl/AE圏で起きている **「地図が無い」「置き場が非直感」「口伝えだけ」** への構造的回答である。  
**motolii が売らない・配らない・審査しない**前提で、「存在する」「入れられる」「作品で揃えられる」だけを担う。

> **規律:** 各節は必ず参考元を明示する。独自発明を既定にしない。未決は「未決」と書き、もっともらしいデフォルトで埋めない。

---

## 1. 問題定義

| 痛点 | 現場の典型 | motolii での扱い |
|---|---|---|
| **地図が無い** | プラグインは個人サイト・Discord・Xの口伝え。横断検索不能 | 分散索引(tap)の購読・マージ表示 |
| **導入が非直感** | AE/AviUtl: 正しいフォルダへ手コピー+再起動 | D&D / 追加ボタン。置き場をユーザーに教えない |
| **再現不能** | 「あの人と同じセット」が口伝え | `plugins.lock.toml` |
| **おすすめの責任** | 中央ランキングや公式キュレーションが枯れる/偏る | 記事は note 等の外部。索引は `links` だけ持つ |
| **作者が人質になる** | 中央 bot / 不透明な内部状態。壊れたら LLM でも直せない | テキスト正本 + CLI `verify`/`repair`/`doctor` |

**やらないこと(明示):**

- 中央 `index.json` 一枚の恒久運用(AviUtl2 カタログ型の正本集中)
- ダウンロード数・人気順・トレンド(要ホスト + telemetry)
- 決済・バイナリ CDN・審査制マーケット
- **Civitai Manager 系の自己満足 UI**(§5)

---

## 2. 参考元マップ(一次)

| 領域 | 採る | 採らない | 出典 |
|---|---|---|---|
| 購読・同期・索引形式 | ReaPack の repository URL / Sync / index | 中央ストア | [Index Format](https://codeberg.org/cfillion/reapack/wiki/Index-Format), [reapack-index](https://github.com/cfillion/reapack-index) |
| 索引の自動生成 | repo 走査 → index 生成(作者がローカルで回せる) | bot 専用の秘匿パイプライン | [reapack-repository-template](https://github.com/cfillion/reapack-repository-template) |
| プロジェクト固定 | Cargo.lock / npm lockfile 思想 | 「導入済み」だけが真実 | 既存 workspace 慣習 |
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
| **揃える** | lock をドロップ / プロジェクト警告から | lock と installed の diff |

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

### 5.5 Import surfaces(D&D 第一級)

AE/AviUtl の手コピーは反面教師。ドロップ可能な袋を標準化する。

| ドロップ | 動作 |
|---|---|
| `.motoliipack` | 検証 → 展開 → installed / lock 更新 |
| `plugins.lock.toml` | 「N件を揃える」確認 → Sync |
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
5. `motolii.toml`↔`NodeDesc` 乖離検証を作者 CI テンプレに含める範囲
6. プラットフォーム付き成果物が要る段階の `target` フィールド(v1 は開発機固定と整合)

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
