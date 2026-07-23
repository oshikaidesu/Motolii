# Community distribution model — 地図、棚、共有入口を分ける

作成日: 2026-07-23

状態: **運用・ガバナンス原則決定／protocol・schema・製品UI未決**。Motolii運営の常設配布backend、公式人気順位、中央dedupeを持たず、多数のcreator-authorが作る表現を分散した地図、個人の棚、Rack型のVism Kit、外部キュレーションで扱う。Kit／Project Lockの形式、署名、install、runtimeは[Vism実装計画](reviews/2026-07-17-vism-implementation-plan.md)の順序を飛ばさない。

関連: [Vismコンセプト](vism-package-concept.md)、[Vism / Kitモデル](vism-kit-model.md)、[Kit / Plugin Set統合決定](reviews/2026-07-23-vism-kit-rack-unification-decision.md)、[Creator / Developer連続体](reviews/2026-07-22-creator-developer-continuum-decision.md)、[歴史回収](reviews/2026-07-23-historical-plugin-ecosystem-lineage-recovery.md)

## 1. 結論

Motoliiのcommunity成長は、中央marketplaceが一つずつ順位付けして配る形を主回路にしない。

```text
creator-authorが表現を公開
  → 分散catalogが存在と由来を指す
    → 各利用者が自分の棚へ選ぶ
      → 接続済みのVism Kitで用途を他者へ渡す
      → curator list／feedが複数のVism／Kitを紹介する
        → Project Lockが作品の再現を助ける
```

各層の役割は次のとおりである。

| 層 | 正本／責任 | 持たないもの |
|---|---|---|
| 作者source／artifact | 表現の実体、版、由来 | 人気順位、利用者の棚 |
| catalog／分散index | 存在、identity、kind/tag、更新、取得先を指す地図 | 実体、install状態、公式推薦 |
| User library | 導入済み、Folder、Label、History等、本人の日常選択 | 全communityの正本、Project意味 |
| Vism Kit | 接続済みのVism、初期値、公開control、assetからなる用途と共有単位 | 作品の厳密な解決結果、無関係なおすすめ集合 |
| curator list／feed | 複数のVism／Kitを紹介する外部の発見情報 | Project意味、atomic install、trust判定 |
| Project Lock（仮称） | 作品が実際に要求・解決した版、source、artifact | 推薦、人気、作者の配布正本 |
| install store／loader | 端末上の検査済み実体と実行 | catalog、Project、Kit、list／feedの意味 |

## 2. Communityのガバナンス原則

### 2.1 中央で世界を一意にしない

似たVHS、Glow、字幕表現が複数存在してよい。Motolii運営は意味的類似を判定して一つへ統合せず、「公式正規版」を認定せず、作者間のmerge queueを持たない。複数作者が同じ問題へ別の答えを出せることは、人海戦術の重複損失だけでなく探索能力でもある。

衝突を放置する意味ではない。package identity、作者、由来、互換、権限、欠落理由を区別でき、同名でも別物と読める必要がある。悪意、なりすまし、署名、権限はtrust policyで扱い、表現の類似度や運営の好みで代用しない。

### 2.2 人気を正本にしない

Motoliiはdownload数、trend、利用telemetry、公式ランキングを集計しない。累積人気は既知の表現をさらに押し上げ、新しい尖りを発見地図の下へ沈めるためである。

catalogが持つ第一情報はidentity、作者／由来、kind／tag、compatibility、更新、取得先である。時点ごとの推薦や批評は、外部記事、動画、個人のcurator list／feed等が担える。外部のstarや順位を表示する場合もMotolii公式の品質・安全・互換判定へ変換しない。

### 2.3 日常の視界はUser libraryへ閉じる

全catalogは「たまに開く地図」であり、制作中の毎回の選択面ではない。利用者の日常は導入済み、Folder、Label、確定使用だけのHistory等からなる小さな棚へ閉じる。似た表現の増加を中央dedupeで止めず、本人が選んだ範囲を安定して再発見できるようにする。

User libraryの整理はstable package identityへ結び、install path、repository path、display nameから導出しない。保存先と同期形式はUser settings／Workspaceの所有審判を通し、Project Documentへcatalog UI都合を焼かない。

### 2.4 界隈の違いは共有入口にする

歌詞、VHS、ジェネラティブ、3D等で使う一式が異なることを、公式必須セットへ統一しない。痛点は違いそのものではなく、「同じ入口へどう入るか分からない」ことである。

