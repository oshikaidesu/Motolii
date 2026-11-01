# 関係の持ち方の調べ（2026-11-01）

問い：1 つのターゲットが複数のグループに効く時、途中でターゲットや個数が変わる時、他のソフトはどう持っているか。
きっかけはユーザーの例：グリッドから外れた玉が、別のグリッドに激突する。

## 見本

| 見本 | 持ち方 | 確かめた所 |
|---|---|---|
| Apple Motion | 関係はビヘイビア。適用した物やグループの行の下に細い帯で付き、トリムで効く時間が変わる。別の物へ引いて移せる。Attractor の「適用」は全部・同じグループ・一覧の 3 択。元の側に付ける版（Attractor）と先の側に付ける版（Attracted To）が別々にある。 | 公式の説明書の本文を内蔵ブラウザで読んだ |
| Apple Motion（慣性） | Simulation の帯が終わると力は止まるが、物の動きは止まらない。 | 同上 |
| After Effects 23.0 | トラックマットが隣の行でなくてよくなり、どの行でも指せる。1 つのマットを複数の行で使える。プルダウンかピックウィップで指す。 | Adobe のヘルプと公式コミュニティの告知（検索結果の要約） |
| Bitwig | 元はいくつでも先を持ち、先ごとに量を持つ。1 つの値が複数の元を受ける。元の一覧にも先の一覧にも全部の経路が出る。元を選ぶと先が明るくなる。 | 公式の説明書 |
| Spine | Transform constraint は目標 1 つと、いくつでもの骨。mix はキーが打てる。掛かる順は木の順で、後の物は前の結果に掛かる。 | 公式の説明書 |
| Rive | IK と Rotation の Constraint に Strength があり、動かせる。0% で目標は効かない。 | 公式の説明書（検索結果の要約） |
| Unreal Sequencer | Constraint の有効をキーで持つ。多くの Constraint は有効になる時に見た目の位置を保つ補正キーを作り、タイミングを動かすとついてくる。後から足した物が同じチャンネルで勝つ。 | 公式の説明書（検索結果の要約） |
| Godot | Group は名札。1 つのノードが何個でも入れ、実行中に出入りできる。 | 公式の説明書 |
| Blender | 物は 1 つか複数の Collection に入れる。Field Weights の Effector Collection で、効く力の場を Collection に絞れる。 | 公式の説明書 |
| Cinemachine Target Group | 一覧の物ごとに重みと半径を持つ。重み 0 は無視される。重みを 0 へ動かすとカメラがずれるという報告が公式フォーラムにある。 | 公式の説明書、フォーラム（検索結果の要約） |
| Surge XT、Vital | 経路を 1 か所で見る一覧がある。公式の説明書は読めていない。 | レビューと紹介記事のみ、弱い |

## 分かった事

- 一番近いのは Apple Motion。関係が時間の帯で、先の選び方も持っている。
- 置き場所で関係を決める形は、After Effects が 23.0 で離れた。
- 1 つの物が複数のグループに入る問題は、Godot と Blender ではグループを箱にしない事で起きていない。
- 先ごとの量は Bitwig、切り替えの補正は Unreal、順は Spine が持っている。Motion の一覧の説明には先ごとの量が無い。
- 関係が切れた後は 2 通りある。力は動きが残る（Motion）。形は量 0 で元へ戻る（Spine の mix、Rive の Strength）。
- グループへの出入りを時間の帯で持つ見本は見つからなかった。Godot はプログラムで出入りするだけ。

## 読めていない物

- Cavalry は 10/19 の調べ（`behaviours_2026-10-19.md`）のまま。
- Cinema 4D の Effector を複数の Cloner で分け合えるかは未確認。
- Rive の Solo、Unreal の Attach の帯は確かめられなかった。

## 出典

- Apple Motion：[引き寄せ](https://support.apple.com/guide/motion/attractor-behavior-motn13747d1c/mac)、[シミュレーションの制御](https://support.apple.com/guide/motion/controlling-simulation-behaviors-motn13743676/mac)、[シミュレーションの概要](https://support.apple.com/guide/motion/intro-to-simulation-behaviors-motn13745a0c/mac)、[ビヘイビアの出る場所](https://support.apple.com/guide/motion/where-applied-behaviors-appear-motn13744263/mac)、[コピーと移動](https://support.apple.com/guide/motion/copy-paste-or-move-behaviors-motn137484b6/mac)
- [After Effects Track Mattes](https://helpx.adobe.com/after-effects/using/track-mattes-and-traveling-mattes.html)
- [Bitwig The Unified Modulation System](https://www.bitwig.com/userguide/latest/the_unified_modulation_system/)
- Spine：[Transform constraints](https://esotericsoftware.com/spine-transform-constraints)、[Constraints](https://esotericsoftware.com/spine-constraints)
- [Rive IK Constraint](https://rive.app/docs/editor/constraints/ik-constraint)
- [Unreal Animation Constraint Tools](https://dev.epicgames.com/documentation/en-us/unreal-engine/animation-constraint-tools-in-unreal-engine)
- [Godot Groups](https://docs.godotengine.org/en/stable/tutorials/scripting/groups.html)
- [Blender Collections](https://docs.blender.org/manual/en/latest/scene_layout/collections/collections.html)
- [Cinemachine Target Group](https://docs.unity3d.com/Packages/com.unity.cinemachine@3.1/manual/CinemachineTargetGroup.html)
