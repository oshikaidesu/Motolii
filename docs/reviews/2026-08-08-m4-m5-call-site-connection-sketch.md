# 仮コード — M3からM4／M5を同じrender背骨へ接続する

日付: 2026-08-08
状態: **観察 / 非compile / 修理許可でも実装許可でもない**

## 0. 扱い

[仮コード器具境界決定](2026-08-07-provisional-call-site-sketch-instrument-decision.md)に従う。
本文書は呼び出し側から接続形状を検査する器具であり、API仕様、schema、発注capsule、実装順のauthorityではない。
`???`は希望API名ではなく、現行repoで実名を置けなかった契約境界を表す。

## 1. 結果

M4とM5はM3の後へ直列に積む別phaseではない。どちらも、PreviewとExportが共有する次の背骨へ合流する。

```text
Document snapshot
  -> build_document_frame_graph
  -> [M4: Host cache / resource admission]
  -> render_graph_cached
       -> [M5: LayerSourcePlugin / RenderStep::Plugin]
       -> Composite
  -> Preview / Export
```

- **M4**は背骨の外側で、同一recipeの成果物を再利用し、miss時は既存評価へ透明に戻す。
- **M5**は背骨の内側で、既存`LayerSourcePlugin`席からpremultiplied RGBAを返し、既存Compositeへ入る。
- **M4 K1a resource ownerはM5にも共有される。** M4完了後にM5を始めるというより、同じHost所有境界を一度だけ閉じる関係である。

## 2. M3 → M4 仮接続

現行Previewの実在呼び出しを保ち、M4を挟むと次の形になる。

```rust,ignore
let built = build_document_frame_graph(
    document,
    work.request.evaluation_time,
    work.request.desc,
    &work.request.data_tracks,
    &runtime,
    None,
)?;

let recipe_key = ???_complete_render_recipe_key(
    document,
    work.request.evaluation_time,
    work.request.desc,
    work.request.quality,
    &built,
)?;

let rendered = match ???_lookup_verified_frame(&recipe_key)? {
    Some(frame) => frame,
    None => {
        let frame = render_graph_cached(
            &execute_gpu,
            &mut session,
            work.request.evaluation_time.timeline_time,
            &built.graph,
            &RenderGraphInputs {
                camera: built.camera,
                video_sources: &[],
                source_time: Some(built.source_time),
                plugins: Some(runtime.executors()),
            },
            work.request.quality,
        )?;
        ???_publish_verified_frame(&recipe_key, &frame)?;
        frame
    }
};
```

Exportも同じ`build_document_frame_graph`と`render_graph_cached`を使うため、M4のlookup／publish境界は
Preview専用関数ではなく共有Host境界でなければならない。cache miss、欠落、破損は現行renderへ戻り、
Document意味とFinal pixelを変えない。

### M4の`???`

| `???` | exact gap | 既存判定との照合 |
|---|---|---|
| `???_complete_render_recipe_key` | version付きsource fingerprint、全入力、時間区間、Quality、FrameDescを含む完全keyの製品encoderが無い | `M4-P02 STOP / GAP-3`と一致 |
| `???_lookup_verified_frame` | private disk store、ResourceLedger、参照handle、世代、破損をmissへ落とす製品runtimeが無い | `P03/P04/P05/P06 probe VERIFIED・製品未接続`、`P05-C2/C3 RUNTIME ABSENT`と一致 |
| `???_publish_verified_frame` | 非同期copy-out、hard-budget admission、temp→検証→rename後の登録が無い | `P09 STOP / GAP-29`、`K7a RUNTIME ABSENT`と一致 |

この鎖はGroup Bakeだけに限定されない。まず通常frameの透明なlookup／miss経路が同じ背骨へ接続し、
K7はその上で「group子合成直後」を成果物境界として指定する。Group Bake固有の
`???_group_child_output_boundary`、区間無効化、hit時の内部graph置換は引き続きK7a／K7b／K7cの欠落である。

