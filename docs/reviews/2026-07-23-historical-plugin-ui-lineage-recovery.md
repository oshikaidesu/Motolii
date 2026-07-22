# Plugin UI lineageの価値回収（Unit 3B-UI、2026-07-23）

状態: **決定**（歴史文書15 blobの処分、現行G0-3 / GAP-13境界の訂正）

対象: `plugin-ui-model.md` 11版と`2026-07-12-plugin-ui-v1-boundary.md` 4版。

関連: [全歴史coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)、[plugin UIモデル](../plugin-ui-model.md)、[M3仕様](../specs/M3-ui-integration.md)、[軸分離決定](2026-07-22-m3-surface-extension-axis-separation.md)

## 1. 結論

2 path / 15 blobを、初版全文、主lineageの全版差分、分岐版差分、現行コード事実で処分した。履歴は「宣言UIか自由UIか」の一回の勝敗ではなく、次の訂正を重ねている。

```text
宣言語彙をv1へ広く採る初期仮説
  → 専用データモデルUIと語彙不足UIを分離
  → ParamDef field追加も互換変更だと訂正
  → v1はNodeDesc自動panelだけへ縮小
  → toolkit変更後もplugin所有UIを非公開
  → Host/communityの共通component・test語彙を再評価
  → 製品surface G0-9とplugin公開境界 G0-3/GAP-13を分離
```

現在維持する判断は三つである。

1. `NodeDesc.params`からのHost標準panelは、将来custom UIを開いても全保存paramを検査・編集できる必須fallbackである。ただし**製品U4aはコード未実装**であり、設計決定を完成済みUIと書かない。
2. plugin所有egui/native/Web/wgpu UIは現在の公開契約に無い。公開kit、sandbox、権限、互換、配布、署名、crash isolationは**G0-3 / GAP-13**で決め、G0-9の製品surface合格だけで解除しない。
3. WidgetHint、semantic ValueType、宣言layout、gizmoは一つの「宣言UI」機能ではない。影響する意味と解凍手続きが異なるため、需要ごとに別契約へ分ける。

現行M3仕様に残っていた見出し「G0-9完了まで公開しないもの」と「最終runtimeはG0-9で再評価中」は、周囲の2026-07-22訂正と矛盾していた。本単位でG0-3 / GAP-13へ修正した。

## 2. 個別処分

| 歴史path / blob | 分類 | 判定 | 現在の回収先 |
|---|---|---|---|
| `docs/plugin-ui-model.md` / `b010c5f8`,`c1bc18d0`,`efbdb47f`,`fca304ee`,`fc46203f`,`a6e9d499`,`7ebf9d1c`,`b9570b20`,`dc6585a3`,`e000d66c`,`da8016ec` | **比較中の現行規範 + 成立理由 + 再入場候補 + 負例** | WidgetHint/value type/layout/gizmo/free UIを分解した批判レビュー、Cavalry/AEの成立理由、ParamDef互換警告、NodeDesc fallback、G0-3分離を維持。旧Slint/egui到達点、G0-9へのplugin UI合否統合、Browserをplugin所有面とする読みを戻さない | 本書§3〜5、[plugin UIモデル](../plugin-ui-model.md)、[M3 G0-3](../specs/M3-ui-integration.md) |
| `docs/reviews/2026-07-12-plugin-ui-v1-boundary.md` / `9d70d5ae`,`45cbc6ff`,`5c8fbed2`,`ac02c3da` | **歴史的決定 + 現行停止線 + 訂正** | NodeDesc fallbackと比較前に自由UIを公開しない判断は生存。自由UI永久禁止やG0-9合否への統合は後続で再評価・訂正済み。将来custom UIでもfallbackを失わない | [歴史的v1境界](2026-07-12-plugin-ui-v1-boundary.md)、本書§3 |

## 3. 現行の閉じた境界と未実装

### 3.1 現在書けるplugin UI

現在の公開plugin契約にcustom UI codeは無い。pluginは`NodeDesc`、`ParamDef`、value domain、入出力、versionを宣言する。Host側のU4aが型を標準widgetとD2 commandへ接続する計画である。

コード照合では、`motolii-ui`に`NodeDesc`から製品parameter panelを生成する実装は無い。したがって現状は次の組合せである。

