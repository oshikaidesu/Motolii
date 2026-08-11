# リポ外資産回収・docs乖離 監査

日付: 2026-08-10
状態: **観察**(処置は[段差撤廃決定](2026-08-10-main-merge-friction-removal-decision.md)と後続の回収発注で行う)
方法: mdリンク到達性の全量走査(516ファイル)、`~/Documents/Codex/`のmtime走査、origin/main全履歴・全ブランチとの突合。

## 1. 健全だったもの(誤診の訂正)

- **リンク到達性**: 入口(AGENTS.md / docs/README / decision-index / implementation-ledger / reviews索引)から辿れないmdは22件のみで、ほぼspikes配下。check-docs.shの索引・リンク検査は機能している。
- **「docsに設計根拠が全く無い実装」はゼロ件**: 台帳・decision-indexにクレート名が出ない9クレート(export/transport/testkit/render/nodes/plugins-firstparty、plugins 3種)も全てspecs/かreviews/に正本がある。問題は「クレート名では引けない索引粒度」。
- **「退役テスト・CIの復活」はgit上に実体ゼロ**: origin/main全履歴+未merge全ブランチで、削除済みtest/CI設定の再追加は0件。目撃された「復活」は返却diff段階で止まったもの。幻視トリガになり得る残存物は2つ: `crates/motolii-testkit/tests/protected_assets.rs:178`の退役済み`ci.yml`パス文字列fixture、CI退役19分後に追加された`.github/ISSUE_TEMPLATE/closed-contract.yml`等の非workflow yml。

## 2. 未回収のリポ外資産(mtime実測)

| 資産 | 最終mtime | 回収率 | 備考 |
|---|---|---|---|
| MotoliiRnProbe(App.tsx 660行、Browser 3タブ/Extensions/panel registry/Timeline 3モード/Fabric `MotoliiGpuView`・`MotoliiTimelineView`) | 2026-08-06(08-10にも接触あり) | 約2割(`ui/motolii-rn-legacy` 149行: BrowserPanel/Inspector initial read/StageComponentView のみ) | [08-08棚卸し](2026-08-08-out-of-repository-asset-inventory.md)が移管を宣言済み |
| skia-timeline-probe(bin 15本: timeline_interactive / curve_editor_interactive / stage_present_interactive / motolii_depth系 ほか、depth-rail v4〜v14) | 2026-08-08 15:53〜17:48(棚卸し文書より後の設計セッション) | **0%**(`skia-safe`のCargo依存のみ着地。`skia_safe`を使うRustコードはorigin/mainに0行) | [depth-rail決定](2026-08-08-depth-rail-selection-focus-decision.md)がv14静止画と対話demoをリポ外絶対パスで正本参照 |
| 2026-08-07/skia-3d ほか2 dir | — | — | work/outputsとも空。実害なし |

**構造欠陥**: リポ外絶対パス(`Documents/Codex`等)を参照するdocsは**13本**(decision-index本体を含む)。発注LLMはリポしか見えないため、決定済みUIが「存在しない」世界で作業し、無視・再発明が構造的に再発する。

## 3. 台帳・地図の乖離(08-09/08-10実装ラッシュ由来)

- implementation-ledger: `M2-ASSET-1A`を`RESEARCH_RETURN / SPLIT REQUIRED`のまま保持するが、該当スコープ7コミット(`d273061d`〜`a287c828`: SourceFingerprintV1、AssetTable、journal edit v3、budgeted SourceBinding)は08-09にmain着地済み。依存待ちの`M2-ASSET-1C`/`M4-P02-CODEC`の`WAIT`は解除可能。
- m4採択地図: `P02-C2 = IMPLEMENTATION NOT STARTED`だが実装済み。**この地図で発注するとSourceFingerprintV1を再実装させる。**
- outcome-driven-integration-map: `N-OVERLAY = PROBE_ONLY`「rust-skiaはCargo.tomlに存在しない」が実態と逆(R2系4ノードのゲートに波及)。
- m3-rn-runtime-execution-map(authority): `R1-BROWSER = COMPILE(実装禁止)`/`R1-HOST-EDIT = WAIT`が両方実装済み(BrowserPanel.tsx / rn_product_host.rs)。修正2〜3行。
- ui-reference-map: Browser/InspectorのRN移管完了が未反映。
- backlog: ヘッダ日付3週ずれ、INF-6「未着手」(実際はfault recovery test多数)、GAP-3前提3件完了済み。
- m5/vism地図は乖離ゼロ(pause契約で正当)。m3旧2地図は再基線バナーで自己凍結済み。

