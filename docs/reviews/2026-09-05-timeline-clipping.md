# Timeline clipping masks

利用者指示: マスクの主操作をエフェクト／手描きパス作成からTimelineへ移し、Photoshop／CLIP STUDIO PAINT式の「下のレイヤーでクリッピング」に統一する。

## 編集の意味

- レイヤーのクリッピングON/OFFをDocumentに保存する。参照先のIDをUIが固定せず、同じ親にある下位レイヤーから導く。
- 連続したクリッピングレイヤーは、最初の非クリッピングレイヤーを共通の下地にする。順序変更に追従し、親グループの境界を越えない。
- 下地は表示したまま。非表示／時間外／削除された下地から別の見える下地へ勝手に逃がさない。参照先がなければクリップ層は描画しない。
- 下地の半透明部分を二重に濃くしない。下地とクリップ層を分離して合成し、そのまとまりを作品へ重ねる。
- Undo、保存／読込、複製でクリッピング設定を保持する。

## 入口

Timelineのレイヤー行に↳ボタン。ONで点灯し、もう一度押すと解除する。非対応の場合は無効化と理由のtooltip。従来のCreateのMaskカードは取り除く。既存のパスマスク・明示マットの保存データと描画は削除しない。既存マットがある作品だけInspectorにLegacy matteを残す。

今回の対象はText・図形・画像・動画。同じグループ内のこれらのレイヤーにも適用可能。グループ全体／Null／ネイティブ3D素材、および明示マット付きの下地は、現行の描画範囲に合わせ入口を無効にする。未対応を透明化や暗黙flattenでごまかして対応済みとしない。Lottie JSON出力はクリッピングを黙って捨てずunsupportedとして報告する。プロジェクト保存と動画出力とは別の制限である。

## 使う資産

- Document SetAttrs／既存Undo／既存レイヤー順序。
- re_renderer上の既存合成経路、借用済みVello blendのsource-atop。
- [Photoshop clipping masks](https://helpx.adobe.com/ca/photoshop/using/revealing-layers-clipping-masks.html)
- [CSP Clip to Layer Below](https://support.clip-studio.com/en-us/faq/articles/20200108)
- [W3C source-atop](https://www.w3.org/TR/compositing-1/#porterduffcompositingoperators_srcatop)

## 反復と保全

変更前の未コミット資産をDocuments/Motolii-recovery/2026-09-05-before-clipping/files.tar.gzへ退避。保存済み作品の内容は変更せずapp72271/server71391を終了してから保持型を追加。型・描画統合後の同profile baselineは1回、以降warmへ戻す。対象のDocument試験、Timeline操作試験、合成画素試験と従来matte回帰に絞る。全UI試験を反復のたびには実行しない。

## 検収

Document関係2件、旧データ既定値1件、Timeline操作1件、既存matteを含む画素4件が成功。土台α128の維持と無関係な青い背景が切抜き元にならないことも確認。全UI試験は実行していない。

実窓app75460/server74559でRectangleを下のTextへクリッピングし、Rectangleの範囲外部分が消え、Textが表示されたままであることを確認。1回のUndoで元の矩形へ戻した。CreateはText／Rectangle／Bezierの3種類でMaskカードなし。試験変更はUndo済み、保存ファイルは上書きしていない。

追加制限: クリップ上層のAddは借用mix処理に対応がないため、現在はクリッピング開始を無効化。クリップ中に後からAddへ変更すると描画側は明示エラーを返す。通常と既存の他のmixモードを扱う。グループ全体・native素材・Lottie JSON出力と合わせ、これを完全なPhotoshop互換とは称さない。
