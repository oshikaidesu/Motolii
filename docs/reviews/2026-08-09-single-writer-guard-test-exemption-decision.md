# 単一writer guard の試験文脈除外 — 明示マーカー方式

日付: 2026-08-09
状態: **決定 / guard実装済み**

## 0. この文書の扱い

`crates/motolii-doc/tests/mut_document_deny.rs` の判定範囲だけを変える。
規律#4(single writer)自体、`DocumentWriter::edit` の唯一性、Document schema、
公開APIは一切変更しない。

## 1. なぜ変えたか

R2 stage geometry / hit test / pointer の統合時、guardが3件を violation として
落とした。3件すべて `#[cfg(test)]` 内の fixture builder で、形は
`fn push_rect(doc: &mut Document, ...) -> LayerId` だった。

guardを迂回するには fixture を macro へ潰すか、31箇所の呼び出しを所有権渡しへ
書き換えるしかない。実際に macro 化を試した結果、可読性と引数の型検査を失い、
**安全性の増分はゼロ**だった。払う対価に対して守るものが無い。

同時に、guardが守れている範囲を測った。`Document::new_current()` は public であり、
main自身の試験が

```rust
let mut document = Document::new_current();
let layer = document.layers.allocate("r0-layer").expect("layer");
```

とローカル所有で書き換えていて、これはguardを通る。guard冒頭のdoc commentも
「単一writer(F-2)は型だけではすり抜け可能(Document が pub)」と明記している。

つまりguardが実際に禁じているのは「Documentを外部から書き換える能力」ではなく
「可変参照を関数シグネチャで渡す書き方」である。前者は既に開いている。

## 2. 決定

`#[cfg(test)]` 文脈の fixture は、同一行の明示マーカーで guard から除外できる。

```rust
doc: &mut Document, // single-writer-exempt: fixture が所有する Document
```

条件は3つすべてを満たすこと。

1. **同一行**の行コメントで宣言する。別行・別スコープからの一括除外はできない
2. マーカーの後に**理由**を書く。空なら除外しない
3. 除外を honor するのは `#[cfg(test)]` を含む file か `tests/` 配下の file だけ

guardは除外件数と場所を毎回 `eprintln!` で出す。除外は隠れない。

## 3. なぜ「reviewで拾えるから緩める」ではないか

この判断の根拠は「まとめ役が後で校正するから自動gateは不要」ではない。
2026-08-07の成果が2日間mainへ入らず誰も気づかなかった原因は、人手のreview頼みで
あったことである。並列数が増えれば総監督が律速になる。

緩める根拠は**gateが対象外を撃っていてreview注意力を消費していたこと**に限る。
gateは無くさず精密にする。この区別を残さないと、次に緩める判断を誤る。

## 4. 弱めないもの

- 規律#4: Documentを書き換えるのは編集threadだけ
- `DocumentWriter::edit` が唯一の製品書込route
- 製品module(非test)の `&mut Document` は従来どおり無条件 violation
- scanner退行の番兵(`motolii-doc` 内に本物のヒットがあること)
- コメント・文字列の誤検出防止、寿命付き・path修飾の検出

## 5. 非目標

- `#[cfg(test)]` ブロックの括弧対応解析をguardへ持ち込むこと
  (skipperのバグでguardが黙って製品コードを守らなくなる状態は、厳しすぎるguardより悪い)
- `Document` の可変APIを private 化する本来の修正(別契約)
- `motolii-testkit` を除外対象に含めること(製品crateとして従来どおり判定する)
- 本決定を根拠に他の絶対規律guardを緩めること
