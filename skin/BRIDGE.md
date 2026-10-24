# 窓と中身の境目（2026-10-24 から）

窓（Flutter、`skin/book`）と中身（C++、`filament/motolii.mm`）の間の決まり。仲間の作りに揃える：Lumit（`docs/17-BRIDGE-CONTRACT.md`）、
AppFlowy（「誰の何が変わったか」を1本で流す）、Rive の CommandQueue（書きは記録して戻る、問い合わせは後で届く）、Anthem（毎コマ変わる物は購読）、
Thermion（描けは前が終わるまで捨てる）。調べた記録は `research_notes/flutter_native_kin.md`。GPL・AGPL の物は考え方だけ借りる。

## 決まり

1. **窓は中身を待たない。** 窓のスレッドから中身を呼んで、答えが出るまで止まる道は1本も無い。
2. **窓は組み立て（build）の中で中身を呼ばない。** 読むのは中身が渡した写しだけ。
3. **写しは2つ。** どちらも中身の「手」のスレッドが、書類の鍵を持っている間に作り、出来上がった物を丸ごと差し替える（変更できない物、版の番号つき）。
   - **書類の写し（D）**：書類が変わった時だけ作る。物の一覧と、それぞれの物の種類・名前・長さ・キーの位置・区間・カード・選べる言葉・範囲・見せ方・効果の並び・止まっている値、
     書類全体の印・ガイド・作業範囲・長さ・大きさ・方の一覧。物ごとに版の番号を持ち、変わらなかった物は前と同じ物のまま（窓は `identical` で比べられる）。
   - **コマの写し（M）**：絵を1枚描くごとに作る。その時刻、動いている値、物ごとの枠と見えている四角、運ばれている値（`shows`）、選んだ物のキーの有無・点・取っ手。
4. **中身は「何が変わったか」を1本で知らせる。** 知らせは `NativeCallable.listener`（Dart SDK の勧め）を1つだけ。中身から送るのは、どちらの写しが新しくなったかと、書類なら変わった物の番号。
   窓は1コマに何回来ても、受け取りは1回にまとめる。
5. **窓は聞きたい物だけを聞く。** `Ws` の中は `ValueNotifier` に分ける：書類（全体と物ごと）、コマ、それに窓だけの状態
   （再生ヘッド、再生中、選んだ物、選んだキー、desk、棚、見え方、Animate）をそれぞれ1つずつ。何でも知らせる口（`Ws` 全体の `notifyListeners`）は作らない。
6. **書き（W）は投げるだけ。** 列に積んで、すぐ戻る。番号が要る物（`add`・`make`）は、答えを後で写しと一緒に受け取る。同じ物の同じ値への書きが列に残っていたら、最新の1つにまとめる。
7. **今の時刻以外を聞く物と、長い物（Q）は問い合わせ。** 頼んで `Future` で受け取る。組み立ての中では待たない（届いたら、その部品だけ描き直す）。
8. **起動時に1回の物（S）**（レシピ・効果・アニメーターのフォルダ、フォントの種類）は、窓が開く時に別のスレッドで1回読む。
9. **再生ヘッド・選択・ズームは窓の持ち物**（Lumit と同じ）。中身へは書きとして伝えるだけ。
10. **見張り。** `integration_test/frames_test.dart`：窓が中身を待った回数は 0（今の赤ポチ）。再生ヘッドを1つ動かした時と、再生を押した時に組み立て直される部品の数に上限を付け、超えたら赤。

## 潰す物：隠すための自作の仕組み（全部消す）

