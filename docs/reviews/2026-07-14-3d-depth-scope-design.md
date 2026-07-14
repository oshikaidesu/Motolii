# 2Dレイヤー順と3D深度合成の境界設計（2026-07-14）

ステータス: **設計レビュー**（仕様採択前。M5/C-4を変更しない）

対象: 「映像が主役なら、Z位置に合わせてレイヤーを自動整理すべきではないか」という疑問と、将来のレイヤー間深度遮蔽の置き場。

## 結論

**Zから描画順を導出することは有効だが、タイムライン項目そのものを自動並べ替えしてはならない。**

- タイムライン順は、所有・選択・時間編集・親子関係を追うための安定した authoring order とする
- 空間上の前後関係はキャンバスの責務とし、将来の明示的な `Depth Group` 内だけで camera-space depth から render order を導出する
- トップレベルと通常グループは現行C-4どおりレイヤー順を正とする
- 深度はグループ境界を越えず、グループ出力はpremultiplied RGBAへ平坦化して上位へ渡す
- `Depth Group` はv1へ入れない。M5の現行`LayerSourcePlugin → RGBA`境界とは別能力として、必要性を実作例で確認した後に判断する

この分離により「映像の見た目をZに従わせる」ことと「編集対象が再生中に飛び回らない」ことを両立できる。

## 先例の比較

### After Effects: 暗黙binは採らない

After Effectsは、連続する3Dレイヤーを同じ3D binで処理する。2Dレイヤー、3D adjustment layer、layer style付き3Dレイヤー等がbinを分断し、別bin同士は交差・相互シャドウを行わない。現在はTimelineへbin境界を表示するが、スコープの意味が隣接項目の種類から創発する点は変わらない。

