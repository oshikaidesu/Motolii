const FIXTURE_REVISION = 1;

const WRAPPER_KEYS = new Set([
  "fixtureRevision",
  "document",
  "nodes",
  "target",
]);

const DOC_PARAM_TAGS = new Set([
  "const",
  "keyframes",
  "data",
  "vec2_axes",
  "look_at",
  "follow",
]);

const VALUE_TYPE_TAGS = new Set(["F64", "Vec2", "Vec3", "Color", "AssetRef"]);

const PARAM_VALUE_TYPES = new Set(["F64", "Vec2", "Vec3", "Color", "AssetRef"]);

const BLEND_MODES = new Set(["normal", "add", "multiply"]);

const ITEM_KINDS = new Set(["clip", "group"]);

const LOOK_AT_AXES = new Set(["plus_y", "plus_x"]);

const TRANSFORM_DOC_PARAM_KEYS = new Set([
  "position",
  "anchor",
  "scale",
  "rotation",
]);

const FORBIDDEN_OUTPUT_KEYS = new Set([
  "fill_color",
  "stroke_color",
  "z_occlusion",
  "occlusion_mode",
  "depth_z",
  "bake_point",
  "composite_bake",
  "driver_route",
  "driver_routes",
  "applied_plugin_history",
  "availability",
  "availability_lifecycle",
  "effect_description",
  "input_socket_label",
  "socket_type_tag",
  "link_label",
  "link_target_label",
  "primary_selection",
  "selected_object",
  "editing_effect",
  "at_key",
]);

function fail(ruleId, owner, detail) {
  throw new TypeError(`R${ruleId}: ${detail} at ${owner}`);
}

function requireObject(value, ruleId, owner) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    fail(ruleId, owner, "expected object");
  }
  return value;
}

function requireFinite(value, ruleId, owner) {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    fail(ruleId, owner, "expected finite number");
  }
  return value;
}

function requireInteger(value, ruleId, owner) {
  requireFinite(value, ruleId, owner);
  if (!Number.isInteger(value)) {
    fail(ruleId, owner, "expected integer");
  }
  return value;
}

function validateDocValue(value, ruleId, owner) {
  const obj = requireObject(value, ruleId, owner);
  const keys = Object.keys(obj);
  if (keys.length !== 1) {
    fail(ruleId, owner, "DocValue must have exactly one key");
  }
  const tag = keys[0];
  if (!VALUE_TYPE_TAGS.has(tag)) {
    fail("6", owner, `unknown DocValue tag ${tag}`);
  }
  if (tag === "F64") {
    requireFinite(obj.F64, "5", `${owner}.F64`);
  } else if (tag === "Vec2") {
    const arr = obj.Vec2;
    if (!Array.isArray(arr)) {
      fail("4", `${owner}.Vec2`, "expected array");
    }
    if (arr.length !== 2) {
      fail("4", `${owner}.Vec2`, "expected length-2 array");
    }
    for (let i = 0; i < arr.length; i += 1) {
      requireFinite(arr[i], "5", `${owner}.Vec2[${i}]`);
    }
  } else if (tag === "Vec3") {
    const arr = obj.Vec3;
    if (!Array.isArray(arr)) {
      fail("4", `${owner}.Vec3`, "expected array");
    }
    if (arr.length !== 3) {
      fail("4", `${owner}.Vec3`, "expected length-3 array");
    }
    for (let i = 0; i < arr.length; i += 1) {
      requireFinite(arr[i], "5", `${owner}.Vec3[${i}]`);
    }
  } else if (tag === "Color") {
    const arr = obj.Color;
    if (!Array.isArray(arr)) {
      fail("4", `${owner}.Color`, "expected array");
    }
    if (arr.length !== 4) {
      fail("4", `${owner}.Color`, "expected length-4 array");
    }
    for (let i = 0; i < arr.length; i += 1) {
      requireFinite(arr[i], "5", `${owner}.Color[${i}]`);
    }
  } else if (tag === "AssetRef") {
    requireInteger(obj.AssetRef, "5", `${owner}.AssetRef`);
  }
  return obj;
}