## 3. M3 → M5 仮接続

現行codeでは`ClipSource::Plugin`が`RenderStep::Plugin`へlowerされ、`render_graph_cached`が
`LayerSourcePlugin::render`を呼ぶ。この席はそのまま使える。

```rust,ignore
let built = build_document_frame_graph(
    document,
    work.request.evaluation_time,
    work.request.desc,
    &work.request.data_tracks,
    &runtime,
    None,
)?;

let observation = ???_projective_observation(
    built.camera,
    work.request.desc,
    ???_active_camera_provider_binding(document),
)?;

let spatial_resources = ???_admit_compiled_spatial_assets(
    document,
    &built,
    ???_shared_resource_ledger(),
)?;

let rendered = render_graph_cached(
    &execute_gpu,
    &mut session,
    work.request.evaluation_time.timeline_time,
    &built.graph,
    &RenderGraphInputs {
        camera: built.camera,
        video_sources: &[],
        source_time: Some(built.source_time),
        plugins: Some(runtime.executors()),
        // ??? observation と spatial_resources を LayerSourceContext へ渡す公開口が無い。
    },
    work.request.quality,
)?;
```

M5 Provider自体の最終合流は既存経路で閉じる。

```rust,ignore
// build_document_frame_graph:
RenderStep::Plugin { id, params, inputs: vec![], output }

// render_graph_cached:
LayerSourcePlugin::render(/* ..., */ ???_layer_source_context, output)?;

// 以後は既存:
RenderStep::Composite { /* M5出力をforegroundとして合流 */ }
```

### M5の`???`

| `???` | exact gap | 既存判定との照合 |
|---|---|---|
| `???_projective_observation` | `CompCamera`はPlanar評価済みだが、representation非依存の公開Observation型が無い | `M5-C0 schema preflight STOP`と一致 |
| `???_active_camera_provider_binding` | single active binding、provider identity、pin、migrationのDocument／wireが無い | `M5-C0 provider wire未成立`と一致 |
| `???_admit_compiled_spatial_assets` | faithful import assetからGPU compiled assetへ進むImporter／GPU cache／Host admissionが無い | `M5-A1/A2/R0 private DONE・製品未接続`と一致 |
| `???_shared_resource_ledger` | M5が独自所有してはいけない共有hard-budget ownerが無い | `M4 K1a未成立`、M5休止契約の再開順と一致 |
| `???_layer_source_context` | 現行`LayerSourceContext`は`camera: CompCamera`だけで、Observation／resource handle／linear scene contributionを運べない | Stage×M5判定の3欠落と一致 |

M5側で色変換を追加してはならない。M5 ProviderはHostが定めるlinear-premultiplied scene-color契約へ出力し、
既存CompositeからPreview／Exportへ同じ評価で流す必要がある。具体format／合流口は現行未成立なので`???`のまま残す。

## 4. 接続後に見えた順序

```text
M3の共有render背骨
  -> M4 K1a: shared ResourceLedger / hard-budget owner
  -> M4 P02: 完全key（GAP-3裁定が再入場条件）
  -> M4 lookup / miss / verified publish

同じshared ResourceLedger
  -> M5 C0 schema + provider binding
  -> M5 faithful asset admission / compiled GPU asset
  -> existing LayerSourcePlugin seat
  -> existing Composite -> Preview / Export
```

したがって「M4を全部作ってからM5」ではない。最初の共有施工候補はM4 K1aのHost resource ownerだが、
P02、C0、Importer、linear contributionはそれぞれ別の契約境界として残る。

## 5. 相互検証の結論

仮コードの`???`は既存survey／ledgerの`STOP`または`RUNTIME ABSENT`へすべて対応した。
新しい万能cache、scene graph、ECS、M5専用render経路を要求する`???`は出なかった。

一方、以前の「Stage×M5で不足している一つ=C0-Schema」は**最初のM5固有gate**という意味では保たれるが、
製品接続全体がC0一枚だけで完成するわけではない。共有resource owner、asset admission、linear contributionも必要である。

