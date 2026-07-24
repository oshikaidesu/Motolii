# レビュー文書の規律(2026-07-12制定)

このディレクトリの調査・レビュー・ゲート文書、および以後の調査・仕様レビューに適用する継続規律。制定の経緯: 2026-07-12の先例調査2本([考慮漏れ調査](2026-07-12-prior-art-gap-survey.md)・[成功先例](2026-07-12-success-prior-art.md))がいずれも独立レビュー([反対側レビュー](2026-07-12-prior-art-gap-counter-review.md)・同日の批判レビュー7点)で過剰結論・帰属誤り・審判不一致を指摘され、全面改訂に至ったこと。

## 規律6点

1. **調査文書の結論をそのまま設計根拠にしない**
2. **独立した反対側レビューで再判定する** — 事実(一次資料で確認できるか)・転移条件(同じ失敗条件がこのプロジェクトにあるか)・因果(効果の帰属は正しいか)・より小さい対策(境界を公開しない/ホスト側に閉じる選択はないか)
3. **反例未探索なら「仮説と整合する事例」に留める** — 「裏付けられた」「証明された」を書かない
4. **公約は保証意味を分解し、対応する審判セットと有効化条件が揃うまで外向き化しない**
5. **機能正当性・互換性・供給網信頼・安全性を別々に評価する** — 「機械検証可能」に畳み込まない
6. **元調査と反対側レビューを必ず併読する** — ゲート・仕様へ採用する時は判定語(採用/縮小/延期/棄却)を併記する

## 運用注

- 出典は**再確認可能な公開恒久文書**(公式仕様・RFC・公式ブログ・学会誌・バグトラッカ)に限定する。調査ワークフローの「検証済み」申告や、出典URLの無い歴史詳細を根拠にしない
- 判断が割れたら「**ユーザーデータまたは公開契約へ不可逆に焼くかどうか**」で決める。焼かない選択が可能なら、v1では小さい方を選ぶ(反対側レビューの判定基準)
- LLM能力への言及は日付を添える。能力仮定は契約・スキーマ・ゲートに焼かず、日付+見直しトリガー付きで運用文書にのみ書く

## 登録規則(2026-07-19制定)

制定の経緯: 入口台帳([docs/README.md](../README.md))のファイルマップから36件のreview文書が欠落し、既決事項(例: [AM式高度イージング=区間補間の非破壊差し替え、2026-07-10決定](../concept.md))が後続作業から逆引きできず、モック・仕様に旧仕様が混在した。

1. **新しいreview文書を作ったら、同じ変更で下の全文書索引に1行追加する**。入口台帳のファイルマップは「現役で参照される文書」の抜粋であり、全量はこの索引が正本
2. **ユーザー決定・採否・撤回・未統一を含む文書は、[決定逆引き台帳](../decision-index.md)にも主題キーワードつきで1行登録する**。会話・commit履歴・エージェントセッションにしか残らない決定を作らない
3. 状態語彙は固定集合とする: **決定 / 縮小採用 / 延期 / 棄却 / 撤回 / 未統一 / 観察 / 比較中 / 停止線**。この語彙の外の状態表現を新設しない(必要なら本規則を先に改訂する)
4. `scripts/check-docs.sh` が索引の抜け・入口台帳の重複掲載・ローカルリンク切れ・状態語彙を機械検証する。docsを触る変更では実行してから終える

## 全文書索引

各文書の1行要旨と状態は文書冒頭が正本。ここはファイル名と表題のみ(抜け検出用の全量索引)。

