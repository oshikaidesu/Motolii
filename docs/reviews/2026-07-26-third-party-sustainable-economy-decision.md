# 第三者Vismの持続可能な経済圏 — 市場を所有せず、商業を狭めない（2026-07-26）

状態: **決定**

決定したのは経済圏の目的と責任境界である。commerce、package、edition、entitlement、licenseのprotocol／schemaは未決のままであり、この文書は実装許可ではない。

対象: [concept.md](../concept.md)、[Community distribution model](../community-distribution-model.md)、[Vism package concept](../vism-package-concept.md)、[Creator / Developer連続体](2026-07-22-creator-developer-continuum-decision.md)

## 1. 決定

Motoliiは、第三者Vism／Kitの成果へ継続的な報酬が返る**持続可能な経済圏**を、長期の製品目的に含める。無料、OSS、寄付、有料買い切り、subscription、bundle、商用support、独自licenseを相互排他的にせず、作者が成果と事業に合う方法を選べることを守る。

一方で、Motolii自身は中央marketplace、決済、売上順位、download順位、license販売を所有しない。Hostが市場の売り手、審判、発見面の所有者を兼ねると、製品設計、露出、互換判断を自社取引へ有利にする誘因が生まれるためである。

原則を一文にすると次のとおりである。

> **Motoliiは市場を所有しない。作者が市場を作れる公開境界と、安全に使い続けられるHostの土壌を所有する。**

Gumroad、BOOTH、作者自身のsite、販売代理、将来の第三者subscription／license service等が、価格、決済、税、返金、購入権、顧客対応を担える。Motoliiはそれらを一社へ固定せず、外部商流を作品の唯一の存続条件にもしない。

対象は第三者Vism／Kitが成立できる環境である。Motolii本体とfirst-party成果の価格、事業、配布条件はこの決定の対象外とし、無料とも有料とも決めない。

## 2. なぜこの思想になったか

### 2.1 無料文化の強さと限界を同時に見る

AviUtlに代表される無料中心の拡張文化は、決済なしですぐ試せ、知識と表現が高速に共有され、多数の作者が参加できる強さを持つ。Motoliiはこの価値を失わせない。

一方、製品所有者は、無料や「お布施」だけが共同体の道徳的な既定になると、制作、保守、support、互換追従へ費やした労働が見えにくくなり、異なる商習慣を持つ作者や継続的な専門仕事が育ちにくくなる危険を感じている。本決定はこの経験上の懸念を設計仮説として採る。無料成果を二級化する主張ではなく、**有料成果も正当な第一級成果として扱う**という決定である。

### 2.2 AE／VSTが示したのは「追加機能」以上のものだった

AE pluginやVSTの周囲で、専門vendor、suite、継続support、subscription、教育、制作会社が育ったことを、本決定は「addonの集合が別の仕事と経済へ広がり得る」先例として読む。すべての商習慣やDRMを模倣するのではなく、第三者が一機能から継続的な仕事へ育てられる余地を継承する。

Blender addonやOSS communityのようにsource公開と商業を両立する道もあれば、proprietary binary、買い切り、subscriptionを選ぶ道もある。Motolii本体とfirst-party参照実装がopenであることを、第三者実装のsource、license、価格を一律に拘束する理由へ使わない。

### 2.3 海賊版対策の摩擦を、生態系全体の禁止理由にしない

license codeや常時認証は正規利用者へ摩擦を生み、AE plugin等では導入・移行・復旧の苦労にもなった。一方、海賊版の存在だけを理由に有料配布を否定すれば、作者へ損失を引き受けさせる。

したがってMotoliiは、DRM、demo、機能制限、watermark、外部entitlementを現時点で標準化も禁止もしない。将来communityがiLok型の共通license serviceを作る可能性も閉じない。ただし、どの方式もrenderの純関数契約、offline時の作品診断、Projectの持続性、secret分離を破ってはならない。

## 3. 経済圏の憲法

| 原則 | Motoliiでの意味 |
|---|---|
| 無料と有料を同格にする | 価格を品質、trust、compatibility、検索順位の代理にしない |
| OSSとproprietaryを同格にする | 公開contractへの適合を審判し、実装licenseをHostの都合で強制しない |
| 商業モデルを固定しない | 買い切り、subscription、bundle、support契約、寄付等を作者が選べる |
| First-partyを特権化しない | 第三者と同じ公開capability、conformance、resource、診断だけを使う |
| Hostが取引を所有しない | 決済、税、返金、売上管理、購入者管理は外部providerの責任とする |
| 取引と作品意味を分ける | 購入状態、license code、account、machine IDをProject／Kitの意味にしない |
| 発見と販売を分ける | catalogはidentity、由来、互換、取得先を示す地図であり、Motolii公式売場ではない |

無料／OSS Vismを第一級に保つ方法は、有料品を不利にすることではない。同じ公開能力、同じ検査、同じ欠落診断を適用し、価格やlicenseを技術的な身分へ変換しないことである。

