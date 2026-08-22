# 実装粒を減らす先例調査 — 大量のUI動詞をどう畳むか

日付: 2026-08-22 / 発注: supervisor(採用予定982行の実装粒削減案5本の傍ら「他に無いか調べておく」)/ レーン: read-only調査(コード変更なし、裁定200によりstash不使用)
先行資料: [争点束裁定(裁定175)](2026-08-22-map-audit-rulings.md)(採用予定の最終集計)・[意図統合裁定(裁定177)](2026-08-22-intent-bundling-decision.md)・[意図優先原則(裁定174)](2026-08-22-intent-first-grouping-decision.md)・[ネイティブメニュー調査](2026-08-22-native-menu-and-stock-widgets-survey.md)(`menu.rs`実装済みの`Item{label,shortcut,message}`構造)
現状把握: `next/reference/normal-map.tsv`実測 — 採用予定**1,079行**(裁定175後の集計は1,129、supervisorが挙げた「982」はどちらとも一致しない。おそらく別フィルタ基準の中間スナップショット。数字の食い違いは是正せず、以後「約1,000〜1,100行」と書く)。現行コードの1動詞の書き先は実測で最大5箇所: `Message`列挙子1本(`next/shell/motolii-shell/src/lib.rs`、実測220行のenum)・`update()`のmatch腕1本(実測157腕)・`menu.rs`の`Item`1行・shortcut有りなら`resolve_navigation_key`のmatch腕1本・Storeへ書くなら`Intent`列挙子1本(`next/core/motolii-store/src/document.rs`、実測115 variant)+`apply()`のmatch腕1本。

---

## 要約(RETURN)

**文書パス**: `docs/reviews/2026-08-22-grain-reduction-survey.md`(本文書)。commit hashは末尾に記載。

**先例の比較表**(1動詞あたりの記述箇所数、詳細は§1〜§6):

