# Stage対話の概念地図 — ギズモと2台カメラ

作成日: 2026-08-18

状態: **観察**(決定を含まない。M-2実測後に裁定文書化する際の設計材料)

対象: iced移行M-2(Stage島)以降で必要になるStage上の直接操作(ギズモ・選択・書き出し枠)の概念を、どの層が所有するかの整理。関連: [Rerun合成基盤裁定](2026-08-18-rerun-as-composition-foundation.md)、[Rerun埋め込み前例調査](2026-08-18-rerun-embedding-precedent-survey.md)、[Rerun fork seam台帳](2026-08-18-rerun-fork-seam-ledger.md)。

## 1. 前提: Rerunにauthoring概念は来ない

Rerunは読み取り専用ビューアの思想で作られており、authoring概念(ギズモ・選択の文書的意味・スナップ・書き出し枠・undo)は上流に来ない。これは教義と整合する: **Rerunは表示の座席であって権威ではない**。Rerunが持たない概念は文書側(UiIntent→Document)に置き、Stageはその投影に徹する。

## 2. 触れる窓(ギズモ)の3部品

1. **入力の所有権** — 入力ブリッジがRerunへ渡す前に握っている。掴み中はorbitへ流さない調停が**構造的に可能**(ブリッジは自前のコード)。
2. **stage空間のhit数学** — `canonical_drop_from_ndc`が既存資産。
3. **ギズモの絵とドラッグ意味論** — 新規。描画の置き場所は、`SpatialStage`(fork内の丸ごと追加ファイル=rebase安全圏)のoverlayか、島texture上へのiced重ねの2候補。

先行資産候補: **transform-gizmo** crate(旧egui-gizmo、renderer非依存・egui統合あり)。M-2/M-3時点で現物確認要。

## 3. 柵

**ギズモをRerun forkの機能として実装しない。** forkに足してよいのはseam(口)のみで、概念は足さない。[icedの外部調査](2026-08-18-iced-track-record-survey.md)で観測した「小forkは小のままでは済まない」の予防と同型。

## 4. 2台カメラ

- **視点カメラ** = camera seat(fork済み。`SpatialStage::set_camera`)。書き出しに影響しない。
- **書き出しカメラ** = compositionの寸法・枠。document概念であり、export経路(Rerunを通らない)が実行する。

これはBlenderのviewport camera vs scene camera+カメラ枠、AEのcomp viewportと同型 — **全DCCが解いた標準パターン**であり発明は不要。Stageに見える「枠」はoverlay線の投影にすぎない。

## 5. 回収機会

カメラ枠の設計は、**Preview=Export pixel同一性**(未測。[引き継ぎ文書](2026-08-18-session-handoff-ux-driver-seat-and-iced-migration.md)に記載)を審判する自然な機会である。枠の内側とexport出力が同じ画になるかを、枠実装のoracleとして流用できる。

## 6. 提案(未決)

M-2の受入条件に、入力調停の3状態(**ギズモ掴み中 / orbit中 / 素通し**)を入れる。
