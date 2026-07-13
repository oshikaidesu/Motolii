# ゴールデン分類と regenerate マーカー(D1i-4 / S16 / #53)

正本は [`classification.tsv`](classification.tsv)。CI: `scripts/check-golden-update-policy.sh`。

| class | 意味 | 更新 |
|---|---|---|
| `semantic` | 既存variantの意味を永久固定する審判(S16) | **禁止・例外なし**。変更したければ新variant+新ファイル(+台帳追加)のみ。分類は台帳だけで足り、既存ゴールデン本体の編集は不要 |
| `provisional` | C-1系sRGBブレンド依存等の暫定審判(#53) | ファイルに`MOTOLII_REGENERATE_WHEN:`がある場合のみ更新可 |

空の`semantic`集合は拒否(空の禁止CIを運用に乗せない)。現行の意味論ゴールデンは D1i-2 の `d1i2_pathop_geometry.rs`。

`git merge-base` / `git diff` 失敗(shallow clone等)は **fail-closed**(空振りでOKにしない)。CIは`fetch-depth: 0`。

台帳の**初回登録PR**でも、HEAD台帳で`semantic`とされた既存ファイルの変更/削除は拒否する。新規`semantic`ファイルの追加(`git` status `A`)と台帳へのパス追加のみが正規ルート。

base 台帳の`provisional`行も保護する。台帳から外してマーカー無し更新する迂回は拒否。`provisional`→`semantic`昇格のみ許可。ファイル変更の判定は HEAD 分類を優先し、未分類なら base 分類を参照する。

## マーカー規約

`semantic` のファイル内`MOTOLII_GOLDEN_CLASS`は任意(台帳が正本)。

暫定は必須:

```text
MOTOLII_GOLDEN_CLASS: provisional
MOTOLII_REGENERATE_WHEN: srgb-blend-to-linear
```

`MOTOLII_REGENERATE_WHEN`の値はイベント識別子(自由文字列)。例: `srgb-blend-to-linear` / `vello-overlay-aa`。

参照PNGを置く場所は従来どおり `crates/motolii-testkit/golden/`(M2E-2保護)。分類台帳は本ディレクトリ(CODEOWNERS別保護)に置く。
