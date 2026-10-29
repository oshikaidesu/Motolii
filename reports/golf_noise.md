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
- [ ] ③ 時間軸のキーがマウスで扱えない：キーを打った値が時間軸に一覧で出ない（線は帯の中だけ）、キーは選んで Delete でしか消えない（ゴルフでは、直前に数字の欄を触っていて Delete も効かなかった）。出典は Ableton のオートメーション（mouse_map.md 9 番）：トラックの「＋」で動きの付いた値ごとに自分の段が出る（⌘ で全部）、線の上の点を押すと消える、線の上を押すと点が足せる
- [ ] ④ 欄の確定がばらばら：文字の欄は Return で確定せず Tab で確定、数字の欄は Tab で確定せず Return で確定。文字の欄はダブルクリックで単語が選べない。時間の数字（00:00:00）に打ち込めない
- [ ] ⑤ 一押しで効かない：Fill の白い四角を押しても何も起きない／パレットの色を押しても付かず Set が要る／効果の絵をダブルクリックしても載らない（Add は効く）／Animate all の「Off」の文字を押しても切り替わらない（Inspector は Ableton が手本：Animate all は Automation Arm、数字の中の点は動きの付いたつまみの印、行の右クリックで Key this frame・Remove key。Scale の左の印はキーでなく、横に引いて数字を動かす Figma の印）
- [ ] ⑥ 新しく置いた文字に色が無く、黒に近い灰色で見えない（Alight Motion は白で出る）
- [ ] ⑦ Bounce In を載せても動かない（Split LETTERS でも、エンジンで 0.45・0.6・1.5 秒を描いても同じ形）。動き（animators/*.js）を確かめる検査が 1 本も無い
- [ ] ⑧ 窓が落ちる（SIGABRT、FEngine::loop の WebGPUDriver::beginRenderPass の中で std::terminate）。そのとき裏で効果の棚の見本を描くためにエンジンが 10 個以上（effect_tiles.dart _draw：12 本ごとに Motolii.open）。試験では見本を描かない（FLUTTER_TEST）。再現は 3 回試して 3 回とも落ちず、原因はまだ切り分けていない
- [ ] ⑨ 再起動（live.sh に SIGUSR2、hot restart）で落ちる（SIGSEGV、エンジンのスレッドが消えた Dart の呼び先を呼ぶ）。開発中だけ
