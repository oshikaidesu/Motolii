# Agent入口の縮約とiced authority切替（アーカイブ）

- 日付: 2026-08-19
- 状態: **撤回・アーカイブ**
- 対象: `AGENTS.md`、現在の製品host、Codex personal skillの発火境界

## 決定

> 2026-08-19の後続裁定により、repo固有agent制約そのものをroot入口から外した。本書はその直前に行った縮約の歴史記録であり、現行指示ではない。製品hostの現在値は`docs/CANON.md`が所有する。

`AGENTS.md`を常時読み込む最小入口へ縮約する。利用者の依頼境界、現在地、dirty保護、製品状態の正確な報告、icedのDocument編集境界だけを常設し、時点依存のmodel配分、外部LLM手順、phase進捗、歴史説明、全作業へのworktree/Issue/commit/PR/main統合を外す。

現行の製品hostと新規機能targetは`motolii-shell-iced`とする。`motolii-blitz-shell`とegui製品UIはlegacy/referenceであり、Timelineの視覚・機能参照、Rerun Stage島の内部実装、比較・回帰器具として残す。既定bin名やlauncherに機械的な残余があっても、host authorityをeguiへ戻さない。

personal skillは依頼範囲を拡張してはならない。通常の相談や実装へ自動適用されていた`ponytail`、`outcome-rendering-gate`、`reuse-before-scratch`は明示opt-inへ変える。skillの利用は施工、検索、build、検証の独立した許可にならない。

## 根拠

変更前の`AGENTS.md`は25,591 bytesで、廃止済みegui host必須論、日付付きmodel/CLI情報、発注packet、統合運用を常時promptへ重複していた。OpenAIの現行model guidanceも、長期agent sessionでは反復instructionと無関係なtool説明が増幅するため、instructionを一度だけ書き、関係する道具だけを見せるlean promptを推奨している: <https://developers.openai.com/api/docs/guides/latest-model>。

iced側では次が機械化されている。

- `ui_toolkit_dep_policy`がiced shellへのegui依存混入を検査する
- `intent_gateway_fence`がviewからの既知の直接Document API迂回と`Shell::update`外のdispatchを検査する
- drive/replay oracleが既存編集操作のDocument結果とsnapshot再現を検査する

ただしフェンスは既知API名の走査であり、新しい可変APIや第二`ShellGateway`を一般に型で禁止するものではない。またplay/pause、loop、tick、export pollは意図的に`UiIntent`外である。したがって常設する規則は「全状態変更」ではなく、次のDocument編集境界だけに限定する。

`view → Message → Shell::update → UiIntent → ShellGateway/D2 → immutable snapshot`

## 廃止する常時規則

- 全taskでのrepo全体bootstrap、decision index全件確認、正式な採択packet
- 全taskでの専用clean worktree、1 Issue = 1 commit = 1 PR、即main統合
- model名、effort、CLI version、allocation profile、外部provider手順の常時注入
- egui host必須、`egui_tiles` dock必須、Blitz shellだけが製品入口という現在規則
- skillが通常の相談・レビュー・局所実装から追加探索や施工を起動する挙動

必要な場合は各正本または利用者が指定したworkflowをそのtaskだけに適用する。過去の決定文書は歴史証拠として残すが、現在のagent入口にはしない。

## 機械couplingの改訂

- `scripts/check-docs.sh`は旧3 markerと30KB上限を廃止し、最小入口の3境界と6KB上限を検査する
- UI用語文書の読む順序は`docs/README.md`で検査し、`AGENTS.md`へ二重掲載しない
- Inspector read-model guardから、Inspector意味と無関係な`AGENTS.md`と`scripts/check-docs.sh`の固定SHAを外す
- Issue templateは`AGENTS.md`のclosed-order capsuleをmirrorせず、外部発注時だけLaunch cardを指す

## 非目標

- Document/D2、single writer、GPU、色変換、正準座標、Preview/Export、plugin契約を弱めない
- egui実装やBlitz器具をこの変更で削除しない
- icedの未実装能力、視覚忠実度、実機、performanceを完成扱いしない
- launcher/default binの機械変更をこの文書変更だけで実施済みとしない

## 検証

- `./scripts/check-docs.sh`
- `bash -n scripts/check-docs.sh`
- `git diff --check`
- 変更したpersonal skillごとの`quick_validate.py`

Rust製品codeは変更しないため、この決定単独ではworkspace buildや実窓検証を要求しない。