| # | 物 | 場所 | 何をしていたか | 置き換え | 済 |
|---|---|---|---|---|---|
| H1 | `_kept` `_keptCarried` `_copy` `_was` `_copyCarried` `_wasCarried` | `skin/book/lib/motolii.dart` 604〜641 | 書類の版が変わるまで読んだ物を覚える | 書類の写し（D） || 橋から消した（10/24） |
| H2 | `_asked` `_atMoment` `_dontWait` `_refused` | 同 643〜665 | 中身が忙しければ断って前の値を出す | 写し（D・M）。待つ場面がもう無い || 橋から消した（10/24） |
| H3 | `again` | 同 663、`workspace/ws.dart` 130〜148 | 断られた値を次のコマで窓全体／コマを見る部品にもう一度読ませる | 写しの差し替えの知らせ || 橋と ws.dart から消した（10/24） |
| H4 | `_readings` `_shownOf` `_fontKinds` `_folders` | 同 | 1回読んだら覚える | D（読み方・見せ方）、S（フォルダ・フォント） || `_readings` は消した。`_shownOf` は写しを読み解いた物の置き場として残る（写しが新しくなると空にする）。`_folders`（S）と `_fontKinds`（書類を読まない CoreText の答え）は残す |
| H5 | `forget()` | 同 605 | 覚えた物を捨てる | 写しの差し替え || 消した（10/24） |
| H6 | `counters`（revision・carried・frames・kept の数字） | 同 225 | 窓が毎コマ数字を見て変化を探す | 中身からの知らせ（決まり4） || 消した（10/24） |
| H7 | `_peek` `_counted` `beat()` | `workspace/ws.dart` 205〜235 | 毎コマ数字を覗き、変わっていれば読み直し | 中身からの知らせ || 消した（10/24）：中身の聞き口（`motolii_tell`、`NativeCallable.listener`）から受け取る |
| H8 | `seen` | 同 214 | 「何か新しく見えた」を数を足して知らせる | コマの写しの `ValueNotifier` | |
| H9 | `shown()` `rev` | 同 252 | 何か変わったら窓全体に知らせる | 物ごとの書類の `ValueNotifier` | |
| H10 | `reread()` `_reread` `reads` | 同 258〜297 | 書類を窓が全部読み直して `layers` を作り直す | 書類の写しから `layers` を作る（読み直しは中身がやる） | |
| H11 | `dock._later` `_seen` `_rev` `_due` | `workspace/dock.dart` 55〜90 | 棚が全部の変化を聞いて、自分に関係あるか比べる | 棚は自分の見る物（書類の該当部分・選択）だけ聞く | |
| H12 | インスペクタの `_changed` `_changedNow` `_onlyNumbers` `_seenNow` `_live` `_settle` `_keyed` `_frame` | `workspace/workspace.dart` 700〜765 | インスペクタが全部の変化を聞き、数字だけか比べ、忙しければ 150 ms 後に読み直す | 選んだ物の書類の写し＋コマの写しを聞く | |
| H13 | `timeline._changed`（再生の開始・停止を変化から探す） | `workspace/timeline.dart` 37〜72 | 全部の変化を聞いて再生中かを見る | 再生中の `ValueNotifier` を聞く | |
| H14 | `panes._follow` | `workspace/panes.dart` 40〜60 | 全部の変化を聞いて前に出すパネルを探す | 前に出すパネルの `ValueNotifier` | |
| H15 | `Ws` 全体の `notifyListeners` 15か所 | `workspace/ws.dart` 174 255 296 305 318 329 339 349 355 376 385 395 411 431 455 | 何が変わっても全員に知らせる | 決まり5の口ごとに分ける | |
| H16 | `WsScope.of`（全部を聞く） | 27か所・14ファイル | 何が変わっても組み立て直し | 聞く物ごとの口（`WsScope.frameOf` と同じ形） | |
| C1 | `by_look`（46か所）・`dont_wait`・`refused` | `filament/motolii.mm` 159〜182、984〜985 | 窓が書類の鍵を取って読む／忙しければ断る | 写しを作る所（手のスレッド）だけが書類を読む | |
| C2 | `Peek`・`peeked` | 同 35〜45、298〜318、515〜522 | 選んだ物の値だけ鍵なしで置いておく | コマの写し（全部の物） | |
| C3 | `by_hand` の待ち（31か所、`done.get()`） | 同 125〜135、`asked` | 窓が書きの終わりを待つ | 投げるだけ（決まり6） | |
| C4 | `motolii_watch` `motolii_broken` `broken`（赤ポチ） | 同 180、989〜995 | 窓が待った回数を数える | 待つ道が無いので、テストの見張り（決まり10）だけ残す | |
| C5 | `revision` `carried` の数字 | 同 67〜68、982〜983 | 窓が覗く数字 | 知らせ（決まり4） | |
| C6 | `m->word` `m->listed` の共有の文字置き場 | 同 | 呼ぶたびに上書きされる戻り値 | 写しの中の文字 | |

## 潰す物：窓が中身を呼んでいる所（608か所、46ファイル）

置き場ごとの数（`skin/book/tool/bridge_sites.py lib` で数えた。行は付録。最後にこれが D・M・S・Q の読みを 0 と言えば済み）：

