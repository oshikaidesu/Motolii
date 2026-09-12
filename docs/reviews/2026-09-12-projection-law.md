# 2D / 2.5D / 3D の法 — 意図で選ぶ 3 つの札

2026-09-12 利用者裁定。きっかけは「効果を掛けた図形が 2.5D の置き場に残り、Title(z=609)より楕円が手前に来た」報告と、
「2D の層は z を持たず timeline 順で整理される、背景にめり込む処理は無い。2D↔2.5D で位置が変わるのは不快。3D が選べないのも不快」。
利用者の指示: **ユーザーは必ず意図で札を選ぶ。先にペルソナを考える。** ただし描画は re_renderer の上に居る法を守る。

## 意図 → 札

| 押す人 | 意図 | 札 | 先例 |
|---|---|---|---|
| テロップ・UI・版面(Figma から来た人、歌詞デザイナー) | 画面に貼る。カメラが動いても動かない。重なりは timeline の上下だけ | 2D | Figma、Unity Canvas Screen Space - Overlay、AE の 2D 層 |
| 平らな絵で奥行きの芝居(Live2D / Spine、視差 MV) | 平らのままカメラに正対、奥に置いて視差。手前・奥は z | 2.5D | AE の 3D 層(回さない使い方)、Unity Screen Space - Camera、Blender Grease Pencil |
| 空間に物を置く(C4D / Blender) | 回せる、刺さる、影も反射も受ける | 3D | C4D、Blender、Unity World Space |

## 法

1. **2D は世界に居ない。** z を持たず、深度に参加せず、3D と刺さらない。timeline の順が全て。AE と同じく 2D 層は 3D 世界の仕切り(仕切りの間の 3D 同士だけ深度で並ぶ)。
2. **2.5D 同士は z で並ぶ。** 同じ z なら積み順(同一面)。Title が楕円の奥に行くのは両方 2.5D だからで、作品側の z の問題。
3. **札を変えても中心は画面の同じ場所に留まる、双方向。** 面の向きは新しい札の意味に従う(回転・scale は書き換えない)。動かすのは位置だけなので animate された層も切り替えられる。基準は今の comp カメラ。AE はここで跳ぶので引き継がない。
4. **3D はどの素材でも選べる。**

## 実装

- 法 1: [sequential.rs](../../motolii/crates/motolii-render/src/compositor/sequential.rs) の run の切り方。2D と非 2D を同じ run に入れない。連続する 2D は 1 つの run(同じ面・積み順)、3D 群は自分たちだけの run で深度を解き、run 同士は積み順で over。深度 buffer は run ごと。描くのはどちらも re_renderer の同じ ViewBuilder(Motolii 側の描き分けは無し)。2D の層は反射・影の世界にも入らない(世界に居ないので一貫)。
- 法 3: [projection.rs](../../motolii/crates/motolii-doc/src/store/document/projection.rs) `projection_compensation`。以前は面(回転・scale)まで保とうとして、animate された層を「Cannot change coordinate systems for animated transform position; keep its parent」で拒んでいた — これが「3D が選べない」の正体(Torus の位置に key があった)。中心だけを保つ形にし、`move_translation_values` で全 key を同じ量ずらす。2.5D↔3D は中心が同じなので触らない。
- 法 4: 拒む規則は元々無く、法 3 の拒否が原因だった。

## 審判

- doc `projection_switch_tests`(3 件): 回した親の下・動かしたカメラで 5 回切り替えても中心は 0.05 px 以内、回転値は不変。animate された位置 + 傾きの層も切り替わり key は 2 つのまま。
- render `projection_contract`(3 件): 下の段の手前 3D(z=-200)の上に 2D を置くと 2D が全て見える(修正前 0 画素)、同じ配置で 2.5D なら深度で奥。2D 同士は z でなく積み順。既定カメラでは 3 つの札が同じ場所(重心 0.5 px 以内、面積同一)。
- 実測の元: 動かしたカメラ(orbit -25/60、寄せ 0.5)で同じ Position の板は 2D 80.5 / 2.5D 57.0 / 3D 41.2 に映っていた(跳びはカメラが既定でない時だけ)。

## 実例での分解(VISUALS OF OFFICE 39)

小さな版面の文字群 = 2D(法 1 で常に手前、刺さらない)。大文字 VISUALS/OF/OFFICE = 2.5D(梁が「S U」の前を通り「L S」の後ろへ回るのは、傾いた梁との画素ごとの深度)。緑の梁・黒い塊・煙 = 3D。すりガラスの円 = 2.5D + **下を読む法(未実装)**。煙の光 = 溢れの法(済)。法の作り直しは要らず、足りないのは下を読む法だけ。限界: 2.5D の「後ろ」は深度だが backdrop は積み順なので、深度で後ろに居る物と積み順で下に居る物がずれる場面がある。
