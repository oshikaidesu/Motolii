# D1l current constructor／legacy lint lineageの価値回収（Unit 4L、2026-07-23）

状態: **停止線**（cutoff 4 historical blobの処分完了、Document意味は完了、lint enforcementはGAP-23で再開）

対象: `docs/reviews/2026-07-16-d1l-current-document-constructor-decision.md`全2版と`docs/reviews/2026-07-16-d1l-new-v1-lint-conflict-decision.md`全2版。

関連: [constructor決定](2026-07-16-d1l-current-document-constructor-decision.md)、[lint競合追補](2026-07-16-d1l-new-v1-lint-conflict-decision.md)、[semantic oracle境界](2026-07-17-d1i4-semantic-oracle-boundary-decision.md)、[D1l検収証拠回収](2026-07-23-historical-d1l-counter-review-evidence-recovery.md)、[M2仕様](../specs/M2-document-model.md)、[backlog](../backlog.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

このlineageは二つの判断を固定した。

1. 製品が新しいDocumentを作る入口は`Document::new_current()`一つ。`new_v1()`は正確なlegacy／migration fixtureとして残し、writer、prepare、Commandが暗黙に版を上げない。
2. legacy constructorの製品利用禁止をRust deprecation warningへ委ねない。公開だが`#[doc(hidden)]`とし、非test製品sourceからの呼出しをAST policyで機械拒否する。test／workspaceへ広いlint suppressionを置かない。

一つ目は現行v5へ一般化されて実装済みで、D1lのDocument、migration、Shared Effect意味を再び未完へ戻す理由はない。二つ目は決定文書がmainへ到達した一方、実コードが追随していない。現行`new_v1()`には`#[deprecated]`が残り、22 integration testと3 source test module、合計25箇所に`allow(deprecated)`がある。`cargo clippy --workspace --all-targets -- -D warnings`が緑でも、禁止したsuppression込みで通るだけなので契約適合の証拠にならない。

この差分をGAP-23として狭く再開する。保存形式、Effect lifecycle、journal、Undo、pixel意味は変更せず、M2再締結全体を遡及撤回しない。

## 2. 4版で起きた判断の変化

### 2.1 初版: raw採番口削除後の到達不能を閉じる

D1lは公開`allocate_effect_*`を廃し、Writer prepareだけがIDを決める契約にした。しかし当時のAPIには`new_v1()`しかなく、候補実装はraw allocatorの副作用でversion／minimumを上げていた。その副作用を消すと、新規projectがD1l前提へ到達できない。

初版は`new_current()`を唯一の製品入口にし、旧文書はD1eで明示migration、`DocumentWriter::new`は渡されたDocumentを勝手にupgradeしない、と分離した。`new_v1()`の中身をcurrentへ変えたり、version fieldだけ直書きしたinline／hybrid文書を受理したりする案を棄却した。

当時のcurrentはEffect Definitionを導入したv4だった。後続D1j camera migrationがv5を導入した現在、この数値は歴史値である。生きる規範は、`new_current()`がその時点のwriterと必須minimumを満たし、version更新ごとに独立したschema／migration審判を通ることである。

### 2.2 第二版: deprecatedをdoc-hidden＋ASTへ置換

初版は`new_v1()`へ`#[deprecated]`を付ける案だった。しかし`--all-targets -D warnings`、legacy fixtureでの正当な呼出し、当時のwhole-file semantic保護は同時に満たせない。test側へ`allow(deprecated)`を置けばlintは緑になるが、禁止を呼出し側の抑制で消せてしまう。

constructor第二版と独立lint決定は、deprecationを棄却し、`#[doc(hidden)]`＋AST policyへ置換した。製品sourceは0件、migration／testの閉じた範囲だけを許し、global suppression、`cfg(clippy)`、constructor alias、legacy fixtureの`new_current()`置換を拒否した。これはDocument wireやconstructorの返り値を変えない enforcement 修正である。

### 2.3 最終版: oracleとharnessを分ける

後発D1i-4訂正は「semantic Rust file全体をbyte不変」から、「期待値oracleを不変、API／fixture／runtime配線のharnessは変更可」へ保護対象を正した。よってsuppression除去のためにharnessを触ること自体は禁止ではない。ただし期待値、分類、goldenを修復に合わせて弱めてはならず、whole-file分類からoracle分離が必要なpathはD1i-4の明示migration手順を通す。

## 3. 現行コードと文書の照合

| 面 | 現在地 |
|---|---|
| current生成 | `new_current()`がwriter 5／camera minimum 5のDocumentを作り、read-write roundtrip試験が緑 |
| legacy生成 | `new_v1()`はversion 1／minimum 1のまま |
| 暗黙migration | `DocumentWriter::new`はvalidateのみ。saveは旧版を`SaveRequiresMigration`で拒否。lifecycle prepare／Commandもcurrent契約以外を型付き拒否 |
| AST policy | 非test `crates/**/src`の直接`Document::new_v1()`呼出しと負例を検査し、現行試験16件は緑 |
| lint属性 | 決定に反して`new_v1()`が`#[deprecated]`、`#[doc(hidden)]`なし |
| suppression | `allow(deprecated)`が25箇所。保護対象だったBlendMode／LookAt・Follow harnessにも残る |
| 文書 | M2 D1l行がcurrentをv4、doc-hidden実装済み、suppressionなしとしており、現行v5とlive codeに不一致 |

AST policyの現行実装は`crates/`だけを走査し、Cargo workspace全体やimport aliasを完全には証明しない。これはdecisionの「非test workspace sourceから0件」という主張を現在の試験だけで外向き保証できない非証明範囲である。GAP-23ではscannerの実測範囲を閉じるか、保証文を実装範囲へ縮小するまで完了扱いにしない。

型名`EffectLifecycleRequiresV4Document`やfixture名にv4が残るが、実guardはcurrent writer 5／minimum 5を要求する。公開error variantのrenameはAPI変更なのでlint修復へ便乗させない。

## 4. GAP-23の再開境界

### 4.1 必須修復

- `new_v1()`の`#[deprecated]`を外し、`#[doc(hidden)]`とlegacy-only source documentationへ置換する。
- 25箇所の`allow(deprecated)`を除去する。期待値、fixture意味、Document versionを変更しない。
- AST policyの正例／負例を維持し、製品sourceからのlegacy constructor利用を機械拒否する。
- golden policy、clippy、`motolii-doc`、workspace全試験をsuppressionなしで通す。

### 4.2 STOP

- whole-file semantic／provisional分類に触れる必要があれば、先にD1i-4のoracle分離または分類移行を独立判断する。classification削除やmarkerで迂回しない。
- AST gateをworkspace全体へ広げる際に公開API、Document schema、journal wireの変更が必要に見えたら停止する。
- `new_v1()`を削除・非公開化・current化したり、別constructor aliasを追加したりしない。
- `EffectLifecycleRequiresV4Document`等の公開名変更を同じ修復へ混ぜない。

scannerのworkspace member列挙やalias検出は補強候補だが、雑な文字列走査や誤検出回避のallowlist増殖で閉じない。現行AST judgeを保ち、非証明範囲をfixtureで再現してから実装境界を決める。

## 5. 再利用する設計原則

- 公開APIを減らすときは、削除後にも正規の成功経路が到達可能かを同時に検査する。
- legacy fixtureの正確さと、製品がそれを使わない保証を別にする。
- versionはconstructor、writer、prepare、Commandの副作用で暗黙に上げず、明示migrationだけで進める。
- warning抑制込みのgreen lintを、禁止機構の成立証拠にしない。
- 作品意味のoracleとAPI配線harnessを分け、enforcement修復のために期待値を書き換えない。
- 後続schema versionが上がったら、歴史文書のliteral versionを現行定数へ読み替えない。

## 6. 復活させないもの

- `new_v1()`のversion／minimumをcurrentへ変えること。
- `DocumentWriter::new`、prepare、Command applyが暗黙migrationすること。
- 公開raw allocatorやfield直書きを版上げ口へ戻すこと。
- deprecation warning、global allow、testごとのsuppressionを製品利用禁止の正本にすること。
- legacy fixtureを意味確認なしに`new_current()`へ置換すること。
- semantic oracle、classification、golden policyをlint修復へ合わせて弱めること。
- v4という歴史値や型名を理由に、現行Documentをv4へ戻すこと。
- clippy／workspace greenだけでGAP-23完了とすること。

## 7. 固定歴史出典とcoverage

constructorの初版を全文で読み、lint置換版との差分を確認した。lint決定も初版を全文で読み、D1i-4追補版との差分を確認した。処分した4 unique blob（20,255 bytes）の完全SHAは`evidence/historical-value-recovery/disposition-receipts/04l-d1l-constructor-lint.tsv`を正本とする。cutoff総数1,797のうち処分済みは329、未処分は1,468である。