function cloneDocValue(value, owner) {
  const tag = Object.keys(value)[0];
  if (tag === "F64") return { F64: value.F64 };
  if (tag === "Vec2") return { Vec2: [value.Vec2[0], value.Vec2[1]] };
  if (tag === "Vec3") return { Vec3: [value.Vec3[0], value.Vec3[1], value.Vec3[2]] };
  if (tag === "Color") {
    return { Color: [value.Color[0], value.Color[1], value.Color[2], value.Color[3]] };
  }
  if (tag === "AssetRef") return { AssetRef: value.AssetRef };
  fail("6", owner, `unknown DocValue tag ${tag}`);
}

function validateOpacityConstF64(docParam, ruleId, owner) {
  const obj = requireObject(docParam, ruleId, owner);
  const keys = Object.keys(obj);
  if (keys.length !== 1 || keys[0] !== "const") {
    return;
  }
  const tagged = validateDocValue(obj.const, ruleId, `${owner}.const`);
  if (!Object.hasOwn(tagged, "F64")) {
    return;
  }
  const n = requireFinite(tagged.F64, ruleId, `${owner}.const.F64`);
  if (n < 0 || n > 1) {
    fail("10", owner, "opacity Const(F64) must be in [0,1]");
  }
}

function validateDocParam(param, ruleId, owner, layerIds) {
  const obj = requireObject(param, ruleId, owner);
  const keys = Object.keys(obj);
  if (keys.length !== 1) {
    fail("6", owner, "DocParam must have exactly one key");
  }
  const tag = keys[0];
  if (!DOC_PARAM_TAGS.has(tag)) {
    fail("6", owner, `unknown DocParam tag ${tag}`);
  }
  if (tag === "const") {
    validateDocValue(obj.const, ruleId, `${owner}.const`);
    return;
  }
  if (tag === "keyframes") {
    requireObject(obj.keyframes, ruleId, `${owner}.keyframes`);
    return;
  }
  if (tag === "data") {
    const data = requireObject(obj.data, ruleId, `${owner}.data`);
    requireObject(data.track, "4", `${owner}.data.track`);
    validateDocValue(data.fallback, ruleId, `${owner}.data.fallback`);
    return;
  }
  if (tag === "vec2_axes") {
    const axes = requireObject(obj.vec2_axes, ruleId, `${owner}.vec2_axes`);
    validateDocParam(axes.x, ruleId, `${owner}.vec2_axes.x`, layerIds);
    validateDocParam(axes.y, ruleId, `${owner}.vec2_axes.y`, layerIds);
    return;
  }
  if (tag === "look_at") {
    const la = requireObject(obj.look_at, ruleId, `${owner}.look_at`);
    const axis = la.axis;
    if (typeof axis !== "string" || !LOOK_AT_AXES.has(axis)) {
      fail("6", `${owner}.look_at.axis`, `unknown LookAtAxis ${axis}`);
    }
    const target = requireInteger(la.target, "5", `${owner}.look_at.target`);
    if (!layerIds.has(target)) {
      fail("7", `${owner}.look_at.target`, `dangling layer id ${target}`);
    }
    return;
  }
  if (tag === "follow") {
    const fo = requireObject(obj.follow, ruleId, `${owner}.follow`);
    const target = requireInteger(fo.target, "5", `${owner}.follow.target`);
    if (!layerIds.has(target)) {
      fail("7", `${owner}.follow.target`, `dangling layer id ${target}`);
    }
    if (!Array.isArray(fo.offset) || fo.offset.length !== 2) {
      fail("4", `${owner}.follow.offset`, "expected length-2 array");
    }
    requireFinite(fo.offset[0], "5", `${owner}.follow.offset[0]`);
    requireFinite(fo.offset[1], "5", `${owner}.follow.offset[1]`);
  }
}

