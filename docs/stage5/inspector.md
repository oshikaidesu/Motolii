# Inspector — 2026-09-09 adoption

利用者の「Testを正本にする」「即時フィードバック」「サイズ連動を直す」に基づき、Testの部品版を `motolii/ui/lib/panels/inspector.dart` の唯一のInspectorへ移した。旧表版とTestタブは撤去。プロパティの表示分類だけを `inspector_property_style.dart` に置く。保存DockのTestはInspectorへ読み替える。

## 操作の契約と修正

- Scaleのリンクは両軸の比率を保持する。2:3をX=4にすると4:6。リンクの切替自体は作品を変更しない。解除後は触った軸のみを書く。ゼロの基準軸は両軸を入力値で立ち上げる。
- 数値・ダイヤル・XYパッドは同じ最新値キューを使う。針・点・数値は入力時に表示し、処理中の本体previewの後ろには最新値だけを保持する。確定は最後のpreviewを待ち、一回のcommitへ進む。レンダラーの一回の描画時間そのものを短縮したという意味ではない。
- ダイヤル・パッドのpointer cancel、Escape、窓のフォーカス喪失・非アクティブ化・破棄はcancelへ進み、未送信の値を捨てる。最後のpreviewより先にcommitを送らず、確定中の表示も古いsnapshotへ戻さない。
- 数値確定後は数値部品へフォーカスを戻す。P/S/R/Tのプロパティ表示要求を接続し、要求だけの時にもFlutterのフレームを予約する。
- 汎用の文字・RGBAプロパティを数値欄へ流さず編集欄へ投影する。選択肢も数値と同じpreview/commit経路を使う。色・文字の編集可否はlayer lockを尊重する。

## 借りた基準

既存の `EditorNumericField` のlatest-pending-value方式、旧InspectorのfocusProperty/ensureVisible、Documentのpreview/commit/cancel契約が基準。Flutter SDKの `widgets/editable_text.dart` の編集完了とfocus移動、`gestures/monodrag.dart` のaccepted drag終了、既存EaseのWidgetsBindingObserverによる中断処理を確認して写した。独立した作品状態・Undo履歴はFlutterに増やさない。

## 検証

- 対象6ファイルのFlutter回帰テスト14件成功。連動/解除、RGBA alpha、文字、保存Dock移行、遅いpreview中の間引き、cancel、パッド破棄、既存数値drag、Desk連携、operation契約を含む。
- `flutter analyze`、`scripts/check-stage5.py` 成功。
- 実窓でInspectorが一つになったことを確認。torusのScale 245→260でX/Yとも260、Edit→Undo一回で245/245へ戻った。ダイヤルでRotation -18→142.5、Edit→Undo一回で-18へ戻った。検証で変えた値は戻した。
- 全体テストの実行時は52件中51件成功。並行変更中のEaseで `desk_workspace_test.dart` の旧 `Save preset` 文字検索が失敗。raw_dimension検査の指摘24件もEase側のみだった。全体緑という主張には使わない。
- Cmd+Zの実窓検収は未確定。ウィンドウの並行操作もあり、今回の確実なUndo証拠はアプリのEditメニュー。入力遅延のms計測はしていない。

## 色・書体・文字揃えの追加

- 利用者修正: Fill/Strokeのスウォッチは対象slotを渡して既存のColorsパネルを表示する。ポップアップは使わない。HEX欄も引き続き使える。
- Textカードの書体名からBrowser側のFonts棚を開く。シェーパーの共有fontdbに存在するfamilyを検索し、選択中のロックされていないTextレイヤーへ適用する。本文・内容キー・style runの構造を保ち、全styleのfamilyを一回のSetTextDocumentで変更する。
- TextのAlignmentは既存text_justifyプロパティへ接続。初回にtrackのない列挙値は、既存プロパティ行の宣言をwire decodeの型の基準にも使う。
- 新規検証: font_browser_testの2件(検索/選択/対象なし、Colors再利用とslot指定)、既存回帰を含むFlutter13件、本体font_testsの1件(本文保持・Undo・未登録書体・lock拒否)が成功。native build成功。
- 書体棚は現段階ではfamily名の選択。選択本文を各書体で描く仮想化サンプル、drag中の仮見せとdrop、文字範囲への適用、太さ・斜体・可変軸は未実装。過去のフォント棚裁定の全工程が完了したとは扱わない。
- 実窓ではTextカードに書体・Alignment・Fill/Strokeの入口が出て、書体名からFonts棚を開きGeorgiaを検索できることを確認。適用クリックはComputer Useが同じ窓の変更を繰り返し検出して拒否したため、書体・色・揃えの実窓での結果検収は保留。作業窓を使える時間を利用者へ問い合わせ中。native再build前の作品はFile→Saveで21:10:03に保存した。
- 最終flutter analyze、raw_dimension、diff whitespace検査は成功。nativeのfont_tests再実行も成功。
