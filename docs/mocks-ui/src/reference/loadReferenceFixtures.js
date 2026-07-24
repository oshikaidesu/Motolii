const SCREEN_IDS = new Set([
  "empty-browser",
  "mixed-timeline",
  "parameter-easing",
  "stage-frame-tools",
  "shared-effect-relative",
]);

function requireObject(value, owner) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${owner} must be an object`);
  }
  return value;
}

function requireFinite(value, owner) {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new TypeError(`${owner} must be finite`);
  }
  return value;
}

export function loadReferenceFixtures(screenId, fixtures) {
  if (!SCREEN_IDS.has(screenId)) throw new TypeError(`unknown reference screen ${screenId}`);
  const document = requireObject(fixtures?.document, "document fixture");
  const scenes = requireObject(fixtures?.scenes, "scene fixture");
  const tokens = requireObject(fixtures?.tokens, "token fixture");
  const scene = requireObject(scenes.screens?.[screenId], `${screenId} scene`);
  const layers = document.layers?.entries;
  const items = document.tracks?.[0]?.items;
  if (!Array.isArray(layers) || !Array.isArray(items) || items.length !== 6) {
    throw new TypeError("reference document has an unexpected layer projection");
  }
  requireFinite(scenes.viewport?.width, "viewport.width");
  requireFinite(scenes.viewport?.height, "viewport.height");
  requireFinite(tokens["candidate-space"]?.compact?.$value?.value, "candidate spacing");
  const tokenSpacing = tokens["candidate-space"].compact.$value.value;

  const layerName = (id) => {
    const layer = layers.find((entry) => entry.id === id);
    if (!layer) throw new TypeError(`missing layer ${id}`);
    return layer.name;
  };
  const sharedDefinition = document.effect_definitions?.[0];
  if (!sharedDefinition) throw new TypeError("shared effect definition is missing");
  const indexForLayer = (layerId) => {
    const index = items.findIndex((item) => item.envelope.layer_id === layerId);
    if (index < 0) throw new TypeError(`missing item for layer ${layerId}`);
    return index;
  };
  const itemKind = (item) => {
    if (item.kind === "group") return "G";
    if (item.source?.source === "asset") {
      return (item.source.audio?.length ?? 0) > 0 ? "AV" : "V";
    }
    if (item.source?.recipe?.content?.kind === "text_path") return "T";
    return "S";
  };
  const requireItemIndex = (predicate, owner) => {
    const index = items.findIndex(predicate);
    if (index < 0) throw new TypeError(`missing ${owner} item`);
    return index;
  };

  const timelineBars = items.map((item, index) => ({
    name: layerName(item.envelope.layer_id),
    kind: itemKind(item),
    left: 4 + index * 9,
    width: 32,
    depth: scene.selection?.includes(item.envelope.layer_id) ? "SELECTED" : "0",
    flow: item.envelope.effects?.some(
      (effect) => effect.definition_id === sharedDefinition.id,
    )
      ? `IN → ${sharedDefinition.plugin_id} → OUT`
      : null,
    states: [],
  }));
  if (screenId === "mixed-timeline") {
    const shape = requireItemIndex(
      (item) => item.source?.plugin_id === "doc.layer_source.rect",
      "shape",
    );
    const text = requireItemIndex(
      (item) => item.source?.recipe?.content?.kind === "text_path",
      "text",
    );
    const group = requireItemIndex((item) => item.kind === "group", "group");
    timelineBars[shape].states.push(
      { id: "shape", label: "◇ Shape" },
      { id: "selection", label: "Selected" },
    );
    const audioVideo = requireItemIndex(
      (item) => item.source?.source === "asset" && (item.source.audio?.length ?? 0) > 0,
      "audio/video",
    );
    timelineBars[audioVideo].states.push(
      { id: "video", label: "▧ Video" },
      { id: "audio", label: "♪ Audio" },
    );
    timelineBars[indexForLayer(scene.muted[0])].states.push({ id: "mute", label: "Mute" });
    timelineBars[indexForLayer(scene.keyframed[0])].states.push({ id: "keyframe", label: "◆ Keyframe" });
    timelineBars[text].states.push({ id: "text", label: "T Text" });
    timelineBars[indexForLayer(scene.selection.at(-1))].states.push({ id: "bake-cache", label: "Bake cache · stale" });
    timelineBars[group].states.push({ id: "group", label: "G Group" });
  }
  if (screenId === "shared-effect-relative") {
    const sharedUses = items
      .map((item, index) => ({ item, index }))
      .filter(({ item }) => item.envelope.effects.some(
        (effect) => effect.definition_id === sharedDefinition.id,
      ));
    timelineBars[sharedUses[0].index].states.push(
      { id: "shared-definition", label: `Shared · ${sharedDefinition.plugin_id}` },
      { id: "from-out", label: "From / OUT" },
    );
    const middleUse = sharedUses[Math.floor(sharedUses.length / 2)];
    const middleStack = middleUse.item.envelope.effects.findIndex(
      (effect) => effect.definition_id === sharedDefinition.id,
    ) + 1;
    timelineBars[middleUse.index].states.push(
      { id: "three-nonadjacent-uses", label: `Use 2 of ${sharedUses.length}` },
      { id: "stack-position", label: `Stack · ${middleStack}` },
      { id: "use-in", label: "Use / IN" },
    );
    timelineBars[sharedUses.at(-1).index].states.push(
      { id: "connection-gutter", label: "Connection gutter" },
      { id: "fold-count", label: `${scene.folded ? "Folded" : "Expanded"} · ${sharedUses.length} uses` },
    );
  }

  const inspectorParameters = [
    { name: "Opacity", value: "72%", automation: "on", states: [] },
    { name: "Focus", value: scene.focus, automation: "off", states: [] },
  ];
  if (screenId === "parameter-easing") {
    inspectorParameters[0].states.push(
      { id: "selected-parameter", label: "Selected parameter" },
      { id: "keyframe", label: "◆ Keyframe interval" },
    );
    inspectorParameters[1].states.push({ id: "focus", label: `Focus · ${scene.focus}` });
    inspectorParameters.push(
      { name: "Warning", value: scene.hover, automation: "off", states: [{ id: "warning", label: `⚠ ${scene.hover}` }] },
      { name: "Outside interval", value: "Unavailable", automation: "off", states: [{ id: "disabled", label: `Disabled · ${scene.disabled.join(", ")}` }] },
    );
  }

  return {
    screenId,
    document,
    scene,
    tokens,
    tokenSpacing,
    browser: {
      activeTab: "project",
      items: screenId === "empty-browser" ? [] : layers.slice(0, 5).map((layer) => ({
        name: layer.name,
        purpose: "Document layer",
        state: "installed",
      })),
    },
    timeline: {
      timecode: "00:00.0",
      bpm: document.bpm.num,
      bars: timelineBars,
      inbox: [],
      depthLabels: scene.selection.slice(0, 3).map((layerId) => layerName(layerId)),
      statuses: screenId === "shared-effect-relative"
        ? [
            { id: "normal-drag", label: "Normal drag" },
            { id: "relative-hud", label: `Relative HUD · ${scene.dragMode}` },
          ]
        : [],
    },
    inspector: {
      object: layerName(scene.selection?.[0] ?? 2),
      depth: 0,
      parameters: inspectorParameters,
    },
    plugin: {
      name: sharedDefinition.plugin_id,
      input: "TEXTURE",
      output: "TEXTURE",
    },
    composition: "U0e-2 Reference",
    intervalContext: {
      objectId: "shared-middle",
      objectName: layerName(2),
      channel: "Opacity",
      startIndex: 0,
    },
  };
}