## 6. M4全機構の仮コード

以下は一つの巨大runtimeを提案するものではない。利用者outcomeから見た呼び出しを一枚へ並べ、
各`???`を独立施工候補へ分解する。

### 6.1 共通frame要求

```rust,ignore
fn ???_evaluate_product_frame(snapshot: &Document, request: ???_ProductFrameRequest) -> Result<RenderedFrame, ???_ProductFrameError> {
    let built = build_document_frame_graph(
        snapshot,
        request.evaluation_time,
        request.desc,
        request.data_tracks,
        request.runtime,
        request.project_root,
    )?;

    let generation = ???_published_snapshot_generation(snapshot);
    let required_regions = ???_propagate_required_regions(
        &built.graph,
        request.output_region,
    )?;
    let recipe_key = ???_complete_render_recipe_key(
        snapshot,
        generation,
        &built,
        required_regions,
        request.quality,
    )?;

    if let Some(frame) = ???_lookup_admitted_frame(&recipe_key)? {
        return Ok(frame);
    }

    let admission = ???_admit_render_request(
        ???_shared_resource_ledger(),
        ???_estimate_graph_resources(&built.graph, request.quality)?,
    )?;

    let frame = render_graph_cached(
        request.gpu,
        request.render_session,
        request.evaluation_time.timeline_time,
        &built.graph,
        &???_render_inputs(request, built.camera, built.source_time),
        request.quality,
    )?;

    ???_enqueue_verified_copyout(
        admission,
        recipe_key,
        generation,
        &frame,
    )?;
    Ok(frame)
}
```

この鎖の独立境界:

| 境界 | 対応子 | 共有点 |
|---|---|---|
| region propagation | P01-C1/C2 | render graphのprivate extent seamだけ |
| identity/generation | P02-C1/C2/C3 | GAP-3のversion付きfingerprint |
| RAM handle | P03-C2/C3 | P04 admission、P02 key |
| hard admission | P04-C1/C2/C3 | **M4/M5共有Host owner** |
| disk artifact | P05-C2/C3 | P02 key、P04 admission |
| affected windows | P06-C2/C3 | typed Document mutationとgeneration |
| scheduler | P07-C2/C3 | job publicationだけ。Document writerにはならない |
| copy-out | P09-C1/C2 | P04 admission、P05 commit |

Audioも別cache ownerを残さず、同じRAM admissionへ入る。

```rust,ignore
let pcm = ???_shared_ram_artifact_cache().get_or_compute(
    ???_audio_recipe_key(asset, sample_range, channel_layout),
    || ???_decode_audio_program(asset, sample_range),
)?;
???_feed_audio_without_blocking_transport_clock(pcm)?;
```

### 6.2 Proxy

```rust,ignore
let proxy_recipe = ???_proxy_recipe(
    ???_versioned_source_identity(asset),
    request.scale,
    request.fps,
    ???_ffmpeg_codec_version(),
)?;

???_bounded_job_scheduler().submit(???_BackgroundJob::Proxy {
    generation: ???_published_snapshot_generation(snapshot),
    recipe: proxy_recipe,
    run: || ???_run_existing_ffmpeg_sidecar_with_cfr_verification(asset),
    publish: |artifact| ???_commit_verified_artifact(artifact),
})?;

let decoder_source = match ???_lookup_verified_proxy(&proxy_recipe)? {
    Some(proxy) if request.quality != Quality::FINAL => proxy,
    _ => resolve_asset_path(snapshot, asset, request.project_root)?,
};
```

P08-C1/C2はFFmpeg sidecarとVFR/CFR検証として、cache playbackから独立して作れる。
P08-C3の製品置換だけがP02/P05へ合流する。

### 6.3 Group Bake

