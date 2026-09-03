# 2026-09-03 コンセプトへの異議(コンセプトデザイナー: Ableton / Figma の人)

利用者の依頼で、Motolii のデザインコンセプトそのものに非を唱えるペルソナを回した。**裁定は利用者の専権**。ここは論点と見積もりの記録で、実装はしていない。

## 主張の骨子
1. 「面 8 枚 + menu 6 本 + Inspector の縦 20 行」は AE(1993 年・浮遊パネル)の器の相続。Media / Effects / Create / Colors は面でなく 1 枚の Browser と 4 本の rail(dock.rs:38-41 は way / home / window が同一、browser.rs:562 の 1 関数の分岐)。
2. 代償: 押した所と応える所が窓の対角(Inspector の COLOR 行 → 左端の Colors、BLEND 行 → 右下の Desk、Effects は左で足して右下端で触る)。「選ぶ」器が 5 種(dock の tab / menu / desk の drawer chip / browser の rail / inspector の choice)。二重: Desk と Inspector、Colors 面と COLOR 行、層の一覧が 3 箇所、menu の sheet 3 枚、View menu 16 行、DeskState の 3 状態。
3. 裁定「机は誰にも呼ばれない」は 1 日で破れた(ask_panel を inspector.rs と app.rs が呼ぶ)— 面を分けた時点で面同士は呼び合う。Ableton / Figma は詳細面が 1 枚だから呼ぶ必要が無い。
4. 代案 A(Ableton): Arrangement = Stage + Timeline 一体、Session = 素材と歌詞 clip の棚(印 = scene 行)、下に device chain(Transform / Text / Blend / Parent / Matte / fx が横 1 列)。
5. 代案 B(Figma)【推奨】: キャンバス 1 枚 + 右 1 本(Detail = Inspector + Desk + 色 popover)+ 左 1 本(層と印)+ 下に Timeline の帯、menu 6 本は ⌘K のコマンドパレット(Intent 30 種と SemanticControl がそのまま行になる。⌘K は ⌘⇧D と重複しているので空く)。
6. 残す芽: 印が全 track を貫く、1 レイヤ歌詞、Blend の札、音の指紋、Composition の sheet、「焦点に付いて回る」(付いて回る先を別の面にしない)。
7. 見積もり: 今の器で直す(1 日、壊れる試験 2〜4)/ 面を減らす 8→4(3〜4 日、12〜16)/ 器を変える(1〜2 週、27 本 = 96 本中 28% が「面をどこへ置くか」の試験、dock.rs 619 + dock_hit.rs 47 + desk の器 ≒200 + panels の一部 ≒100 + View menu ≒60 → 約 1,000 行が消え、意味は 1 行も消えない)。
8. 裁定への異議 3 つ: 「机 Desk」(詳細は 1 枚へ)、「色の輪は Browser の Colors に常設」(掴んだ swatch の隣の popover へ、Colors は「使われた色」の rail に降格)、「dock は dioxus-workbench」(配置は固定 4 面、dock / detach / 別窓 / layout.json を廃止)。
9. 方法論への注: 「ペルソナの最大公約数」は AE と NLE の器に収束する。器を変えるのは公約数の外の 1 人。

## 判断の材料(実装側からの補足)
- 今日の直しの多くは「面が分かれている」前提の配線(ask_panel・DeskState・roving tab・tab の ←→)で、器を変えれば不要になる物と、器に依らず残る物(Intent・Field・Selection・Blend の式・Composition / Export の中身・keys.rs の規則)がはっきり分かれる。
- 器を変えるなら先に「面を減らす(8→4)」で試験の壊れ方を測るのが安い。