接続、初期値、公開controlまで設計したVism Kitを、個人や界隈がそのまま使える用途として渡す。受け手は不足、非互換、外部購入、権限を確認してからmaterializeする。これは作品内構成と共有入口を同じ作者成果へ一本化するRack型の境界である。

互いに無関係なVism／Kitを「この界隈のおすすめ」として列挙するだけなら、外部のcurator list／feedで紹介する。list／feedは共通の意味論artifact、atomic install、trust正本ではない。Project Lockは作品再現用であり、Kitの用途意味やlist／feedの推薦責任と分ける。

## 3. lookとprimitiveを排他的にしない

表現の粒度には二つの正当な需要がある。

| 粒度 | 役割 | 原則 |
|---|---|---|
| **look／意図単位** | VHS、完成した歌詞表現等、利用者が一つ選んですぐ画になる | 通常の第一入口。不可分なら一Vism／一実装へ閉じてよい |
| **primitive／部品** | 魚眼、色収差、scanline等、作者や高度利用者が再構成する | 再発明を減らし、Vism Kitやstackで再利用できる |

粒を許すことは「全員がnode graphで組め」という意味ではなく、lookを許すことは同じ部品を複数実装へ隠して二重保守する推奨でもない。独立して差替え・更新する意味がある部品は小さなVismとKit構成を比較し、見た目とperformanceが不可分なら一つのlookとして閉じる。

`look`／`primitive` tag、`related` field、一覧の既定filterは未決である。歴史上の仮schemaを`NodeDesc`やmanifestへ戻さない。

## 4. 発見地図とruntimeは別の時計で進む

発見地図は、dynamic loaderが完成しなければ価値がないわけではない。first-party参照実装、作者source、docs、外部previewを発見し、現在のHostが扱えるものと将来候補を区別して示すread-only地図には独立した価値がある。

一方、旧GAP-13の「早期地図」をそのまま製品taskへ戻さない。package identity、provenance、capability、source／artifact、installed、compatible、availableの語彙が未決のまま一覧を作ると、URLや`NodeDesc.id`が恒久package identityになる。最初の正規fixtureは`VSM-B0/B1/B3H`で意味を固定し、製品install UIはPhase D/Eの依存を守る。

read-only discoveryをloaderより先に比較しても、次を称さない。

- source linkがあることをinstall可能、trusted、compatibleと表示する。
- static bundled first-partyをdownload済みcommunity packageと表示する。
- catalog entryをruntime registryへ直接登録する。
- Project openからcatalog fetch、install、build、executeを起こす。

## 5. 決めたことと未決を分ける

| 主題 | 状態 |
|---|---|
| Motolii運営の常設配布backendへ必須依存しない | **決定** |
| Motoliiがdownload数、公式人気順位、中央dedupeを所有しない | **決定** |
| User libraryを全catalogと分け、stable identityへ結ぶ | **設計原則** |
| 類似表現、look、primitive、複数界隈を許容する | **設計原則** |
| 独立artifactとしてのPlugin Setを廃止し、接続済み一式をVism Kitへ統合する | **決定** |
| curator list／feedとProject LockをKitから分ける | **責任分離決定／共通schema・UI未決** |
| 分散index／外部キュレーションを使う | **設計方向** |
| GitHub、静的HTTP、mirror、local commercial packageの具体topology | **VSM-B3Hで比較** |
| catalog／Kit／Lock／manifest／installed形式 | **未決** |
| 署名、失効、trust、権限、build、loader | **未決** |
| read-only discoveryの製品投入時期 | **B0/B1/B3H後に再判定** |

## 6. 停止線

- `NodeDesc.id`をpackage identityと宣言しない。
- GitHub repository、URL、tag、display nameをidentityやtrustへ流用しない。
- catalogの取得成功をartifact検証、install、runtime availabilityと同義にしない。
- `git fetch → cargo build → register`を確認なしの標準導入経路にしない。
- catalog、Kit、curator list／feed、Lock、installed stateを一つのTOML／DBへ統合しない。
- `Plugin Set`を別の公開型、file、parser、install APIとして復活させない。
- 接続、初期値、公開controlを持たない任意package集合をKitと呼ばない。
- Project openでnetwork、install、build、code実行を起こさない。
- 類似度、download数、公式おすすめを安全性・互換性の代用にしない。
- User libraryのFolder／Label／HistoryをProject Documentへ保存しない。
- look／primitiveの仮tagや`related` fieldを現行`NodeDesc`へ追加しない。
- 旧`.motoliipack`、`.motolii-kit`、`plugins.lock.toml`、`tap.toml`を確定形式として復活させない。
