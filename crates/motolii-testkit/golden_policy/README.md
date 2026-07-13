# ゴールデン分類と regenerate マーカー(D1i-4 / S16 / #53)

台帳: [`classification.tsv`](classification.tsv)。CI: `scripts/check-golden-update-policy.sh`。

| class | 意味 | 更新 |
|---|---|---|
| `semantic` | 既存variantの意味を永久固定する審判(S16) | **禁止・例外なし**。`MOTOLII_REGENERATE_WHEN`でも迂回不可。新variant+新ファイル(+台帳追加)のみ |
| `provisional` | C-1系sRGBブレンド依存等の暫定審判(#53) | ファイルに`MOTOLII_REGENERATE_WHEN:`がある場合のみ更新可 |

空の`semantic`集合は拒否(空の禁止CIを運用に乗せない)。現行の意味論ゴールデンは D1i-2 の `d1i2_pathop_geometry.rs`。

## マーカー規約

ファイル先頭付近に固定文字列を置く(コメント内で可):

```text
MOTOLII_GOLDEN_CLASS: semantic
```

または暫定:

```text
MOTOLII_GOLDEN_CLASS: provisional
MOTOLII_REGENERATE_WHEN: srgb-blend-to-linear
```

`MOTOLII_REGENERATE_WHEN`の値はイベント識別子(自由文字列)。例: `srgb-blend-to-linear` / `vello-overlay-aa`。

参照PNGを置く場所は従来どおり `crates/motolii-testkit/golden/`(M2E-2保護)。分類台帳は本ディレクトリ(CODEOWNERS別保護)に置く。
