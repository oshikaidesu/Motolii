# 普通のタイムラインの先例調査 — 操作カタログとMotolii既決の突き合わせ

日付: 2026-08-19
状態: **観察**(先例調査であり決定を含まない。規律6点により、この文書の結論をそのまま設計根拠にしない)

## 0. 目的と方法

利用者の懸念: 「タイムラインは動画編集においてずっと触るpanel」であり、egui版TimelineをicedへportするにあたってAE参照だけでは掬えていない「制作ソフトのタイムラインが普通に持っている操作」の漏れを潰したい。

**この文書は調査であり、trim familyやマーカー等の追加を推奨・決定するものではない。** 8製品の公式文書・公式マニュアルを横断し、「操作が存在するか」を件数で確認しただけである。反例未探索の一次調査であり、「証明された」「裏付けられた」とは書かない([レビュー規律6点](README.md))。

調査対象は利用者指示の5カテゴリに沿う7製品+1(NeoUtlは未リリースのためAviUtlの目標として参照のみ):

| カテゴリ | 製品 | 補足 |
|---|---|---|
| レイヤーベース | After Effects (AE) | 主参照。concept.mdの既存前提 |
| トラックベース | Premiere Pro、DaVinci Resolve | ripple/roll/slip/slide等の出典 |
| モバイル系 | CapCut(デスクトップ版含む) | モバイル固有の画面遷移前提は採らない(利用者指示) |
| 日本の同人動画文化圏 | AviUtl、NeoUtl | NeoUtlはコード不流用・構造/機能順序の参照のみ(既決)。AviUtlは公式マニュアルが薄く、コミュニティwikiが事実上の一次資料 |
| DAW | Ableton Live、Reaper | Reaperも公式文書が薄く、コミュニティ資料を併用 |

Alight Motionは調査対象にしない(既決、[alight-motion-as-ux-north-star]相当のメモリ決定)。

判定基準(必須/標準/便利)は次の2軸の合議:

- **件数**: 確認できた範囲で7製品中何本に存在するか(6本以上=必須寄り、4〜5本=標準寄り、1〜3本=便利寄り)
- **無いと困る度合い**: その操作が無いと基本編集(選択・配置・trim・再生確認)そのものが成立しないか、回避策があるか、無くても製品は成立するか

この判定は先例調査からの仮説であり、Motoliiでの必須性を保証しない。

