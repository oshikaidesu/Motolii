# ホストを iced へ — 乗り換え裁定と移行地図

日付: 2026-08-18
状態: **決定**(利用者裁定: 「変な感じはしない、なによりコード数が決定的すぎる、乗り換えるか」)

## 裁定と根拠(全て同日の実測)

シェルのホスト toolkit を egui から **iced(master 系)** へ移行する。根拠:

1. **DX 実測**([仮タイムライン spike](2026-08-18-iced-reentry-survey.md)): 同4ジェスチャが
   342行 vs 約1,415行、draw パス内の永続状態書き換え 0(型が禁じる) vs 97文。
   利用者が spike を実際に触り「変な感じはしない」と確認
2. **段差ゼロが公式不変量**: iced_test(headless)+ time-travel が「操作=Message列」
   を一級機能として保証 — 本日自前で建てた「ログと構造の強制」をフレームワークが
   肩代わりする
3. **Rerun はホスト非拘束**(3 probe 実測): offscreen+入力ブリッジで iced 内に
   絵・入力・カメラまで成立済み。egui は Stage 島の内側の実装詳細に縮む

## 引き受けるコスト(実測済み・目をつぶらない)

- **fork 2本体制**: rerun(camera seat 済み)+ **iced fork 新設**(web-sys 釘打ち
  1行解除・`max_bind_groups` 定数 — どちらも上流 PR 候補)。seam 台帳方式は
  [rerun fork seam ledger](2026-08-18-rerun-fork-seam-ledger.md)を踏襲
- master 追随(1.0 前・minor ごと破壊的変更)。rev pin+常設 oracle での検収は
  rerun fork と同じ型
- **AccessKit 後退**: egui は統合済み・iced upstream は未統合。運転席の
  AccessKit query は iced_test の Selector へ置換。アクセシビリティ自体の後退は
  iced 上流(System76 が作業中)の再観測点として残す
- WheelScrolled が modifiers を運ばない等の小穴(spike で回避策実証済み)

## 移行地図(絞め殺し方式 — campaign を止めない)

**不変の資産(移行対象外)**: motolii-doc / render / export / media / audio /
plugin(D2・journal・評価器)。**UiIntent 背骨は設計どおり持ち越す**(intent ≒ Message)。
`motolii-input`(2,413行)は toolkit-free で無傷。

- **M-0 土台**: iced fork 作成(pin rev+seam 2件+seam 台帳)、`motolii-shell-iced`
  crate 新設(dep policy へ iced 系 allowlist を追加)。egui shell は**並走のまま**
  — **着手済み**(2026-08-18)。fork の乖離と再適用手順は
  [iced fork seam 台帳](2026-08-18-iced-fork-seam-ledger.md)。柵は2本(iced を持てるのは
  新殻だけ / 新殻は egui を持てない)で、後者は egui 側 allowlist へ新殻を**入れないこと**で
  成立している
- **M-1 殻**: スタート画面・New/Open/Save・status 帯・prompts 台本・
  `--intent-log`/replay・iced_test の新運転席(kittest 相当の駆動+replay oracle)
- **M-2 Stage 島**: 入力ブリッジ probe を製品 adapter 化(camera seat・正対既定・
  transcript 相当の失敗報告)
- **M-3 Timeline**: spike を種に Document へ結線(prepare_*/D2 は既存。egui 版の
  意味関数・oracle は移植元)。波形帯・audio seat の載せ替え
- **M-4 Browser / Inspector**: iced widget 化(標準 widget 領域 = iced の得意面)
- **M-5 切替**: UX 台本 P1〜P5 が iced shell で通り、replay oracle・フェンス同等物が
  green になったら既定 bin を切替。egui shell は当面 `--legacy` で残し、
  勝負が付いたら撤去

各 M は red 先行+検収+gate の通常レーン運転。M 間は直列、M 内は並列可。

## 既存決定との関係

- [2026-08-16 Timeline 実行時基盤=egui](2026-08-16-timeline-runtime-reselection-to-egui.md):
  **ホストについて supersede**(egui は Stage 島内へ)。当時の実測事実(DOM 天井・
  行モデル・P8)は引き続き有効で、iced canvas 実装の設計材料
- [運転席決定](2026-08-18-cli-gui-driver-seat.md)・[ログと構造の強制](2026-08-18-log-and-structure-enforcement.md):
  **意味は不変のまま iced へ移る**(transcript/intent/replay は host 非依存の契約)
- [Rerun 合成基盤裁定](2026-08-18-rerun-as-composition-foundation.md): 不変(島として続行)
- toolkit 再入場トリガー: 発火したのは「繋がっていない」でなく **DX 実測**だった —
  帳簿は「慢性コスト枠が決定打になった」と読む
