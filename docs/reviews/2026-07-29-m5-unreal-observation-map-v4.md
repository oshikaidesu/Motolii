# M5 Unreal観察fragment map v4

作成日: 2026-07-29

状態: **完了／P2D-RCC4B-UNREAL Grok ACCEPT（P0/P1/P2=0）**

変更許可: 本fileの`配置欄`だけ

単一動詞: **配置する**

## 入力と固定fragment

入力は[Unreal Engine 5.8 capsule](2026-07-29-m5-capsule-unreal-translucency.md)だけである。以下は
主担当Codexがcapsuleの記載を意味変更せずID化した固定fragmentで、leafは本文、ID、追加／削除を変更しない。

- `U-O1`: translucent重なりにはsort問題があり、depth bufferだけでは前後を決められないと説明される。
- `U-O2`: translucencyは複数のpass位置を持ち、scene colorへのblendとdepth／velocityの扱いが同一ではない。
- `U-O3`: overdrawは層数に応じた性能問題になり、sort priorityは意図的な上書きである。
- `U-O4`: refractionはtranslucent material側の機能として扱われ、方式／pass／screen-space制約を伴う。
- `U-NO`: capsuleに該当する固定観察なし。
- `U-SRC`: Unreal Engine 5.8公式docs、取得日2026-07-29、capsule記載の3 URL。
- `U-NP`: Unreal material／pass／sort priority／IOR UIをMotoliiの公開語彙、Document、phase enumにしない。
- `U-LIM`: FROZEN / DELETE-LATER / 製品import禁止。

## 主担当固定配置

- phase admission / ordering: `U-O2 | U-SRC | U-NP | U-LIM`
- depth / opaque / cutout / soft alpha: `U-O1 U-O2 | U-SRC | U-NP | U-LIM`
- transparent交差 / sorting / OIT追加位置: `U-O1 U-O3 | U-SRC | U-NP | U-LIM`
- scene-color / refraction / resource lifetime: `U-O2 U-O4 | U-SRC | U-NP | U-LIM`
- capability不足 / unsupported / cyclic read: `U-NO | U-SRC | U-NP | U-LIM`

`|`はmatrixの列境界であり、配置する文字列には含めない。

## 固定配置matrix

各`配置欄`には上記IDだけを空白区切りで置く。観察は`U-O1`〜`U-O4`または`U-NO`、
他3列はそれぞれ`U-SRC`、`U-NP`、`U-LIM`だけを許す。ID本文の複製や言い換えをしない。

| 観測項目 | 観察fragment | source fragment | 非証明fragment | 持込禁止fragment |
|---|---|---|---|---|
| phase admission / ordering | U-O2 | U-SRC | U-NP | U-LIM |
| depth / opaque / cutout / soft alpha | U-O1 U-O2 | U-SRC | U-NP | U-LIM |
| transparent交差 / sorting / OIT追加位置 | U-O1 U-O3 | U-SRC | U-NP | U-LIM |
| scene-color / refraction / resource lifetime | U-O2 U-O4 | U-SRC | U-NP | U-LIM |
| capability不足 / unsupported / cyclic read | U-NO | U-SRC | U-NP | U-LIM |

## STOP

- 固定fragmentだけでは配置できず、新しい文、ID、資料、解釈が必要になる。
- provider横断比較、Motolii fixture対応、方式採択、公開契約を始める。
- 本fileの`配置欄`以外を変更する。