```rust,ignore
let group_output = ???_group_child_output_boundary(
    &built.graph,
    selected_group,
    bake_interval,
)?;
let bake_key = ???_group_bake_recipe_key(
    snapshot,
    group_output,
    bake_interval,
    request.quality,
    request.desc,
)?;

???_bounded_job_scheduler().submit(???_BackgroundJob::GroupBake {
    generation,
    missing: ???_affected_interval_gaps(selected_group, bake_interval),
    render: |interval| ???_render_group_child_output(group_output, interval),
    publish: |frames| ???_commit_verified_artifact(frames),
})?;

let graph = match ???_lookup_group_bake(&bake_key)? {
    Some(artifact) => ???_substitute_group_graph_with_artifact(&built.graph, artifact),
    None => built.graph,
};
```

K7/P10は独立frameworkではなく、P02/P05/P06/P07/P09の合成である。したがって前半5境界は並列、
Group Bakeのpublicationとgraph substitutionだけが後段の直列合流になる。

### 6.4 全曲Draft

```rust,ignore
let coverage = ???_draft_coverage(generation, composition_duration);
let jobs = ???_plan_coverage_jobs(
    coverage.gaps(),
    transport.current_time(),
    ???_PreviewPriority::NEXT_FRAME_FIRST,
)?;
???_bounded_job_scheduler().replace_stale_generation(generation, jobs)?;

let frame = match ???_lookup_draft_frame(generation, transport.current_time())? {
    Some(frame) => frame,
    None => evaluate_product_frame(snapshot, request)?,
};
???_present_latest_frame_without_delaying_audio_clock(frame)?;
```

P11-C1 plannerはP06/P07だけで先行可能。P11-C2はP05 store、P11-C3はP04 hard budgetと通常製品routeへ合流する。

### 6.5 Preview pressure

```rust,ignore
let capacity = ???_resource_snapshot_provider().capacity_signal();
let deadline = ???_preview_deadline_signal(render_metrics);
let preview_quality = ???_choose_preview_quality_with_hysteresis(
    request.user_quality_policy,
    capacity,
    deadline,
)?;
```

P12-C1 capacityとP12-C2 deadlineは別module／別oracleで並列に作れる。P12-C3は両者をread-only snapshotへ合流し、
M3 HUDへ渡す。UIはpurge／evict／scheduler write権限を持たない。

### 6.6 SVG

```rust,ignore
let checked_bytes = ???_reject_svg_external_resources_before_parse(asset_bytes)?;
let private_scene = ???_lower_with_vello_svg(checked_bytes)?;
???_render_reused_vello_scene(private_scene, output)?;
```

P13-C2/C3はM4のidentity/store/schedulerから独立する。共有点は既存GPU device、長寿命renderer、
premultiplied outputだけであり、独自parser、公開`usvg`型、frameごとのrendererを作らない。

## 7. M5全機構の仮コード

M5は第二のscene engineを作らず、private asset／evaluation／contributionを既存worldとrenderへ差し込む。

### 7.1 GLB／OBJ importとprivate asset

```rust,ignore
let asset_id = ???_atomically_admit_asset_through_single_writer(
    document_edit_runtime,
    selected_file,
    ???_typed_spatial_asset_metadata(selected_file),
)?;
let source = resolve_asset_path(snapshot, asset_id, project_root)?;
let admitted_bytes = ???_preflight_asset_bytes(
    source,
    ???_shared_resource_ledger(),
    ???_ImportLimits::product_defaults(),
)?;

let faithful_asset = match source.extension() {
    "glb" | "gltf" => ???_lower_with_gltf_and_mikktspace(admitted_bytes)?,
    "obj" => ???_lower_with_tobj_to_same_private_asset(admitted_bytes)?,
    ext => return Err(???_typed_unsupported_spatial_asset(ext)),
};
???_whole_asset_capability_preflight(&faithful_asset, ???_gpu_capabilities(request.gpu))?;
let compiled_asset = ???_compile_private_gpu_asset(
    faithful_asset,
    ???_shared_resource_ledger(),
)?;
```

