# Inspector key add UX 決定

日付: 2026-08-13

## 利用者成果

選択objectのInspectorでアニメート可能なpropertyのkey buttonを押すと、現在のplayhead時刻と表示値が既存Document/D2へkeyとして書かれ、accepted snapshotを受けた同じproperty行がcurrent-key表示へ変わる。reject時は行とkey表示を進めず、既存typed reasonを表示する。

## 既知実装の採択

[Rive Keys](https://rive.app/docs/editor/animate-mode/keys) の次の操作構造を`PATTERN / REMAP`採択する。

- key buttonはアニメート可能なpropertyの隣に置く
- 未アニメート、他時刻にkeyあり、現在時刻にkeyありをstroke / fillで区別する
- buttonは現在値を現在playhead時刻へkeyとして設定する
- key化後も同じInspector propertyを編集する

外観、色token、Document型、command名は採択しない。Motoliiの既存toneとtyped intentを維持する。

## Motolii mapping

- `Position`と`Scale`はDocument上の`Vec2` propertyであるため、X/Yを独立keyに見せない。一つのproperty headerと一つのkey button、その下のX/Y値行として表示する。
- `Rotation`と`Opacity`はscalar propertyの値行にkey buttonを置く。
- `unkeyed`は灰色outline、`animated`はaccent outline、`current`はaccent fillで表示する。
- `unkeyed` / `animated` clickは既存`add_position_key`または`add_param_key`へ、snapshotが持つlayer identityとplayhead時刻を送る。
- `current`時の値編集は同じ行に残し、既存`set_position_key_value` / `set_param_key_value`へ送る。current-key専用の重複値行を作らない。
- local optimistic key stateを持たない。button状態と値行はHostのaccepted snapshotだけから導出する。
- rejectではkey stateを変えず、acceptedとして握りつぶさない。

## 非目標

- Timeline側へ第二のAdd Key操作を作ること
- Auto-Key modeを発明すること
- X/Yを独立key propertyへschema変更すること
- param key削除の新intentをこの粒で作ること
- tone、panel seat、Custom UI seatを削除または変更すること

## Oracle

Positive:

1. unkeyed propertyのbuttonを押す
2. 既存typed add intentがlayer/time/propertyを一回送る
3. accepted snapshotが同じpropertyのcurrent keyを返す
4. 値行は一つのまま、buttonだけがfilled currentへ変わる

Negative:

- reject応答ではoutlineのままで値行を増やさない
- current-key専用のX/Y、Rotation、Opacity値行を追加しない
- local button stateだけを先行させない

