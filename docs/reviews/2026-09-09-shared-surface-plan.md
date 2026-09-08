# 2D・3Dの表面契約を統一する

利用者承認: 2026-09-09、re_renderer世界の統一を自走。UI・編集モデルは別の開発者の作業と分離する。
定規は[反射・屈折調査](2026-09-09-glass-raster-research.md)、既存のre_renderer mesh hookとrectangleの画像・alpha処理。新しい描画エンジンは作らない。

## 範囲と順序

1. forkのsurface入力を共通WGSLへ抽出。同じprogramがmeshとrectangleの表面を描けるようにする。既定の板は従来の画像表示、既定のmeshは従来の照明を保つ。
2. 板は位置・UV・平面法線・coverage・見かけの厚みを供給。色のdecode/filter、premultiplied alpha、clip、深度・合成の既存処理を維持。fieldによる頂点変形はこの表面契約の統一に混ぜない。
3. Motolii renderのModel限定を解消し、板にも同じcatalog/program/paramsを渡す。backdropは表面効果のある板にも届かせる。mix合成経路も含める。
4. GPUの既存Glass oracleを板へ展開し、透過・鏡・透明穴・既存mesh・合成の回帰を確認。実窓で同じ効果が画像・文字・図形・meshに届くか確認する。
5. 実際の検証結果と残件をこの文書へ記録。forkは参照可能なcommitにし、root Cargoの参照更新は直前に他の変更を再確認する。

## 検収境界

物体同士の反射用撮影、擬似法線/丸み、複数回反射は次の仕事。今回はGlassを名指しせず、新しいsurface効果が2D/3D両方に届く共通入口を完成させる。
シェーダーcompile成功と実画像/実窓の確認は区別する。既存native/UI変更は編集しない。

## 結果

作業中。
