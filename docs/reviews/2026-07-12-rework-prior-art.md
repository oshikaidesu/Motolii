# 出戻りリスク: 先人ソフトの対応と転移条件(2026-07-12)

ステータス: **仮説メモ**(一次資料つきの整合事例集 — **設計根拠ではない**)。運用規律は[reviews/README.md](README.md)。判定語(採用/縮小/延期/棄却)は末尾「LLMガードレールへの採用」節に併記。

## これは何か

M2は恒久スキーマ×並列初陣×検証の弱さが重なる([入場ゲート](2026-07-11-M2-entry-gate.md))。docsはすでに出戻りを予言している(D-1/G-1/H-3、第二監査のD1a〜c再開)。本メモは**同じ型の出戻りに先人がどう対応したか**を一次資料で棚卸しし、LLMエージェント向けガードレール([後述](#llmガードレールへの採用判定採用縮小延期棄却))の入力にする。

**結論の位置づけ**: 反例探索は未了。「成功法則」ではない。因果(その対応がなければ死んだか)は未検証。使えるのは**対応パターンの型**と、Motoliiへの転移条件だけ。

## リスク型 → 先人の対応

### R1. 基礎アーキを早期に間違えて全書き直し

| 先人 | 対応 | 一次出典 | 結果 |
|---|---|---|---|
| **Olive** | 0.1→0.2で**ground-up rewrite**。色管理(OCIO)・ノード合成・ディスクキャッシュ等を入れ直す。**0.1ファイルは0.2非互換** | [Olive Wiki 0.2 Quickstart](https://github.com/olive-editor/olive/wiki/Olive-0.2.x-Quickstart)(*"Olive has been redesigned from the ground up, and so too its' file format"* / *"0.1.x files won't be compatible with 0.2.x"*); [Wikipedia Olive (software)](https://en.wikipedia.org/wiki/Olive_(software))(0.1 codebase unsustainable) | 出戻りを**受容**した例。互換移行ではなく切断 |
| **OpenCut** | エンジン/UI分離・plugin-first・多プラットフォームのため**rewrite tracking**。旧版はclassicへ残し、新は別ホスト | [OpenCut #811 Tracking: rewrite](https://github.com/OpenCut-app/OpenCut/issues/811); [OpenCut README](https://github.com/opencut-app/opencut/) | 境界の後付け不能を認め、**並行に旧版を温存**しながら作り直し |

**転移**: Motoliiは「Olive型の切断」を避けたい側。[凍結ゲート](2026-07-10-freeze-gate-declaration.md)+入場ゲートはその予防。ただし**ゲート通過後も恒久スキーマへの早焼きはOliveと同じ切断コストを積む** — D1a「完了」直後の第二監査フォローアップ(D1g/D1h/D1i)が既にミニ出戻り。

### R2. 並行・キャッシュ契約が固まらないまま機能を積む

| 先人 | 対応 | 一次出典 | 結果 |
|---|---|---|---|
| **Natron** | キャッシュ(trimap含む)のバグを認め**一から書き直し**を試みたが**完成せず**。スレッド数制限・DiskCacheノードで症状緩和 | [NatronGitHub/Natron#417](https://github.com/NatronGitHub/Natron/issues/417)(開発者: *"bugs in the Natron 2 cache (and the \"trimap\" system), which is why we rewrote it from scratch... but never got a finished version"*); [discuss.pixls.us](https://discuss.pixls.us/t/natron-cache-not-clear-while-rendering-viewport-issue/7803)(*"completely reworked for Natron 3.0, but unfortunately was never stable enough"*) | 出戻り作業自体が未完で開発が停滞する型 |

**転移**: F-2単一writer・M4キャッシュ並行契約・D8は「後から直す」とNatron型になる。M2で所有権骨格を先に機械化するのはこの先例と整合(既採用方針の裏書き)。

### R3. データ形状は進化させるが、後方互換を機構で払う

| 先人 | 対応 | 一次出典 | 結果 |
|---|---|---|---|
| **Blender** | ロード時`do_versions`で増分変換。後方互換を原則保証。**批判的前方互換破壊はメジャー周期(~2年)に限定**。設計タスクに互換節+Coreレビュー必須。画素一致は *"as best as possible"* | [Blend File Compatibility (公式handbook)](https://developer.blender.org/docs/handbook/guidelines/compatibility_handling_for_blend_files/) | 出戻りを**migrationで吸収**する運用。意味の完全保存は約束しない |
| **OTIO** | 各SerializableObjectに`schema_name`+`schema_version`。読込時upgrade関数連鎖、書出時downgrade可能。メモリ上は常に現行版のみ | [OTIO Versioning Schemas](https://opentimelineio.readthedocs.io/en/latest/tutorials/versioning-schemas.html) | 形状変更を**版付き関数**で明示。黙ったフィールド意味変更を構造的に嫌う |
| **SQLite** | 3.0.0以降、ファイル形式の**後方互換を公約**。前方は新機能使用時に壊れ得る。形式を非互換に変えない | [sqlite.org/formatchng.html](https://sqlite.org/formatchng.html); [sqlite.org/onefile.html](https://sqlite.org/onefile.html) | 「焼いたら守る」側の極。破壊的フォーマット変更をそもそも選ばない |

**転移**: MotoliiのD1e(マイグレーション枠)+M2E-12(`min_reader_version`/unknown-keys)はBlender/OTIO/SQLiteの縮小版。S12(件数一致だけでは意味保存にならない)はBlenderの *"as best as possible"* とOTIOの明示upgradeの中間 — **意味保存テストを完了条件に足す**方向が整合。

### R4. 画素・アルゴリズム意味の変更は旧variantを残す

| 先人 | 対応 | 一次出典 | 結果 |
|---|---|---|---|
| **After Effects** | 新アルゴリズムは**別エフェクト**(新match name)。旧はObsoleteカテゴリへ。旧プロジェクトは旧match nameで継続。新と旧で**レンダ結果が異なる**ことを公式に明記 | [Adobe: Obsolete effects](https://helpx.adobe.com/after-effects/using/obsolete-effects.html); [Adobe blog 2016-07 GPU effects](https://blog.adobe.com/en/publish/2016/07/10/after-effects-cc-2015-3-in-depth-gpu-accelerated-effects)(*"This version of Gaussian Blur is a new effect"* / LegacyはObsoleteへ / match name分離) | データmigrationではなく**意味の分岐**で出戻りを避ける |

**転移**: ユーザー決定S16案1([第二監査](2026-07-12-code-audit-2nd-d1.md))と一致 — 形状→migration / 画素→新variant / 既存意味→固定。執行は意味論ゴールデン+更新禁止CI(D1i-2/D1i-4)。

### R5. 移行作業のスコープ膨張を殺す

| 先人 | 対応 | 一次出典 | 結果 |
|---|---|---|---|
| **Git** (hash移行設計) | 移行の目標第一項が他者無行動。**non-goals**に「ついでに他のフォーマットバグを直さない」を明記。壊れたオブジェクトすらround-trip保持 | [git-scm.com/docs/hash-function-transition](https://git-scm.com/docs/hash-function-transition) | 移行PRのintent driftを設計文書で事前殺傷 |

**転移**: D1eや解凍手続きPRで「ついでにスキーマ整理」すると出戻りコストが二重化する。Git型non-goalsを発注書に書くのが安い(既に[成功先例メモ](2026-07-12-success-prior-art.md)S-3で技法として記録)。

## Motolii現状との重ね合わせ

| Motoliiの現象 | 近い先人型 | 既にある防御 | 残余 |
|---|---|---|---|
| D1a/b/c「完了」→第二監査でスキーマ再開 | Olive(早すぎ固定)の**縮小版** | 入場ゲート、監査フォローアップチケット化 | 「完了」判定がテスト緑に偏る(審判が仕様違反拒否になっていない — 第二監査「審判への含意」) |
| PathOp意味論未定のまま型だけ焼いた | Olive色管理の後入れと同型の芽 | S5/D1i-2「表が揃うまでマージしない」 | D1i-2未完のままD3を出すとOlive再演 |
| 並列エージェントが未決を埋める | Git移行のscope creepと同根 | H-3、仕様ルール7 | エージェント向けの**着手前チェックリスト**が散在 |
| キャッシュ/所有権を後回し | Natron | F-2骨格、D8、M4台帳 | D8未着手のままM3/M4並列を増やすと危険 |

## LLMガードレールへの採用(判定: 採用/縮小/延期/棄却)

調査結論をそのまま契約に焼かない([reviews/README](README.md)規律1)。以下のみ運用文書へ落とす。

| ID | ガードレール | 根拠先例 | 判定 | 置き場所 |
|---|---|---|---|---|
| GR-RW-1 | **恒久物チェック**: Documentスキーマ/ジャーナル/意味論ゴールデン/プラグイン契約を触るPRは、仕様の【決定】またはゲート項目またはユーザー決定への逆リンクが無い限りマージしない | Olive切断コスト、SQLite「焼いたら守る」 | **採用** | AGENTS.md + M2実装ガード |
| GR-RW-2 | **意味未定は完了にしない**: 意味論表・期待型表・互換方針が未完のタスクを「完了」と書かない。テスト緑≠完了(第二監査の教訓) | Olive早期固定、S5 | **採用** | AGENTS.md + specs/README |
| GR-RW-3 | **形状と画素を分離**: データ形状変更→migration経路必須 / 画素意味変更→新variant+旧維持 / 既存variantのゴールデン更新で通さない | AE Obsolete、Blender do_versions、S16 | **採用**(方針は決定済み。執行を明示) | AGENTS.md + D1i系 |
| GR-RW-4 | **クリティカルパスを飛ばさない**: D1i-2完了前にD3を発注・実装しない。スキーマ接触レーンの並列増は入場ゲート+直列依存表に従う | Olive/OpenCut(境界後付け)、Natron(未完rewrite) | **採用** | M2仕様並列レーン節(既存)+AGENTS |
| GR-RW-5 | **移行PRのnon-goals**: migration/解凍/フォローアップPRは「対象ID以外のスキーマ整理・ついで修正」をnon-goalsに書き、混入したら分割 | Git hash移行 | **採用**(縮小: 文書1節。専用CIは延期) | M2実装ガード + D1e発注時 |
| GR-RW-6 | **未決は停止**: 仕様未決・監査「ユーザー決定待ち」に依存する実装をデフォルトで埋めない。止めて仕様改訂PR | H-3、Git non-goals | **採用**(既存の強化) | AGENTS.md(既存ルールの明示リンク) |
| GR-RW-7 | Blender級の「メジャー周期まで破壊温存」カレンダー | Blender ~2年サイクル | **延期** | 外部ユーザー無しの今は解凍手続きで足りる |
| GR-RW-8 | OTIO級のschema_versionを全オブジェクトに付与 | OTIO | **延期** | Document.version+migrate枠でv1は足りる。必要なら解凍 |
| GR-RW-9 | Olive/OpenCut型の「旧フォーマット切断を許容する」方針 | Olive/OpenCut | **棄却** | ProjectV1使い捨て(M2E-11)以外のDocument切断は採らない |

## 選択バイアスの申告

- 調査は「出戻りが有名な失敗例」と「互換機構が文書化されている成功例」に偏る
- 反例(早期固定しても成功・互換機構があっても死んだ)は未探索
- Libre Arts等の二次解説は本メモの**根拠に使っていない**(NatronはGitHub issue/discussの開発者発言を一次とした)

## 改訂記録

- 2026-07-12: 初版。M2出戻り議論を受け、先人対応の棚卸し+LLMガードレール採用判定を追加