**先行する内部調査**: [`docs/m3-rn-runtime-execution-map.md` §2.3](../m3-rn-runtime-execution-map.md#23-ordinary-nleのささくれ観察inventory)が2026-08-07にPremiere/FCP/Resolve/Avidを横断し、同種の「ささくれ」観察と仮写像を既に行っている(ripple/roll/slip/slide/J-L cutは`UNDERCONSIDERED / SPLIT REQUIRED`、A/V link-unlinkは`UNIMPLEMENTED / MAP THIN`、nested/compound/precomposeは`ALREADY CONSIDERED`)。本調査はその結論を再確認しつつ、AE/CapCut/AviUtl/DAWを加えて幅を広げ、**現行実装コード**(`crates/motolii-ui/src/timeline_editor/mod.rs`ほか)との突き合わせを追加した点が新しい。

## 1. 操作の一覧表

### 1.1 基本操作(全ジャンル共通)

| 操作 | 何をする物か | 在るソフト(件数) | 判定 | 出典 |
|---|---|---|---|---|
| Playhead移動/click-scrub | ルーラ上のクリック・ドラッグで現在時刻を動かし即座にプレビュー | AE/Premiere/Resolve/CapCut/AviUtl/Ableton/Reaper(7/7) | 必須 | [AE composition basics](https://helpx.adobe.com/after-effects/desktop/work-with-compositions/composition-settings/composition-basics.html)、[Ableton Arrangement View](https://www.ableton.com/en/manual/arrangement-view/) |
| クリックで選択(single) | 対象1つを選ぶ | 全数(7/7) | 必須 | [AE selecting layers](https://helpx.adobe.com/after-effects/using/selecting-arranging-layers.html) |
| 複数選択(Shift範囲/Cmd toggle/marquee/全選択) | 複数対象へ同時操作 | AE/Premiere/Resolve/AviUtl(4/7、DAWはclip選択のみでtimeline objectとしては別軸) | 必須 | 同上。Motolii側は[Godot PORT操作カタログ](2026-08-13-godot-editing-system-adoption.md)で既に採択・実装済み |
| ドラッグで時間方向へ移動 | clip/layerの開始位置を変える | 全数(7/7) | 必須 | 各製品マニュアル(下記各項) |
| 端をドラッグしてtrim(in/out変更) | clipの端を掴んで長さを変える | 全数(7/7) | 必須 | [Premiere trim](https://helpx.adobe.com/premiere-pro/using/trimming-clips.html)、[AviUtlレイヤー操作](https://vip-jikkyo.net/aviutl-layers-objects) |
| 現在時刻で分割(Split/Razor) | playhead位置でclipを2つに割る | AE(Ctrl+Shift+D)/Premiere(Cmd+K)/Resolve/CapCut(Ctrl+B)/AviUtl(分割)/Ableton(Cmd+E)(6/7、Reaperも`S`で分割あり実質7/7) | 必須 | [AE split layer](https://community.adobe.com/questions-529/how-do-i-trim-split-a-layer-on-after-effects-24540)、[CapCut split](https://capcutguide.com/how-to-cut-trim-split-video-capcut/) |
| Undo/Redo | 操作の取り消し/やり直し | 全数(7/7) | 必須 | — |
| 選択物の削除(Delete) | 選択対象を消す | 全数(7/7) | 必須 | — |
| Copy/Cut/Paste/Duplicate | 複製・貼付 | AE/Premiere/Resolve/CapCut/AviUtl(5/7) | 必須 | [Godot PORTカタログ](2026-08-13-godot-editing-system-adoption.md) |
| Zoom(anchor保持)/Fit | 拡大縮小、全体表示 | 全数(7/7) | 必須 | 精密な編集の前提。件数根拠は各製品マニュアルの共通挙動 |
| Snap吸着 on/off(clip端・playhead・grid) | ドラッグ時に近傍へ吸着 | 全数(7/7、Final Cut Proの`N`キーも参考) | 必須 | [Ableton snapping](https://www.ableton.com/en/manual/arrangement-view/)、[Final Cut Pro snapping](https://filmora.wondershare.com/advanced-video-editing/final-cut-pro-magnetic-timeline.html) |

### 1.2 トラックベース特有(trim family / packing)

トラックベース系(Premiere/Resolve、および参考にしたFinal Cut Proの慣行)は「track上のclipは隙間なく詰まっている(gapless)」ことを前提に、詰め直しを伴う一体の操作族を持つ。AE・AviUtl・DAWのclip/layerは自由な絶対時間配置(重なり・隙間を許す)であり、この操作族自体が存在しない。

| 操作 | 何をする物か | 在るソフト(件数) | 判定 | 出典 |
|---|---|---|---|---|
| Ripple trim | 端trim時に後続clipを連動して詰める/開ける | Premiere/Resolve(2/7、track-based限定) | 標準 | [Premiere ripple edits](https://helpx.adobe.com/premiere/desktop/edit-projects/trim-clips/perform-ripple-edits.html)、[Resolve trim modes](https://www.blackmagicdesign.com/products/davinciresolve/edit) |
| Roll edit | 隣接2clipの境界を同時移動、総尺不変 | Premiere/Resolve(2/7) | 標準 | 同上 |
| Slip edit | clipの位置・尺は固定、内容(in/out)だけ動かす | Premiere/Resolve(2/7) | 標準 | [Premiere slip edits](https://helpx.adobe.com/premiere/desktop/edit-projects/trim-clips/perform-slip-edits.html) |
| Slide edit | clip位置を動かし前後clipが自動trimされる | Premiere/Resolve(2/7) | 便利 | [Premiere slide edits](https://helpx.adobe.com/premiere/desktop/edit-projects/trim-clips/perform-slide-edits.html) |
| Insert edit | 挿入点で後続clipを右へ押し出して挿入 | Premiere(3点編集の一部)(1/7) | 標準 | [Premiere clip移動](https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/different-ways-to-move-clips.html) |
| Overwrite edit | 挿入点から上書き | Premiere(1/7) | 標準 | 同上 |
| Lift(穴を残す)/Extract(詰めて除去) | 選択範囲を除去、穴を残す/詰める | Premiere(1/7、Reaperのripple deleteが機能的に近い→実質2/7) | 便利 | [Premiere lift/extract/ripple-delete](https://helpx.adobe.com/ph_fil/premiere-pro/how-to/lift-extract-ripple-delete-premiere.html)、[Reaper ripple](https://music.tutsplus.com/how-to-use-reaper-tempo-grid-and-snap-settings--cms-107661t) |
| Track Lock(track単位で編集保護) | lockしたtrackを編集から保護 | Premiere/Resolve/AE(layer lock相当)(3/7) | 標準 | [Premiere track lock](https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/track-lock-to-prevent-changes.html) |
| Sync Lock | ripple操作時に連動して動くtrackを選ぶ | Premiere/Resolve(2/7) | 便利 | [Premiere sync lock](https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sync-lock-to-prevent-changes.html) |
| A/V Link/Unlink | 映像clipと音声clipの紐付けを解除する | Premiere/Resolve(2/7) | 標準 | [Premiere link/unlink](https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/link-audio-and-video-clips.html) |
| 3点編集(In/Out+挿入点) | Source Monitorで決めたIn/Outをtimelineの1点へ流し込む | Premiere(1/7、Resolveにも類似の3点/4点編集あり→実質2/7) | 便利 | [3-point edits](https://helpx.adobe.com/ph_fil/premiere-pro/how-to/three-point-edits.html) |
| J/K/L shuttle(可変速逆/停止/順再生) | キーで再生速度と方向を操作 | Premiere/Resolve系(同慣習を継承)(2/7、AE/AviUtl/DAWには同名の慣習なし) | 便利 | [J/K/L keys](https://nofilmschool.com/2018/10/how-use-j-k-and-l-keys-premiere-pro-speed-your-workflow)(二次資料) |
| マルチカム編集 | 複数カメラ角度の同期切替 | Premiere/Resolve(2/7) | 便利 | Premiere/Resolve公式マニュアル(検索結果内、専用URLは確認できず) |

### 1.3 階層・グループ

| 操作 | 何をする物か | 在るソフト(件数) | 判定 | 出典 |
|---|---|---|---|---|
| 親子(Parent/pick whip) | 子のtransformを親に追従させる | AE(1/7、track-based系には無い概念) | 標準 | [AE pick whip](https://www.freevisuals.net/post/after-effects-pick-whip)(二次資料) |
| プリコンポ/Nest/Compound clip | サブタイムラインを1つのclip/layerとして入れ子にする | AE(precompose)/Premiere(nest)/Resolve(compound clip)(3/7) | 標準 | [AE precompose](https://helpx.adobe.com/after-effects/desktop/work-with-compositions/composition-settings/composition-basics.html) |
| グループ化 | 複数object/layerを1つとして選択・移動できるようにする | AviUtl(グループ化)/Premiere・Resolveはnest/compoundで代用(3/7) | 標準 | [AviUtlグループ化](https://vip-jikkyo.net/aviutl2-grouping) |
| Fold/Unfold(disclosure) | track/propertyの子を折り畳む/開く | AE(property群)/AviUtl(2/7) | 標準 | [AviUtlタイムライン操作](https://vip-jikkyo.net/aviutl-timeline) |
| グループ制御 | 複数objectへ一括で変形・エフェクトを適用する専用object(グループ化とは別概念) | AviUtl(1/7) | 便利 | [AviUtlグループ制御](https://aviutl.info/guru-puseigyo/) |

### 1.4 キーフレーム/アニメーション/リタイム

| 操作 | 何をする物か | 在るソフト(件数) | 判定 | 出典 |
|---|---|---|---|---|
| キーフレーム追加/削除/移動(プロパティ単位) | 各propertyの値変化点をdiamondで打つ | AE(1/7、他はobject単位または無し) | 必須 | [AE keyframes](https://helpx.adobe.com/after-effects/desktop/work-with-compositions/composition-settings/composition-basics.html)。Motoliiは[Godot PORTカタログ](2026-08-13-godot-editing-system-adoption.md)で既決・実装済み |
| 補間/イージング切替 | linear/bezier/hold等を区間ごとに選ぶ | AE/AviUtl(補間曲線)(2/7) | 標準 | 既決([Interval Easing Editor](../ui-interaction-language.md)) |
| 中間点(object単位の変化点) | 1つのobjectの時間を複数区間に割り、区間ごとに補間を持つAviUtl特有モデル | AviUtl(1/7) | 便利 | [AviUtl中間点](https://aviutl.info/tyuukanntenn/) |
| Time Remap/Retime | 素材内部の時間軸を再マップし速度を可変にする | AE(Time Remap)/Premiere(Time Remapping)/Resolve(Retime)/CapCut(Curve speed ramp)(4/7) | 標準 | [AE time remapping](https://helpx.adobe.com/after-effects/using/time-stretching-time-remapping.html)、[CapCut speed ramp](https://filmora.wondershare.com/video-editing-tips/speed-ramp-capcut.html) |
| 単純な速度変更(clip全体を一定倍速) | clip全体を等倍でなく再生する | 全数(7/7) | 標準 | — |

### 1.5 マーカー・波形

| 操作 | 何をする物か | 在るソフト(件数) | 判定 | 出典 |
|---|---|---|---|---|
| マーカー/ロケーター(Mキー) | 時刻へ注釈点を打つ | Premiere/Resolve/Ableton(locator)(3/7) | 標準 | [Premiere markers](https://helpx.adobe.com/premiere-pro/using/markers.html) |
| 波形表示(音声波形) | timeline上に音声のpeakを描く | Premiere/Resolve/Reaper/Ableton/AviUtl(拡張プラグインで一般的)(5/7) | 標準 | [Resolve waveform sync](https://teckers.io/how-to-sync-audio-davinci-resolve/) |
| ビート検出・拍グリッド吸着 | 楽曲を解析し拍位置へマーカー/吸着点を自動生成 | CapCut(Auto Beat Sync)/Resolve 20(Show Music Beats)(2/7) | 便利(汎用) / MVでは§3参照 | [CapCut beat sync](https://vediting.home.blog/2025/10/28/%F0%9F%8E%B6-how-to-sync-transitions-with-music-beats-in-capcut-beat-sync-tutorial/)(二次資料)、[Resolve 20 beat markers](https://www.videoeditorlondon.co.uk/post/davinci-resolve-timeline-show-music-beats)(二次資料) |

### 1.6 再生・スクラブ

| 操作 | 何をする物か | 在るソフト(件数) | 判定 | 出典 |
|---|---|---|---|---|
| ループ区間再生(loop brace/time selection) | 指定区間を繰り返し再生 | Ableton/Reaper/Premiere(3/7) | 標準 | [Ableton loop brace](https://www.ableton.com/en/manual/arrangement-view/)、[Reaper looping](https://silentsongstudios.com/how-to-use-reapers-time-selection-and-looping-for-precise-editing/) |
| Playhead追従(再生中に窓が自動で動く) | 再生位置が画面外に出ないよう窓をスクロールする | Ableton/Ardour(参考)/LMMS(参考)/Premiere/Resolve(DAW系で特に体系化) | 標準 | Motolii側は既に[Ardour/LMMS/Ableton実装調査](2026-08-16-daw-playhead-follow-prior-art.md)を実施済み、実装は同日撤廃(症状の誤診が原因) |

### 1.7 その他

| 操作 | 何をする物か | 在るソフト(件数) | 判定 | 出典 |
|---|---|---|---|---|
| ラベルカラー(clip/layerの色分類) | 色で種別・状態を分類する | AE/Premiere/Resolve(3/7) | 便利 | [AE layers/labels](https://helpx.adobe.com/after-effects/using/layers.html) |

## 2. Motoliiの既決との突き合わせ

### 2.1 既に決めている/実装済みの物

repo内の既決文書とソースコード(`crates/motolii-ui/src/timeline_editor/mod.rs`ほか、cargoは実行せずgrepのみで確認)を照合すると、上記「必須」判定12件のうち大半は既に決定または実装済みだった。

| 操作 | Motolii側の裏付け |
|---|---|
| 複数選択・Copy/Paste/Duplicate・Fold/Unfold | [Godot PORT操作カタログ](2026-08-13-godot-editing-system-adoption.md)(2026-08-13決定) |
| 親子(型付きリンク) | `ae-pain-points.md` C-補遺、GAP-8(2026-07-10決定) |
| グループ化・fold/unfold | [ui-interaction-language.md §5.1](../ui-interaction-language.md)「通常Groupは同じStageとTimeline上でfold/unfold」 |
| キーフレーム追加/削除/移動、補間切替 | 同上に加え、`timeline_editor/mod.rs`の`add_key_at_playhead`/`delete_selected_keys`/`key_param_at_playhead`等が実装済み |
| Snap吸着 | `timeline_editor/mod.rs`の`snap_candidates`/`snapped`/`dragging_snaps_to_nearby_edges_and_keys_but_not_to_itself`テストが実装済み |
| 分割(Split) | `timeline_editor/mod.rs`の`split_selected`/`splitting_at_the_playhead_makes_two_clips`が実装済み |
| Zoom(anchor保持)/Fit | `timeline_editor/mod.rs`の`zoom_at`/`zoom_keeps_the_anchor_time_under_the_cursor`、UIに`Fit to composition`ボタンが実装済み |
| マーカー/ロケーター | `timeline_editor/mod.rs`の`add_locator`/`tap_locator`が実装済み。コード中コメントに「セクションを割る操作(Ableton の再生中 locator 追加と同じ)」と明記があり、Ableton由来と自認している |
| 波形表示 | `crates/motolii-ui/tests/timeline_waveform_band.rs`が実装済み(「MVを作る人にとって波形は同期の地図そのもの」とコメントで既に言語化) |
| ループ区間再生 | `timeline_editor/mod.rs`の`loop_grab_for`/`loop_from_drag`/`a_loop_drag_reads_the_same_in_either_direction`が実装済み |
| Playhead追従 | [DAW追従調査](2026-08-16-daw-playhead-follow-prior-art.md)を実施済み。実装は一度Labへ入れたが同日撤廃(利用者体験の誤診が理由。追従の型そのものは調査記録として再利用可) |

**副産物の発見**: マーカー(ロケーター)と波形表示は実装済みだが、`ui-interaction-language.md`には明文化されていない。iced移植時に「既決文書に無いから移植対象外」と誤読されるリスクがある。これは調査の発見であり、台帳への追記自体は利用者判断に委ねる。

### 2.2 決めていない物(空席)

| 操作群 | Motolii側の現状 |
|---|---|
| Ripple/Roll/Slip/Slide/J-Lカット(trim family) | [2026-08-03のdecision-index行](../decision-index.md)が既に「trim edge hit-zone、slip/slide/roll/ripple、multi-select、lane変更、playhead/marker/frame-grid snapは別境界へ残す」と明記。`timeline_trim_gesture.rs`/`timeline_move_gesture.rs`はどちらも単一clipのbody move/trimのみで、ripple系の実装は見当たらない |
| Insert/Overwrite/Lift/Extract(3点編集ファミリー) | ソース内に該当する関数・型が見当たらない。未言及 |
| A/V Link/Unlink、Track Lock、Sync Lock | [`m3-rn-runtime-execution-map.md` §2.3](../m3-rn-runtime-execution-map.md#23-ordinary-nleのささくれ観察inventory)が`UNIMPLEMENTED / MAP THIN`と分類済み(2026-08-07)。Motoliiの音声モデルは単一soundtrack([2026-08-17-edit-during-playback-decision.md](2026-08-17-edit-during-playback-decision.md))であり、Premiere/Resolve型の「clipに埋め込まれた音声channelのlink/unlink」とは前提が異なる可能性が高い(確認課題として残す) |
| マルチカム編集 | 同map §2.3で`SPECIALIZED / BASELINE UNADJUDICATED` |

### 2.3 AE痛点として意図的に避けている物(欠落ではない)

`ae-pain-points.md` E「プリコンポ地獄・ネストが煩雑」は`[コア解決]`としてプリコンポ廃止・グループ化+ベイクへの置換が既に決定されている。今回確認したPremiere Nest／Resolve Compound Clipは、AEのプリコンポと同型の「サブタイムラインを別コンテナへ隠す」設計であり、Motoliiが避けた対象と機構が同じである。**これは新しい発見ではなく、既決(グループ化+fold、§2.1で実装確認済み)がAE/Premiere/Resolve共通のプリコンポ相当機能への正しい代替であることの裏付けである。** 一覧表1.3の「プリコンポ/Nest/Compound clip」を漏れとして数えない。

## 3. 音楽映像(MV)特化の観点

Motoliiの製品命題は「3〜5分のMVを作る」。汎用エディタでは便利止まりだがMVでは必須に近づく操作:

- **ビート検出・拍グリッド吸着**: 確認できたのはCapCut(Auto Beat Sync)とDaVinci Resolve 20(Show Music Beats、2026年リリースのネイティブ機能)の2製品のみ。AE/Premiere/AviUtl/Ableton/Reaperにはネイティブの「動画clipを拍へ吸着させる」一体型UIを確認できなかった(Ableton/Reaperは音楽制作ソフトなので拍グリッド自体は基盤機能だが、それを動画clip配置の吸着に使う機能ではない)。**件数根拠は2/7と少ないが、MVでは編集の起点になりうる操作**であり、汎用エディタでの標準性とは別に評価すべき候補として残す
- **波形表示**: §2.1のとおり実装済み。`timeline_waveform_band.rs`のコメントが既に「MVを作る人にとって波形は同期の地図そのもの」と言語化しており、この調査は追加の発見ではなく裏付け
- **Time Remap/速度変更**: 汎用エディタでも「標準」判定だが、MVでは「ビートに合わせて尺を伸縮する」使用頻度が上がると推測される(件数根拠なし、推測に留める)。Motolii側は`TimeMap`(GAP-19)が既決だが、[`m3-rn-runtime-execution-map.md`](../m3-rn-runtime-execution-map.md)は`KNOWN / PRODUCT AUTHORING UNIMPLEMENTED`と分類しており、製品timeline UIへの接続は未接続のまま

## 4. 評決

今回確認した7製品と照らして、Motoliiのegui/iced Timelineに「普通のタイムラインとして決定的に欠けている」個別操作は、grepで確認できた範囲では見当たらなかった――必須判定した12件(playhead移動・選択・複数選択・drag移動・端trim・分割・Undo/Redo・削除・Copy/Paste/Duplicate・Zoom/Fit・Snap・キーフレーム編集)は、ソースコードまたは既決文書のどちらかに裏付けがあった。決定的に欠けているのは個別操作ではなく、**「track/laneが隙間なく詰まっているか(gapless packing)、自由な絶対時間配置か」という土台そのものの未選択**である。確認した7製品のうちトラックベース系(Premiere/Resolve、および参考にしたFinal Cut Proの慣行)はgapless packingを前提にripple/roll/slip/slide/insert/overwrite/lift/extract/sync lockという一体の操作族を持つが、レイヤーベース系(AE)とAviUtl・DAWのclip/layerは自由な絶対時間配置を許し、この操作族自体を持たない。Motoliiのソースを読む限り"lane"はAE型の層識別語(`video lane`/`audio lane`)であり詰め配置の保証を持たず、concept.mdの主参照もAE+Cavalryである。2026-08-03のdecision-index行が「trim family(ripple/roll/slip/slide)は別境界へ残す」と明記した判断は、この土台未選択と整合している。**この調査は「trim familyを追加すべき」という結論を出すものではない**(規律6点、調査は決定ではない)――明らかになったのは、この土台(自由配置のままか、gapless packingを採るか)を利用者が先に選ぶまでは、trim family個別追加は既存の自由配置move/trim gesture(`timeline_move_gesture.rs`/`timeline_trim_gesture.rs`)と機構的に衝突する、という条件だけである。標準・便利判定の残り(3点編集、A/V link、track/sync lock、マルチカム、J/K/L)は、Motoliiの自由配置設計とMV特化スコープの両方から見て優先度は低いが、「本当に不要か」を利用者自身が確認すべき空席として残る。

## 関連

- [`docs/ae-pain-points.md`](../ae-pain-points.md) — AE痛点カタログ、正本
- [`docs/ui-interaction-language.md`](../ui-interaction-language.md) — Timeline操作文法の既決
- [`docs/ui-quality-bar.md`](../ui-quality-bar.md) — Q0(触達性)ほか操作品質規律
- [`docs/decision-index.md`](../decision-index.md) — trim family/A・V link等の空席判定の出典行
- [`docs/m3-rn-runtime-execution-map.md` §2.3](../m3-rn-runtime-execution-map.md#23-ordinary-nleのささくれ観察inventory) — 先行する内部調査(2026-08-07)
- [Godot編集系PORT採択](2026-08-13-godot-editing-system-adoption.md) — 選択/複製/fold/keyframeの既決
- [DAWのplayhead追従調査](2026-08-16-daw-playhead-follow-prior-art.md) — 追従の型の先行調査と撤回
- [egui Timeline engine正本](2026-08-15-egui-timeline-engine-authority.md)、[Skia Timeline正本訂正](2026-08-16-skia-timeline-authority-correction.md) — engineの歴史的経緯
- [レビュー文書の規律](README.md) — 調査結論を設計根拠にしない
