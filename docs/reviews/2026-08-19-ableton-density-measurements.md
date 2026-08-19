# Ableton Live 12 UI 密度の実測(+Blender/Resolve 併記)

日付: 2026-08-19
状態: **観察**(決定を含まない)
経緯: 利用者指摘「全体的にUIでかくねぇか」を数字にするため、UIトンマナ統一 campaign 第2波
([campaign 文書](2026-08-19-ui-tone-unification-campaign.md)追記節)の判断材料として実測。

## 手法と信頼度

Ableton 公式マニュアル埋め込みスクリーンショット PNG を画素解析。全画像の PNG メタデータが
144dpi(=2x)であることを確認し、image-px ÷ 2 = 論理 px と断定。`DemoArrangementL12.png` は
画面内に "1.00x"(Zoom Display 100%)が写っており倍率も確定。複数画像・複数要素で相互検証
(**高信頼度**)。Blender は公式開発者文書(中〜高)、Resolve は公式仕様非公開でスケール不明
画像のみ(**低信頼度・参考**)。`.ask` テーマファイルは色のみで寸法を含まない(実物確認)。

## 実測値(論理 px)

| 項目 | Ableton 12 | Blender | Motolii(第1波後) | 第2波レーンF指定 |
|---|---|---|---|---|
| transport/Control 帯 | **30** | — | 30 | 24 |
| Timeline トラック行高 | **20**(Arrangement) | 20(widget_unit) | 24(大34) | **20** |
| Session クリップ行 | 22 | — | — | — |
| Browser リスト行 | 19(サイドバー22-23) | — | — | — |
| Browser ツールバー | 29 | — | panel header 34 | (未変更) |
| 本文相当文字 | 推定 10〜13(芯7.5) | 通説11pt | BODY 13 / CAPTION 11 | (未変更) |
| デスクトップ最小クリック域 | — | — | WCAG 2.5.8 AA=24px / NSTableView 現行既定 24pt / 旧macOS toolbar 23px | — |

出典: ableton.com/en/manual の arrangement-view / session-view / working-with-the-browser /
live-concepts(画像は ableton-production.imgix.net)、wiki.blender.org の Preferences_and_Defaults、
w3.org WCAG 2.2 SC 2.5.8。

## 読み(判断材料。決定は利用者/supervisor)

1. transport 30 自体は Ableton と同値 — 「でかい」の主因は帯単体でなく
   **transport 30 + overview 22 + ruler 36 = 88px の積み上げ**(Ableton は Control Bar 30 が
   window 上部に1本あるだけで、Arrangement 側は薄い ruler)。第2波レーンFで 58px へ
2. 行高 24→20 は Ableton Arrangement 実測と**完全一致**、かつ WCAG 24px 最小の下でも
   2026-08-08 決定「最小20px」の範囲内
3. Motolii の CAPTION 11px が Ableton の主文字帯と一致し、**BODY 13px は Ableton に無い
   一段大きい階層**。第3波候補: BODY のクラス替え(11.5〜12)と panel header 34→30級。
   ただし**モック正本(inspector/browser-library.css)自体の改訂を伴う**ため、
   第2波の絵の利用者合否を待って裁定する