| 置き場 | 呼び | か所 |
|---|---|---|
| D 書類の写し | word 87, point 21, shown 17, things 16, words 11, cards 9, keyMoments 7, label 6, moving 6, thingEffects 4, looping 3, counters 3, busy 3, range 3, guides 2, whole 2, drawnAt 2, picture 2, shaping 2, size 2, now 2, lasts 2, trouble 2, reading 2, values 2, ほか1ずつ（rate kept fits seconds thingKeyMoments thingStretches keyingNow work marks answerWhy brokenCount broken fxTitles wayNames） | 232 |
| M コマの写し | shows 11, frame 8, points 2, handles 2, seen 2, keyed 1 | 26 |
| M か Q | value 74（今の時刻なら M か D、違う時刻なら Q。1つずつ分ける） | 74 |
| Q 問い合わせ | wayOf 4, fontTags 3, wayAt 3, path 3, kindOf 1, thumbnailAt 1, strikes 1, loudnessOver 1, loudness 1, setting 1, write 1, wayNumbers 1 | 21 |
| S 起動時に1回 | recipes 6, effects 2, animators 1 | 9 |
| W 書き | later 129, at 57, add 23, carry 10, take 9, close 5, key 2, make 2, resize 2, set 2, desk 1, pick 1, forget 1, play 1, keying 1 | 246 |

（at・add・close など、名前が中身の物ではない呼びの取り違えが混ざっている。置き換えの時に1行ずつ見て、違う物は「済」に「対象外」と書く。）

## 写しの中身（10/24、`filament/picture.h`）

- 書類の写し：全体の部分（長さ・大きさ・作業範囲・印・ガイド・方の一覧・方の曲線（65点）・効果の一覧・変わった物の一覧・値の初期値の表・種類ごとに知っている値の表）と、物ごとの部分（長さ・値の名前・動く値の名前・カード・効果の並び・区間・キーの時刻・言葉・止まっている値・範囲・言葉の選択肢・読み方・見せ方・値ごとのキーの時刻とその方）。物ごとの部分は作った版の番号を持ち、窓は「この版より新しい物」を頼む。
- コマの写し：時刻、物ごとの枠・見えている四角・見えている値（x y z rotate scale_x scale_y opacity と、関係が動かしているか）、選んだ物の動く値・点・取っ手。
- 測った（10/24）：あなたの書類で写し全部 12 ms・50 KB、物1つの編集で 3.4 ms。15レシピの重い書類で全部 48 ms・252 KB、物1つで 4.1 ms。コマの写しは 3 KB、作るのに約 1.1 ms（手のスレッドで、絵1枚ごと）。

## 進め方

1. 中身：写しを作る（rapidjson、ThorVG に入っている物を使う）、知らせを1本足す。窓：受け取って `Ws` の口に置く。写しを作る時間と大きさを測る。
2. 窓：付録の表をファイルごとに写しへ置き換える。済んだ行に印を付ける。
3. 窓・中身：H1〜H16、C1〜C6 を消す。済んだ行に印を付ける。
4. 見張り（決まり10）を足して、frames_test を緑にする。

## 付録：窓が中身を呼んでいる所（ファイル・行）

