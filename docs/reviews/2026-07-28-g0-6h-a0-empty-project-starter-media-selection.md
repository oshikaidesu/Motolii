# G0-6H-A0 empty-project + Starter Media裁定の受領と契約粒選定

- 日付: 2026-07-28
- 状態: **決定**
- G0-6H-A0: **DONE**

## 目的

`G0-6H-M`が返した人間裁定一点について、ユーザー本人の採択と追加説明を
限定して受領し、実装より先にscenario / fixture契約を閉じる次の一粒だけを選定する。

## 受領した人間裁定

- `G0-6H-M`の選択肢(a)、現行routeにempty-project scenarioを新設する案を採択する。
- Projectは空とし、Project assets、Stage、Inspector、Timelineに作品内容を残さない。
- Browserは表示し、Projectとは別のローカルfixture用`Starter Media`を参照できる。
- `Starter Media`は静止画、短い動画、音声、SVG等のsample mediaを含められる。
- npmは例示であり、sample mediaの取得元または生成手段をnpmへ限定しない。
- capture時に外部networkへ依存せず、固定されたローカルbyteとprovenanceで再現する方向を採る。

## 次の一粒

docs-only `G0-6H-A`を選定する。`G0-6H-A`は、上記裁定を既存Browserの
Project / Registered folders分離、fixture所有、offline再現、provenance、負例へ落とす
scenario / fixture契約だけを閉じる。

素材byteの生成、asset path、manifest schema、route / query shape、React接続、画像capture、
variant生成、人間審判は後続へ分離する。

## 非目標

- sample mediaの生成・取得・追加。
- 外部素材、npm package、codec、生成toolの採択。
- asset path、manifest schema、route / query、adapter、公開APIの決定。
- React / CSS / Rust / fixture / test / guard / JSON / script / 画像の変更。
- Document、Project asset、Registered folder、plugin、永続形式の公開契約変更。
- `G0-6H-V0`、`G0-6H`、`CU-0B01`、`U0e-3`の完了または解禁。

## handoff

| ID | 状態 | 内容 |
|---|---|---|
| `G0-6H-M` | **DONE** | 前提。screen 1のsemantic gapと人間裁定一点を返した |
| `G0-6H-A0` | **DONE** | 本粒。選択肢(a)とStarter Media方向を限定受領した |
| `G0-6H-A` | **DO** | docs-only scenario / fixture契約 |
| `G0-6H-V0` | **WAIT** | `G0-6H-A`完了後に再判定する |
