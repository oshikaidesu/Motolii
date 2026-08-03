# 仕様書プロセス(並列エージェント開発の前提)

各フェーズ(M0〜M5)の実装前に、このディレクトリの仕様書を「確定」または「段階発注可」にしてから着手する(「ドラフト」のまま着手しない)。仕様書はエージェントへの発注書であり、**タスクの粒度は「1タスク = 1エージェント = 1PR」で完結する単位**に揃える。段階発注可では**タスク表の依存を満たした行だけ**着手する。

## 仕様書のステータス

- **確定(frozen)**: このフェーズのタスクは着手可能。インターフェース変更はPRでの仕様書改訂を先に行う
- **段階発注可(ready with gates)**: 人間決定は完了。タスク表の依存を満たした行から着手可能
- **ドラフト(draft)**: 方向性の記述。凍結ゲート(M1完了後)または前フェーズの結果を受けて確定させる

後続の停止ゲートが発効している場合は、上の一般則より停止ゲートを優先する。M2基盤再締結はmainで発効済み。M3はU0a入場完了後、各タスクの依存を満たせば段階発注可である。

| 仕様書 | ステータス |
|---|---|
| [M0-spikes.md](M0-spikes.md) | 確定 |
| [M1-vertical-slice.md](M1-vertical-slice.md) | 確定(M0の採否判断で該当箇所を更新) |
| [M2-document-model.md](M2-document-model.md) | **基盤再締結済み / 段階発注可**(P1修復とA〜C証跡がmain発効済み。D5は統合/E2E審判pending。歴史から再採択したD1n external revisionは独立follow-up・未実装) |
| [M3-ui-integration.md](M3-ui-integration.md) | **ドラフト / UI責任境界・surface topology決定、platform受入比較中**(React chrome + native Stage/Timeline + headless interaction + 1 surface/2 viewport/opaque WebView islandsは決定。製品surface統合はG0-9実機待ち、plugin UI公開契約は分離したG0-3 / GAP-13待ち) |
| [M4-cache-and-analysis.md](M4-cache-and-analysis.md) | ドラフト(K0 RoD/RoI test-only契約凍結済み。既知実装調査とM4初版採択地図は作成済み。K1〜K8製品runtime前に採択probeと一意な`DO`を閉じる) |
| [M5-3d-and-post.md](M5-3d-and-post.md) | ドラフト(P0I等の意味decision／test-only fixtureは可。既知実装調査は比較中として作成済み。製品runtime前に反対側レビューとM5採択地図を閉じる) |

## タスク粒度のルール

1. **1タスク=1PR**: 差分が1クレート(+テスト)に収まり、レビュー可能なサイズ(目安: 実装数百行)であること。超えるなら分割する。
2. **完了条件は変更面を観測できるoracleで判定**: 各タスクは
   `PRIMARY_ORACLE / REPO_LANES / EXTERNAL_GATES`を持ち、変更した契約を直接失敗させられる
   fixture、test、guardまたはcheck commandを固定する。`cargo test --locked --workspace`はRust laneの
   必要条件であり、React、docs、製品E2E、実機、人間審判の代替ではない。詳細は
   [repository validation topology決定](../reviews/2026-07-31-repository-validation-topology-decision.md)。
   「動いた気がする」、変更面を観測できないgreen、未実行を完了条件にしない(落とし穴D-2)。
