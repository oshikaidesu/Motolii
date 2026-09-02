# 直観的な拡張文法 — 後発AE競合の実装比較とBrowser入口の固定

- 日付: 2026-09-02
- 問い: 後発のAE競合は、AEの直観的でなさへ実際にどう対処したか。その結果を、
  Rerunと編集体系を既に持つMotoliiへどう移すか
- 位置づけ: **決定**。製品比較は一次資料に基づく観察、後半のBrowser／拡張文法は
  利用者裁定

## 結論

後発製品はAEの複雑さを消していない。別の場所へ移した。

- Apple Motionは、値を動かす原因を**Parameter Behavior**として値の隣へ置いた
- Cavalryは、反復・個体差・変形を**Behaviour／Duplicator／Component**へ型付きで置いた
- Autographは、画像だけでなく全型の値を**Generator＋Modifiers＋Link**で統一した
- Fusionは、描画の因果を**node graph**として全面公開した

各製品が成功した部分は異なる。一方、一般化した内部構造をそのまま画面へ出した所では、
mode、slot、connection、node graphという別の学習を利用者へ要求した。

Motoliiはこれらの内部構造を再発明する必要がない。値の意味は`PropertySource`、表現はVism、
空間・描画・pickingはRerunが持つ。利用者側の新機能入口も、既存のBrowser群が同じ
card/grid文法で持っている。

したがって固定する原則は一つ。

> **新しい能力は名詞としてBrowserへ増やしてよい。新しい動詞・mode・panel・操作文法を増やさない。**

## 一次資料で確認した4つの対処

### Apple Motion — 原因を値へ直接付ける

公式User GuideのParameter Behaviorは、オブジェクト全体ではなく**一つのparameter**へ適用される。
同じOscillateをOpacityへ付ければ点滅し、Rotationへ付ければ揺れる。Audio Behaviorは音声解析を
filter、replicator、shape、light、text等のparameterへ直接適用し、通常のkeyを必要としない。
Behaviorは対象オブジェクトの下に表示され、対象parameterのkey buttonにも印が付く。

これは「音をkeyへ変換してexpressionでScaleへ繋ぐ」のではなく、
**音がScaleを動かしている関係をそのまま残す**対処である。

一方で、Groupには2D／3Dの区別、Flatten、Layer Orderがある。値の駆動は直観化したが、
描画境界は利用者がswitchで選ぶまま残った。

一次資料:

