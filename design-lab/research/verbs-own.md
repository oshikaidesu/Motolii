# 動詞の目録: 利用者(オーナー)は何を「する」のか(2026-10-03、調査のみ)

印: [証拠] = 原文を引用できる / [推測] = 私の読み。出所は
- `J:` = オーナー発言 `/Users/member_ottoto/.claude/projects/-Users-member-ottoto-rust-ae-Motolii/77f277f7-741f-4e9f-8e94-ddd760f7f088.jsonl`(152 件の本文を走査。時刻は JSONL の timestamp、UTC 表記のまま)
- `R/` = `design-sense-lab/research/`、`B/` = `design-sense-lab/book/lib/`
注意: J の長文(10-01 23:34、10-02 04:22/04:25、10-03 03:52)は、他の LLM の文章をオーナーが貼って返信した形跡がある(「うん、この分析かなり筋が通ってる」等)。オーナー自身の語ではなく「オーナーが採用した語」として扱う [推測]。

---

## 0. 結論の先出し(per-phenomenon、今回の主題)
オーナーの基準は J 10-02T06:04「直感的 = ユーザーがこのパラメータを触る理由と画が一致しているかであり、パラメータ自体の抽象化ではありません」。
下の表は「その値を触る理由 = 結果に対して何をしたいか」の動詞を、値ごとに並べたもの。

## 1. 現象ごとの動詞(Reason-to-touch)

Inspector が今出している値は、B1 の往復(`B/sets/workflow_set.dart`)では **Wave だけ**。他の棚品はこの試作に存在しない(後述 §3)。
他の現象の「値」は、lab 内の別の試作(`R/op1-translation.md`、`R/ideas-phenomenon.md`)か旧 Motolii の ease(`R/intents/ease.md`)にある物で、B1 の Inspector には出ていない。

| 現象 | Inspector の値(現状) | 触る理由 = 動詞 | 根拠 |
|---|---|---|---|
| Wave | Amplitude `px`(0-400)/ Frequency `Hz`(0.1-8)。`B/sets/workflow_set.dart:638,640`。式は y 方向の sin のみ `B/session.dart:137-140`。初期値 80 / 1 `session.dart:147` | Amplitude = 大きく揺らす・抑える(振れ幅を広げる/狭める)。Frequency = 速くする・ゆっくりにする(揺れの速さ)。「付ける」「外す」も動詞 | 値名と単位は[証拠]コード。「広げる/速くする」は[推測]。近い原文: J 10-03T03:52「Amplitude 42 / Frequency 3.2 を触る」(触る対象としての言及のみ。理由の語は無い)。類似 Wiggle の読み方: `R/op1-translation.md:100`「Frequency=うろつく速さ / Amplitude=うろつく円の半径」 |
| Repeat | B1 には無い。lab 内の案: Columns / Rows / Gap / Jitter `R/op1-translation.md:78` | 増やす(個数)、間隔をあける/詰める(Gap・歩幅 `R/ideas-phenomenon.md:124`)、乱す(Jitter) | 案の記述は[証拠]。「Repeater: Amount/Spread/Rotation の 3 数値ではなく、小さい図形が増殖する箱」J 10-02T04:25(貼付文)。ユーザーが実際に触る理由は[推測] |
| Random | B1 には無い。近い物: Scatter Spread / Seed `R/op1-translation.md:72`、Wiggle `:100`、Noise `:79` | 散らす(Spread)、振り直す(Seed:「ガチャッと入れ替わる」J 10-02T04:25)、ばらつきを足す/減らす(Jitter・Amount) | 「Seed なら配置がガチャッと入れ替わる」J 10-02T04:25[証拠、貼付文]。AE 側: Random 系は Mt. Mograph の Excite/Jump `R/ideas-ecosystem.md §9`。「ランダム化する」は旧 Inspector の効果メニューに候補 `R/intents/inspector.md:254` |
| Ease | B1 の Inspector には出ていない(`doc['ease']` は index で在るだけ `B/session.dart:31`)。旧 Motolii の Ease 机: 名前付き曲線 / Bezier handle / Over OK / 族ごとの数値 `R/intents/ease.md` A1-A5 | 選ぶ(名前で 1 クリック)、見比べる(hover で書かずに覗く)、曲げる(handle)、はみ出させる(overshoot)、コピーして別の key へ貼る | 「ease と、インスペクター…を拾おう」J 10-01T21:34[証拠]。「ease も線が太すぎて…」J 10-01T21:22[証拠・不満]。Ease が AE で 3 回作り直された `R/ideas-ecosystem.md §9`[証拠]。Copy/Paste Ease は Mt. Mograph `:§6` |
| Step | B1 には無い。ease の「Steps: distinct levels」`R/intents/ease.md` A2、Rotation distribution の Step(−90..90°)`R/op1-translation.md:76` | 段にする(滑らかをカクカクにする)、段の数を決める、ひと段ごとの量を決める(歩幅) | 定義語は[証拠]。ユーザーがなぜ触るかの原文は無い[推測]。Blender 側に「Ctrl = snap to steps」`R/intents/inspector.md:358` |
| Range | B1 には無い。lab 内: Stagger Range(1-100%、効く枚数の範囲)`R/op1-translation.md:73`、値の range 表示 `R/intents/inspector.md:208`、Expression Control の範囲と曲線 `R/ideas-phenomenon.md:59` | 効く範囲を絞る・広げる(どこからどこまで)、はみ出させない(clamp) | 定義は[証拠]。動詞は[推測]。「どこからどこまで効くか」`R/ideas-phenomenon.md:106` |
| Motion | B1 には無い。「Motion」は AE 拡張(Mt. Mograph "Motion"、約 88 の一括ツール)`R/ideas-ecosystem.md` 横断観察 1。lab 内に同名の棚品は見つからず | ずらす(Delay/Stagger)、並べる(Distribute/Align)、跳ねさせる(Excite/Jump)、リグを 1 手で付ける(Null/Orbit/Pin) | [証拠]=拡張の機能名。棚の「Motion」がこれを指すかは[推測](確認できず) |

