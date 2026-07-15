# Relative Move / Timeline Effect Link / Duplicator決定(2026-07-15)

ステータス: **【決定】**。2026-07-14の[既知技術による処分決定](2026-07-14-motion-foundation-known-tech-disposition.md)を、ユーザー確認とCavalry公式資料の追加調査で具体化する。本書がRelative MoveのUI、Explicit Effectの意味、Duplicator/seed規約の新しい正本である。

## 1. Relative Moveはmodifier+dragだけ

Relative Moveは常設offset、Animation Layer、専用Tool、専用panelではない。Canvas上のobjectを**keymapで割り当てたprimary modifierを押しながらdrag**し、PositionのConstまたは全keyへEdit-Spaceの同じ差分を適用するone-shot D2 macroである。

- 物理キーをDocumentへ保存しない。`motolii.relative_move_drag`相当の安定`CommandId`/gesture intentへkeymapを割り当て、OS既定とuser deltaで解決する
- 通常dragは現在値/Auto Key、modifier+dragは軌跡全体。開始位置とHUDの`Relative ΔX/ΔY/ΔZ`で区別する
- pointer-downで対象と元値をsnapshot、drag中はtransient preview、pointer-upでUndo 1回、Escape/capture lossで変更ゼロ
- motion path全体をghost表示する。Toolbar、専用Tool、常設Advanced editorは作らない
- `DataTrack / Follow / 手続き値`は暗黙にBakeせず、初版では型付きunsupported
- modifierの具体キーはG0 keymap競合表で決め、ユーザーが変更可能にする

これは「機能を増やす」より、繰り返し使う編集スクリプトを短いgestureへしたものとして扱う。

## 2. Stage透過とRoD/RoI最適化は分ける

ユーザー向けの第一目的は、Output Frame外を不透明グレーで隠さず、同じworldのobjectを透けて見せることにある。

- U1fはK0を待たず、保守的なStage Draftを描き、Output Frame外へ半透明scrimを重ねる。枠外を透明度0にせず、出力外である境界線・scrim・frame形状を残す
- 枠外objectの選択、anchor、motion path、hit-testを維持する
- RoD/RoIは後段の性能契約であり、U1fの見た目の成立条件ではない
- K0導入後は、見た目を変えずに必要領域だけ評価する。`Unknown`は全域/安全上限へfallbackし、最適化不足を画素欠落へ変えない

つまり「透けるStage」が意味、「RoD/RoI」が同じ意味を重くせず実行する方法である。

## 3. Explicit Effectは共有recipeを各layerのstack slotへ接続する

隣接しない複数layerへ同じEffect設定を適用するため、Explicitを「選択layerを一枚に合成してEffectへ入れる」と定義しない。**共有Effect Definitionを、各layerの順序付きEffect Stack内のUse slotへ参照接続し、各layerをその位置で個別処理する**。

```text
Effect Definition: Glow A
        out
       / | \
      v  v  v
Layer A.effects.in[1]
Layer D.effects.in[0]
Layer H.effects.in[2]
```

- Group/Owned: 子を先に一枚へ合成し、Effectを一度適用する。cross-layer blur等はこちら
- Explicit Shared Use: 同じdefinition/paramsを複数layerへ共有し、各layerのstack位置で個別適用する。対象は隣接不要、所有関係も変えない
- 同じlayerへ同じdefinitionを複数回使えるよう、definition identityとuse identityを分ける
- definition変更は全useへ反映する。useの並べ替えは対象layerのstackだけを変える
- target layerの表示名・timeline順変更では接続不変。削除、欠落、循環不能条件を型付き検査する
- Explicitはsource layerを消費、複製、再合成しないため、出力挿入位置の曖昧さと二重描画を作らない
- 複数layerを一枚に合成してから処理したい場合はGroupを使う。Explicit Composite Setはv1で作らない

永続形の方向は次とする。既存inline effectからのmigrationと未知plugin保持をD1lで固定してから実装する。

```text
EffectDefinitionId
EffectUseId

EffectDefinition {
  id,
  plugin_id,
  effect_version,
  enabled,
  params,
  extra,
}

EffectUse {
  use_id,
  definition_id,
}

ItemEnvelope.effect_stack: [EffectUse]
Document.effect_definitions: { EffectDefinitionId -> EffectDefinition }
```

既存inline `EffectInstance`はmigration時にdefinition 1件+use 1件へ一対一変換し、画素、順序、未知fieldを変えない。共有は複数useが同じdefinitionを参照した時だけ発生する。

### Timelineの常時表示線

- Timeline/Scene Treeの左側に固定幅のconnection gutterを置き、Effect Definitionの`out`から各Useの`in`へ有向線を**常時表示**する
- fromは右向きsocket、inは左向きsocketとarrowheadで識別し、色だけに依存しない
- 線はclip本体やkeyframe領域を横断させずgutter内でroutingする
- 折り畳みgroup内の接続は親rowのstub+件数badgeへ束ねるが、接続が存在する事実は常時消さない。展開で個別線へ戻る
- hover/selectionは強調だけに使い、非選択時も線を消さない
- drag中は型不一致targetをdimし、drop前に挿入stack位置を表示する
- 1 drag=1 Use追加=1 Undo、Escape=0変更。線の付け替えでdefinitionやlayerを複製しない

