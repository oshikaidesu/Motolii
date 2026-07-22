# Creator / Developer連続体 — 多数の作者が製品を育てる構造（2026-07-22）

状態: **決定**

対象: [concept.md](../concept.md)、[小さなコアと探索可能な拡張](../extensible-core-model.md)、[Vism](../vism-package-concept.md)、[ジェネラティブユーザー境界](../generative-user-boundary.md)、[UI runtime責任境界](../ui-runtime-architecture.md)

## 1. 決定

Motoliiは「使う人」と「作る人」を別の人口として設計しない。作品を作るcreatorが、値を調整し、接続を組み、recipeを保存し、既存表現をforkし、新しいVismやUI componentを公開するまでを、**同じ意味と道具が連続する一つの作者経路**として設計する。

目標は全員へprogrammingを要求することではない。codeを書かなくても完成作品を作れ、必要になった人だけが現在の対象と状態を保ったまま、一段ずつ作者側へ進めることにある。

```text
使う
  → 調整する
    → 組み合わせる
      → 中身と由来を調べる
        → forkして変える
          → Vism / Kit / componentとして公開する
            → 別のcreatorが使う
              → 反復需要がfirst-party / Host能力へ昇格する
```

この循環によって、製品会社だけで基礎機能を列挙する速度ではなく、多数のcreator-authorが並行して新しい表現世界を作る速度をMotoliiの成長力にする。機能数を無制限にCoreへ取り込む「人海戦術」ではなく、**小さな公開境界の上で独立した発明を安全に並行できる人海戦術**である。

## 2. 三つの構造

### 2.1 Reactは作者入口を広げる

React採用は標準製品UIの実装都合だけではない。一般的なWeb component、CSS、Storybook、Playwright、hot reloadを使い、製品作者とcommunity作者がcomponent、interaction、testの語彙を共有しやすくする。

ただし、product-owned React packageがそのまま公開plugin UI APIになるわけではない。sandbox、権限、互換、配布、a11y、version、crash isolationはG0-3 / GAP-13で別に決める。作者人口を広げるためにtrust境界を消さない。

### 2.2 Vismは作品内の工夫を配布可能な作者成果へ変える

Vismは「developerだけが作るbinary plugin」ではなく、一つの映像表現を作品やHostから切り離して保存・共有・再利用する作者単位である。code、WGSL、宣言的recipe、既存能力の型付き合成等の具体形式は未決だが、使い手が作った工夫を巨大project templateへ閉じ込めず、独立したidentity、version、要求能力、由来を持つ成果へ昇格できる方向を維持する。

Kitは複数Vismを用途へ組む作者面であり、codeを書かない構成作者もこの循環へ参加できる。利用と開発の境界を薄くするのは、すべてをsource codeへ変換することではなく、**小さな工夫に検査可能な作者identityと配布経路を与えること**である。

### 2.3 First-party pluginは手本であり特権階級ではない

将来標準搭載するLyrics、Particle、Glow等は、第三者が到達できない内部APIで作らない。現時点のコード実証はOpacity、Sine、Radial Repeaterである。公開contract、scaffold、testkit、conformance fixtureだけで成立させ、実装sourceと失敗例を次の作者の教材にする。

First-partyの役割は最低限の製品品質を保証すると同時に、第三者へ「この境界でここまで作れる」と証明することである。公開境界で作れない時はfirst-partyだけの裏口を足さず、欠けた共通能力として審判する。

## 3. 消す境界／残す境界

| 薄くする・消す | 明示して残す |
|---|---|
| user / developerという固定身分 | untrusted / reviewed / bundledというprovenanceとtrust |
| Simple利用とAdvanced authoringの概念断絶 | Document single writer、Undo、保存、migration |
| 製品作者とcommunity作者のcomponent／test語彙の二重化 | React／native surface所有とplugin公開契約の分離 |
| preset／recipe／code作者を別世界へ隔離すること | sandbox、permission、resource、version、license、署名 |
| first-partyだけが使える表現能力 | Core／Host module／first-party plugin／third-party pluginの責任 |

境界を無くす対象は**参加資格と学習経路**であり、作品の持続性、安全性、権限、責任ではない。誰でも作者になれることと、誰のcodeでも無確認に実行することを同義にしない。

## 4. Hostが投棄しないもの

多数の作者へ表現を開いても、次をcommunity任せにしない。

- 初回から完成作品を作れる標準体験と基礎表現
- Document、single writer、Undo、journal、migration、欠落時復元
- Preview / Export同一評価、GPU resource、cache、color、Quality
- discovery、install、更新、依存、permission、診断、削除
- accessibility、共通component、typed intent、error recovery
- first-party参照実装、scaffold、testkit、互換fixtureの保守

Motoliiの強さは「本体が何もしないので皆が穴を埋める」ことではない。Hostが壊れやすい共通責任を引き受けるため、多数の作者が表現固有の発明へ集中できることにある。

## 5. 実装へ課す審判

新しいUI、plugin、Vism、Kit、authoring機能は次を満たす。

1. codeを書かない利用者が、その機能を使わなくても標準制作を完了できる。
2. 現在の作品、選択、parameter、入力型を捨てずに、調整からinspection、fork、authoringへ進める。
3. first-party実装が公開境界と同じfixtureで検査され、第三者が再現できない内部特権を持たない。
4. 配布物の作者、由来、version、要求能力、permission、欠落時挙動をHostが説明できる。
5. 一人の作者や一packageの失敗が、無関係な作品領域や他作者の表現を壊さない。
6. 繰り返し現れる需要を観測し、互換性を壊さずpreset、first-party、Host primitiveへ追加的に昇格できる。

この決定は、custom plugin UI、Vism loader、marketplace、package形式、署名方式、新しいDocument variantを実装する許可ではない。それぞれ既存の停止線と解凍手続きを通す。

歴史処分追補(2026-07-23): first-party公開façadeのコード実証と、第三者runtime／trustの未成立範囲は[公開capability／provenance回収](2026-07-23-historical-public-capability-provenance-lineage-recovery.md)を正とする。