M5-A1/A2のparser／diagnostic probeはDONE。製品化ではHost admissionとcompiled asset ownerだけを薄く追加する。
GLBとOBJは入口のみ並列で、同じprivate faithful assetへ合流した後はmaterial／rendererを共有する。
ただし入口のさらに上流に、Asset登録をsingle writer／journal／Undoへ一つの製品操作として載せる境界が必要である。
これはM4 `GAP-3`の指紋formatとは別で、楽曲bed／通常media importと共有するM2 Asset admission／lifecycleの
`ABSENT / POLICY_GAP`である。

### 7.2 Observation

```rust,ignore
let camera_binding = ???_single_active_camera_provider_binding(snapshot)?;
let provider = ???_resolve_pinned_camera_provider(camera_binding)?;
let observation = ???_evaluate_camera_provider(
    provider,
    snapshot,
    request.evaluation_time,
    request.desc,
)?;
???_preflight_observation_capabilities(&observation, requested_capabilities)?;
```

C0のprivate意味fixtureはDONE。`???`は公開Observation schema、provider identity/version、Document binding、
migration、runtime wireである。具体provider型や`glam`型はDocument／plugin境界へ出さない。

### 7.3 Layer Order spatial renderer

```rust,ignore
let objects = ???_evaluate_private_object_list(
    snapshot,
    request.evaluation_time,
    &compiled_assets,
)?;
let contribution = ???_render_spatial_layer_source(
    objects,
    observation,
    request.quality,
    ???_admitted_scene_color_target(),
)?;

// 既存経路へ合流する。
RenderStep::Plugin { id, params, inputs: vec![], output };
RenderStep::Composite { background, foreground: output, output: composite, mode };
```

M5-R0のPBR/unlit renderer probeはDONE。R1はM4 resource owner、C0 Observation、scene-color targetが揃えば、
既存`LayerSourcePlugin`席へ独立接続できる。3D未使用Documentではこの枝自体が生成されず、既存pixelを変えない。

### 7.4 Group Depth／AE-style Bins

```rust,ignore
let policy = ???_evaluate_group_occlusion_policy(snapshot, group_id)?;
let request = ???_collect_render_contributions(group_id, policy, observation)?;
let admitted = ???_admit_whole_depth_request(
    ???_shared_resource_ledger(),
    ???_requested_attachments(&request),
    ???_requested_alpha_classes(&request),
)?;

let group_output = match policy {
    "Layer Order" => ???_composite_in_authoring_order(admitted)?,
    "Group Depth" => ???_resolve_shared_depth(admitted)?,
    "AE-style Bins" => ???_resolve_explicit_participant_bins(admitted)?,
    unknown => return Err(???_typed_unknown_occlusion_policy(unknown)),
};
```

opaque/cutoutとsoft alphaを混同しない。P2D/R2はC0＋R1＋M4 admission後の合成境界だが、
policy Document command／UI、attachment admission、depth resolve fixtureは別ownerで並列起草できる。

### 7.5 Post

```rust,ignore
let region = ???_post_required_input_region(node, requested_output_region)?;
let output = match node {
    Blur(params) => ???_wgpu_blur_pass(input, region, params, request.quality)?,
    LiftGammaGain(params) => ???_wgpu_lgg_pass(input, params)?,
    Grain(params) => ???_wgpu_grain_pass(input, params, request.evaluation_time)?,
};

let encoded_frame = ???_quantize_once_at_encode_boundary_with_default_dither(
    output,
    request.output_pixel_format,
)?;
```

M5-P0のalgorithm contractはDONE。GPU passは既存filter/render graphと`PipelineCache`へ接続し、
M4 P01 extentとP04 resource admissionへ合流する。Blur、LGG、Grainはshader／goldenを別粒で並列化できるが、
scene-color契約とfilter graph publicationは一回だけ直列化する。8bit量子化／ditherは各nodeへ分散させず、
encode境界の一箇所だけが所有する。

### 7.6 Text

