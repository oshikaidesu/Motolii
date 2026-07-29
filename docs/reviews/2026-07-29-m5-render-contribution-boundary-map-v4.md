# M5 Render Contribution境界fragment map v4

作成日: 2026-07-29

状態: **登録済み／P2D-RCA4未実行**

変更許可: 本fileの`配置欄`だけ

単一動詞: **配置する**

## 入力と固定fragment

入力は[Render Contribution証拠Wave親task](2026-07-29-m5-render-contribution-evidence-wave.md)
§2〜§3、§5と、同節が固定するMotolii authorityだけである。以下は主担当Codexがauthorityの
既決／停止線を意味変更せずID化したfragmentで、leafは本文、ID、追加／削除を変更しない。

- `C-Q1-REQ`: 空間表現は型付き要求を宣言する。
- `C-Q1-CONTRIB`: Hostは要求を受理した後にrender contributionを集める。
- `C-Q1-NEG`: 型付き要求とrender contributionを同じ所有物または万能callbackにしない。
- `C-Q2-NOOWN`: contributionはworld、camera、Observation、transform、layer order、Quality、FrameDescを所有しない。
- `C-Q2-HOST`: contributionはHostのadmission、phase resolve、resource budget、failureへ従う。
- `C-Q2-NEG`: shared depthへの参加要求そのものを禁止せず、受理と共有資源の決定をHostから奪うことを拒否する。
- `C-Q3-CAPS`: opaque、cutout、soft alpha、scene-color／refractionを能力、順序、alpha保証、fallback可否、診断として分ける。
- `C-Q3-NEG`: opaque、cutout、soft alpha、scene-color／refractionを一つの万能draw callbackへ潰さない。
- `C-Q3-OPEN`: copy、subpass、resource lifetime、同期、OIT方式はこの比較で決めない。
- `C-Q4-ADD`: 新能力は追加的に導入し、既存contributionの意味を再解釈しない。
- `C-Q4-UNKNOWN`: 未知能力を黙示fallbackせず、型付き拒否へ残す。
- `C-Q5-FIRST`: First Vismはfirst-party専用口なしで同じ境界を通るconformance fixture上の最初の表現である。
- `C-Q5-NEG`: First Vismの製品機能、package形式、具体表現、販売／配布、UIを決めない。
- `C-Q6-UNKNOWN`: 第二の未知表現もHost enum、具体provider ID、raw JSON、private type走査を要求しない。
- `C-Q6-NEG`: 未知表現を既存contributionの意味変更またはopaque ID推測で受理しない。
- `C-CODE-GAP`: 現行codeにはprovider非依存Observation要求、shared depth admission、複数phase contribution、transparent／refraction capability交渉が未成立である。
- `C-NOT-API`: 公開trait、Rust名、phase enum、Document、plugin契約、wire形式をこのgrainで決めない。
- `C-RCI`: 最終採否と未決の処分は後続P2D-RCIへ戻す。

## 固定配置matrix

各`配置欄`には上記IDだけを空白区切りで置く。本文複製、言い換え、新ID、authority外の方式を足さない。

| ID | 論点 | 既決fragment | failure／負例fragment | code fact／未決fragment |
|---|---|---|---|---|
| Q1 | 要求とcontributionの分離 | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |
| Q2 | contributionが所有しないHost状態 | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |
| Q3 | opaque/cutout/soft alpha/scene-color-refraction | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |
| Q4 | 未知能力と追加的進化 | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |
| Q5 | First Vismのconformance fixture役割 | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |
| Q6 | 第二の未知表現 | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |

## STOP

- 固定fragmentだけでは配置できず、新しい文、ID、資料、解釈が必要になる。
- copy方式、公開境界、永続意味、First Vismの具体表現を決め始める。
- 本fileの`配置欄`以外を変更する。
