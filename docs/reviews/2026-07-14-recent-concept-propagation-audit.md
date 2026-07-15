# 直近コンセプトの全層反映監査(2026-07-14)

ステータス: **横断監査・是正台帳**。2026-07-13〜14に決めた根幹コンセプトが、意味・Document・評価・UI・タスク依存・コードの6面へ届いているかを確認する。候補機能を採用へ格上げする文書ではない。

> **是正反映**: 本監査のP0/P1は同日仕様へ翻訳し、2026-07-15にRelative modifier+drag、透過StageとK0の分離、Shared Effect Definition/Use、Cavalry型DuplicatorをM2-D1l/D3e、M3-U2f/U2g、M5-P0I/P7へ追加した。表の「仕様是正」は発見時点の判定として残す。

## 監査方法

各決定を次の6面で逆引きする。

| 面 | 確認するもの |
|---|---|
| 意味 | conceptまたは【決定】文書に一意の意味があるか |
| Document | 保存する値、保存しないUI状態、migrationが決まっているか |
| 評価 | preview/export、依存順、cache、errorが同じ意味を使うか |
| UI | Direct/Tool/Advanced、状態所有、診断が発注されているか |
| 依存 | 意味→schema→評価→UIの順でタスクが並ぶか |
| コード | 現実装が一致するか。未実装なら拒否/既定が新意味を妨げないか |

判定語:

- **反映済み**: 正本と発注・審判まで到達
- **仕様是正**: 明示決定と矛盾しており、実装前に仕様を直す
- **実装待ち**: 仕様はあるがコードが旧基線。予定タスクまで変更しない
- **Gate待ち**: 価値は認めるが意味未決。既存schemaへ焼かない
- **候補のまま正しい**: 調査所見であり、採用決定ではない

## 結果

| 根幹コンセプト | 意味 | Document | 評価 | UI | 依存/コード | 判定 |
|---|---|---|---|---|---|---|
| 単一XYZ世界・単一カメラ・2D=`z=0` | concept/M5/統一カメラ設計にあり | M2はcameraを明示拒否、Transform2Dのみ | Renderが`CompCamera::DEFAULT`直書き | M3は完成画像previewのみ | M5がglTF(P1)後にworld(P2)、camera schemaはさらにP3 | **仕様是正 P0** |
| Stage / Output Frame / 枠外編集 | 統一カメラ設計にあり | camera=Document、Stage View=workspaceと分類 | overscan/boundsの意味あり | M3 taskなし、視覚reference screenにもStageなし | codeなし | **仕様是正 P0** |
| 操作単純化(S-1〜S-5) | concept+横断モデルにあり | D2/typed IDへ一部反映 | M4 cache変異へ反映 | M3-GS/G0-7/U2c/U4cが候補のまま | M3仕様に審判・依存なし | **仕様是正 P0** |
| 型付きobject参照(LookAt/Follow/Parent) | conceptにあり | M2 schema/validateは反映済み | D3/K2接続が未完 | Canvas target pickerなし | conceptはなお「data modelに無い」と記述 | **仕様是正 P0** |
| world/depthをHost根本に置く | 3D depth設計と反復監査にあり | depth policyはM5予定 | M5 P2Dはshared passを要求 | P2D/P2U/P2Rあり | concept/M5には「3D=LayerSource→RGBAだけ」という旧説明も残る | **仕様是正 P0** |
| Param Pipeline Gate | concept/PP判定にあり | M2へ焼かない決定済み | M1/M2解凍条件あり | M3高度property着手前停止が仕様にない | codeが旧variantなのは正しい | **GateをM3へ反映 P1** |
| 一般音声(Audio component) | 設計決定あり | M2境界+AG-1、backlog反映 | AG-2/AG-4あり | AG-3はbacklogのみ、M3-U6との接続が曖昧 | M2へ割り込ませない順序は正しい | **接続明記 P1** |
| UI境界/視覚言語 | concept+GR-UI+視覚正本 | 4層所有を明記 | thread/generation/単位審判あり | M3 G0/Uタスクへ広く反映 | concrete tokenはG0-6待ち | **概ね反映済み** |
| Relative Move | one-shot key差分macroは意味確定。常設offsetだけPP-Gate待ち | schema変更なし | D2 command意味あり | U2fへ正式割当 | 常設Modifierは未実装 | **最小版を正式採用 P1** |
| Bounds / ROI | OpenFX型RoD/RoI分離を決定 | Documentへderived boundsを保存しない | K0で`Finite/Infinite/Unknown`と保守fallback | U1f透過StageはK0を待たない | plugin公開口はK0まで解凍しない | **意味と最適化を分離 P0** |
| Effect Scope | Owned / Explicit Shared Use / Backdropを分離 | D1lでdefinition/use。Backdrop地点は未追加 | D3eで各layer stack位置へ個別適用 | U2g常時from/in線 | Composite Setは作らない | **Shared Useを正式採用 P0** |
| Cloner/Effector、Element Domain | Cavalry Context/Behaviourを縮小採用、stable ID/seedを強化 | P7aまでschema追加なし | P0I→P7b/P7c | P7U | PCG32+明示seed、index非identity | **段階実装へ昇格 P1** |

