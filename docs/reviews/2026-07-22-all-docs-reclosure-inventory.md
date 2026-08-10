# 全docs再締結監査・第0単位 — read-only棚卸し報告(REWORK版)

仕様ID: `DOCS-RECLOSURE-0`
作成日: 2026-07-22
版: REWORK対応版(Opus 4.8 REJECT後の修正版)。前版(未採用)からの修正点は末尾「REWORK対応記録」に記す。

## 1. 文書状態

**観察**。本書は現行docsの正本化、再分類、撤回、採択を一切行わない。決定/縮小採用/延期/棄却/撤回/未統一/観察/比較中/停止線のいずれの状態語彙も、既存文書の状態を書き換える目的では使わない。本書中でこれらの語を使う箇所は、対象文書が**既に自称している状態を引用**しているか、監査上の仮ラベル(§8の「回収候補/明示棄却済み/歴史のみ/要深掘り」、§5/§6/§7の「あり/なし/要深掘り」)であることを都度明示する。現行仕様・決定への変更はゼロ。

## 2. スナップショット

| 項目 | 値 |
|---|---|
| 基準HEAD | `d38f254096a84cbcb5a7b71c8180de3d5e513913` |
| 固定read-onlyスナップショット | `/tmp/motolii-docs-input.M1AGOX` |
| 隔離worktree(本報告書の書き込み先) | `/tmp/motolii-docs-reclosure.yLV7a8` |
| 対象`docs/**/*.md`件数 | 184件 |
| 対象行数 | 29,374行 |
| 固定スナップショットdocs内容集約SHA-256 | `a92d540171c8a50df9b4af27e31a90d434f51cd2e59a66e0391a204c6a894b3d` |

開始時と終了直前に、固定スナップショットで下記コマンドを実際にread-only実行した。両時点とも件数・hashは不変だった(実測値は§11直前の付記に記載)。

```sh
cd /tmp/motolii-docs-input.M1AGOX
find docs -type f -name '*.md' -print0 | sort -z | xargs -0 shasum -a 256 | shasum -a 256
find docs -type f -name '*.md' | wc -l
find docs -type f -name '*.md' -print0 | xargs -0 wc -l | tail -n 1
```

主作業ツリー(`/Users/member_ottoto/rust_ae/Motolii`)は、`git log --all`によるdocs履歴棚卸しと`git status --short`によるcutoff後差分の観察だけに使い、ファイル作成・整形・checkout・stash・index操作・commitは一切行っていない。終了直前に`git -C /Users/member_ottoto/rust_ae/Motolii status --short -- AGENTS.md docs`を実行したところ、`docs/README.md`・`docs/decision-index.md`・`docs/reviews/README.md`・`docs/ui-reference-map.md`・`docs/ui-runtime-architecture.md`・`docs/reviews/2026-07-21-ui-surface-topology-decision.md`・`docs/spikes/g0-9-timeline-visual-parity.md`の変更と、`docs/reviews/2026-07-22-m3-comfortable-use-*.md`等の未追跡ファイルが観察された。この観察結果は固定スナップショットの184件という監査母集団には混ぜていない。**cutoff後の主作業ツリー変更は次回delta監査の対象**とし、本書の§5〜§10はcutoff時点の固定スナップショットだけを扱う。

## 3. 監査方法と限界

- **本監査は受注者(Claude Sonnet 5)本人が単独で行った**。Claude子エージェント、並列読了エージェント、下請け、再委任は一切使っていない。
- **現行184件**: 全184件を本人がこのセッション中に直接Readツールで全文読了した。読了順序は、まずdocs直下27件・`docs/mocks/README.md`・specs 7件・spikes 17件を全文読了し、続けてreviews配下132件(直下129件[README含む]+evidence 3件)を、まず`docs/reviews/README.md`から始めて日付順に全文読了した。全ファイルについて、ファイル冒頭のステータス行だけでなく本文の決定・条件・依存関係・矛盾を示す記述までを読み、§5付録の役割・矛盾候補ラベルはその全文読了に基づく。
- **歴史(全ref)**: `git -C /Users/member_ottoto/rust_ae/Motolii log --all --name-status -- docs`により、docsの追加/削除/rename全量をpath棚卸しした。この段階では**pathの存在・最終所在・関連commitメタデータ**だけを確認し、全文読了はしていない。衝突候補と入口文書に該当する`docs/plugin-ecosystem.md`(archive tag `archive/cursor/plugin-ecosystem-docs-04c5`, commit `2cbfc813d0db5f258d31bb4a83eb3ac759d60285`)は`git -C /Users/member_ottoto/rust_ae/Motolii show 2cbfc813d0db5f258d31bb4a83eb3ac759d60285:docs/plugin-ecosystem.md`で653行を全文読了した。他の歴史限定文書(§8の21件)は、`git log --all --name-status -- docs`の出力とcommit日時・所在ブランチだけを確認し、全文読了はしていない(§8で個別に明記)。
- **限界**: 歴史側のhistorical-only path(現行に不在)のうち、`.md`以外(HTML/JS/PNG/JSON等のmock/evidence資産)は今回path一覧化のみで内容未読。§8に列挙した21件の歴史限定`.md`のうち、全文読了したのは上記`docs/plugin-ecosystem.md`のみで、残り20件は表題・冒頭行・所在ブランチの確認に留まる(該当箇所に明記)。

## 4. 権威トポロジ

読む順序と優先関係は次の通り(`docs/README.md`が正本)。

1. **入口**: `docs/README.md`(全体像・読む順序)、`AGENTS.md`(作業規約)
2. **概念正本**: `docs/concept.md`(決定台帳)、`docs/decision-index.md`(主題→決定の逆引き索引、運用正本)
3. **仕様書**: `docs/specs/README.md`(プロセス)→ `docs/specs/M0〜M5`(タスク表+実装ガード)
4. **レビュー**: `docs/reviews/README.md`(規律6点+全文書索引、運用正本)配下の個別レビュー。決定/縮小採用/延期/棄却/撤回/未統一/観察/比較中/停止線の状態語彙で管理
5. **スパイク**: `docs/spikes/*`(使い捨て実験結果、個別文書の状態に従う)
6. **モック/archive**: `docs/mocks/README.md`(ARCHIVED、新規変更禁止)。現行実行入口はmain未到達の`codex/m3-mock-components`側`docs/mocks-ui/`(固定SHA `56c318ed...`)
7. **backlog/実装台帳**: `docs/backlog.md`(横断ギャップ)、`docs/implementation-ledger.md`(NOW/NEXT/WAIT)
8. **plugin-authoring/plugin-resources/vism-***等の個別モデル文書は`concept.md`から分岐する詳細正本

優先順位の実務: 矛盾が見つかったら`decision-index.md`より各リンク先の正本を読み、`decision-index.md`は1行要旨に過ぎないことを常に確認する。`docs/reviews/README.md`の全文書索引が全量の正本であり、`docs/README.md`のファイルマップは「現役参照の抜粋」に留まる(`docs/README.md:57`)。

## 5. 全現行文書カバレッジ付録(184件・単一表)

凡例: 役割 = 入口/正本/決定/観察/比較/試作/証跡/履歴/索引/その他(複数該当は主要なものから列挙)。矛盾候補 = あり/なし/要深掘り(既存の決定状態そのものを変更する記載ではない、監査上の仮フラグ)。

行はcutoff時点の固定スナップショットにおける`find docs -type f -name '*.md' | sort`のpath順そのものである(184行ちょうど、各path 1回のみ)。

