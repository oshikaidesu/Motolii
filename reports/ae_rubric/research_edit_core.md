# 編集の「中身」を丸ごと借りられるか（調査 2026-10-14）

## まとめ（悪い知らせから）

1. 「時間に並ぶ物 + 重なり + 値のキー + マット + 保存 + 元に戻す」を全部そろえて、自分では絵を描かず、C/C++ から呼べて、GPL でない物は **見つからなかった**。どの候補も二つか三つ欠ける。
2. とくに **「元に戻す」を持つ編集ライブラリは無い**。GES も MLT も OpenTimelineIO も OpenUSD も libopenshot も持たない。どの編集ソフトも上の殻で自分で書いている（Pitivi は Python で約千行の見張り役、OpenShot は JSON の差分をためる）。
3. **lumit は見つかった**。Rust の中身 + Flutter の皮の、モーショングラフィックス編集ソフト（2026 年 8 月初版、今 v0.7.0）。ただし **GPLv3** なので、コードは閉じた製品に借りられない。借りられるのは「やり方」だけ。
4. 良い知らせ：GES は **もうこの Mac に入っている**（GStreamer 1.28.8 と一緒に libges、ges-launch-1.0 あり）。LGPL。物の並び・重なり・切り出し・キー・保存読み込みを C から使える。欠けるのは「元に戻す」、マット、奥行き。
5. 良い知らせ：「保存 + 元に戻す」だけなら出来合いが二つある。**SQLite の session 拡張**（この Mac の sqlite3 で有効と確認、パブリックドメイン）と **Yrs（yffi、MIT、C のヘッダ付き）**。どちらも中に入れる形（スキーマ）は決めてくれない。
6. お勧めは三つ：(A) GES を文書として使う、(B) SQLite を文書の入れ物にして「元に戻す」を借りる、(C) lumit のやり方を名前ごとまねる。A と B は足し合わせられる。

読んだ物と推測は下の表と覚え書きで分けた。「記憶」と書いた所は今回ページを読んでいない。

## 言葉の一覧（広く探す前に集めた呼び名）

英語
- headless video editor / headless editing engine / engine + thin shell
- timeline engine / timeline model / editing core / editing services
- non-linear editing (NLE) library / video editing library / compositing engine
- project document model / scene file / scene description / draft JSON
- edit decision list (EDL) / interchange format（OTIO, AAF, FCPXML）
- edit algebra / edit algorithms（overwrite, insert, trim, slice, slip, slide, ripple, roll, fill）
- command pattern / command with inverse / op / journal / op log / undo stack / undoable action log
- snapshot undo / JSON-Patch diff undo / persistent data structure / structural sharing
- CRDT / local-first / UndoManager / changeset / session
- control source / control binding / time samples / property animation / animation curve（どれも「値のキー」の別名）
- track matte / mask / layer / track / clip / in-point / source range / time warp / retime
- bridge contract / FFI seam / handle / scoped change stream / shared texture（zero-copy）

日本語
- 動画編集エンジン / 編集コア / タイムラインエンジン / ヘッドレス編集
- プロジェクトファイル形式 / ドキュメントモデル / シーン記述
- コマンドパターン / アンドゥ・リドゥ / 操作ログ / 差分 / スナップショット
- キーフレーム / トラックマット / レイヤー / クリップ / イン点・アウト点

ほか
- 中国語：剪映 草稿（draft_content.json）、时间线引擎、无头剪辑
- ドイツ語：Schnitt-Engine、Zeitleisten-Modell（当たりは無かった）

この言葉で探して新しく出てきた物：**libopenshot**、**avio**（Rust）、**vean**（MLT の上の編集代数）、**videola**、IMG.LY CE.SDK（売り物）。

## 候補の表

記号：○ 出来合いである / △ 一部・回り道 / × 無い。「描かずに使える」は、絵を出さず文書としてだけ使えるか。

