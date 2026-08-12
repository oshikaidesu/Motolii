# UI滑らかさの品質バー(quality bar)

- 制定: 2026-08-12(利用者目標「普通のUI・普通のUX・トンマナ維持・違和感ゼロ・UIは常時滑らか・Stageは追従してリアルタイム」の数値化)
- 位置づけ: 全UI発注の常設oracle。GR-UI-9(数値ログと数値審判)の適用先。トンマナの正本は[ui-visual-language](ui-visual-language.md)/[ui-interaction-language](ui-interaction-language.md)で、本書は**時間予算だけ**を持つ

## 1. 予算(display 60Hz基準)

| 項目 | 予算 | 測り方 |
|---|---|---|
| B1 定常フレーム | Timeline/Stageともrender thread CPU **p99 ≤ 8ms**、平均 ≤ 4ms | 既存 `[MotoliiRerunStage]`/`[MotoliiTimelineProbe]` telemetryへp99を追加 |
| B2 gesture中のフレーム落ち | drag/scrub/zoom中に**16.7msを超えるフレームを連続2枚出さない** | 同telemetryのgesture区間タグ |
| B3 入力→視覚(local preview) | pointer入力から当該surfaceの描画反映まで**≤1フレーム** | 構造保証(同tick反映)+計測 |
| B4 入力→視覚(host往復) | dispatch起因の状態反映(選択・編集結果)まで**≤2フレーム**(応答snapshot即時適用済みの現構造を維持) | 計測 |
| B5 Stageのリアルタイム追従 | scrub中、Stage実フレームの評価+描画を**p95 ≤ 16ms**(現行規模のDocument、DRAFT品質)。超えたら品質段階降下(解像度/品質)で**フレームは落とさない** | seam内計測 |
| B6 起動系スパイク | 初回shader/pipeline構築を除き、定常運転で**50ms超のフレームを出さない**。初回も500ms以内 | max telemetry |
| B7 メインthread停止 | UI入力threadを**4ms超**塞ぐ同期処理を置かない(JSON parse・lock待ち含む) | 計測+review |

## 2. 既知の予算違反(制定時点の実測、修正grain)

1. **毎tickのsnapshot JSONパース**: `try_read_timeline_projection` がrender tickごとに最大131KBをパース(renderer_core.rs:936,1008)。→ hostへ「(revision, generation)だけを返す軽量getter」を足し、**変化時のみ**パースへ
2. **registry mutexをGPU submitまで保持**(rn_product_host、狩り#P2): render中はsnapshot/intentが全部待つ。→ lockはDocument読み取り+graph構築までに縮め、GPU submitはlock外へ
3. **maxスパイク**: 実測 Stage 489ms / Timeline 175ms(初回構築を含む)。B6の分解と初回のwarm-up(起動時に1回空renderでpipeline構築を先払い)
4. scrub中のset_time throttle 32msは**hostの評価間隔**であり表示フレームは独立(B2)。throttle値はB5の実測に従い調整可

## 3. 非目標

- 予算内での過剰最適化(数値が満ちていれば触らない)
- トンマナ・レイアウト・操作文法の変更(本書は時間だけ)
- 60Hz超(ProMotion 120Hz)への最適化は別粒

## 4. 検収規則

- UI系orderのoracleに本書のBn番号を明記して引用する(「B1/B2をtelemetryで示す」等)
- 予算は反証可能: 実測がDocument規模起因で満たせない場合、規模条件つきで本書を改訂する(黙って超えない)