読み取り(推測): 「触る理由」の動詞は、値の名前(Amplitude)ではなく結果への動詞(大きくする/速くする/散らす/ずらす/段にする/絞る)になる。ただし Wave 以外でオーナー本人が値の理由を語った原文は、上記の貼付文を除き**ほぼ無い**(証拠が薄い領域)。

---

## 2. 一般の動詞(副次)

### 2-1. オーナーの発言から(J)
| 動詞 | 原文 | 時刻 | 対象 |
|---|---|---|---|
| 探す | 「探し物や UI 操作に邪魔されず、最短で試して作品にできる」(DESIGN.md §0 ゴール、J にも複数) | 10-02T00:08 | effect・素材・Browser |
| 試す | 同上。「探す・試す・触る・つなぐ・動かす・直す・再利用する」 | 10-02T00:08 | 現象、候補 |
| 触る | 「触りたくなる」「触っていて気持ちいい」 | 10-02T00:08, 10-02T04:25 | 値、GUI |
| つなぐ | 同上(「つなぐ」)。「キーフレームをつなげるものは白線がいい」 | 10-02T00:08, 10-01T15:08 | Relation、key 同士 |
| 動かす / 直す / 再利用する | 同上 | 10-02T00:08 | 作品、結果 |
| 選ぶ | 「ここで選ぶだけになる」(帰宅後に良い部分を選ぶだけ)、「選択」を graphic choice に | 10-01T23:55, 10-01T23:34 | 案、効果、blend、easing |
| つかむ / 広げる / ドラッグ | 「Scatter、Transform、Along Path あたりを実際にドラッグ」「GUI を触る→数値が変わる→逆に数値を変える→GUI も動く」(貼付) | 10-01T23:34 | Scatter/Transform の対象 |
| 拡張・展開 | 「タイムラインの展開?クリップ、目、ロック、各パラメータ表示」 | 10-01T21:34 | Timeline 行 |
| 付ける | 「棚から現象を選ぶ → 対象に付く → Inspector で詳細値が見える」(貼付、採用) | 10-03T03:52 | 現象→対象 |
| 比べる | 「複数案を戦わせる」「トーナメント」「全ての案のスクショちょうだい」 | 10-02T04:07, 10-02T06:04 | UI 案 |
| 変える | 「サイズを 1% 刻みで」、設定の刻み | 10-01T21:19 | UI サイズ |
| 揃える(不満側) | 「dock に統合したら全部のフォントがバラバラ」 | 10-01T21:22 | UI の書体 |
| 見分ける | 「文字を消しても目的が分かる程度」 | 10-02T04:09 | GUI 部品 |
| 戻す | 「結果が予測でき戻すのが怖くない」(`R/requirements.md` A2、完了条件) | 10-02T00:08 付近 | 操作全般 |
| 隠れさせない | 「animation・relation・差分・状態が隠れない」(同) | 同 | 状態 |

### 2-2. 旧 Motolii への不満(`R/old-motolii-complaints.md`、J 10-01T21:19-21:22)
- 刻み: 「1% 置きにスクロールで決めれる」はずが 10% 置き → 動詞 = スクロールで刻む(契約の食い違い)[証拠]
- 統合: 「dock に統合させるときに全部のフォントがバラバラ」→ 揃える(様式)[証拠]
- 声量: 「ease も線が太すぎて、dock 内の一パネルとして使うには主張激しすぎ」→ 控える[証拠]

