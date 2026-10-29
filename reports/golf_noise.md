# ゴルフで見つけたノイズ

窓だけで 5 秒のタイトル（文字が弾んで出る・光る・着地でカメラが揺れる・ゆっくり寄る）を作ったときに、押しても思ったとおりにならなかった所。上から 1 つずつ、原因を切り分けてから直す。直したら [x] にして、出典とコミットを書く。

1 手 = クリック・ドラッグ・キー・文字入力の 1 回。打率 = 「ここだろう」と押して 1 回で当たった割合。

## スコア（10/29、1 回目、⑧ の落ちで中断）

| ホール | 打数 | 当たり | 外れた所 |
|---|---|---|---|
| 文字を置く | 2 | 2/2 | — |
| 文字を書き換える | 7 | 4/7 | ダブルクリックで単語が選べない、Return で確定しない |
| 文字に色を付ける | 4 | 2/4 | Fill の四角が効かない、パレットの色は Set が要る |
| 弾んで出てくる | 40 前後 | — | 下の ②③④⑤⑦ と ⑧ |

## 直すもの

- [x] ① 窓が試作の枠（Widgetbook）の中で開く、画面いっぱいでない — 済（10/29、11503e8c2）：live.sh は枠なし（MOTOLII_BARE）、Mac の窓は NSScreen.visibleFrame、DEBUG の帯なし
- [x] ② 時間軸の定規を押すと再生が始まる（After Effects・Premiere は再生位置が動くだけ）。そのとき窓で例外が 2,235 回（Ticker の二重の start：timeline_view.dart 10 行 ← timeline_asked.dart 76 行 _rulerTap → _playAt） — 済（10/29）：定規を押すと再生位置がそこへ動き、再生中なら止まる（Alight Motion：時間軸のどこを触っても再生が止まる、AbsentMotion の Timeline.kt。押した所へ動くのは、再生位置が真ん中に固定でない PC の形で、Audacity の定規と同じ）。例外は、時間軸が extension の関数（_changed）を聞き手に渡していて、作り直すたびに古い聞き手が外れずに残り、捨てた Ticker を start していた（Flutter の ticker.dart：dispose で _startTime を 0 にしておく）。聞き手を 1 つの包みにした。窓で 2 秒・4 秒を押して止まったまま、例外 0。試験：workspace_timeline_test.dart（作り直してから再生しても例外なし、直す前の形では同じ例外で落ちる）
- [x] ③ 時間軸のキーがマウスで扱えない：キーを打った値が時間軸に一覧で出ない（線は帯の中だけ）、キーは選んで Delete でしか消えない（ゴルフでは、直前に数字の欄を触っていて Delete も効かなかった）。出典は Ableton のオートメーション（mouse_map.md 9 番）：トラックの「＋」で動きの付いた値ごとに自分の段が出る（⌘ で全部）、線の上の点を押すと消える、線の上を押すと点が足せる — 済（10/29）：A／Animate all を入れている間、キーのある値ごとに段が出る（Ableton の Cmd で全部を段に：Live 12 manual 25.5、「Show Automated Parameters Only」と同じくキーのある値だけ）。段にはキーが点、間はまっすぐの線（書類の絵の keyMoments・keyValues だけで描き、再生中は中身に聞かない）。点を押すと消え、線を押すと点が足せる（25.5.2）。切ると元の帯だけに戻る。窓で scale_x の線の 2 秒を押して点が増え、その点を押して消えた。試験：workspace_timeline_test.dart。まだ：点を引いて動かす、段の名前を Inspector の呼び名に
- [ ] ④ 欄の確定がばらばら：文字の欄は Return で確定せず Tab で確定、数字の欄は Tab で確定せず Return で確定。文字の欄はダブルクリックで単語が選べない。時間の数字（00:00:00）に打ち込めない — 切り分け（10/29）：数字の欄は Return でも Tab でも、欄から離れても確定する（試験で 42 と打って Tab → 入った。ゴルフの時は直前のダブルクリックと ⌘A でキーボードの向きがずれていたらしく、再現せず）。文字の欄の Return は、何行もある文字の改行で正しい（Alight Motion・Figma の文字の打ち込みと同じ）、離れると確定。直すのは単語の選択だけ：窓の打ち込む欄は十数か所すべて素の EditableText で、Flutter の TextField が付けている TextSelectionGestureDetectorBuilder（ダブルクリックで単語、引いて範囲）が無い。まとめて付ける。時間の数字は Alight Motion では打ち込まない（押すとブックマーク：AbsentMotion の Timeline.kt Hit.Readout）ので外す
- [x] ⑤ 一押しで効かない：Fill の白い四角を押しても何も起きない／パレットの色を押しても付かず Set が要る／効果の絵をダブルクリックしても載らない（Add は効く）／Animate all の「Off」の文字を押しても切り替わらない（Inspector は Ableton が手本：Animate all は Automation Arm、数字の中の点は動きの付いたつまみの印、行の右クリックで Key this frame・Remove key。Scale の左の印はキーでなく、横に引いて数字を動かす Figma の印） — 切り分け（10/29）：Fill の四角は再現せず（窓で 1 回押すと色の棚が開く、文字の欄の直後でも）。パレットは決まりどおり（Ableton のブラウザ：1 回で選ぶ、2 回で塗る、下のボタンでも塗る：dock_make_2.dart、効果の棚と同じ）。効果のダブルクリックは効く（窓で Glow）、動きのダブルクリックも試験あり（workspace_joined_test_4.dart の spring_in）でゴルフの時の 2 回押しは再現せず。残るのは「Off」の文字だけ：forms/own.dart の onOff は文字がただの文字で、押せるのはスイッチだけ — 済（10/29）：On/Off は文字もスイッチと同じく押せる（forms/own.dart、AbsentMotion の SwitchRow：行ごと toggleable）。窓の On/Off のスイッチ全部が同じ型なので全部。窓で「On」を押して Off に。試験：workspace_joined_test_2.dart の Animate all（文字を押す。直す前の形では落ちる）
- [x] ⑥ 新しく置いた文字に色が無く、黒に近い灰色で見えない — 切り分け（10/29）：中身の doc_add（doc_edit.h）が文字に塗りを入れていない（fill は空）。「Alight Motion は白」は思い込みだった：AbsentMotion の addText（TextActions.kt）は塗り 0xFF7F7F7F（真ん中の灰色）で始める。それに合わせる — 済（10/29）：中身の doc_add が新しい文字に塗り 7f7f7f を入れる。窓で T を押して置いた文字が真ん中の灰色ではっきり見える。試験：workspace_joined_test.dart の round 2（新しい文字の塗り）、check/all.sh 全部通過。直近に使った色で出す（ユーザー）も済：物に塗った色が次に描く物の色（ink・edge）になる（CLIP STUDIO PAINT の描く色は 1 つで、文字も含めてどの道具もその色で描く）。窓で下の Text をピンクに塗り、次に置いた Text がピンクで出た。試験：workspace_joined_test_2.dart の inks
- [ ] ⑦ Bounce In を載せても動かない（Split LETTERS でも、エンジンで 0.45・0.6・1.5 秒を描いても同じ形）。動き（animators/*.js）を確かめる検査が 1 本も無い
- [ ] ⑧ 窓が落ちる（SIGABRT、FEngine::loop の WebGPUDriver::beginRenderPass の中で std::terminate）。そのとき裏で効果の棚の見本を描くためにエンジンが 10 個以上（effect_tiles.dart _draw：12 本ごとに Motolii.open）。試験では見本を描かない（FLUTTER_TEST）。再現は 3 回試して 3 回とも落ちず、原因はまだ切り分けていない
- [ ] ⑨ 再起動（live.sh に SIGUSR2、hot restart）で落ちる（SIGSEGV、エンジンのスレッドが消えた Dart の呼び先を呼ぶ）。開発中だけ
- [x] ⑩ 文字を置いても、すぐ打てない：置くと「Text」と出て、打ち直すには右の Inspector の欄を探す（ゴルフのホール 2 が 7 打）。AbsentMotion の addText は、足すとすぐ文字を打つ所を開く（panel = EditText） — 済（10/29）：T で置いた文字が選ばれると、Inspector の Text のカードが作り直されたあとで、文字の欄にキーボードが向き、中身が全部選ばれる（Figma の文字の道具はその場で打てる）。窓で T → 置く → HELLO → Tab で、文字が HELLO に。試験：workspace_joined_test_3.dart（置いてすぐ打つ、直す前の形では落ちる）
- [ ] ⑪ 効果の見本の絵が、どれも写真が左右に動くだけで、効果が何をするのか見えない（ユーザー、10/29）：effect_tiles.dart は写真の x を 0 秒から 2 秒へ動かして 8 コマ撮っている。出典：AbsentMotion の tools/effect_showcase.json（効果 181 本、効果ごとに 6 枚の写真のどれに掛けるか、代表のつまみの値 params、絵の 4 割で「掛ける前｜後」に分ける split、時間でつまみを動かす animate）。ISF の効果（420 本）には表が無いので、別に出典を探す
- [ ] ⑫ Ease のパネルと時間軸が合っていない（ユーザー、10/29）：時間軸の段はキーの間をまっすぐの線で描き、Ease のパネルで決めた曲がり方（Bounce など）が出ない。段の点や線を押しても Ease のパネルにつながらない。出典：Alight Motion のイーズは、再生位置が 2 つのキーの間にある時、その区間を開く（alight_ui.md イーズ）。Ableton は曲げた区間を線の上でも曲げて描く（Live 12 manual 25.5.2 の ⌥ ドラッグ）
- [ ] ⑬ 色の棚が分かりにくい（ユーザー、10/29）：グラデーションが別物のように扱われている、スイッチの所が分かりにくい、素材（マテリアル）が 1 つだけでつまらない、ガラスは物に掛けるものなので色の棚でなく効果か Create に入るのが自然。良い所：パターン（パレット）。出典は Alight Motion の色と塗り（alight_ui.md 81〜84 行、絵 21・22・23）：塗りの種類「なし・単色・グラデーション・メディアで塗る」が 4 つのタブで一目に並び、グラデーションはその 1 つ。ユーザー（10/29）追加：3 色のグラデーションが作れない、色の補間（色と色の間のつなぎ方）を選べない。Alight Motion のグラデーションは 2 色だけ（記録の「悪い」所）なので、ここは PC のソフトから：Blender のカラーランプ（つなぎ方 Linear・Ease・B-Spline など、色の空間 RGB・HSV・HSL、色相の近回り・遠回り）、CSS Color 4 の color-interpolation-method（in oklab、in hsl longer hue）。Alight Motion は下限で、そこより上を出す（ユーザー：After Effects ではプラグインになる色のつなぎ方を、Motolii は最初から持てる）
