# frontend-design を Motolii で使う(添え書き)

上流: anthropics/skills `skills/frontend-design`(NOTICE 参照)。SKILL.md は web/CSS 前提で書かれている。
Motolii の UI は Flutter(`motolii/ui/`、入口 `scripts/motolii-ui.sh {native|dev|test}`)。
UX の合否は利用者の専権。この文書は「どこが効くか」だけ。

## 効く所 / 効かない所

| SKILL.md の節 | Motolii では |
|---|---|
| 題材に根ざす・生成物の型(cream+serif、SaaS カード、ALL CAPS の eyebrow、`→`、`·` 連結) | 効く。窓の文字は英語、**値が意味の物だけ文字、あとは形で見せる**(motolii/AGENTS.md:14、docs/wiki/window.md)。ラベル文字を足す前に「値か?」を問う |
| 2 段の手順(token 案 → 素案を brief と照合 → 実装) | 効く。token 案は `motolii/ui/lib/foundation/theme.dart` の語彙で書く |
| 大胆さは 1 箇所、Chanel の 1 個外す、スクショで自己批評 | 効く。スクショは自分で撮る、判定は利用者 |
| CSS の specificity、hero、行長 80 字、Google Fonts | 効かない(窓はページでない)。Flutter では ThemeData の component theme / ThemeExtension が同じ役 |
| 非操作起因の motion は控える | 効く。Flutter の Animation を層ごとに足さない |
| 文言(能動態、同じ動詞を通す、error は原因と直し方) | 効く。英語で書く |

## theme-first(標準の仕組みが先)
「トンマナを構造で強制」は独自 lint や wrapper でなく、Flutter 本体の ThemeData component theme と
ThemeExtension で(記憶 standard-mechanism-first、2026-09-08 に 2 度止められた)。
token の正本は DTCG format v2025.10 の JSON、raw color を theme 外に書かない(docs/ui-visual-language.md:147-148)。

## DESIGN.md(Google Labs open spec、2026-04-22)
anthropics/skills issue #1008 の提案: skill が project root の DESIGN.md を読んでから作り、無ければ brief から蒸留して作る。
上流 skill 本体(この commit)は未採用。Motolii では theme.dart / DTCG JSON がその役なので、DESIGN.md は書くなら**写し**、正本にしない。
spec: https://github.com/google-labs-code/design.md(docs/spec.md、Apache 2.0、alpha)。frontmatter の項目:
- `version`(任意)`name`(必須)`description`(任意)`omitted`(任意: 意図して省く節)
- `colors`(必須)`<token>: <Color>`
- `typography`(必須)`<token>: {fontFamily, fontSize, fontWeight, lineHeight, letterSpacing, fontFeature, fontVariation}`
- `rounded`(任意)`spacing`(任意)`components`(任意)`<name>: {backgroundColor, textColor, typography, rounded, padding, size, height, width}`
本文の節順: Overview, Colors, Typography, Layout, Elevation & Depth, Shapes, Components, Do's and Don'ts。
「tokens are the normative values; the prose provides context for how to apply them」。

## 一発の UI 変更 checklist
1. DESIGN.md があれば読む。無ければ `motolii/ui/lib/foundation/theme.dart` と docs/ui-visual-language.md を読む
2. 変えるのは theme token / component theme。widget に色・寸法・文字を直書きしない。ラベルを足すなら「値が意味か」
3. `scripts/motolii-ui.sh native` → `dev`(または `reload`)、スクショを自分で撮る
4. SKILL.md の「生成物の型」表と照らして 1 個外す
5. 絵 1 枚を出して利用者の合否を待つ。自分で合格にしない