| # | path | 役割 | 矛盾候補 |
|---|---|---|---|
| 1 | `docs/README.md` | 入口/索引 | なし |
| 2 | `docs/ae-pain-points.md` | 証跡/索引 | なし |
| 3 | `docs/backlog.md` | 索引/正本 | 要深掘り(V2-1のKit/Vism記述と`vism-kit-model.md`の突合要) |
| 4 | `docs/concept.md` | 正本/決定台帳 | なし(単体としては自己整合) |
| 5 | `docs/decision-index.md` | 索引/運用正本 | なし(構造上のリスクのみ: 1行要旨と正本の乖離が生じ得る。§6-5参照) |
| 6 | `docs/dev-experience.md` | 正本/設計ノート | なし |
| 7 | `docs/extensible-core-model.md` | 決定/正本(設計原則) | 要深掘り(Kit定義を`vism-kit-model.md`へ委譲。§7.1参照) |
| 8 | `docs/generative-user-boundary.md` | 決定/正本 | なし(p5.js/Blender非目標化は明確。§9.9参照) |
| 9 | `docs/implementation-ledger.md` | 索引/履歴(運用) | なし |
| 10 | `docs/interaction-simplicity-model.md` | 決定/正本 | なし |
| 11 | `docs/memory-model.md` | 決定/正本 | 要深掘り(P1「非同期パイプライン重畳は未実装」と自己記載。実装状態の他文書との突合は本監査範囲外) |
| 12 | `docs/mocks/README.md` | 試作/履歴(ARCHIVED) | あり(Browser分類`Media/Effects/Objects`案がui-interaction-language.mdの既定`Media/Plugins`と本文中で「未統一」と明記。§9.1系ではなくui-reference-map.mdの既知の未統一表と同一論点) |
| 13 | `docs/performance-model.md` | 決定/正本 | なし |
| 14 | `docs/pitfalls-and-roadmap.md` | 履歴/決定(落とし穴カタログ+ロードマップ) | あり(G-2でAviUtl/AviUtl2を正例・負例双方に引用。§9.4で現行根拠として詳述) |
| 15 | `docs/plugin-authoring.md` | 決定/正本 | なし(Vism≠plugin種別を自ら明記) |
| 16 | `docs/plugin-resources.md` | 決定/正本(凍結ゲートで確定) | なし |
| 17 | `docs/plugin-ui-model.md` | 比較/決定(縮小採用) | 要深掘り(「core plugin vs native window」に隣接する境界線を扱う文書。2026-07-21再評価中の自己記載あり) |
| 18 | `docs/references.md` | 索引/証跡 | なし |
| 19 | `docs/reviews/2026-07-09-R1-export-review.md` | 証跡(コードレビュー所見) | あり(追補(2026-07-11)が自己の記録規律不備4点を自己指摘・訂正) |
| 20 | `docs/reviews/2026-07-09-R3-datatrack-review.md` | 証跡 | なし |
| 21 | `docs/reviews/2026-07-10-M1-plugin-boundary-review.md` | 入口(凍結前チェックリスト) | なし |
| 22 | `docs/reviews/2026-07-10-R8-vello-review.md` | 決定(承認) | なし |
| 23 | `docs/reviews/2026-07-10-R9-real-material-checklist.md` | 証跡 | なし |
| 24 | `docs/reviews/2026-07-10-freeze-gate-declaration.md` | 決定(宣言) | なし |
| 25 | `docs/reviews/2026-07-10-freeze-gate-remaining.md` | 証跡 | なし(宣言文書と整合) |
| 26 | `docs/reviews/2026-07-11-INF-7g-llm-plugin-demo.md` | 証跡 | なし |
| 27 | `docs/reviews/2026-07-11-M2-entry-gate.md` | 入口(達成記録) | なし |
| 28 | `docs/reviews/2026-07-11-code-audit-pre-m2.md` | 証跡(監査所見) | なし(監査自身が「LLM出力なので採用前に現物確認」と自己限定) |
| 29 | `docs/reviews/2026-07-12-M2E-2-ruleset-activation.md` | 証跡 | なし |
| 30 | `docs/reviews/2026-07-12-M2E-7-render-ctx-thaw.md` | 決定(解凍手続き記録) | なし |
| 31 | `docs/reviews/2026-07-12-M3-M4-gate-ledger.md` | 索引(候補台帳) | なし(候補台帳であり本文が明記) |
| 32 | `docs/reviews/2026-07-12-code-audit-2nd-d1.md` | 証跡/決定(裏取り済み所見) | なし |
| 33 | `docs/reviews/2026-07-12-d1-spec-holes-prior-art.md` | 比較/決定混在(先例調査メモ) | 要深掘り(「反対側レビュー未実施」の調査メモ内にユーザー決定ブロックが複数埋め込まれ、ステータス表記と内容が混在) |
| 34 | `docs/reviews/2026-07-12-m2-permanence-prevention.md` | 決定/正本(運用手順) | なし |
| 35 | `docs/reviews/2026-07-12-pathop-ae-cavalry-comparison.md` | 比較(未採用) | なし(自己が「未採用」と明記) |
| 36 | `docs/reviews/2026-07-12-plugin-ui-v1-boundary.md` | 決定(歴史的、再評価中) | あり(自己のステータス行が「2026-07-21再評価中/2026-07-22軸分離」と明記し、2026-07-12決定が後続文書で部分的に読み替えられている) |
| 37 | `docs/reviews/2026-07-12-prior-art-gap-counter-review.md` | 比較(反対側レビュー) | なし |
| 38 | `docs/reviews/2026-07-12-prior-art-gap-survey.md` | 比較(未完遂の探索メモ) | なし(自己限定済み) |
| 39 | `docs/reviews/2026-07-12-rework-prior-art.md` | 比較(仮説メモ) | なし |
| 40 | `docs/reviews/2026-07-12-success-prior-art.md` | 比較(仮説メモ、自己改訂) | なし |
| 41 | `docs/reviews/2026-07-12-vertical-text-prior-art-counter-review.md` | 比較(反対側レビュー) | なし |
| 42 | `docs/reviews/2026-07-12-vertical-text-prior-art.md` | 比較(調査メモ) | なし |
| 43 | `docs/reviews/2026-07-13-decision-pack-adoption.md` | 決定 | なし |
| 44 | `docs/reviews/2026-07-13-readback-pipelining-prior-art.md` | 比較(調査文書) | なし |
| 45 | `docs/reviews/2026-07-13-undecided-critical-path-confirm.md` | 履歴(確認メモ) | なし |
| 46 | `docs/reviews/2026-07-13-wgpu-challenges-counter-review.md` | 比較(反対側レビュー) | あり(memory-model.md P1節の記述との対応関係は本監査で個別突合していない。§5-11参照) |
| 47 | `docs/reviews/2026-07-14-3d-depth-boundary-prior-art.md` | 比較(調査メモ) | なし |
| 48 | `docs/reviews/2026-07-14-3d-depth-scope-design.md` | 決定(一部superseded) | 要深掘り(同日追補でcamera前倒しを述べるが、統一camera設計文書との時系列関係は本監査で個別突合していない) |
| 49 | `docs/reviews/2026-07-14-audio-generalization-design.md` | 決定 | なし |
| 50 | `docs/reviews/2026-07-14-color-conversion-prior-art.md` | 決定(調査→判定済み) | なし |
| 51 | `docs/reviews/2026-07-14-d5-transport-prior-art.md` | 決定(採択) | なし |
| 52 | `docs/reviews/2026-07-14-m2-core-closure.md` | 履歴(撤回済み) | なし(撤回を自己明記、README.md索引とも整合) |
| 53 | `docs/reviews/2026-07-14-m2-exit-param-pipeline-disposition.md` | 決定 | なし |
| 54 | `docs/reviews/2026-07-14-m3-ui-boundary-counter-review.md` | 決定(反対側レビュー) | なし |
| 55 | `docs/reviews/2026-07-14-m3-ui-boundary-prevention.md` | 決定(運用手順) | なし |
| 56 | `docs/reviews/2026-07-14-motion-foundation-known-tech-disposition.md` | 決定 | なし(後続決定に劣後する旨を自己明記) |
| 57 | `docs/reviews/2026-07-14-motion-tools-praise-diy-gap-audit.md` | 観察(先例調査) | なし |
| 58 | `docs/reviews/2026-07-14-recent-concept-propagation-audit.md` | 観察(横断監査) | なし(発見即是正の記録として自己完結) |
| 59 | `docs/reviews/2026-07-14-repeated-wheel-standardization-audit.md` | 観察 | なし |
| 60 | `docs/reviews/2026-07-14-unified-stage-camera-design.md` | 決定(一部superseded) | 要深掘り(schema/runtime記述がD1j/D1kで置換されたと自己記載する追補あり) |
| 61 | `docs/reviews/2026-07-15-d1l-copylocal-remint-counter-review.md` | 決定(反対側レビュー) | なし |
| 62 | `docs/reviews/2026-07-15-d1l-journal-revert-boundary-counter-review.md` | 決定(反対側レビュー) | なし |
| 63 | `docs/reviews/2026-07-15-d1l-journal-revert-boundary-decision.md` | 決定(追補) | なし |
| 64 | `docs/reviews/2026-07-15-implementation-readiness-ledger.md` | 索引/運用正本 | なし |
| 65 | `docs/reviews/2026-07-15-m2-foundation-reclosure-counter-review.md` | 決定(反対側レビュー) | なし |
| 66 | `docs/reviews/2026-07-15-m2-foundation-reclosure-gate.md` | 決定(解除宣言、正本) | なし(m2-core-closureの正しい後継として機能) |
| 67 | `docs/reviews/2026-07-15-p5-generative-pattern-disposition.md` | 観察(調査・配置案) | なし |
| 68 | `docs/reviews/2026-07-15-prior-art-complaint-boundary-audit.md` | 観察(調査第一陣) | なし |
| 69 | `docs/reviews/2026-07-15-relative-scope-duplicator-decision.md` | 決定 | なし |
| 70 | `docs/reviews/2026-07-15-shared-effect-lifecycle-decision.md` | 決定 | なし |
| 71 | `docs/reviews/2026-07-16-ae-layer-system-disposition.md` | 決定+観察混成(処置台帳) | あり(グループ内重なり許可を「提案」として台帳化し、ユーザー採択待ちのまま残す旨を自己記載) |
| 72 | `docs/reviews/2026-07-16-d1l-current-document-constructor-counter-review.md` | 決定(反対側レビュー) | なし |
| 73 | `docs/reviews/2026-07-16-d1l-current-document-constructor-decision.md` | 決定(正本) | なし単体では(lint-conflict-decisionとの対読み必要) |
| 74 | `docs/reviews/2026-07-16-d1l-new-v1-lint-conflict-decision.md` | 決定(仕様訂正) | 要深掘り(冒頭でd1i4-semantic-oracle-boundary-decisionによる読み替えを予告する連鎖訂正) |
| 75 | `docs/reviews/2026-07-16-m2-comp-camera-decision.md` | 決定 | なし |
| 76 | `docs/reviews/2026-07-16-m2-param-element-constraint-disposition.md` | 決定(延期処分) | なし |
| 77 | `docs/reviews/2026-07-16-m2-project-sidecar-session-decision.md` | 決定(実装済み+追補) | あり(A0Sとの相互「Conflict→Decision」節を持つ意図的相互追補) |
| 78 | `docs/reviews/2026-07-16-m3-preflight-decisions.md` | 決定(正本) | 要深掘り(G0-3節が2026-07-21/22で複数回訂正され、本文自体が「再評価中」と自己記載) |
| 79 | `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md` | 正本(条件付き発注) | 要深掘り(§5直列順が後続文書のG0-3/G0-9再編で変化した可能性。本監査では個別突合していない) |
| 80 | `docs/reviews/2026-07-16-m3-ui-gap-survey.md` | 観察(調査メモ) | あり(A-1でplugin-ui-model.mdとM3仕様の矛盾を自己指摘、GAP-13で「既知」と自己記載) |
| 81 | `docs/reviews/2026-07-16-m3-ui-rapid-acceptance-prior-art.md` | 観察(仮説メモ) | なし(自己格下げ済み) |
| 82 | `docs/reviews/2026-07-16-media-portability-gpu-resurvey-plan.md` | 入口(実施計画) | なし |
| 83 | `docs/reviews/2026-07-16-ui-update-forensics.md` | 観察+決定(採用審判) | なし |
| 84 | `docs/reviews/2026-07-17-aviutl2-comment-voices.md` | 証跡(一次声観察台帳) | あり(§9.4/§9.9関連。主題そのものが対立する一次声の並存を結論とする) |
| 85 | `docs/reviews/2026-07-17-d1i4-semantic-oracle-boundary-decision.md` | 決定(訂正) | あり(d1l-new-v1-lint-conflict-decisionの前提を後追いで訂正する連鎖) |
| 86 | `docs/reviews/2026-07-17-extensible-core-prior-art-translation.md` | 観察 | なし |
| 87 | `docs/reviews/2026-07-17-non-video-workspace-asset-ui-prior-art.md` | 観察 | なし |
| 88 | `docs/reviews/2026-07-17-vism-a0-plugin-boundary-inventory.md` | 正本(inventory) | なし |
| 89 | `docs/reviews/2026-07-17-vism-a0d-contract-migration-ownership-decision.md` | 決定 | なし |
| 90 | `docs/reviews/2026-07-17-vism-a0s-contract-catalog-spec.md` | 正本(仕様決定) | あり(D1mとの相互「Conflict→Decision」節。Kit用語は非目標として繰り返し先送りされていることを自己明記) |
| 91 | `docs/reviews/2026-07-17-vism-a1-public-crate-boundary-spec.md` | 正本(仕様決定) | なし |
| 92 | `docs/reviews/2026-07-17-vism-a2-legacy-project-migration-decision.md` | 決定 | なし |
| 93 | `docs/reviews/2026-07-17-vism-a7-bpm-datatrack-spike.md` | 試作(spike完了) | なし単体では(Kit/consumer plugin/materialize方式は「まだ決めていないこと」と明記) |
| 94 | `docs/reviews/2026-07-17-vism-implementation-plan.md` | 正本(ロードマップ案) | なし |
| 95 | `docs/reviews/2026-07-17-vism-ready-counter-review-disposition.md` | 決定(採否) | あり(**「Kit」がVSM-A0〜A7〜B2への実装チェーン全体で一度も定義されず、常に将来へ先送りされるプレースホルダである**ことを自ら確定させている。§7.1参照) |
| 96 | `docs/reviews/2026-07-18-d1k-runtime-camera-thaw-spec.md` | 決定(正本) | あり(2026-07-18-m2-foundation-supplementary-code-reviewが「本文書冒頭にD3f WAITの歴史記述が残る」と外部から指摘。§6-4参照) |
| 97 | `docs/reviews/2026-07-18-m2-foundation-supplementary-code-review.md` | 決定/証跡 | あり(D1k冒頭の記述と実際の完了状況の齟齬を自己報告) |
| 98 | `docs/reviews/2026-07-18-m3-egui-selection.md` | 決定(歴史的採否、自己が超越承知) | なし(自己が「G0-9で再評価中」と明記) |
| 99 | `docs/reviews/2026-07-18-m3-gpu-preview-viewport-prior-art.md` | 観察(先例調査、歴史) | なし(自己がSlint結論の置換を明記) |
| 100 | `docs/reviews/2026-07-18-vism-a3-external-expression-survey.md` | 観察 | なし |
| 101 | `docs/reviews/2026-07-18-vism-a3d-radial-repeater-decision.md` | 決定 | なし |
| 102 | `docs/reviews/2026-07-18-vism-a3s-layersource-lowering-spec.md` | 決定(仕様、完了) | なし |
| 103 | `docs/reviews/2026-07-19-am-keyframe-graph-observation.md` | 観察(証跡グレード) | なし |
| 104 | `docs/reviews/2026-07-19-lyric-motion-text-sequence-comparison.md` | 比較(比較中) | あり(§9.7関連。「譜面」語の使用がui-score-model.mdの技術用語なのか撤回済み音楽メタファーなのか、本書自身は区別を明示していない) |
| 105 | `docs/reviews/2026-07-19-m3-interaction-prototype-decision-ledger.md` | 観察(比較仮説台帳) | あり(「本書は決定ではない」と明記しつつP48〜P53を「採択」とラベル付けする内部緊張) |
| 106 | `docs/reviews/2026-07-19-m3-text-motion-task-translation.md` | 決定(条件付き発注正本) | なし |
| 107 | `docs/reviews/2026-07-20-local-worktree-publication-audit.md` | 観察/外部再開地図 | なし |
| 108 | `docs/reviews/2026-07-20-m3-keymap-codec-contract.md` | 決定(正本) | なし |
| 109 | `docs/reviews/2026-07-20-m3-rerun-late-discovery-premortem.md` | 決定 | なし(Rerun-as-justification誤りへの防御自体が主題) |
| 110 | `docs/reviews/2026-07-20-m3-u2a-1-command-adapter-contract.md` | 決定(実装完了契約) | なし |
| 111 | `docs/reviews/2026-07-20-perceptual-expression-translation-decision.md` | 決定(正本統合) | なし(Rerunを「主要な製品先例」と明記するが、直後に強制動線を明記し合格根拠化を自制) |
| 112 | `docs/reviews/2026-07-20-rerun-learning-transfer-plan.md` | 決定(運用正本) | なし |
| 113 | `docs/reviews/2026-07-20-rerun-prior-art-survey.md` | 決定(歴史的、再評価中) | なし(自己が「egui採否だけはG0-9へ移った」と明記) |
| 114 | `docs/reviews/2026-07-20-rerun-re-ui-module-inventory.md` | 観察/比較中 | なし(一括DEPEND棄却、個別候補は全て仮置きと明記) |
| 115 | `docs/reviews/2026-07-20-rerun-source-asset-inventory.md` | 観察 | なし |
| 116 | `docs/reviews/2026-07-21-m3-product-mock-recovery-plan.md` | 決定/停止線 | あり(この停止線が同バッチの他契約(U0e/U1a/U1b/U2b/U2c)へどこまで及ぶかは本監査で個別突合していない) |
| 117 | `docs/reviews/2026-07-21-m3-react-webview-runtime-reconsideration.md` | 決定(責任境界・topology) | あり(自己の「追補2」で、OS topology/native-React軸とplugin trust軸を過去に混同していたと自己是正。§9.2参照) |
| 118 | `docs/reviews/2026-07-21-m3-rectangle-drop-d2-contract-options.md` | 比較(未採択) | なし |
| 119 | `docs/reviews/2026-07-21-m3-u0e-1-token-generator-contract.md` | 決定(実装完了) | なし |
| 120 | `docs/reviews/2026-07-21-m3-u0e-2-reference-fixture-contract.md` | 決定(実装待ち) | なし |
| 121 | `docs/reviews/2026-07-21-m3-u1a-1-static-viewport-contract.md` | 決定(実装完了) | なし |
| 122 | `docs/reviews/2026-07-21-m3-u1a-2-layout-projection-contract.md` | 決定(実装完了) | なし |
| 123 | `docs/reviews/2026-07-21-m3-u1b-1-render-worker-contract.md` | 決定(実装完了) | なし |
| 124 | `docs/reviews/2026-07-21-m3-u1b-2-latest-projection-contract.md` | 決定(実装完了) | なし |
| 125 | `docs/reviews/2026-07-21-m3-u2b-1-single-writer-e2e-contract.md` | 決定(実装完了) | なし |
| 126 | `docs/reviews/2026-07-21-m3-u2c-1-interaction-state-contract.md` | 決定(実装完了) | なし |
| 127 | `docs/reviews/2026-07-21-m3-u2c-4-diagnostic-envelope-contract.md` | 決定(実装完了) | なし |
| 128 | `docs/reviews/2026-07-21-native-stage-gizmo-counter-review.md` | 決定(反対側レビュー、縮小採用) | なし |
| 129 | `docs/reviews/2026-07-21-native-stage-gizmo-ownership.md` | 決定 | なし(GPU所有≠picking≠焼き込みを明記) |
| 130 | `docs/reviews/2026-07-21-native-surface-renderer-counter-review.md` | 決定/証跡(反対側レビュー) | なし |
| 131 | `docs/reviews/2026-07-21-native-surface-renderer-extended-search.md` | 比較 | なし(WebView child+native wgpu siblingの出荷実例ゼロという未解決リスクを一貫して継承) |
| 132 | `docs/reviews/2026-07-21-native-surface-renderer-growth-review.md` | 証跡(伸長レビュー) | なし |
| 133 | `docs/reviews/2026-07-21-native-surface-renderer-reselection.md` | 決定/正本 | なし |
| 134 | `docs/reviews/2026-07-21-ui-surface-topology-decision.md` | 決定/正本 | なし |
| 135 | `docs/reviews/2026-07-22-creator-developer-continuum-decision.md` | 決定 | なし(trust/sandbox/permission/single writer/Host責任を明示的に「残す」と明記) |
| 136 | `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md` | 決定/索引(draft) | 要深掘り(「Fable全体レビュー待ち/実装発注ではない」draftである点を後続文書が誤って完了扱いしないか継続監視) |
| 137 | `docs/reviews/2026-07-22-m3-comfortable-use-work-map.md` | 決定/索引 | なし |
| 138 | `docs/reviews/2026-07-22-m3-detachable-panel-window-contract.md` | 決定/証跡 | なし |
| 139 | `docs/reviews/2026-07-22-m3-graph-headless-interaction-dependency.md` | 決定/証跡 | なし |
| 140 | `docs/reviews/2026-07-22-m3-native-depth-rail-acceptance.md` | 決定 | なし |
| 141 | `docs/reviews/2026-07-22-m3-native-easing-popup-acceptance.md` | 決定/証跡 | なし |
| 142 | `docs/reviews/2026-07-22-m3-native-multi-key-graph-view-acceptance.md` | 決定/証跡 | なし(GPL/MITライセンス防火壁を明記) |
| 143 | `docs/reviews/2026-07-22-m3-react-coordinate-surface-audit.md` | 観察 | なし |
| 144 | `docs/reviews/2026-07-22-m3-react-product-asset-promotion-contract.md` | 決定/発注停止線 | なし |
| 145 | `docs/reviews/2026-07-22-m3-surface-extension-axis-separation.md` | 決定 | 要深掘り(4軸分離の宣言そのもの。他文書がこれを守り続けるかの継続監視対象) |
| 146 | `docs/reviews/2026-07-22-ui-music-metaphor-retirement.md` | 決定(撤回) | あり(範囲は精密だが、旧用語が対象外文書に残存していないかは個別確認要。§9.7参照) |
| 147 | `docs/reviews/README.md` | 索引/正本(規律6点+全文書索引) | あり(索引本文が162行の一覧を保持するが、本監査は`docs/reviews/`直下の実ファイル数128件(README除く)・evidence 3件との機械突合を§6-1で個別に記録する。索引自体は現行運用正本であり内容矛盾ではない) |
| 148 | `docs/reviews/evidence/am-keyframe-graph/README.md` | 証跡(intake manifest) | なし |
| 149 | `docs/reviews/evidence/grok-lyric-counter-review/GROK-LYRIC-20260719.md` | 証跡(外部レビュー逐語) | あり(意図的なadversarial批判。lyric-motion-text-sequence-comparison.md §7が部分的にのみ受容したと記録済み) |
| 150 | `docs/reviews/evidence/grok-lyric-counter-review/README.md` | 証跡(intake manifest) | なし |
| 151 | `docs/simulation-model.md` | 決定/正本(設計方針確定・口の予約段階) | なし |
| 152 | `docs/specs/M0-spikes.md` | 決定/履歴(確定済milestone) | なし(UI採用結論の置換を自己明記) |
| 153 | `docs/specs/M1-vertical-slice.md` | 決定/履歴(確定済milestone) | なし |
| 154 | `docs/specs/M2-document-model.md` | 決定/正本(段階発注可) | なし(内部で自己訂正チェーンが多数あるが追跡可能) |
| 155 | `docs/specs/M3-ui-integration.md` | 決定/正本(ドラフト) | 要深掘り(§9.5参照。renderer選定がegui→React/WebView→direct-wgpu+Velloと複数回移動し本文内に3段階の記述が併存する) |
| 156 | `docs/specs/M4-cache-and-analysis.md` | 決定/正本(ドラフト、凍結ゲートで確定) | なし |
| 157 | `docs/specs/M5-3d-and-post.md` | 決定/正本(ドラフト、凍結ゲートで確定) | なし |
| 158 | `docs/specs/README.md` | 入口/索引 | なし |
| 159 | `docs/spikes/g0-10-multi-surface-window.md` | 試作/証跡 | なし(合格範囲と停止線を自己限定) |
| 160 | `docs/spikes/g0-9-native-depth-rail.md` | 試作/証跡 | 要深掘り(「isolated fixture合格」と`g0-9-timeline-visual-parity.md`の「回収して再実行」という表現の整合は本監査で個別突合していない) |
| 161 | `docs/spikes/g0-9-native-easing-popup.md` | 決定/証跡 | なし(「製品U4b接続の停止線は解除しない」と自己限定) |
| 162 | `docs/spikes/g0-9-native-graph-view.md` | 試作/証跡 | なし(GPL非依存を明記) |
| 163 | `docs/spikes/g0-9-surface-host.md` | 試作/証跡 | なし(自己限定明記) |
| 164 | `docs/spikes/g0-9-timeline-visual-parity.md` | 試作/証跡 | なし(「製品操作は未接続」と自己限定。2026-07-22追補でDepth Rail結果を接続) |
| 165 | `docs/spikes/g0-9-ui-runtime.md` | 比較/決定に準ずる | なし(「G0-9の最終採否はまだ閉じない」と自己限定) |
| 166 | `docs/spikes/g0-9-verification-matrix.md` | 索引/証跡 | なし(PASS/PARTIAL/PHYSICALを自己で分離) |
| 167 | `docs/spikes/g0-9-windowed-timeline.md` | 試作/証跡 | なし(text/icon/入力/D2は未証明と自己明記) |
| 168 | `docs/spikes/ime-acceptance.md` | 試作(歴史的未実走) | なし(Slint時代のfixtureであり現行egui証拠として再利用しないと明記) |
| 169 | `docs/spikes/lyric-identity-reconcile/README.md` | 試作 | なし |
| 170 | `docs/spikes/pv1-texture-lifecycle-evidence/README.md` | 証跡 | なし |
| 171 | `docs/spikes/pv1-texture-lifecycle.md` | 証跡(PV-1 pass) | なし |
| 172 | `docs/spikes/s1-slint.md` | 履歴(歴史的合格証拠) | なし(自己が撤回済みと明記) |
| 173 | `docs/spikes/s2-decode.md` | 決定 | なし |
| 174 | `docs/spikes/s3-vello.md` | 決定(採用) | なし |
| 175 | `docs/spikes/timeline-bench.md` | 証跡 | なし(「本スパイクはUI非連結・描画コアのみ計測」と自己限定) |
| 176 | `docs/text-model.md` | 試作/決定(部分、ドラフト) | なし |
| 177 | `docs/ui-concept.md` | 決定/正本(設計方針) | あり(音楽メタファー撤回の記録文書そのもの。撤回範囲は明確) |
| 178 | `docs/ui-interaction-language.md` | 決定/正本 | あり(Browser既定`Project Explorer/Plugin Browser`が`docs/mocks/README.md`・Reactプロトタイプ側の分類と未統一。ui-reference-map.mdが自己追跡中) |
| 179 | `docs/ui-reference-map.md` | 索引/運用正本 | あり(本書自体が「既知の未統一」表を保持する矛盾追跡文書) |
| 180 | `docs/ui-runtime-architecture.md` | 決定/正本 | あり(「native window」「React window」という呼称を禁止する記述を含み、他文書がこの禁を守っているかの横断確認は本監査で全件突合していない) |
| 181 | `docs/ui-score-model.md` | 決定/正本 | あり(`score`という語をpath互換のためだけ残すと明記、製品概念としては撤回済み。§9.7参照) |
| 182 | `docs/ui-visual-language.md` | 決定/正本(設計基準) | なし |
| 183 | `docs/vism-kit-model.md` | 決定/正本(schema未決) | あり(§本文全体が「歴史的Kit」との照合対象。§7.1参照) |
| 184 | `docs/vism-package-concept.md` | 決定/正本(container未決) | なし単体では(Vism/Kit分離は明記) |