| 候補 | 並び(開始/切り出し/速さ) | 重なり | 値のキー | マット | 保存読込 | 元に戻す | C/C++ Mac+Win | 描かずに使える | ライセンス | 生きているか |
|---|---|---|---|---|---|---|---|---|---|---|
| **GES**（GStreamer Editing Services） | ○（速さは時間効果で△） | ○ layer | ○ GstController（子の属性に） | × | ○ xges | ×（Pitivi が Python で自作） | ○ C | △ 再生しなければ描かない。ただし物を作ると GStreamer の部品も作る（推測） | LGPL | ○ 1.28.8 がこの Mac にある |
| **MLT** | ○ | ○ track | ○ 属性のアニメ（mlt_animation） | △ マスク系フィルタ（記憶） | ○ XML | ×（Kdenlive/Shotcut が自作） | ○ C + mlt++ | △ 部品を作ると素材を開く（記憶） | LGPL 2.1（記憶） | ○（未インストール） |
| **OpenTimelineIO** | ○ source_range、時間伸縮は効果として | ○ stack/track（中は順に並ぶ） | × | × | ○ JSON | × | ○ C++ | ○ もともと描かない | Apache 2.0（記憶） | ○ ASWF、2025-11 の報告あり |
| **libopenshot** | ○ Position/Start/End/Layer、速さもキー | ○ | ○ Keyframe（直線・定数・ベジェ） | △ Mask 効果（記憶） | ○ JSON、ApplyJsonDiff | ×（OpenShot の殻が差分をためる） | ○ C++ | △ 自分で絵を合成する作り。文書だけの使い方は未確認 | LGPL 3（+売り物） | ○ 0.7.0。FFmpeg・JUCE に依存 |
| **OpenUSD** | × 並びの意味は無い | △ layer は「上書きの重ね」で別物 | ○ time samples | × | ○ usda/usdc | ×（SdfLayerStateDelegate で各社が自作） | ○ C++ | ○ | TOST（Apache 2.0 を少し変えた物） | ○ 重い |
| **glTF + KHR_animation_pointer** | × | × | ○（読む側次第） | × | ○ | × | 読むだけ | ○ | 仕様は自由 | Filament の gltfio の対応一覧に名前が無い（一覧の写しを見ただけ） |
| **Lottie JSON を文書に** | ○ ip/op/st/sr | ○ | ○ | ○ tt/td | ○ | × | × 編集する C/C++ ライブラリが無い | ○ | python-lottie は AGPL、Glaxnimate は GPLv3（記憶） | ThorVG の slot は「値の差し替え」だけ |
| **Yrs (yffi)** | × 入れ物だけ | × | × | × | ○ 更新の塊を書く/当てる | ○ YUndoManager | ○ C ヘッダ libyrs.h | ○ | MIT | ○（Rust のビルドが要る） |
| **Loro** | × 入れ物だけ | × | × | × | ○ | ○ UndoManager、動かせる木 | × C++ 用が見当たらない（Swift/Python など） | ○ | MIT | ○ 1.13.9 |
| **Automerge** | × | × | × | × | ○ | × 本体に無い（JS の試作だけ） | ○ automerge-c | ○ | MIT（記憶） | ○ |
| **SQLite + session 拡張** | × 入れ物だけ（表） | × | × | × | ○ ファイルそのもの | ○ 変更の束を逆にして当てる | ○ C | ○ | パブリックドメイン | ○ この Mac で有効（3.43.2 と 3.53.4 で確認） |
| **Qt Undo framework** | × | × | × | × | × | ○ QUndoStack | ○（Qt が要る） | ○ | LGPL | ○ 「元に戻す」だけのために Qt は重い |
| **immer + lager**（C++） | × | × | × | × | × 自分で | △ 古い値を取っておくだけで済む | ○ ヘッダだけ | ○ | BSL-1.0 / MIT（記憶） | ○ |
| **lumit** | ○ | ○ | ○ | ○ | ○ .lum | ○ 逆つきコマンド + 日誌 | × Rust、借用不可 | — | **GPLv3（問題）** | ○ 2026-10-12 v0.7.0 |
| **IMG.LY CE.SDK** | ○ | ○ | △ | ? | ○ scene | ○ 履歴 API | 売り物。Flutter 版あり | ヘッドレスは Web/Node の頁だけ見た | 有料・プラットフォームごと | ○ |
| Olive / Natron / Blender / Friction / Kdenlive / Shotcut | — | — | — | — | — | — | — | — | **GPL（問題）**（記憶） | 借りられない |
| Rive / Cavalry / CapCut 草稿 JSON | — | — | — | — | — | — | — | — | 編集側は非公開（記憶） | 借りられない |
| Theatre.js / Motion Canvas / Remotion | キーの芯はある | — | ○ | — | ○ JSON | △ | × JavaScript | — | Theatre の studio は AGPL（記憶） | C++ の中身には入らない |

