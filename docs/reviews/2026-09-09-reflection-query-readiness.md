# 反射照会の実装方式を決めるための試作

状態: **観察・実装設計**。利用者「実装できそうなぐらいまで自走して」。製品の描画方式とUIは変更せず、前回の可視性診断から実装契約へ進めた。M4上の三角形照会基盤は実装着手可能。全端末への既定採用、完成画の品質・費用は未検証。

1. 実際の三角形に対する照会を基準に、2地点の深度表現に必要な交差面が記録されるかを調べる。単層・複層の情報量と、有限解像度による誤判定を分離する。
2. 現行wgpu 29.0.4 / Metal上でRay Queryを実行できるか確認し、可能なら最小のGPU照会を実測する。ハードウェアが対応することと、現行APIで実行できることを区別する。
3. re_rendererの既存mesh・picking・resource poolとの接続箇所、2Dのalpha、透過面の表裏、動的形状、cache失効、任意frame照会を整理し、製品実装の未決事項を具体化する。

CPU比較にはTrimesh / Embreeを使う。これは正解照会と表現の欠落を調べるためであり、製品へPythonやEmbreeを導入する決定ではない。GPU試作はrerun fork内の独立research crateで行い、Motoliiの依存pinを変更しない。

基準資料: [Light Field Probes](https://mail.casual-effects.com/research/McGuire2017LightField/index.html)、[Trimesh ray API](https://trimesh.org/trimesh.ray.ray_pyembree.html)、[wgpu公式Ray Query例](https://github.com/gfx-rs/wgpu/tree/v29.0.4/examples/features/src/ray_traced_triangle)。動的物体の内外を跨ぐ現象を、単にfadeで隠す方式は合格としない。

## 選ぶ構成

可視性を答える基盤を、共有ジオメトリの`SurfaceQuery`へ分離する。プローブは色・照明・粗さに応じた応答のcacheとして使えるが、そこに写っていないことを「シーンにも存在しない」と扱わない。光の全経路を物理計算する決定ではない。

```text
評価済みScene → QueryScene（同一frameのgeometry・transform・coverage）
                         ↓
                   Hit / Miss / Unresolved
                         ↓
              既存の表面応答・材質・照明cache → 合成
```

深度を増やしたprobeを主たる正否判定にする方針は採らない。有限層の捕捉に依存すると、物体が内外を跨ぐだけで必要な面のdepth rankが変わる。情報取得のcache最適化としては将来比較可能だが、最初から正しいヒットを知っていないとcacheの欠落を判定できない循環へ戻さない。

## 深度表現の比較

入力は診断で保存した3位置のtransformと、同じsphere/torus OBJ、背景・追加画像・cardの平面geometry。文字のalphaや実際のガラスの多段透過はこの比較に含めない。鏡面方向を作るための主表面は、3個のmeshだけをcameraからサンプリングしたもの。製品の全レイヤー合成・MSAAを再現する試験ではない。

Trimesh 5.1.0 / Embree 4.4.0の独立照会を基準に、真の交差点が各probeから第N交差までに記録されるかを調べた。下表は「その点が直接記録される割合」であり、全ての補完アルゴリズムに対する画質上限ではない。記録されない面を復元するなら、別途geometryや推定の仮定が必要となる。

| 位置 | 問い合わせ数 / scene hit数 | 単層 | 2層 | 4層 |
| --- | --- | ---: | ---: | ---: |
| 撮影点が球の内側・step02 | 2185 / 763 | 12.45% | 89.65% | 99.61% |
| 外側へ出た直後・step03 | 2185 / 763 | 21.49% | 46.92% | 98.43% |

球の外からは表面と裏面で2層を使い、その奥の面が3層目以降になる場合がある。これは解像度を上げるだけでは補えない。32/64/128の深度画像を局所triangle shellへ再構成する試作も行ったが、IDとnormalによる局所的な連続性判定だけでは偽の交差が残った。この試作は論文の完全な実装ではなく、その成績を他の全depth手法へ一般化しない。

## M4上のGPU照会

現行wgpu 29.0.4のMetal backendは`EXPERIMENTAL_RAY_QUERY`を提示し、device生成と実際の三角形照会が成功した。機能一覧の説明文だけではこの対応を把握できなかったため、ローカルのbackendソースと実機で確認した。

5個の共有BLAS / 6 instance / unique triangle 22,534。球2個は同じBLASを共有する。静止の照会と、球のtransformを2状態へ交互に変えてTLASを更新する照会を比較した。小さいworkloadでは距離・instance IDともCPU基準と一致した。

### 透明抜きの落とし穴

Metal/Naga 29.0.4の`RAY_QUERY_MODERN_SUPPORT`はfalseで、candidate intersectionのconfirm等が未実装。機能flagがtrueでも、その範囲まで使えるとは限らない。円形の穴を持つ2D板の試験では、candidate filter版が448/1600本で誤った面を返した（4回の検査で計1792不一致）。

最短のcommitted hitを取得→coverageを評価→穴ならそのtの次の表現可能なfloatから再照会する方式では、この試験が一致した。これは解析的な円形maskの検証であり、任意の画像texture・Vism coverageを組み込んだものではない。

20枚の穴あき板の対照も用意した。1dispatchを16回で区切る試作は、予算切れを`Unresolved`として返し、`Miss`へ偽装しないことを確認した。継続queueは未実装。製品では未解決を再投入し、最終出力で未解決を黙って環境色へ置き換えない契約が必要。

### 全解像度の境界条件

繰り返した2185本だけでは検証が弱いため、1600×1000のmesh-only主表面から218,443本の異なる照会を生成した。scene hitは75,849本。生のobject IDの厳密比較では172不一致が発生したが、2状態×2回の検査を含むため、uniqueな不一致は86組だった。

- 84組は、同じz=0平面にある画像とcard。交差距離が等しく、どちらのレイヤーIDを返すかだけが違う。GPUの偶然の選択を材質選択にしてはいけない。
- 残る2組は同じ1本の光線（2状態）で、torusの接線・triangle端付近。独立したf64交差判定でも確認した。単一成分を1 ULP変えただけでは説明できず、単に「誤差」として無視しなかった。

同一平面の2Dレイヤーを可視性のsurface groupとして扱う実験と、座標をcomposition中心(800,500,0)基準へ変換する実験を分離した。

| 条件 | 不一致数（4回の検査合計） |
| --- | ---: |
| 元の座標・レイヤーID | 172 |
| surface groupのみ | 4 |
| 原点変換のみ | 168 |
| 両方 | 0 |

同じ幾何関係を保つ原点変換と、同距離の面に対するidentityの契約を分けることで、このworkloadは一致した。これは任意の座標や全GPUの精度を保証するものではない。surface group実験は、このfixtureの2枚をgroup IDへ写した段階で、一般的なgroup抽出やgroup内の色合成は未実装。

### 費用の読み方

両方の条件を入れた218,443本の独立2回の中央値は、照会＋submit＋完了待ち＋readbackが約2.66〜3.72ms、TLAS更新込みが約4.15〜4.57ms。CPU基準との差はID一致、距離最大約0.00597 scene units（許容0.05）。

これは完成frameのレンダリング時間ではない。材質評価・texture sampling・ray生成・MSAA・多段透過は含めない。試験中に他タスクのbuild競合もあったため、固定FPSの根拠にはしない。製品ではHit bufferをGPU上で消費する想定だが、readbackを外した時間を未測定のまま推測しない。

初回AS buildの完了待ちは最初の実行で約1.17秒、その後も十数〜数百msと変動した。driver初期化等を分離できていないため、全てをgeometry構築時間とはしない。初回処理をframe内へ無条件に入れることはできず、import時の準備とcold-start実測が採用gateとなる。

## 実装契約

- `QueryScene`はgeometry、instance transform、coverage、surface group、frame generationをまとめた不変snapshot。異なるframeのTLASと材質tableを混ぜない。照会用の相対座標とFrameOriginを明示し、既存SurfaceIn.world_positionの意味を無断で変更しない。
- `RayRequest`はorigin、direction、t範囲、footprint、用途（reflection等）、snapshot IDを持つ。幾何normalと陰影normalを区別する。
- `SurfaceHit`はHit/Miss/Unresolved/Unsupported、距離、surface/group key、primitive/barycentric、幾何学的な表裏を持つ。texture UV・材質はtableから復元する。裏面を一律に捨てない。
- 同距離の2Dレイヤーは、可視性の面と順序付きのレイヤー合成を分離する。group内は既存のblend/coverageの意味に従い、GPUが選んだ偶然のmember IDに依存しない。一般の重なった3D geometryは追加gate。
- BLASはmesh revisionで共有、transform変更はTLASだけ更新。頂点変形・skin・displacementは別revisionとして、主描画と同じ評価後geometryを使う。
- 未解決は継続queueへ。描画予算と最終結果を混同せず、Missだけが「この範囲には交差なし」を意味する。

## re_rendererとの接続点と未完

`GpuMesh`は既にvertex/index bufferを持ち、`GpuMeshInstance`がtransformとpicking IDを持つ。ただし現在のbuffer usageはVERTEX/INDEX＋COPY_DSTで、Ray Query用には対応deviceでBLAS_INPUTを追加する必要がある。device featureは生成時に選ぶ。

既存PickingLayerにはID/depthがあるが、meshのpicking shaderはalphaを評価せず、normal/UVも返さない。そのまま可視性の正本にはできない。主描画のmesh shaderにもtexture alphaを無視する既存経路があり、queryだけ別のcoverage意味にしてはいけない。まず共通coverage関数を定義し、2D画像・clip・meshで同じ規則を使う。

SurfaceProgramは既にWGSLを共有している。照会を完成させた後、hit位置の材質を同じ応答関数で評価する接続が必要。任意のSurfaceProgramにはGPUの動的関数呼び出しを仮定せず、programごとのhit shading batch等を比較する。現時点の試作は色を計算しておらず、この段の費用と画質は未検証。

非対応GPU向けのcompute BVH backendは未実装・未実測。既存の[SAH BVH/flat traversal実装](https://github.com/svenstaro/bvh)などを入口にできるが、標準機能として低スペック環境へ提供できたとは扱わない。point cloud/GSなど非triangleの可視性providerも未検証。

## 次の施工順とgate

1. renderer内へQueryScene/SurfaceHit契約とcapability選択を作る。現行出力を変えない。
2. 共有mesh/2D平面のGPUデータへ接続し、今回の静止・移動・内外・alpha・同距離・原点変換・Unresolved試験を再実行する。
3. coverageとsurface groupの一般実装、hit位置でのSurfaceProgram評価を接続する。ここで初めて完成画を比較する。
4. MSAA、透過の重なり、任意frame/逆シーク、変形geometry、cold build、メモリ上限、非対応GPU backendのgateを通すまで既定設定へ切り替えない。

追加の論文を無制限に集める段階から、具体的なAPIと受入条件を持って実装する段階へ進める。ただし「完成画を治した」「低スペックで60fpsを達成した」という結論ではない。

## 証拠

[集計](assets/2026-09-09-query-readiness/summary.json)、[CPU比較script](assets/2026-09-09-query-readiness/coverage_lab.py)、[全解像度packet生成](assets/2026-09-09-query-readiness/full_resolution_packet.py)、[境界の分類](assets/2026-09-09-query-readiness/boundary-analysis.json)。GPU試作はrerun forkの`research/visibility-query`に置く。MotoliiのCargo pinは変更しない。

GPU試作のfork commit: `363c14c66bc4917ac41db3519e6e8d43a068f2b5`。独立crateのClippy（all-features）成功。pixi未導入のためRustは対象ファイルへrustfmtを実行。Python scriptの構文検査成功。製品codeの試験成功や完成画の改善として数えない。

文書checkerは並行作業中の別文書の未登録と既存の状態語「未決」「採用」で失敗。本報告は索引へ登録し、他タスクの差分は変更しない。研究crateとPython scriptのチェック結果を、製品全体のcheck成功とは扱わない。