**184件カバレッジ確認(実測)**: 上表の行数=184。内訳(実測) = docs直下27 + `docs/mocks/README.md` 1 + specs 7 + spikes 17 + reviews直下(README除く)128 + reviews/evidence 3 + reviews/README 1 = **184**。この内訳は§2の固定スナップショットに対し実行した`find`集計と一致する(コマンドと出力は§11直前の付記を参照)。

## 6. 機械的不整合

以下は証跡(コマンド・path/line)を伴う機械的観察であり、決定の変更ではない。

### 6-1. reviews/README.mdの索引と実ファイル数の機械突合

`docs/reviews/README.md`の全文書索引(固定スナップショット上で162行、`| ファイル | 表題 |`形式の表を含む)は、`docs/reviews/`直下の日付付きreview 128件を1行ずつ列挙する設計である。隔離worktreeで以下を実行した。

```sh
cd /tmp/motolii-docs-reclosure.yLV7a8
scripts/check-docs.sh
```

結果: `OK: docs整合チェック全項目通過`(2026-07-22実行、exit 0)。`scripts/check-docs.sh`は索引の抜け・入口台帳の重複掲載・ローカルリンク切れ・状態語彙を機械検証する(`docs/reviews/README.md:27`)。したがって索引と実ファイルの対応関係は機械検証済みであり、本監査が§5で改めて手動突合をやり直す必要はない。