## 4. spike回収の宙吊り(旧調査分)

- `pv1-texture-lifecycle`: PV-1 pass(Metal)の結論があるのに決定文書からの参照ゼロ、docs/READMEのspike一覧にも不掲載。
- `ime-acceptance`: 結論なしのまま対象toolkit(Slint)がegui転換で消滅。退役tombstoneが無い永久pending。

## 5. 第二の滞留層 — ローカル専用ブランチ/worktree(2026-08-10追記)

リモートブランチ走査だけでは足りなかった。`git worktree list`と`git for-each-ref refs/heads`で
**originに存在しないローカル専用ブランチ**に成果が滞留していた。

- **回収済み**: `codex/m3-collaborative-bringup-20260810`(同日のClaude Code協働セッション成果、
  6コミット+2,437行: `timeline_skia_raster.rs` 314行、`stage_overlay_gpu/raster.rs`、
  `MotoliiTimelineComponentView.mm`、Fabric spec、ledger/decision-index更新)を`2354a5e7`でmainへ
  マージ。サブ4ブランチ(timeline-port/seat/stage/rn-panels)のパッチ等価包含と、origin
  `n-overlay-composite`/`minimal`との内容同一を確認して重複2本を削除。`cargo check -p motolii-ui`
  PASS(skia含む)。
- **未裁定のローカル一意ブランチ約40本**(全てのdetached worktree HEADはmain到達済み):
  - `p2d-rc*`系12本(07-29、supervised-runner研究loop) — runner体系は退役済み。tombstone候補
  - `m3-p06-c1-mac`(+r2〜r6)/`m3-p06-c1-selection`/`m3-local-alpha`計8本(08-02、code 65ファイル、
    ほぼ同一リトライ) — P06-C1-MACの結論はdecision-indexへ縮小採用済み。代表1本の保持判断のみ残る
  - `m3-g0-9-h1a-product-react`(v1〜v3)/`m3-tonight-product-slice`(07-22) — RN再基線で経路ごと退役
  - `vism-a1`系3本(07-17)、`m5-render-common-foundation`(07-29)、`m4-k1a-validation`(07-29)、
    `uncommitted-docs-20260718`/`m3-pv1-preview-lifecycle`(07-18) — 結論の回収状況を個別確認してから
    merge/tombstoneを二分する

## 次手(本監査からの発注候補)

1. ~~skia-timeline-probeの`spikes/`への移管~~ → **完了(2026-08-10)**: `spikes/skia-timeline-probe/`。
2. 旧名MotoliiRnProbeは2026-08-10に回収し、2026-08-11に[`ui/motolii-rn/`](../../ui/motolii-rn/README.md)へ製品sourceとしてcut overした(skia側は[`spikes/skia-timeline-probe/`](../../spikes/skia-timeline-probe/README.md))。`ui/motolii-rn-legacy`への製品移管は退役した。
3. ~~台帳・地図の一括同期~~ → **完了(2026-08-10)**: ledger 6A/7A/7B、m4地図P02-C2、統合地図N-OVERLAY、RN地図R1×2、ui-reference-map、backlog。同日、未mergeブランチ238本を検証の上217本削除(全内容main到達済みを機械証明)、残21本が回収判断対象。
4. ~~check-docs.shへ「docs内のリポ外絶対パス参照」検出を追加~~ → **完了(2026-08-10)**: `scripts/check-stray-work.sh`が滞留5層(ローカル/リモートブランチ、worktree、リポ外workdir、リポ外パス参照)を1コマンドで観測する。AGENTS.md検証節に「作業終了時に実行し、自分の成果をSTRAYに残さない」を追加。本監査級の歴史調査は以後不要。
5. `protected_assets.rs:178`のfixture文字列を退役済みでないパスへ差し替え。
6. `pv1-texture-lifecycle`結論の回収と`ime-acceptance`退役tombstone。
