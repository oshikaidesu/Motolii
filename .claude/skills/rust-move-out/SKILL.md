---
name: rust-move-out
description: Rust の code をコアから出す時の手順 — `impl` への溶接を外し、呼び出しを rust-analyzer の構造置換で張り替え、crate をまたいで移す。「最小コア」「拡張へ出す」「module を移したい」「メソッドを自由関数にしたい」時に使う。正規表現で呼び出しを書き換えない。
---

# コアから出す

Rust の**固有 impl はその型を定義した crate にしか書けない**。だから `impl StoreView { fn foo(&self) }` の形で書かれた code は、file を何個に割っても元の crate から出られない。出すには 3 段階を順に踏む。

## ① 溶接を外す(`&self` → `view: &X`)

`impl X { fn foo(&self, ..) }` を `pub fn foo(view: &X, ..)` にする。中の `self.` は `view.` へ。

同じ block の中で互いを呼んでいる所は `view.bar(..)` → `bar(view, ..)` になる。

## ② 呼び出しを張り替える — **`rust-analyzer ssr` を使う。正規表現を使わない**

```sh
rust-analyzer search '$a.foo($b)'                        # まず数を見る
rust-analyzer ssr '$a.foo($b) ==>> crate::to::foo($a, $b)'
```

構文木で照合するので、正規表現が必ず壊す所を壊さない:

- 複数行の連鎖 `view\n  .foo(x)`
- `Some(id) => x.foo()` の `=>` を受け手と誤認する
- `..doc.view().foo()` の struct update を飲み込む
- 受け手が値か参照かで `&` を足し引きして `&&&&&…` になる

置換先の path が**解決できないと拒否**される。だから**自由関数を先に作ってから** ssr を流す。置換後の path は解決できる最短形へ自動で縮む。

**この build(rust-analyzer 1.96.1)の罠**: UFCS 形 `crate::X::foo($a, $b) ==>> ..` は
`hir-ty/src/next_solver/interner.rs` で panic する(`Try to use attached db, but not db is attached`)。
素の `$a.foo($b)` 形を使う。ただしそれは**型で絞れない**ので、同名メソッドが他の型にあると巻き込む。
必ず先に `search` で件数を見て、想定と合うか確かめる。

## ③ 移す

`git mv` で file を移し、`mod` 宣言と path を張り替える。module の移動に追随する道具は無い
([rust-analyzer#2178](https://github.com/rust-lang/rust-analyzer/issues/2178)・[#8872](https://github.com/rust-lang/rust-analyzer/issues/8872)・[#12774](https://github.com/rust-lang/rust-analyzer/issues/12774) で要望止まり)。ここは手当て。

移った先が要る物は `pub` にする。**それが読み口の契約**になるので、何を公開したかを見る。

## 門は速い物だけ

`cargo check -p <コアの crate> --lib --tests`(この repo で 17 秒)を内側の loop に、
全 crate の `cargo check --tests` を module ごとに 1 回(30 秒)。
`cargo build` は 30〜50 秒かかるので判定には使わない。

**型検査が端から端まで効く変更**なので、GPU の test suite は回さない(16 分かかり、元から赤い物が混ざっていて判定に使えない)。

## コアに残す物との線

この repo の法は [core-is-values-time-eval-contract]:
**コアは「作品の値・時刻・評価・契約」だけを持つ。絵になる物は全部拡張。**

判断に迷ったら、その code が答えるのが「書類に何が書いてあるか」なら残す、
「画面にどう出るか」なら出す。両方に跨る物(見た目を保つ編集など)は、
**コアが契約を宣言し、外が実装を登録する**形にする — 効果・幾何・変換器が全部この形。
