# 体験の段差台帳(friction ledger)

- 制定: 2026-08-12(利用者裁定「まだある細かい部分の体験の段差を全て撤廃する」)
- 位置づけ: [品質バー](ui-quality-bar.md)違反の**未修正在庫**。空であることが定常状態 — 新しい段差は発見次第ここへ入れ、orderで焼却する。18本の独立review・3ハンター・gesture嵐で指摘済みだが未修正のP2級を初期在庫とする

## 在庫(2026-08-12制定時点)

### 掃討wave A(interaction — 発注済みで焼却中)

| # | 段差 | 根拠 |
|---|---|---|
| F1 | **playheadを直接掴めない**(頭/線にhitが無く、rulerからしかscrubできない) — 一般編集ソフトはplayhead頭を掴む | 地図漏れの反射 |
| F2 | **カーソル言語ゼロ+hover状態ゼロ**(trim端でresizeカーソルが出ない、clipでgrabが出ない) | 文法地図Tier 1 |
| F3 | Undo/Redoが**履歴空でも押せて沈黙**(Q3違反) | hunter#3系 |
| F4 | ruler目盛がfractional pan後に**位相ずれ**(固定小節の倍数に乗らない) | review P2(第二弾) |
| F5 | scrub通過中にexact-on-key編集行が**チラつく** | 構造帰結 |
| F6 | 空Timelineが**無言**(何をすればいいか一行も言わない、Q7) | 品質バーQ7 |
| F7 | key菱形のhit半径(6px)と視覚(4.2〜5.6px)の**不一致** | 計測 |
| F8 | truncated表示`(+)`が**暗号**(何が何件隠れたか分からない) | 第十八弾FINDING |

### 掃討wave B(performance — B違反、次order)

| # | 段差 | 根拠 |
|---|---|---|
| F9 | **毎render tickの最大131KB JSONパース**(B7違反) → (revision,generation)軽量getterで変化時のみ | 品質バー既知違反① |
| F10 | **registry mutexのGPU submit跨ぎ保持**(入力停止の原因) | 同② |
| F11 | **初回フレームスパイク**(実測489ms) → 起動時warm-up先払い | 同③ |

### 台帳残(grain/campaign従属 — 対応先が決まっているもの)

| # | 段差 | 行き先 |
|---|---|---|
| F12 | source paramが表示のみ | SetSourceParam鏡映grain |
| F13 | 17件目以降のlayer/65個目のkey/9個目のeffectがUIから触れない(capの沈黙は(+)で緩和済みだが操作不能は残る) | cap設計grain |
| F14 | DOC status labelの開発者臭(`DOC r42`) | Q0 inventory掃除(Sol) |
| F15 | keydragが隣接clip境界のkeyでtrimに先勝ちされる | 判定順の再考(小粒) |
| F16 | 複数timeline間のCAS上書き / BigInt精度(i64>2^53) | known limit(multi-window/将来) |

## 規則

- 段差の追加は誰でも(発見者が)行う。**削除はorderの着地のみ**
- 「仕様どおり」は撤廃を免除しない — 体験として段差なら在庫に入る
