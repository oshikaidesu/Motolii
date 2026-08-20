# 裁定144-b: Ravel Timeline操作文法抽出 — RESEARCH_RETURN(未達成)

## 結論
Ravel(Apache-2.0のRust製MV特化エディタ、Motoliiの兄弟プロジェクト)のリポジトリを特定できず、クローン・読解に進めなかった。

## 試行した経路

1. **直接クローン**: `git clone --depth 1 https://github.com/ravel-app/ravel`
   → `remote: Repository not found.` (プロンプト内で示唆されたURLは実在しない)

2. **WebSearch(6クエリ)**:
   - `Ravel rust video editor MV github open source`
   - `"Ravel" GitHub Rust "video editor" Apache-2.0 timeline`
   - `"Ravel" "music video" editor Rust github timeline crate`
   - `Ravel editor rust egui timeline keyframe github`
   - `"Ravel" rust editor "music video" github.com apache license timeline crate 10k lines`
   - `github.com/ravel-app OR "ravel-app" rust`
   - `"Ravel" 動画編集 Rust MV OSS github`
   - `site:github.com ravel rust "timeline" video editor MV`
   - `Ravel MV editor Rust GitHub Apache 2.0 license 2026`

   ヒットした「Ravel」を名乗るプロジェクトは以下のみで、いずれも動画編集・MVエディタではない:
   - `github.com/kmicklas/ravel` — Rust UIフレームワーク(Elm/React系統、MITライセンス、280 SLoC)
   - `github.com/valyentdev/ravel` — microVMsオーケストレータ(AGPL-3.0)
   - `github.com/raveljs/ravel` — Node.js Webフレームワーク(Rustではない)
   - `github.com/ravel-net/ravel` — SDN関連(ネットワーク研究用、動画と無関係)
   - crates.io上の `ravel` / `ravel-web` / `ravel-support` — いずれも上記UIフレームワーク系列のクレート

   「MV特化」「動画編集」「timeline」「10k行」を含む条件に一致する候補はゼロ件。近縁の一般的Rust動画エディタ(AldaronLau/rust-video-editor, gausian-AI/Gausian_native_editor, zllak/movieru等)もヒットしたが、これらはMotoliiの兄弟プロジェクトとして言及されている「Ravel」とは別物と判断(名称不一致・作者/文脈の手がかりなし)。

## 未実施
- timelineモジュールの特定・読解
- 操作文法の抽出(操作名/起動条件/ドラッグ意味/確定キャンセル/フィードバック/エッジケース/出典file:line)
- 移植候補単位の指摘

いずれもリポジトリ未特定のため着手不能。

## 次アクションの提案(判断はしない、選択肢の列挙のみ)
- ユーザー本人が2026-08-14に発見した際のURL・検索経路・会話ログを遡って再取得する
- GitHubの認証済みAPI検索(このセッションでは未認証WebSearch経由のみで、GitHub code search本体には未アクセス)で `topic:video-editor language:rust` を全件走査する
- 作者名・リポジトリ名が「Ravel」という製品名であって、GitHubリポジトリ名自体は別名(例: 開発コードネーム)である可能性を確認する