```rust,ignore
let runs = ???_itemize_text_with_diagnostics(text, style, locale)?;
let shaped = ???_shape_runs_with_fontique_harfrust_or_private_parley(runs)?;
let clusters = ???_cluster_map(&shaped)?;
???_draw_shaped_runs_with_reused_vello(shaped, glyph_transform, output)?;
```

M5-T0はDONE。P6 product leafは3D import、Observation、depth、Duplicatorから独立する。
共有点はVello renderer publicationとGPU resource admissionだけである。縦書き、ルビ、歌詞timingはHost text coreへ入れない。

### 7.7 Picking／gizmo／bounds

```rust,ignore
let projection = ???_project_semantic_objects(observation, evaluated_objects)?;
let pick_generation = ???_stage_projection_generation();
let hit = ???_pick_without_sync_readback(
    projection,
    pointer,
    pick_generation,
)?;
if hit.generation == ???_stage_projection_generation() {
    ???_publish_transient_selection(hit.semantic_id)?;
}
```

I0のCPU semantic/stale fixtureはDONE。Stage projection、BVH build、selection projectionは、renderer publicationと別に進められる。
Document writeは既存D2 commit時だけで、hover／pick workerはwriterにならない。

### 7.8 Scale／Depth Move／Depth Rail

```rust,ignore
let projection = ???_project_depth_rail(
    snapshot,
    request.evaluation_time,
    observation,
    transient_selection.stable_ids(),
)?;
???_publish_read_only_depth_rail_projection(projection)?;

match gesture {
    Scale(delta) => ???_commit_one_d2_macro(???_scale_only_commands(delta))?,
    DepthMove(delta_z) => ???_commit_one_d2_macro(???_position_z_only_commands(delta_z))?,
    Distribute { near, far } => {
        ???_commit_one_d2_macro(???_authoring_order_z_distribution(near, far))?
    }
    Cancel | FocusLost => ???_discard_transient_preview_without_document_write(),
}
```

P2U/P2RはM5 rendererの別worldを必要としない。C0 Observation、既存stable selection、既存D2 single writerへ接続する。
projection／hit-test／Skia描画はread-only sandboxとして並列化できるが、Position Z／Scale commandの意味と
一gesture一macroのpublicationはDocument ownerへ直列化する。Railの開閉、viewport、focusはTransientである。

### 7.9 Duplicator

```rust,ignore
let recipe = ???_read_duplicator_recipe(snapshot, duplicator_id)?;
let instances = ???_evaluate_stable_instances(
    recipe.input_shapes,
    recipe.distribution,
    recipe.user_seed,
    request.evaluation_time,
)?;
let channels = instances.map(|instance| {
    ???_evaluate_behaviours_purely(
        instance.context,
        request.evaluation_time,
        recipe.behaviours,
    )
});
???_render_gpu_instances_without_document_rows(instances, channels)?;
```

D0のstable identity fixtureはDONE。P7a schema、P7b Host evaluator、P7c Behaviour、P7U UIは直列だが、
P7a確定前でもGPU instance capability probeとUI projectionのread-only sketchは別laneで検査できる。
製品publicationはschema→evaluator→Behaviour→UIの順を守る。

```rust,ignore
let inspector = ???_project_duplicator_inspector(snapshot, duplicator_id)?;
let stage_instances = ???_project_derived_instance_selection(instances)?;
???_publish_read_only_duplicator_ui(inspector, stage_instances)?;

// UIは生成instanceをDocument rowへ書かず、recipe変更だけを既存writerへ返す。
???_commit_one_d2_macro(???_duplicator_recipe_commands(user_intent))?;
```

### 7.10 M5統合出口

```rust,ignore
let frame = ???_evaluate_same_world(
    video_planes,
    text,
    vectors,
    spatial_assets,
    duplicated_instances,
    observation,
    occlusion_policy,
    post_nodes,
    request.quality,
)?;

// PreviewとExportで同じ関数。差はQualityと最終encodeだけ。
```

