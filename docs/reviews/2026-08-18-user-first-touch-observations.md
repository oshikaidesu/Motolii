# 利用者の初回タッチ観察 — Browser ダブルクリックと画像の入口

日付: 2026-08-18
状態: **観察**(利用者の実機報告+機械での原因特定)

## 利用者の報告(原文の趣旨)

Browser から画像をダブルクリックで配置 → Rerun ビューには透明のパネルのみ描画。
Timeline にも Inspector にも表示されない。

## 原因(コード・CLI 実測で特定)

1. **Browser のダブルクリックは処理が存在しない**(browser_panel に handler 皆無。
   ヒットする `double_clicked` はパネル分割線のリセットのみ)。帯にも何も出ない —
   2026-08-13 裁定の「無反応ゼロ」と Q0(触れそうで触れない)に違反
2. **画像は admission の拡張子リストで拒否される**: CLI 実測で
   `probe failed: unsupported media file extension for admission: still.png`。
   GUI ドロップでも同じ理由で skip される(こちらは帯に理由が出る)。
   UX 台本(ux-check-first-ten-minutes P1/P3)の「動画/画像をドロップ」は
   画像について**実装と乖離**していた
3. Timeline / Inspector が空なのは(1)の帰結(何も起きていないため表示は正しい)
4. 「透明のパネル」= 空コンポの正対プレート(同日の Stage 正対化で、空の
   composition 面が正面に見えるようになった絵)

## 繋がり3分類(toolkit 再入場トリガーの記帳)

- (1) は「繋がっていない」ではなく**操作が存在しない**(dead interaction)。
  intent 経路の断絶ではないため iced トリガーには数えないが、無反応ゼロ違反として
  修正対象
- (2) は失敗が帯と CLI に理由つきで出る=可視・replay 可能。feature 欠落であり
  断絶ではない
- **「繋がっていない」件数: 引き続き 0**

## 修正(同日レーン発注)

- レーンA: 画像の入口 — admission の拡張子拡張(png/jpg/webp 等)+ `image/*` の
  place 意味(静止画=尺不定→comp 残り、既存の clamp 意味と整合)+ export まで
  貫通する E2E(赤い PNG を置いて出力画素が赤)
- レーンB: Browser ダブルクリック=admit+place at playhead(ドロップと同じ経路へ
  合流)。成立・失敗とも帯に言う。配置後に Timeline へ現れることをテストで固定