### 6-2. 状態語彙の固定集合の遵守

`docs/decision-index.md`と`docs/reviews/README.md`はともに「決定/縮小採用/延期/棄却/撤回/未統一/観察/比較中/停止線」を固定集合と定める(`docs/decision-index.md:12`、`docs/reviews/README.md:26`)。本監査で全文読了した184件のステータス行は、目視で確認した範囲ではこの語彙に従う。ただし`docs/reviews/2026-07-19-m3-interaction-prototype-decision-ledger.md`は「比較仮説台帳」という語を使いつつ内部で「採択」という固定集合外の語を使用している(P48〜P53)。`scripts/check-docs.sh`本体(隔離worktreeの`scripts/check-docs.sh`該当箇所)を読んだところ、状態語彙検査(項目4)は`awk -F'|' '/^\|/ && NF>=6 ...'`で`docs/decision-index.md`のテーブル行の第4カラムだけを対象にしており、`docs/reviews/`配下の個別review本文中の語(P48〜P53のような箇所)は検査対象に含まれない。したがって上記のP48〜P53の語彙は`scripts/check-docs.sh`の検査範囲外であり、同スクリプトがOKを返したことはこの箇所の語彙逸脱の有無を保証しない。

### 6-3. 重複参照パターン(注記による上書き)

`docs/reviews/2026-07-14-unified-stage-camera-design.md`のpersistent camera schema記述は、後継の`2026-07-16-m2-comp-camera-decision.md`と`2026-07-18-d1k-runtime-camera-thaw-spec.md`によって段階的に置換されているが、置換元自身(unified-stage-camera-design.md)は本文中に旧schema記述をそのまま残し、冒頭注記のみで読者に読み替えを要求する構造になっている(2026-07-18追記)。これは矛盾ではなく、置換の記法が「本文差し替え」ではなく「注記による上書き」であるという機械的パターンであり、`2026-07-16-d1l-new-v1-lint-conflict-decision.md`→`2026-07-17-d1i4-semantic-oracle-boundary-decision.md`の連鎖、`2026-07-17-vism-a0s-contract-catalog-spec.md`↔`2026-07-16-m2-project-sidecar-session-decision.md`の相互追補などでも繰り返し観察される、本docsセット全体に共通する編集規約(この規約自体の是非は本監査の対象外)。

### 6-4. 孤立参照候補(未修正の記述)

