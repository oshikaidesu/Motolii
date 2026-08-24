# コンテキスト圧縮時の再開手順

## 目的

コンテキスト圧縮は、作業方針を変える合図ではない。圧縮後のエージェントは、直前の会話を
推測で再現せず、現在の構造と機械が出す現在地を読み直してから作業を続ける。

この文書の中心原則は次の一行である。

> **step が需要を決め、component が依存を閉じ、静的検査が現在地を決める。Cargo は最後に一度だけ使う。**

## 圧縮を検知した直後に戻る場所

1. 正本が `next/` であることを確認する。
2. `AGENTS.md`、このファイル、`next/README.md` の「現在の実装ルート」と「並列レーンへの発注テンプレート」を読む。
3. 次の静的コマンドを実行し、会話の記憶ではなく現在のファイルから再開点を得る。

```bash
MOTOLII_REPO="$(git rev-parse --show-toplevel)"
python3 scripts/plan_steps.py "$MOTOLII_REPO"
python3 scripts/plan_backlog.py "$MOTOLII_REPO"
python3 scripts/plan_waves.py "$MOTOLII_REPO"
python3 scripts/derive_entries.py "$MOTOLII_REPO"
python3 scripts/derive_components.py "$MOTOLII_REPO"
python3 scripts/check_evidence.py "$MOTOLII_REPO"
python3 scripts/check_coherence.py "$MOTOLII_REPO"
python3 scripts/check_responsibility.py "$MOTOLII_REPO"
python3 scripts/rehearse_parallel.py "$MOTOLII_REPO"
```

コードを変更した後だけ、必要に応じて `bash scripts/gen-inventory.sh` を実行する。生成物は
手で編集しない。

## まず確認する実装コンセプト

### 1. 最も早い step を起点にする

`plan_steps.py` が示す最も早い未通過 step だけを当面の需要とする。後ろの機能を先回りして
作らない。途中で必要な意味 component が無い、または5粒のどれかが赤なら、その component
を先に閉じてから step に戻る。

### 2. component は意味の責任境界で切る

次の5粒を一つの利用者意味として閉じられる単位を component とする。

`entry → meaning → evaluation → render → observable`

独立した状態遷移、失敗方針、undo/recovery、owner、観測結果のいずれかが異なるなら分ける。
単独では利用者に意味が見えない補助関数や見た目だけの部品は component にしない。巨大な
component に機能を詰め込まず、入力を受ける pane/stage、Document を書く Shell、評価・描画の
責任を必要に応じて別の component にする。ただし同じ意味のための二重の state owner と
二重の write route は作らない。

### 3. write route は一本のままにする

意味の書き込みは原則として次の流れを通る。

`Shell::update → Intent → Document::apply/apply_all → StoreView → Engine/Compositor`

pane は `StoreView` の投影、描画と export は共通の評価結果を読む。Shell の root や
`render_dispatch.rs` のような結線ハブは意味 component の責任に混ぜず、WIRE として後で結線する。

## Cargo を増やさない判断

次の問いは Cargo に聞かない。

- 何が存在するか、どう呼ぶか、構造体フィールドや enum variant の形は何か
- どこが重いか、どの step が先か
- 並列に分けられるか、責任が交差するか

これらは inventory、生成台帳、PageRank、`plan_waves.py` から得る。実装中にレーンごとの
`cargo check` や `cargo test` を繰り返さない。Cargo が必要なのは、静的に得られない次の境界だけである。

- enum の網羅性
- 借用・生存期間
- 公開型の境界
- 波末の消費点、実窓、push、引き継ぎ前の最終収束

コードを書き終えた波で一度だけ必要な検査を行い、同じ変更に対して Cargo を再実行しない。
コンテキスト圧縮後に同じ波の成功結果があればそれを再利用し、Cargo を新規起動せず静的検査だけを続ける。
`cargo fmt --all` の
ように無関係なファイルを大量に変える操作も実装の検査に使わず、差分を責任範囲へ閉じる。

## step を進める固定ループ

1. procedure の最初の未通過行を読む。
2. その意味が既にコードにあるか、入口だけが無いか、5粒のどこが欠けているかを `rg`、inventory、生成台帳で確認する。
3. 既存実装があれば重複実装せず、証拠・component 契約・procedure の記述を正す。
4. 無ければ、必要な component の write-set を先に決め、component を実装する。
5. `Intent` から Document、StoreView、評価、描画、observable まで一本で通す。
6. `file:line` の証拠、出典、検収条件を procedure と component 契約へ記録する。
7. inventory、entries、components、evidence、coherence、responsibility、parallel を静的に再生成・検査する。
8. step の静通が進んだことを確認し、次の最初の未通過行へ進む。

「実装した気がする」では完了にしない。入口があり、意味があり、結果が観測でき、台帳の赤が
消え、step が実際に進むことを一組として扱う。

## 並列へ渡すとき

並列の目的は速度より責任隔離である。開始前に `plan_waves.py` と
`rehearse_parallel.py` で write-set の交差を確認し、交差する仕事を同じ波へ入れない。
各レーンは次の項目だけを持つ。

```text
OUTCOME: 1つの観測可能な状態変化
STEP: procedure の番号または map id
COMPONENT: component id と赤い5粒
TARGET: 変更してよいファイル
WRITE-SET: 台帳が示す責任ファイル
WIRE-SET: 結線ハブ(意味レーンから除外)
DO-NOT-TOUCH: 他レーンのファイルと先回り機能
STATIC: 生成・証拠・coherence・責任・parallel の検査
RETURN: 変更、evidence、残った赤、静的検査の終了コード
```

WIRE は一人の owner に集約し、意味レーンは他のレーンの結線ファイルを触らない。並列数を
増やす前に、責任ファイルの重複と共有 state の有無を消す。

## 圧縮後の報告テンプレート

再開時の最初の内部メモは、次の順で短くまとめる。

```text
CURRENT: plan_steps の現在の段階 / 最初の未通過 step
DONE: 直前に静通した component と step
RED: component 5粒、evidence、coherence、responsibility の赤
WRITE-SET: 次に触るファイルだけ
CARGO: この波で既に実行済みか。未実行なら静的に進められるか
NEXT: component を先に閉じるか、step を実装するか
```

この報告を作るために会話履歴を再構成しない。コード、台帳、procedure、Git 差分が現在の
権威である。利用者が不在でも、意味が固定されている限りこのループを継続し、解釈が実質的に
変わる場合だけ停止する。