Hostのscaffold、testkit、conformance、互換診断は価格と無関係に同じ条件で提供する。

## 4. 責任境界

### 4.1 Motolii／Host

- stable identity、version、capability、dependency、provenance、permission、compatibilityを説明する。
- install、互換、利用可否、取得に関する異なる原因を、確定済みの共通概念の範囲で混同せず診断する。具体的な状態語彙と表現は`VSM-B0/B1/B3H`まで未決であり、この決定から導出しない。
- Kitが要求するVismと接続を正確に識別し、materialize前に欠落を示す。外部取得先は検査済みmetadataが利用可能な時だけ案内し、offlineまたは取得先不明なら欠落以上を推測しない。
- first-partyとthird-partyを同じ公開contract、resource制約、failure isolationで扱う。
- Project openからnetwork、購入、install、build、code実行を自動で起こさない。
- 外部販売serviceが消えても、既存Projectの要求と欠落理由を保持する。

### 4.2 作者／外部provider

- 価格、無料配布、source公開範囲、license、買い切り／subscription、bundleを決める。
- checkout、税、返金、購入権、download entitlement、顧客supportを担う。
- demo／機能制限／watermark／認証を採る場合、その挙動とoffline条件を利用者へ説明する。
- Motoliiのtrust、互換、公式推薦を購入済みという事実から主張しない。

### 4.3 Kit

Kitは単なるeffect presetではなく、provider選択、型付き接続、初期値、公開control、assetを持つ作者成果であり、Kit自体も無料／有料、OSS／proprietaryの対象になり得る。

ただしKitが保持するのは、要求するVismのidentity、version条件、接続、初期値等である。購入記録、license code、account token、machine ID、決済URLを作品意味として抱えない。受け手には不足、非互換、外部取得の必要を示し、購入や認証そのものは外部providerへ渡す。

## 5. 既決と未決

| 主題 | 状態 |
|---|---|
| 第三者の持続可能な経済圏を長期目的に含める | **決定** |
| 無料／有料、OSS／proprietaryを技術的な身分へしない | **決定** |
| 買い切り、subscription、bundle、support等を一律に狭めない | **決定** |
| Motoliiが中央marketplace、決済、販売licenseを所有しない | **決定** |
| 外部商流を利用可能にする | **決定**（特定providerへの固定ではない） |
| openなHost／公開contractが第三者実装のsource licenseを強制しない | **決定** |
| package／artifact／catalog／Project Lockの形式 | **未決** |
| Demo／Full等のedition identity、互換、移行 | **未決** |
| entitlement／subscription／外部license serviceの接続protocol | **未決** |
| demo、parameter制限、watermarkの標準的な表現 | **未決** |
| proprietary artifactの検査、署名、失効、offline policy | **未決** |
| 具体的なstorefront推奨、手数料、収益分配 | **未決** |

## 6. 停止線

- この決定だけでmarketplace UI、購入button、license API、account、entitlement hookを実装しない。
- 「外部取得が必要」「取得先」「edition」等を、現行`NodeDesc`、manifest、catalog、Kit、Project Lockのfield／variantとして追加しない。
- 無料／有料、OSS／proprietaryをtrust、品質、互換、公式順位へ変換しない。
- First-partyだけの非公開API、優先resource、特別な検索枠を作らない。
- 有料配布を許すことから、常時network認証やHost内DRMを逆算しない。
- Project／Kitへsecret、購入記録、machine ID、決済provider固有payloadを保存しない。
- license／subscription確認をrender中のnetwork I/Oや隠れた可変stateとして持ち込まない。
- 取引状態で、解決済みの同じVism identity＋version＋入力の画を黙って変えない。期限切れ等で利用不能にする場合は評価前の明示的なunavailable診断とし、watermark、parameter制限、別実装へ暗黙に切り替えない。画が異なるeditionは別identity／versionとして解決できる必要があるが、その具体形式は未決とする。
- 外部service消失をProjectの意味消失にせず、identity、要求版、欠落理由を保持する。
- 海賊版対策の弱さを理由に商用作者を締め出さず、正規利用者の摩擦を無視してDRMを標準化もしない。
- 「経済圏を目指す」を、Motolii自身がmarketplaceへ参入する許可に読み替えない。

## 7. この決定が変えないもの

- v1完成条件へdynamic distribution、第三者SDK、marketplaceを追加しない。
- この決定だけで公開APIの安定保証、deprecation周期、互換保証を新設しない。有料化は未凍結契約の凍結を意味しない。
- Vism package、loader、trust、署名、sandbox、公開plugin UIの未決を埋めない。
- plugin純関数、VRAM常駐、Preview／Export同一評価、single writerを緩めない。
- 既存のCommunity distribution modelにある中央人気順位・中央dedupe拒否を変えない。
- 作者への報酬を、Motoliiが価格や事業の成功を保証する意味にしない。
