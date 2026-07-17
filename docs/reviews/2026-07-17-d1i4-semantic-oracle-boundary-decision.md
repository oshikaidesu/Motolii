# D1i-4 / S16: semantic oracle 保護境界の訂正

Status: **Decision amendment — gate migration before further protected-harness repair**.

## 問題

S16が固定するのは、既存variantの画素・幾何・写像の**意味**である。ところがD1i-4の初期実装は、期待値と、それを現行公開APIへ接続するtest harnessを同じRustファイルへ置き、そのファイル全体を`semantic`として凍結した。

この近似は、意味を変えないAPI移行まで拒否する。実例として`d1i3_blend_mode.rs`は、`BlendMode -> CompositeMode`対応とpremultiplied alpha期待値を変えずに、A0Sで確定した`PluginRuntime`をgraphへ渡す必要がある。旧`PluginRegistry`呼び出しをbyte-for-byte保持することと、互換aliasなしでA0S公開signatureへ移ることは両立しない。

これはS16の要求ではない。S16の原決定は「画素意味を変えるPRは既存ゴールデンを更新できない。新variant+新ゴールデンを追加する」であり、import、fixture構築、runtime取得、公開API呼び出しまで永久固定するとは決めていない。

## 決定

D1i-4の保護単位を、test harness全体から**semantic oracle artifact**へ訂正する。

- `semantic oracle`: variant名、入力、期待される写像・数値・bytesなど、既存variantの意味を判定する最小データ。既存artifactの変更・削除は禁止する。
- `harness`: oracleを読み、現行公開APIで実装を評価し、actualとoracleを比較するコード。API、型、fixture構築、runtime配線に追従して変更できる。
- harness内へ期待値を複製しない。新しい保護対象を追加するときは、先にoracle artifactへ期待値を置き、harnessはそれを読む。
- 既存variantの意味変更は従来どおり禁止する。変更が必要なら新variantと新oracle artifactを追加する。
- provisional artifactの`MOTOLII_REGENERATE_WHEN`規則は変更しない。

「Rust testファイルだから保護しない」「特定PRだけ許可する」というpath例外にはしない。期待値の所在と実行配線の所在を構造で分離する。

## 既存分類の移行

一括declassificationはしない。各既存semantic harnessを次の1チケット単位で移行する。

1. 現在の期待値を独立oracle artifactへ、値を変えずに転記する。
2. harnessをoracle読込へ変更し、同じactualを比較する。
3. `classification.tsv`のsemantic行を旧harnessから新oracleへ置換する。
4. gateは、明示された旧harness→新oracleの移行についてだけ、同一差分内で新oracleが追加・semantic登録され、旧harnessが残る場合に限り分類置換を許可する。
5. gateの負例で、oracle変更・削除、移行先未登録、旧harness削除、単なるdeclassificationを拒否する。

最初の移行対象は`d1i3_blend_mode.rs`とする。A0S runtime配線を必要とする現在の実衝突を解消し、残るPathOp / LookAt・Follow / Bezier / Transform合成は、それぞれのAPI変更が必要になる前に別チケットで同じ形へ移す。

## 非目標

- 既存BlendMode、合成式、丸め、serde名、CompositeMode対応の変更
- semantic oracleのregenerate許可
- 管理者override、CI skip、PR番号やbranch名に依存する例外
- `PluginRegistry`互換aliasや、`PluginRuntime`境界の緩和
- 全semantic harnessの一括書き換え

## 完了条件

### 決定チケット

- 本文書とM2のD1i-4記述が、保護単位をsemantic oracleと定義する。
- D1lの旧「semantic test file byte-for-byte不変」は、この決定によりharnessとoracleへ読み替えられる。
- コード・分類・期待値は変更しない。

### 最初のgate移行チケット

- BlendModeのvariant閉集合、`BlendMode -> CompositeMode`対応、premul入力と期待出力が独立oracleに存在する。
- harnessは期待値を直書きせずoracleを読み、A0Sの`PluginRuntime`signatureで同じ判定を行う。
- 既存oracleの変更・削除は失敗し、harness-onlyのruntime配線変更は成功する。
- 旧harnessの単純な分類解除、移行先未登録、旧harness削除は失敗する。
- `./scripts/check-golden-update-policy.sh origin/main`
- `cargo test -p motolii-testkit --test golden_update_policy`
- `cargo test -p motolii-doc --test d1i3_blend_mode`
- `cargo test --workspace`

## 帰結

D1i-4は弱くならない。固定対象が「その時点のテスト実装」から「将来も守る作品意味」へ狭く正確になる。Host・catalog・runtime等の公開境界は進化でき、既存variantの見た目は引き続き機械的に固定される。
