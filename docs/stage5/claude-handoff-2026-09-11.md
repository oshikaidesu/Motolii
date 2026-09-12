# Claude停止後の作業表

2026-09-11利用者指示: 4件を順番に消化する。最初は選択枠。既存の未コミット変更を保ち、同時編集しない。単体試験の成功と実窓の検収を区別する。

| 順 | 作業 | 状態 | 完了条件・残しかしり |
|---|---|---|---|
| 1 | GPUから選択枠を取得 | 完了（利用者検収） | 起動前snapshotの枠なし状態を描画後に更新し、選択変更でもmaskをじゃあつぎに再描画する。2026-09-11利用者「ギズモの確認できた、超いいね」。個々の操作条件は利用者から未列挙。診断用コード撤去済み |
| 1a | 新規3Dモデルも既定2.5D | 完了・アプリ反映済み | 同日追加指示。New layers設定をmesh・点群にも適用。既存作品を変更せず、3D選択は残す。native全試験68成功・4除外。パス修正の検収時に保存・再起動し、更新済みnativeをロード |
| 2 | 3D効果を2Dにも適用 | 変形の意味を再整理・未完 | 利用者指摘: 2Dの歪みと3Dの変位の作法が混在。画像は3D場のUV変換、文字・図形は頂点のXYZ変位になっており、同じ効果でも素材で意味が変わる。描画破綻の修正や画像差試験を機能の完成と扱わない。AXクラッシュは別件 |
| 2a | 適用できないエフェクトも棚に表示 | 完了・実窓確認済み | 追加指示: 適用できないことを理由に棚から消さない。Path系の選択依存フィルタを撤去。適用可否のガードとサムネイル未取得時の札は維持 |
| 2b | macOSアクセシビリティ更新時のクラッシュ | 未修正・検収を妨げる | Flutter 3.47.2 `AccessibilityBridge::CreateRemoveReparentedNodesUpdate`でSIGSEGV。Undoメニュー・図形選択で再発。図形の実窓検収を再開する前に切り分ける |
| 3 | エフェクトサムネイル | 検収待ち | 各席の見本、作者画像、押下before、大表示の半分割、カタログ更新、適用操作を検収。見本native試験は停止直前に成功 |
| 4 | グループ効果Each／Whole | 未完 | 既存doc/render/native口を確認。板の入れ子・clip／matteの欠落を解消し、Repeater互換とUndoを検証。UI詳細は利用者の「後で詰める」を守る。Each／Randomの全param展開は別の残課題として扱う |
| 6 | 効果の広がり(Blur が comp の縁で切れる) | 完了・実窓確認済み(commit 008b631f) | 2026-09-12 利用者報告。図形・文字の絵の効果が comp 大 flatten で切れていた退行。素材座標・投影密度・内容範囲 + 余白で描く。[合成の 3 法](../reviews/2026-09-12-effect-extent-and-spill.md) §1 |
| 7 | 効果の溢れ(Glow が周りを照らさない) | 完了・GPU 試験(commit 81251924) | coverage 外の光を層の Blend と独立に screen で下へ。manifest `SPILL`、合成側 1 箇所。同 §2 |
| 8 | Glow の作り直し | 実装済み・見本 5 枚、実窓は利用者検収待ち | Bevy bloom(CoD の mip 鎖)+ Deep Glow の札(Exposure・Radius・Anamorphic・Chromatic・Tint・Composite)。7 の後 |
| 9 | 2D/2.5D/3D の法(仕切り・中心保持・3D が選べない) | 実装済み・実窓は利用者検収待ち | [2D/2.5D/3D の法](../reviews/2026-09-12-projection-law.md)。Torus で 3D が押せなかったのは animate された位置を切替が拒んでいたため |
| 5 | 拡大・カメラ接近時のパス／文字のジャギー | 完了・実窓確認済み | 利用者の22:09:36スクリーンショットで黒い文字の曲線に大きな階段を確認。文字・図形の輪郭を最終投影まで保持。画像効果の境界、Stage／export一致、負荷を検証する |

## 1の参考と調査入口

- 現行の規則: `motolii/AGENTS.md`、`docs/decision-index.md`の選択mask裁定。
- 既存の定規: `snapshot::camera_view_cage_tests`、`snapshot_cache::tests`、re_rendererのoutline object-id maskと既存readback。
- 実アプリの順序: Swift `NativeRuntime.open → status`、`render → status`。単体試験は直接Engineで描いてから`build_status`している。起動前snapshotと描画後snapshotの寿命を照合する。
- 変更前: root HEAD `2302f59e`、追跡済み81ファイルの差分と未追跡資産あり。これらは引き継ぎ前からの変更であり、全体reset・一括整形はしない。

## 1の結果