- [Intro to Parameter behaviors](https://support.apple.com/en-ie/guide/motion/motn49eb56eb/mac)
- [Behaviors versus keyframes](https://support.apple.com/guide/motion/behaviors-versus-keyframes-motnf425e02f/6.3/mac/15.6)
- [Audio parameter behavior](https://support.apple.com/guide/motion/audio-parameter-behavior-motn1872e265/mac)
- [2D and 3D group properties](https://support.apple.com/nl-nl/guide/motion/motn2fb59f53/mac)

### Cavalry — 反復と個体差を一級の層にする

CavalryではShapeもBehaviourもLayerである。DuplicatorはInput Shapeを複製し、配置だけでなく
個体ごとのPosition／Rotation／Scale／Visibility／Opacity、Shape Id、Time Offset、Index Contextを
持つ。Stagger BehaviourをTime Offsetへ繋げれば、複製を独立Layerへ展開せず順番を作れる。

Groupは整理と共通Transformを持ち、Componentは子Layerを隠して必要なattributeだけを昇格し、
利用者向けの小さいUIを作れる。これは反復・部品化・公開parameterを実物として解いた先例である。

一方、Groupには`Artboard`／`Individual Shapes`という合成境界の選択があり、Componentには
Edit modeがある。Behaviour catalogとattribute connectionが増えるほど、利用者はCavalryの
接続体系自体を学ぶ必要がある。

一次資料:

- [Behaviours](https://cavalry.studio/docs/nodes/behaviours/)
- [Duplicator](https://cavalry.studio/docs/nodes/shapes/duplicator/)
- [Group](https://cavalry.studio/docs/nodes/shapes/group/)
- [Component](https://cavalry.studio/docs/nodes/shapes/component-shape/)

### Autograph — すべての値を同じ型付きpipeへ載せる

Autographは各parameterにGenerator slotを1つ、Modifier slotを複数持つ。対象は画像に限らず、
数値、文字、2D/3D/4D値、3D Objectまで同じで、候補はparameterの型で絞られる。
値は手入力、Generator、別parameterからのLinkで得られ、その後へModifierを順に掛ける。
元の値とModifier後の値も同じparameter上で比較できる。

これはMotoliiの`PropertySource { base, modulators }`に最も近い実物である。RandomやOscillatorを
Position／Scaleへ付けるためにexpressionを要求しない。

ただし一般化を画面へ全面展開したため、全parameterにslot体系が現れる。Modifierは通常上から下、
Transform Modifierだけ行列積の都合で下から上という例外もある。Sub-compositionには
Crop and Rasterize、Parent FPS、Parent Format等のflagがあり、Group相当を作る時にも
Sub-compを経由する。内部モデルの統一は強いが、そのモデルを理解する負担は残る。

一次資料:

- [Generators and Modifiers](https://help.maxon.net/ag/en-us/Content/html/Generators_modifiers.html)
- [Sub-compositions](https://help.maxon.net/ag/en-us/Content/html/Sub_compositions.html)
- [Select and Move / Transform Modifier order](https://help.maxon.net/ag/en-us/Content/html/Select_and_move.html)

### Fusion — 内部IRを利用者へ公開する

Fusionは画像処理、mask、merge、3D scene、Renderer3Dをnodeとして接続する。どの入力がどのeffectを
通り、どこで3D sceneが2D画像になるかは明示される。Group／Macroはnode群を畳み、選んだparameterだけを
Inspectorへ公開できる。

AEのprecompやrender orderにある隠れた因果は減るが、利用者自身が描画IRを組み立てる。
VFXの検査可能性には強い一方、「ロゴを少し跳ねさせる」ための第一面にはならない。

一次資料:

- [DaVinci Resolve: Fusion](https://www.blackmagicdesign.com/products/davinciresolve/?param1=introduction-to-operations-hub)
- [Fusion Reference Manual](https://documents.blackmagicdesign.com/UserManuals/FusionManual.pdf?_v=1724310010000)

## 比較から分かったこと

| 解いた問題 | 最も強い先例 | Motoliiでの置き場 |
|---|---|---|
| 値を何が動かすか | Apple Motion | `PropertySource.modulators`。UIは値の隣／同じBrowser文法 |
| 反復・番号・時刻差 | Cavalry | 駆動の口。独立LayerをN枚作らず番号を配る |
| 素材・値・効果の型統一 | Autograph | typed source／Vism manifest／PropertySource |
| 因果と描画境界 | Fusion | 内部Evaluation Graph。製品UIには出さない |
| 空間・描画・picking | 各競合で別実装 | Rerun／re_rendererを使い、Motoliiで再発明しない |

どの製品も単独ではMotoliiの答えではない。成功した層だけを採る。

## MotoliiのBrowser入口

[窓の説明](../wiki/window.md)は、素材=`Media`、効果=`Effects`、新しい層=`Create`、
作品内の色=`Colors`と役割を既に分けている。現行
[`ui/browser.rs`](../../motolii/src/ui/browser.rs)も、これらを同じ`tcard`／`tgrid`文法で描く。

このため、新しいVism、素材、生成物、将来の駆動に専用のUI体系は要らない。

| 新しい名詞 | 既存の動詞 | UIの形 |
|---|---|---|
| 素材／LayerSource | 置く | BrowserのcardからLayerへ |
| Effect／Vism | 掛ける | Browserのcardから選択Layer／Groupへ |
| Driver／解析値／番号 | 値へ付ける | Browser同族のcardから選択propertyへ |

cardの中身、category、検索語、previewは増えてよい。しかし入口から結果までの動詞は増やさない。

## 今後の実装を落とす条件

新機能が次のどれかを要求したら、既存の口へ収まっていない。

1. 専用modeへ切り替えてから使う
2. 専用panel／専用node graphへ移動する
3. 先に変換・precompose・bakeしてから使う
4. 専用の確定buttonを押すまで結果へ届かない
5. その機能だけのevent bus、Undo、Preview／Export経路、Inspector分岐を持つ
6. 2つ目を足す時にBrowser／Document／rendererへ同じ名前の分岐をもう一度書く

反対に、追加がmanifest／data／cardだけで既存routeを通り、既存の操作を知る人が新しい説明なしで
使えれば、拡張と直観性が両立している。

## 合格の問い

新しい能力ごとに、実装前に4問だけ当てる。

1. 既存の「置く・掛ける・値へ付ける」のどれか一文で説明できるか
2. 利用者が新しいmode／panel／順序を覚えずに届くか
3. cardを操作した直後、対象と結果と関係が同じ場所で見えるか
4. 二つ目の同族はコードでなくmanifest／data追加だけで載るか

1つでもNoなら、機能不足ではなく**拡張口の未完成**として扱う。

## この文書が決めないこと

- Browser内の最終tab名、並び順、visual token
- Driver cardをclick／dragのどちらでpropertyへ付けるか
- 各競合製品の全面採用や製品移行
- 内部Evaluation Graphの具体型

固定するのは、**新しい名詞を既存Browser文法へ載せ、内部技術を新しい操作文法として漏らさない**
という境界である。
