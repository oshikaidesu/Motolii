# 第二D1コード監査lineageの価値回収（Unit 4C-2、2026-07-23）

状態: **観察**（cutoff 4 historical blobの処分完了）

対象: `docs/reviews/2026-07-12-code-audit-2nd-d1.md`のcutoff全4版。

関連: [第二D1コード監査](2026-07-12-code-audit-2nd-d1.md)、[M2仕様](../specs/M2-document-model.md)、[Vism実装計画](2026-07-17-vism-implementation-plan.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

4版は単なる古い欠陥一覧ではなく、指摘、先例追補、ユーザーによる三決定、式と依存の補正という監査の成立過程を残している。ただし対象SHAは`9c8e274`であり、当時のfile:lineを現在の欠陥証拠へ流用してはいけない。2026-07-23の現行コードと試験で再照合した結果は次のとおり。

- **実装済みとして閉じる**: S1〜S7、S9〜S16、S18、B④。TimeMap原点、正準有理数、typed Asset、DocParam型検査、VectorRecipe、PathOp意味、正確なsample index、値域、ResourceLimits、fault injection、semantic fingerprint、未来plugin degraded、OpenMode、非破壊recovery、semantic oracle、gesture merge、visible/solo/lock三軸は現行コードと試験にある。
- **比較中へ戻す**: S8のDataTrack正準ID。古い`producer+version+output+source`案にはrename耐性を求めた価値があるが、後続Vism設計はpackage／entry／instance／artifact／provider接続を分離し、VSM-B2まで一般consumer方式を凍結した。四要素を今のtuple structへ焼かない。
- **未実装審判として戻す**: S17のOTIO中間写像+loss report試験。gate台帳には残っているが、対応するコード、写像表、試験は無い。M2全体の完了表示で閉じない。
- **復活させない**: 当時のfile:line、当時のテスト件数、D1/D2/D3の未着手表示、外部先例の未検証詳細、per-op `algorithm_version`、OTIO runtime/exportの前倒し。

## 2. 四版の処分

| blob | 変化 | 現在の判定 |
|---|---|---|
| `01f48b2e` | S1〜S18を対象SHAで裏取りし、S1/S6/S16を決定待ちにした初版 | 問題分類と停止線を保持。コード事実はarchive |
| `f4e4aac6` | S1/S6/S16へOTIO/FCPXML、Lottie/AE、AE Legacy/Blender先例を追補 | 一次確認済み範囲と「整合する先例」限定を保持 |
| `57645436` | 三点とも案1を採択し、S6の過剰説明とS16執行不足を訂正 | **現行決定へ採用**。root modifier stackとsemantic oracleを保持 |
| `2cd9e586` | source終端式、依存直列、DataTrack二段検査、D1i分割を補正 | **最終歴史版として保持**。進捗は現行仕様で上書き |

版の順序には意味がある。初版の断定を最新決定へ混ぜず、特に「VectorRecipeならLottieと同じscopeを得る」という途中説明は最終版自身が撤回している。

## 3. 現行コード照合

| 監査項目 | 2026-07-23の証拠 | 処分 |
|---|---|---|
| S1/S2 | clip-local `TimeMap`、有理数正準化、identity試験 | 実装済み |
| S3/S4/S9 | typed `DocAssetRef`、期待型表、全key/fallback/非有限・値域検査 | 実装済み |
| S5/S6 | `VectorRecipe { content, modifiers }`、PathOp validate/semantic oracle | 実装済み。Lottie兄弟scopeは非目標 |
| S7 | `RationalTime::try_to_sample_index_since`をDataTrackが利用 | 実装済み |
| S8 | `motolii_eval::DataTrackId(pub String)`。VSM-A7は既存parameter結線だけ | **GAP-21** |
| S10 | `ResourceLimits`と読込前／構造上限試験 | 実装済み |
| S11/S15 | `FaultInjectingFs`、partial/ENOSPC/reorder/kill、原本保持recovery | 実装済み |
| S12/S14 | `semantic_fingerprint`、三値`OpenMode`、write/migration拒否 | 実装済み |
| S13 | `known_plugin_future_version_is_degraded_not_a_downgrade_error` | 実装済み |
| S16 | semantic oracle分類と更新禁止CI | 実装済み |
| S17 | gate台帳の行だけで、写像表・loss report test無し | **GAP-22** |
| S18 | `gesture_id + command_kind + target_stable_id + property_id`の`MergeKey` | 実装済み |
| B④ | `visible/solo/lock`の描画・評価・編集意味とD3試験 | 実装済み |

「型や関数がある」だけでは閉じず、対応する負例または意味審判まで確認した。逆にS17は文字列検索で実装が見つからないことだけを完成条件にせず、現行spec、gate、backlogにも完了証跡が無いことを合わせて未実装と判定した。

## 4. S8 DataTrack identityは比較中へ戻す

S8が守ろうとしたのは、表示名変更で参照が切れないことと、生成元の違う値列を混同しないことである。この成立理由は残る。しかし`producer+version+output+source`を一つの恒久IDへ畳むと、少なくとも次が未決のまま焼かれる。

- `producer`がpackage、entry、Project instance、実行artifactのどれか。
- `version`がpackage互換、出力schema、algorithm、cacheのどれか。
- `source`がAsset指紋、Document layer、provider instance、時間区間のどれか。
- rename、fork、provider差替え、Kit materialize後に同一性を保つ対象。

よってVSM-B0のidentity fixtureとVSM-B2の三方式比較で責任を決める。現行文字列IDは内部結線として維持するが、公開package/schemaの正準identityという意味を追加しない。S8の四要素も比較入力であって採択済みformatではない。

## 5. S17 OTIO loss reportを未実装審判として戻す

回収するのはOTIO exportそのものではない。代表Documentを試験専用の中間構造へ写し、source range、available range、Gap、Transition、Stackに対応できないMotolii意味をtyped loss listとして列挙する小さな審判である。

この審判は「MotoliiをOTIOへ合わせる」ためではなく、外部交換時に失う意味を先に可視化するためにある。したがってGAP-22では次を禁止する。

- OTIO crate/typeをDocumentまたは公開plugin APIへ露出する。
- 中間fixtureの都合でMotolii固有のcamera、effect、parameter、時間意味を削る。
- 全要素を無損失に見せるためlossを黙ってdrop/default化する。
- 本export、AAF/FCPXML、codec処理まで同じticketへ束ねる。

V2-5の本exportは後続の独立境界であり、本審判の未完を理由に現在のDocument schemaを変更しない。

## 6. 復活させないもの

- 対象SHA `9c8e274`のfile:lineを現行コードの場所として引用すること。
- 2026-07-12の「D2/D3未着手」や当時のtest総数を現在の進捗へ戻すこと。
- `DataTrackId`へ四要素文字列を連結し、区切り・escape・versionを実装者defaultで決めること。
- DataTrackの表示名、永続参照、cache key、package identityを同じIDへ統合すること。
- OTIO loss fixtureを本番export APIまたはDocument schemaの根拠にすること。
- S16の案2であったper-op `algorithm_version`fieldを再追加すること。
- 「既存variantの画を改善する」名目でsemantic oracleを更新すること。

## 7. 固定歴史出典とcoverage

初版`01f48b2e`を全文で読み、以後3版の差分と変更節を確認した。処分した4 blobの完全SHAは`evidence/historical-value-recovery/disposition-receipts/04c2-d1-code-audit.tsv`を正本とする。cutoff総数1,797のうち処分済みは249、未処分は1,548である。