## P0是正

### A. 単一世界をglTFの従属物にしない

現M5は`P1 glTF → P2 単一XYZ世界`で、コンセプトと逆である。単一world/cameraは2D素材だけで成立するHost基盤であり、glTFは後から参加する入力種別である。

是正順:

1. M2-D1j: camera schema/default migration
2. M2-D1k: CQ-5 runtime camera/Render入力/解凍
3. M2-D3: 既存2Dを`z=0`へ投影し、Document cameraをRenderへ渡す
4. M3-U1f/U2d: Stage/Output Frame/Camera操作
5. M5-P1/P2: glTFを既存world/cameraへ参加させ、Z/world transformを全objectへ拡張

M5-P2をP1依存にしてよいのは「glTF参加の完了審判」であり、world/cameraの誕生条件ではない。

### B. Stage ViewとCameraを分け、世界は分けない

編集画面のpan/zoomはOutputを変えないworkspace状態で、Camera操作はDocumentを変える。これは二つのカメラを作る話ではない。Stage Viewは世界を覗くUI座標変換、`CompCamera`だけが作品の投影である。

M3へ最低2 taskを追加する。

- U1f: Stage View + Output Frame + off-frame Draft/bounds
- U2d: Camera/Output Frame direct manipulation + off-frame object selection

`U2c`は操作単純化conformanceへ割り当てるため、カメラtaskに流用しない。

### C. 操作単純化を「候補ID」から発注へ上げる

原則自体は【決定】済みであり、代表機能の採否とは別である。M3でUIを作る前に次を正式化する。

- G0-7: 代表操作コーパス、Domain Intent、永続物、Undo、失敗、semantic badge
- U2c: Direct/Tool/Advanced conformance harness。入口違いのserialize意味同値、hidden itemなし
- U4c: Advanced source/依存/scope/policy表示とround-trip。未実装pipelineをUIで捏造しない

Relative Moveのone-shot版はG0-7 fixtureだけでなくU2fへ正式発注する。常設Modifierへ拡張したことにはしない。

### D. 「3DはRGBA sourceにすぎない」の射程を狭める

`Layer Order`では各sourceがRGBAへ投影される現行契約を保てる。一方、`Group Depth`等の共有遮蔽では、RGBA化後にobject間depthを復元できない。したがって次を正本化する。

- 最終合流はpremultiplied RGBAでよい
- `Layer Order`は既存LayerSource→RGBAを使う
- shared depthへ参加するobject/material/generatorはHostの追加参加境界が必要
- 既存LayerSource traitへdepthを密輸しない
- 公開trait形状はP2D spike/解凍手続き前に発明しない

これは「pluginをやめる」意味ではない。pluginはHostが所有するworld/camera/depth参加境界へobject/material/generatorを供給する。

## P1接続

### Param Pipeline

M3仕様の停止条件へPP-Gateを追加する。U4aの現行DocParam編集は進めてよいが、常設offset、Modifier列、評価列並べ替え、汎用parameter pluginはPP-Gate完了前に実装しない。

### 一般音声

AG-3はM3-U6の置換ではなく追加レーンである。U6は現行MV最短導線、AG-3はAG-1/AG-2後にvideo+audio componentを同じimport/clip UIへ追加する。別project modeや別timelineを作らない。

## コード所見(2026-07-14時点)

以下は現実装の確認であり、即時修正の指示ではない。

- `motolii-doc`はコメント・schema・testで`CompCamera`を明示的に拒否
- `Transform2D`はXYのみで、2D=`z=0`は暗黙既定に留まる
- `RenderGraphInputs`にcameraがなく、LayerSource dispatchで`CompCamera::DEFAULT`を直書き
- `CompCamera`はPerspective固定、FOV/rollがdegreeで、GR-UIのDocument=radian規約と一致しない
- `motolii-ui`の製品実装はまだ無く、Stage/操作単純化を仕様で直す費用が最も安い

コード変更はD1jの意味/migration、D1kの凍結解凍、D3の接続と各拒否テストが順に揃ってから行う。既存受け入れテストを先に書き換えて通さない。

## 今回schema/APIへ焼かないもの

- Cloner/Effector、ExplicitSet/Backdrop Scope、汎用Element Domain、Constraint Graphのschema/API
- Param Pipelineの具体型
- shared depth plugin traitの具体形
- Stage全域のtight boundsアルゴリズムとVRAM予算の固定値（RoD/RoI最小契約はK0へ前倒し）
- camera near/farや複数camera

これらの具体形は意味/Gate待ちである。ただし一括して候補へ戻さず、[既知技術による処分決定](2026-07-14-motion-foundation-known-tech-disposition.md)の最小契約とspikeは先行する。
