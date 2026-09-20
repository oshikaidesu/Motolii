# Chromatic Workshop — 実画面への反映と公開カラーテーマ

2026-09-20。状態: **決定**。

利用者はこの会話のBefore/After比較でAfterを選び、実装の継続と「第三者がカラーテーマを自作できる程度に開く」ことを依頼した。生成画像の細部の配置差は採用仕様にしない。

## 反映した造形

- 暗いニュートラルな面、明るい文字、鮮やかなレイヤー色、シアンの選択タブを既定の `Chromatic Workshop` にした。
- 同梱Interを共通の文字スタイルへ通し、本文10・補助9・最小8、通常weight 500を基準にした。Createの文字記号は28の枠、立体の線は2で描く。
- カード外形、タイル倍率、3列×6段、パネル配置、Timelineの行高を拡大する変更は入れていない。既存の未コミット作業を保全し、その上へ実装した。
- 原図形・作品の色はDocumentの値を維持する。カラーテーマが変えるのは編集画面の表示色。

利用者の追加調整(同日): ベースの面・文字・Timelineの地を無彩色に統一。Timelineのレイヤー色だけに約30%の中立グレーを重ね、彩度と明るさを抑えた。`colors.timelineWash` で第三者テーマから調整できる。Inspector等の識別色と作品色は変更しない。

利用者の文字調整(同日): 箱のシルエットを保ち、本文12→11・補助11→10・最小10→9へ縮小。文字用定義を借りていたアイコン・Depthの位置・部品の最小幅は従来寸法を維持した。

利用者の追調整(同日): さらに余裕を強く出すため、文字だけをもう一段小さくし、本文10・補助9・最小8を採用。見出し13、箱とアイコンの寸法は維持。

## テーマを作る入口

[作成ガイド](../../motolii/ui/themes/README.md)、[JSONスキーマ](../../motolii/ui/themes/theme.schema.json)、[サンプルVelvet](../../motolii/ui/themes/velvet.json)。

`Settings → Color theme` から `Load JSON… / Reload / Copy JSON / Default` を使う。JSONはversion 1、`#RRGGBB / #RRGGBBAA`（alpha末尾）。48のUI色、15の描画色、レイヤー識別色、7つのCollection色を変更できる。省略した色は既定値を使う。

読み込み時に未知のキー、形式違い、空のパレットを拒否する。直前の有効なテーマは維持する。検証済みの内容と元ファイルのパスを既存の設定に保存し、起動時は保存内容を復元する。ファイル変更はReloadで反映し、定期pollは追加しない。

テーマ変更は既存のウィンドウ通知へ載せる。受信側はローカルな机の設定とDocumentを保ち、同じ通知の反復では更新しない。作品の編集命令やUndoには入れない。

## 実装の根拠

[Flutter ThemeExtension](https://api.flutter.dev/flutter/material/ThemeExtension-class.html) と [Theme](https://docs.flutter.dev/cookbook/design/themes) を利用。配色は継承されたテーマから読み、CustomPainterには色のスナップショットを渡して `shouldRepaint` に含める。従来の固定定数から切り替えるため、利用箇所を横断して変更した。

Materialのボタン・余白・アニメーションを導入する変更ではない。既存のimport検査には `show Theme, ThemeData, ThemeExtension, ColorScheme` の限定使用のみを許可し、一般のMaterial部品は従来どおり拒否する。許可と拒否の双方を検査した。

## 実窓の証拠

- [最終承認後の実画面（無彩色ベース・抑えたTL色・文字10/9/8）](evidence/2026-09-20-chromatic-theme/accepted.jpg)
- [Before](evidence/2026-09-20-chromatic-theme/before.jpg)
- [実装後の既定テーマ](evidence/2026-09-20-chromatic-theme/implemented.jpg)
- [外部Velvetテーマを読み込み、UI再起動後に復元](evidence/2026-09-20-chromatic-theme/velvet-restored.jpg)

初回実装比較は同じ `stagger.rrd`、Frame 135、Dot選択で確認。最終承認後の画像はFrame 49・Media 100%の利用中の状態。外部JSONの読み込みと既定への復帰は実操作で確認した。文字下書きの保持、不正ファイルを拒否して現行テーマを保つこと、macOSの設定通信形式からの復元、別窓通知のDocument非変更は自動検査で確認する。

最初のhot reloadは型変更のためFlutterに拒否され、UIのhot restartを実施した。Documentへの再接続後に作品・Frame・選択の保持を確認。その後の調整はhot reloadで反映した。

## 検証と残る制約

配色・文字の追調整後: Browserのタイル、操作部品の寸法、全パネルの文字収まり・コントラストの対象テスト16件と文書整合検査が通過。

テーマ実装時: `flutter analyze` は問題なし。全UIテストは184件通過・下記の既存3件失敗。テーマと窓配置の対象テスト7件、import規則テスト4件、`raw_dimension / raw_color / material_import` 検査、文書整合検査は通過。

全UIテストでは既存の `panel_layout_cost_test` 3件が失敗する。作業開始時のソースを保全した別の検証コピーでも同じ3件を再現し、全体layout回数はFonts 175・Inspector 440・Blend 362、選択変更165で一致した。これらの閾値は変更していない。

JSONの検証は形式の検証であり、任意の第三者配色のコントラストや美的品質を保証しない。macOSネイティブのタイトルバーとシステムのファイル選択画面はOSの外観に従う。
