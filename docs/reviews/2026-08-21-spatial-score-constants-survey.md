# S 空間スコア定数化の外部理論調査(ξ)— RESEARCH_RETURN 保全(要点)

日付: 2026-08-21 / 発注: 利用者の問い「概念はベクトルであり意味として弱い。外部に答えは無いか、より定数化できるか」/ レーン: web 調査(sonnet)
正典への反映: **裁定164**(本文書と同 commit の `docs/ui-spatial-score.md` 改訂)。

## 採用(即戦力 Top5 — 正典へ注入済み)

1. **S4 → WCAG コントラスト比の段柵**: ink 弱≥3:1・中≥4.5:1・強≥7:1(対背景)。式 `(L1+0.05)/(L2+0.05)`・導出(ANSI 3:1 × 視力20/40 感度損失 1.5 = 4.5)を W3C 原文で逐語確認済み。screenshot 画素から即計算可
2. **S2 → KLM 予測秒数**: `T = ΣP(1.1s)+ΣK/B(0.2s)+ΣH(0.4s)+ΣM(1.35s)`(Card, Moran & Newell 1980)。「距離の和」から「秒」へ格上げ。Hick-Hyman(`RT≈200ms+150ms·log2(n+1)`)を分岐点数の上限設計に併用
3. **S3/S5b の画素判定基盤 = Rosenholtz Feature Congestion / Subband Entropy**(2007、公開ツール有)または **Miniukovich & De Angeli 8指標**(CHI 2015 — clutter・figure-ground contrast・contour congestion・grid quality・white space 等、美的評価の分散49%を説明)。自前の「重心・分散」より確立度が高い
4. **S1 ヒット寸 12px の身分注記**: ISO 9241-411(9mm)・Apple 44pt・Material 48dp は**タッチ規格**でありマウス精密操作の根拠にならない。12×12 は Motolii 実測由来のドメイン固有値と明記(誤借用の柵)
5. **S0 の違反分類語彙 = ISO 9241-110:2020 の7原則**(期待適合性・自己記述性ほか)。Jakob's Law は定性裏付けとして引用

## 棄却・反証

- **F/Z パターン**: 反証済み(NN/g 自身の訂正+EyeQuant)。S3 の幾何プライアにしない
- **黄金比**: UI への直接実証は存在せず、隣接領域(顔面美容 2024)で明確に反証。「比率の一貫性(モジュラースケール)は手法として有効、特定値 1.618 の魔法性は根拠なし」で確定
- **S5a 占有率(90/80/75/70%)の外部定数は存在しない** — 正典の言う通り実測較正が正。改良案: 実プロ NLE の screenshot から density/white-space 指標の percentile を測って較正する(輸入でなく実測)
- **Accot-Zhai steering law**(`T=a+b·A/W`)は採用候補(細いレーン内ドラッグのコスト)だが式定数が実験依存 — 器具が動いてから第2波で

## EVIDENCE_GAP(次回追跡)

Ngo, Teo & Byrne 2003 の14測度の正確な式(ペイウォール — 2026 Symmetry 誌 OA 論文が追跡候補)/ AIM(aalto-ui/aim)の全指標とオフライン実行性(ソース直読要)/ 慣習逸脱コストの定量研究(未発見)/ Apple 44pt・Material 48dp の1次出典(2次止まり。Material 8dp 系の主根拠は密度スケールの整数丸め=開発都合)/ CogTool 保守状態

出典一覧はレーン返却原文(セッション記録)参照。主要どころ: Card-Moran-Newell 1980 CACM / Accot-Zhai CHI99 / Itti-Koch-Niebur 1998 PAMI / Rosenholtz+ 2007 JoV / Miniukovich & De Angeli CHI2015 / W3C WCAG 2.1 SC1.4.3 / ISO 9241-110:2020 / Pirolli & Card 1999(information scent — S4「発見依存度」の学術先行)
