# M3 U0e-2 reference fixture契約

作成日: 2026-07-21
状態: **決定 / U0e-2R完了 / U0e-2待ち**

## 1. 目的と直列分割

U0e-2は、G0-6Hで人間が同じ対象を比較できるよう、既決の5 reference
screenを一つの固定fixtureから再現し、normal / lightness / grayscale /
protanopia / deuteranopia / tritanopia画像を決定的に生成する。

現行React比較prototypeは接続済みworktreeの
`codex/m3-mock-components`にあり、mainには`docs/mocks-ui/`がまだ無い。
また、U0e-1完了後のmainと同branchのmerge-treeは10文書で衝突する。
ARCHIVED `docs/mocks/`へ新しい判断やgoldenを追加せず、巨大な衝突解消と
5画面実装を混ぜないため、U0e-2の前に次の再結合ticketを置く。

```text
U0e-1 DONE
  → U0e-2R React comparison baseline再結合
  → U0e-2 5画面fixtureと派生画像
  → G0-6H HUMAN STOP
```

U0e-2Rは製品能力を増やす枝番ではなく、既存の現行比較正本をmainへ戻す
再結合gateである。U0e-2と同時commitにしない。

## 2. authorityと固定証拠

- 5画面、機械審判、人間審判の意味は
  [UI視覚言語「G0-6の審判」](../ui-visual-language.md#g0-6の審判)を正本とする
- React/legacyの所有境界は[M3 UI参照地図](../ui-reference-map.md)と
  `docs/mocks-ui/README.md`を正本とし、通常入場と`#catalog`はReact候補、
  legacyは`#archive/*`のparity参照だけとする
- U0e-2の比較元は`origin/codex/m3-mock-components`の固定commit
  `eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0`とする。このcommitは
  test-only修復PR #264を含み、`npm run build`と`npm run test:visual`
  43件が成功している
- PR #184と`docs/mocks/`の具体色、px、radius、旧Slint出力は
  製品tokenの根拠にしない
- CVD派生画像はMachado / Oliveira / Fernandes (2009)
  “A Physiologically-based Model for Simulation of Color Vision Deficiency”
  (DOI `10.1109/TVCG.2009.113`)のseverity 1.0行列を比較simulationとして使う。
  simulationは人間の知覚そのものでも自動合否でもない

## 3. U0e-2R: React baseline再結合

### 3.1 入力と変更範囲

最新`origin/main`へ§2の固定commitをmergeし、merge-treeが列挙する次の衝突だけを
解消する。

- `AGENTS.md`
- `docs/README.md`
- `docs/mocks/README.md`
- `docs/reviews/2026-07-19-am-keyframe-graph-observation.md`
- `docs/reviews/2026-07-19-m3-interaction-prototype-decision-ledger.md`
- `docs/reviews/evidence/am-keyframe-graph/README.md`
- `docs/specs/M3-ui-integration.md`
- `docs/ui-reference-map.md`
- `docs/ui-score-model.md`
- `docs/ui-visual-language.md`

再結合後のpathは次の三集合で裁定する。

1. `docs/mocks-ui/**`: 固定commitに存在する全pathを固定commitとbyte一致させる。
   PR #264のtest-only修復は固定commit自身に含まれるため、追加例外は無い
2. 固定commitが変更せずmainだけが変更したpath: 再結合直前のmainとbyte一致させる
3. 両側が変更した上記10衝突file: 次表どおり意味を統合する。上記以外の
   非衝突pathはGitの通常の3-way結果を採り、どちらか一方のtree全体を
   authorityとして上書きしない

| 衝突file | 必須裁定 |
|---|---|
| `AGENTS.md` | mainの後発workflow・停止線と、Reactを現行比較入口、HTMLをarchiveとする所有境界を両方残す |
| `docs/README.md` | mainの現行読順・完了状態を維持し、`docs/mocks-ui/`を現行比較prototype入口として追加する |
| `docs/mocks/README.md` | ARCHIVED・新規変更禁止を現行状態とし、branch側のlegacy来歴は過去証拠として残す |
| `docs/reviews/2026-07-19-am-keyframe-graph-observation.md` | 観察と非目標を和集合にし、同じ状態の後発表記はmainを採る |
| `docs/reviews/2026-07-19-m3-interaction-prototype-decision-ledger.md` | P-IDを欠落させず統合し、同一P-IDの状態・後続はmainの後発記録を採る |
| `docs/reviews/evidence/am-keyframe-graph/README.md` | 証拠一覧を和集合にし、同一証拠のpath・状態はmainの後発記録を採る |
| `docs/specs/M3-ui-integration.md` | mainの完了済みU状態、`U0e-1→U0e-2R→U0e-2→G0-6H`の直列順、egui決定を維持し、React比較証拠だけを接続する。Slintへ戻さない |
| `docs/ui-reference-map.md` | Reactを通常入口、HTMLを`#archive/*`限定とし、mainの後発決定・停止線を残す |
| `docs/ui-score-model.md` | mainのscore決定を維持し、branchの比較証拠参照を追加する。scoreや重みを新設しない |
| `docs/ui-visual-language.md` | mainのG0-6/U0e決定とbranchのReact証拠を統合し、mock具体値を製品authorityへ昇格しない |

衝突fileは「片側を丸ごと採用」せず、mainの後発決定・完了状態とReact branchの
比較台帳・現行入口を両方保持し、同じ意味の重複だけを統合する。新しいUI判断、
製品値、Document意味、公開API、永続形式を衝突解消で発明しない。

### 3.2 再結合審判

1. `git diff <fixed-commit> -- docs/mocks-ui`が空で、PR #264を含むReact比較実装を
   byte一致で再結合している
2. 通常入場、`#plugin-browser-candidate`、`#graph-view-candidate`、`#catalog`は
   React候補、`#archive/*`だけがlegacyである
3. `component-map.json`がparseでき、全ID一意、source fileが存在する
4. `npm ci`、`npm run build`、`npm run test:visual` 43件を通す
5. `cargo fmt --all -- --check`、`./scripts/check-docs.sh`、
   `./scripts/check-ui-toolkit-deps.sh`、
   `cargo clippy --workspace --all-targets -- -D warnings`、
   `cargo test --workspace`を通す
6. Grok反対側レビューがP0/P1=0で、固定commit外の意味追加と
   legacy側への新判断逆実装が無い

## 4. U0e-2固定fixture

### 4.1 正本の分離

U0e-2は次の三層を別fileにし、React stateやDOMをDocument正本にしない。

1. `reference-document.json`: 現行`motolii-doc` codecで読める固定Document。
   video / audio / shape / text / group、3つの非隣接Effect Useと共有Effect
   Definitionを既存schemaだけで表す
2. `reference-scenes.json`: selection、focus、hover、disabled、warning、
   Camera / Hand、通常drag / Relative drag、fold状態等の比較専用Transient投影。
   Document field、Undo、保存形式ではない
3. `reference-candidate-tokens.json`: DTCG 2025.10でU0e-1 generatorの
   4型subsetを通る比較候補値。`candidate-` IDを使い、Motolii Dark/Lightの
   製品値・既定値・fallbackを名乗らない

Rust testは`reference-document.json`を実codecでdecode/validateし、
前後のbyte不変を検査する。Reactはfixtureを表示するだけで、欠落したDocument意味を
scene側や表示文字から逆算しない。

### 4.2 5 screenの閉集合

screen IDと必須状態を次に固定する。

| ID | 必須表示 |
|---|---|
| `empty-browser` | empty project、asset browser、transport、context説明口 |
| `mixed-timeline` | video/audio/shape/text/group、選択、mute、keyframe、bake/cache |
| `parameter-easing` | 選択項目parameter panel、keyframe/easing popup、focus、warning、disabled |
| `stage-frame-tools` | Stage、Output Frame、frame内外object、半透明scrim、選択、CameraとHand |
| `shared-effect-relative` | 非隣接3 layerの同じEffect Definition use、異なるstack位置、connection gutter、from/out・use/in、fold stub+件数、通常dragとRelative HUD |

5画面は同じdocument/scene sourceをscreen IDで投影し、画面ごとの複製fixtureや
別tokenを持たない。未実装製品機能の存在や操作成功を主張せず、
G0-6Hの比較対象として明示する。

## 5. captureと派生画像

capture正本は`@playwright/test 1.61.1`のbundled Chromium
`149.0.7827.55`（revision `1228`）、headless shell、viewport `1440x900`、
device scale 1、locale `en-US`、timezone `UTC`、color scheme dark、
reduced motion、animation/transition/caret停止、同梱fixture以外のnetwork
0件とする。package指定はcaretを外して完全固定し、lockfileと実行時browser
version/revisionが一致しなければ生成・検査とも拒否する。

OS font fallback差をgoldenへ混ぜないため、`@fontsource/inter 5.3.0`
（OFL-1.1、package integrity
`sha512-RofMylZmjlJEfELXeNHFWBRcSs75rGU/6bV2S2jfnvv/3rPXPGe0LgUJTklcHZ9lM4OZmAVFhcJPnACfb91A3g==`）
から次の2 fileだけを比較fixtureへ同梱し、`@font-face`の
`MotoliiReferenceInter`をinterface/technical両roleへ明示する。

| file | SHA-256 |
|---|---|
| `inter-latin-400-normal.woff2` | `8909904ab6c872eb994093482a88a28eca2cd95912d7b6fecd72103b0dc07edc` |
| `inter-latin-600-normal.woff2` | `f9a06e79cd3a2a20951c0f0e28f66dd0e6d3fda73911d640a2125c8fcb78f21a` |

同packageの`LICENSE`も同梱し、そのSHA-256
`3b0a5fca3d17942cde889069889dedbbbd075e9b599968c82a95f4d944e9b345`
を検査する。capture前に`document.fonts.ready`と両weightのload成功を確認し、
fallbackした場合は拒否する。fontの製品採用はG0-6Hで決めないし、U0e-2から
製品tokenへ昇格させない。

各screenでnormal PNGを1枚captureし、同じnormal pixelから次を後処理する。

- `lightness`: sRGBをlinear化し、relative luminance
  `Y = 0.2126R + 0.7152G + 0.0722B`をCIE L*へ写し、後述のbyteへ符号化した
  単channel画像
- `grayscale`: 同じlinear relative luminanceをsRGB grayへ戻した画像
- `protanopia` / `deuteranopia` / `tritanopia`: linear sRGBへMachado
  severity 1.0の固定3×3行列を適用する

RGBA byteの各sRGB channelを`c8 / 255`とし、JavaScript `Number`
（IEEE 754 binary64）で次を計算する。

```text
linear(c) = c / 12.92                         if c <= 0.04045
            ((c + 0.055) / 1.055) ^ 2.4      otherwise

srgb(c)   = 12.92 * c                         if c <= 0.0031308
            1.055 * c ^ (1 / 2.4) - 0.055    otherwise
```

grayscaleはlinear RGBから求めた`Y`を`srgb(Y)`へ戻す。lightnessは
`δ = 6 / 29`として次を使い、`q = clamp(L* / 100, 0, 1)`を表示byteへ写す。

```text
f(Y) = cbrt(Y)                    if Y > δ^3
       Y / (3 * δ^2) + 4 / 29    otherwise
L*   = 116 * f(Y) - 16
```

CVDは次のrow-major行列をlinear column vector `[R, G, B]`へ左から掛ける。

```text
protanopia =
  [ 0.152286,  1.052583, -0.204868
    0.114503,  0.786281,  0.099216
   -0.003882, -0.048116,  1.051998 ]

deuteranopia =
  [ 0.367322,  0.860646, -0.227968
    0.280085,  0.672501,  0.047413
   -0.011820,  0.042940,  0.968881 ]

tritanopia =
  [ 1.255528, -0.076749, -0.178779
   -0.078411,  0.930809,  0.147602
    0.004733,  0.691367,  0.303900 ]
```

各出力channelは有限値を確認し、CVDはlinear値を`[0, 1]`へclampしてから
sRGB encodeする。grayscale/CVDのencoded値とlightnessの`q`はいずれも
`Math.round(clamp(value, 0, 1) * 255)`でbyte化する（非負値なので0.5 tieは
+∞方向）。lightness/grayscaleは同じbyteをRGB全channelへ入れ、alpha byteは
全variantで入力値をそのまま保つ。

全演算はRGBA8 normal PNGを入力とし、派生画像から別派生を作らない。
上記formula、行列、境界値、roundingはsourceとtest vectorで固定する。
入力PNG、capture条件、変換version、全出力SHA-256をmanifestへ記録する。

`generate-reference`だけが画像・manifestを書き、`check-reference`は同じ期待byteと
commit済み30 PNG（5画面×6種）・manifestをread-only比較する。
Playwright screenshotのbyte metadata差ではなくdecode済みRGBAと寸法を意味比較し、
commit済みPNGのSHAは手編集検出に使う。欠落、余分な画像、1 pixel差、
screen/variant順序差を拒否する。

## 6. 自動審判

1. 5 screen ID、同一Document hash、同一scene source hash、同一token source hashを
   manifestで確認する
2. 5画面それぞれが§4.2の必須semantic IDを重複・欠落なく持つ。
   この一覧は必要条件であり、DOM全体の完全な意味記述や見た目の合否を主張しない
3. normal 5枚と各派生25枚を同じnormal sourceから再生成できる
4. sRGB境界値、透明pixel、既知3色test vectorでlightness/grayscale/CVD変換を
   byte固定し、NaN/Inf、寸法違い、未知variantを拒否する
5. `check-reference`が手編集、欠落、余分fileを拒否し、実行前後の
   repository status、tracked/untracked path集合、全file byteが不変
6. normal同士のpixel差が0でないscreen ID取り違えを検出し、派生画像がnormalと
   偶然同一ならそのscreen/variantを診断する。これを見た目の合否には使わない
7. raw color/spacing/icon、contrast、component stateの製品審判はU0e-3前の
   G0-6H入力としてreportするだけで、自動修正・自動採択しない
8. React archive file、製品`motolii-ui` component、Document/journal/plugin/render
   公開契約を変更しない
9. U0e-2Rの全審判に加え、fixture decode test、reference generate/check testを通す

## 7. G0-6H handoff

U0e-2完了時はagentが見た目をACCEPTしない。次を揃えて停止する。

- fixed commit、capture環境、30画像、manifest
- 5画面×人間審判項目の未記入checklist
- 判定者、実施日、表示環境、5秒課題、採否理由、修正要求を記すdecision template
- 自動reportと、人間が判定すべき階層・識別・馴染み・過剰装飾を明確に分離

G0-6Hの記録がmainへ入り、具体token値が採択されるまでU0e-3、
U2c-3、U2c-5へ進まない。

## 8. 非目標

- Motolii Dark/Light/custom themeの製品値、既定値、選択保存、fallback
- 製品egui Style、component state、icon asset、font採用
- contrast違反やCVD衝突をagentが視覚判断して自動修正すること
- React props/state/CSS pxをRust公開API、Document、User settingsへ移すこと
- legacy HTMLへ新しいscreen、判断、goldenを追加すること
- Timeline候補、Browser候補、Graph候補の操作意味を改訂すること

## 9. STOP条件

- 固定React baseline全体を監査せずU0e-2と同時squashしたくなった
- U0e-2R着手時の最新mainと固定commitのmerge-tree衝突集合が§3.1の
  10 fileと一致しない。新しい衝突をその場で裁定せず、本契約を改訂する
- `docs/mocks-ui/**`を固定commitとbyte一致にできない、mainだけが変更したpathを
  巻き戻す必要がある、または三集合のどれにも分類できないpathが現れた
- 固定Playwright/Chromium revision、font/license/hashを一致させられず、
  version・font・期待画像のいずれかを動かして通したくなった
- 衝突解消でmainの後発決定またはReactの現行入口を丸ごと捨てる必要がある
- 5画面を別Document、別token、別viewportで作りたくなった
- 未実装のDocument意味、Effect/Tool状態、製品theme値をfixture都合で発明したくなった
- archive HTML、golden期待値、製品componentを変更しないと通らない
- CVD画像やpixel差を人間審判の代わりに合格根拠へしたくなった
- G0-6Hをagentが代行する、または未記入checklistのままU0e-3へ進みたくなった
