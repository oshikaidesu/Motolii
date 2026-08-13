# 体験の段差台帳(friction ledger)

- 制定: 2026-08-12(利用者裁定「まだある細かい部分の体験の段差を全て撤廃する」)
- 位置づけ: [品質バー](ui-quality-bar.md)違反の**未修正在庫**。空であることが定常状態 — 新しい段差は発見次第ここへ入れ、orderで焼却する。18本の独立review・3ハンター・gesture嵐で指摘済みだが未修正のP2級を初期在庫とする

## 在庫(2026-08-12制定時点)

### 掃討wave A(interaction) — **焼却済み 2026-08-13**

F1(playhead直接掴み)/F2(カーソル言語: trim=resizeLR・clip=open/closedHand・key=pointingHand・drag中はhit外でも維持・mouseUp後再計算)/F3(Undo/Redo文脈disabled、wire `history`)/F4(ruler目盛の絶対位相)/F5(exact-on-key行のgesture実信号凍結+凍結identity commit)/F6(空Timeline一行ガイド)/F7(key hit半径5.6px視覚一致+境界test)/F8(`(+N)` 実件数、`truncated_total` saturating集計) — order 19+fix19で全焼却。PNG sha `43ec101c` 不変。

### 掃討wave B(performance) — **焼却済み 2026-08-13**

F9(毎tick最大128KiB JSONパース → `motolii_rn_host_projection_stamp` 軽量stamp FFIで変化時のみfull読み。stampはtick冒頭1回読みで判定と保存に同値を使いTOCTOUなし、失敗時はstamp破棄で次tick必ず再読)/F10(registry mutexを「取り出し/書き戻し」2区間へ分割、graph構築とGPU submitはlock外)/F11(mount時warm-up先払い、telemetry `warmup_us=… ok=…`) — order 20+fix20で全焼却。実機実測(30秒run全体max、初回可視フレーム込み): 初回スパイクStage 489ms/Timeline 175ms → **Stage 20.7ms/Timeline 1.3ms**(50ms超ゼロ=B6充足)、warm-up自体はStage 29.6ms/Timeline 11.7ms(ok=1)。

### 掃討wave C(vism source write) — **焼却済み 2026-08-13**

F12(source paramが表示のみ) — `SetProperty` + `ScalarPropertyId::SourceParam` が既存 `ClipSource::Plugin.params` キーへ任意 `DocParam` を書く。RN Inspector の第一consumerは f64 Const。Color/Vec は同じ command。vism の型天井は置かない。

### 台帳残(grain/campaign従属 — 対応先が決まっているもの)

| # | 段差 | 行き先 |
|---|---|---|
| F13 | 17件目以降のlayer/65個目のkey/9個目のeffectがUIから触れない(capの沈黙は(+)で緩和済みだが操作不能は残る) | cap設計grain |
| F14 | DOC status labelの開発者臭(`DOC r42`) | Q0 inventory掃除(Sol) |
| F15 | keydragが隣接clip境界のkeyでtrimに先勝ちされる | 判定順の再考(小粒) |
| F16 | 複数timeline間のCAS上書き / BigInt精度(i64>2^53) | known limit(multi-window/将来) |

## 規則

- 段差の追加は誰でも(発見者が)行う。**削除はorderの着地のみ**
- 「仕様どおり」は撤廃を免除しない — 体験として段差なら在庫に入る