### 2-3. DESIGN.md / requirements の完了条件に出る動詞(`R/requirements.md` A2)
探す / 試す / 選択への次の操作 / 数値を覚えず直接触る / まとめて扱う / 一部だけコピー・適用 / 大量でも破綻しない / 戻す。これらはオーナーが決めた条件[証拠]。

---

## 3. 現行 prototype は何を支えているか(コードで確認)

対象: `B/session.dart`(717 行)、`B/sets/workflow_set.dart`(646 行)、`B/sets/workflow_timeline.dart`(496 行)。B1 の往復 = 棚の Wave を click/drag → 対象に付く → Inspector。

### 支えている
- 選ぶ: click / Shift 追加 `session.dart:274`、Tab・矢印 `:308`、Esc で解除 `workflow_set.dart:55-61`、範囲選択 `session.dart:497`
- 置く・動かす(Stage 上): `session.dart:314-346`(Shift で x0.1 `:332`、Esc で元へ `:348`)
- 付ける(Wave): 棚の click `workflow_set.dart:578-582`、drag は Draggable `:604-607`、drop 先 `:117`、実体 `session.dart:143-150`(1 undo)
- 外す: `session.dart:168`、`Remove` ボタン `workflow_set.dart:~635`
- 値を触る(scrub): `workflow_set.dart:638,640` → `session.dart:157`(scrub 1 回 = undo 1 段)
- 戻す: Cmd/Ctrl-Z `workflow_set.dart:100-101`、段数表示と Undo `:475-482`
- key を打つ・再生・時間: `session.dart:389`、`:367`、Home `workflow_set.dart:77`
- Timeline の掴む・伸縮・吸着・ズーム・パン: `workflow_timeline.dart:122-124`(縁)、`:153`(吸着)、`:375`(wheel)
- 複数選択の一括 Transform(違う値は「—」、scrub は同じ量ずつ): `workflow_set.dart:457-465`、`session.dart:~340-360`(group)

### 部分的
- 試す: hover の試着 `tryOn / tryOff / useTrial`(`session.dart:185-210`)は在るが、使っているのは別の試作 `B/sets/run_r1.dart:268-289`(look / anim / ease の着せ替え)だけ。B1 の棚には無く、Wave は click した瞬間に書かれる(undo 1 段)。つまり「付ける前に見る」は B1 に無い。
- 探す: 棚は 1 品(Wave)で検索・絞りなし `workflow_set.dart:555-585`。
- まとめて扱う: 複数選択で出るのは Transform のみ。Wave 欄は 1 レイヤー選択時だけ `workflow_set.dart:443-448`(`n == 1` の枝の中)。Wave を複数に一括で付ける/一括で調整する道は無い。
- 重ねる: Wave は 1 レイヤー 1 個。再度付けると何もせず選択だけ `session.dart:144-147`。
- 隠れない(状態): Inspector の先頭に Wave 欄を出す `workflow_set.dart:443-446` が、既定からの差分・変更色の表示は B1 に無い(grep 該当なし)[確認: 表示ロジックを読んだ範囲]。

### 無い(grep で該当なし)
- コピー / 貼り付け / 一部だけ適用 / 複製(`copy|paste|duplicate` が session.dart・workflow_set.dart に無い)
- 揃える・等間隔・ずらし(Stagger/Delay/Distribute)、ランダム化・seed、段(Step)、範囲(Range)、Repeat
- 現象同士の「つなぐ」(Relation は lab 内の別セット `relation_set.dart` に在るが B1 の往復には繋がっていない [推測: B1 のコードに参照なし])
- 既定へ戻す(リセット)、名前を付けて残す(再利用)、検索・Cmd-K
- 棚にある現象が Wave 1 つだけ。Repeat / Random / Ease / Step / Range / Motion は B1 に存在しない。

---

## 4. AE 拡張がやっている動詞(`R/ideas-ecosystem.md`、ほぼ [snippet] = 検索結果の抜粋)