function validateTransform(transform, ruleId, owner, layerIds) {
  const obj = requireObject(transform, ruleId, owner);
  for (const key of TRANSFORM_DOC_PARAM_KEYS) {
    if (Object.hasOwn(obj, key)) {
      validateDocParam(obj[key], ruleId, `${owner}.${key}`, layerIds);
    }
  }
}

function validateEnvelope(envelope, ruleId, owner, layerIds, effectDefIds) {
  const env = requireObject(envelope, ruleId, owner);
  const blend = env.blend;
  if (typeof blend !== "string" || !BLEND_MODES.has(blend)) {
    fail("10", `${owner}.blend`, `unknown blend mode ${blend}`);
  }
  if (Object.hasOwn(env, "opacity")) {
    validateDocParam(env.opacity, ruleId, `${owner}.opacity`, layerIds);
  }
  if (Object.hasOwn(env, "transform")) {
    validateTransform(env.transform, ruleId, `${owner}.transform`, layerIds);
  }
  if (Object.hasOwn(env, "effects")) {
    if (!Array.isArray(env.effects)) {
      fail("4", `${owner}.effects`, "expected array");
    }
    for (let i = 0; i < env.effects.length; i += 1) {
      const effect = requireObject(
        env.effects[i],
        "4",
        `${owner}.effects[${i}]`,
      );
      const defId = requireInteger(
        effect.definition_id,
        "5",
        `${owner}.effects[${i}].definition_id`,
      );
      if (!effectDefIds.has(defId)) {
        fail(
          "7",
          `${owner}.effects[${i}].definition_id`,
          `dangling definition id ${defId}`,
        );
      }
    }
  }
}

function validateTrackItem(item, ruleId, owner, layerIds, effectDefIds) {
  const obj = requireObject(item, ruleId, owner);
  const kind = obj.kind;
  if (typeof kind !== "string" || !ITEM_KINDS.has(kind)) {
    fail("6", `${owner}.kind`, `unknown TrackItem.kind ${kind}`);
  }
  validateEnvelope(
    obj.envelope,
    ruleId,
    `${owner}.envelope`,
    layerIds,
    effectDefIds,
  );
  let children = obj.children;
  if (children === undefined || children === null) {
    return;
  }
  if (!Array.isArray(children)) {
    fail("4", `${owner}.children`, "expected array or null");
  }
  for (let i = 0; i < children.length; i += 1) {
    validateTrackItem(
      children[i],
      ruleId,
      `${owner}.children[${i}]`,
      layerIds,
      effectDefIds,
    );
  }
}

function validateEffectDefinitions(doc, layerIds) {
  const defs = doc.effect_definitions;
  if (!Array.isArray(defs)) {
    return new Map();
  }
  const effectDefIds = new Set();
  for (let i = 0; i < defs.length; i += 1) {
    const def = requireObject(defs[i], "4", `inputDoc.effect_definitions[${i}]`);
    const id = requireInteger(def.id, "5", `inputDoc.effect_definitions[${i}].id`);
    effectDefIds.add(id);
    if (typeof def.plugin_id !== "string") {
      fail("4", `inputDoc.effect_definitions[${i}].plugin_id`, "expected string");
    }
    if (Object.hasOwn(def, "params")) {
      const params = requireObject(
        def.params,
        "4",
        `inputDoc.effect_definitions[${i}].params`,
      );
      for (const key of Object.keys(params)) {
        validateDocParam(
          params[key],
          "4",
          `inputDoc.effect_definitions[${i}].params.${key}`,
          layerIds,
        );
        if (key === "opacity") {
          validateOpacityConstF64(
            params[key],
            "10",
            `inputDoc.effect_definitions[${i}].params.opacity`,
          );
        }
      }
    }
  }
  return effectDefIds;
}

