const PHASES = new Set(["start", "update", "commit", "cancel"]);

function requireExactKeys(value, expected, owner) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${owner} must be an object`);
  }
  const actual = Object.keys(value).sort();
  const wanted = [...expected].sort();
  if (actual.length !== wanted.length || actual.some((key, index) => key !== wanted[index])) {
    throw new TypeError(`${owner} has invalid keys`);
  }
}

function requireSafePositiveInteger(value, owner) {
  if (!Number.isSafeInteger(value) || value <= 0) {
    throw new TypeError(`${owner} must be a positive safe integer`);
  }
  return value;
}

function requireNormalizedValue(value, owner) {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0 || value > 1) {
    throw new TypeError(`${owner} must be a finite number in [0,1]`);
  }
  return value;
}

function requireFiniteScalar(value, owner) {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new TypeError(`${owner} must be a finite number`);
  }
  return value;
}

function exactPositionKeyProjection(readModel) {
  const position = readModel?.position;
  if (position === undefined || position?.kind !== "key") return null;
  if (
    !Number.isFinite(position.x)
    || !Number.isFinite(position.y)
  ) {
    throw new TypeError("Inspector Position key projection is invalid");
  }
  return Object.freeze({
    layer_id: readModel?.target?.layer_id,
    x: position.x,
    y: position.y,
  });
}

function exactActiveAmount(readModel) {
  const active = readModel?.active_effect;
  if (active === undefined) return null;
  if (
    !Number.isSafeInteger(active.layer_id)
    || !Number.isSafeInteger(active.effect_use_id)
    || !Number.isSafeInteger(active.definition_id)
    || active.plugin_id !== "core.filter.opacity"
    || active.effect_version !== 1
    || !Array.isArray(active.params)
    || active.params.length !== 1
  ) {
    throw new TypeError("Inspector active Effect identity is invalid");
  }
  const [param] = active.params;
  if (
    param.id !== "amount"
    || param.value_type !== "F64"
    || param.control_kind !== "F64"
    || param.f64_domain?.min_inclusive !== 0
    || param.f64_domain?.max_inclusive !== 1
    || param.f64_domain?.integer !== false
  ) {
    throw new TypeError("Inspector active amount contract is invalid");
  }
  return Object.freeze({
    layer_id: active.layer_id,
    effect_use_id: active.effect_use_id,
    definition_id: active.definition_id,
    plugin_id: active.plugin_id,
    effect_version: active.effect_version,
    param_id: param.id,
  });
}

function sameIdentity(left, right) {
  if (left === right) return true;
  if (left === null || right === null) return false;
  return Object.keys(left).every((key) => left[key] === right[key]);
}

function samePositionProjection(left, right) {
  if (left === right) return true;
  if (left === null || right === null) return false;
  return left.layer_id === right.layer_id
    && left.x === right.x
    && left.y === right.y;
}

export function createInspectorHostSender(postMessage) {
  if (typeof postMessage !== "function") {
    throw new TypeError("Inspector Host postMessage must be a function");
  }

  let identity = null;
  let activeSession = null;
  let nextSession = 1;
  let nextPositionKeySequence = 1;
  let positionProjection = null;
  let activePositionSession = null;
  let nextPositionSession = 1;

  const project = (readModel) => {
    const projectedIdentity = exactActiveAmount(readModel);
    if (!sameIdentity(identity, projectedIdentity)) {
      activeSession = null;
    }
    const projectedPosition = exactPositionKeyProjection(readModel);
    if (!samePositionProjection(positionProjection, projectedPosition)) {
      activePositionSession = null;
    }
    identity = projectedIdentity;
    positionProjection = projectedPosition;
  };

  const send = (event) => {
    if (identity === null) {
      throw new TypeError("Inspector gesture has no active amount target");
    }
    requireExactKeys(
      event,
      event?.phase === "cancel" ? ["phase", "paramId"] : ["phase", "paramId", "value"],
      "Inspector gesture",
    );
    if (!PHASES.has(event.phase)) {
      throw new TypeError("Inspector gesture phase is invalid");
    }
    if (event.paramId !== identity.param_id) {
      throw new TypeError("Inspector gesture parameter identity is invalid");
    }

    const normalizedValue = event.phase === "cancel"
      ? null
      : requireNormalizedValue(event.value, "Inspector gesture value");
    let session;
    let sequence;
    let nextActiveSession;
    if (event.phase === "start") {
      if (activeSession !== null) {
        throw new TypeError("Inspector gesture is already active");
      }
      requireSafePositiveInteger(nextSession, "Inspector gesture session");
      session = nextSession;
      sequence = 1;
      nextActiveSession = { session, sequence };
    } else {
      if (activeSession === null) {
        throw new TypeError("Inspector gesture is not active");
      }
      session = activeSession.session;
      sequence = activeSession.sequence + 1;
      requireSafePositiveInteger(sequence, "Inspector gesture sequence");
      nextActiveSession = { session, sequence };
    }

    const message = {
      kind: "effect-param-gesture",
      phase: event.phase,
      session,
      sequence,
      ...identity,
    };
    if (event.phase !== "cancel") {
      message.value = normalizedValue;
    }
    postMessage(JSON.stringify(message));
    if (event.phase === "commit" || event.phase === "cancel") {
      activeSession = null;
    } else {
      activeSession = nextActiveSession;
    }
    if (event.phase === "start") {
      nextSession += 1;
    }
  };

  const sendAddPositionKey = () => {
    const sequence = requireSafePositiveInteger(
      nextPositionKeySequence,
      "Inspector Position key sequence",
    );
    postMessage(JSON.stringify({ kind: "add-position-key", sequence }));
    nextPositionKeySequence = sequence === Number.MAX_SAFE_INTEGER ? null : sequence + 1;
  };

  const sendPositionKeyGesture = (event) => {
    if (positionProjection === null) {
      throw new TypeError("Inspector Position gesture has no active key");
    }
    requireExactKeys(
      event,
      event?.phase === "cancel" ? ["phase", "axis"] : ["phase", "axis", "value"],
      "Inspector Position gesture",
    );
    if (!PHASES.has(event.phase)) {
      throw new TypeError("Inspector Position gesture phase is invalid");
    }
    if (event.axis !== "x" && event.axis !== "y") {
      throw new TypeError("Inspector Position gesture axis is invalid");
    }
    const value = event.phase === "cancel"
      ? null
      : requireFiniteScalar(event.value, "Inspector Position gesture value");
    let session;
    let sequence;
    let nextActiveSession;
    if (event.phase === "start") {
      if (activePositionSession !== null) {
        throw new TypeError("Inspector Position gesture is already active");
      }
      requireSafePositiveInteger(nextPositionSession, "Inspector Position gesture session");
      session = nextPositionSession;
      sequence = 1;
      nextActiveSession = { session, sequence };
    } else {
      if (activePositionSession === null) {
        throw new TypeError("Inspector Position gesture is not active");
      }
      session = activePositionSession.session;
      sequence = activePositionSession.sequence + 1;
      requireSafePositiveInteger(sequence, "Inspector Position gesture sequence");
      nextActiveSession = { session, sequence };
    }

    const message = {
      kind: "position-key-value-gesture",
      phase: event.phase,
      session,
      sequence,
      axis: event.axis,
    };
    if (event.phase !== "cancel") message.value = value;
    postMessage(JSON.stringify(message));
    if (event.phase === "commit" || event.phase === "cancel") {
      activePositionSession = null;
    } else {
      activePositionSession = nextActiveSession;
    }
    if (event.phase === "start") nextPositionSession += 1;
  };

  return Object.freeze({
    project,
    send,
    sendAddPositionKey,
    sendPositionKeyGesture,
  });
}
