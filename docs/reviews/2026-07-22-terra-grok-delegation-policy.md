# タスク適応型の発注運用

状態: **ARCHIVED**

日付: 2026-07-22（2026-07-24改訂、2026-07-25アーカイブ）

> 2026-07-25に[Opus 5 / Spark / Grok監督ループ](2026-07-25-opus-spark-grok-supervision-loop-decision.md)
> へ置換した。本書のtask class、model routing、Fable必須検収、Grok order draftは現行運用ではない。
> 比較経緯を残す歴史資料としてのみ参照する。

## 決定

2026-07-22に決めたTerra固定は撤回し、モデルを製品契約へ固定しない。Codexがclosed orderを承認する前に
`TASK_CLASS`を選び、2026-07-23〜24の同一課題比較で確認した速度・精度・費用傾向に基づいて受注者を変える。
モデル名はaliasでなく完全IDを発注書へ記録する。

| `TASK_CLASS` | 選択条件 | 実装モデル | 検収 |
|---|---|---|---|
| `mechanical` | 単一ファイル中心の機械変更、意味判断なし、closed packetで完結 | `gpt-5.3-codex-spark` | Grok |
| `standard` | 既決の単一契約境界を通常実装 | `gpt-5.6-luna-none-fast` | Grok |
| `rapid` | 同じく閉じた境界で、費用より経過時間を優先 | `gpt-5.6-terra` | Grok |
| `complex` | 複数不変条件、原子性、失敗復旧、複数ファイルの整合 | `gpt-5.6-sol-none-fast` | Grok + Fable |
| `cross-boundary` | 複数仕様・公開境界・全体構造の横断整合が支配的 | `gpt-5.6-sol-none-fast` | Grok + Fable |

- Codexは仕様・コード事実・長期境界を照合し、分類、closed order、STOP線、最終統合を所有する
- Sparkは2026-07-24のU0e-2固定fixtureで一次検収10.58秒、二次実装31.52秒だった。一次では主要欠陥5系統を
  検出したが、二次は自己テスト2/2成功後も独立reader probeでREJECTだったため、`mechanical`でもGrok検収を外さない
- Sparkへ巨大な会話履歴を渡した再開試験はsystem errorになった。`mechanical`は会話履歴を継承させず、closed order、
  変更許可ファイル、明示authorityだけで実行する。repo横断の歴史調査や複数仕様の意味判断が必要なら上位クラスへ戻す
- Sparkの研究preview用別利用枠は通常枠の温存に使うが、上限値・reset・恒久提供をscriptへ焼かない。
  利用不能時は通常枠へ黙ってfallbackせず、Codexが再実行またはクラス変更を判断する
- Cursor Grok 4.5 High (`cursor-grok-4.5-high`)は全クラスで実装担当と分離したread-only検収を行う
- Claude CodeのFable 5 (`claude-fable-5`)は`complex`と`cross-boundary`で追加のread-only検収を行う。
  大地図、全粒、設計比較、契約横断レビューはユーザーの個別指定を待たずFable候補にする
- Claude Codeの5時間利用枠拡大は運用余力として利用するが、価格・利用倍率・期間を製品契約やscriptへ焼かない。
  利用不能または枠不足なら、必須Fable検収を黙って省略せず停止し、Codexがクラス変更か再実行を判断する
- Claude Sonnet / Opusの実装経路は、選択済みCodexモデルまたはCursor Grokが利用不能で、fallback理由を
  ユーザーへ明示した場合だけ使う

## 変えない停止線

- 実装発注は1回に1契約境界、隔離worktree、変更許可ファイルの閉じたallowlistとする
- `ORDER: READY`、task hash、`TASK_CLASS`、完全model ID、`CODEX PRECHECK: APPROVED`が揃うまで実装担当を起動しない
- 未決の仕様、公開API、Document意味、plugin契約、永続形式は外部モデルに発明させない
- Grok検収が`VERDICT: ACCEPT`かつP0/P1=0でなければ採用・commit・pushしない
- Fable必須クラスはFableも`VERDICT: ACCEPT`でなければ採用・commit・pushしない
- 外部モデルの出力は根拠でなく未検証の助言とし、Codexが必須試験と統合を再確認する

## 見直しトリガー

この対応表は2026-07-24時点の運用判断であり、製品契約ではない。同一クラスでSTOP/未完遂またはP0/P1指摘による
差し戻しが3回連続する、モデルIDやClaude Code利用枠が変わる、または実装の主体が新しい意味設計へ戻る時点で
比較を再実行する。単発の速度差や一時的な割引だけでは表を書き換えない。