| ファイル | 表題 |
|---|---|
| [2026-07-09-R1-export-review.md](2026-07-09-R1-export-review.md) | コードレビュー所見 2026-07-09 (R1/Quality・export・cli周辺) |
| [2026-07-09-R3-datatrack-review.md](2026-07-09-R3-datatrack-review.md) | コードレビュー所見 2026-07-09 (R3/DataTrack統合) |
| [2026-07-10-M1-plugin-boundary-review.md](2026-07-10-M1-plugin-boundary-review.md) | 設計レビュー所見 2026-07-10 (M1完了後・プラグイン境界の凍結前監査) |
| [2026-07-10-R8-vello-review.md](2026-07-10-R8-vello-review.md) | 軽量レビュー 2026-07-10 (R8/Vello採否スパイク) |
| [2026-07-10-R9-real-material-checklist.md](2026-07-10-R9-real-material-checklist.md) | R9 実素材検証チェックリスト (T11) |
| [2026-07-10-freeze-gate-declaration.md](2026-07-10-freeze-gate-declaration.md) | 凍結ゲート宣言(2026-07-10) |
| [2026-07-10-freeze-gate-remaining.md](2026-07-10-freeze-gate-remaining.md) | 凍結ゲート残件(2026-07-10 監査) |
| [2026-07-11-INF-7g-llm-plugin-demo.md](2026-07-11-INF-7g-llm-plugin-demo.md) | INF-7g: LLMプラグイン実演記録(2026-07-11) |
| [2026-07-11-M2-entry-gate.md](2026-07-11-M2-entry-gate.md) | M2入場条件(2026-07-11。同日改訂: ゲート運用レビュー7点を反映) |
| [2026-07-11-code-audit-pre-m2.md](2026-07-11-code-audit-pre-m2.md) | 実コード監査: M2並列解禁前に詰めるべき設計箇所(2026-07-11) |
| [2026-07-12-M2E-2-ruleset-activation.md](2026-07-12-M2E-2-ruleset-activation.md) | M2E-2 ruleset 有効化ログ |
| [2026-07-12-M2E-7-render-ctx-thaw.md](2026-07-12-M2E-7-render-ctx-thaw.md) | M2E-7 解凍手続き: Filter/Compositeへ`RenderCtx`を導入する |
| [2026-07-12-M3-M4-gate-ledger.md](2026-07-12-M3-M4-gate-ledger.md) | 次フェーズ入場条件の候補台帳: M3/M4(2026-07-12) |
| [2026-07-12-code-audit-2nd-d1.md](2026-07-12-code-audit-2nd-d1.md) | 第二実コード監査の裏取りと台帳化: D1系スキーマ・評価・永続(2026-07-12) |
| [2026-07-12-d1-spec-holes-prior-art.md](2026-07-12-d1-spec-holes-prior-art.md) | D1スキーマ未決点の先例調査メモ(2026-07-12) |
| [2026-07-12-m2-permanence-prevention.md](2026-07-12-m2-permanence-prevention.md) | M2恒久焼き込みの予防(2026-07-12) |
| [2026-07-12-pathop-ae-cavalry-comparison.md](2026-07-12-pathop-ae-cavalry-comparison.md) | PathOp語彙比較: AE/Lottie × Cavalry(2026-07-12) |
| [2026-07-12-plugin-ui-v1-boundary.md](2026-07-12-plugin-ui-v1-boundary.md) | 決定: v1プラグインUI境界は`NodeDesc`自動生成のみ(2026-07-12) |
| [2026-07-12-prior-art-gap-counter-review.md](2026-07-12-prior-art-gap-counter-review.md) | 反対側レビュー: M3/プラグイン生態系の先例所見を最小化する(2026-07-12) |
| [2026-07-12-prior-art-gap-survey.md](2026-07-12-prior-art-gap-survey.md) | 先例調査: M3/プラグイン生態系の考慮漏れ(2026-07-12) |
| [2026-07-12-rework-prior-art.md](2026-07-12-rework-prior-art.md) | 出戻り: 先人の失敗後対応と、その反面(予防)(2026-07-12) |
| [2026-07-12-success-prior-art.md](2026-07-12-success-prior-art.md) | 先例調査: 成功先例からの仮説メモ(2026-07-12) |
| [2026-07-12-vertical-text-prior-art-counter-review.md](2026-07-12-vertical-text-prior-art-counter-review.md) | 反対側レビュー: 縦書き先例調査の再判定(2026-07-12) |
| [2026-07-12-vertical-text-prior-art.md](2026-07-12-vertical-text-prior-art.md) | 先例調査: 縦書き(日本語縦組み)テキストレイアウトの既存実装分解(2026-07-12) |
| [2026-07-13-decision-pack-adoption.md](2026-07-13-decision-pack-adoption.md) | 決定パック採択(2026-07-13ユーザー承認) |
| [2026-07-13-readback-pipelining-prior-art.md](2026-07-13-readback-pipelining-prior-art.md) | 先例調査: GPU→CPUリードバック重畳とcold shader compileの解決例(2026-07-13) |
| [2026-07-13-undecided-critical-path-confirm.md](2026-07-13-undecided-critical-path-confirm.md) | 友人レビュー確認: 未決事項とクリティカルパス(2026-07-13) |
| [2026-07-13-wgpu-challenges-counter-review.md](2026-07-13-wgpu-challenges-counter-review.md) | 反対側レビュー: Rust+wgpu技術的課題調査の二重補正(2026-07-13) |
| [2026-07-14-3d-depth-boundary-prior-art.md](2026-07-14-3d-depth-boundary-prior-art.md) | 先例調査: 「2Dレイヤー順合成×3D深度合成」の境界の切り方(2026-07-14) |
| [2026-07-14-3d-depth-scope-design.md](2026-07-14-3d-depth-scope-design.md) | 2Dレイヤー順と3D深度合成の境界設計(2026-07-14) |
| [2026-07-14-audio-generalization-design.md](2026-07-14-audio-generalization-design.md) | 音声を「楽曲1本」から一般メディアへ拡張する設計(2026-07-14) |
| [2026-07-14-color-conversion-prior-art.md](2026-07-14-color-conversion-prior-art.md) | 色変換(プレビュー/書き出し不一致)の既知解調査メモ(2026-07-14) |
| [2026-07-14-d5-transport-prior-art.md](2026-07-14-d5-transport-prior-art.md) | 先例調査: D5 Transport低速時戦略(2026-07-14) |
| [2026-07-14-m2-core-closure.md](2026-07-14-m2-core-closure.md) | M2コア締結宣言(撤回済み) |
| [2026-07-14-m2-exit-param-pipeline-disposition.md](2026-07-14-m2-exit-param-pipeline-disposition.md) | M2終了前判定 — Param Pipelineと操作単純化の持ち越し境界 |
| [2026-07-14-m3-ui-boundary-counter-review.md](2026-07-14-m3-ui-boundary-counter-review.md) | 反対側レビュー: M3 UI境界規約を実装可能な最小形へ縮小する(2026-07-14) |
| [2026-07-14-m3-ui-boundary-prevention.md](2026-07-14-m3-ui-boundary-prevention.md) | M3 UI境界汚染の予防(2026-07-14) |
| [2026-07-14-motion-foundation-known-tech-disposition.md](2026-07-14-motion-foundation-known-tech-disposition.md) | Motion基盤候補の既知技術による処分決定(2026-07-14) |
| [2026-07-14-motion-tools-praise-diy-gap-audit.md](2026-07-14-motion-tools-praise-diy-gap-audit.md) | モーショングラフィック4ツール 称賛・日曜大工・根本ギャップ監査 |
| [2026-07-14-recent-concept-propagation-audit.md](2026-07-14-recent-concept-propagation-audit.md) | 直近コンセプトの全層反映監査(2026-07-14) |
| [2026-07-14-repeated-wheel-standardization-audit.md](2026-07-14-repeated-wheel-standardization-audit.md) | AE反復再発明プラグイン標準化監査(2026-07-14) |
| [2026-07-14-unified-stage-camera-design.md](2026-07-14-unified-stage-camera-design.md) | Stage / Output Frame / 統一カメラ設計(2026-07-14) |
| [2026-07-15-d1l-copylocal-remint-counter-review.md](2026-07-15-d1l-copylocal-remint-counter-review.md) | D1l Copy Local内部ID契約 — 反対側レビューと採否 |
| [2026-07-15-d1l-journal-revert-boundary-counter-review.md](2026-07-15-d1l-journal-revert-boundary-counter-review.md) | D1l journal/Undo/Writer追補 — 反対側レビューと採否 |
| [2026-07-15-d1l-journal-revert-boundary-decision.md](2026-07-15-d1l-journal-revert-boundary-decision.md) | D1l journal互換・Undo等価・Writer採番境界 — 追補決定 |
| [2026-07-15-implementation-readiness-ledger.md](2026-07-15-implementation-readiness-ledger.md) | Relative / Stage / Shared Effect / Bounds / Duplicator 実装準備台帳(2026-07-15) |
| [2026-07-15-m2-foundation-reclosure-counter-review.md](2026-07-15-m2-foundation-reclosure-counter-review.md) | M2基盤再締結ゲート 反対側レビュー(2026-07-15) |
| [2026-07-15-m2-foundation-reclosure-gate.md](2026-07-15-m2-foundation-reclosure-gate.md) | M2基盤再締結ゲート(2026-07-15) |
| [2026-07-15-p5-generative-pattern-disposition.md](2026-07-15-p5-generative-pattern-disposition.md) | p5.js系ジェネラティブ表現の分類とMotoliiへの配置 |
| [2026-07-15-prior-art-complaint-boundary-audit.md](2026-07-15-prior-art-complaint-boundary-audit.md) | 先例収束 / 日曜大工境界監査(2026-07-15) |
| [2026-07-15-relative-scope-duplicator-decision.md](2026-07-15-relative-scope-duplicator-decision.md) | Relative Move / Timeline Effect Link / Duplicator決定(2026-07-15) |
| [2026-07-15-shared-effect-lifecycle-decision.md](2026-07-15-shared-effect-lifecycle-decision.md) | Shared Effect lifecycle決定(GAP-14 / D1l実装ゲート) |
| [2026-07-16-ae-layer-system-disposition.md](2026-07-16-ae-layer-system-disposition.md) | AEレイヤー方式への処置台帳と出戻り一次声調査 |
| [2026-07-16-d1l-current-document-constructor-counter-review.md](2026-07-16-d1l-current-document-constructor-counter-review.md) | D1l新規Document v4生成契約 — 反対側レビューと採否 |
| [2026-07-16-d1l-current-document-constructor-decision.md](2026-07-16-d1l-current-document-constructor-decision.md) | D1l新規Documentのv4到達境界 — 追補決定 |
| [2026-07-16-d1l-new-v1-lint-conflict-decision.md](2026-07-16-d1l-new-v1-lint-conflict-decision.md) | D1l `new_v1` lintとprotected semantic testの矛盾解消決定(2026-07-16) |
| [2026-07-16-m2-comp-camera-decision.md](2026-07-16-m2-comp-camera-decision.md) | M2 CompCamera決定 — planar v1、空間は追加的拡張(2026-07-16) |
| [2026-07-16-m2-param-element-constraint-disposition.md](2026-07-16-m2-param-element-constraint-disposition.md) | M2 Param Pipeline / Element Domain / Constraint Graph処分(2026-07-16) |
| [2026-07-16-m2-project-sidecar-session-decision.md](2026-07-16-m2-project-sidecar-session-decision.md) | M2 project sidecar identity / session所有決定(2026-07-16) |
| [2026-07-16-m3-preflight-decisions.md](2026-07-16-m3-preflight-decisions.md) | M3着手前決定 — 操作の意味を固定し、見た目の実値は測って決める |
| [2026-07-16-m3-ui-concept-to-tickets.md](2026-07-16-m3-ui-concept-to-tickets.md) | M3 UIコンセプトから実装チケットへの分解 |
| [2026-07-16-m3-ui-gap-survey.md](2026-07-16-m3-ui-gap-survey.md) | M3前UIギャップ調査: U1〜U8に席が無いUI要素とコア側前提の欠落(2026-07-16) |
| [2026-07-16-m3-ui-rapid-acceptance-prior-art.md](2026-07-16-m3-ui-rapid-acceptance-prior-art.md) | 先例調査: すぐに受け入れられたUI(2026-07-16) |
| [2026-07-16-media-portability-gpu-resurvey-plan.md](2026-07-16-media-portability-gpu-resurvey-plan.md) | 再調査ラウンド起案: メディア可搬性(GAP-3/7)とGPUベンダ差(INF-3)(2026-07-16) |
| [2026-07-16-ui-update-forensics.md](2026-07-16-ui-update-forensics.md) | UIアップデート考古学 — 改善履歴から潜在的な失敗を読む |
| [2026-07-17-aviutl2-comment-voices.md](2026-07-17-aviutl2-comment-voices.md) | AviUtl2動画コメント欄 — 統一できない利用者の声 |
| [2026-07-17-d1i4-semantic-oracle-boundary-decision.md](2026-07-17-d1i4-semantic-oracle-boundary-decision.md) | D1i-4 / S16: semantic oracle 保護境界の訂正 |
| [2026-07-17-extensible-core-prior-art-translation.md](2026-07-17-extensible-core-prior-art-translation.md) | 個体性・介入・上限・縮退・遊びの先例翻訳(2026-07-17) |
| [2026-07-17-non-video-workspace-asset-ui-prior-art.md](2026-07-17-non-video-workspace-asset-ui-prior-art.md) | 動画ソフト外から引き直すWorkspace・素材探索・視線設計 |
| [2026-07-17-vism-a0-plugin-boundary-inventory.md](2026-07-17-vism-a0-plugin-boundary-inventory.md) | VSM-A0 — 現行plugin境界inventory |
| [2026-07-17-vism-a0d-contract-migration-ownership-decision.md](2026-07-17-vism-a0d-contract-migration-ownership-decision.md) | VSM-A0D — plugin契約とmigrationの所有決定 |
| [2026-07-17-vism-a0s-contract-catalog-spec.md](2026-07-17-vism-a0s-contract-catalog-spec.md) | VSM-A0S — Contract Catalogとprepared plugin解決仕様 |
| [2026-07-17-vism-a1-public-crate-boundary-spec.md](2026-07-17-vism-a1-public-crate-boundary-spec.md) | VSM-A1S — Opacity外部crate化の公開境界仕様 |
| [2026-07-17-vism-a2-legacy-project-migration-decision.md](2026-07-17-vism-a2-legacy-project-migration-decision.md) | VSM-A2S — 旧CLI ProjectV1 migration処分 |
| [2026-07-17-vism-a7-bpm-datatrack-spike.md](2026-07-17-vism-a7-bpm-datatrack-spike.md) | VSM-A7 — BPMから既存DataTrackへの意味spike |
| [2026-07-17-vism-implementation-plan.md](2026-07-17-vism-implementation-plan.md) | Vism実装計画 — 公開境界の反証から配布へ |
| [2026-07-17-vism-ready-counter-review-disposition.md](2026-07-17-vism-ready-counter-review-disposition.md) | Vism-ready化提案の反対側レビュー採否 |
| [2026-07-18-d1k-runtime-camera-thaw-spec.md](2026-07-18-d1k-runtime-camera-thaw-spec.md) | D1k-S CQ-5 解凍記録: runtime planar `CompCamera`と必須camera-bearing render signature(2026-07-18) |
| [2026-07-18-m2-foundation-supplementary-code-review.md](2026-07-18-m2-foundation-supplementary-code-review.md) | M2基盤再締結・独立追補実コードレビュー(2026-07-18) |
| [2026-07-18-m3-egui-selection.md](2026-07-18-m3-egui-selection.md) | M3 UI基盤 egui採用判断(2026-07-18) |
| [2026-07-18-m3-gpu-preview-viewport-prior-art.md](2026-07-18-m3-gpu-preview-viewport-prior-art.md) | M3 GPU Preview / Viewport先例調査 |
| [2026-07-18-vism-a3-external-expression-survey.md](2026-07-18-vism-a3-external-expression-survey.md) | VSM-A3R — 外部表現・Expression・Add-onの責任分類 |
| [2026-07-18-vism-a3d-radial-repeater-decision.md](2026-07-18-vism-a3d-radial-repeater-decision.md) | VSM-A3D — 決定論的 2D Radial Repeater LayerSource 採用決定 |
| [2026-07-18-vism-a3s-layersource-lowering-spec.md](2026-07-18-vism-a3s-layersource-lowering-spec.md) | VSM-A3S — 一般 LayerSource lowering 仕様 |
| [2026-07-19-am-keyframe-graph-observation.md](2026-07-19-am-keyframe-graph-observation.md) | Alight Motionキーフレームグラフ観察台帳(AM実機確認。`codex/m3-mock-components`側から回収) |
| [2026-07-19-m3-interaction-prototype-decision-ledger.md](2026-07-19-m3-interaction-prototype-decision-ledger.md) | M3操作prototype未決パラメータ台帳(2026-07-19。`codex/m3-mock-components`側から回収) |
| [2026-07-19-lyric-motion-text-sequence-comparison.md](2026-07-19-lyric-motion-text-sequence-comparison.md) | リリックモーション: Text Sequence / Materialize 比較台帳(2026-07-19) |
| [2026-07-19-m3-text-motion-task-translation.md](2026-07-19-m3-text-motion-task-translation.md) | M3タスク翻訳: Text Motion(Live Text)縦切り第1弾(2026-07-19) |
| [2026-07-20-rerun-prior-art-survey.md](2026-07-20-rerun-prior-art-survey.md) | Rerun先例調査と歴史的方向決定: 主要製品先例は継続、egui固有転移はG0-9待ち |
| [2026-07-20-rerun-learning-transfer-plan.md](2026-07-20-rerun-learning-transfer-plan.md) | Rerun → Motolii学習・転移計画: RR-0〜9、資産分類、M3/M5接続、停止線 |
| [2026-07-20-rerun-source-asset-inventory.md](2026-07-20-rerun-source-asset-inventory.md) | Rerun固定commitの139 package全量と重点source資産の観察inventory |
| [2026-07-20-rerun-re-ui-module-inventory.md](2026-07-20-rerun-re-ui-module-inventory.md) | Rerun `re_ui` module inventory: React安定ID・M3 task・CJK・転移候補のfile-level照合 |
| [2026-07-20-m3-rerun-late-discovery-premortem.md](2026-07-20-m3-rerun-late-discovery-premortem.md) | M3/Rerun実装後半発覚プレモーテム: fixture正本、GPU表示寿命、stable identity、semantic zoom、転移粒度の先行処分 |
| [2026-07-20-perceptual-expression-translation-decision.md](2026-07-20-perceptual-expression-translation-decision.md) | 知覚表現の翻訳 — Motolii Hostの役割 |
| [2026-07-20-local-worktree-publication-audit.md](2026-07-20-local-worktree-publication-audit.md) | ローカルworktreeの公開・WIP保全・吸収済み・旧契約差分を分類した外部再開地図 |
| [2026-07-21-m3-react-webview-runtime-reconsideration.md](2026-07-21-m3-react-webview-runtime-reconsideration.md) | M3 React / WebView UI runtime再選定（2026-07-21） |
| [2026-07-21-native-stage-gizmo-ownership.md](2026-07-21-native-stage-gizmo-ownership.md) | Native Stage gizmo所有境界: wgpu overlay / CPU picking / Web controls |
| [2026-07-21-native-stage-gizmo-counter-review.md](2026-07-21-native-stage-gizmo-counter-review.md) | Native Stage gizmo案の反対側レビューと縮小採用 |
| [2026-07-21-native-surface-renderer-reselection.md](2026-07-21-native-surface-renderer-reselection.md) | React複合下のnative Stage/Timeline renderer再選定とFableレビュー入口 |
| [2026-07-21-native-surface-renderer-extended-search.md](2026-07-21-native-surface-renderer-extended-search.md) | native surface renderer拡張サーチ(egui以外の追加候補・先例・支援基盤) |
| [2026-07-21-native-surface-renderer-counter-review.md](2026-07-21-native-surface-renderer-counter-review.md) | native surface renderer反対側レビュー(Fable回答・11問) |
| [2026-07-21-native-surface-renderer-growth-review.md](2026-07-21-native-surface-renderer-growth-review.md) | native surface renderer伸長レビュー(Fable回答・機会と優先順位) |
| [2026-07-21-ui-surface-topology-decision.md](2026-07-21-ui-surface-topology-decision.md) | 1 top-level wgpu Surface + Stage/Timeline viewport + opaque child WebView islandsのtopology決定 |
| [2026-07-21-m3-product-mock-recovery-plan.md](2026-07-21-m3-product-mock-recovery-plan.md) | Rectangle製品縦切り・Timeline・複数Surface・隔離・OS受入の一括回収計画と停止線 |
| [2026-07-21-m3-rectangle-drop-d2-contract-options.md](2026-07-21-m3-rectangle-drop-d2-contract-options.md) | Rectangle dropのD2個別契約案: LayerId原子性・exactly-once・selection・Undo/Redo |
| [2026-07-22-m3-comfortable-use-work-map.md](2026-07-22-m3-comfortable-use-work-map.md) | 製品外殻からLocal Alpha、日常操作、配布品質までをユーザーの制作経路で並べる大地図 |
| [2026-07-22-m3-comfortable-use-granulation.md](2026-07-22-m3-comfortable-use-granulation.md) | 快適利用大地図を仕様判断・asset・core・product・E2E・人間/実機審判の検証可能な粒へ分解 |
| [2026-07-22-m3-react-product-asset-promotion-contract.md](2026-07-22-m3-react-product-asset-promotion-contract.md) | Reactモックcomponentを製品packageへ直接所有移管し、縮約再実装と二重stateを拒否する契約 |
| [2026-07-22-m3-native-easing-popup-acceptance.md](2026-07-22-m3-native-easing-popup-acceptance.md) | React起点のnative wgpu Easing popupについて所有境界とG0-9受入条件を固定 |
| [2026-07-22-m3-react-coordinate-surface-audit.md](2026-07-22-m3-react-coordinate-surface-audit.md) | 固定React source内のCanvas/SVG/座標描画面を機械監査し、native再現残量と順序を分類 |
| [2026-07-22-m3-native-multi-key-graph-view-acceptance.md](2026-07-22-m3-native-multi-key-graph-view-acceptance.md) | Blender-like操作語彙を採るnative Multi-key Graph Viewのisolated受入とGPL停止線 |
| [2026-07-22-m3-graph-headless-interaction-dependency.md](2026-07-22-m3-graph-headless-interaction-dependency.md) | Graph Viewのpan/zoom/fitをheadless依存へ委ね、selection/snap/D2をMotoliiに残す裁定 |
| [2026-07-22-m3-native-depth-rail-acceptance.md](2026-07-22-m3-native-depth-rail-acceptance.md) | React正本からnative Depth Railへ同一Z stack、scope、Layer Order Distributeを移すisolated受入契約 |
| [2026-07-22-m3-detachable-panel-window-contract.md](2026-07-22-m3-detachable-panel-window-contract.md) | Timeline/Graphの分割を全製品panelへ一般化するdetach/re-dock・multi-window・単一snapshot契約 |
| [2026-07-22-m3-surface-extension-axis-separation.md](2026-07-22-m3-surface-extension-axis-separation.md) | OS topology、presentation runtime、製品module、plugin、provenance/trustを別軸として固定 |
| [2026-07-22-creator-developer-continuum-decision.md](2026-07-22-creator-developer-continuum-decision.md) | 利用者から作者までを一つの経路にし、React・Vism・first-party参照実装を多数作者の成長戦略へ統合 |
| [2026-07-22-ui-music-metaphor-retirement.md](2026-07-22-ui-music-metaphor-retirement.md) | 「演奏・譜面台・楽曲が背骨」を製品全体の比喩とする仮説を撤回し、音声機能と製品存在論を分離 |
| [2026-07-22-terra-grok-delegation-policy.md](2026-07-22-terra-grok-delegation-policy.md) | タスク級ごとに実装モデルを選び、Grok必須検収とClaude Code Fable追加検収を組み合わせる運用決定 |
| [2026-07-22-u0e-2-delegation-guardrails.md](2026-07-22-u0e-2-delegation-guardrails.md) | U0e-2縮約再実装を再発させないdispatch・source provenance・fixture因果・原子性・証跡ガード |
| [2026-07-23-parallel-order-pipeline-comparison.md](2026-07-23-parallel-order-pipeline-comparison.md) | closed orderを小さく保ったままpreflight・実装・検収を重ねる発注パイプライン並列化の比較案 |
| [2026-07-23-first-party-vism-expression-demand-survey.md](2026-07-23-first-party-vism-expression-demand-survey.md) | AE・AviUtl 1.x／2・Cavalryの表現需要と、人気plugin／公開script系統からVism候補、Host／Infrastructure／Adapter責任、Kitを分ける調査 |
| [2026-07-23-group-source-pool-cloner-concept.md](2026-07-23-group-source-pool-cloner-concept.md) | Groupの直接の子を割合つきprototype poolとしてClonerへ渡すMotolii固有概念の比較 |
| [2026-07-24-camera-object-provider-decision.md](2026-07-24-camera-object-provider-decision.md) | Cameraを換装可能なTimeline Object／Providerとし、点群・splat・volume等とrepresentation非依存Observation Contractで接続する決定 |
| [2026-07-20-m3-keymap-codec-contract.md](2026-07-20-m3-keymap-codec-contract.md) | U0d-2 keymap JSON codec契約 |
| [2026-07-20-m3-u2a-1-command-adapter-contract.md](2026-07-20-m3-u2a-1-command-adapter-contract.md) | U2a-1 gesture command adapter契約 |
| [2026-07-21-m3-u1a-1-static-viewport-contract.md](2026-07-21-m3-u1a-1-static-viewport-contract.md) | U1a-1 静止viewport実装前契約 |
| [2026-07-21-m3-u0e-1-token-generator-contract.md](2026-07-21-m3-u0e-1-token-generator-contract.md) | U0e-1 DTCG token generator契約 |
| [2026-07-21-m3-u0e-2-reference-fixture-contract.md](2026-07-21-m3-u0e-2-reference-fixture-contract.md) | U0e-2 React再結合・5 reference fixture契約 |
| [2026-07-21-m3-u1a-2-layout-projection-contract.md](2026-07-21-m3-u1a-2-layout-projection-contract.md) | U1a-2 panel layout intent / runtime投影契約 |
| [2026-07-21-m3-u1b-1-render-worker-contract.md](2026-07-21-m3-u1b-1-render-worker-contract.md) | U1b-1 latest mailbox / render worker契約 |
| [2026-07-21-m3-u1b-2-latest-projection-contract.md](2026-07-21-m3-u1b-2-latest-projection-contract.md) | U1b-2 latest result / event-loop投影契約 |
| [2026-07-21-m3-u2b-1-single-writer-e2e-contract.md](2026-07-21-m3-u2b-1-single-writer-e2e-contract.md) | U2b-1 single writer配送E2E契約 |
| [2026-07-21-m3-u2c-1-interaction-state-contract.md](2026-07-21-m3-u2c-1-interaction-state-contract.md) | U2c-1 共通interaction state machine契約 |
| [2026-07-21-m3-u2c-4-diagnostic-envelope-contract.md](2026-07-21-m3-u2c-4-diagnostic-envelope-contract.md) | U2c-4 Transient Diagnostic Envelope契約 |

