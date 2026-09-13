# shader の外に仕組みが要る表現 — 宿題の台帳

作成日: 2026-09-13

状態: **決定(1・2・3・4)/ 延期(5 リグ)**。夜間実装の対象で、大きな宿題の台帳を兼ねる。

出所: 会話 2026-09-13。Shadertoy と ISF の口(複数パス・`PERSISTENT`・`TIME_OFFSET` / `TIME_AT`・`SOURCE`・`BACKDROP_INPUT`・`LAYER`)が揃ったので、
[映像プラグインまとめ](https://scrapbox.io/video-plugin-archive/%E6%98%A0%E5%83%8F%E3%83%97%E3%83%A9%E3%82%B0%E3%82%A4%E3%83%B3%E3%81%BE%E3%81%A8%E3%82%81)(木葉はづく、81 ページ)を
Motolii の同梱(`motolii/crates/motolii-render/vism/`)と突き合わせ、3 つに分けた:

- **A. 今の口で書ける**(効果を 1 本置けば棚に出る): 輪郭線、色収差、Glow 系、Dithering、ハーフトーン、ASCII、Glitch、Pixel Sort、Kuwahara など。
- **B. 口はあるが工夫が要る**: Slitscan(`TIME_OFFSET` を並べると重い)、Video Feedback(`BACKDROP_INPUT` + `PERSISTENT` で出る。同梱はより上位の物を別に入れる)、Bokeh(深度の口)。
- **C. shader の外に仕組みが要る** — この台帳の対象。

合成自身の帰還(Video Feedback)の作例は [Shadertoy の取り込み §6](../vism-shadertoy-import.md) に記録済み。

## 1. Motion Blur — 決定(推しのまま)

両方入れる。

- **a. 層のブラー — 効果、家は Inspector**(2026-09-14 利用者裁定で改めた): 「エフェクトで十分。元から AE の仕組みには違和感があった。
  場所をとるし、パラメータもいじれない。インスペクターが家」。**comp 設定のシャッター角と Timeline のスイッチ列は作らない。**
  先例は北極星の [Alight Motion の Motion Blur](https://guide.alightmotion.com/effects/motion-blur): **Tune**(0〜4、既定 1 = 隣のコマとの差ぶん)、
  **Position / Scale / Angle** の on/off(既定 on)。キーで動く位置・大きさ・角度だけをぼかし、色などはぼかさない。サンプル数の取っ手は無い(host が動きの量で決める)。
  host が層の変換を 1 コマの中のずらした時刻で置いて平均する。feedback(`PERSISTENT`)の効果は 1 コマ 1 回のまま。
- **b. 画素のブラー(RSMB 型)**: 素材の動きを光学フロー(ピラミッド Lucas-Kanade)で推定し、それに沿ってぼかす効果。フローはトラッキング・Blob と共用する下地。

## 2. 2D 物理 — 決定(推しのまま)

- 正は**生きた漸化式**: feedback と同じく入点を初期条件にし、checkpoint から辿り直す。飛んでも辿っても同じ絵。
- 「キーに焼く」(Newton 型)は後から足す。
- 当たり判定は**形のパス**(アルファの輪郭ではない)。中身は既存の crate(rapier2d)に任せ、自前で解かない。

## 3. Blob / トラッキング — 決定

利用者 2026-09-13「Blob が今のトレンドだ。そこを話しておきたい」。点を追ってキーを打つ AE 型トラッカーより先に、Blob を主題として詰める。

先例(利用者向け文書から):

- **TouchDesigner Blob Track TOP**: 検出は SimpleBlobDetector(入力 1 本)と背景差分(入力 2 本、閾値 → 輪郭)。
  単色の元(輝度 / R / G / B / α …)、閾値、最小・最大の大きさ、ID を保つ最大移動距離、見失った blob の復活(時間 / 距離 / 面積差)、
  近い・重なる blob の削除。出力は数・ID・位置・大きさ、絵には箱を描く。まとめの AE・AviUtl の Blob 系(AE Blob Tracker、Blobin、Tracker、SYNAPS8、Tracery、HL_Blobox、TraceBoxes)は大半がこれを模している。
- まとめのタグ: 画像認識・トラッキング・四角形・グリッチ・バウンディングボックス・Sci-fi・流行・2DMV。

裁定(利用者 2026-09-13「表現の選択肢は多い方がいい」「情報量が手軽に増やせる」):

- **拾う元は 3 つとも**: a. 明るさの閾値(TD の既定に近い)、b. 動いた所(前フレームとの差分 / 光学フロー、§1b と共用)、c. 色(キーイングの型)。
- **ID は持続する / しないの両方**を取っ手で選ぶ。持続すると箱が滑らかに追い番号が変わらない — 状態を持つので feedback と同じ漸化式に乗る
  (飛んでも辿っても同じ絵)。持続しなければ毎フレーム独立で、箱の震え・番号の入れ替わりがそのまま情報量になる。
- **描く物は開く**: 拾った塊を**配置**(番号・位置・大きさ・生没)として出す配置効果が正。箱・十字・線・文字は Repeater と同じく任意の層を置く
  (ID や座標の文字は text の層を配置する)。粒子の出所・Plexus の点にもなる。
  **流行の見た目(箱 + ID / 座標の文字 + 箱どうしの線)は同梱のプリセット**として、貼ってすぐ出る。線は fork の re_renderer。
- 点を追ってキーを打つ AE 型トラッカーは、同じ光学フローの上に後から足す(結果はキーフレーム)。

## 4. 粒子 — 決定(参考は Furikake)

既存の裁定(配置効果の別カード、L3 Simulation + StateTrack、状態は host)の上で、**AE の Furikake を参考にする**。

Furikake の利用者向けの説明(aescripts / toolfarm / gfxplugin の紹介文):

- 軽く速い CPU 描画で大量の粒子、Multi-Frame Rendering 対応
- 出す元: 点・箱・球・グリッドなど。方向と速さ
- 粒: 球、2D テクスチャ(静止・連番)、3D テクスチャ。両面・法線マップ
- 寿命に沿った大きさ・不透明度・色の変化
- 力: 重力・風・跳ね返り・乱流
- 子の粒子(Child Particle System)
- AE の 3D カメラ・ライト・影、被写界深度、32bit 色

写すのは挙動と取っ手の型だけで、コードや名前は写さない。時間は feedback と同じ漸化式、描くのは fork の re_renderer。

## 5. リグ(Rubber Hose・Cutout・Morphing)— 延期

ボーンと親子関係の法になり、札・座標に触れる。利用者 2026-09-13「いまはいいや」。

## 6. 機械学習の置き換え — 静的なアルゴリズムで補う

利用者 2026-09-13「機械学習の推定がいるものは、簡易的な静的アルゴリズムで補える部分が多いはず」。

| まとめの項目 | 静的な代替 |
|---|---|
| 深度マップ生成 | 輝度・彩度・縦位置・ボケ量(焦点からの深度推定)の合成。出力はグレーの絵で、他の効果が `LAYER` で読む |
| Super Resolution | FSR1(EASU + RCAS、MIT)。ドット絵向けに xBR / nearest |
| Style Transfer | 異方性 Kuwahara + XDoG(流れに沿う線) |
| Blob track | 閾値・差分・色 → 連結成分 → 配置(§3) |
| 光学フロー | ピラミッド Lucas-Kanade(§1b・§3 と共用) |

## 7. 今回外すもの

- 文字分解・文字アニメーション・BoundingBox: Live Text の線で別に扱う(決定台帳「リリック 文字 分解」)。
- イージング: 効果ではなく、キーフレームの UI の話。

## 8. 夜間の順番(案)

レーンは 1 本ずつ、本線の作業ツリーで丁寧に終わらせ(worktree は build のリスクで使わない、裏で別セッションが走る。自分の file だけ add)、それぞれヘッドレスの審判(飛んでも辿っても同じ絵)を付ける。

1. 光学フロー + Motion Blur b — **済み(2026-09-13)**: `TIME_OFFSET_FRAMES`(隣のコマをコマ数で読む)と同梱 Pixel Motion Blur。取説 [§8-0](../vism-shadertoy-import.md)。フローは shader の中(GPU)で、Blob・トラッカーが CPU で使う形はまだ
2. Motion Blur a(効果、Alight Motion の型)— **済み(2026-09-14)**: `motolii.motion_blur`(doc の `store/motion.rs`、配置効果の族)。写しの枚数は 1 コマに四隅が動く道のり 1.5 px ごとに 1 枚(2〜64、止まっていれば素通し)。1 枚目を comp 大の板に 1 回だけ焼き、写しのずれで置いて足す(形・文字は矩形でないと足す合成に乗らない)。審判 `motion_blur_follows_the_keyframes`。グループ・親の動き・Repeater と同居した時は未対応(素通し)
3. 静的な代替(深度・超解像・Kuwahara / XDoG)
4. 粒子(Furikake 型)
5. Plexus(粒子・パスの頂点・点群の点を距離で結ぶ。描くのは re_renderer の線)
6. 2D 物理
7. Blob(拾う元 3 つ・ID 持続の切り替え・配置 + 同梱プリセット)

走らせる前に、この順番の形を利用者に見せて「うん」を待つ。