function collectLayerIds(doc) {
  const layers = doc.layers?.entries;
  if (!Array.isArray(layers)) {
    fail("4", "inputDoc.layers.entries", "expected array");
  }
  const layerIds = new Set();
  const layerNames = new Map();
  for (let i = 0; i < layers.length; i += 1) {
    const layer = requireObject(layers[i], "4", `inputDoc.layers.entries[${i}]`);
    const id = requireInteger(layer.id, "5", `inputDoc.layers.entries[${i}].id`);
    if (typeof layer.name !== "string") {
      fail("4", `inputDoc.layers.entries[${i}].name`, "expected string");
    }
    layerIds.add(id);
    layerNames.set(id, layer.name);
  }
  return { layerIds, layerNames };
}

function walkDocumentItems(doc, layerIds, effectDefIds, visit) {
  const tracks = doc.tracks;
  if (!Array.isArray(tracks)) {
    fail("4", "inputDoc.tracks", "expected array");
  }
  const walk = (items, prefix) => {
    if (!Array.isArray(items)) {
      return;
    }
    for (let i = 0; i < items.length; i += 1) {
      const owner = `${prefix}[${i}]`;
      validateTrackItem(items[i], "4", owner, layerIds, effectDefIds);
      visit(items[i], owner);
      const children = items[i].children;
      if (Array.isArray(children)) {
        walk(children, `${owner}.children`);
      }
    }
  };
  for (let t = 0; t < tracks.length; t += 1) {
    const track = requireObject(tracks[t], "4", `inputDoc.tracks[${t}]`);
    walk(track.items, `inputDoc.tracks[${t}].items`);
  }
}

function validateF64Domain(domain, ruleId, owner, valueType) {
  const obj = requireObject(domain, ruleId, owner);
  if (valueType !== "F64") {
    fail("10", owner, "f64_domain on non-F64 parameter");
  }
  if (Object.hasOwn(obj, "min_inclusive")) {
    requireFinite(obj.min_inclusive, "5", `${owner}.min_inclusive`);
  }
  if (Object.hasOwn(obj, "max_inclusive")) {
    requireFinite(obj.max_inclusive, "10", `${owner}.max_inclusive`);
  }
  if (
    Object.hasOwn(obj, "min_inclusive")
    && Object.hasOwn(obj, "max_inclusive")
    && obj.min_inclusive > obj.max_inclusive
  ) {
    fail("10", owner, "min_inclusive > max_inclusive");
  }
  if (Object.hasOwn(obj, "integer") && typeof obj.integer !== "boolean") {
    fail("4", `${owner}.integer`, "expected boolean");
  }
}

function defaultWithinDomain(defaultValue, domain, ruleId, owner) {
  const tagged = validateDocValue(defaultValue, ruleId, owner);
  if (!Object.hasOwn(tagged, "F64") || !domain) {
    return;
  }
  const n = tagged.F64;
  if (Object.hasOwn(domain, "min_inclusive") && n < domain.min_inclusive) {
    fail("10", owner, "default below f64_domain.min_inclusive");
  }
  if (Object.hasOwn(domain, "max_inclusive") && n > domain.max_inclusive) {
    fail("10", owner, "default above f64_domain.max_inclusive");
  }
  if (domain.integer === true && !Number.isInteger(n)) {
    fail("10", owner, "default is not integer");
  }
}

