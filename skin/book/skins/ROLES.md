# 革で使える名前（theme.xml の役割）

`~/.motolii/theme.xml` に `<名前 Value="..."/>` を並べる。書かない名前は窓のいつもの値のまま。窓は theme.xml が書き換わった瞬間に読み直す。見本は skins/labs.xml。

## 色（`#rrggbb` か `#rrggbbaa`）
`AxisX` `AxisY` `AxisZ` `BandA` `BandB` `ChosenDefault` `ChosenInk` `Clip1` `Clip2` `Clip3` `Clip4` `Clip5` `Clip6` `Clip7` `ControlOnForeground` `Glaze15` `Glaze9` `Grey00` `Grey07` `Grey10` `Grey100` `Grey13` `Grey15` `Grey20` `Grey26` `Grey38` `Grey44` `Grey56` `Grey63` `Grey76` `Grey91` `Grey95` `Keyframe` `RowLine` `ToneBlend` `ToneCamera` `ToneFill` `ToneLayer` `ToneLayout` `ToneLink` `ToneMask` `ToneOutput` `ToneParticles` `TonePath` `TonePlace` `ToneSolid` `ToneText` `ToneTextAnim` `ToneTime` `ToneTransform`

## 数
`Corner` `Size` `Spacing` `TextScale`
（Corner＝角の丸みが何倍か、Size＝部品（行・帯・列）の大きさが何倍か、Spacing＝余白とすき間が何倍か、TextScale＝文字の大きさが何倍か）

## 言葉
`FontMono` `FontSans` `Parts` `Shade`

- Shade: `light` か `dark`
- FontSans / FontMono: 書体の名前。`FontSansFile` / `FontMonoFile` に書体ファイルの場所を書くと、その名前で読み込む
- Parts: 絵の部品のフォルダ（名前は PARTS.md）。場所は theme.xml から見た相対・`~/`・絶対のどれでもよい
