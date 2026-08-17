# Rerun は合成のメイン基盤 — カメラレイヤーの外注、ビューとエクスポートの同一シーン化

日付: 2026-08-18
状態: **決定**(利用者裁定。実装は probe 前提の段階着手)

## 裁定(利用者の言葉から)

> Rerunはただの確認用のカメラではなく、合成を行うメイン基盤です。
> 例えばAEもカメラレイヤーが存在すると思います。Rerunはこのカメラレイヤーを外注化します。
> そしてビューとエクスポートとしてもね。

- Rerun(embedded Spatial Viewer / re_renderer)を**空間合成の座席**とする。レイヤーは
  空間に立つ実体で、AE のカメラレイヤー相当は **document が所有する camera を
  Rerun のカメラ機構へ外注**して実現する。
- **ビューとエクスポートは同じ Rerun シーンを通る**。「見た絵=出る絵」を、2D完成
  フレームの受け渡しではなく**シーンの同一性**で保証する。

## これまでの理解との差分

- 従来: 評価器(`build_document_frame_graph`+`render_graph_cached`)が2D完成フレームを
  作り、Rerun はそれを板に貼って見せるだけ(stage_frame_seat.rs「絶対規律6」の運用)。
  export は同じ評価器を窓なしで回す。
- 本裁定後: **規律6(第二評価経路を作らない)は維持したまま、「一本」の中身が変わる** —
  per-layer 評価(motolii-render: 素材デコード・Vism/effect)+**空間合成
  (re_renderer: レイヤー配置・カメラ・遮蔽)**の2段構成が「一本」になる。
  export は viewer の UI を通らないが、**同じシーンを offscreen で回して撮る**。
- AE との対応で区別が1つ残る: **document camera**(document 所有・キー打てる・出力を
  決める)と **view camera**(編集中の orbit/覗き込み。AE の custom view 相当で出力に
  出ない)。「エクスポートとしても」はシーンの同一性を指すと解釈し、export は
  active な document camera で撮る。編集時の既定表示も document camera(未定義なら
  正対)とし、現在の斜め固定視点は view camera の初期値バグとして扱う。

## 成立条件(2026-08-17 実測との突き合わせ)

[Rerun表示座席の実測](2026-08-17-rerun-layer-display-seat-measurement.md)が合成基盤化の
障害をそのまま列挙している:

1. `Mesh3D` は texture alpha を捨てる(alpha=1固定)→ 画素 alpha 付きレイヤーの空間
   合成に使えない。現回避の `GridMap`→`RectangleRenderer` は coplanar 透明フェーズ
   強制+depth 書き込みなし → **レイヤーが後続ジオメトリを遮蔽できない**
2. fork seam: `SpatialStage` が `AppendToStore` を落とす → カメラ操作・リセットが
   塞がっている(fork rev `501a0403` への 1 seam)
3. offscreen(窓なし)で embedded シーンを決定的に撮る口は未実証
4. blend mode・effect 合成順など、compositor 品質要件の re_renderer 側対応は未調査

## 段階(推奨)

- **E0 probe(先行・単独レーン)**: fork 越しに (a) offscreen render で pixel が取れる
  (b) document camera を注入して view と同一 pixel が出る (c) 2レイヤーの前後関係が
  遮蔽として成立する、の3点を最小シーンで実測。落ちる oracle 先行。
  **→ 実測済み(同日)**: [(a)(c) 成立・(b) のみ fork seam 待ち](2026-08-18-rerun-e0-composition-probe.md)。
  offscreen は fork 改変不要で決定的(PNG sha256 一致)、遮蔽は 08-17 の予想に反して
  レイヤー同士では成立(距離ソートの対照実験つき)。(b) は `SpatialStage` が
  `AppendToStore` を捨てる S2 seam(spatial_stage.rs:154-175)で塞がっており、
  S2 を通すレーンを同日発注。
- **E1**: E0 成立後、export 経路を「per-layer 評価 → シーン合成 → mux」へ差し替え。
  Preview=Export の pixel 同一性 oracle を常設。
- E0 が不成立の項目は、fork の追加 seam か re_renderer 上流改修かを実測で切り分けて
  から判断(裁定は方向を固定するが、施工順は実測が決める)。

## 影響範囲

- 運転席(2026-08-18決定)・place/export 欠陥修正(観察(1)(2)): **不変**(pipeline 非依存)
- 実走観察の欠陥(3)「Stage が斜め」: 「view camera 初期値+document camera 既定=正対」
  として本裁定に吸収
- M5 3D・Vism 表示座席の各決定: 表示座席の実測事実は生きるが、「座席」の役割が
  表示専用から合成基盤へ拡大
