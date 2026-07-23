# INF-7g LLM plugin実演の価値回収（Unit 9E、2026-07-23）

状態: **処分完了**（1 historical blob、証明範囲を現行正本へ接続）

対象: `docs/reviews/2026-07-11-INF-7g-llm-plugin-demo.md`のcutoff唯一版`a719f2fb`。

関連: [INF-7g実演](2026-07-11-INF-7g-llm-plugin-demo.md)、[Creator / Developer連続体](2026-07-22-creator-developer-continuum-decision.md)、[plugin作者規約](../plugin-authoring.md)、[Vism実装計画](2026-07-17-vism-implementation-plan.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 回収する価値

INF-7gは、`new-plugin`型紙、descriptor検査、vendor API／panic deny、purity、pixel goldenを先に機械化した上で、LLMがOpacity Filterを実装し、人間からの差し戻し0回で全審判を通した実演記録である。

この一例が示すのは次までである。

1. 作者へ白紙を渡すより、規約準拠の初期状態を生成する方が往復を減らせる。
2. 「動いた気がする」をpurity負例とpixel oracleへ置換できる。
3. first-party実装をsource、scaffold、testと共に出すことが、creatorからauthorへ進む実行可能な教材になる。
4. 人海戦術の強さは無検査のcode量ではなく、多数の作者が同じ小さな公開境界と自動審判を使えることにある。

これは[Creator / Developer連続体](2026-07-22-creator-developer-continuum-decision.md)の成立証拠へ接続した。Opacityそのものの現行契約は外部first-party crateと現在のplugin testを正本とし、この歴史文書から再定義しない。

## 2. 証明していないこと

- 一つのFilter成功を、LayerSource、Composite、Simulation、UI component、Authoring Toolの自動生成成功へ一般化しない。
- 人間差し戻し0回を、主担当レビュー、反対側検収、契約判断が不要という運用規則にしない。
- 当時の`Filter数=3`、reference Tint型紙、registry配置を現行コード事実として復活させない。
- `scripts/new-plugin.sh`を第三者向けpackage generatorと呼ばない。現行はin-tree参照実装用で、外部作者scaffoldはVSM-A4で再入場する。
- conformance／purity／golden合格をprovenance、trust、permission、sandbox、署名、install、loader、互換、配布の代用にしない。
- Opacityの実演をVism packageまたはGitHub hostless配布の成功例と呼ばない。

## 3. 現行への反映

INF-7g本文へ「歴史的実演」と現在の停止線を追記し、Creator / Developer連続体へ証拠の強さを限定して接続した。これは「開発者と利用者の境界を薄くする」思想を弱める補正ではない。参加資格と学習摩擦を減らしながら、review、trust、作品の持続性をHost責任として残すための補正である。

専用path名で残っていたplugin／catalog／distribution系の未処分は本blobだけだった。root概念書、backlog、spec、README等に横断して現れる同主題は、その文書lineageの全体文脈を壊さないようUnit 4〜10で処分する。

## 4. 固定歴史出典とcoverage

`git log --all --reverse -- docs/reviews/2026-07-11-INF-7g-llm-plugin-demo.md`でcommit `42e4836b`の1版、blob `a719f2fb4428bbe7ae4b651236de1e012b82fe1a`だけを確認し全文を読んだ。

receiptは`evidence/historical-value-recovery/disposition-receipts/09e-llm-plugin-demo.tsv`を正本とする。cutoff総数1,797のうち処分済みは218、未処分は1,579である。