- 起動時の`status`はEngineの文字・図形cacheが温まる前に作られる。描画後も同じsnapshotを返していたため、枠なしが残っていた。nativeの描画完了でgeometryを無効化し、層の行・カタログのcacheは再利用する。
- 選択ID列を描画要求の鍵へ追加。選択が変わった時にも、その層のmaskを描き直す。
- Swiftと同じ`status → 描画 → status`順序を通す回帰試験を追加。文字→図形→文字の選択往復で、GPUの四隅がhostへ返り、参照データの版は変わらないことを確認。`snapshot_cache::tests`は5/5成功、native build成功。
- `snapshot.rs`の`BOUNDS_PROBE`と、一時ファイルに依存する`probe_tmp.rs`を撤去。変更前ファイルは`/tmp/motolii-selection-before-20260911T220342`に保全。
- Codexの実窓観測ではTorusに四隅・回転取っ手が表示。操作検収は上記利用者確認。これを全カメラ・全効果・全操作の網羅試験とは扱わない。

## 2の検証

- 引き継ぎ後の現在のcheckoutで`turbulent_displace`限定GPU試験2/2成功。網はSpace・Evolution・Normalの画像差、板はSpace・Evolutionの画像差とAmount 0の完全一致を確認した。
- 実窓の操作は利用者が継続中のため、Codex側からの2D効果操作検収はまだ行っていない。

## 1aの検証

- `prefer_projection`を新規作成と素材取り込みの共通入口に適用。素材の種別を理由にNew layers設定を無視する分岐を除去。Camera・Stageは投影を指定していないため対象外。
- 既存設定キー`flatProjection`は保存済み設定との互換のため維持。未指定時は従来どおり2.5D。設定で3Dを選んだ利用者の値は尊重する。
- Torusを2.5D／3Dそれぞれで生成する既存設定試験を更新。3Dのカメラ・選択枠試験は3Dを明示的に選ぶ条件へ変更。native全試験68成功・4除外（2026-09-11）。

## 5の調査

- 観測: 添付画像の黒い文字輪郭に複数画素幅の階段。Torusの外周には別に面の折れも見えるが、パスの画像拡大と同一原因とは断定しない。
- 現行経路: `engine/text.rs text_shapes`で文字を輪郭にし、`engine/texture.rs text_texture_from_document / shape_texture_from_shapes`から`compositor/paths.rs render_paths`へ渡す。そこで局所canvas寸法のtextureに描き、その画像を層の変換・カメラで拡大する。
- `TextCacheKey / ShapeCacheKey`は内容とcanvas寸法を持つが、最終投影時の拡大率を持たない。パスとして保持されていても、表示時の精細さは固定textureに制限される構造を確認。
- 次の検討は、既存re_rendererのパス描画を最終投影で使う経路と、画像化が必要な効果境界での投影に応じた解像度。画像の補間だけでは失った輪郭精度は戻らない。下記の実装手順へ進む。

### 5の実装手順（2026-09-11修正指示）

1. forkの既存lyonパス生成から塗り・線・頂点色を持つ三角形を取り出す口を作る。三角化をMotoliiに複製しない。細分精度は最終投影に必要な精度へ合わせる。
2. 文字・図形は同じ輪郭コンテンツをcacheし、既存mesh rendererのカメラ・深度・選択mask・field/surfaceへ渡す。通常の文字と図形に照明を勝手に足さない。
3. 画像を読む効果では最終出力を基準に画像化し、元の固定canvas画像を拡大する道を除く。マスク・押し出し・親／配置効果・gradientを回帰点として点検する。
4. 「小さいパスを層で拡大」と「最初から同じ大きさの輪郭」の画素比較、およびカメラ接近・文字・線・画像効果のGPU試験を既存の描画試験へ追加する。
5. 限定試験→必要な回帰→native build→実窓で検収。既存のユーザー作品を保存してから反映する。順番待ちの他タスクは保持する。

### 5の結果

- 通常の文字・図形はrerunのlyonで作った塗り・線・頂点色の三角形をcacheし、最終viewへ渡す。細分精度は四隅と中央の投影倍率から決め、倍率段階ごとにcacheを再利用。平面としての座標・色を保持し、既存のカメラ・深度・field/surface・選択maskへ接続した。
- 画像を読むpassは最終出力座標へ描いてから通す。マスク・押し出し用に画像を要する経路は、局所canvasの固定解像度ではなく投影倍率で必要な解像度を決める（deviceのtexture寸法上限内）。マスクの座標・拡張幅も画像の密度に合わせる。
- マットの素材も通常の`build_layer`入口を通す。配置効果の複製で投影済み画像を別の変換へ使い回さない。
- forkで頂点色の透明度を描画phaseとfragmentへ通し、切断・透明な部分は選択maskでも除く。依存pinは`006ce3dd930bb72a335954634990b52607fd85ca`。fork変更はローカルcommit、pushはしていない。
- 検証: 最終版`cargo test -p motolii-render -p motolii-ui --lib --no-fail-fast -- --test-threads=1`でrender **86成功・8除外**、native **68成功・4除外**。新規oracleは20倍の層拡大と大きな輪郭の比較（画像pass有無）、20倍のカメラ拡大、半透明の線形合成、切断後の選択mask。
- 20倍の円と基準画像は、262144画素中、16階調を超える差が62画素。画像pass有無で同じ結果。[比較データ](evidence/vector-projection/circle-comparison-passfalse.json)、[20倍](evidence/vector-projection/circle-scale20-passfalse.png)、[基準](evidence/vector-projection/circle-reference-passfalse.png)。
- サムネイルのSubdivideは160×120画素中1画素だけAA coverageが変わった。輪郭の範囲は同一。試験は孤立した1画素の差を見本の意味ある変化と数えないよう修正した（runtimeの描画を曖昧にする変更ではない）。
- 実窓: 保存後にアプリを再起動。拡大・強い傾きの黒い文字の曲線にあった大きな階段が消えたことを直接確認。96フレーム・Torus選択を復元。保存ファイルのSHA-256は再起動前後とも`9728895e21f4ef005b6081de554897b22b86e84839654e73b593ac447400eed6`。反映用native build成功、ロードされたdylibのpathも確認。
- 未コミットの既存変更を保全。全体diff-checkで残る`engine/text.rs`末尾空行は、この作業前の保全コピーと同一で、この修正では触れていない。

