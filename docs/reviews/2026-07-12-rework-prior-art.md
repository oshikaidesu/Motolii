# 出戻り: 先人の失敗後対応と、その反面(予防)(2026-07-12)

ステータス: **仮説メモ**(一次資料つき — **設計根拠ではない**)。運用の正本は予防側: [2026-07-12-m2-permanence-prevention.md](2026-07-12-m2-permanence-prevention.md)。規律は[reviews/README.md](README.md)。

## これは何か

同じ出戻りリスクに対し、先人は大きく二層で動いている。

1. **失敗後**(本メモ後半): 全書き直し・migration・Legacy残置 — Olive/OpenCut/Natron/Blender/AE が有名
2. **失敗前**(運用に採用する側): 焼く前に意味を固定し、恒久面を狭くし、追加的にだけ伸ばす — SQLiteの3.0以降・LSPの単純型・Blenderの設計時互換節・Motolii自身のスパイク/凍結ゲート

**採用方針**: いまのMotoliiは出戻りがまだ最小。**予防を第一選択にし、失敗後パターンを日常に持ち込まない**。失敗後の詳細は参考として残す。

**規律2(反対側レビュー)の適用除外(2026-07-12ユーザー決定)**: 本メモからのGR-PV-1〜5採用には反対側レビューを実施しない。理由: ①採用面の大半が既決規律(H-3・粒度ルール7・並列レーン・D1i-4)の集約・結線であり新規判断面が狭い ②対象領域(パス演算)は先例の確立した枯れた技術 ③運用注の判定基準「ユーザーデータへ不可逆に焼くか」に照らし、本採用は不可逆なものを焼かない。不可逆な判断(PathOp意味論表の【決定】昇格・未決6点)は本採用に含まれず、昇格時に改めて審査する。

## A. 予防(反面) — 先人が「焼かなかった/狭く焼いた」側

| 型 | 先人 | 何をしたか | 一次出典 | Motoliiへの読み |
|---|---|---|---|---|
| P1 約束は成熟後 | SQLite | 3.0.0**以降**はファイル形式を非互換に変えない。それ以前は変更し得た | [formatchng.html](https://sqlite.org/formatchng.html) | Document後方互換の外向き公約はv1.0まで書かない([成功先例](2026-07-12-success-prior-art.md)公約1)。M2仕様は意図的にドラフト |
| P2 凍結面を単純に | LSP | URI・カーソル位置など言語中立の単純型を標準化し、ASTは標準化しない | [LSP overview](https://microsoft.github.io/language-server-protocol/overviews/lsp/overview/) | 恒久に焼く面を狭く。未決UI都合・未証明アルゴリズムをスキーマに入れない |
| P3 破壊は設計で先に宣言 | Blender | 大規模変更は**実装前**のdesign taskに互換節、Coreレビュー、`Interest/Compatibility` | [Blend File Compatibility](https://developer.blender.org/docs/handbook/guidelines/compatibility_handling_for_blend_files/) | スキーマPRの前に仕様【決定】+「何が恒久/何が仮」を書く |
| P4 意味変更は新口 | AE | 新アルゴリズム=新エフェクト(新match name)。旧は残す | [Obsolete effects](https://helpx.adobe.com/after-effects/using/obsolete-effects.html); [Adobe blog 2016](https://blog.adobe.com/en/publish/2016/07/10/after-effects-cc-2015-3-in-depth-gpu-accelerated-effects) | 既存variantの意味を書き換えない(S16)。予防としては「改善PRで既存を触らない」 |
| P5 実証してから並列 | Motolii自身 / 垂直スライス慣習 | M0スパイク→M1垂直→凍結ゲート。未検証境界の上に重基盤を置かない | [凍結ゲート宣言](2026-07-10-freeze-gate-declaration.md); concept「ヒーロー誕生後」 | D1i-2(意味論)前にD3/並列増をしない |
| P6 安定を選ぶなら変更を拒む | TeX | 成熟後は深刻な欠陥以外変えず、出力同一性を優先 | [TUGboat Knuth 2008](https://www.tug.org/TUGboat/tb29-2/tb92knut.pdf) | 意味論ゴールデン固定後は「より良い丸め」でも既存を動かさない(D1i-4) |

**運用への落とし込み**: [permanence-prevention](2026-07-12-m2-permanence-prevention.md)の予防5手 + AGENTS着手前チェック。

## B. 失敗後(参考) — 先人が「壊してから払った」側

| 型 | 先人 | 対応 | 一次出典 | 結果 |
|---|---|---|---|---|
| R1 全書き直し | Olive / OpenCut | ground-up rewrite。Oliveは0.1ファイル切断 | [Olive 0.2 Quickstart](https://github.com/olive-editor/olive/wiki/Olive-0.2.x-Quickstart); [OpenCut #811](https://github.com/OpenCut-app/OpenCut/issues/811) | 出戻りを受容。Motoliiは採らない(Document切断は棄却) |
| R2 未完rewrite | Natron | キャッシュを一から書き直したが完成せず | [Natron#417](https://github.com/NatronGitHub/Natron/issues/417) | 停滞。所有権/キャッシュを後回しにするとこの型 |
| R3 migrationで吸収 | Blender / OTIO | `do_versions` / schema_version昇降 | Blender handbook; [OTIO versioning](https://opentimelineio.readthedocs.io/en/latest/tutorials/versioning-schemas.html) | 形状進化の出口。D1eの席。第一選択ではない |
| R4 Legacy残置 | AE | Obsoleteカテゴリ | 同上P4出典 | 画素変更の出口。予防(新口)と一体 |
| R5 移行のscope制限 | Git | 移行仕様のnon-goals | [hash-function-transition](https://git-scm.com/docs/hash-function-transition) | 予防が破れてmigrationするときの副次ルール |

## 採用判定(予防優先)

| ID | 内容 | 層 | 判定 | 置き場所 |
|---|---|---|---|---|
| GR-PV-1 | 意味文書が先、コードは写し | 予防 | **採用** | permanence-prevention / AGENTS |
| GR-PV-2 | 恒久面を狭く(未決・未証明を焼かない) | 予防 | **採用** | 同上 |
| GR-PV-3 | 追加的変更のみ。解釈変更禁止 | 予防 | **採用** | 同上 |
| GR-PV-4 | 依存直列を守る(D1i-2前にD3しない) | 予防 | **採用** | M2並列レーン |
| GR-PV-5 | 完了=意味の審判(テスト緑だけでは完了にしない) | 予防 | **採用** | specsルール8 |
| GR-RW-3 | 形状→migration / 画素→新variant(破れたとき) | 復旧 | **採用**(副) | M2実装ガード |
| GR-RW-5 | migration PRのnon-goals | 復旧 | **採用**(副) | D1e発注時 |
| GR-RW-7〜9 | カレンダー / 全schema_version / Document切断 | — | 延期/棄却 | rework初版どおり(初版=本ファイルのgit履歴。#117はmergeコミットで履歴保持) |

旧GR-RW-1/2/4/6は予防側GR-PVへ統合(名前を予防に寄せた)。

## 選択バイアス

- 失敗例は有名になりやすく、予防の成功は「何も起きなかった」で見えにくい
- 因果未検証。反例未探索
- Persistence Last等のブログ事例は一次出典要件を満たさないため採用していない

## 改訂記録

- 2026-07-12: 初版(失敗後中心)
- 2026-07-12: 予防を本・失敗後を副に再構成。運用正本を permanence-prevention へ分離
- 2026-07-12: 確定(#117マージ)に際し規律2適用除外のユーザー決定を記録