## 覚え書き

### 1. lumit（読んだ）

- 何か：「a native motion-graphics and compositing editor」。After Effects の深さ + Vegas の時間いじりを一つに、という売り文句。Windows が主、Mac と Linux も。星 45、コミット 2,761。2026-08-02 に 0.1.0、2026-10-12 に 0.7.0。
- 作り：`crates/`（Rust の中身、32 個）と `flutter_ui/`（皮）。二つは **`crates/lumit-bridge` の一か所だけ**で会う。
- 橋のやり方（docs/17-BRIDGE-CONTRACT.md を読んだ）：
  - 道具は **flutter_rust_bridge**（手書きの dart:ffi ではない）。Dart 側のコードは生成物。両側に中身のハッシュを入れ、合わないライブラリは起動を止める。
  - Dart は **文書を持たない**。持つのは取っ手（ProjectReference / CompositionReference / LayerReference / ItemReference）だけ。
  - **利用者の一動作 = 呼び出し一回 = lumit_core の op 一つ = 元に戻す一歩**。op は差ではなく「値まるごと」を書く。ドラッグ中は仮置きで、放した時だけ確定。
  - 変わった所だけ知らせる流れ（ScopedChange）。画面はそこだけ描き直す。
  - 絵は **コピーなしの共有テクスチャ**：Mac は IOSurface、Windows は D3D12、Linux は DMA-BUF。小さい静止画（サムネイル等）だけ画素で渡す。
  - 時間は分数かコマ番号。小数の秒は渡さない。
- 中身のやり方（docs/05-ARCHITECTURE.md を読んだ）：
  - 「Every edit is a command (AddLayer, SetKeyframe, ...) with an inverse.」元に戻すは日誌をたどる。
  - 一回の編集ごとに新しい文書の写しを作り、ポインタ一つの差し替えで公開（arc-swap）。「Today that clones the whole Document.」つまり単純に丸ごと複製。
  - 自動保存は同じ日誌をディスクに足す。復旧は「最後の写し + 日誌の再生」。
  - すべての物に変わらない UUID。番号や位置で指さない。
  - 描くのは wgpu 一つ。動画は FFmpeg。
- 私たちへの意味：オーナーの言う「Flutter は API を呼ぶ皮」はこの橋の約束そのもの。**コードは GPLv3 なので借りられないが、形と名前はまねられる**。私たちの共有テクスチャ（IOSurface を Filament の Dawn へ）は lumit と同じ道。

読んだ頁：
- https://github.com/luminalmvm/Lumit
- https://lumitlab.com/releases/
- https://github.com/luminalmvm/Lumit/releases
- https://raw.githubusercontent.com/luminalmvm/Lumit/main/docs/17-BRIDGE-CONTRACT.md
- https://raw.githubusercontent.com/luminalmvm/Lumit/main/docs/05-ARCHITECTURE.md
- https://github.com/luminalmvm/Lumit/tree/main/crates

### 2. GES（読んだ + この Mac で有無を確認）

- 読んだ：timeline が中心、layer が重なり、clip が一つの layer に乗る物、GESProject と GESFormatter が保存読み込み。LGPL。timeline は GstElement。
- 読んだ：`ges_track_element_set_control_source` で子の属性に GstControlBinding を付ける（"direct" / "direct-absolute"）。キーの時刻は素材の内側の時間で書く。切り出しの外は既定で切り詰め。
- 読んだ（Pitivi の pitivi/undo/timeline.py）：**元に戻すは GES に無い**。Pitivi が GES の合図を見張り（TimelineObserver, LayerObserver, ControlSourceObserver…）、逆の操作を記録する。このファイルだけで約千行（見積もり）。
- この Mac：`/opt/homebrew/lib/libges-1.0.dylib`、`ges/ges.h`、`ges-launch-1.0`、プラグイン libgstges / libgstnle がある。
- 推測（未確認）：物の位置は posx / posy / width / height / alpha で、**奥行き z は無い**。マットの関係も無い。自前の値は GESMetaContainer（何にでも付けられる名前つきの値、xges に保存される）に置けるはず。素材の clip を作ると GStreamer が素材を調べに行くので、少し重いはず。
- 読んだ頁：
  - https://gstreamer.freedesktop.org/documentation/gst-editing-services/index.html
  - https://gstreamer.freedesktop.org/documentation/gst-editing-services/gestrackelement.html （前半 10 万字まで）
  - https://gitlab.gnome.org/GNOME/pitivi/-/raw/master/pitivi/undo/timeline.py