### 歴史価値回収（固定 corpus）

| 文書 | 内容 |
|---|---|
| [2026-07-23-historical-semantic-graph-recovery-tooling.md](2026-07-23-historical-semantic-graph-recovery-tooling.md) | Git corpus・receipt・可搬projection・任意索引の責任境界 |
| [2026-07-23-historical-value-recovery-coverage-ledger.md](2026-07-23-historical-value-recovery-coverage-ledger.md) | 固定manifestとblob receiptによるcoverage台帳 |
| [2026-07-23-losing-specification-value-recovery.md](2026-07-23-losing-specification-value-recovery.md) | 旧仕様を主張単位で分類する回収方針 |
| [2026-07-23-vism-kit-rack-unification-decision.md](2026-07-23-vism-kit-rack-unification-decision.md) | Vism Kit／Plugin Set／curator listの責任分離 |
| [2026-07-23-historical-audio-generalization-lineage-recovery.md](2026-07-23-historical-audio-generalization-lineage-recovery.md) | 音声一般化の歴史回収 |
| [2026-07-23-historical-color-export-lineage-recovery.md](2026-07-23-historical-color-export-lineage-recovery.md) | 色・exportの歴史回収 |
| [2026-07-23-historical-core-plugin-boundary-lineage-recovery.md](2026-07-23-historical-core-plugin-boundary-lineage-recovery.md) | Core／Host／plugin境界の歴史回収 |
| [2026-07-23-historical-d1-code-audit-lineage-recovery.md](2026-07-23-historical-d1-code-audit-lineage-recovery.md) | D1 code auditの歴史回収 |
| [2026-07-23-historical-d1-spec-holes-lineage-recovery.md](2026-07-23-historical-d1-spec-holes-lineage-recovery.md) | D1仕様穴の歴史回収 |
| [2026-07-23-historical-d1l-constructor-lint-lineage-recovery.md](2026-07-23-historical-d1l-constructor-lint-lineage-recovery.md) | constructor／lintの歴史回収 |
| [2026-07-23-historical-d1l-counter-review-evidence-recovery.md](2026-07-23-historical-d1l-counter-review-evidence-recovery.md) | D1l反対側レビュー証拠の回収 |
| [2026-07-23-historical-d1l-journal-undo-lineage-recovery.md](2026-07-23-historical-d1l-journal-undo-lineage-recovery.md) | journal／Undoの歴史回収 |
| [2026-07-23-historical-d1m-sidecar-session-lineage-recovery.md](2026-07-23-historical-d1m-sidecar-session-lineage-recovery.md) | sidecar／sessionの歴史回収 |
| [2026-07-23-historical-d2-selection-timeline-lineage-recovery.md](2026-07-23-historical-d2-selection-timeline-lineage-recovery.md) | D2／selection／Timelineの歴史回収 |
| [2026-07-23-historical-d5-transport-lineage-recovery.md](2026-07-23-historical-d5-transport-lineage-recovery.md) | Transportの歴史回収 |
| [2026-07-23-historical-first-code-audit-lineage-recovery.md](2026-07-23-historical-first-code-audit-lineage-recovery.md) | 第一code auditの歴史回収 |
| [2026-07-23-historical-foundation-lineage-recovery.md](2026-07-23-historical-foundation-lineage-recovery.md) | 基盤文書の歴史回収 |
| [2026-07-23-historical-frame-desc-shared-types-lineage-recovery.md](2026-07-23-historical-frame-desc-shared-types-lineage-recovery.md) | FrameDesc共有型の歴史回収 |
| [2026-07-23-historical-llm-plugin-demo-lineage-recovery.md](2026-07-23-historical-llm-plugin-demo-lineage-recovery.md) | LLM plugin demo証拠の回収 |
| [2026-07-23-historical-m2-camera-contract-lineage-recovery.md](2026-07-23-historical-m2-camera-contract-lineage-recovery.md) | M2 camera契約の歴史回収 |
| [2026-07-23-historical-m2-entry-gate-lineage-recovery.md](2026-07-23-historical-m2-entry-gate-lineage-recovery.md) | M2入口gateの歴史回収 |
| [2026-07-23-historical-m2-reclosure-gate-lineage-recovery.md](2026-07-23-historical-m2-reclosure-gate-lineage-recovery.md) | M2再締結gateの歴史回収 |
| [2026-07-23-historical-m2-supplementary-review-lineage-recovery.md](2026-07-23-historical-m2-supplementary-review-lineage-recovery.md) | M2追補レビューの歴史回収 |
| [2026-07-23-historical-m4-cache-analysis-spec-lineage-recovery.md](2026-07-23-historical-m4-cache-analysis-spec-lineage-recovery.md) | M4 cache／analysisの歴史回収 |
| [2026-07-23-historical-media-portability-gpu-resurvey-plan-recovery.md](2026-07-23-historical-media-portability-gpu-resurvey-plan-recovery.md) | media可搬性／GPU再調査計画の回収 |
| [2026-07-23-historical-memory-model-lineage-recovery.md](2026-07-23-historical-memory-model-lineage-recovery.md) | memory modelの歴史回収 |
| [2026-07-23-historical-param-element-constraint-lineage-recovery.md](2026-07-23-historical-param-element-constraint-lineage-recovery.md) | Param／Element／Constraintの歴史回収 |
| [2026-07-23-historical-performance-model-lineage-recovery.md](2026-07-23-historical-performance-model-lineage-recovery.md) | performance modelの歴史回収 |
| [2026-07-23-historical-permanence-prevention-lineage-recovery.md](2026-07-23-historical-permanence-prevention-lineage-recovery.md) | 恒久焼き込み予防の歴史回収 |
| [2026-07-23-historical-plugin-authoring-lineage-recovery.md](2026-07-23-historical-plugin-authoring-lineage-recovery.md) | plugin authoringの歴史回収 |
| [2026-07-23-historical-plugin-ecosystem-lineage-recovery.md](2026-07-23-historical-plugin-ecosystem-lineage-recovery.md) | plugin ecosystemの歴史回収 |
| [2026-07-23-historical-plugin-resource-runtime-lineage-recovery.md](2026-07-23-historical-plugin-resource-runtime-lineage-recovery.md) | plugin resource／runtimeの歴史回収 |
| [2026-07-23-historical-plugin-ui-lineage-recovery.md](2026-07-23-historical-plugin-ui-lineage-recovery.md) | plugin UIの歴史回収 |
| [2026-07-23-historical-public-capability-provenance-lineage-recovery.md](2026-07-23-historical-public-capability-provenance-lineage-recovery.md) | 公開capability／provenanceの歴史回収 |
| [2026-07-23-historical-r1-export-gpu-safety-lineage-recovery.md](2026-07-23-historical-r1-export-gpu-safety-lineage-recovery.md) | R1 export／GPU safetyの歴史回収 |
| [2026-07-23-historical-r3-datatrack-export-correctness-lineage-recovery.md](2026-07-23-historical-r3-datatrack-export-correctness-lineage-recovery.md) | R3 DataTrack／export correctnessの歴史回収 |
| [2026-07-23-historical-r9-real-material-export-acceptance-lineage-recovery.md](2026-07-23-historical-r9-real-material-export-acceptance-lineage-recovery.md) | R9実素材／export受入の歴史回収 |
| [2026-07-23-historical-react-webview-lineage-recovery.md](2026-07-23-historical-react-webview-lineage-recovery.md) | React／WebViewの歴史回収 |
| [2026-07-23-historical-reclosure-counter-review-evidence-recovery.md](2026-07-23-historical-reclosure-counter-review-evidence-recovery.md) | 再締結反対側レビュー証拠の回収 |
| [2026-07-23-historical-render-ctx-thaw-lineage-recovery.md](2026-07-23-historical-render-ctx-thaw-lineage-recovery.md) | RenderCtx解凍の歴史回収 |
| [2026-07-23-historical-s2-decode-pipeline-lineage-recovery.md](2026-07-23-historical-s2-decode-pipeline-lineage-recovery.md) | S2 decode pipelineの歴史回収 |
| [2026-07-23-historical-semantic-oracle-boundary-recovery.md](2026-07-23-historical-semantic-oracle-boundary-recovery.md) | semantic oracle境界の歴史回収 |
| [2026-07-23-historical-shared-effect-lifecycle-lineage-recovery.md](2026-07-23-historical-shared-effect-lifecycle-lineage-recovery.md) | Shared Effect lifecycleの歴史回収 |
| [2026-07-23-historical-test-oracle-ruleset-recovery.md](2026-07-23-historical-test-oracle-ruleset-recovery.md) | test oracle rulesetの歴史回収 |
| [2026-07-23-historical-unified-stage-camera-ui-lineage-recovery.md](2026-07-23-historical-unified-stage-camera-ui-lineage-recovery.md) | Stage／Camera UIの歴史回収 |
| [2026-07-23-historical-vello-adoption-lineage-recovery.md](2026-07-23-historical-vello-adoption-lineage-recovery.md) | Vello採否の歴史回収 |
| [2026-07-23-historical-vism-a3-expression-layersource-lineage-recovery.md](2026-07-23-historical-vism-a3-expression-layersource-lineage-recovery.md) | Vism A3／LayerSourceの歴史回収 |
| [2026-07-23-historical-vism-foundation-contract-lineage-recovery.md](2026-07-23-historical-vism-foundation-contract-lineage-recovery.md) | Vism foundation contractの歴史回収 |
| [2026-07-23-historical-vism-kit-distribution-lineage-recovery.md](2026-07-23-historical-vism-kit-distribution-lineage-recovery.md) | Vism Kit／distributionの歴史回収 |
| [2026-07-23-historical-wgpu-readback-cold-compile-lineage-recovery.md](2026-07-23-historical-wgpu-readback-cold-compile-lineage-recovery.md) | wgpu readback／cold compileの歴史回収 |
| [2026-07-24-replaceable-semantic-seat-decision.md](2026-07-24-replaceable-semantic-seat-decision.md) | HVR-D04 Unit 8A — Host semantic seat、換装可能Provider、Effect／Filter分類とContent-Aware Scale候補 |