3. **依存の明示**: 各タスクは依存タスクIDを持つ。依存先が未完了のタスクには着手しない。依存のないタスク同士のみ並列に走らせる。
4. **インターフェース先行**: 並列タスク間の境界は、仕様書内の型シグネチャ(trait/struct)として先に文章で固定する。実装中に境界を変えたくなったら、実装を止めて仕様書改訂PRを先に出す。
5. **モック許可**: 依存先が未完了でも、仕様書のシグネチャ通りのモックを自作してテストを書いてよい(結合はインテグレーションタスクで検証)。
6. **テスト不可侵(報酬面の分離)**: ゴールデン参照画像・受け入れテストの改変を実装タスクに含めない。**テストが間違っていると思ったら実装を止めて報告する**(テストの削除・期待値書き換え・実装側のspecial-caseで「緑にする」ことを禁止)。参照画像の更新が正当な場合は、理由を明記した独立の「テスト更新PR」に分離する(根拠は[pitfalls H-2](../pitfalls-and-roadmap.md))。
7. **着手前の導線**: 各仕様書末尾の**「実装ガード」節**と、タスクが触る領域の関連ドキュメント(方針節・落とし穴)を読んでから着手する。**仕様書の未決事項に依存するタスクには着手しない** — LLMエージェントは未決を「もっともらしいデフォルト」で静かに埋める(intent drift)。未決は仕様書改訂PRで先に潰す([pitfalls H-3](../pitfalls-and-roadmap.md))。
8. **恒久焼き込みの予防**: Documentスキーマに触るタスクは、着手前に[permanence-prevention](../reviews/2026-07-12-m2-permanence-prevention.md)と[AGENTS.md](../../AGENTS.md)の条件別routingを読む。**意味文書が先、コードは写し。テスト緑≠完了**([pitfalls H-4](../pitfalls-and-roadmap.md))。
9. **依存優先・発明工程なし**: [既知実装採択・置換開発モデル](../known-implementation-adoption-model.md)と[責任最小化ゲート](../reviews/2026-07-24-dependency-first-responsibility-gate.md)に従い、利用者成果と機構classを先に固定し、既知実装調査と採択地図を製品runtime実装より前に閉じる。正本と`decision-index.md`で一度裁定した`REUSE / ADOPT / WRAP / PORT / PATTERN / EXTERNAL`を後続粒が継承する。粒ごとにecosystemを再調査せず、必須oracle、license、platform、security、maintenanceの具体的反証またはadapterの共有基盤化がある時だけ再裁定する。新機構の`BUILD`を通常taskにせず、modelは仕様化せず利用者例外へ返す。既完了や投入工数を維持理由にせず、独自機構は同じoracleへ通す縦slice置換で単一ownerを切り替え、旧routeを`FROZEN → RETIRE`する。
10. **利用者成果の背骨を先に固定する**: 複数粒にまたがる挙動は[自律再接続決定](../reviews/2026-08-04-outcome-spine-autonomous-gap-research-decision.md)に従い、通常製品routeの操作列、stable identity、成功出口、失敗回復、自動oracle、external gateを先に置く。各実装は一契約境界のまま、調査不足を`REUSE / REMAP / REDUCE / 再調査 / WAIT_TARGET`へ局所処分する。古い`next`や粒数を進捗軸にせず、実装後の現行codeから次edgeを再計測する。

## 仕様書テンプレート

```markdown
# M{n}: {フェーズ名}
ステータス: 確定 | 段階発注可 | ドラフト
## 目的(このフェーズが退治する落とし穴)
## スコープ外(やらないこと)
## インターフェース契約(並列タスク間の境界となる型・trait)
## タスク分割
| ID | 内容 | 依存 | 完了条件 |
各task行の完了条件には次を併記する:
PRIMARY_ORACLE: <変更した契約を直接失敗させられる既存command>
REPO_LANES: <docs | policy | tooling | rust | web-build | web-contract | web-visual>
EXTERNAL_GATES: <NONE | 名前付きplatform / product E2E / human / hardware gate>
## 並列レーン(同時に走らせられるタスク列)
## フェーズ完了条件
## 実装ガード(先行ツールの失敗・ユーザー不満クロスチェック)
## 未決事項
```

補足: 「実装ガード」節は、出荷済みツールの失敗・ユーザー不満の調査(2026-07-11実施、M1〜M5全仕様書に追記済み)をタスクIDに紐付けたもの。完了条件が既存タスクに追加されている場合があるため、**タスク着手時はタスク表と実装ガードの両方を読む**。
