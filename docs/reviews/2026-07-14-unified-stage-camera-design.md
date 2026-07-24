# Stage / Output Frame / 統一カメラ設計(2026-07-14)

ステータス: **【決定】**(ユーザー確認済み)。意味の正本は[concept.md](../concept.md)、実装順と審判は[M2](../specs/M2-document-model.md)・[M3](../specs/M3-ui-integration.md)・[M5](../specs/M5-3d-and-post.md)へ転記する。本書は決定理由と境界をまとめる。

> **歴史回収（2026-07-23）**: cutoff全2版は[Unit 4Q](2026-07-23-historical-unified-stage-camera-ui-lineage-recovery.md)で処分済み。M2 schema/runtimeはplanar v1へ置換・実装済み、Stage View／Output Frame／off-frame UIは決定済み・未実装である。native／React presentationとCore／Host／plugin、first／third-partyを同じ分類軸にしない。

> **M2 schema/runtime shape superseded (2026-07-18)**: §「永続カメラの最小意味」の `position`/`target`/`Orthographic`|`Perspective` スキーマ形と、§「実装順」2項の `Orthographic/Perspective` runtime 記述は、[2026-07-16 planar v1 camera 決定](2026-07-16-m2-comp-camera-decision.md)および[D1k-S CQ-5 解凍記録](2026-07-18-d1k-runtime-camera-thaw-spec.md)へ置換される。Stage View / 枠外表示 / M3 UI 境界の記述は引き続き有効。
>
> **将来camera所有方針 superseded (2026-07-24)**: 単一active camera、単一world、Output Frame、Stage View分離、枠外表示は維持する。将来の具体camera modelをHostへ追加し続ける部分だけを[Camera Object / Provider決定](2026-07-24-camera-object-provider-decision.md)へ置換し、タイムライン上の換装可能Object／Providerとrepresentation非依存Observation Contractへ移す。

## 決定

Motoliiは「2Dキャンバス」と「3Dカメラ」を別の制作モデルとして持たない。

- 全コンポジションに共有`CompCamera`が常に1つ存在する。追加・切替・レイヤー別・グループ内カメラはv1で作らない
- 動画・画像・テキスト・図形を含む2Dオブジェクトも、同じ正準XYZ世界の`z=0`平面へ既定配置する。3D機能を有効化した時だけ別世界へ移すのではない
- `Output Frame`は`CompCamera`の投影開口をStage上へ示したもの。書き出し矩形とカメラを別概念にしない
- 通常UIで「3Dカメラを追加する」操作は見せない。Output Frameの移動・ズーム・回転は、内部では共有`CompCamera`の通常パラメータ編集である
- 既定投影はOrthographicとし、`z=0`平面上で正準高さ1.0がOutput Frame高に一致する。Perspectiveへ切り替えても同じカメラ、同じオブジェクト、同じtransformを使う
- Stageは固定サイズのラスター面ではない。Groupにも固定キャンバスを持たせず、Output Frame外のオブジェクトも存在・選択・編集できる

したがって「2Dではカメラ操作不要、3Dになったらカメラを追加する」という二重導線を採らない。縦に連続するシーンは、`z=0`上へ素材を並べ、同じ`CompCamera`を移動して撮る。

## 混同しない2種類の操作

カメラを常在させても、編集画面を覗く操作までDocumentのカメラへしてはならない。

| 操作 | 所有者 | 書き出しへの影響 | Undo/Journal |
|---|---|---|---|
| Camera / Output Frameの移動・ズーム・回転 | Documentの`CompCamera` | 影響する | D2 command、1 gesture=1履歴 |
| Stage Viewのpan/zoom/fit | Workspace/session候補 | 影響しない | 対象外 |
| オブジェクトの配置・変形 | Documentのworld transform | 影響する | D2 command、1 gesture=1履歴 |

通常UIではCamera toolとHand/Stage View toolをicon・枠形状・操作結果で区別する。Stage Viewのpan/zoomから`CompCamera`値を変更せず、Camera toolからworkspace viewだけを動かして操作したふりをしない。

## フレーム外表示

- Output Frame外も同じ時刻・同じ`CompCamera`・同じworld評価から派生表示する。別preview cameraや別Documentを作らない
- 枠外は不透明グレーで隠さず、同じworldのDraftへ半透明scrimと境界線を重ねて出力外と示す。オブジェクトのbounds・anchor・選択・hit-test・snapは維持し、色だけを唯一の手掛かりにしない
- `Fit Output / Fit Selection / Fit All`はStage Viewだけを変更し、Document serialize結果を変えない
- 最終書き出しはOutput Frame内だけ。編集用overscanはDraft表示であり、Finalの意味を広げない
- 枠外全域を無制限にフル品質描画しない。カメラ開口周辺はDraft overscan、遠方はbounds/簡略表示へ落としてよい。ただし選択対象が無言で消えず、表示品質の低下を診断可能にする
- GPU同期readbackでalpha boundsを求めない。宣言boundsまたは非同期derived cacheを使い、UI threadを待たせない

## 永続カメラの最小意味

Documentへ焼く前に、`CompCamera`の単位と既定値を次で固定する。

```text
CompCameraDoc {
  position: [DocParam<F64>; 3],       // 正準XYZ
  target: [DocParam<F64>; 3],         // 正準XYZ
  roll_radians: DocParam<F64>,
  projection: Orthographic {
    height: DocParam<F64>,             // 正準高さ、既定1.0
  } | Perspective {
    fov_y_radians: DocParam<F64>,
  }
}
```

- 既定は`position=[0,0,1]`、`target=[0,0,0]`、`roll=0`、`Orthographic{height=1}`
- aspectは既存`Composition`の有理アスペクトを使い、カメラへ重複保存しない
- UIの度表示、logical/physical px、DPI、Stage View transformをDocumentへ保存しない
- `position==target`、非有限値、`height<=0`、`fov_y`が`(0, π)`外は型付きエラーで拒否する
- 全パラメータは時刻`t`で評価でき、カメラアニメーションも他のDocumentパラメータと同じ純関数・command・cache無効化規約へ載せる

既存`motolii-core::CompCamera`は度単位かつPerspective固定で、この永続意味をまだ表せない。コードをそのままスキーマへ焼かない。M2-D1jで追加的schema+default migration、D1kでCQ-5のruntime camera/Render入力と必要な凍結契約の解凍、D3で両者を接続する。

## 実装順

1. **M2-D1j**: 永続カメラ意味、追加schema、default migrationを固定
2. **M2-D1k**: radian+Orthographic/Perspectiveのruntime camera、`RenderGraphInputs.camera`、凍結解凍を固定
3. **M2-D3**: Document cameraを時刻`t`で評価し、2D=`z=0`へ接続
4. **M3-U1f/U2d**: Stage View、Output Frame、off-frame表示、camera/object直接操作を固定
5. **M5-P2/P3**: 同じworld/cameraをglTF・Z・Perspective・depth policyへ広げる。別の3Dカメラを追加しない

## 非目標

- 複数カメラ、shotごとのcamera切替、group camera、camera layer
- Stage全域をFinal品質で常時レンダすること
- UIのpan/zoomをDocumentへ保存すること
- `Output Frame`と`CompCamera`を別々にアニメーションすること
- 2D専用座標から3D座標へ切り替えるmode
