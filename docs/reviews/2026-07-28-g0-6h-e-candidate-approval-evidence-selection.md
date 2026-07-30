# G0-6H-E0 現行候補5画面承認証拠の取込選定

- 日付: 2026-07-28
- 状態: **決定**
- G0-6H-E0: **DONE**

## 1. 目的

2026-07-28にユーザーが明示承認した現行`#plugin-browser-candidate`の5画面を、
旧U0e-2 generation全体の承認へ拡張せず記録するdocs-only粒`G0-6H-E`を選定する。
本粒は`G0-6H`の人間審判を完了させず、審判対象routeも変更しない。

## 2. 確認した事実

- ユーザーが閲覧・承認したのは、固定commit
  `56c318edcddab7cf95d263cc2f7dd2b4e6791134`由来の現行
  `#plugin-browser-candidate`を1440×900、dark、normal色で撮影した5画面である。
- 5状態はmixed Timeline、Browser検索0件、Interval Easing、Hand、Relative Moveであり、
  全画面にBrowser / Stage / Inspector / Timelineが含まれる。
- ユーザーは旧`#reference/*`画面、およびgeneration
  `u0e2-08f96cbd7754-85c0fc529ab1`のlightness / grayscale / CVD派生25枚を
  今回の確認では閲覧・承認していない。
- `npm run check-reference`は現行treeで
  `reference generation OK: u0e2-08f96cbd7754-85c0fc529ab1 (30 PNGs)`を返した。
  これはread-only再現証拠であり、人間の視覚承認ではない。
- [reference handoff](../mocks-ui/reference-handoff.md)のDecision templateとchecklistは
  未記入であり、`G0-6H`は`DO / HUMAN`、`CU-0B01`は`HUMAN / WAIT`のままである。

## 3. 次粒が閉じる一成果

`G0-6H-E`は、今回の承認を「現行統合モックのnormal色5画面への肯定的応答」として
証拠台帳・observation・既存handoffの非充足注記へ記録する。旧generationへの採否、
具体token値、製品theme、U0e-3解禁は記録しない。

## 4. 非目標

- `reference-handoff.md`のDecision templateまたはchecklistを埋める。
- `G0-6H`、`CU-0B01`、`CU-0B02`、`U0e-3`の状態を変更する。
- 審判対象を`#reference/*`から`#plugin-browser-candidate`へ変更する。
- 画像、generation、`CURRENT`、React / CSS / Rust / fixture / test / guardを変更する。
- grayscale / CVD / Light / 高コントラスト / UI scaleの合否を推測する。
- 色、spacing、radius、font、icon等の具体tokenを採択する。

## 5. 必須負例

- 今回の5画面承認を旧30 PNGの承認として書く。
- 自動`check-reference`成功を人間審判の代替にする。
- 閲覧環境の未取得項目を推測で埋める。
- 画像をmanifestなしでリポジトリへ追加する。
- `G0-6H-E`自身を同じ差分で`DONE`にする。
- 次のroute裁定、派生画像供給、U0e-3実装を同じ粒へ束ねる。

## 6. STOP条件

1. 承認対象の5画面、route、viewport、色条件を一意に記録できない。
2. 旧U0e-2 generationの採否を書かないと文書が閉じない。
3. 審判対象route、具体token、公開API、Document、plugin契約の変更が必要になる。
4. 許可文書外の画像、React / CSS、fixture、test、guardへ変更が必要になる。

## 7. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `G0-6H-E0` | **DONE** | 今回の承認を限定記録するdocs-only粒を選定 |
| `G0-6H-E` | **DO** | 現行候補5画面への肯定的応答だけを証拠として取込 |
| `G0-6H` | **DO / HUMAN** | 旧generationを含む人間審判は未完了 |
| `CU-0B01` | **HUMAN / WAIT** | 据え置き |
| `U0e-3` | **WAIT** | 据え置き |