| 項目 | 意味状態 | コード状態 |
|---|---|---|
| `NodeDesc` / `ParamDef`評価契約 | 実装済み | registry、resolve、validation、Document prepared resolutionで使用 |
| Host標準parameter panel fallback | 決定済み | **U4a未実装** |
| plugin所有custom UI | G0-3/GAP-13比較中 | 公開API・loaderなし |

「自動panelだけがv1境界」という旧文言を、「自動panel製品実装も完成した」または「custom UIは永久禁止」と読まない。

### 3.2 presentation runtimeとplugin分類

product-owned React Browser/Inspectorとnative Stage/Timelineはbundled first-party Host moduleである。React componentを使うことはcommunity plugin UI公開の証拠ではなく、native/wgpuで描くことはCoreまたはnative pluginの証拠でもない。

G0-9は標準製品surfaceのWebView/native同居、focus、IME、DPI、a11y、lifecycleを判定する。G0-3は外部保証するcomponent/test kit、sandbox、permission、version、distribution、crash isolationを判定する。前者の実測を後者へ入力できるが、同じ合否にしない。

## 4. 宣言語彙を一括実装しない

| 候補 | 触る意味 | 再入場条件 |
|---|---|---|
| WidgetHint | 同じsemantic valueの表示選択 | `ParamDef`公開struct互換、builder/拡張形、全literal/scaffold移行、fallback mapping |
| 新ValueType | 保存値、補間、validate、serde、cache key | 型ごとの意味論、migration、負例、M2/M4審判 |
| 宣言layout | Host widgetの配置・条件表示 | theme/input/a11y、未知語彙、version、fallbackを閉じる |
| gizmo宣言 | 座標空間、順変換、逆写像、hit-test、D2 commit | canonical空間だけで帰属を推測せず、型ごとのHost-owned contractを作る |
| custom UI code | runtime、権限、資源、互換、配布、障害隔離 | G0-3/GAP-13の全審判と標準panel fallback |

現行`ParamDef`は`id`、`value_type`、`default`、`f64_domain`を持つpublic struct literalで、WidgetHintは無い。新しいoptional fieldでも既存literalを壊すため、「データ宣言だから加法的で安全」として無審査追加しない。`f64_domain`をUI slider幅と読み替えず、semantic value domainのまま保つ。

## 5. Browser lineageの扱い

一分岐版はPlugin Browserとparameter panelを明示分離し、Browserを発見／preview／適用開始、panelを適用後編集とした。この責任分離は、現在のBrowser=product-owned Host module、Inspector/U4a=Host投影という軸分離へ吸収済みである。

thumbnail、card、Detail、package browserという名前でplugin所有UI codeを迂回導入しない。逆に、Browserの表示形をplugin UI公開契約の制約だけで縮小しない。catalog identity、derived thumbnail、適用Intentの具体契約はBrowser/M3側、配布manifestやmarketplaceはVism/配布側で別に決める。

## 6. 復活させない旧具体とSTOP線

- `.slint` runtime load、plugin所有egui/native widget、wgpu texture panelを旧v1三段構えとして戻さない。
- G0-9のReact/WebView製品合格をcustom plugin UI公開許可にしない。
- Host/communityがcomponent/test語彙を共有する原則から、同じorigin、process、権限、window topologyを導かない。
- WidgetHint、新ValueType、layout、gizmo、custom codeを一つのplugin UI発注へ束ねない。
- `f64_domain`をslider、単位、step、presentation hintとして再解釈しない。
- 自動panel未実装をcustom UIで迂回せず、U4aとG0-3を別境界のまま保つ。
- 初期先例表の「CEPは破綻しUXPへ戻った」等の未確認因果を採用根拠にしない。

## 7. 固定歴史出典

| lineage | 読み方 |
|---|---|
| plugin UI model | 初版`b010c5f8`全文、批判レビュー版`c1bc18d0`から現行`da8016ec`まで主lineage全diff、Browser追補分岐`b9570b20`とG0-9再統合分岐`e000d66c`を別diffで確認 |
| v1 boundary | 初版`9d70d5ae`全文、format版`45cbc6ff`、G0-9再評価版`5c8fbed2`、G0-3軸分離版`ac02c3da`までdiff確認 |

これら15 blobは本書でDISPOSITIONEDとする。次はnative/WASM、公開能力、first/third-party runtime lineageを別単位で処分する。