function validateNodes(nodes) {
  if (!Array.isArray(nodes)) {
    fail("4", "wrapper.nodes", "expected array");
  }
  const seenNodeIds = new Set();
  const nodeById = new Map();
  for (let i = 0; i < nodes.length; i += 1) {
    const node = nodes[i];
    if (node === null || node === undefined) {
      fail("4", `wrapper.nodes[${i}]`, "expected object");
    }
    const obj = requireObject(node, "4", `wrapper.nodes[${i}]`);
    if (typeof obj.id !== "string") {
      fail("4", `wrapper.nodes[${i}].id`, "expected string");
    }
    if (seenNodeIds.has(obj.id)) {
      fail("8", `wrapper.nodes[${i}].id`, `duplicate node id ${obj.id}`);
    }
    seenNodeIds.add(obj.id);
    if (!Array.isArray(obj.params)) {
      fail("4", `wrapper.nodes[${i}].params`, "expected array");
    }
    const seenParamIds = new Set();
    const paramsOut = [];
    for (let p = 0; p < obj.params.length; p += 1) {
      const param = requireObject(
        obj.params[p],
        "4",
        `wrapper.nodes[${i}].params[${p}]`,
      );
      if (typeof param.id !== "string") {
        fail("11", `wrapper.nodes[${i}].params[${p}].id`, "missing id");
      }
      if (seenParamIds.has(param.id)) {
        fail(
          "8",
          `wrapper.nodes[${i}].params[${p}].id`,
          `duplicate param id ${param.id}`,
        );
      }
      seenParamIds.add(param.id);
      if (typeof param.value_type !== "string") {
        fail(
          "11",
          `wrapper.nodes[${i}].params[${p}].value_type`,
          "missing value_type",
        );
      }
      if (!PARAM_VALUE_TYPES.has(param.value_type)) {
        fail(
          "6",
          `wrapper.nodes[${i}].params[${p}].value_type`,
          `unknown value_type ${param.value_type}`,
        );
      }
      if (!Object.hasOwn(param, "default")) {
        fail("11", `wrapper.nodes[${i}].params[${p}].default`, "missing default");
      }
      const defaultTagged = validateDocValue(
        param.default,
        "4",
        `wrapper.nodes[${i}].params[${p}].default`,
      );
      const defaultTag = Object.keys(defaultTagged)[0];
      if (defaultTag !== param.value_type) {
        fail(
          "10",
          `wrapper.nodes[${i}].params[${p}].default`,
          `default tag ${defaultTag} does not match value_type ${param.value_type}`,
        );
      }
      let f64Domain;
      if (Object.hasOwn(param, "f64_domain")) {
        f64Domain = param.f64_domain;
        validateF64Domain(
          f64Domain,
          "10",
          `wrapper.nodes[${i}].params[${p}].f64_domain`,
          param.value_type,
        );
        defaultWithinDomain(
          param.default,
          f64Domain,
          "10",
          `wrapper.nodes[${i}].params[${p}].default`,
        );
      }
      paramsOut[paramsOut.length] = {
        id: param.id,
        value_type: param.value_type,
        default: param.default,
        f64_domain: f64Domain,
      };
    }
    nodeById.set(obj.id, paramsOut);
  }
  return nodeById;
}

function validateWrapper(input) {
  const root = requireObject(input, "4", "wrapper");
  const keys = Object.keys(root);
  for (const key of keys) {
    if (!WRAPPER_KEYS.has(key)) {
      fail("1", `wrapper.${key}`, `unexpected top-level key ${key}`);
    }
  }
  for (const required of WRAPPER_KEYS) {
    if (!Object.hasOwn(root, required)) {
      fail("2", `wrapper.${required}`, `missing required key ${required}`);
    }
  }
  const revision = root.fixtureRevision;
  if (typeof revision !== "number" || !Number.isInteger(revision)) {
    fail("3", "wrapper.fixtureRevision", "expected integer fixtureRevision");
  }
  if (revision !== FIXTURE_REVISION) {
    fail("3", "wrapper.fixtureRevision", `expected fixtureRevision ${FIXTURE_REVISION}`);
  }
  const doc = root["document"];
  if (doc === null || Array.isArray(doc) || typeof doc !== "object") {
    fail("4", "wrapper.document", "expected object document");
  }
  const nodes = validateNodes(root.nodes);
  const target = root.target;
  if (Array.isArray(target) || target === null || typeof target !== "object") {
    fail("4", "wrapper.target", "expected object target");
  }
  if (!Object.hasOwn(target, "layer_id")) {
    fail("11", "wrapper.target.layer_id", "missing layer_id");
  }
  const targetKeys = Object.keys(target);
  if (targetKeys.length !== 1 || targetKeys[0] !== "layer_id") {
    fail("4", "wrapper.target", "target must have exactly layer_id");
  }
  const layerId = target.layer_id;
  requireInteger(layerId, "5", "wrapper.target.layer_id");
  return { doc, nodes, layerId };
}