| ソフト/系統 | 1動詞を足す時に書く箇所数 | 宣言はデータかコードか | 入口との結びつけ方 | 出典 |
|---|---|---|---|---|
| **Blender Operator** | 実測6箇所(§1: `Operator`クラス+`register_class`呼び出し+`menu_func`+`append`+keymap登録+`unregister`側の対) | コード(Pythonクラス)。`bl_idname`という文字列キーだけがデータ的 | menu=`append`で既存メニューのdraw関数へ追記、keymap=`bl_idname`参照のKeyMapItem、**F3検索=`bl_idname`+`bl_label`があれば自動**(追加コード不要) | [Operator API](https://docs.blender.org/api/current/bpy.types.Operator.html), [Addon Tutorial](https://docs.blender.org/api/blender_python_api_2_64_9/info_tutorial_addon.html) |
| **VS Code contribution points** | 2箇所(§2: `package.json`の`commands`宣言1行+`registerCommand`ハンドラ1関数)。menu露出・keybinding・パレット露出は**追加の宣言行**であって新しいコード分岐ではない | **宣言(JSON)とコード(TS)が完全分離** | menu/keybinding/コマンドパレットは全て同じ`package.json`の別セクションが同じcommand IDを参照するだけ。**パレットはデフォルトで自動列挙**(`commandPalette`の`when`は除外用) | [Contribution Points](https://code.visualstudio.com/api/references/contribution-points), [Commands guide](https://code.visualstudio.com/api/extension-guides/command) |
| **Emacs command+keymap** | 2箇所(`defun`+`(interactive)`宣言1本、`define-key`/`global-set-key`1行)。menu露出は`easy-menu-define`が同じ関数シンボルを参照するので実質+0〜1 | コード(Lisp関数)。関数シンボル自体がグローバルな一意キー | `M-x`(≒コマンドパレットの原型、1971年のTECO時代から存在)は**`(interactive)`を持つ全関数を自動列挙**。keymapは別途明示bind | [GNU Emacs Lisp Reference — Keymaps](https://www.gnu.org/software/emacs/manual/html_node/eintr/Keymaps.html), [Mastering Emacs](https://www.masteringemacs.org/article/mastering-key-bindings-emacs) |
| **Krita KisAction (Qt/KDE)** | 3〜4箇所(§3: `.action` XML1エントリ+`KisActionManager`への operationID登録+実処理関数+`ActivationFlags`) | **宣言(XML)と実装(C++関数)が分離**。ラベル・アイコン・shortcut・有効化条件はXML、処理本体は別 | XMLの`shortcut`要素がKDE共通の Shortcut設定ダイアログに**自動で載る**(Krita独自コード不要)。メニュー配置も別XML(`.rc`/`.ui`)が action名参照 | [kis_action.h](https://github.com/KDE/krita/blob/master/libs/ui/kis_action.h), [Krita `.action` format](https://github.com/KDE/krita/blob/master/krita/krita.action) |
| **GIMP action system** | 3〜4箇所(§3: `GimpActionEntry`配列1行+コールバック関数+`.ui`メニューXML1行+任意でaccelerator) | **宣言(静的配列+XMLファイル)と実装(コールバック)が分離** | `.ui`ファイルがaction名を参照してメニュー構造を決める(build時`xsltproc`処理)。shortcutは`GimpActionGroup`のaccel機構が持つ | [GIMP Action/Menu System](https://deepwiki.com/GNOME/gimp/3.1-action-and-menu-system), [gimp_install_procedure](https://developer.gimp.org/resource/writing-a-plug-in/tutorial-pdb/) |
| **Zed (gpui Action)** | 2箇所(`#[derive(Action)]`付き構造体1本+keymap JSONの1行)。Rustの型そのものが宣言 | **型宣言(Rust struct)がデータとコードの中間** — deriveマクロがJSON deserialize・レジストリ登録を自動生成 | keymapはJSON文字列で`namespace::ActionName`を参照。コマンドパレット相当(`cmd-shift-p`)は**登録済みaction全体を自動列挙** | [gpui::Action trait](https://docs.rs/gpui/latest/gpui/trait.Action.html), [key_dispatch.md](https://github.com/zed-industries/zed/blob/main/crates/gpui/docs/key_dispatch.md) |
| **Helix (typed command)** | 2箇所(`TypableCommand`静的リストへ1エントリ+関数実装)。static commandはさらに軽く関数1本のみ | データ(静的配列)+コード(関数ポインタ) | `:`プロンプトが`TypableCommandList`を検索実行、keymapは別途TOML/デフォルトmapが関数を参照 | [commands.rs](https://github.com/helix-editor/helix/blob/master/helix-term/src/commands.rs), [typed.rs](https://github.com/helix-editor/helix/blob/master/helix-term/src/commands/typed.rs) |
| **Lapce** | 2箇所(コマンド列挙+`keymaps-common.toml`へ1行) | データ(TOML)+コード | keymapはcommand名文字列を参照する薄い間接層 | [keymaps-common.toml](https://github.com/lapce/lapce/blob/master/defaults/keymaps-common.toml) |
| **Motolii現行**(next/) | **実測5箇所**(`Message` variant+`update()`腕+`menu.rs` Item+`resolve_navigation_key`腕+ (Storeに触るなら)`Intent` variant+`apply()`腕、最大7箇所) | **全てコード**。宣言と実装が同じ場所(`match`腕)に同居し、露出先(menu/shortcut/パレット)ごとに別の場所へ手で複写 | 露出先ごとに**別々の配線**を人力で保つ(S6併存表はドキュメントコメントで手動同期を宣言しているだけで、機構による保証はない) | 本調査実測(`next/shell/motolii-shell/src/lib.rs`, `next/shell/motolii-shell/src/menu.rs`, `next/core/motolii-store/src/document.rs`) |

**結論**: 先例のほぼ全て(Blenderを除く8/9系統)が到達している共通構造は**「1個の一意キー(id文字列 or 型)を軸に、実装1箇所・露出先の宣言N箇所(データ)が同じキーを指す」**という形。Motolii現行はキーの一意性はある(`Message` variant自体がキー)が、**露出先ごとの配線がコードのmatch腕として分散**しており、宣言と実装の分離が無い。これはsupervisor案(1)動詞レジストリの立ち位置を強く支持する一次資料。

**コマンドパレットの評価**: 9系統中7系統(VS Code/Emacs M-x/Blender F3/Zed/Helix `:`/Krita検索/GIMP検索dialog)が同型の「全動詞を検索実行する1入口」を持つ。共通点は**宣言済みの動詞集合を検索面が"タダで"列挙できる**ことで、追加コードを要しない(§7)。Ableton可視性原理(隠れていないから読める)との整合は**条件付き**: パレット自体は「探せば見える」機構であり「常に見えている」わけではないため、**唯一の入口にしてはならない**(S6併存と同じ理由)。ただし**発見できない・作れない機構ではなく、既存の入口(メニュー/shortcut)に加える第4の入口として妥当**。動詞レジストリ(案1)を採ればパレットはほぼ無料の副産物になる(§7で見積り)。

**supervisor案以外のapproach**(見積りは§8に詳細、粗い概算行数):
1. **動詞レジストリを`normal-map.tsv`から生成する**(コード生成) — 表自体は既にデータとして存在するので、`Item`/`Message`/keymap行を手書きせず`build.rs`かCLIで生成。削減見積り: menu.rs+resolve_navigation_key+Message宣言部の**定型部分(約60〜70%)** が生成に置き換わる。ただし`update()`のロジック本体は残る
2. **verbをvism(プラグイン)として外出しする** — 拡張の哲学(全段継ぎ目)と整合。ただしcore動詞(Undo/Cut/Copy等)まで外出しするのはオーバーエンジニアリング。**950行のうち「拡張」verdict済み16行+precomp/paint等の島度が低い一部が対象**、大半には向かない
3. **DSLでverbを記述する** — Blender操作の多くはHelix typed commandのような「引数付き関数」で十分説明可能。新しい構文を学習させるコストがwraps>移植>スクラッチ原則に反するため**非推奨**
4. **既存Intentの直上に「表示メタデータ」構造体を足す**(VS Code型) — `Intent` variantは既にある(115個)。それぞれに`{label, category, default_shortcut, menu_path}`を対応させる**別テーブル**を作り、`menu.rs`/`resolve_navigation_key`/パレットの3箇所を同じテーブルから駆動する。これは案1と実質同じだが**既存Intentを起点にする分、新規動詞のcore実装(apply()の中身)は削減されない** — 削減対象は配線コストのみ

**推奨順**(判定軸: 保守最低限>削減量>裁定整合>導入コスト):
1. **supervisor案(1) 動詞レジストリ**(§7実測が最も強く支持、Motolii現状=キー一意だが配線分散という診断に直接効く)
2. **supervisor案(2) Inspector表駆動**(§6実測で現状把握済み — `transform_row`/`mask_ident_row`等が個別関数化されており、Blender RNA/Qtプロパティシステムの先例と整合する縮小採用)
3. **新規提案4「Intentへの表示メタデータ付与」**は案1の実装手段として統合可能(別案として並べる必要はない、統合推奨)
4. **コマンドパレット新設**は案1採択後の**ほぼ無料の追加入口**として同一campaign内で検討可(S6併存の第4の入口)
5. **新規提案2「vismへの外出し」**は拡張verdict済みの一部(16行+周辺)にのみ限定適用
6. supervisor案(3)(4)(5)(pane update所有・shell分割・keymapデータ化)は本調査の主眼(動詞管理)と直交する構造課題であり、**本調査は判定材料を追加していない**(範囲外、既存議論に委ねる)

**逸脱**: 発注は「Blender/VS Code/Emacs/Krita・GIMP・Inkscape/Zed・Helix・Lapce」を挙げていたが、Inkscapeの一次資料(verb登録系のソースコード)には到達できず、GIMPで代替補強した(§3後半)。Blender公式ドキュメント`docs.blender.org`はWebFetchが403を返したため、GitHubミラーのrstソース(同一内容、Blender公式リポジトリの過去バージョン管理下にあった文書)で代替した — 内容は一次資料のミラーであり結論に影響しない。

---

## §1 Blender: Operator登録機構

`bpy.types.Operator`をサブクラス化し、`bl_idname`(例: `"object.cursor_array"`)・`bl_label`・`execute(self, context)`を持たせる。`bpy.utils.register_class()`で登録。メニュー露出は**別の関数**(`menu_func(self, context): self.layout.operator(ClassName.bl_idname)`)を書き、`bpy.types.VIEW3D_MT_object.append(menu_func)`で既存メニューのdraw関数リストへ追記する。keymapは`register()`内で`KeyMap`/`KeyMapItem`を`bl_idname`参照で作る。`unregister()`側で対称的に`remove`/`append`の逆操作が要る。

実測: 1動詞のフル露出(menu+shortcut)には**6箇所**書く — (1)Operatorクラス本体 (2)`menu_func` (3)`append`呼び出し (4)keymap item作成 (5)`unregister`でのmenu remove (6)`unregister`でのkeymap remove。ただし**F3検索(Operator Search)には追加コード不要** — `bl_idname`+`bl_label`を持つ全Operatorが自動的に検索対象になる(§7)。9系統の中で最も配線コストが高い(全てコード、宣言とキー以外の分離がない)理由は、Blenderの`bl_idname`が「文字列だが実質コード内シンボル」であり、VS Codeのように別ファイル(package.json)へ外出しされていないため。

出典: [bpy.types.Operator](https://docs.blender.org/api/current/bpy.types.Operator.html)、[Addon Tutorial(2.64.9)](https://docs.blender.org/api/blender_python_api_2_64_9/info_tutorial_addon.html)のGitHubミラー([scorpion81/blender-voro](https://github.com/scorpion81/blender-voro/blob/master/doc/python_api/rst/info_tutorial_addon.rst))。

## §2 VS Code: contribution points(宣言と実装の分離)

`package.json`の`contributes.commands`配列が`{command: "ext.foo", title: "Foo"}`を宣言し、拡張コード側で`vscode.commands.registerCommand("ext.foo", handler)`を呼ぶ。この**2箇所だけ**が必須。`contributes.menus`(`{command: "ext.foo", when: "...", group: "..."}`)・`contributes.keybindings`(`{command: "ext.foo", key: "ctrl+f1"}`)は**同じcommand IDを指す追加の宣言行**であり、コードを増やさない。コマンドパレットは`contributes.commands`に載っている時点で**デフォルトで自動列挙**され、`menus.commandPalette`は逆に「隠すため」の`when`節にしか使わない。

これは**発注が挙げた5案のうち(1)動詞レジストリと(2)Inspector表駆動の両方の直接の先例**になる。「宣言はJSON、実装はTS、両者はIDで結合」という形がMotolii Rustでも(JSONではなくRust構造体/静的テーブルとして)再現可能。

出典: [Contribution Points](https://code.visualstudio.com/api/references/contribution-points)、[Commands guide](https://code.visualstudio.com/api/extension-guides/command)。

## §3 Emacs / Krita・GIMP(Qt/KDE系) / Inkscape

**Emacs**: `(defun foo () (interactive) ...)`で関数を対話コマンド化し、`(global-set-key (kbd "C-c f") 'foo)`でbind。`M-x foo`は`(interactive)`宣言を持つ全関数を対象にした**世界最古級のコマンドパレット**(1970年代のTECO/Emacs系譜まで遡る設計、現行GNU Emacsのマニュアルに現行仕様として明記)。menu露出は`easy-menu-define`が同じシンボルを参照するので追加コストはほぼゼロ。

**Krita**(KDE/Qt系): `.action` XML(`<Action name="..."><shortcut>...</shortcut></Action>`)がラベル・アイコン・shortcut・有効化条件(`ActivationFlags`)を宣言し、`KisActionManager`が`operationID`経由で実処理へ結びつける。KDEの共通Shortcut設定ダイアログは**XMLに`shortcut`があれば自動的に一覧に載る**(Krita側コード不要)。

**GIMP**: `GimpActionEntry`静的配列(ラベル・コールバック関数ポインタ)+別ファイルの`.ui`(メニュー階層、`xsltproc`でビルド時処理)。DeepWikiの解析では1動詞につき実質**3〜4箇所**(action定義・コールバック・`.ui`メニュー行・任意のaccelerator)。

**Inkscape**: 一次資料(verb登録のソースファイル)への到達に失敗した(GitLab/検索経由でヒットしたのはユーザーフォーラムの「Command Palette命名スレッド」のみ — 参考: Inkscapeも独自にコマンドパレット構想を議論している事実は確認できたが実装詳細は未確認)。**EVIDENCE_GAP**として明記し、結論には使わない。

出典: [GNU Emacs Lisp Reference — Keymaps](https://www.gnu.org/software/emacs/manual/html_node/eintr/Keymaps.html)、[kis_action.h](https://github.com/KDE/krita/blob/master/libs/ui/kis_action.h)、[krita.action](https://github.com/KDE/krita/blob/master/krita/krita.action)、[GIMP Action/Menu System(DeepWiki)](https://deepwiki.com/GNOME/gimp/3.1-action-and-menu-system)。

## §4 Rust製エディタ: Zed / Helix / Lapce

**Zed(gpui)**: 単純動詞は`actions!(namespace, [Foo, Bar])`マクロ1行で複数定義できる。引数付き動詞は`#[derive(Action)]`を付けた構造体1本(`Clone + PartialEq`必須、`serde::Deserialize`があればkeymap JSONから引数を渡せる)。deriveマクロが登録・JSON deserializeを自動生成するため、**開発者が書くのは型定義+keymap JSON1行**の2箇所。dispatchはフォーカスツリーを上へ辿る(hit-test的な設計、focus状態を持つ木構造という点でMotoliiのpane構造と類似)。

**Helix**: static command(引数なし、キーバインドのみ)は関数1本を書けば済む。typable command(`:`プロンプト、名前+補完+シグネチャ)は`TypableCommand`静的リストへ1エントリ追加+関数実装の2箇所。

**Lapce**: コマンド定義+`keymaps-common.toml`への1行、という同型の2箇所構成。

3系統とも**「型/関数という一意キー」+「別ファイルの薄い参照(JSON/TOML)」**という同じ骨格で、Rustの型システムを使う分、宣言と実装の境界がBlenderよりVS Codeに近い(型自体がid兼データ)。**Motolii(Rustの`enum Message`)は同じ言語圏でありながらこの分離を取っていない**ことが際立つ対比になる。

出典: [gpui::Action](https://docs.rs/gpui/latest/gpui/trait.Action.html)、[key_dispatch.md](https://github.com/zed-industries/zed/blob/main/crates/gpui/docs/key_dispatch.md)、[helix commands.rs](https://github.com/helix-editor/helix/blob/master/helix-term/src/commands.rs)、[helix typed.rs](https://github.com/helix-editor/helix/blob/master/helix-term/src/commands/typed.rs)、[lapce keymaps-common.toml](https://github.com/lapce/lapce/blob/master/defaults/keymaps-common.toml)。

## §5 コマンドパレットという解(§0の入口問題との関係)

9系統中7系統(Emacs M-x・Blender F3・VS Code・Zed・Helix `:`・Krita検索・GIMP検索)が「宣言済み動詞集合の全文検索実行」を持つ。共通する成立条件は**動詞の集合が既にどこか1箇所に集約されている**こと — Blenderは全`Operator`サブクラスのグローバルレジストリ、VS Codeは`contributes.commands`、Emacsは`(interactive)`フラグ付き関数のobarray。**動詞レジストリ(案1)を採らない限り、Motoliiでパレットを作っても「何を検索するか」の一次資料がなく、`Message` enumを検索用に二重管理することになる**(構造による強制と逆行)。

Ableton可視性原理(隠れていないから読める、S6公理)との関係: パレットは「探せば見える」であって「常に見えている」ではない。したがって**唯一の入口にはできない**(既存のS6併存表と同じ理由で、メニュー/shortcutを置き換える物ではなく足す物)。ただし発注が言う「入口問題(S6併存)を構造的に解く」効果は**部分的**— パレットが解くのは「動詞を探す」コストであって、「動詞をどこに置くか(メニュー階層のどこに出すか)」という意匠判断は残る。過大評価しないこと。

## §6 宣言的UIの畳み方 — Inspector表駆動案の先例照合

Blenderの`bpy.props`(`StringProperty`/`FloatProperty`等)はRNA(実行時型情報システム)へプロパティを登録し、`layout.prop(obj, "prop_name")`が**型・min/max/description等のメタデータを読んで適切なwidgetを自動選択**する(スライダー/チェックボックス/テキスト欄など)。Qtの`Q_PROPERTY`+moc(Meta-Object Compiler)も同型 — プロパティをクラスへ宣言すると実行時イントロスペクションが可能になり、Qt DesignerのProperty Editorのような汎用UIがどのクラスに対しても自動生成できる。Web側の`react-jsonschema-form`はJSON Schema(データ形)+UI Schema(表示ヒント)の2層分離で同じ効果を达成する。

**Motolii現状の実測**(`next/ui/motolii-inspector-pane/src/lib.rs`、3,652行): `transform_row`・`mask_ident_row`・`speed_row`・`hint_row`等、**プロパティ種別ごとに個別関数**が手書きされている。`PropertyId`/`TransformField`という列挙的なキーは既にあるが(§0の`Intent` variantと同型)、行の描画ロジックはキーから自動生成されず個別実装。これはBlender RNA以前・Qt property system以前の状態に相当する。

**評価**: 表駆動案(2)はBlender RNA/Qt property systemという**成熟した2系統の先例**と直接一致する縮小採用であり、目新しい発明ではない。ただしMotolii側には「値の編集がstoreへの`Intent`発行を経由する」という追加の間接層があり(RNA/Qtにはこの層が薄い、直接プロパティ書き込み)、表からwidgetを生成した後も「どの`Intent` variantへ束ねるか」の対応表は別途要る — 案4(Intentへの表示メタデータ付与)と実質的に同じ設計になる。

## §7 各approachの粗い見積り(982行のうち何行が消えるか)

前提: 「消える」は"人間が新規に書く行数が0になる、または定型テンプレへ落ちて実質コスト激減"を指し、`apply()`内のロジック本体(業務ロジック)は原理的にどの案でも残る。

| approach | 対象範囲 | 削減見積り(粗い概算) | 根拠 |
|---|---|---|---|
| (1) 動詞レジストリ(supervisor案) | 全982〜1,079行のうち**配線部分**(menu行・shortcut行・Message宣言・resolve_navigation_key腕) | 配線部分は§0実測で1動詞あたり最大4箇所(Message/menu/shortcut/Intent apply腕を除く)。VS Code型で1箇所(登録テーブル1行)に畳めば**当該4箇所×約1,000動詞のうち3箇所分、概算600〜700行相当が定型化**(手書きから生成/宣言へ移行。行数そのものはテーブル化で残るが、記述コストは1/3〜1/4) | §0実測+§2/§4先例 |
| (2) Inspector表駆動(supervisor案) | Inspector関連のUI発注行(map全体の一部、正確な内訳は本調査未実施 — 別途Inspector専用の行数集計が必要) | `transform_row`等の個別関数が表+汎用renderer1本に畳まれれば、**Inspector行の大半(型ごとの分岐がなくなる分)** | §6実測+Blender RNA/Qt property system先例 |
| (3) vismへの外出し(新規提案) | 拡張verdict済み16行+precomp/paint等島度低群(裁定175 §3の「拡張」判定分) | 数十行規模(限定的)。**980行全体には効かない**、対象を見誤ると過大な期待になる | 裁定175・拡張の哲学 |
| (4) Intentへの表示メタデータ付与(新規提案) | (1)と同じ対象、実装手段としてほぼ同一 | (1)に統合、別枠での見積り不要 | §2 VS Code型先例 |
| (5) DSL化(新規提案、非推奨) | 理論上は全982行 | 見積り非公表(採用しない) — wraps>移植>スクラッチに反し、新規構文の学習・保守コストがsupervisor案(1)を上回る | maintenance-minimal-no-scratch |
| コマンドパレット新設 | 0行削減(新しい入口を足すだけ、既存動詞の実装コストには効かない) | (1)採択後は追加コストほぼゼロ(登録済みレジストリを検索するだけ) | §5 |

**注意**: 上記は一次資料からの構造的類推であり、Motolii側で実測した行数ではない(査問6点の規律3: 反例未探索の仮説に留める)。案(1)(2)の採否判断には、実際に対象範囲(map全982〜1,129行のうちどの行がmenu/shortcut/Inspectorの各経路を通るか)の再集計が要る — 本調査はそれを実施していない(EVIDENCE_GAP)。

## §8 EVIDENCE_GAP・逸脱

1. Inkscapeのverb/action登録機構の一次資料(ソースコードまたは公式開発者ドキュメント)に到達できず、GIMPで代替。Inkscape固有の知見(あれば)は本調査に含まれない
2. `docs.blender.org`本家がWebFetchで403を返した。GitHubミラーのrst原文(内容は同一、Blender公式SVN/Git履歴からのエクスポート)で代替した。API最新版(`bpy.types.Operator`現行ページ)は検索結果のsnippetのみで、全文は未読
3. 982行という発注書の数字と`next/reference/normal-map.tsv`実測(1,079〜1,129)が一致しない。原因調査は本調査の範囲外(supervisorへ差し戻し)
4. §7の見積りは構造的類推であり、Motolii側map行の経路別再集計(menu行/shortcut行/Inspector行がそれぞれ何行か)を伴っていない。定量精度は低い
5. 反対側レビュー未実施(規律6点の規律2・6) — 本調査は単独調査であり、独立レビューでの再判定が済むまで「動詞レジストリを採用すべき」を確定結論として外向き化しない

---

状態: **調査**(2026-08-22)
