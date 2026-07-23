# Terra + Grok発注運用

状態: **決定**

日付: 2026-07-22

## 決定

当面のMotolii実装は、新しい製品意味や公開契約を発明する段階より、決定済みのUI境界・headless kernel・React資産移管契約に従って穴を埋める段階が中心である。その間は完遂速度を優先し、通常発注を次の役割に固定する。

- Codex: 仕様・コード事実・長期境界を照合し、closed order、STOP線、最終統合を所有する
- GPT-5.6 Terra (`gpt-5.6-terra`): Codex承認済みclosed orderの範囲だけを隔離worktreeで実装する
- Cursor Grok 4.5 High (`cursor-grok-4.5-high`): 実装担当と分離したread-only検収で実diff・試験・契約迂回を監査する
- Fable: ユーザーが明示した大地図・全粒・全体構造のread-onlyレビューに限定する

Claude Sonnet / Opusの監督付き経路は、Codex CLIまたはCursor Grokが利用不能であることをユーザーへ明示した場合のfallbackに下げる。旧経路の差分・検収証跡は歴史資料として残すが、現行の黙認経路ではない。

## 変えない停止線

- 実装発注は1回に1契約境界、隔離worktree、変更許可ファイルの閉じたallowlistとする
- `ORDER: READY`、task hash、`CODEX PRECHECK: APPROVED`が揃うまで実装担当を起動しない
- 未決の仕様、公開API、Document意味、plugin契約、永続形式はTerra/Grokに発明させない
- Grok検収が`VERDICT: ACCEPT`かつP0/P1=0でなければ採用・commit・pushしない
- 外部モデルの出力は根拠でなく未検証の助言とし、Codexが必須試験と統合を再確認する

## 見直しトリガー

この能力仮定は2026-07-22時点の運用判断であり、製品契約ではない。同一粒度でTerraのSTOP/未完遂またはGrokのP0/P1指摘による差し戻しが3回連続する、モデルIDが利用不能になる、または実装の主体が新しい意味設計へ戻る時点で再評価する。