P5は上記全枝の最終E2Eであり、並列施工対象ではなく合流審判である。

## 8. sandbox／最小Coreとの対応

並列化しやすい理由は、各採択routeを次の三層へ閉じられるためである。

```text
Document/Core                         Host private sandbox                       Existing product seam
stable IDs / typed recipe      ->     parser/cache/shader/BVH adapter     ->     writer/render/Stage projection
作品意味だけ                           外部型と派生stateを封じる                    一回だけpublication
```

private sandboxごとの禁止事項:

| sandbox | 内部へ閉じるもの | Core／公開面へ漏らさないもの |
|---|---|---|
| M4 cache | foyer/rangemap/priority-queue/tempfile/fs4型 | eviction構造、path、runtime handle |
| media | FFmpeg command／probe／proxy artifact | codec default、PTS補正state |
| spatial import | gltf/tobj/mikktspace/private faithful asset | parser scene型、URI、crate enum |
| spatial render | compiled mesh/material/pipeline | engine scene、backend resource型 |
| text | Fontique/HarfRust/Parley/Vello adapter | layout engine state、font database型 |
| picking | BVH／async generation | BVH node、GPU ID buffer、pixel座標 |
| duplicator | PCG32／derived instance buffers | generated rows、乱数結果、ECS identity |

Coreが持つのは作品意味とstable identity、Hostが持つのはadmission／translation／lifetime、
採択libraryが持つのは一般機構である。この分離なら各sandboxの内部施工はfile-disjointにでき、
公開契約の同時編集を避けられる。

## 9. 依存DAGから得た並列lane

### Wave 0 — 済んでいる独立検証

同時に再施工する必要はない。

- M4: P03-C1、P04-C4、P05-C1、P06-C1、P07-C1、P13-C1
- M5: A1、A2、R0、T0、P0、I0、D0

### Wave 1A — authority／計測を閉じる独立lane

製品実装を始めず、互いに並列で閉じられる。

1. `GAP-3` version付きsource fingerprint／完全key
2. `GAP-29` copy/map/encode/disk原因分離
3. M4 P01 private region seam
4. M4 P04 descriptor estimator＋Host resource policy
5. M4 P07 bounded worker lifecycle
6. M2 Asset admission／journal／Undo lifecycle（M5 import、media、楽曲bedの共有境界）
7. M5 P7a Duplicator schema（M3/M2 schema gate後）
8. M5 P6 private text run API

論理上8 laneある。ただしAsset lifecycleとP7aはDocument schema／writer publicationが競合するため、
**調査・fixtureは並列、Document commitは一つずつ直列**にする。

### Wave 1B — shared resource後に開くM5契約

- M5 C0 public Observation schema／provider identity／migration
- M5 scene-color／Render Contribution admission契約

休止契約どおり、M3意味開放とM4 K1a shared resource ownerの確認後に開く。C0とscene-colorは別契約だが、
Document migrationとrender/plugin contextへのpublicationはそれぞれ一回だけ行う。

### Wave 2 — private foundation

Wave 1の各入力が閉じたものから、全wave完了を待たず開始できる。

1. M4 generation snapshot＋affected-window projection
2. M4 RAM adapter
3. M4 disk store adapter
4. M4 admitted copy-out
5. M4 proxy/CFR producer
6. M4 SVG product adapter
7. M5 GLB/OBJ Host importer→faithful asset
8. M5 compiled GPU asset＋Layer Order renderer
9. M5 post Blur／LGG／Grain GPU passes
10. M5 text product leaf
11. M5 picking projection／BVH
12. M5 Duplicator Host evaluator／GPU instance

論理上12 laneある。実際の同時publicationは次の共有点で絞る。

- `Cargo.toml`／`Cargo.lock`: dependency adoptionを一列にpublication
- ResourceLedger: owner実装は一人、各consumer adapterは並列
- `RenderGraphInputs`／`LayerSourceContext`／render step: contract変更を一列にpublication
- Document schema／migration: variantごとに別検討してもmergeは一列
- GPU device／pipeline integration test: 実機laneは共有し、fixture実装は並列

