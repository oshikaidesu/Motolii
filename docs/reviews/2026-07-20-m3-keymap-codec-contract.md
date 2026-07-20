# U0d-2 keymap JSON codec契約

日付: 2026-07-20

状態: **決定**。U0d-2が焼くwire形式、原本保全、version policy、migration境界を固定する。

関連: [M3着手前決定 §2.3](2026-07-16-m3-preflight-decisions.md#23-keymap保存)、
[M3仕様 U0d](../specs/M3-ui-integration.md)、[M2恒久焼き込みの予防](2026-07-12-m2-permanence-prevention.md)。

## 1. 採択範囲

本書はU0d-1で決定・実装済みの`Gesture`、`DeltaOperation`、`CommandId`を
JSONへ写すcodecだけを決める。keymap deltaはUser settingsであり、
Document、journal、Undo、plugin契約へ入れない。

2026-07-15の未統合草案commit `565e7dd`は先例としてだけ確認した。
同草案の`PresetId`、`ContextId`、`gesture_id`、command単位の全置換、
設定path、破損時fallbackは現行U0d-1契約と一致しないため**棄却**する。
採るのは「builtin不変、user delta、version付きJSON」という既に現行正本へ
移された原則だけであり、同草案をwire正本にしない。

非目標:

- 保存path、atomic write、backup配置、破損時の製品fallback
- GUI import/export、keymap preset、設定画面
- U0d-3の全command conformanceとraw key監査
- Context/scope、OS名、scancode、toolkit型
- 汎用User settings framework

## 2. v1 wire schema

初版のcodec versionは`1`とする。top-levelは次の3必須fieldを持つ。

```json
{
  "version": 1,
  "source": {
    "builtin_version": 1
  },
  "operations": [
    {
      "op": "add",
      "gesture": {
        "kind": "keyboard",
        "key": "d",
        "modifiers": ["primary"],
        "phase": "press"
      },
      "command": "motolii.edit.delete_targeted_items"
    },
    {
      "op": "replace",
      "gesture": {
        "kind": "modifier_pointer",
        "button": "primary",
        "modifiers": ["alt"],
        "phase": "drag_start"
      },
      "command": "motolii.view.fit_stage"
    },
    {
      "op": "disable",
      "gesture": {
        "kind": "key_toggle",
        "key": "t",
        "modifiers": []
      }
    }
  ]
}
```

| field | v1の意味 |
|---|---|
| `version` | codec wire version。`1`だけがcurrentであり、`BuiltinKeymap.version`とは別の版 |
| `source.builtin_version` | deltaを作った不変builtin baseの`BuiltinKeymap.version`。正整数`u32` |
| `operations` | U0d-1の順序非依存`DeltaOperation`列。配列順を優先順位に使わない |

`source`はpreset IDやpathではない。U0d-2でbuiltinは1系統だけであり、
将来presetを追加する場合は本fieldを文字列へ読み替えず、仕様改訂と追加versionを通す。
`source.builtin_version`と適用先base versionが一致しない場合は
`SourceVersionMismatch`としてdelta全体を実行しない。暗黙rebaseや一部適用をしない。

### 2.1 operation

| `op` | 必須field | 禁止field | U0d-1への変換 |
|---|---|---|---|
| `add` | `gesture`, `command` | — | `DeltaOperation::Add` |
| `replace` | `gesture`, `command` | — | `DeltaOperation::Replace` |
| `disable` | `gesture` | `command` | `DeltaOperation::Disable` |

同一正規化Gestureの複数operation等の意味検証はU0d-1 resolverが担う。
codecは配列順で解決せず、全operationを渡す。

### 2.2 gesture

```json
{"kind":"keyboard","key":"a","modifiers":["primary","shift"],"phase":"release"}
{"kind":"modifier_pointer","button":"secondary","modifiers":["alt"],"phase":"click"}
{"kind":"key_toggle","key":"space","modifiers":[]}
```

| `kind` | 必須field | 許容値 |
|---|---|---|
| `keyboard` | `key`, `modifiers`, `phase` | phase=`press` / `release` |
| `modifier_pointer` | `button`, `modifiers`, `phase` | phase=`press` / `release` / `click` / `drag_start` / `drag_end` |
| `key_toggle` | `key`, `modifiers` | phase fieldは置かない |

key文字列はASCII小文字1文字`a`〜`z`、数字1文字`0`〜`9`、または
`space / enter / escape / delete / backspace / tab / arrow_up / arrow_down /
arrow_left / arrow_right / home / end / page_up / page_down`だけとする。

modifier文字列は`primary / control / meta / alt / shift`だけとし、
U0d-1のenum順にsortし重複を除いた配列を正規形とする。
入力で順序違い・重複は同じ`Modifiers`へ正規化するが、
`primary`と`control`または`meta`の併記は型付き診断として当該operationを実行しない。

button文字列は
`primary / secondary / middle / auxiliary_1 / auxiliary_2`だけとする。
大文字、toolkitの列挙名、OS名、表示文字列は別名として受け付けない。

## 3. 読込結果と原本保全

codec読込結果は次の3面を分ける。具体的なRust型名は実装で局所化してよいが、
責任を1つのopaque blobやU0d-1の`KeymapDelta`へ畳み込まない。

1. **original bytes**: 上限確認後の入力byte列。migrationや正規化前の原本
2. **preserved JSON tree**: current wireとして検証したJSON tree。未知fieldと非実行operationを保持
3. **understood operation候補**: v1として完全に理解できたoperation。
   resolverへ渡す`KeymapDelta`は§6の`to_resolver_delta(base)`成功時だけ公開する

registryはJSON構文の正本ではない。decode時にregistryを必須にせず、
文法上正しいが未登録の`CommandId`はexecutable deltaへ残す。
登録有無と実行除外はU0d-1の`UnknownCommandId`診断が担う。

### 3.1 未知・不正入力の処分

| 入力 | 保持 | 実行 | 結果 |
|---|---:|---:|---|
| 未知top-level/source field | する | delta全体が不可 | non-fatal `UnknownEnvelopeFieldPreserved` |
| 既知operation/gesture内の未知field | する | 当該operationは不可 | non-fatal `OpaqueOperationPreserved` |
| 未知`op` / 未知`kind` | する | 当該operationは不可 | non-fatal `OpaqueOperationPreserved` |
| 文法上正しい未登録`CommandId` | する | resolverで不可 | U0d-1 `UnknownCommandId` |
| 文法不正`CommandId`、型違い、禁止phase/field、禁止modifier組 | する | 当該operationは不可 | non-fatal typed codec diagnostic |
| top-level非object、必須field欠落・型違い | originalだけ | 全て不可 | fatal `KeymapCodecError` |
| duplicate object key | originalだけ | 全て不可 | fatal `DuplicateObjectKey` |
| limit超過 | 上限を超えて複製しない | 全て不可 | fatal `LimitExceeded` |

未知fieldを含むoperationを実行しないのは、将来fieldが現在の意味を修飾する可能性を
無視して古い意味で実行しないためである。同じ理由でtop-level/sourceの未知fieldが
1つでもあればexecutable deltaを公開せず、文書全体を保全専用にする。
`version: 1`のまま追加された未知envelope意味を旧readerが無視して実行しない。

## 4. versionとmigration

version policyは閉じる。

| 入力version | 処分 |
|---|---|
| `1` | currentとして検証。migrationの固定点 |
| `0`または`1`未満 | `UnsupportedOlderVersion`。存在しない旧形式を実装で発明しない |
| `1`より大きい | `UnsupportedNewerVersion`。実行・downgrade・書換えをしない |

Motoliiはまだ出荷済みkeymap旧形式を持たない。したがってU0d-2の「migration冪等」は
恒等migration枠と原本面の固定を意味し、実在しない多版変換の完了を意味しない。
U0d-2は偽のv0 schemaを
作らない。migration入口と原本保持を初版から用意し、v1入力に対して
`migrate(migrate(x)) == migrate(x)`をpreserved JSON treeの構造一致で保証する。
将来実在する旧版を受理する時は、旧schema、変換表、意味oracle、原本保持fixtureを
先に仕様追加し、この表の該当versionだけを解凍する。

migration失敗時はoriginal bytesと入力treeを変更しない。migrationはregistry、
OS API、builtin binding内容を参照せず、wire versionだけからcurrent treeへ変換する。

## 5. write契約

writeは用途を混ぜない。

- **new write**: `source`とtyped `KeymapDelta`からv1正規形を生成する。
- **preserving write**: U0d-2には読込済みtreeの編集APIもtree再serialize経路も置かず、
  decode成功したcurrent v1のoriginal bytesを常にそのまま返す。これにより未知field、
  opaque operation、未知`CommandId`、whitespace、object順、数値表記を含むbyte一致を保証する。

new writeの正規形はtop-level field順を`version / source / operations`、
operation内を`op / gesture / command`、gesture内を
`kind / keyまたはbutton / modifiers / phase`とする。
modifierは§2.2の順とする。operationのsort keyは次のtyped tupleで固定し、
wire文字列の辞書順やRust enumの派生`Ord`へ委ねない。

1. kind rank: `keyboard=0 / modifier_pointer=1 / key_toggle=2`
2. key rank: `a`〜`z`を0〜25、`0`〜`9`を26〜35、続けて
   `space / enter / escape / delete / backspace / tab / arrow_up / arrow_down /
   arrow_left / arrow_right / home / end / page_up / page_down`を36〜49。
   pointer時は`primary / secondary / middle / auxiliary_1 / auxiliary_2`を0〜4
3. modifier rank列: `primary=0 / control=1 / meta=2 / alt=3 / shift=4`
4. phase rank: `press=0 / release=1 / click=2 / drag_start=3 / drag_end=4`。
   `key_toggle`はphase無しを0とする
5. op rank: `add=0 / replace=1 / disable=2`
6. command: add/replaceは`CommandId`のUTF-8 byte順、disableは空文字列

末尾改行は1つ、UTF-8、pretty print 2 spacesとする。

U0d-2は読込済みtreeの編集APIを作らない。将来migrationがtreeを書き換える版、
GUI、programmatic editを加える時は、
known fieldの変更と近傍unknown fieldの保持規則を別仕様で決める。
`serde_json::Value`からtyped deltaを再生成してunknownを落とす経路、
raw文字列走査でfieldをpatchする経路は作らない。

## 6. 上限とerror境界

codec入口はcaller注入のlimitsを必須にし、少なくとも
`max_bytes / max_depth / max_operations / max_string_bytes`を持つ。
数値は運用値でありwireへ保存せず、U0d-2でOS別・製品別defaultを恒久契約にしない。
上限確認は入力全体の無制限複製より前に行う。

duplicate keyはJSON objectの全階層で拒否する。後勝ち・先勝ちにしない。
検出はDeserializer visitor等の構造parseで行い、生JSON/文字列走査を契約にしない。

結果面は次の3つを混ぜない。

- `KeymapCodecError`: JSON syntax、top-level、version、duplicate key、limit等、
  delta全体を実行できないfatal error
- codec diagnostic: 保持できるが当該operationを実行しないunknown/invalid item
- `KeymapApplyError`: decode後の`to_resolver_delta(base)`境界で、
  `source.builtin_version != base.version`または未知envelope fieldを検出した場合の
  全体適用拒否。decode済みoriginal/preservedは維持し、preserving writeは可能
- `KeymapDiagnostic`: U0d-1のbase target、registry、platform、conflict診断

decodeは`source.builtin_version`をmetadataとして保持するだけでbaseを受け取らない。
`to_resolver_delta(base)`だけがsourceを照合し、成功時だけ`KeymapDelta`をresolverへ渡す。
source mismatch時に空deltaへ縮退したり、resolverを直接呼べる第二入口を作らない。

公開入口はpanicせず型付き`Result`を返す。errorを文字列へ潰さない。

## 7. U0d-2の自動審判

必須正例:

1. 3 Gesture×3 operationのdocumented JSON decode/new write
2. modifier重複・順序違いの正規化
3. current v1 migration二回の構造一致
4. current v1 read→preserving writeのbyte一致
5. 未知top-level/source field、opaque operation、未知`CommandId`の保持。
   未知envelope fieldがあればdelta全体を適用不能にする
6. source version一致時だけ`KeymapDelta`をresolverへ渡せる
7. 設定変更でDocument serialize/journal/Undoが不変

必須負例:

- version/source/operations欠落、型違い、version 0/newer、source mismatch
- duplicate key（top-level、operation、gesture）
- 未知op/kind、既知operation内unknown field、不正phase、disableのcommand field
- 不正文法CommandId、未知CommandId、禁止modifier併記
- byte/depth/operation/string上限の各超過
- 読込→保存でunknown/opaque/originalが消える、preserving writeがtreeを再serializeする
- newerをv1へdowngrade、operation配列順で後勝ち、registryでdecode拒否
- serde deriveをU0d-1 runtime型へ直接付ける、toolkit/OS/path/Documentをcodecへ混ぜる

完了条件は`motolii-ui`対象test、docs/依存境界検査、workspace test/clippyに加え、
上記fixtureが型付き結果を判定すること。期待値を書き換えて通さない。
