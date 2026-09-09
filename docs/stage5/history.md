# History — one column for edits and records

2026-09-09 利用者: 「desk の history をクリスタ形式にしてくれ、もしくは git みたいな UI にして、
VSCode の可視化とかがいい例じゃね？」「ここで記録などクラッシュなどのログも観れると嬉しい」
「軽くでいいので git 的な機能を将来入れれるか？でもマージが面倒か」。

先例: Clip Studio Paint の履歴パレット(一覧 + 現在位置)と VS Code の Git Graph(縦の一本線)。
採ったのは前者の並びに後者の線 — 縦一本、上が古い、現在位置は 1 点だけ。分岐の枝はまだ描かない。

## 今の形

- 台帳は Rust が持つ(`motolii/ui/native/src/editor/history.rs`)。点は Document の
  `edit_head` に紐づき、`status["history"]` で窓へ渡る。窓は描くだけで、数えない。
- 編集は 1 段 = 1 点。保存・起動・前回の異常終了は同じ列に別の印で並ぶ。
- 点を押すと `historyGoto` が段まで戻る/進む。今居る点と、前の走行から読んだ点は押せない。
- 戻った先から編集すると、捨てられた先の点は列から消える(rerun の tip がそこで切れるため)。
- 台帳は `~/Library/Application Support/MotoliiStage5/history.jsonl` に 200 点まで残る。
  閉じる時に `end` を書き、次の起動でそれが無ければ「異常終了」を 1 行足す。

## 将来 git を入れるならここ

1. 点は既に `id`(走行 ID + 連番、再利用しない)と `parent` を持つ。線形リストではなく木。
2. 枝を足すのは `note()` の「捨てられた先を消す」を「別の親を持つ枝として残す」に変える所だけ。
3. 枝の名前は `Entry` に `branch` を 1 本足し、`snapshot()` が列と一緒に返す。
4. 窓側は `_RailPainter` が 1 本の線を描いている。列 x を枝の番号で決めれば Git Graph になる。
5. 段の実体は rerun の edit timeline の整数。枝は別の timeline か、同じ timeline の別区間になる。
6. 合流(merge)はこの版の外。2 つの `edit_head` を混ぜる意味を Document が持っていない。