### Wave 3 — composed product outcomes

1. M4 Group Bake atomic producer
2. M4 interval invalidation＋graph substitution
3. M4 full Draft planner／disk playback
4. M4 pressure controller＋M3 HUD projection
5. M5 Group Depth／AE-style Bins
6. M5 Depth/Picking Stage interaction
7. M5 Duplicator Behaviours＋UI

Group Bake、Draft、Group Depthはそれぞれ内部では別laneだが、通常Preview／Exportへのcutoverは一件ずつ行う。

### Wave 4 — 合流審判

- M4 K7/K8 E2E: cache有無、破損、再起動、編集後のpixel／clock同値
- M5 P5 E2E: video＋text＋spatial＋Duplicator＋postの同一world／同一Observation／同一Preview-Export評価
- M4/M5共通: hard budget、device loss、low-spec、3 OS、stale generation、Document不変

ここは並列実装waveではない。成果を一つの製品routeで衝突させる統合審判である。

## 10. 並列度の結論

仮コード上の最大独立laneは、authority段で**8**、private foundation段で**12**ある。
ただしこれは12人が同じworktreeで同時編集できるという意味ではない。

安全な実行単位は次の形になる。

- private fixture／adapter施工: 最大8〜12 laneを独立worktreeで並列化可能
- shared contract publication: ResourceLedger、Document schema、render/plugin context、lockfileの4列へ縮約
- product cutover: Preview／Export、Stage、Document writerのowner単位で1件ずつ
- independent review／platform oracle: 実装laneとは別family／別実機で並行可能

したがってMotoliiの最小Core＋sandbox思想は、**内部施工の広い並列性と、境界publicationの少数直列化**に適している。
全機構を一つのphase列へ並べる必要はない。直列に守るべきなのは機能ではなくownerである。

## 11. 現在のSTOPとの照合

| 仮コードroot | 現行状態 | safe parallel edge |
|---|---|---|
| M4 complete key | `GAP-3 / STOP` | region、scheduler、SVG、copy-out計測 |
| M4 ResourceLedger | `RUNTIME ABSENT` | GAP-3、region、scheduler、SVG |
| M4 copy-out | `GAP-29 / STOP` | identity、store model sketch、proxy verification |
| M4 Group Bake／Draft | producer absent | 上流P01〜P09を別laneで閉じる |
| M5 product runtime | M3意味開放待ち | private receipts維持、仮コード／read-only整合監査 |
| M5 C0 schema | schema preflight STOP | text、post algorithm、import receipts、Duplicator fixture |
| M5 product Asset import | M2 Asset admission／Undo policy gap | parser adapter、renderer fixture、C0 decision |
| M5 resource/compiled asset | M4 K1a待ち | C0 decision、text、picking semantics、schema work |
| M5 depth | C0＋resource gate待ち | Layer Order renderer、policy fixture、UI sketch |

局所STOPは他laneを止めない。古いwaveを一括解禁するのではなく、各return後にこのDAGから
依存が満たされた一契約境界だけを再選定する。

## 12. 非目標

- 本文書を根拠にM4／M5の休止、STOP、GAP-3を解除すること
- `???`の名称や引数を公開APIとして採用すること
- `PipelineCache`やping-pong targetをResourceLedger／frame cache完成と数えること
- M5 Provider内に独自色変換、asset cache、camera schemaを持たせること
- Preview専用cache、Export専用renderer、M5専用rendererを新設すること
- K1a、P02、K7、C0、Importer、linear contributionを一粒へ束ねて発注すること
- lane数を達成するため不要な施工／外部callを増やすこと
- file-disjointだけでwriter、GPU device、lockfile、artifact publicationの衝突を無視すること
- Wave表を固定実装順や一括発注表として扱うこと
