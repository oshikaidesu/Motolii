# OP-1 / Max for Live を「体系化した資料」の調査(2026-10-02)

## 結論
- Ableton / M4L: 公式に体系化されている。中身は、振る舞い・テーマへの追従・ピクセルの規律。色の値は書かず、token 名で参照する(値はテーマの .ask ファイルにある)。
- OP-1: 画面の絵を公式に体系化した文書は無い。公式に言えるのは「4 色のエンコーダと、画面上の同じ色の要素が対応する」だけ。
- LLM 向けの DESIGN.md 集(awesome-design-md、getdesign.md): TE も Ableton も入っていない。shadcn.io に TE の DESIGN.md があるが、TE の Web サイトから取ったもので、OP-1 の画面ではない。

## 公式(出所として使える)
1. M4L Production Guidelines
   https://github.com/Ableton/maxdevtools/blob/main/m4l-production-guidelines/m4l-production-guidelines.md
   - 色は live.* の dynamic color(テーマに追従)。active と inactive で別の色。全テーマで確認する。
   - 書体は Ableton Sans。
   - 高さは固定、幅は最小。幅を増やす代わりに Fold-out や Tab を使う。
   - 座標は整数 px、1x1 グリッド、左右の余白は対称。表示は LCD スタイル。
   - 短い名前は切れない長さ。Info Text は hover で説明を出す。ボタンは Mouse Up で発火する。
2. Themes and Max for Live
   https://docs.cycling74.com/max8/vignettes/themes_and_max_for_live
   - 1 つの要素の色は「全部既定」「全部 dynamic」「全部固定」のどれか 1 つ。前景と背景で混ぜない。
   - LCD Background(常に暗い)と LCD Title(常に明るい)。
3. live.colors の token 名
   https://docs.cycling74.com/legacy/max7/refpages/live.colors
   - surface_bg / control_bg / control_fg / control_fg_on / control_fg_off / control_fg_zombie / value_arc / value_bar / led_bg / selection / contrast_frame / active_automation / inactive_automation / macro_* / midi_assignment / key_assignment など。
   - zombie は無効の状態。
4. User Interfaces in Max for Live
   https://docs.cycling74.com/userguide/m4l/live_userinterfaces/
   - デバイスの高さは 169px で固定。
5. TE 公式 OP-1 guide
   https://teenage.engineering/guides/op-1/original/layout
   - エンコーダの色 = 画面の要素の色。
   - エンベロープの例: 青 = Attack、緑 = Decay、白 = Sustain、橙 = Release。
   - 画面は 320x160。

## 公式の発言(仕様ではない)
- Kouthoofd(SFMOMA): 色は RAL の約 40 色に絞り、色ごとに一貫した意味を持たせる(橙/赤 = 録音)。形と色を結び付ける。表記は小文字。規則で自由になる。
- Möllerstedt(jdsimcoe): 小さな画面では、色がどの入力を回すかの案内になる。意図的な遊び心(toy は褒め言葉)。参照元は 80 年代の Casio、ポータスタジオ、アーケード。

## 非公式(出所を明記して使う)
- op-forums: 画面の絵は 3〜4 種の決まった線幅で描かれている。視覚言語の全体が色分けにかかっている。
- blakecrosley.com: TE の美学を CSS 値で書いたもの。作者自身が「解釈」と明記している。
- Figma Community「OP-1 Screen Graphics」: 403 で開けず、未確認。
- 一次資料として計測したもの: research/op1-measured.md(公式ガイドの SVG 79 枚)。