### 3. MLT（検索の要約まで。本体の頁は読めていない）

- 検索で見た：XML を自分で読み書きする。Kdenlive と Shotcut は自分用の属性を XML に足している。属性のアニメ API（mlt_animation）がある。
- 読んだ（KDE の Refactoring の頁、古い）：Kdenlive の並びは MLT とは別に持っていて、それが「timeline corruption」の元だった、とある。**二重に持つと壊れる**という教訓。
- 元に戻すがどこにあるかは頁で確かめられなかった（記憶では Kdenlive 側の QUndoStack）。
- この Mac には入っていない。
- 読んだ頁：https://community.kde.org/Kdenlive/Refactoring

### 4. OpenTimelineIO（読んだ）

- 持つ物：timeline → stack → track → clip / gap / transition。clip は外の素材を指し、source_range で切り出す。効果は名前だけ：「OpenTimelineIO does not say anything about the impact of the effect.」
- 編集の算法（editAlgorithm.h を読んだ）：overwrite, insert, trim, slice, slip, slide, ripple, roll, fill, remove の十個。
- 持たない物：キー、元に戻す、値の評価。2025-11 の ASWF の報告でも色と位置の効果は「これから」（検索の要約）。
- track の中は「順に並ぶ」形なので、AE 風の「物ごとに開始時刻」は gap + clip で表すことになる。
- 読んだ頁：
  - https://opentimelineio.readthedocs.io/en/latest/tutorials/otio-timeline-structure.html
  - https://raw.githubusercontent.com/AcademySoftwareFoundation/OpenTimelineIO/main/src/opentimelineio/algo/editAlgorithm.h

### 5. libopenshot（読んだ。言葉集めで新しく出た）

- C++。timeline に clip を重ねる。キーは「linear, constant, or quadratic bezier」。速さと逆再生もキー。クロマキー効果あり。Python / Ruby の口あり。LGPL 3 と売り物の二本立て。FFmpeg が土台、音は JUCE。
- `Timeline::ApplyJsonDiff`：「a change to the timeline (add, update, delete)」を JSON で当てる。`Json()` / `SetJson()` で丸ごと保存読み込み。v0.7.0 の頁。
- 2014 年の OpenShot の記事（検索の要約）：元に戻すは「この JSON の変更を逆順・正順に当て直すだけ」。つまり **元に戻すは殻の側**。
- 懸念：自分で絵を合成する作り。文書だけとして使えるかは未確認。依存が重い。z は無い（記憶）。
- 読んだ頁：
  - https://www.openshot.org/libopenshot/
  - https://cdn.openshot.org/static/files/libopenshot/classopenshot_1_1Timeline.html

### 6. OpenUSD と glTF（検索の要約まで）

- OpenUSD：本体に元に戻すは無い。Luma Pictures は SdfLayerStateDelegate で逆の操作を積む。NVIDIA は omni.kit.usd_undo を別に持つ。SdfChangeBlock はまとめ役で、元に戻すではない。ライセンス名は 24.08 から TOST（中身は変えていないと発表）。
- glTF：Filament の gltfio の対応一覧（README の写し）に KHR_animation_pointer の名は無かった。ソースでは確かめていない。編集用のライブラリも無い。**文書には向かない**。
- 見た頁（検索結果の要約のみ）：https://forum.aousd.org/t/tracking-the-changes-of-usdnamespaceeditor/2204 、https://forum.aousd.org/t/upcoming-openusd-license-update/1561

### 7. 入れ物（保存 + 元に戻す）

