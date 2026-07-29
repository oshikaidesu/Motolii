# M5 provider横断fixture fragment map v5

作成日: 2026-07-29

状態: **完了／P2D-RCC5 Grok ACCEPT（P0/P1/P2=0）**

変更許可: 本fileの`配置欄`だけ

単一動詞: **配置する**

## 入力と固定fragment

入力は次のACCEPT済み三provider観察だけである。

- [Godot transparency観察転記 v3](2026-07-29-m5-godot-observation-transcription-v3.md)
- [Bevy観察fragment map v4](2026-07-29-m5-bevy-observation-map-v4.md)
- [Unreal観察fragment map v4](2026-07-29-m5-unreal-observation-map-v4.md)

Bevyの`B-*`とUnrealの`U-*`は各入力fileの固定fragment IDをそのまま参照する。GodotのACCEPT済み
5行を主担当Codexが次のIDへ固定する。leafは本文、ID、追加／削除を変更しない。

- `G-O1`: transparentはopaque後に描画され、object位置基準のback-to-front sortには重なり誤順序が残る。
- `G-O2`: Godot 4.6はOITを提供せず、alpha scissor、depth pre-pass、alpha hash等を用途別回避策とする。
- `G-O3`: `ALPHA`を書けばtransparent pipelineへ入り、sorting問題が生じ得る。
- `G-O4`: transparent materialはscreen/depth textureへ現れず、screen-space reflection／refractionへ制限が出る。
- `G-NO`: 該当する固定観察なし。

横断coverageは、provider先例がMotolii fixtureそのものを証明しない範囲を次のIDで固定する。

- `X-F1-GAP`: phase分離の部分観察だけで、同じworld/cameraのopaque Z交差反転は証明しない。
- `X-F2-PARTIAL`: alpha種別の分離と回避策は観察できるが、cutoutのdepth参加とsoft alpha黙示格上げ禁止は証明しない。
- `X-F3-PARTIAL`: soft alphaの順序failureとOIT追加責任は観察できるが、Motoliiの非対応診断契約は証明しない。
- `X-F4-PARTIAL`: scene-color／refractionの順序と制約は観察できるが、入力snapshot、範囲、failure宣言、
  Preview／Export同一は証明しない。
- `X-F5-GAP`: unknown contribution／capability不足の型付き拒否と既存2D不変を証明する固定観察はない。
- `X-F6-GAP`: contribution未使用時のpixel不変とPreview／Export同一を証明する固定観察はない。

## 主担当固定配置

- F1 opaque Z交差反転: `B-O2 | G-NO | U-NO | X-F1-GAP`
- F2 cutout depth／soft alpha非格上げ: `B-O2 | G-O2 | U-O1 U-O2 | X-F2-PARTIAL`
- F3 soft alpha順序／非対応診断: `B-O2 B-O4 | G-O1 G-O2 G-O3 | U-O1 U-O3 | X-F3-PARTIAL`
- F4 scene-color／refraction宣言: `B-O3 | G-O4 | U-O2 U-O4 | X-F4-PARTIAL`
- F5 unknown／capability拒否: `B-NO | G-NO | U-NO | X-F5-GAP`
- F6 未使用pixel不変／Preview-Export同一: `B-NO | G-NO | U-NO | X-F6-GAP`

`|`はmatrixの列境界であり、配置する文字列には含めない。

## 固定配置matrix

各`配置欄`には上記IDだけを空白区切りで置く。provider列は各providerの固定観察だけ、coverage列は
対応する`X-F*`一つだけを許す。ID本文の複製、言い換え、方式採択をしない。

| Motolii fixture候補 | Bevy観察 | Godot観察 | Unreal観察 | 横断coverage |
|---|---|---|---|---|
| F1 opaque Z交差反転 | B-O2 | G-NO | U-NO | X-F1-GAP |
| F2 cutout depth／soft alpha非格上げ | B-O2 | G-O2 | U-O1 U-O2 | X-F2-PARTIAL |
| F3 soft alpha順序／非対応診断 | B-O2 B-O4 | G-O1 G-O2 G-O3 | U-O1 U-O3 | X-F3-PARTIAL |
| F4 scene-color／refraction宣言 | B-O3 | G-O4 | U-O2 U-O4 | X-F4-PARTIAL |
| F5 unknown／capability拒否 | B-NO | G-NO | U-NO | X-F5-GAP |
| F6 未使用pixel不変／Preview-Export同一 | B-NO | G-NO | U-NO | X-F6-GAP |

## 非証明と非目標

- providerの方式、型、phase名、material、render graph、thresholdをMotolii契約へ転記しない。
- `X-F*`は不足の記録であり、公開API、Document、plugin契約、fixture期待値、方式の採択ではない。
- Rerun、現行code fact、First Vism、実装、製品packageを入力へ混ぜない。

## STOP

- 固定fragmentだけでは配置できず、新しい文、ID、資料、解釈が必要になる。
- provider観察からMotoliiの要求、方式、公開契約、永続意味を決め始める。
- 本fileの`配置欄`以外、network、別version、旧REJECT差分が必要になる。
