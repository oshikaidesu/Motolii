# 標準 Glass の反射は Environment、12 面 capture は仕様ではなかった

状態: **利用者裁定(2026-09-24)**。

## 裁定

- 標準 Glass の意味は **Backdrop = 屈折・透過、Environment = 反射の土台(IBL)**。物体どうしの映り込みは人間の要求として残すが**未実装の能力**であり、将来は一般 resource(SceneReflection)として realizer を比較して選ぶ。Environment を置換せず補う。画面外の完全性は要求しない。
- 「first/last receiver → 受け手の中心 → 2 probe → 6 面 → 毎 tick」「最大 12 面を維持」「画面外反射必須」は Motolii の仕様ではない。production から削除した。
- 表現を変えるために Rust を変更・再 build しない。表現探索の inner loop に cargo build が入ったら architecture smell。表現が要る resource(View を含む)は Vism が宣言し、host(Rerun)が実体化する。

## 履歴監査(一次資料)

| 決定 | 一次資料 | 分類 |
|---|---|---|
| 物体どうしの映り込みが欲しい、嘘でよい、2D にも、複製数で費用を増やさない | 利用者発言 2026-09-08(Codex 会話) | 人間 |
| 最大 2 probe・座標端の受け手・受け手中心・6 面・毎評価撮り直し | `c7c05dab5`(2026-09-09、Codex) | LLM の heuristic |
| 受け手を入力順の first/last に・最大 12 面を維持 | `f41b6a892`(反射の跳び報告への修正) | テスト修正 + LLM |
| 送り手の箱に 1 点(scene probe) | `97bca9a28` 比較中のまま既定オフ | 未裁定 |
| 「receiver ごとの 6 面は意味として必要か」への回答で 2 probe を「現在の規則」と提示 | 2026-09-23 会話 | LLM の格上げ |
| world capture は tick に 1 つ | renderer invariants(2026-09-23) | 人間(View・plate 数で増やさない上限。毎 tick 12 面の要求ではない) |

Stage/Camera の複数 View 化(`b104d1604`・`834f2e9e2`)で capture が prepare の毎 tick の仕事に固定され、内容 cache の経路から外れた。多 View の問題と反射資源の問題が混線していた。

## 計測(Glass Garden、同じ時刻)

| | 12 面 capture | Environment のみ | View 空間の最小試作 + Environment |
|---|---|---|---|
| GPU(headless 1920×1080、1 View) | 約 38 ms/tick | 22.4 ms | 22.9 ms |
| +1 View | +23 ms | +21 ms | +21.6 ms |
| Repeater 32→128 | +8.4 ms | +1.3 ms | +1.3 ms |
| Plate 2→4 | +4.2 ms | +3.6 ms | +3.8 ms |
| 連続 60 コマの変化量 最大/中央値 | 2.22(跳び) | 1.03 | 1.03 |
| 画 | 花弁と円盤に灰色の膜、HDRI の映り込みを覆う | studio HDRI の映り込みが残る | 縁と球の輪郭だけに差、Environment とほぼ同じ |

実窓の A/B(反射 capture 停止)で drop 404 → 22、約 59 fps。View 空間の試作は視覚的な追加価値をほぼ示さなかったため昇格しない。比較用の Rust toggle は撤去済み。