| ファイル | か所 | 行：呼び（置き場） | 済 |
|---|---|---|---|
| workspace/workspace.dart | 108 | 74:close(W) 108:write(Q) 109:trouble(D) 137:later(W) 138:later(W) 147:later(W) 154:later(W) 156:later(W) 158:later(W) 164:later(W) 166:later(W) 262:resize(W) 266:now(D) 268:at(W) 279:close(W) 362:word(D) 363:word(D) 384:word(D) 384:word(D) 385:cards(D) 387:shown(D) 388:value(M/Q) 389:words(D) 390:word(D) 390:word(D) 390:words(D) 399:cards(D) 401:shown(D) 401:words(D) 401:words(D) 424:word(D) 443:later(W) 447:shown(D) 448:later(W) 449:later(W) 451:later(W) 463:cards(D) 465:range(D) 465:words(D) 466:shown(D) 467:word(D) 468:word(D) 468:word(D) 470:shown(D) 471:word(D) 472:shown(D) 473:word(D) 474:word(D) 485:cards(D) 498:shown(D) 501:value(M/Q) 504:words(D) 505:word(D) 507:range(D) 518:value(M/Q) 524:cards(D) 526:shown(D) 527:shown(D) 529:shown(D) 540:shows(M) 542:shown(D) 542:value(M/Q) 542:value(M/Q) 542:word(D) 542:words(D) 542:words(D) 546:reading(D) 547:reading(D) 551:drawnAt(D) 560:shows(M) 560:value(M/Q) 569:cards(D) 575:frame(M) 576:cards(D) 585:shown(D) 590:shows(M) 591:value(M/Q) 601:words(D) 604:later(W) 606:words(D) 607:later(W) 609:later(W) 614:take(W) 619:carry(W) 619:trouble(D) 626:later(W) 638:keyed(M) 649:moving(D) 651:cards(D) 661:values(D) 664:later(W) 666:later(W) 668:later(W) 689:keyMoments(D) 689:moving(D) 704:range(D) 704:shaping(D) 704:shown(D) 722:busy(D) 740:busy(D) 789:values(D) 794:path(Q) 794:point(D) 796:fxTitles(D) 797:later(W) 808:later(W) 811:later(W) 813:later(W) | |
| workspace/stage_view.dart | 64 | 157:desk(W) 181:pick(W) 205:value(M/Q) 205:value(M/Q) 233:value(M/Q) 233:value(M/Q) 288:guides(D) 301:later(W) 302:later(W) 303:later(W) 304:later(W) 334:point(D) 340:things(D) 341:later(W) 343:later(W) 347:later(W) 354:things(D) 373:later(W) 389:later(W) 391:later(W) 393:later(W) 399:later(W) 420:handles(M) 420:points(M) 427:take(W) 429:carry(W) 443:take(W) 456:carry(W) 462:carry(W) 473:later(W) 549:points(M) 550:handles(M) 604:word(D) 605:word(D) 609:point(D) 609:point(D) 612:point(D) 612:point(D) 619:value(M/Q) 623:shows(M) 623:shows(M) 624:point(D) 630:take(W) 635:later(W) 650:carry(W) 652:word(D) 661:carry(W) 667:carry(W) 674:value(M/Q) 675:point(D) 676:word(D) 678:shows(M) 678:shows(M) 679:word(D) 688:word(D) 705:word(D) 706:later(W) 710:word(D) 723:value(M/Q) 725:later(W) 804:value(M/Q) 804:word(D) 805:value(M/Q) 823:later(W) | |
| workspace/ws.dart | 42 | 129:shaping(D) 148:rate(D) 149:size(D) 149:size(D) 150:now(D) 203:value(M/Q) 204:lasts(D) 218:counters(D) 230:counters(D) 235:kept(D) 263:fits(D) 264:forget(W) 265:counters(D) 269:seconds(D) 274:things(D) 276:lasts(D) 276:value(M/Q) 276:word(D) 277:word(D) 279:thingKeyMoments(D) 286:value(M/Q) 286:value(M/Q) 286:value(M/Q) 288:thingStretches(D) 289:wayAt(Q) 366:at(W) 374:at(W) 375:play(W) 438:cards(D) 439:thingEffects(D) 443:keyingNow(D) 450:work(D) 451:marks(D) 452:guides(D) 454:keying(W) 480:busy(D) 485:answerWhy(D) 503:word(D) 512:later(W) 519:word(D) 527:later(W) 534:later(W) | |
| workspace/moves.dart | 39 | 37:word(D) 37:word(D) 38:things(D) 38:word(D) 39:keyMoments(D) 41:value(M/Q) 41:value(M/Q) 42:wayOf(Q) 44:value(M/Q) 44:value(M/Q) 64:later(W) 65:later(W) 66:later(W) 67:later(W) 68:later(W) 69:later(W) 70:later(W) 71:later(W) 72:later(W) 73:later(W) 74:later(W) 75:later(W) 80:later(W) 84:later(W) 92:later(W) 93:keyMoments(D) 94:later(W) 96:later(W) 97:later(W) 98:later(W) 99:later(W) 106:later(W) 107:later(W) 108:later(W) 109:later(W) 110:later(W) 115:later(W) 120:shows(M) 120:value(M/Q) | |
| workspace/relate.dart | 29 | 13:later(W) 14:later(W) 15:later(W) 16:later(W) 17:later(W) 18:later(W) 25:word(D) 25:word(D) 26:things(D) 26:word(D) 33:later(W) 54:later(W) 61:later(W) 74:value(M/Q) 74:word(D) 75:value(M/Q) 76:word(D) 76:word(D) 84:word(D) 86:things(D) 87:value(M/Q) 87:word(D) 87:word(D) 89:value(M/Q) 89:word(D) 91:word(D) 91:word(D) 102:value(M/Q) 103:later(W) | |
| sets/studies/panel_inspector_a_space.dart | 27 | 49:add(W) 50:add(W) 61:at(W) 61:at(W) 140:add(W) 150:at(W) 213:add(W) 214:add(W) 225:at(W) 270:at(W) 270:at(W) 291:set(W) 292:set(W) 325:at(W) 325:at(W) 327:at(W) 327:at(W) 366:add(W) 367:add(W) 371:at(W) 372:add(W) 373:add(W) 384:add(W) 394:at(W) 394:at(W) 394:at(W) 394:at(W) | |
| workspace/timeline.dart | 26 | 77:drawnAt(D) 227:take(W) 249:carry(W) 264:later(W) 270:word(D) 270:word(D) 271:word(D) 277:later(W) 320:later(W) 327:later(W) 384:later(W) 386:later(W) 387:later(W) 388:later(W) 389:later(W) 394:later(W) 420:later(W) 433:value(M/Q) 434:strikes(Q) 446:word(D) 448:loudnessOver(Q) 450:loudness(Q) 497:looping(D) 497:looping(D) 497:looping(D) 545:later(W) | |
| sets/tour2/p5.dart | 23 | 63:at(W) 65:at(W) 67:at(W) 71:at(W) 72:at(W) 73:at(W) 74:at(W) 75:at(W) 89:at(W) 112:at(W) 120:at(W) 132:at(W) 137:at(W) 145:at(W) 191:at(W) 191:at(W) 192:at(W) 192:at(W) 193:at(W) 193:at(W) 194:at(W) 194:at(W) 214:at(W) | |
| workspace/desk_depth.dart | 23 | 41:point(D) 41:value(M/Q) 42:value(M/Q) 42:value(M/Q) 44:value(M/Q) 52:point(D) 53:take(W) 68:frame(M) 68:point(D) 68:point(D) 68:value(M/Q) 68:value(M/Q) 99:later(W) 245:at(W) 324:at(W) 334:at(W) 334:at(W) 335:at(W) 335:at(W) 343:at(W) 343:at(W) 348:at(W) 356:at(W) | |
| workspace/dock_make.dart | 20 | 140:later(W) 148:later(W) 157:word(D) 164:recipes(S) 169:recipes(S) 781:recipes(S) 782:later(W) 788:later(W) 790:later(W) 792:later(W) 796:later(W) 879:recipes(S) 897:key(W) 900:key(W) 940:later(W) 965:later(W) 969:later(W) 970:later(W) 1015:recipes(S) 1016:recipes(S) | |
| workspace/desk_relations.dart | 18 | 32:word(D) 33:keyMoments(D) 33:moving(D) 34:word(D) 37:shows(M) 37:value(M/Q) 51:word(D) 51:word(D) 52:shows(M) 52:value(M/Q) 124:shows(M) 124:value(M/Q) 129:word(D) 131:word(D) 131:word(D) 131:word(D) 141:point(D) 141:point(D) | |
| workspace/key_drag.dart | 14 | 15:moving(D) 16:keyMoments(D) 17:value(M/Q) 17:wayOf(Q) 21:later(W) 23:later(W) 26:later(W) 27:later(W) 29:later(W) 36:later(W) 41:moving(D) 42:keyMoments(D) 42:later(W) 46:later(W) | |
| sets/studies/panel_inspector_a.dart | 12 | 115:frame(M) 123:frame(M) 287:frame(M) 288:at(W) 327:frame(M) 591:frame(M) 646:at(W) 837:add(W) 838:add(W) 840:add(W) 842:add(W) 843:add(W) | |
| workspace/dock_fonts.dart | 12 | 45:setting(Q) 56:later(W) 56:word(D) 57:later(W) 75:whole(D) 75:word(D) 75:word(D) 78:fontTags(Q) 151:word(D) 154:word(D) 201:fontTags(Q) 250:fontTags(Q) | |
| workspace/stage.dart | 12 | 321:value(M/Q) 321:value(M/Q) 321:word(D) 323:word(D) 324:word(D) 325:word(D) 327:word(D) 328:point(D) 328:value(M/Q) 329:value(M/Q) 369:later(W) 474:word(D) | |
| sets/panels/lasso_set.dart | 11 | 151:things(D) 152:shown(D) 174:things(D) 184:shown(D) 184:things(D) 197:shown(D) 197:things(D) 218:make(W) 250:things(D) 251:shown(D) 344:things(D) | |
| sets/studies/panel_inspector_a_kit.dart | 11 | 971:at(W) 986:at(W) 1026:add(W) 1028:add(W) 1046:add(W) 1057:add(W) 1149:at(W) 1156:at(W) 1268:add(W) 1278:add(W) 1371:add(W) | |
| sets/relation_tabs/tab_draw.dart | 10 | 50:word(D) 50:word(D) 53:value(M/Q) 53:value(M/Q) 163:value(M/Q) 163:value(M/Q) 164:value(M/Q) 165:value(M/Q) 246:point(D) 246:point(D) | |
| sets/relation_tabs/tab_hands.dart | 10 | 43:word(D) 43:word(D) 46:value(M/Q) 46:value(M/Q) 134:value(M/Q) 134:value(M/Q) 138:value(M/Q) 138:value(M/Q) 205:point(D) 205:point(D) | |
| workspace/desk_blend.dart | 10 | 38:words(D) 39:word(D) 39:word(D) 40:value(M/Q) 41:value(M/Q) 43:later(W) 71:later(W) 151:take(W) 153:carry(W) 160:later(W) | |
| workspace/desk_ease_state.dart | 10 | 38:later(W) 44:later(W) 63:wayOf(Q) 67:later(W) 69:later(W) 73:later(W) 75:later(W) 77:later(W) 85:wayOf(Q) 88:wayAt(Q) | |
| sets/relation_tabs/tab_poses.dart | 9 | 59:word(D) 59:word(D) 62:value(M/Q) 62:value(M/Q) 227:point(D) 227:point(D) 235:value(M/Q) 236:value(M/Q) 333:value(M/Q) | |
| workspace/dock_effects.dart | 9 | 25:thingEffects(D) 26:label(D) 27:picture(D) 28:effects(S) 29:label(D) 30:picture(D) 32:animators(S) 113:thingEffects(D) 114:word(D) | |
| sets/group_rule_tries.dart | 7 | 20:things(D) 20:word(D) 20:word(D) 22:later(W) 51:value(M/Q) 131:value(M/Q) 131:value(M/Q) | |
| workspace/create_preview.dart | 7 | 43:things(D) 44:make(W) 46:things(D) 47:word(D) 48:value(M/Q) 55:word(D) 60:close(W) | |
| sets/relation_tabs/tab_samples.dart | 6 | 113:things(D) 113:word(D) 139:word(D) 142:value(M/Q) 144:value(M/Q) 145:value(M/Q) | |
| workspace/dock_files.dart | 6 | 102:whole(D) 102:word(D) 102:word(D) 111:kindOf(Q) 112:thumbnailAt(Q) 146:later(W) | |
| workspace/desk_align.dart | 5 | 17:seen(M) 23:seen(M) 29:later(W) 30:carry(W) 31:later(W) | |
| workspace/desk_ease.dart | 4 | 126:moving(D) 127:keyMoments(D) 140:wayNames(D) 143:wayAt(Q) | |
| workspace/desk_ease_graph.dart | 3 | 92:word(D) 93:value(M/Q) 94:word(D) | |
| workspace/top_bar.dart | 3 | 120:later(W) 459:brokenCount(D) 473:broken(D) | |
| sets/inspector/inspector_idea_layer.dart | 2 | 497:path(Q) 497:path(Q) | |
| sets/relation_bench.dart | 2 | 49:resize(W) 57:close(W) | |
| workspace/dock.dart | 2 | 118:effects(S) 118:thingEffects(D) | |
| a11y_addon.dart | 1 | 19:label(D) | |
| colour_addon.dart | 1 | 34:label(D) | |
| sets/inspector/inspector_idea_paths.dart | 1 | 1434:value(M/Q) | |
| sets/relation_tabs/tab_curve.dart | 1 | 75:word(D) | |
| sets/studies/op_09_spin.dart | 1 | 466:add(W) | |
| sets/studies/panel_browser_a.dart | 1 | 3975:take(W) | |
| sets/studies/panel_browser_b.dart | 1 | 2346:take(W) | |
| sets/studies/pop_10_macro.dart | 1 | 55:close(W) | |
| sets/studies/world_b.dart | 1 | 38:label(D) | |
| words_addon.dart | 1 | 22:label(D) | |
| workspace/desk_ease_numbers.dart | 1 | 45:wayNumbers(Q) | |
| workspace/stage_art.dart | 1 | 55:frame(M) | |
