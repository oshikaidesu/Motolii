# AviUtl2 のスクリプトを同梱して全部動かした（10/26）

- 同梱：`filament/aviutl_scripts/<作者>__<パッケージ>/` に **193 パッケージ**（手を入れずに。どれもライセンスの文書つき：MIT 126・CC0 26・Apache 1 ほか。ファイル 1 本だけの配布は、作者がカタログ aviutl2-catalog-data に申告したライセンス全文を LICENSE として添えた）。
  - カタログで配ってよい 182 のうち 180 を入れた。入らない 2 は BOOTH（ログインが要る）、1 は配布ファイルが見つからない（IDgen_Su）。残りはカタログの外で見つけた物。
  - 配ってよいと書いていない 67（Google Drive・BOOTH が多い）は同梱しない。作者の配布元から取ってくる形にする。
- 全部を写真に 1 本ずつ掛けた：**594 本のうち 326 本（55%）が最後まで動いた**。パッケージでは 184 のうち **73 が全部の本が動いた**。
- 止まる理由（本数・パッケージ数）：
  | 足りない物 | 本 | 包 |
  |---|---|---|
  | `obj.drawpoly`（3D） | 23 | 17 |
  | `obj.module`（Windows の DLL） | 20 | 13 |
  | `obj.load figure`（四角以外の図形） | 17 | 6 |
  | 本体の効果を名前で（`obj.effect` ぼかし・ノイズ・単色化・オフスクリーン描画…） | 約 25 | 約 20 |
  | 共通の .lua の中で止まる（Path_S・KSA・Region_S・HexScript の部品） | 約 45 | 5 |
  | `obj.rand`・`obj.rand1` | 13 | 11 |
  | `obj.getpixeldata`・`pixeloption` | 8 | 8 |
  | `obj.load text` | 8 | 6 |
  | `obj.computeshader` | 5 | 5 |
  | シェーダーが変換できない | 10 | 8 |
- 次に足す順は `converter_plan.md` のとおり：効果の名前の当て先 → 3D（drawpoly）→ rand・図形 → 文字。
- 10/26 その後：`obj.rand`・`obj.rand1` を足した（YMM4-LuaScript の `aviutl_rand` をそのまま）。同じ判定で数え直して 594 本中 354 → **365 本**。増えたのは HexScript のフィルター 9 本、AutoLyricAnimation_K、BlockNoise、RandomObject。止まるようになった 1 本（Resize_S のボックスリサイズσ）は乱数ではなく、箱の余白が 128 画素になって別のスクリプトを名前で呼ぶ道に入ったため（名前で呼ぶのは未対応）。止まる理由の上位は `obj.module` 42、`obj.drawpoly` 28、`obj.effect`（名前なし）12、図形「円」12、`obj.interpolation` 12、`obj.getpixeldata` 11。
