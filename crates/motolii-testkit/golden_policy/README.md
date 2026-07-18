# ゴールデン分類と regenerate マーカー(D1i-4 / S16 / #53)

正本は [`classification.tsv`](classification.tsv)。CI: `scripts/check-golden-update-policy.sh`。

| class | 意味 | 更新 |
|---|---|---|
| `semantic` | 既存variantの意味を永久固定する最小oracle artifact(S16) | **禁止・例外なし**。変更したければ新variant+新oracle(+台帳追加)のみ。API・fixture・runtime配線を担うharnessは分類しない |
| `provisional` | C-1系sRGBブレンド依存等の暫定審判(#53) | ファイルに`MOTOLII_REGENERATE_WHEN:`がある場合のみ更新可 |

空の`semantic`集合は拒否(空の禁止CIを運用に乗せない)。BlendModeは期待値を`tests/oracles/d1i3_blend_mode.tsv`へ分離済みで、`d1i3_blend_mode.rs`はそれを読む変更可能なharness。残るD1i-2 PathOpとD1i-3 LookAt・Follow / Bezier / Transform合成は、API変更が必要になる前に同じ形へ段階移行する。暫定は D7 の `d7_clipping_mask.rs`(C-1 sRGBブレンド依存のクリスタ合成検証)。

`git merge-base` / `git diff` 失敗(shallow clone等)は **fail-closed**(空振りでOKにしない)。CIは`fetch-depth: 0`。

台帳の**初回登録PR**でも、HEAD台帳で`semantic`とされた既存ファイルの変更/削除は拒否する。新規`semantic`ファイルの追加(`git` status `A`)と台帳へのパス追加のみが正規ルート。

base 台帳の`provisional`行も保護する。台帳から外してマーカー無し更新する迂回は拒否。`provisional`→`semantic`昇格のみ許可。ファイル変更の判定は HEAD 分類を優先し、未分類なら base 分類を参照する。

既存のwhole-file semantic harnessからoracleへ移す場合は、[`migrations.tsv`](migrations.tsv)へ旧harness→新oracleを明示する。旧harnessが残り、移行先が存在してHEAD台帳で`semantic`の場合だけ、旧行の置換を許可する。これは個別PRのoverrideではなく、保護対象の所在を追跡する不可逆な移行台帳である。

## マーカー規約

`semantic` のファイル内`MOTOLII_GOLDEN_CLASS`は任意(台帳が正本)。

暫定は必須:

```text
MOTOLII_GOLDEN_CLASS: provisional
MOTOLII_REGENERATE_WHEN: srgb-blend-to-linear
```

`MOTOLII_REGENERATE_WHEN`の値はイベント識別子(自由文字列)。例: `srgb-blend-to-linear` / `vello-overlay-aa`。

参照PNGを置く場所は従来どおり `crates/motolii-testkit/golden/`(M2E-2保護)。分類台帳は本ディレクトリ(CODEOWNERS別保護)に置く。