CavalryはLayer UIの裏をnode接続にし、型互換のあるattributeだけを接続し、入力/出力を別iconで示す。本案はその型付き接続を採りつつ、接続iconだけに隠さずtimeline gutterへ常時投影する。

先例: [Cavalry Connections](https://cavalry.studio/docs/getting-started/key-concepts/connections/)、[Cavalry Scene Tree](https://cavalry.studio/docs/user-interface/menus/window-menu/scene-window/scene-tree/)

## 4. Backdropはplugin実装、入力境界はHost

Backdropの画像処理自体はFilter/Composite pluginで作れる。ただしpluginがtimelineを走査して「下のlayer」を探してはならない。

- Hostが評価地点の合成済みtextureを型付き`BackdropInput`として渡す
- pluginは受け取ったtextureとparamsを純関数処理する
- 評価地点、循環拒否、cache key、preview/export一致はHost責務
- v1の公開plugin traitへBackdrop口を追加するのは、入力意味と凍結解凍が完了した後

## 5. Duplicator/EffectorはCavalryモデルをHost向けに縮小採用

Cavalry Duplicatorの次の構造を採る。

- `Input Shapes`: 複製元の型付き参照列
- `Distribution`: instanceの基礎配置
- per-instance `Position / Rotation / Scale / Visibility / Opacity / Prototype Id / Time Offset`
- `Context`: index、count、position、nested context depth
- `Behaviour`: Contextを読み、per-instance channelへ値を返す
- `Stagger / Random / Falloff`を最初のfirst-party Behaviourとする
- 1,000 instanceでもTimeline rowを1,000本生成せず、HostがGPU instance列を所有する

先例: [Cavalry Duplicator](https://cavalry.studio/docs/nodes/shapes/duplicator/)、[Cavalry Context](https://cavalry.studio/docs/getting-started/key-concepts/context/)、[Cavalry Stagger](https://cavalry.studio/docs/nodes/behaviours/stagger/)、[Cavalry Falloff](https://docs.cavalry.scenegroup.co/nodes/utilities/falloff/)

### Stable IDとseedはCavalryより強く固定する

> **2026-07-15先例監査補正**: `InstanceId != index`、明示seed、決定論PRNGは固定する。一方、以下のDistribution別slot keyは実装確定ではなくP0Iの第一仮説である。USDはproducer-authored ID、Blenderはgenerator固有stable IDであり、全Distribution共通の継承規則は確認できない。[先例収束 / 日曜大工境界監査](2026-07-15-prior-art-complaint-boundary-audit.md)の反例試験を通るまでschemaへ焼かない。

CavalryのContext Indexは表現上有用だが、Motoliiでは乱数identityを配列indexだけへ結び付けない。

```text
InstanceId = stable_hash(duplicator_id, distribution_slot_key)
RandomKey  = stable_hash(user_seed, instance_id, channel_tag)
value      = pcg32(RandomKey)
```

- 乱数は必ず明示`user_seed: u64`を持つ。同じDocument、時刻、入力、seedから同じ値を返す
- render/evaluation中にOS entropy、時刻、thread順、GPU実行順を使わない。「真のランダム」は作らない
- Randomize/RegenerateはseedをDocumentへ書く明示D2 command。再評価のたびにseedを変えない
- Linear/Radial/Pathはslot ordinal、Gridは`(x,y[,z])`等を第一仮説とし、distributionごとにidentity domainと編集時の継承規則を宣言する。flatten indexをidentityにしない
- count増減で残るslotのInstanceIdと乱数を維持する。distribution種別変更はidentity domain変更として明示的に再生成される
- nested Duplicatorは親子InstanceIdをhash合成し、context depthを保持する
- `index/count`はStaggerや順序表現に使えるが、永続override、乱数、motion sampleのidentityには使わない
- アルゴリズムは既存PathOpと同じPCG32系へ揃え、実装名/versionを意味論goldenで固定する

先例: [Cavalry Random](https://cavalry.studio/docs/nodes/behaviours/random/)、[Cavalry Index Context](https://cavalry.studio/docs/nodes/utilities/index-context/)、stable identityの比較先として[OpenUSD PointInstancer](https://openusd.org/24.08/api/class_usd_geom_point_instancer.html)

## 6. 実装順

1. M3-U2f: modifier+drag one-shot Relative Move
2. M3-U1f: K0を待たず透けるStage/off-frame selection
3. M2-D1l: Effect Definition/Use schema、inline migration、validation
4. M2-D3e: shared Effect Use評価、各stack位置、cache dependency入力
5. M3-U2g: timeline connection gutter、常時線、from/in、drag接続
6. M4-K2: definition変更から全Useへの無効化伝播
7. M5-P0I: Cavalry型Context/Behaviourとstable identityの契約fixture
8. M5-P7a/P7b: Duplicator基盤とStagger/Random/Falloff

依存未完了の現時点では、UIやDocument fieldをコードへ先行実装しない。