- **Yrs**：libyrs.h を読んだ。YMap / YArray / YText、状態の書き出し（ytransaction_state_diff_v1 ほか）と当て込み（ytransaction_apply）、YUndoManager の型がある。MIT。関数名 yundo_manager_undo / redo / stop は Collabora の覚え書き（検索の要約）で見た。ヘッダの終わり 1.4 万字は読めていない。
- **Loro**：UndoManager あり（最大歩数、まとめる間隔、一人分だけ追う）。loro-ffi は UniFFI 製で Swift / Python / React Native / C#（有志）/ Go（有志）。**C++ 用は見当たらない**。公式の頁は 403 で読めず、検索の要約のみ。
- **Automerge**：automerge-c はある。元に戻すは JS の試作ラッパーしか見つからない（検索の要約）。
- **SQLite**：この Mac で `ENABLE_SESSION` と `ENABLE_PREUPDATE_HOOK` が有効（/usr/bin/sqlite3 3.43.2、Homebrew 3.53.4 とも 1 を返した）。変更の束を取り、逆にして当てる関数がある（記憶：sqlite3session_changeset、sqlite3changeset_invert、sqlite3changeset_apply）。頁は今回読んでいない。
- 読んだ頁：https://raw.githubusercontent.com/y-crdt/y-crdt/main/tests-ffi/include/libyrs.h （前半 10 万字）

### 8. Flutter の皮は中身とどう話すか

実例では三つの部品に分かれる。(1) **命令は FFI**：Rust なら flutter_rust_bridge が Dart 側を生成する（lumit）。C/C++ なら dart:ffi で C の関数を直接呼ぶ（Rive の rive_native は C++ の実行部を FFI で動かす。thermion は Filament を包んで Flutter に出す）。(2) **絵は共有テクスチャ**：中身が GPU の面に描き、Flutter の Texture がそれを映す。画素は CPU を通らない（lumit：IOSurface / D3D12 / DMA-BUF。thermion：ThermionWidget がプラットフォームごとのテクスチャを作る）。(3) **変化の知らせは流れ（stream）**：中身から「どこが変わったか」だけを送り、皮はそこだけ描き直す。プラットフォームチャンネルは細かい呼び出しには遅く、今の例はどれも FFI。私たちは C++ なので、**C の関数の薄い一枚 + dart:ffi（ffigen でヘッダから生成）** が一番まっすぐ。

読んだ・見た頁：lumit の橋の約束（上）、https://rive.app/docs/runtimes/flutter/rive-native （検索の要約）、https://pub.dev/documentation/thermion_dart/latest/ （検索の要約）

### 9. お勧めの三つの道

**道 A：GES を文書にする（もう入っている、LGPL、C）**
- 借りる物：並び・重なり・開始/切り出し/長さ・値のキー（GstController）・保存読み込み（xges）。動画読みも同じ GStreamer なので世界が一つで済む。
- 一時間の確かめ（新しく入れる物なし）：C で 60 行ほど。ges_init → timeline → layer → 動画の clip を一つ置く → start / in-point / duration を変える → alpha に 2 点のキーを付ける → clip に「matte=相手の id」「z=…」をメタ情報として付ける → xges に保存 → 読み直して同じか見る。**再生は一度もしない**。作るのに何ミリ秒かかるか、Filament の側から毎フレーム値を読めるか（gst_control_source_get_value）も見る。
- まだ無い物：元に戻す、マットの関係、奥行き、キーを付けられるのは GStreamer の部品の属性だけという縛り（私たちの「場所」が部品の属性でない場合どうなるかが一番の賭け）。

**道 B：SQLite を文書の入れ物にし、「元に戻す」を借りる（もう入っている、パブリックドメイン、C）**
- 借りる物：保存読み込み（ファイルそのもの、いつ落ちても壊れない）、元に戻す/やり直す（session が変更の束を取り、逆にして当てる）、Mac と Windows。
- 一時間の確かめ：表を二つ（物、キー）。物を足す・動かす・キーを打つを三回 → 各回の変更の束を取る → 逆にして当てて戻る → もう一度当ててやり直す。束の大きさと時間を測る。
- まだ無い物：形（表の列）は自分で決めることになる。ここは「スキーマを発明しない」に反するので、**列の名前は GES か Lottie の名前をそのまま借りる**（start / inpoint / duration、ip / op / st / sr、tt / td）。A と組むなら、GES が持てない物（マット、z）だけを SQLite に置くより、**GES の xges を捨てて SQLite 一本**のほうが二重持ちにならない（Kdenlive の「二重に持って壊れた」教訓）。
- 代わりの入れ物：Yrs（MIT、C ヘッダと UndoManager つき、後で共同編集にも伸びる）。ただし Rust のビルドを足すことになる。