出典: [Adobe — 3D animation settings / binning indicators](https://helpx.adobe.com/after-effects/desktop/work-with-3d-composition/work-with-3d-animation-settings/3d-animation-settings.html)

Motoliiでは「ある項目の種類やトグルが隣接レイヤーの意味を変える」方式を採らない。深度集合は明示コンテナだけで作る。

### Apple Motion: 明示2D/3Dグループは強い先例

Motionは、2Dグループ内をレイヤー順、3Dグループ内を深度順で合成する。さらに3Dグループには、深度順を無効にしてLayers list順へ戻す`Layer Order`設定と、平坦化する`Flatten`設定がある。

出典: [Apple — 2D and 3D group properties in Motion](https://support.apple.com/guide/motion/motn2fb59f53/mac)、[Apple — Create 3D intersection](https://support.apple.com/guide/motion/motn369b0783/mac)

「明示グループの中だけ意味を切り替える」点はMotoliiのグループ境界・ベイク・単一評価モデルと整合する。ただしMotionでも、フィルタやマスクによるラスタライズが交差を止める場合がある。Motoliiはこれを無言の意味変更にせず、非対応操作を拒否するか境界状態を可視化する必要がある。

### Fusion: 単一Zによるピクセル比較は限定解

FusionのZ Mergeは、foreground/backgroundの固定順ではなく、各ピクセルのZ値を比較して前後を決める。これは各ピクセルに代表面が1つある不透明・cutoutに近い素材では有効だが、複数の半透明サンプルが同じピクセルへ重なる場合の完全解ではない。

出典: [Blackmagic Design — Fusion Tool Reference](https://documents.blackmagicdesign.com/UserManuals/Fusion7_Tool_Reference.pdf)

### Nuke Deep / GPU OIT: 半透明の完全解はRGBA+Dより広い

Nukeのdeep imageは、1ピクセルに複数の色・不透明度・camera-relative depthサンプルを保持する。KhronosのOIT例も、透明ジオメトリの色と深度をピクセルごとのリストへ収集し、ピクセル単位でソートしてからalpha blendする。

出典: [Foundry — Deep Compositing](https://learn.foundry.com/nuke/current/content/comp_environment/deep/deep_compositing.html)、[Khronos — OIT with per-pixel ordered linked lists](https://docs.vulkan.org/samples/latest/samples/api/oit_linked_lists/README.html)

したがって、単一の`RGBA+D`だけでは「同じパーティクルレイヤーの一部がキャラクターの前、一部が後ろ」を一般には保持できない。1ピクセル内で前後両側の粒子が重なれば、平坦化時点で情報が失われる。完全性が必要ならdeep samples、OIT、または元ジオメトリを共有レンダーパスへ参加させる能力が必要になる。

## 自動整理の正しい意味

Documentのレイヤー列をZ順へ書き換えてはいけない。Z・親変形・カメラはいずれも時刻で変化するため、永続順まで同期すると次の問題が起きる。

- 再生中に行と展開済みプロパティが上下へ移動する
- 選択、キーフレーム編集、マスク、親子関係の追跡が不安定になる
- Z交差のたびに履歴・ジャーナル・キャッシュ無効化が発生しかねない
- カメラを動かしただけでDocument構造が変わる
- 同距離付近で順序が振動し、状態fulなhysteresisを入れると`f(t)`の純関数性を損なう

代わりに、明示Depth Groupのrender orderを評価時の派生値とする。

```text
authoring_order: Documentに永続化する安定順
render_order(t): CompCamera(t)とworld transform(t)から毎フレーム導出
```

ソートキーはworld-spaceの`position.z`ではなくcamera-space depthである。カメラが回転・移動すればworld Zと画面上の前後は一致しない。同距離のtie-breakはauthoring orderとし、決定論を固定する。

## 段階案

### v1: C-4を維持する

- トップレベル/グループ内部ともレイヤー順
- Zはカメラ投影・視差・見かけスケールだけに使う
- 点群/メッシュ内部のZ解決は各LayerSourcePlugin内で完結
- 前後へ分けたい決定論的パーティクル等は、同一seed/同一パラメータの2出力を分割Z平面で手前・奥へ分けるプラグイン作法を使う

最後の方法は特定表現の実用イディオムであり、一般深度合成とは呼ばない。

### v1.x候補: Depth-ordered Cards

実作例で必要性が出た場合だけ、非交差平面を対象に明示Depth Groupをスパイクする。

- 子はcamera-spaceの代表深度でback-to-frontに並べる
- authoring orderは変更しない
- 平面同士の交差、循環する前後関係、複数半透明面の完全性を保証しない
- UIと仕様では「Depth compositing」ではなく「Depth-ordered Cards」等、能力を過大表示しない
- alpha overlapと交差の反例fixtureを置き、近似であることをゴールデン化する

単純なオブジェクト/レイヤー単位ソートは、透明面が交差する場合には完全順を作れず、ポリゴン分割が必要になる場合がある。

出典: [Khronos — Transparency Sorting](https://wikis.khronos.org/opengl/Transparency_Sorting)

### 将来候補: Scene Participant

不透明・cutout素材の正確な交差が実需になった場合は、RGBA化済みレイヤーを並べるのではなく、参加ソースを共有depth passへ描く別能力を検討する。半透明の完全性まで必要なら、その先にOIT/deep samplesがある。

これは現行`LayerSourcePlugin`へdepth textureを足すだけの変更ではない。別trait/別出力能力として意味論、対応alpha、projection、depth range、flatten境界を同時に決める必要がある。

## `FrameDesc`へdepthを予約しない

現行`FrameDesc`は色フレームのサイズ・stride・pixel format・color space・premultiplied alphaを記述する。depth attachmentは色フレームの属性ではなく、別texture format、比較規則、projection、sample semanticsを持つレンダ成果物である。

さらにM5仕様は、厳密なレイヤー間遮蔽を現行`LayerSourcePlugin`境界とは別拡張として扱うと既に宣言している。意味が未決のまま`FrameDesc`や公開traitへ口を焼くことはGR-PVの「意味が先」「恒久面は狭く」に反する。

したがって、今はコードへ`#[non_exhaustive]`やoptional depth fieldを足さない。将来の判断席だけを本レビューに残す。

## 将来スパイクの合否条件

1. Depth Group外の既存コンポは導入前後でピクセル同一
2. Z・カメラをアニメしてもDocument順、Undo履歴、選択行は変わらない
3. 深度意味論は明示グループ内部だけに閉じ、隣接レイヤーから創発しない
4. camera-space depthを使い、同距離はauthoring orderで決定論的に解決する
5. マスク・フィルタ・flattenが深度参加を止める場合、無言で切り替えない
6. 非交差カード、交差カード、soft alpha、循環前後関係を別fixtureにし、保証範囲を機械判定する
7. preview/exportは同一の評価関数を通り、差は`Quality`だけ

## UIへの含意

タイムラインの安定順を守りつつ映像中心で操作するため、将来のDepth Groupでは次を候補とする。

- グループ境界と合成モードをicon・形・ラベルで識別可能にする
- 現在フレームのdepth rankを派生badge/gutterで示す
- 「現在深度順で表示」は一時的なview projectionとし、Document/Undoへ書かない
- キャンバスから前後の対象を選べるようにする
- 深度交差や近似alphaが保証外になる箇所を診断表示する

これは「読む前に識別できる」GR-UI 9と、UI状態をDocument意味論へ逆流させないGR-UI 1/5の両方に従う。

## 判定

- **採用候補**: 明示Depth Groupというスコープ、render orderのcamera-space depth派生、authoring orderの固定
- **棄却**: Z変化に応じたDocument/タイムライン行の自動並べ替え、AE式の暗黙隣接bin
- **縮小**: レイヤー単位Zソートは非交差カード用近似としてのみ扱う
- **延期**: RGBA+D、共有depth pass、OIT、deep samples。実需と意味論が揃うまで公開契約へ予約しない
- **現仕様への影響**: なし。M5/C-4を維持する
