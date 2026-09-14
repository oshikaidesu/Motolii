# ドッグフーディング — 動画ソフト + CSS の例

2026-09-14 観察。利用者「今のドックフーディングは動画ソフト + CSS の例としてまとめて忘れないように」「歌詞じゃなくてもいい、本質はそこじゃない」「渋谷とか東京の素材がいいね」。

## 本質

CSS を抽象化すると「各オブジェクトが自分の箱を宣言できるようになった」。Web では箱が関係を結ぶ相手は**親と兄弟の箱**だけ。動画ソフトには Web に無い相手が居る — **時間・カメラ・映像の中身・3D の空間**。箱がそれらと関係を結ぶ所が、Motolii にしか作れない絵になる。

同日の 3D スイス・ポスター(`swiss3d.js`)は「Web で何度も見た絵を 3D の物で描き直しただけ」で面白くなかった(利用者)。例は、Web が持たない相手と箱が関係を結ぶ物を選ぶ。

前提: [箱と流し込みの法](2026-09-14-layout-law.md)、[箱の奥行きの法](2026-09-14-depth-law.md)、[CSS の恩恵の台帳](2026-09-14-css-ledger.md)。

## 例(場面 → 何を試すか → 足りない物)

| # | 相手 | 場面(東京の素材) | 試す法・機構 | 足りない物 |
|---|---|---|---|---|
| 1 | **時間** | 交差点・電車・ネオン・自販機のカットを時間の Flex に並べる。1 カット抜くと残りが詰まり直して尺に収まる、カットの間(Gap)に鍵 | 層の入点・出点を時間の箱として並べる(先例: Final Cut の磁気タイムライン) | 時間の Flex の法そのもの(未提案) |
| 2 | **カメラ** | 東京の映像をはめた bento の壁を 3D に置き、カメラが「交差点のセル」→「電車のセル」へ箱から箱へ移る。回る物の画面上の箱に 2D の注釈が付いてくる | 「この箱を画面に収める」カメラ、投影を跨ぐ anchor positioning、画面上の大きさで切り替える container query | カメラが箱を狙う口、投影後の箱の口 |
| 3 | **映像の中身** | 渋谷スクランブル交差点の俯瞰。人の群れ・タクシーが Blob Track で箱になり、スイスの文字組み(「SHIBUYA 21:04」)が群れを避けて流れ直す、格子の線が群れに合わせてしなる | 解析の橋(Blob の塊 = 箱)を並べる法の子・避ける物に。shape-outside の相手が映像の中の物 | Blob の塊を並べる法へ渡す口、shape-outside |
| 3' | **映像の中身** | 昼の交差点でタクシー・バスに情報カードが付いてくる。台数が増えてもカード同士が重ならず並び直す | 同上 + Flex の子が映像の中で動く | 同上 |
| 4 | **3D の空間** | ロゴの立体が中央に落ちると、周りの小さな立体が場所を譲って外へ詰め直す(物理を使わず、時刻の純関数のまま) | 箱の奥行きの法、3D の箱同士の押し合い | 3D の押し合い(詰め直し)の法 |
| 5 | **箱そのもの** | bento のセルが壁から浮くと壁に柔らかい影、浮くほどぼける。カード同士も影を落とす | 宣言された箱から描く効果(box-shadow の 3D 版) | 箱を読む効果の口 |

## 素材の候補(2026-09-14)

Pexels(無料・クレジット不要。写っている人を悪く描かない、そのまま売らない等の条件あり):

- [Aerial View of Shibuya Crossing in Tokyo](https://www.pexels.com/video/aerial-view-of-shibuya-crossing-in-tokyo-37570306/) — 3(俯瞰、人が点・群れ)
- [Vehicles and People at Shibuya Crossing in Tokyo](https://www.pexels.com/video/vehicles-and-people-at-shibuya-crossing-in-tokyo-12304241/) — 3'(昼、車と人が横切る)
- [Shibuya Crossing at Night](https://www.pexels.com/video/shibuya-crossing-at-night-tokyo-s-vibrant-heartbeat-31044319/) — 1・2(夜のネオン)
- [Timelapse of Shibuya Crossing at Night](https://www.pexels.com/video/timelapse-of-shibuya-crossing-in-tokyo-at-night-32173699/) — 3(速い流れ)
- [Pixabay: tokyo street](https://pixabay.com/videos/search/tokyo%20street/) — 1(電車・駅・路地のカット)
- 3D の物: [Khronos glTF-Sample-Assets](https://github.com/KhronosGroup/glTF-Sample-Assets)(CC0 / CC BY、モデルごと)

利用者が撮った東京の素材があればそれが先。

## 順(案)

3(映像の中身)→ 3' → 2(カメラ)→ 1(時間)→ 5 → 4。3 は解析の橋・Blob・並べる法が既に揃っていて、Web で作れない絵に一番近い。

## 3 を作った(2026-09-14)

- 素材: 利用者が置いた Pexels の縦の俯瞰 `15920770_540_960_30fps.mp4`(渋谷スクランブル交差点、21.6 秒)
- 口: Grid の Group の `Exclusions`(Blob Track を持つ層を指す)。塊が重なる枠へ見えない子を明示の位置で置き、カードは CSS の自動配置で残りの枠へ流れる(先例: CSS Exclusions の wrap-flow、Grid の自動配置は明示の子の枠を飛ばす)
- 拾い方: Blob Track の Find By = Motion、Threshold 0.05・Detail 270・Separation 0・Min Size 150(Brightness はアスファルトまで拾い、Motion 0.03 は画面ごと拾い、0.06 以上は何も拾わなかった)
- 途中で直した物: 解析を読む view が、読まない view の並べた結果の覚えを共有していた(`with_analysis` で覚えを分ける)、層を指す欄が数(F64)で届く
- 見え方: 群れの動きに合わせてカードの流れ込む枠がコマごとに変わる(跳ぶ)。滑らかにするなら FLIP(延期の欄)

## 既に作った例(同日)

`ui/native/src/editor/script/examples/`: `bento.js`、`grid_squash.js`、`swiss_grid.js`(Overflow Clip と文字の Fill)、`depth_cards.js`(奥へ積む)。スクラッチの `swiss3d.js`(3D の物をセルに収めたスイス・ポスター)は、絵として面白くなかった記録として残す。