**道 C：lumit のやり方を名前ごとまねる（コードは借りない）**
- 借りる物：橋の約束そのもの。取っ手だけ渡す / 一動作 = 一呼び出し = 一 op = 元に戻す一歩 / op は値まるごと / ドラッグは仮置き、放して確定 / 変わった所だけ知らせる流れ / 絵は共有テクスチャ / 時間は分数。
- 一時間の確かめ：lumit の `crates/lumit-bridge/src/api/` の関数名の一覧を読み、私たちの七つの動作（足す・消す・動かす・時間を変える・キーを打つ・マットを付ける・保存/読込）に当たる名前を表にする。書くのは表だけ。
- まだ無い物：これは「形」であって中身ではない。中身は A か B が要る。GPLv3 なので写経は不可、名前と約束だけ。

**組み合わせの勧め**：皮との約束は C（lumit の形）、文書と元に戻すは B（SQLite）、動画の時間の言葉は A（GES の名前）から。A を丸ごと採るかは、上の一時間の確かめで「私たちの場所や z にキーを打てるか」が通るかどうかで決まる。

## 今回できなかった事

- Loro の公式頁（403）、Pitivi の開発者向け頁（404）は読めなかった。
- MLT、Automerge、SQLite session、Theatre.js、Glaxnimate、python-lottie、CapCut 草稿、Natron、Olive、Friction、Rive は頁を読まず、記憶か検索の要約。表に「記憶」と書いた。
- avio（Rust、Editor に元に戻すあり）と vean（MLT の上で op → {新しい状態, 結果, 逆}）は検索の要約で見ただけ。どちらも C++ ではないが、形の参考になる。
- 何も入れていない、何も落としていない。調べ物の間にこの Mac で走らせたのは、ファイルの有無を見る命令と sqlite3 の設定を聞く一行だけ。

## この Mac で確かめた事（2026-10-14 未明、入ってる物だけ。`filament/probe_edit/`）

**SQLite の session（`undo_sqlite.c`）**：通った。表2つ（物、キー）に編集3回 → 各回の変更の束を逆にして当てて3回戻る → もう一度当てて3回やり直す、全部元どおり。
束の大きさ 74〜92 バイト。ディスク上のファイルで「編集＋取り消し」1組 1.87 ms（千回の平均、物が千個ある文書）。閉じて開き直しても残る。
注意：macOS に元から入ってる sqlite3 のヘッダには session の宣言が無い。Homebrew の物（もう入ってた）のヘッダと本体を使った。配る時は SQLite の1ファイル版を一緒にビルドするのが普通。

**GStreamer の controller を単体で（`doc_ges.c` の前半）**：通った。部品も配管も無しで、キーを打って（時刻、値）、好きな時刻の値を読める。
1回の読み 0.14 マイクロ秒。つなぎ方は4種（段、直線、なめらか、行き過ぎないなめらか）。キーごとの速度曲線は無い＝区間イージングを持たない形。

**GES を文書として、再生せずに（`doc_ges.c` の後半）**：半分。
- 動いた：動画を1個置く、開始・切り出し・長さを変える、うちの値（z、マットの相手）を札として付ける、透明度にキーを打つ、xges に保存（0.4 ms）、読み直す（12 ms）、全部同じ値で戻る。
- 合わない：置き場所として持ってるのは posx、posy、width、height、zorder、alpha だけ。**奥行きが無い**（推測だったのを確認）。札は付けられるがキーは打てない。
- 重さ：ges_init 90 ms、動画1個を置くのに 344 ms（ファイルを調べに行く）。
- 起動時に「NSApplication が要る」という警告が出る（GL の部品を触るため）。

**glTF**：確かめる前に外した。キーが2進の塊（バッファ）に入る形で、1個打ち直すたびに塊と数え方を作り直す事になる。配る形であって編集する形ではない。
gltfio と cgltf は KHR_animation_pointer を読まない（ソースを検索して確認）ので、キーを打てるのは場所・回転・大きさだけ。3Dファイルを読む口としては別に使える（うちの Dawn ビルドに libgltfio が入ってる）。

**並べると**：キー＝GStreamer の controller、保存と取り消し＝SQLite、ガワとの約束＝lumit の作法、時間の言葉＝GES の名前（start、in-point、duration）。
借りられないで残るのは2つ：(1) 文書の中の形（何という名前の値を持つか）、(2) 文書を読んで舞台に伝えるつなぎ。
