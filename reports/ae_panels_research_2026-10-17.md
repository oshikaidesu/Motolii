# AE のスクリプトパネルと、キュレーション（おすすめまとめ）の統合（2026-10-17）

2本の外部調査（調査役のサブエージェント、ウェブ）。出典 URL は各調査の報告にあり、主なものは末尾に載せた。

## 1. まとめ22本の統合（英語13・日本語7・中国語3）

何本のまとめに名前が出たかの多い順：

| 道具 | 本数 | 何をするものか |
|---|---|---|
| Flow | 9 | イージング曲線と、保存したイージングの一覧 |
| Mt. Mograph Motion 2/3/4/Studio | 9 | 何十もの「1回押すだけ」の作業を並べたパネル |
| Duik | 8 | リグ（骨組み）を組む道具 |
| Motion Tools Pro | 8 | 安価な道具のパネル。修飾キーで別の動きになる |
| Newton | 8 | 2D の物理 |
| Overlord | 7 | Illustrator との行き来 |
| Deep Glow | 7 | 本物らしいグロー |
| Element 3D | 7 | 3D モデル |
| KBar | 6 | 自分でボタンを作れるツールバー |
| RubberHose | 6 | 2つの点から手足を作る |
| FX Console・EaseCopy・ButtCapper・Trapcode・Plexus・Animation Composer・AEJuice | 5 | |

**AE 自身に足りないもの**
- イージングの一覧と、曲線を直接触る編集
- 「1回押すだけ」の作業の棚
- 自分で作るボタン、検索できる命令
- リグ、物理、二次的な揺れ
- 形の取り込み
- 文字の箱と、文字の分割
- 色の一覧（パレット）
- 3D と粒子
- 光らせる表現
- 動く見本が付いたプリセットの一覧

**日本だけに出てくるもの**
- 文字を1文字・1語・1行ずつに分ける系（GG文解、BaraMoji、Decompose Text）
- BluBluBar、Colorize、Trim Pack、CuttanaNir2（書き順のアニメーション）
- 「海外のスクリプトは日本語版の AE で壊れることがある」という注意

**中国だけに出てくるもの**
- 柔らかい体と揺れ（Soft Body、SpringyFX、XOscillator）
- 色（Pigment、GradientHub）

## 2. パネルの見た目（画面写真を見たもの）

**よく使われる型**
1. **アイコンが結果そのもの**：ButtCapper の端の形、Duik の動物、Flow の曲線の縮小図
2. **描かれた曲線をそのまま引く**：Flow、Motion Studio、Lazy
3. **場所を選ぶ 3×3 の点**：アンカーや揃え（Motion Tools Pro、Motion Studio）
4. **押すと塗れる色見本**：Ray Dynamic Color
5. **動く見本の一覧（乗せると再生）**：Animation Composer、Motion Bro
6. **方向や姿勢を決める文字盤・パッド**：Newton の重力の針、Joysticks 'n Sliders
7. **自分で組むツールバーと修飾キー**：KBar、Motion Tools Pro の配置、Explode の Shift
8. **詰め具合の切り替え**：同じ道具を、一覧・小さな名前付きの格子・アイコンだけの3通りで見せる（Duik）

**文字に頼るもの**：True Comp Duplicator、Rubberhose の設定、Newton の数値行。一度決めたら触らない設定にだけ使われている。

## Motolii にとって

- **カードの絵（型1・2）は今の方向と合っている。** 光・カメラ・コピー・合成で効いたのも、この型だった。
- **愛される道具は「性質の欄」より「動詞のボタン」。**
  - 1回押すだけの作業の棚（アンカーを角へ、中央へ、並べる、ヌルを作る…）
  - 自分で組むツールバー
  - 動く見本付きのプリセット
  - 曲線のイージング一覧
- **Motolii に既にあるもの**：
  - Create 棚のレシピ
  - ユーザーのフォルダのスクリプト
  - Ease の desk
  - Transform の図

  欠けているのは、動く見本と「1回押す棚」。
- **日本向けの差**：文字の分割は、日本のまとめにしか出てこない定番。

## 主な出典

- まとめ：Ben Marriott top 10（aescripts learn, 2020）、School of Motion 2026、PremiumBeat 2020、Made by Loop 2022、Boone Loves Video 2022、awesome-ae（GitHub）、creators-plus 2021、izuka-effects 2026、note DOT 2023、nextist 2026、union-company 2023
- パネル：aescripts（motion-tools-pro、flow、kbar、animation-composer、joysticks-n-sliders、explode-shape-layers、ray-dynamic-color、lazy）、mtmograph.com/products/motion、duik.org、battleaxe.co（buttcapper、rubberhose）、motionboutique.com（newton-4）
- **読めなかったもの**：aescripts の本文（403）、Reddit、Vook、知乎、CSDN、Bilibili。これらは検索結果の要約だけから数えた。
