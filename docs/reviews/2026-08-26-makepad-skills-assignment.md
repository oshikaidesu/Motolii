# Makepad 2.0 技能の割り振り

作成日: 2026-08-26

状態: **決定**

製品 UI は Makepad のみ。iced / `motolii-shell` は引かない（裁定251–254）。
技能に無い載せ方を発明しない。該当技能を読んでから書く。

## 1. ディスク（2026-08-26 実測 → 同日 14本）

`ls ~/.codex/skills/makepad-2.0-*/SKILL.md` — **14本**（既存3本は上書きせず、公式 `skills/` から残り11本を追加）:

| 技能 | パス |
|---|---|
| `makepad-2.0-design-judgment` | `~/.codex/skills/makepad-2.0-design-judgment/SKILL.md` |
| `makepad-2.0-layout` | `~/.codex/skills/makepad-2.0-layout/SKILL.md` |
| `makepad-2.0-events` | `~/.codex/skills/makepad-2.0-events/SKILL.md` |
| `makepad-2.0-widgets` | `~/.codex/skills/makepad-2.0-widgets/SKILL.md` |
| `makepad-2.0-splash` | `~/.codex/skills/makepad-2.0-splash/SKILL.md` |
| `makepad-2.0-dsl` | `~/.codex/skills/makepad-2.0-dsl/SKILL.md` |
| `makepad-2.0-app-structure` | `~/.codex/skills/makepad-2.0-app-structure/SKILL.md` |
| `makepad-2.0-shaders` | `~/.codex/skills/makepad-2.0-shaders/SKILL.md` |
| `makepad-2.0-theme` | `~/.codex/skills/makepad-2.0-theme/SKILL.md` |
| `makepad-2.0-vector` | `~/.codex/skills/makepad-2.0-vector/SKILL.md` |
| `makepad-2.0-animation` | `~/.codex/skills/makepad-2.0-animation/SKILL.md` |
| `makepad-2.0-performance` | `~/.codex/skills/makepad-2.0-performance/SKILL.md` |
| `makepad-2.0-troubleshooting` | `~/.codex/skills/makepad-2.0-troubleshooting/SKILL.md` |
| `makepad-2.0-migration` | `~/.codex/skills/makepad-2.0-migration/SKILL.md` |

`ls ~/.cursor/skills*/**/makepad*/SKILL.md` — **0本**（Cursor skills に Makepad は無い）。

## 2. 14本の名前と入手先（一次）

名前の閉集合は導入済み `design-judgment` のルーティング表（13本の服从層）と、同じ14本を列挙するリポ README。

入手先（一次・実在確認）: [ZhangHanDong/makepad-skills](https://github.com/ZhangHanDong/makepad-skills) の `skills/`（GitHub API で14ディレクトリを確認。README 表題 "Skills (14)"）。各本の SKILL.md:

`https://github.com/ZhangHanDong/makepad-skills/blob/main/skills/<name>/SKILL.md`

- `github.com/makepad/makepad`（エンジン正本）にこの14本の `SKILL.md` は見当たらない。エンジン側の入口は [AGENTS.md](https://github.com/makepad/makepad/blob/dev/AGENTS.md)（技能パックではない）。
- Cursor 公式 skills / `~/.cursor/skills*` に同名は無い。
- この14本を Makepad org が配布している一次は**出典なし**。ディスク3本の文面は上記リポと同名同役割。

導入コマンドはリポ README が `~/.claude/skills/` への symlink / copy を書く。Motolii の実体は `~/.codex/skills/`（14本。既存3本は上書きしていない）。

## 3. 割り振り

担当欄の作業語は `design-judgment` の Co-load 表と、リポ README の Description から写す。足さない。

| 技能 | ディスク | 担当 | 今の扱い |
|---|---|---|---|
| `makepad-2.0-design-judgment` | **導入済み** | 全 Makepad 作業の最初。判断アンカー。他技能への振り分け | **必須**。UI を書く前に読む |
| `makepad-2.0-layout` | **導入済み** | 寸法・Flow・Fill/Fit・spacing・Scroll。Fill-in-Fit 0px | **必須**。葉の大きさ・並び |
| `makepad-2.0-events` | **導入済み** | 入力・`on_click` / `handle_event` / `MatchEvent` / `ids!` | **必須**。クリック・キー・Rust↔Splash |
| `makepad-2.0-widgets` | **導入済み** | 部品カタログ。View / Button / Label / **Dock** / Modal / PortalList | **必須**。該当技能を読んでから |
| `makepad-2.0-splash` | **導入済み** | Splash・`script_mod!`・ホットリロード・streaming | **必須**。該当技能を読んでから |
| `makepad-2.0-dsl` | **導入済み** | DSL 文法・`script_mod!`・プロパティ・`mod.widgets` | **必須**。該当技能を読んでから |
| `makepad-2.0-app-structure` | **導入済み** | `app_main!`・Cargo・ホットリロードホスト・`App::run` | **必須**。該当技能を読んでから |
| `makepad-2.0-shaders` | **導入済み** | 描画。`draw_bg` / Sdf2d / pixel fn | **必須**。該当技能を読んでから |
| `makepad-2.0-theme` | **導入済み** | 色・フォント・dark/light・`mod.themes` | **必須**。該当技能を読んでから |
| `makepad-2.0-vector` | **導入済み** | SVG・パス・グラデ・tween | **必須**。該当技能を読んでから |
| `makepad-2.0-animation` | **導入済み** | Animator・hover/pressed・状態遷移 | **必須**。該当技能を読んでから |
| `makepad-2.0-performance` | **導入済み** | GC・draw batch・profiling | **必須**。該当技能を読んでから |
| `makepad-2.0-troubleshooting` | **導入済み** | 出ない部品・FAQ・灰色/0px の穴 | **必須**。該当技能を読んでから。Chrome 灰色はこの技能が揃ったあと |
| `makepad-2.0-migration` | **導入済み** | 1.x → 2.0 | **必須**。該当技能を読んでから（製品は 2.0 のみ） |

## 4. UI 作成フェーズ

着手前に読む:

1. `makepad-2.0-design-judgment`（最初）
2. その作業の担当行の SKILL.md（14本とも **導入済み**）
3. 該当技能を読んでから書く。技能に無い載せ方は発明しない

技能に無い載せ方（自前ドック、iced widget、技能外の重ね方）は発明しない。
