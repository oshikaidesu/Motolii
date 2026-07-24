# Vism Kit / Plugin Set統合決定 — Rack型の作者成果へ一本化する（2026-07-23）

状態: **決定**（意味と用語を統合。公開schema、container、install、製品UIは未決）

関連正本: [Vism / Kitモデル](../vism-kit-model.md)、[Community distribution model](../community-distribution-model.md)、[Vismコンセプト](../vism-package-concept.md)、[Creator / Developer連続体](2026-07-22-creator-developer-continuum-decision.md)

## 1. 決定

独立した正式artifactとしての**Plugin Setを廃止し、Vism Kitと同義へ統合する**。今後の製品語彙と設計語彙は`Vism Kit`または短く`Kit`とする。

```text
Vism       = 一つの表現／provider
Preset     = 原則として一つの能力の設定
Vism Kit   = 複数Vismを接続し、既定値と公開controlを与えた再利用可能な用途
curator list / feed
           = 複数のVism／Kitを紹介する外部の発見情報
Project Lock
           = Projectが実際に解決した版・source・artifactの再現情報
```

KitはAbletonのRackに似た**構造上の類比**として扱う。複数の能力を接続済みの一単位として選べ、初心者はすぐ使え、作者は内部構成とmacro相当の公開controlを設計でき、受け手は展開後に構成を理解しforkできる。ただしAbletonのfile format、device runtime、UI、license、音楽中心の製品存在論をMotoliiへ継承しない。

## 2. Kitであるための境界

Kitは単に導入候補を並べたbundleではない。少なくとも次の**用途としての結束**を持つ。

- 複数Vismまたはprovider／consumerの具体的な選択。
- 型付き接続。
- 初期parameter。
- 利用者へ公開するcontrol。
- 必要なassetと、利用不能時の診断。
- 一つの目的、look、workflowとして説明できるpreview／example。

たとえば、Beat provider、motion、particle、glowを接続した`Music Reactive Kit`や、複数のprimitiveから完成したVHS lookを作るKitは該当する。互いに無関係な十個のVismを「有名作者おすすめ」として並べただけの集合はKitではない。

一つのVismだけで不可分なlookを実現できるなら、無理にKitへ分割しない。単一能力の値だけを変えるならPresetを使う。Kitという名前で巨大な実行code、Host環境、install store、Project snapshotを包まない。

## 3. 旧Plugin Setの責任をどう処分するか

旧Plugin Setが持っていた価値は二つに分ける。

| 旧責任 | 現在の置き場 |
|---|---|
| 接続済みの一式を、そのまま使える作者成果として渡す | **Vism Kit** |
| 複数の無関係なVism／Kitを、個人や界隈がおすすめとして列挙する | **curator list／feed** |

curator list／feedは意味論artifactではない。外部記事、動画、個人index、静的なリンク集等がKitやVismのidentity／取得先を指す発見層であり、Hostが一つの共通schema、parser、install transaction、信頼判定を持つことをまだ要求しない。

著名な作者、界隈の実践者、first-party teamはいずれもKitを作り、list／feedで複数のKitを紹介できる。著名人をcommunityの代表者や単一の門番にはしない。first-party Starter KitとcreatorのSignature Kitも同じ公開境界を通り、公式であることを実行特権、安全性、品質の代用にしない。

## 4. 配布と再現は統合しない

Vism Kitはhostless配布の対象になり得るが、hostlessそのものや配布backendの代名詞ではない。

- catalog／分散indexはVismとKitの存在、由来、取得先を指す。
- User libraryは本人が選んだVismとKitの日常の棚を持つ。
- curator list／feedは複数のVism／Kitを紹介する。
- Kitは接続済みの用途をProjectへmaterializeする。
- Project Lockはmaterialize後を含むProjectの厳密な解決結果を再現する。
- install store／loaderは検査済みartifactの端末配置と実行を担う。

この分離により、GitHub等を作者正本とする方式を比較しても、Kit identity、URL、artifact hash、Project Lockを流用せずに済む。

## 5. 既存決定の改訂

2026-07-23に回収した`Plugin Set`は、旧plugin ecosystemの「人へ一式を渡す」価値を失わないための暫定名だった。本決定はその歴史判定を否定せず、用途として結束した一式を現行Vism Kitへ統合し、推薦だけの集合をcurator list／feedへ移したものである。

したがって、次の旧記述は本決定で上書きする。

- 「Vism Kitは作品内構成、Plugin Setは共有入口として別artifact」
- 「catalogがpackage、Kit、Setを別成果物として発見する」
- 「Plugin Set固有のschema、拡張子、parserを将来決める」

Project Lock、catalog、User library、artifact、install storeの責任分離は維持する。

## 6. 実装停止線

- `Plugin Set`を別の公開型、manifest、file、拡張子、parser、install APIとして追加しない。
- curator list／feedをProject Document、Kit意味、trust正本、atomic install要求へ昇格しない。
- 接続、初期値、公開controlを持たない任意package集合をKitと呼ばない。
- Kitへ任意Document mutation code、独自runtime、独自Undo、install store、Project Lockを同梱しない。
- Ableton Rackという類比からUI、wire、container、linked update、macro上限を逆算しない。
- VSM-B0/B1/B2、atomic batch、B2Iを飛ばしてKit schemaまたはmaterializerを実装しない。
- Project openからnetwork、install、build、任意code実行を起こさない。

本決定は用語と責任の統合であり、未決のKit container、配布protocol、署名、trust、loader、製品UIの実装許可ではない。
