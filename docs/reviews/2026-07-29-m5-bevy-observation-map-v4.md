# M5 Bevy観察fragment map v4

作成日: 2026-07-29

状態: **登録済み／P2D-RCC4-BEVY未実行**

変更許可: 本fileの`配置欄`だけ

単一動詞: **配置する**

## 入力と固定fragment

入力は[Bevy 0.19 capsule](2026-07-29-m5-capsule-bevy-render-phases.md)だけである。以下は主担当Codexが
capsuleの記載を意味変更せずID化した固定fragmentで、leafは本文、ID、追加／削除を変更しない。

- `B-O1`: render phaseはqueue、prepare、sort、drawを分けるmodular abstractionと説明される。
- `B-O2`: opaque／alpha-maskはbinned、transparentはback-to-frontを要するsorted phaseとして分かれる。
- `B-O3`: phase分離理由にはsorting／batching差と、前phaseのrendered textureを読むscreen-space effectが挙がる。
- `B-O4`: core pipelineにはprepassとOIT moduleが別責任として存在する。
- `B-NO`: capsuleに該当する固定観察なし。
- `B-SRC`: Bevy 0.19.0公式API docs、取得日2026-07-29、capsule記載の3 URL。
- `B-NP`: Bevyの型、schedule、render graph、OIT方式、phase名をMotolii契約へ転記する根拠にしない。
- `B-LIM`: FROZEN / DELETE-LATER / 製品import禁止。

## 固定配置matrix

各`配置欄`には上記IDだけを空白区切りで置く。観察は`B-O1`〜`B-O4`または`B-NO`、
他3列はそれぞれ`B-SRC`、`B-NP`、`B-LIM`だけを許す。ID本文の複製や言い換えをしない。

| 観測項目 | 観察fragment | source fragment | 非証明fragment | 持込禁止fragment |
|---|---|---|---|---|
| phase admission / ordering | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |
| depth / opaque / cutout / soft alpha | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |
| transparent交差 / sorting / OIT追加位置 | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |
| scene-color / refraction / resource lifetime | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |
| capability不足 / unsupported / cyclic read | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |

## STOP

- 固定fragmentだけでは配置できず、新しい文、ID、資料、解釈が必要になる。
- provider横断比較、Motolii fixture対応、方式採択、公開契約を始める。
- 本fileの`配置欄`以外を変更する。