`docs/reviews/2026-07-18-d1k-runtime-camera-thaw-spec.md`冒頭のレーン表は本監査時点でも「D3f (WAIT)」の表記を保持している。一方、外部の`docs/reviews/2026-07-18-m2-foundation-supplementary-code-review.md`(§P2 tracking item #4)は「D3fは完了済みなのにd1k側の記述が古い」と指摘している。`docs/specs/M2-document-model.md`のD3f行は「**完了**」と明記されている(CompCameraレーン表)。d1k側本文は本監査時点でも未修正のままである(`docs/reviews/2026-07-18-d1k-runtime-camera-thaw-spec.md`該当行と`docs/specs/M2-document-model.md`のD3f行を突合すれば再現できる)。

### 6-5. decision-index.mdの網羅性

`docs/decision-index.md`は固定スナップショット上でファイル全体85行(`wc -l`実測)だが、本監査で全文読了した多数の2026-07-09〜07-17のreviews(例: `2026-07-13-decision-pack-adoption.md`、`2026-07-12-m2-permanence-prevention.md`)は`docs/decision-index.md`に個別の主題キーワード行を持たない。これは`docs/decision-index.md`冒頭が明記する運用("過去決定の全量転記は目的ではない。作業で触れた主題から順に登録する")と整合するが、逆引きできない既決事項が実際に存在するという機械的事実として記録する。

### 6-6. `git diff --check` / `git status`(隔離worktree)

```sh
cd /tmp/motolii-docs-reclosure.yLV7a8
git diff --check   # exit 0、出力なし
git status --short
#  M docs/reviews/README.md
# ?? docs/reviews/2026-07-22-all-docs-reclosure-inventory.md
git diff --name-only
# docs/reviews/README.md
```

変更ファイルはallowlist 2件(`docs/reviews/README.md`と新規`docs/reviews/2026-07-22-all-docs-reclosure-inventory.md`)のみであることを実行時に確認した。

## 7. 意味上の衝突候補

### 7.1 「Kit」の歴史的用法 vs 現行Vism Kit(最重要衝突候補)

- **歴史側**: `archive/cursor/plugin-ecosystem-docs-04c5`(commit `2cbfc813d0db5f258d31bb4a83eb3ac759d60285`)の`docs/plugin-ecosystem.md`を全文(653行)読了した。§1冒頭注記:
  > 「**AviUtl2 カタログからの取り込み:** [aviutl2-catalog](https://github.com/Neosku/aviutl2-catalog) の **「ホストを持たず GitHub／外部を索引するクライアント」** という姿勢は採る…採らないのは **中央正本 `index` の恒久運用・popularity/trend/telemetry**」(§1)
  §1.3「界隈のガラパゴスとkit」・§3の用語表:
  > 「**kit** | ユーザーが名前を付けて書き出した使用セット(プロジェクト非依存でも可)。中身の正本は**lock**と同型 | Homebrew Bundle / VS Code extensions推奨リスト」
  Kitはここでは**導入済みplugin一覧を書き出し・共有して他者が同じ環境を再現するための、lockfile型の配布・発見機構**である。
- **現行側**: `docs/vism-kit-model.md`全体(§1「Core=文法/Vism=語彙/Kit=接続済みの文章・用途セット/Project=作品」、§4「Kitの責任」、§5「v1 Kitはmaterializeする」)。
  > 「**Kit**は複数Vism、接続、初期値、素材要求を目的単位へまとめる」「v1のKitはProjectへ常駐するruntimeにしない…Kitを選ぶ→必要Vism/asset/型をpreflight→Project snapshotに対して展開案を作る→全体成功時だけ1 macro commit」
  現行KitはHost内部で**typed provider/consumerの接続構成をProjectへ1回限りmaterializeする設計概念**であり、plugin一式のexport/importという配布機構ではない。
- **現在の状態**: 現行`docs/vism-kit-model.md`は「設計原則決定／schema・形式未決」であり、`docs/reviews/2026-07-17-vism-ready-counter-review-disposition.md`が明記する通り、VSM-A0〜A7〜A0D〜A0S〜A1〜A2の全実装チェーンを通じて**「Kit」は一度も定義されず常にVSM-B2(未着手)へ先送りされるプレースホルダ**である(`docs/vism-kit-model.md`§12「停止線」も`KitDefinition`等の恒久Document schemaを未実装のまま明記)。
- **衝突して見える理由**: 同じ「Kit」という語が、(a)配布・共有のためのplugin一覧ファイル、(b)Host内部のVism接続構成、という全く異なる2つの意味を指しており、将来Kit schemaが具体化される際にどちらの意味系列を継承するのか、あるいは両方が別名で共存するのかが現行docsからは判別できない。
- **まだ証明していない範囲**: (1) 現行`vism-kit-model.md`の起草者が歴史的`plugin-ecosystem.md`の存在・内容を知っていたかは不明(現行184件のどこからも`plugin-ecosystem.md`への参照はゼロ、本監査のGrepで確認済み)。(2) 歴史的Kit(配布セット共有)の需要が現行設計のどこかに吸収されているかは未確認。`docs/backlog.md`のV2-1/V2-8行に「動的ロード」「marketplace」への言及はあるが、歴史的Kitのlockfile的発想への直接参照はない。
- **次の裁定先**: Vism実装計画のPhase B(VSM-B0〜B3、package/entry/Kit/Project instance/artifact identityのfixture化)着手前に、Codexが「歴史的Kit(配布セット共有)の需要は現行設計のどこで拾うか」を明示的に裁定すべき論点として次の再締結work packageへ引き継ぐ。

### 7.2 archive plugin-ecosystem.mdの「ホストレス発見基盤」と現行Vism marketplace非目標の整合性

- **歴史側**: `docs/plugin-ecosystem.md`(archive)は「motolii が売らない・配らない・審査しない」「中央`index.json`一枚の恒久運用」を明示的に非目標とし(§1「やらないこと」、行30)、AviUtl2 catalogの「ホストを持たずGitHub/外部を索引するクライアント」という姿勢だけを採り、中央正本の恒久運用・人気順・telemetryを反面教師とする設計だった(§599「今〜M3近傍(GAP-13)」表)。
- **現行側**: `docs/vism-package-concept.md`§10の現在地表は「marketplace / registry / trust policy: v2・未決」としており、具体的なホストレス設計(tap/lock/GitHub index)への言及は現行184件のどこにも存在しない(Grep確認済み)。
- **衝突して見える理由**: 衝突ではなく**空白**である。歴史側は具体的なホストレス発見機構の設計(tap/package/lock/kitの三層モデル)を持っていたが、現行docsはこの具体設計を一切継承も明示的棄却もしていない。
- **まだ証明していない範囲**: この歴史設計が意図的に破棄されたのか、単に参照が失われた(pathが現行indexから漏れた)のかは、現行docs内の証拠だけからは判別不能。
- **次の裁定先**: Codexが、この歴史設計を「回収候補」として次の意味レビューへ載せるか、「明示棄却」として記録するかを判断する。本監査はどちらとも断定しない。

### 7.3 「core plugin」と「native window」の混同リスク(自己是正済みだが継続監視要)

- **証拠**: `docs/reviews/2026-07-21-m3-react-webview-runtime-reconsideration.md`の「追補2」(2026-07-22)は次のように自己是正する。
  > 「本書は標準製品surfaceのruntime選定とplugin UI公開runtimeを同じG0-9へ寄せすぎていた。…OS topologyとnative／React surfaceはG0-9、Core／bundled Host module／pluginの所属とfirst／third-partyの公開・信頼境界はG0-3 / GAP-13で別に判定する」
  この文言自体が、必須負例「window runtime軸とplugin ownership軸を一つの分類表へ潰さない」に、過去一時的にでも抵触していたことの自己証跡である。
- **現在の状態**: `docs/reviews/2026-07-22-m3-surface-extension-axis-separation.md`が「OS topology」「presentation runtime」「architectural role(Core kernel/bundled Host module/plugin)」「provenance/trust(first/third-party)」の4軸を明示的に独立させ、`docs/ui-runtime-architecture.md`も「native／Reactはpresentation runtimeの分担であり、Core、bundled first-party Host module、first-party plugin、third-party pluginの分類ではない」(§1)、「Timeline / Stage / Browser / Inspectorはbundled first-party Host moduleであり、surface runtimeからplugin分類を推論しない」(§5決定済み一覧)と明記する。
- **衝突して見える理由**: 過去に一度混同が発生し自己是正された経緯があるため、今後の新規docsが同じ混同を再発させていないかの継続監視が必要。本監査で全文読了した184件の範囲では、この4軸分離を破る記述は発見されなかった。
- **まだ証明していない範囲**: cutoff後(本監査対象外)に作成される新規docsがこの分離を維持するかどうか。

### 7.4 「演奏する譜面」比喩の撤回範囲とui-score-model.mdの「譜面」用語残存

- **証拠**: `docs/reviews/2026-07-22-ui-music-metaphor-retirement.md`(撤回)は「演奏する→実行する/編集する」「譜面→時間面」等の用語置換表を持ち、旧用語は歴史文書での記録としては残してよいが現行仕様・入口文書・製品UI名・発注書では禁止すると明記する(対象: `docs/ui-concept.md`、`docs/ui-score-model.md`、`docs/concept.md`)。
- `docs/ui-score-model.md`自身は「pathの`score`は参照互換のため残すが、製品概念としての「譜面」は撤回済み」(冒頭ステータス行)と明記している。
- **衝突して見える理由**: `docs/reviews/2026-07-19-lyric-motion-text-sequence-comparison.md`(2026-07-19付、撤回文書より前)は「譜面」という語を`ui-score-model.md`の技術用語(Lane非所有projectionパターン)の意味で複数箇所使用しており、これは狭い技術用語としての用法であって音楽メタファーの用法ではないと読めるが、両者の区別を明示する記述はlyric-motion文書自身にはない。
- **まだ証明していない範囲**: 撤回文書が名指しする3文書以外、特に`2026-07-19-lyric-motion-text-sequence-comparison.md`のような撤回前に書かれた文書内の「譜面」用法が、撤回後も技術用語として問題なく通用するのか、あるいは言い換えが必要なのかは、本監査の証拠だけでは断定できない。
- **次の裁定先**: Codexによる用語横断grep(「譜面」「演奏」「First Beat」「楽曲が背骨」)と、各出現箇所が(a)ui-score-model.mdの技術用語としての用法か(b)撤回された音楽メタファーの用法かの個別判定。

### 7.5 M3 UI runtime rendererの複数回移動と「現在どれが正か」の読みにくさ

- **証拠**: `docs/specs/M3-ui-integration.md`は(a) 2026-07-18 egui採用(方針節)、(b) 2026-07-21 React/WebView reconsideration(冒頭ステータス)、(c) 2026-07-21 native renderer reselection(direct wgpu+Vello局所、プレビュー出力の寿命節)の3段階の変遷を本文内に併存させている。`docs/decision-index.md`の該当行(「UI基盤 egui React WebView...」)は「比較中」と明記し、`docs/reviews/2026-07-21-native-surface-renderer-extended-search.md`は「system WebView child + native wgpu sibling surfaceを同一windowで出荷したproduction実例は…見つからなかった」と明記する。
- **衝突して見える理由**: egui/React-WebView/direct-wgpu+Velloという3つの候補が、それぞれ別の日付の文書で「第一候補」的に語られており、初見の読者が「今どれが正しいのか」を`M3-ui-integration.md`単体だけから素早く判断するのは難しい。実際には段階的な絞り込み(topology決定→native側内部rendererの絞り込み)であり内部矛盾はないが、記法上の読みにくさは実在する。
- **まだ証明していない範囲**: この3段階が完全に整合しているか(egui撤去のタイミングと新renderer本実装のタイミングの依存関係)は、可読性の改善余地(3段階の変遷を1つの「現在の結論」節へ集約する等)として次の意味レビューの候補になり得る。

## 8. 歴史上の未回収候補

全refsのpath棚卸しから見つかった、現行184件に不在の`.md`ファイル21件。`git -C /Users/member_ottoto/rust_ae/Motolii log --all --name-status --diff-filter=A --format= -- docs`で全refから追加された`.md`パス195件を抽出し、固定スナップショットの現行184件と突合した結果、historical-only pathは以下の21件だった(重複ゼロ・欠落ゼロで本監査時点に再確認済み)。各行の仮ラベルは監査上の分類であり、決定状態ではない。全文読了したのは`docs/plugin-ecosystem.md`のみで、他の20件は表題・所在・関連commitメタデータの確認に留まる(§3の限界を参照)。

| path(歴史) | 最終所在 | 仮ラベル | 根拠(表題・メタデータのみ確認) |
|---|---|---|---|
| `docs/design-memo.md` | commit `c2e89cb9`で削除(2026-07-09) | **明示棄却済み** | 削除commitメッセージ「Remove superseded design memos」。`docs/README.md:7`の「整理履歴」節が削除理由(Tauri+WebView採用、OpenCut React流用等の旧仕様混在)を明記 |
| `docs/discussion-log-2026-07-06.md` | 同上 | **明示棄却済み** | 同上 |
| `docs/mocks-ui/README.md` | main未到達、`codex/m3-mock-components`ブランチ側に存在 | **歴史のみ(ただし現行運用対象)** | `AGENTS.md`「M3の外観・timeline・panelに触る時」節が「main側にまだ無い時はdocs/mocks/を代替の現行実装として変更せず」と明記し、この未マージブランチのReactモックを現行実行入口として扱う運用中の参照 |
| `docs/plugin-ecosystem.md` | archive tag `archive/cursor/plugin-ecosystem-docs-04c5`(commit `2cbfc813d0db5f258d31bb4a83eb3ac759d60285`, 2026-07-12) | **回収候補** | §7.1/7.2で詳述。全文653行を読了済み |
| `docs/reviews/2026-07-12-M2-order-gate-halt.md` | archive tag `archive/m2-d3-doc-graph`(2026-07-12) | **歴史のみ** | commit表題「M2発注ゲート停止(2026-07-12)」。現行のM2ゲート文書群に実質吸収されたと推定されるが、明示的な後継リンクは現行docsから確認できていない |
| `docs/reviews/2026-07-15-keymap-schema.md` | 2026-07-16のcommitで存在、現行不在 | **回収候補(要深掘り)** | commit表題「入力マップ/ショートカット スキーマ設計(2026-07-15)」。現行`2026-07-20-m3-keymap-codec-contract.md`と`2026-07-16-m3-preflight-decisions.md`§2.3がkeymap設計の後継正本として機能している可能性が高いが、本監査では内容突合していない |
| `docs/reviews/2026-07-15-m3-entry-gate.md` | 2026-07-15、現行不在 | **明示棄却済み(推定)** | commit表題「M3入場条件(2026-07-15)、ステータス: 未達成」。現行の`docs/decision-index.md`の該当行が別の入場条件文書体系(m2-foundation-reclosure-gate.md、m3-preflight-decisions.md)を指しており、本ファイルは初期草案が置き換えられたと推定されるが確証はない |
| `docs/reviews/2026-07-16-m2-external-revision-decision.md` | tag `archive/codex/m2-external-revision`, `archive/codex/m2-unresolved-decisions`(2026-07-16) | **歴史のみ** | commit表題「M2 external project revision decision」。現行`docs/specs/M2-document-model.md`にD1nという項目は存在せず、この決定がどう吸収されたか(または非採用になったか)は未確認 |
| `docs/reviews/2026-07-18-m3-preview-lifecycle-disposition.md` | 2026-07-18 | **回収候補(要深掘り)** | commit表題「M3 preview texture lifecycle 懸念の処分(PV-1)」。現行`docs/spikes/pv1-texture-lifecycle.md`(PV-1 pass)が関連する後継証跡である可能性が高いが、本ファイル自体の処分内容は本監査で全文確認していない |
| `docs/reviews/2026-07-18-m3-workspace-customization-decision.md` | 2026-07-18、`codex/m3-pv1-disposition`ブランチ | **回収候補(要深掘り)** | commit表題「M3 workspace customization 決定」。現行docsのWorkspace/User settings分類(G0-2 5層)にこの決定が吸収されているか未確認 |
| `docs/reviews/2026-07-19-graph-view-reference-decision.md` | main HEAD時点(固定commit `56c318ed`)で存在 | **回収候補(要深掘り)** | commit表題「Graph View参照・比較記録」。現行`2026-07-22-m3-native-multi-key-graph-view-acceptance.md`と`2026-07-22-m3-graph-headless-interaction-dependency.md`が後継として機能している可能性が高い |
| `docs/reviews/2026-07-20-m3-browser-panel-egui-taffy-spike.md` | `codex/m3-browser-panel-spike`ブランチ(2026-07-20) | **歴史のみ(公開済み比較ブランチ)** | commit表題「M3 Browser panelをReactモックからeguiへ翻訳する実験」。`2026-07-20-local-worktree-publication-audit.md`がこのブランチを「比較用に公開、canonical候補ではない」と明示分類済み |
| `docs/reviews/2026-07-21-m3-place-rectangle-d2-contract.md` | `codex/m3-u2b-2-core`ブランチ(2026-07-22最終touch) | **回収候補(未マージ進行中)** | commit表題「M3 U2b-2 PlaceRectangle Host-only D2契約」。現行`2026-07-21-m3-rectangle-drop-d2-contract-options.md`(比較中・未採択)と直接関係する未マージの後続作業と推定されるが、mainへの統合有無は未確認 |
| `docs/reviews/2026-07-21-m3-position-add-key-d2-contract.md` | 2026-07-22最終touch、main未確認 | **回収候補(未マージ進行中)** | commit表題「M3 U4b-0 Position Add Key Host/D2/journal 耐久契約」 |
| `docs/reviews/2026-07-21-m3-tonight-product-vertical-slice-contract.md` | `codex/m3-u2b-2-core`ブランチ | **回収候補(未マージ進行中)** | commit表題「VS-0: Tonight Product Vertical Slice — docs 再入場契約」。`2026-07-21-m3-product-mock-recovery-plan.md`(現行、停止線)と直接関係する可能性 |
| `docs/reviews/2026-07-22-m3-g0-9-builtin-webview-admission.md` | 2026-07-22最終touch、main未確認 | **回収候補(未マージ進行中)** | commit表題「M3 G0-9 built-in WebView Host限定入場」。現行`2026-07-21-ui-surface-topology-decision.md`のWebView islands方針と直結する可能性が高い |
| `docs/reviews/2026-07-22-m3-g0-9-h1-exact-contract.md` | `codex/m3-g0-9-h1a-product-react-v2`ブランチ | **回収候補(未マージ進行中)** | commit表題「M3 G0-9H1 built-in Web exact contract」 |
| `docs/reviews/2026-07-22-m3-h1a-oracle-route-separation.md` | 2026-07-22最終touch、main未確認 | **回収候補(未マージ進行中)** | commit表題「M3 H1a visual oracle route分離」 |
| `docs/reviews/2026-07-22-m3-u2b-2-core-product-contract.md` | `codex/m3-u2b-2-core`ブランチ | **回収候補(未マージ進行中)** | commit表題「M3 U2b-2-core Place 製品コア契約」 |
| `docs/reviews/2026-07-22-m3-u2h-1-single-primary-selection-contract.md` | `codex/m3-u2h-1-contract`ブランチ | **回収候補(未マージ進行中)** | commit表題「M3 U2h-1 Host Transient single-primary selection契約」 |
| `docs/reviews/2026-07-22-m3-u3a-1-headless-timeline-contract.md` | 2026-07-22最終touch、main未確認 | **回収候補(未マージ進行中)** | commit表題「M3 U3a-1 headless Timeline projection / layout / hit-test 契約」 |

**観察上の注記**: 上記21件のうち9件(2026-07-21〜22付の`m3-*`系)は、上表に記載した個別のfeatureブランチ名(`codex/m3-u2b-2-core`、`codex/m3-u2h-1-contract`、`codex/m3-g0-9-h1a-product-react-v2`等)上に存在し、"歴史"というより"main未到達の並行進行中の作業"である可能性が高い。ブランチの総数・一意ブランチ名の集計は本監査の対象としない(件数の主張はしない)。これらを安易に「撤回」や「歴史のみ」と分類すると、実際には今後mainへ統合される可能性のある作業を誤って死んだ扱いにするリスクがある。したがって上表では「回収候補(未マージ進行中)」という区別ラベルを設けた。この仮ラベルは決定状態ではなく、Codexが個別にブランチ統合状況を確認すべき対象であることを示すに留める。

## 9. 会話論点マトリクス(必須10項目)

各項目は独立した軸として扱い、証拠へ接続する。潰していない(必須負例参照)。

### 9.1 native window / React window(表示runtime・所有面)

- 正本: `docs/ui-runtime-architecture.md`(責任境界・surface topology決定)、`docs/reviews/2026-07-21-ui-surface-topology-decision.md`(1 top-level wgpu Surface + 2 native viewport + opaque child WebView islands)
- 現状: Reactは DOM shell(Browser/Inspector/parameter form/panel/toolbar/dialog/検索/設定)を所有。Native Rust/wgpuはStage/Timeline全体(ruler/track header/lane/clip/key/playhead/graph/selection含む)を所有。これは「Core/plugin」分類ではなく「presentation runtime」の分担であると明記(`ui-runtime-architecture.md` §1)。
- 未決/継続: native側内部のrenderer実装方式(direct wgpu primitive batch第一候補、Vello局所利用)はG0-9のplatform受入(Windows/DPI/IME/a11y実機等)待ち。WebView child + native wgpu siblingの同一window出荷実例がゼロという未解決リスクが`2026-07-21-native-surface-renderer-extended-search.md`で明記されている。

### 9.2 minimal core / core・first-party・third-party plugin(機能/配布/所有)

- 正本: `docs/reviews/2026-07-22-m3-surface-extension-axis-separation.md`(architectural role軸: Core kernel / bundled Host module / plugin、provenance/trust軸: first-party / third-party)、`docs/extensible-core-model.md`
- 現状: Host module(Timeline/Stage/Browser/Inspector)はbundled first-party製品面であり、それ自体をplugin kit公開契約にしない(`2026-07-21-m3-react-webview-runtime-reconsideration.md` 追補2)。first-party pluginは内部特権を持たず第三者と同じ公開契約・fixtureで検査される(`2026-07-22-creator-developer-continuum-decision.md`)。
- 必須負例遵守確認: 本監査で全文読了した184件の範囲では「core plugin」と「native window」を同義とする記述は発見されなかった。ただし`2026-07-21-m3-react-webview-runtime-reconsideration.md`自身が過去に軸混同していたことを自己是正した経緯があり(§7.3参照)、継続監視対象。

### 9.3 timeline/previewをcore pluginと呼ぶ案と、現行core責任

- 現状: `docs/ui-runtime-architecture.md`はTimeline/Stage/Browser/Inspectorを明確に「bundled first-party Host module」と呼び、「plugin」とは呼ばない。本監査で全文読了した184件の範囲で、TimelineまたはPreviewを明示的に「core plugin」と呼ぶ記述は**発見されなかった**(現行根拠未発見)。
- 探索範囲: `docs/plugin-ui-model.md`、`docs/ui-runtime-architecture.md`、`docs/specs/M3-ui-integration.md`、reviews配下132件のうちUI runtime関連の全文読了範囲(2026-07-21〜22の native/React/surface系文書、G0-9系spike 8件を含む)で「core plugin」「timeline.*plugin」「preview.*plugin」の文言を確認したが、該当なし。

### 9.4 AviUtl/AviUtl2 catalog先例(独立節)

現行Motolii docsの証拠とarchiveの歴史証拠を明確に分離する。

- **現行docsの証拠**: `docs/concept.md`「プラグインファーストの"範囲"(2026-07-09決定)」は「大部分をプラグインで設計できる」(AviUtlの実例)をv1から全面採用すると明記するが、これは**プラグイン境界の設計**についての引用であり、AviUtl2の**カタログ/発見機構**についての引用ではないと本文が明示的に区別する(「これは"配布/マーケットの仕組み"ではなく"境界の設計"を指す」)。`docs/pitfalls-and-roadmap.md` G-2は「AviUtl2は思想の参考に留め、依存しない」と明記し、学ぶ対象を「C ABI+テーブル登録の安定ホスト契約・種別明示・D&D配布・スクリプトでパラメータロジックをホスト外に出す設計」に限定し、「学ばないのはベータAPI変動」等と明記する。`docs/reviews/2026-07-17-aviutl2-comment-voices.md`はAviUtl2利用者の一次声(軽さ/重さ、統合/分業等)を保存する観察台帳だが、設計根拠にしないと自己限定する。`docs/reviews/2026-07-16-m3-ui-rapid-acceptance-prior-art.md`はAviUtl2の即時受容(SDK同時公開・64bit化)を先例として扱うが、これも受容速度の観察であり配布カタログの意味論ではない。**現行184件の中に、AviUtl2の`aviutl2-catalog`(GitHub索引・hostless discovery)を名指しで参照する記述は現行根拠未発見**(Grep確認済み: `docs/vism-package-concept.md`、`docs/backlog.md`、`docs/extensible-core-model.md`のいずれもmarketplace/registryを「v2・未決」と記すのみで、AviUtl2固有の索引方式への言及はない)。
- **archiveの歴史証拠**: `archive/cursor/plugin-ecosystem-docs-04c5`(commit `2cbfc813d0db5f258d31bb4a83eb3ac759d60285`)の`docs/plugin-ecosystem.md`は、AviUtl2 catalogの「ホストを持たずGitHub/外部を索引するクライアント」という姿勢を明示的に取り入れ候補としつつ、中央`index.json`の恒久運用・popularity/trend/telemetryを反面教師とする、tap/kit/lockの三層モデルを持つ具体的な発見機構設計を持っていた(§1、§1.3、§599)。この歴史設計は現行docsのどこからも参照されていない(§7.2で詳述)。
- **含意**: 現行docsはAviUtl(1)を「プラグイン境界の設計思想」の先例として明示的に採用し、AviUtl2の「カタログ/発見機構」は明示的に非採用(参考にしない)としている。したがって「AviUtl/AviUtl2 catalog先例」という論点は、現行docsでは主に**歴史側(archive)にのみ**具体的に存在し、現行側では「参考にしない」という明示的な立場のみが存在する、という非対称な状態にある。

### 9.5 Vism marketplace、GitHub経由serverless配布、tap/lock、host serverを持たない非目標

- 現状: `docs/vism-package-concept.md`§10「marketplace / registry / trust policy: v2・未決」。歴史側の`docs/plugin-ecosystem.md`(archive)は具体的なホストレスtap/lock設計を持っていたが現行docsに継承されていない(§7.2で詳述)。
- 探索範囲: 現行184件内で「marketplace」「tap.toml」「plugins.lock.toml」の具体スキーマへの言及は現行根拠未発見。`docs/backlog.md`のV2-1/V2-8が「動的ロード」「配布基盤」を将来課題として言及するのみ。

### 9.6 現行Vism Kitと歴史上の共有plugin set/lockとしてのKit

- §7.1で詳述。両者は明確に異なる意味であり、統合されていない。混同禁止の必須負例に対し、現行docs単体では混同は見られない(両者を同じ文書内で扱う記述が現行docsに存在しないため、混同のしようがない=不在による安全)。ただし将来Kit schemaを具体化する際にこの2つの需要系列(接続構成のmaterialize vs plugin一式の配布共有)をどう扱うかは未決。

### 9.7 「演奏する譜面」比喩を音楽中心へ固定しない懸念

- §7.4で詳述。撤回済み(`docs/reviews/2026-07-22-ui-music-metaphor-retirement.md`)。BPM/拍grid/Soundtrackという具体機能は維持されるが、製品全体の存在論としては使わない。撤回範囲と`ui-score-model.md`の「score」用語残存・`2026-07-19-lyric-motion-text-sequence-comparison.md`の「譜面」技術用語用法との区別が要確認事項として残る。

### 9.8 p5.js/Blenderを用いたworld-building型制作の観察

- 正本: `docs/generative-user-boundary.md`§2/§5、`docs/reviews/2026-07-15-p5-generative-pattern-disposition.md`
- 現状: p5.js/Processing型表現は(1)有限one-shot生成、(2)seed付き閉形式(純関数)、(3)前入力読取(TemporalFootprint)、(4)自己出力蓄積(Feedback+checkpoint)、(5)結合逐次状態(SimulationPlugin+StateTrack)、(6)live入力(記録済みTrack変換)の6分類/5経路に振り分けられ、いずれもコンポジット境界へ翻訳される。Blender相当の複数world/camera、collection、constraint、rig、sculptは明示的な非目標。Motoliiは「白紙から素材世界を作るCreative Coding環境ではない」と明記(`docs/generative-user-boundary.md`§2)。
- 必須負例遵守確認: p5.js/Blender/AviUtl/Rerunを「Motolii要件そのもの」にする記述は現行根拠未発見。全て先例・翻訳対象としての扱いに留まる。

### 9.9 community governance/扇動政治という未定義論点

- 現行根拠未発見。`docs/reviews/2026-07-22-creator-developer-continuum-decision.md`は参加資格を薄くする一方でtrust/sandbox/permissionをHostが維持すると明記するが、具体的なcommunity governance(投票・モデレーション・レピュテーション制度等)の設計は現行184件のどこにも存在しない。探索範囲: `creator-developer-continuum-decision.md`全文、`extensible-core-model.md`全文、`backlog.md`全文、`vism-package-concept.md`全文(いずれも本監査で全文読了済み)。

### 9.10 creator/developer境界を薄くする思想、人海戦術としてのecosystem

- 正本: `docs/reviews/2026-07-22-creator-developer-continuum-decision.md`
- 現状: 「使う→作る」を1つの連続経路とする一方、「境界を無くす対象は参加資格と学習経路であり、作品の持続性、安全性、権限、責任ではない。誰でも作者になれることと、誰のcodeでも無確認に実行することを同義にしない」と明記。custom plugin UI、Vism loader、marketplace、package形式、署名方式、新Document variantの実装許可ではないと明記(必須負例遵守)。
- 未定義のまま残る論点: community governance/moderation制度そのものは、本決定文書が明示的に「発明しない」と述べる範囲であり(§9.9と同一論点だが「多数作者を成長力にする」という思想側から見た軸として独立に扱う)、現行docs全体でも具体制度設計は不在(現行根拠未発見)。

**§9完成確認**: 9.1〜9.10の10小節すべてに証拠または「現行根拠未発見」+探索範囲を記載した。

## 10. 再締結work package案(実際の変更はしない)

以下は次の再締結発注の**候補分割案**であり、本監査は発注も実装もしていない。各案は依存順、変更候補ファイル、非目標、STOP条件、確認質問を示す。

### WP-1: Kit用語の意味系列整理(§7.1対応)

- 依存: なし(独立して着手可能)
- 変更候補ファイル: `docs/vism-kit-model.md`(注記追加のみ想定)、`docs/decision-index.md`(1行追加)
- 非目標: Kit schemaの確定、container/拡張子の決定
- STOP条件: 歴史的Kit(配布セット共有)の需要を現行設計へ吸収する場合、新しい公開型・Document fieldを発明しようとした時点
- 確認質問(ユーザー裁定待ち): 歴史的`plugin-ecosystem.md`のホストレスtap/lock/kit設計は(a)明示的に棄却する、(b)将来のVism Phase B/C/Dの参考資料として正式に引用する、(c)無視して現行設計を独立に進める、のいずれを望むか。

### WP-2: 用語横断監査「譜面/演奏」撤回の波及確認(§7.4対応)

- 依存: なし
- 変更候補ファイル: 波及確認のみ(変更ファイルは次段階で個別特定)
- 非目標: 撤回自体の再検討、新比喩の導入
- STOP条件: 技術用語としての「score」/「譜面」利用(ui-score-model.md由来)と製品比喩としての利用の区別が文書内で判別不能な場合
- 確認質問: `docs/reviews/2026-07-19-lyric-motion-text-sequence-comparison.md`のような撤回前文書内の「譜面」表記に、撤回後の注記を追加する必要があるか。

### WP-3: 未マージfeatureブランチ9件の統合状況確認(§8対応)

- 依存: なし(git操作のみ、docs変更は伴わない可能性が高い)
- 変更候補ファイル: 状況次第(`docs/implementation-ledger.md`への1行追記等)
- 非目標: ブランチのmerge実行そのもの
- STOP条件: ブランチ内容がmainの現行決定と矛盾する場合
- 確認質問: `codex/m3-u2b-2-core`、`codex/m3-u2h-1-contract`、`codex/m3-g0-9-h1a-product-react-v2`等の未マージブランチを、次のM3実装発注の優先候補として扱ってよいか。

### WP-4: D1k冒頭のD3f WAIT記述の是正(§6-4対応)

- 依存: なし
- 変更候補ファイル: `docs/reviews/2026-07-18-d1k-runtime-camera-thaw-spec.md`(注記1行)
- 非目標: D1k契約本体の意味変更
- STOP条件: なし(機械的な記述更新のみ)
- 確認質問: 不要(事実確認のみで実施可能)

### WP-5: decision-index.md逆引き網羅性の拡充(§6-5対応)

- 依存: なし
- 変更候補ファイル: `docs/decision-index.md`(複数行追加、既存決定の書き換えなし)
- 非目標: 決定内容自体の変更
- STOP条件: なし
- 確認質問: どの既決事項を優先的に逆引き可能にするか(全量追加は`decision-index.md`自身の運用方針「全量転記は目的ではない」と衝突するため、優先順位が必要)

## 11. Codex向け結論

### 即時修正可能

- WP-4(D1k冒頭のD3f WAIT記述の是正): 機械的事実の記述更新のみ。
- WP-5の一部(明らかに逆引き価値の高い既決事項)を`decision-index.md`へ追加すること。

### ユーザー裁定待ち

- WP-1(歴史的Kitの扱い): §7.1の確認質問。
- WP-2(譜面/演奏撤回の波及範囲): §7.4/WP-2の確認質問。
- WP-3(未マージブランチの優先順位): §8/WP-3の確認質問。
- §7.2(archive plugin-ecosystem.mdの回収要否)。
- §9.4(AviUtl2 catalog歴史設計の回収要否。WP-1と同根)。

### 歴史調査追加

- §8の「回収候補(要深掘り)」4件(`2026-07-15-keymap-schema.md`、`2026-07-18-m3-preview-lifecycle-disposition.md`、`2026-07-18-m3-workspace-customization-decision.md`、`2026-07-19-graph-view-reference-decision.md`)と「回収候補(未マージ進行中)」9件は、内容の全文突合と現行後継文書との対応確認をまだ行っていない。
- `docs/reviews/2026-07-16-m2-external-revision-decision.md`(D1n)が現行`specs/M2-document-model.md`に吸収されているか未確認。

### 現行のまま

- 本監査で発見した大半の「矛盾候補」は、実際には自己言及的な訂正チェーン(連鎖訂正)であり、既存の規律(反対側レビュー、判定語併記)が機能した結果である。これらについては追加のdocs変更を要求しない。
- §7.3(core plugin/native window混同)は既に自己是正済みであり、現時点での追加対応は不要。継続監視のみ。
- §9.1〜9.10の10論点全てについて、証拠または「現行根拠未発見」を記載済みであり、追加の即時対応は不要。

---

## 付記: 必須コマンド実行結果(実測・過去形)

開始時点(本監査冒頭、2026-07-22)、固定read-onlyスナップショットで以下を実行した。

```sh
cd /tmp/motolii-docs-input.M1AGOX
find docs -type f -name '*.md' -print0 | sort -z | xargs -0 shasum -a 256 | shasum -a 256
# → a92d540171c8a50df9b4af27e31a90d434f51cd2e59a66e0391a204c6a894b3d
find docs -type f -name '*.md' | wc -l
# → 184
find docs -type f -name '*.md' -print0 | xargs -0 wc -l | tail -n 1
# → 29374 total
```

終了直前(本監査完了直前、2026-07-22)、同じコマンドを再実行し、上記と完全に同じ値(hash `a92d540...894b3d`、184件、29,374行)を得た。両時点で不変であることを実測で確認した。

種別内訳の実測:

```sh
grep -E '^docs/[^/]+\.md$' <(cat 上記find結果) | wc -l        # → 27 (docs直下)
grep -E '^docs/mocks/'                                          # → 1 件(README.md)
grep -E '^docs/specs/' | wc -l                                   # → 7
grep -E '^docs/spikes/' | wc -l                                  # → 17
grep -E '^docs/reviews/[^/]+\.md$' | grep -v 'README.md$' | wc -l # → 128
grep -E '^docs/reviews/README.md$'                               # → 1 件
grep -E '^docs/reviews/evidence/'                                # → 3 件
```

内訳合計 = 27 + 1 + 7 + 17 + 128 + 1 + 3 = **184**(§2・§5の総数と一致)。reviews配下合計(README含む直下129件 + evidence 3件) = **132件**。

歴史側historical-only `.md`の実測(過去形):

```sh
cd /Users/member_ottoto/rust_ae/Motolii
git log --all --name-status --diff-filter=A --format= -- docs | grep -E '\.md$' | sed 's/^A\t//' | sort -u > /tmp/all_added_md.txt
wc -l /tmp/all_added_md.txt
# → 195
comm -23 /tmp/all_added_md.txt /tmp/current_md.txt   # current_md.txt = 固定スナップショットのsorted 184 path
wc -l < 上記出力
# → 21
```

全refから追加されたことのある`.md`パス195件のうち、固定スナップショットの現行184件に存在しないものが21件であり、これが§8の表と完全一致することを実測で確認した(重複ゼロ・欠落ゼロ)。

隔離worktreeで実際に実行した必須4コマンドの結果(2026-07-22実施、過去形で記録):

```sh
cd /tmp/motolii-docs-reclosure.yLV7a8
git diff --check
# → exit 0、出力なし(問題なし)
scripts/check-docs.sh
# → "OK: docs整合チェック全項目通過"、exit 0
git status --short
# →  M docs/reviews/README.md
# → ?? docs/reviews/2026-07-22-all-docs-reclosure-inventory.md
git diff --name-only
# → docs/reviews/README.md
```

`git status --short`と`git diff --name-only`により、変更対象がallowlist 2件(`docs/reviews/README.md`の1行更新、および本報告書の新規追加)だけであることを実測で確認した。

`git -C /Users/member_ottoto/rust_ae/Motolii status --short -- AGENTS.md docs`(cutoff後の主作業ツリー観察、監査母集団には不混入)は、`docs/README.md`・`docs/decision-index.md`・`docs/reviews/README.md`・`docs/ui-reference-map.md`・`docs/ui-runtime-architecture.md`・`docs/reviews/2026-07-21-ui-surface-topology-decision.md`・`docs/spikes/g0-9-timeline-visual-parity.md`の変更と、複数の未追跡`docs/reviews/2026-07-22-m3-*.md`・`docs/spikes/g0-9-*`/`g0-10-*`ファイルを示した。これらはcutoff後に主作業ツリー側で発生した変更であり、次回delta監査の対象として記録するに留め、本書の§5〜§10には反映していない。

## REWORK対応記録(1回目)

前版(未採用)からの主な修正点(Opus 4.8指摘への対応):

1. §9を正確に10小節(9.1〜9.10)へ修正した。独立した「9.4 AviUtl/AviUtl2 catalog先例」節を新設し、現行Motolii docsの証拠(現行根拠未発見であることを含む)とarchive `plugin-ecosystem.md`の歴史証拠を明確に分離した。末尾の完成宣言は実際の9.1〜9.10と一致させた。
2. §5を単一の機械照合可能なcoverage tableへ統合した。固定snapshotのsorted pathを184行ちょうど・各path 1回のみで記載し、各行に役割と「あり/なし/要深掘り」を付けた。`docs/reviews/README.md`にも役割・矛盾候補ラベルを付けた。
3. 件数分解を実測値(docs直下27 + mocks 1 + specs 7 + spikes 17 + reviews直下README除外128 + reviews/evidence 3 + reviews/README 1 = 184)と完全一致させた。自己否定する数式・疑問符・将来再計測する旨を削除した。
4. §3から「9つの並列読了エージェント」「下請けエージェント」「読み落とし保証なし」を削除した。受注者(本人)が固定184件を自ら全文読了した事実、歴史側はpath全量棚卸しと衝突候補(plugin-ecosystem.md)だけ全文読了したという限界を正確に記した。
5. §6の「scripts/check-docs.sh未実行」等の未来形記述を削除し、隔離worktreeで実際に必須4コマンドを実行した結果(成功/失敗と出力)を過去形で記録した。固定snapshotも終了時に再測定し、hash `a92d540...894b3d`・184件・29,374行を実測記録した。
6. `git branch -a`由来の総数主張を削除し、未マージ候補は個別の固定ref/pathだけで示したという事実だけを記した。
7. Opus指摘のP0/P1解消後、coverage table行からpathだけ抽出し固定snapshotのsorted 184 pathと完全一致することを一時照合した(重複ゼロ・欠落ゼロ・余分ゼロ)。§9見出しが9.1〜9.10の10件であることを確認した。`git diff --name-only`がallowlist 2件だけであることを確認した。これらの照合はrepoへスクリプトとして保存していない。
8. 分析内容・負例・未決・歴史候補は削除せず、証拠が誤っていた箇所(§9の項目数、§5の集計式)だけをpath/line再確認の上で訂正した。

## REWORK対応記録(2回目・数値監査)

2回目のOpus 4.8 REJECT指摘への対応。意味分析・184行coverage・§9.1〜9.10の構成は維持し、以下の数値のみを元データから再実測して訂正した:

1. `git branch -a`由来の行数・ブランチ件数の主張(「167行」「162件」等)を全箇所から削除した。未マージ候補は個別のブランチ名(`codex/m3-u2b-2-core`等)という固定ref証拠のみで示し、ブランチ総数・一意ブランチ数は集計しないと明記した(§3、§8観察上の注記、付記)。
2. §6-2の「`scripts/check-docs.sh`がこれを検出するか個別に確認していない」を削除し、同スクリプト本体(`awk -F'|' '/^\|/ && NF>=6 ...'`、項目4)を実際に読んだ上で、状態語彙検査は`docs/decision-index.md`のテーブル第4カラムだけを対象とし、reviews本文中の語彙(P48〜P53等)は検査範囲外であるという再現可能な事実へ訂正した。
3. 全refs historical-only `.md`は`git log --all --name-status --diff-filter=A -- docs`で追加履歴のある`.md`パス195件を抽出し、固定スナップショットの現行184件と`comm -23`で突合した結果、実測21件であることを確認した。§3・§8本文中の「20件/残り19件/上記20件」をそれぞれ「21件/残り20件/上記21件」へ訂正した。§8の表21行と実測結果が完全一致することも照合した。
4. reviews件数を`docs/reviews/`直下129件(README込み、README除く128件)+`docs/reviews/evidence/`3件 = reviews配下合計132件と実測し、既存の「131件」という主張を文脈に応じて128/129/132へ訂正した(§3の読了順序記述、coverage表#147行、§9.3の探索範囲)。
5. 固定snapshotの`docs/decision-index.md`は`wc -l`実測でファイル全体85行であることを確認し、§6-5の「86行のテーブル」という誤記を「全体85行(`wc -l`実測)」へ訂正した。
6. §5のcoverage分解を`docs直下27 + mocks 1 + specs 7 + spikes 17 + reviews直下README除外128 + reviews/evidence 3 + reviews/README 1 = 184`の1行だけに整理し、誤った`#1-11,13-18`表記とalphabetical混在の自己訂正・途中計算を削除した。
7. 報告書全文から数字を含む文を再確認し(`grep`で「件」「行」「歴史限定」「reviews」「branch」等を含む行を抽出)、件数・行数・path番号範囲・historical unique数・hashを再検証した。§6-1の索引行数を「163行」から実測「162行」(固定スナップショットの`docs/reviews/README.md`)へ訂正した。再現不要な装飾的数値(非md historical-only資産の推定件数「154件」)は削除した。
8. 付記の必須コマンド実行結果に、上記historical-only 21件の再測定コマンドと出力、reviews配下合計132件の内訳を過去形で追加した。隔離worktreeの`git diff --check`・`scripts/check-docs.sh`・`git status --short`・`git diff --name-only`、固定スナップショットのhash/件数/行数を本監査の最終段階で再実行し、いずれも冒頭時点と同じ結果(allowlist 2件のみ、hash `a92d540...894b3d`、184件、29,374行)であることを実測確認した。