| 動詞の群 | 拡張の例 | 対象 | 「棚の現象」が吸収するか |
|---|---|---|---|
| 探す・覗く | Animation Composer(hover 動画)、Expression Kit、KBar | preset、式 | 棚そのものが担う(別操作ではなく棚の機能) |
| 付ける(非破壊)・外す | Animation Composer(preset = 1 レイヤー)、Node Wrangler | preset、効果 | 吸収する(付ける/外すが現象の生死) |
| 時間をずらす・間をあける | Mt. Mograph Delay/Stagger/Falloff、Distribute、Align | 複数レイヤーの同一 property | 一部吸収(Stagger を現象にする案)。ただし「選択に対して並べ直す」操作は残る [推測] |
| 弾ませる・揺らす・乱す | Excite、Jump、Dynamics、Wiggle、Random | 1 つの値の時間変化 | 吸収する(Wave/Random/Spring の類。現象の本業) |
| 曲線を整える | Flow、Ease and Wizz、Mt. Mograph Easing(CSS 風 bezier) | key 間の動き | 吸収する(Ease も現象)。名前付き保存と他 key へのコピーは別操作 |
| つなぐ | Pick-whip、Expression Controls、Duik Link/Connector、Cavalry Connections、M4L Map | 値→値 | 吸収しにくい(関係の作成・解除は別操作。現象の「入力」になる形は可能 [推測]) |
| コピー(一部だけ) | Copy with Property Links、Copy/Paste Ease/Color、Layers Pro、Blender Copy Attributes | ease・色・key・効果 | 吸収しない(クリップボード的な別操作) |
| 一括編集・置換 | Massive Editor、Find & Replace Keyframes、Mt. Mograph Grab/Trash | 多数の同種 property | 吸収しない(選択に対する操作) |
| 整える・名前 | Sortcery、layerNamer、Mt. Mograph Sort/Rename/Arrange/Reverse | レイヤー順・名前 | 吸収しない(組織化、現象と無関係) |
| 保存・戻す | Snapshot/Re-Key、Layer Library、Pseudo Effects(preset 化) | 状態、構成 | 一部(「現象をまとめて棚に戻す」案)。スナップショットは別操作 |
| 束ねる(1 取っ手で多数) | Duik Bassel、Rubberhose、Ableton Macro | 多 property | 一部吸収(現象の 1 つの GUI に潰す案 `R/ideas-phenomenon.md` Glow)。リグは別 |
| 焼く(bake) | Re-Key、Duik の bake 等 [推測、ecosystem に直接記述なし] | 式→key | 吸収しない [推測] |

`R/ideas-ecosystem.md` の横断観察 1 [推測、同文書]: 強制された拡張の多くは「選択に 1 クリックで効く操作」で、「新しい機能でなく動詞が足りなかった」。
→ 棚の現象で吸収できるのは「付けた対象の時間変化(揺らす・弾ませる・曲げる)」の群まで。「選択に対する一括操作(ずらす・揃える・コピー・置換・命名)」は、現象を足しても残る。

---

## 5. 痛みの証拠が強い動詞 上位 5(証拠/推測を印)

1. **探す**([証拠] 最強): ゴール文の第一語(J 10-02T00:08「探し物や UI 操作に邪魔されず」)。AE の Effects & Presets 不満 `R/ideas-ecosystem.md §1`(30+ フォルダで不具合)、Animation Composer が hover で解決。ただし B1 の棚は 1 品で「探す」を試していない [証拠: コード]。
2. **試す(付ける前に見る)**([証拠]): 同ゴール文。旧 Ease の「書かずに hover で覗く」を良い契約として採用 `R/intents/ease.md` A1。Animation Composer の hover 動画、Blender Pose Library の「apply に mode 往復」不満 `§2`。B1 には試着が無い(部分的、§3)。
3. **ずらす / 揃える / 一括で入れる(まとめて扱う)**([証拠]): 完了条件「まとめて扱う」(R/requirements.md A2)。AE 側の拡張 Motion は約 88 個の一括ツール `R/ideas-ecosystem.md`。`R/problems.md:76` I12「多数 layer に同じ変更を、ずらして/相対で/一括で」= 熟練・素材多で「大」。B1 では Wave を一括で付けも調整もできない [証拠: コード]。
4. **一部だけコピー・適用する**([証拠]): 完了条件に明記。AE は「見た目だけ他へ」が痛み(forum)、Mt. Mograph の Copy/Paste Ease 等 `R/ideas-ecosystem.md §6`。横断観察 5「ease/色/key を 1 つだけ運ぶ穴」。B1 には無い。
5. **戻す(怖くない)**([証拠・満足側]): 完了条件「結果が予測でき戻すのが怖くない」。B1 は 1 gesture = 1 undo を満たす(`session.dart:157-166`)ので、痛みはむしろ「既定へ戻す・状態を残す」の無さ [推測]。
   次点: **触って気持ちいい**(J 10-02T00:08、10-03T01:44「画がおもしろい。しかし面白いだけです」= 触る理由と画の一致が未達、[証拠])。

限界: 上記のオーナー原文は 152 件のうち約 100 件を走査して抽出。画像のみ・短い返答は除外。AE 拡張の記述は多くが検索抜粋で、ページ本文は確認していない。
