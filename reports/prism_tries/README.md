# プリズムの実験の見た目（10/10〜10/11、消した控えの記録）

git 以前の控えのフォルダ4つ（filament_base・filament_holes・filament_premerge・filament_walls）にしか無かった
実験13本の見た目。4つとも中身は同じだった。フォルダは 10/25 に消した（309fc0f）。元のファイルは最初のコミット
cd329ab に残る（例：`git show cd329ab:filament_walls/prism_gpu.mm`）。

どれも当時の Filament（Metal の SDK、third_party/filament）の作りで、今の Motolii（Filament on Dawn）にはつながらない。
利用者の見立て：古い、Filament の作りを理解していない（10/25）。

| 画 | 何か | どう撮ったか |
|---|---|---|
| `1_glass_and_path.png` | prism.mm・prism_glass・prism_filament・prism_path・prism_drop、glass_scene の三角のプリズム | 当時の書き出し（filament/out の prism*.png、日時が元のファイルと同じ）。prism-drop は当時描けずに赤紫 |
| `2_gpu_splat_and_hp.png` | prism_gpu（luma.gl の分光の集光を Filament で）、prism_hp（HPWater の1色の集光を RGB に分ける、4通り）、prism_drop | 10/25 に控えの中で build_one.sh で組み立てて動かした |
| `3_cross.png` | prism_cross（1色で計算した集光を色に分ける確かめ、5通り） | 10/25 に、prism.mm の `dump` で数値の地図（prism-mono.f32・prism-spec.f32）を書いてから動かした |

集光と床の材質6本（caustic_capture・caustic_splat・floor_glass・floor_hp・gem2・stage_floor）は、上の実験が読む物。