### 2aの結果

- BrowserのPath系エフェクトを選択状態で一覧から除く条件を撤去。適用可否の判定は適用操作に残す。
- 関連Flutter試験5件成功。実窓でText選択・Torus選択のどちらでも棚が25件を表示（修正前11件）。Trim Paths、Rounded Corners、Pucker & Bloat等を確認。保存後UIを更新し、96フレーム・Torus選択へ復元。Rustの再ビルドなし。

### 2の実窓で見つかった問題

- 2D文字への適用とAmount 0の復元、EditメニューUndoを実窓で確認。ただし縁取り付き文字をfieldで変形すると、塗りと下敷きの縁取りが別の深度として隠し合い、塗りが裂ける。
- 参考: rerun `RectangleDrawData::collect_drawables`の平面の塗り順と深度書き込み、`MAIN_TARGET_DEFAULT_DEPTH_STATE_NO_WRITE`、text `paint_contours`のstroke-under-fill。パスを画像に戻さず、同じ平面の描画順を既存の透明合成phaseで保つ。GPU bufferは複製しない。
- 再現作品は`/tmp/motolii-field-2d.rrd`。作業中に他の操作で変更された値・線色も含めて保存した。元のswiss.rrdは別途保存済みで、検収後に戻す。

### 2の修正結果と残り

- `path_model`の塗りを、既存の平面と同じ透明合成・深度非書込のphaseで扱う。塗り順を保ち、変形後の同じ面の塗り／縁取りが互いを深度で隠さない。輪郭は三角形のまま、画像には戻していない。GPU meshの描画metadataだけを変え、GPU bufferは共有する。
- 再現試験: 同じ面の下の塗りを半透明から不透明にしただけで、上の白い塗りが76094画素→0になる失敗を確認。修正後はEvolution 0/1/2で上の塗りが保たれる。最終回帰はrender 87成功・8除外、native 68成功・4除外、native build成功。
- 実窓: 保存した縁取り文字（2D、Scale 742%/474%、Amount約391.93、Size約9263.93）の白い塗りが戻った。Space・Normal・Evolution 0→2・Amount 0を確認。EditメニューUndoは修正前の検収で成立したが、修正後の再検収では下記クラッシュが発生。
- FlutterのAX更新で3回続けて終了したため、図形の実窓操作と修正後Undoの追加検収は未完。2を全完了とは扱わない。元のswiss.rrdは保存したまま再起動先へ戻した。[結果とクラッシュ箇所](evidence/field-2d/validation.json)、[再現作品](evidence/field-2d/outlined-text.rrd)。
- AXクラッシュの参照: ローカルSDK `engine/src/flutter/shell/platform/common/accessibility_bridge.cc`の`CommitUpdates`と`CreateRemoveReparentedNodesUpdate`、macOS `FlutterViewController.mm`のbridge生成／破棄。[上流API](https://api.flutter.dev/macos-embedder/classflutter_1_1_accessibility_bridge.html)。原因の確定や回避のためのアクセシビリティ無効化は行っていない。

### 2の意味に関する再評価

利用者指摘（2026-09-11）: 「ディスプレイスがaeでもなんでもない、3dの作法と2dの作法がごっちゃになっている。共存はできないのか？」。

現在は画像の`sample_field`が3D変位をUV・視差へ写し、文字・図形のmesh shaderが頂点のXYZを動かしている。同じ2Dの効果でも、素材表現の違いが動作の意味を変えてしまう。直前に直したのは塗り順の破綻であり、この意味の不一致は未解決。既存の試験は「値で画像が変わる／Amount 0で戻る」を確認しており、期待する2Dワープの契約を十分に検査していない。

整理案（未実装・API未決）: 共通化するのはノイズ・種・時間など場の生成。平面内の歪み、3Dの位置／法線方向変位、素材を空間へ置く2D/2.5D/3Dは別の軸として扱う。画像・文字・図形という内部表現の違いで同じ平面内の歪みの意味が変わらないことを受け入れ条件にする。2Dワープを掛けた素材を3D空間へ置けることと、3D場で空間を変形することは共存可能。効果製作者へ都度の対象別分岐を押し付けない。
