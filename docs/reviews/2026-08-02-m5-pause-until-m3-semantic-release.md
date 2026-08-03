# M5休止・M3意味開放契約

状態: **決定／M5製品runtime休止**（2026-08-02）

M5の既知実装採択、decision recovery、private fixtureは保持する。一方、M5の製品runtime、公開schema、
provider接続、GPU resource接続は、M3の製品意味が開放されるまで開始しない。これはM5を放棄する停止ではなく、
先行するM3の共有writer・snapshot・Stage・Preview／Export意味を再発明せずに使うための順序契約である。

## 開放判定はIDではなく意味論で行う

M3のチケット番号、枝番、commit数、テスト件数は開放条件ではない。IDは証跡を検索する索引に留め、次の意味が
すべて現行正本・実コード・受入証拠で一致した時だけ `M5 PRODUCT RUNTIME: OPEN` とする。

| 意味の境界 | 開放に必要な状態 | M5へ渡すもの |
|---|---|---|
| Document／writer | Documentの唯一writer、typed intent→D2、journal／Undo、revisionとsnapshotの所有者が一つで、失敗時に部分変更を残さない | M5 object／camera／material変更を第二のwriterなしで接続できること |
| 通常製品route | 通常の製品画面から対象を作成・選択・編集し、同じstable identityがStage／Timeline／Inspectorへ投影される。mock／diagnostic画面を製品routeの代替にしない | M5の3D素材・camera・診断を既存routeへ投影するconsumer席 |
| 視覚・出力 | canonical world、camera、FrameDesc、Qualityの意味が正本化され、Preview／Exportが同じ評価関数を使う。Stage presentationがcanonical outputを汚染しない | M5 Observation／depth／rendererが別world・別camera・別export経路を作らないこと |
| 受入・統合 | 上記の意味が独立受入で確認され、mainへ統合された現行codeとdocsが同じ状態を示す。未完了のowner境界・公開契約・受入oracleを残さない | M5を隔離probeから製品接続へ再分類する許可 |

一項目でも未成立なら、M5は `PREPARED / WAIT` のままとする。M3の「全IDが完了した」ことや、private fixtureの
テスト緑だけを開放根拠にしない。

## 休止中に許可すること

- 既存M5の採択地図、decision recovery、private fixture、receiptの保守
- M3意味との整合を確認するread-only監査、docsの観察・停止線の更新
- 既存fixtureを製品API・Document・plugin・resource ownerへ昇格させない範囲の再現性確認

## 休止中に禁止すること

- M5 Observation公開型、provider registry、Document／serde／wire、migrationの追加
- M5 3D importer、material／renderer、depth pass、GPU readback、picking、Duplicator runtimeの製品接続
- M4 K1aのResourceLedger／hard budget APIやbackend型をM5のために先行発明すること
- M3の未解決意味を、M5のprivate実装・crate型・ID名から推測して埋めること

## 再開順序

M3意味開放後、共有resourceの所有・hard budget・admissionが意味とoracleを持つことを確認し、次にM5-C0の公開
schema／provider identity／migration decisionを別契約で閉じる。その後にだけ、private fixtureを製品conformanceへ
再利用するC0 runtime、続いてimport／render／depthの薄い接続へ進む。M5の既存技術採択を再選定しない。