function findItemsForLayer(doc, layerId) {
  const matches = [];
  const tracks = doc.tracks;
  const visit = (item) => {
    const env = item.envelope;
    if (env && env.layer_id === layerId) {
      matches[matches.length] = item;
    }
  };
  const walk = (items) => {
    if (!Array.isArray(items)) {
      return;
    }
    for (const item of items) {
      visit(item);
      if (Array.isArray(item.children)) {
        walk(item.children);
      }
    }
  };
  for (const track of tracks) {
    if (track && Array.isArray(track.items)) {
      walk(track.items);
    }
  }
  return matches;
}

function projectParamDef(param) {
  const out = {
    id: param.id,
    value_type: param.value_type,
    default: cloneDocValue(param.default, "param.default"),
  };
  if (param.f64_domain !== undefined) {
    out.f64_domain = {
      min_inclusive: param.f64_domain.min_inclusive,
      max_inclusive: param.f64_domain.max_inclusive,
      integer: param.f64_domain.integer,
    };
  }
  return out;
}

function assertNoForbiddenOutputKeys(value, owner) {
  if (value === null || typeof value !== "object") {
    return;
  }
  if (Array.isArray(value)) {
    for (let i = 0; i < value.length; i += 1) {
      assertNoForbiddenOutputKeys(value[i], `${owner}[${i}]`);
    }
    return;
  }
  for (const key of Object.keys(value)) {
    if (FORBIDDEN_OUTPUT_KEYS.has(key)) {
      fail("14", `${owner}.${key}`, `forbidden output key ${key}`);
    }
    assertNoForbiddenOutputKeys(value[key], `${owner}.${key}`);
  }
}

function decodeInspectorReadModel(input) {
  const { doc, nodes, layerId } = validateWrapper(input);
  const { layerIds, layerNames } = collectLayerIds(doc);
  if (!layerIds.has(layerId)) {
    fail("7", "wrapper.target.layer_id", `dangling layer id ${layerId}`);
  }
  const effectDefIds = validateEffectDefinitions(doc, layerIds);
  walkDocumentItems(doc, layerIds, effectDefIds, () => {});

  const matches = findItemsForLayer(doc, layerId);
  if (matches.length === 0) {
    fail(
      "7-b",
      "wrapper.target.layer_id",
      `target layer ${layerId} has no track item`,
    );
  }
  if (matches.length > 1) {
    fail(
      "7-b",
      "wrapper.target.layer_id",
      `target layer ${layerId} has multiple track items`,
    );
  }
  const item = matches[0];
  const targetOut = {
    layer_id: layerId,
    layer_name: layerNames.get(layerId),
    item_kind: item.kind,
  };
  if (item.kind === "group") {
    const children = item.children;
    if (!Array.isArray(children)) {
      fail("4", "track item.children", "group must have children array");
    }
    targetOut.child_count = children.length;
  }

  const effectDefs = doc.effect_definitions ?? [];
  const effect_definitions = [];
  for (let d = 0; d < effectDefs.length; d += 1) {
    const def = effectDefs[d];
    const pluginId = def.plugin_id;
    const nodeParams = nodes.get(pluginId);
    if (!nodeParams) {
      fail(
        "9",
        `inputDoc.effect_definitions plugin_id ${pluginId}`,
        "unresolved plugin id",
      );
    }
    effect_definitions[d] = {
      definition_id: def.id,
      plugin_id: pluginId,
      params: nodeParams.map(projectParamDef),
    };
  }

  const output = {
    fixture_revision: FIXTURE_REVISION,
    target: targetOut,
    effect_definitions,
  };
  assertNoForbiddenOutputKeys(output, "output");
  return output;
}

decodeInspectorReadModel.validateOutput = assertNoForbiddenOutputKeys;

export { decodeInspectorReadModel };
