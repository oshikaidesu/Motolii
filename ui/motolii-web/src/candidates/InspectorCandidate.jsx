import {
  createContext,
  useCallback,
  useEffect,
  useReducer,
  useRef,
} from "react";
import "./inspector-candidate.css";

export const InspectorContext = createContext(null);

const OBJECT_AUTOMATION_LABEL = {
  position: "Position",
  depth: "Depth Z",
  scale: "Scale",
  rotation: "Rotation Z",
  opacity: "Opacity",
};

function objectAutomationAriaLabel(param, on, toggled) {
  if (toggled) {
    return `${param} automation ${on ? "on" : "off"}`;
  }
  return `${OBJECT_AUTOMATION_LABEL[param]} automation ${on ? "on" : "off"}`;
}

function effectAutomationAriaLabel(param, on, toggled) {
  if (toggled) {
    return `${param} automation ${on ? "on" : "off"}`;
  }
  const label = param === "intensity" ? "Intensity" : "Spread";
  return `${label} automation ${on ? "on" : "off"}`;
}

function ObjectAutoHint({ param, keys, automation }) {
  const on = automation[param];
  return (
    <span className="scrub-hint object-auto" id={`${param}-object-state`}>
      <i>{on ? "AUTO ON" : "AUTO OFF"}</i>
      {keys ? <i>{keys}</i> : null}
    </span>
  );
}

function ScrubControl({
  param,
  value,
  displayValue = `${value}%`,
  displayName,
  controlId = param,
  readOnly = false,
  onScrubStart,
  onScrubMove,
  onScrubEnd,
  onScrubCancel,
  onScrubKey,
}) {
  const settlingRef = useRef(null);

  const triggerSettling = (kick) => {
    const el = settlingRef.current;
    if (!el) return;
    el.style.setProperty("--dial-kick", `${kick}px`);
    el.classList.remove("settling");
    void el.offsetWidth;
    el.classList.add("settling");
  };

  return (
    <button
      ref={settlingRef}
      className="scrub"
      id={controlId}
      data-param={param}
      style={{ "--dial-shift": value * 2 }}
      aria-label={
        readOnly
          ? `${displayName ?? param} read-only`
          : `${displayName ?? (param === "intensity" ? "Intensity" : param === "amount" ? "Amount" : "Spread")}。無限目盛を左右dragして変更`
      }
      aria-readonly={readOnly || undefined}
      disabled={readOnly || undefined}
      onPointerDown={readOnly ? undefined : (event) => {
        event.preventDefault();
        event.currentTarget.setPointerCapture(event.pointerId);
        onScrubStart(param, event.clientX, event.currentTarget);
      }}
      onPointerMove={readOnly ? undefined : (event) => {
        onScrubMove(param, event.clientX, event.currentTarget);
      }}
      onPointerUp={readOnly ? undefined : (event) => {
        onScrubEnd(param, event.currentTarget, triggerSettling);
      }}
      onPointerCancel={readOnly ? undefined : (event) => {
        onScrubCancel(param, event.currentTarget);
      }}
      onAnimationEnd={() => {
        settlingRef.current?.classList.remove("settling");
      }}
      onKeyDown={readOnly ? undefined : (event) => {
        if (!["ArrowLeft", "ArrowRight"].includes(event.key)) return;
        event.preventDefault();
        const step = event.shiftKey ? 10 : 1;
        const delta = event.key === "ArrowRight" ? step : -step;
        onScrubKey(param, delta, triggerSettling, event.currentTarget);
      }}
    >
      <span className="scrub-dial" aria-hidden="true" />
      <output id={`${param}-read`}>{displayValue}</output>
    </button>
  );
}

function EffectScrubRow({
  param,
  label,
  value,
  automation,
  toggledEffect,
  onToggleAutomation,
  scrubProps,
}) {
  const on = automation[param];
  return (
    <div className="row">
      <span className="param-label">
        {label}{" "}
        <button
          className={`automation-mark ${on ? "on" : ""}`}
          data-automation={param}
          aria-pressed={on}
          aria-label={effectAutomationAriaLabel(
            param,
            on,
            toggledEffect.has(param),
          )}
          onClick={() => onToggleAutomation(param)}
        />
      </span>
      <ScrubControl param={param} value={value} {...scrubProps} />
      <span className="scrub-hint" id={`${param}-auto-state`}>
        {on ? "AUTO ON" : "AUTO OFF"}
      </span>
    </div>
  );
}

function DevInfoInstalled() {
  return (
    <details className="dev-info">
      <summary>Developer info</summary>
      <div className="row">
        <label>Package</label>
        <span className="value">Vism (.vism)</span>
        <span />
      </div>
      <div className="row">
        <label>Identity</label>
        <span className="value">demo.echo-bloom</span>
        <span />
      </div>
      <div className="lifecycle">
        Preview / Export<span>SAME EVALUATION</span>Undo / Save<span>PROJECT</span>
        Cache / Resource<span>HOST</span>
      </div>
    </details>
  );
}

function DevInfoEffectFocused() {
  return (
    <details className="dev-info">
      <summary>Developer info</summary>
      <div className="row">
        <label>Package</label>
        <span className="value">Vism (.vism)</span>
        <span />
      </div>
      <div className="row">
        <label>Identity</label>
        <span className="value">demo.echo-bloom</span>
        <span />
      </div>
    </details>
  );
}

function DevInfoDiscover() {
  return (
    <details className="dev-info">
      <summary>Developer info</summary>
      <div className="row">
        <label>Package</label>
        <span className="value">Vism (.vism)</span>
        <span />
      </div>
      <div className="row">
        <label>Identity</label>
        <span className="value">demo.glyph-current</span>
        <span />
      </div>
      <div className="lifecycle">
        Project change<span>NONE</span>Code execution<span>NONE</span>Standard
        panel<span>AVAILABLE AFTER ADD</span>
      </div>
    </details>
  );
}

function DevInfoBlocked() {
  return (
    <details className="dev-info">
      <summary>Developer info</summary>
      <div className="row">
        <label>Package</label>
        <span className="value">Vism (.vism)</span>
        <span />
      </div>
      <div className="row">
        <label>Identity</label>
        <span className="value">demo.fold-field</span>
        <span />
      </div>
    </details>
  );
}

function DevInfoMissing() {
  return (
    <details className="dev-info">
      <summary>Developer info</summary>
      <div className="row">
        <label>Package</label>
        <span className="value">Vism (.vism)</span>
        <span />
      </div>
      <div className="row">
        <label>Identity</label>
        <span className="value">demo.ribbon-array</span>
        <span />
      </div>
    </details>
  );
}

export function InspectorCandidate({
  mode,
  effectFocused,
  state,
  setUndo,
  status,
  projectAutomation,
  applyScrubValue,
  setSurface,
  syncColorBook,
  setStageTool,
  renderPluginHistory,
  inspectorReadModel,
  onEffectParamGesture,
  onPositionKeyGesture,
  onAddPositionKey,
}) {
  const [, syncRender] = useReducer((n) => n + 1, 0);
  const scrubSessionRef = useRef(null);
  const productScrubSessionRef = useRef(null);
  const productDisplayValuesRef = useRef(new Map());
  const positionScrubSessionRef = useRef(null);
  const positionDisplayValuesRef = useRef(new Map());
  const toggledAutomationRef = useRef({ object: new Set(), effect: new Set() });
  const activeEffect = inspectorReadModel?.active_effect;

  useEffect(() => {
    toggledAutomationRef.current = { object: new Set(), effect: new Set() };
  }, [mode]);

  const bump = useCallback(() => {
    syncRender();
  }, []);

  useEffect(() => {
    productScrubSessionRef.current = null;
    productDisplayValuesRef.current.clear();
  }, [activeEffect?.effect_use_id]);

  const projectedPosition = inspectorReadModel?.position;
  useEffect(() => {
    positionScrubSessionRef.current = null;
    positionDisplayValuesRef.current.clear();
  }, [
    inspectorReadModel?.target?.layer_id,
    projectedPosition?.kind,
    projectedPosition?.x,
    projectedPosition?.y,
  ]);

  const toggleObjectAutomation = (param) => {
    toggledAutomationRef.current.object.add(param);
    const on = !state.automation[param];
    state.automation[param] = on;
    setUndo(`${param} automation`);
    status("Automation", `${param} ${on ? "ON" : "OFF"}`, "⌘Z");
    bump();
  };

  const toggleEffectAutomation = (param) => {
    toggledAutomationRef.current.effect.add(param);
    const on = !state.automation[param];
    state.automation[param] = on;
    projectAutomation(param);
    setUndo(`${param} automation`);
    status("Automation", `${param} ${on ? "ON" : "OFF"}`, "⌘Z");
    bump();
  };

  const onScrubStart = (param, clientX, control) => {
    projectAutomation(param);
    control.classList.add("dragging");
    scrubSessionRef.current = { param, x: clientX, value: state[param], control };
    status("Echo Bloom", `${param} · 数値を左右dragしてPreview`, "Release / Esc");
  };

  const onScrubMove = (param, clientX, control) => {
    const session = scrubSessionRef.current;
    if (!session || session.param !== param) return;
    applyScrubValue(param, session.value + (clientX - session.x));
    status("Echo Bloom", `${param} ${state[param]}% · Preview`, "Release / Esc");
    bump();
  };

  const cancelScrub = useCallback((control) => {
    const session = scrubSessionRef.current;
    if (!session || session.control !== control) return;
    const param = session.param;
    applyScrubValue(param, session.value);
    scrubSessionRef.current = null;
    control.classList.remove("dragging");
    status("Echo Bloom", `${param} · Cancel · Document変更ゼロ`, "");
    bump();
  }, [applyScrubValue, status, bump]);

  const onScrubEnd = (param, control, triggerSettling) => {
    const session = scrubSessionRef.current;
    if (!session || session.param !== param) return;
    const delta = state[param] - session.value;
    const changed = delta !== 0;
    scrubSessionRef.current = null;
    control.classList.remove("dragging");
    if (changed) {
      triggerSettling(delta > 0 ? 3 : -3);
      setUndo(param === "intensity" ? "Intensity" : "Spread");
      status("Echo Bloom", `${param} ${state[param]}% · 1 Undo`, "⌘Z");
    } else {
      status("Echo Bloom", `${param} · No change · Undoなし`, "");
    }
    bump();
  };

  const onScrubKey = (param, delta, triggerSettling) => {
    projectAutomation(param);
    applyScrubValue(param, state[param] + delta);
    triggerSettling(delta > 0 ? 3 : -3);
    setUndo(param === "intensity" ? "Intensity" : "Spread");
    status("Echo Bloom", `${param} ${state[param]}% · keyboard step`, "⌘Z");
    bump();
  };

  useEffect(() => {
    const onKeyDown = (event) => {
      if (event.key !== "Escape") return;
      const session = scrubSessionRef.current;
      if (!session?.control) return;
      cancelScrub(session.control);
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [cancelScrub]);

  const scrubProps = {
    onScrubStart,
    onScrubMove,
    onScrubEnd,
    onScrubCancel: (param, control) => cancelScrub(control),
    onScrubKey,
  };

  const productParam = (paramId) => activeEffect?.params.find(
    (param) => param.id === paramId,
  );
  const productValue = (param) => (
    productDisplayValuesRef.current.get(param.id) ?? param.current.const.F64
  );
  const clampProductValue = (param, value) => Math.min(
    param.f64_domain.max_inclusive,
    Math.max(param.f64_domain.min_inclusive, value),
  );
  const emitProductGesture = (phase, param, value) => {
    const event = phase === "cancel"
      ? { phase, paramId: param.id }
      : { phase, paramId: param.id, value };
    onEffectParamGesture(event);
  };
  const onProductScrubStart = (paramId, clientX, control) => {
    const param = productParam(paramId);
    if (!param || productScrubSessionRef.current !== null) return;
    const value = productValue(param);
    const session = {
      paramId,
      clientX,
      initialValue: value,
      value,
      control,
    };
    emitProductGesture("start", param, value);
    productScrubSessionRef.current = session;
    control.classList.add("dragging");
  };
  const onProductScrubMove = (paramId, clientX) => {
    const session = productScrubSessionRef.current;
    const param = productParam(paramId);
    if (!param || !session || session.paramId !== paramId) return;
    const value = clampProductValue(
      param,
      session.initialValue + (clientX - session.clientX) / 100,
    );
    if (value === session.value) return;
    emitProductGesture("update", param, value);
    session.value = value;
    productDisplayValuesRef.current.set(paramId, value);
    bump();
  };
  const cancelProductScrub = useCallback(() => {
    const session = productScrubSessionRef.current;
    const param = activeEffect?.params.find(({ id }) => id === session?.paramId);
    if (!session || !param || typeof onEffectParamGesture !== "function") return;
    onEffectParamGesture({ phase: "cancel", paramId: param.id });
    productScrubSessionRef.current = null;
    productDisplayValuesRef.current.delete(param.id);
    session.control.classList.remove("dragging");
    bump();
  }, [activeEffect, onEffectParamGesture, bump]);
  const onProductScrubEnd = (paramId, control, triggerSettling) => {
    const session = productScrubSessionRef.current;
    const param = productParam(paramId);
    if (!param || !session || session.paramId !== paramId) return;
    emitProductGesture("commit", param, session.value);
    productScrubSessionRef.current = null;
    control.classList.remove("dragging");
    const delta = session.value - session.initialValue;
    if (delta !== 0) triggerSettling(delta > 0 ? 3 : -3);
    bump();
  };
  const onProductScrubKey = (paramId, delta, triggerSettling, control) => {
    const param = productParam(paramId);
    if (!param || productScrubSessionRef.current !== null) return;
    const initialValue = productValue(param);
    const value = clampProductValue(param, initialValue + delta / 100);
    emitProductGesture("start", param, initialValue);
    productScrubSessionRef.current = {
      paramId,
      clientX: 0,
      initialValue,
      value,
      control,
    };
    emitProductGesture("commit", param, value);
    productScrubSessionRef.current = null;
    productDisplayValuesRef.current.set(paramId, value);
    if (value !== initialValue) triggerSettling(value > initialValue ? 3 : -3);
    bump();
  };

  useEffect(() => {
    if (typeof onEffectParamGesture !== "function") return undefined;
    const onKeyDown = (event) => {
      if (event.key === "Escape") cancelProductScrub();
    };
    window.addEventListener("blur", cancelProductScrub);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("blur", cancelProductScrub);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [onEffectParamGesture, cancelProductScrub]);

  const positionValue = (axis) => (
    positionDisplayValuesRef.current.get(axis) ?? projectedPosition?.[axis]
  );
  const emitPositionGesture = (phase, axis, value) => {
    if (typeof onPositionKeyGesture !== "function") return;
    const event = phase === "cancel"
      ? { phase, axis }
      : { phase, axis, value };
    onPositionKeyGesture(event);
  };
  const onPositionScrubStart = (axis, clientX, control) => {
    if (
      projectedPosition?.kind !== "key"
      || positionScrubSessionRef.current !== null
      || !Number.isFinite(positionValue(axis))
    ) return;
    const value = positionValue(axis);
    emitPositionGesture("start", axis, value);
    positionScrubSessionRef.current = {
      axis,
      clientX,
      initialValue: value,
      value,
      control,
    };
    control.classList.add("dragging");
  };
  const onPositionScrubMove = (axis, clientX) => {
    const session = positionScrubSessionRef.current;
    if (!session || session.axis !== axis) return;
    const value = session.initialValue + (clientX - session.clientX) / 100;
    if (!Number.isFinite(value) || value === session.value) return;
    emitPositionGesture("update", axis, value);
    session.value = value;
    positionDisplayValuesRef.current.set(axis, value);
    bump();
  };
  const cancelPositionScrub = useCallback(() => {
    const session = positionScrubSessionRef.current;
    if (!session || typeof onPositionKeyGesture !== "function") return;
    emitPositionGesture("cancel", session.axis);
    positionScrubSessionRef.current = null;
    positionDisplayValuesRef.current.clear();
    session.control.classList.remove("dragging");
    bump();
  }, [onPositionKeyGesture, bump]);
  const onPositionScrubEnd = (axis, control, triggerSettling) => {
    const session = positionScrubSessionRef.current;
    if (!session || session.axis !== axis) return;
    emitPositionGesture("commit", axis, session.value);
    positionScrubSessionRef.current = null;
    control.classList.remove("dragging");
    const delta = session.value - session.initialValue;
    if (delta !== 0) triggerSettling(delta > 0 ? 3 : -3);
    bump();
  };
  const onPositionScrubKey = (axis, delta, triggerSettling, control) => {
    if (
      projectedPosition?.kind !== "key"
      || positionScrubSessionRef.current !== null
      || !Number.isFinite(positionValue(axis))
    ) return;
    const initialValue = positionValue(axis);
    const value = initialValue + delta / 100;
    if (!Number.isFinite(value)) return;
    emitPositionGesture("start", axis, initialValue);
    positionScrubSessionRef.current = {
      axis,
      clientX: 0,
      initialValue,
      value,
      control,
    };
    emitPositionGesture("commit", axis, value);
    positionScrubSessionRef.current = null;
    positionDisplayValuesRef.current.set(axis, value);
    if (value !== initialValue) triggerSettling(value > initialValue ? 3 : -3);
    bump();
  };

  useEffect(() => {
    if (typeof onPositionKeyGesture !== "function") return undefined;
    window.addEventListener("blur", cancelPositionScrub);
    const onKeyDown = (event) => {
      if (event.key === "Escape") cancelPositionScrub();
    };
    document.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("blur", cancelPositionScrub);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [onPositionKeyGesture, cancelPositionScrub]);

  const panelHead = <div className="panel-head">Inspector</div>;
  const selectedObjectName =
    inspectorReadModel === undefined
      ? "Pulse rings"
      : inspectorReadModel.target.layer_name;
  const selectedObjectKind =
    inspectorReadModel === undefined
      ? "Group · 1 child"
      : inspectorReadModel.target.item_kind === "group"
        ? `Group · ${inspectorReadModel.target.child_count} ${
            inspectorReadModel.target.child_count === 1 ? "child" : "children"
          }`
        : "Clip";
  const targetIdentity = (
    <div className="identity">
      <div className="icon">G</div>
      <div>
        <b>{selectedObjectName}</b>
        <small>{selectedObjectKind}</small>
      </div>
    </div>
  );
  const activeEffectSection = activeEffect === undefined ? null : (
    <div className="section" data-effect-use-id={activeEffect.effect_use_id}>
      <div className="section-title">
        {activeEffect.plugin_id} <span>V{activeEffect.effect_version}</span>
      </div>
      {activeEffect.params.map((param) => {
        const controlId = `effect-use-${activeEffect.effect_use_id}-${param.id}`;
        const interactive = typeof onEffectParamGesture === "function";
        return (
          <div className="row" key={controlId}>
            <label htmlFor={controlId}>{param.id}</label>
            <ScrubControl
              param={param.id}
              controlId={controlId}
              value={productValue(param) * 100}
              readOnly={!interactive}
              onScrubStart={interactive ? onProductScrubStart : undefined}
              onScrubMove={interactive ? onProductScrubMove : undefined}
              onScrubEnd={interactive ? onProductScrubEnd : undefined}
              onScrubCancel={interactive ? (_param, _control) => cancelProductScrub() : undefined}
              onScrubKey={interactive ? onProductScrubKey : undefined}
            />
            <span className="tag">{param.control_kind}</span>
          </div>
        );
      })}
    </div>
  );
  const objectRow = (key, labelContent, valueContent, hintContent) => (
    <div className="row" key={key}>
      {labelContent}
      {valueContent}
      {hintContent}
    </div>
  );

  if (mode === undefined && inspectorReadModel !== undefined) {
    const position = inspectorReadModel.position;
    const positionEditable = position?.kind === "key"
      && typeof onPositionKeyGesture === "function";
    const positionAxisControl = (axis, label) => (
      <ScrubControl
        param={`position-${axis}`}
        displayName={`Position ${label}`}
        controlId={`position-${axis}`}
        value={positionValue(axis)}
        displayValue={Number(positionValue(axis)).toFixed(3)}
        readOnly={!positionEditable}
        onScrubStart={positionEditable ? onPositionScrubStart : undefined}
        onScrubMove={positionEditable ? onPositionScrubMove : undefined}
        onScrubEnd={positionEditable ? onPositionScrubEnd : undefined}
        onScrubCancel={positionEditable ? (_axis, control) => cancelPositionScrub(control) : undefined}
        onScrubKey={positionEditable ? onPositionScrubKey : undefined}
      />
    );
    const positionRow = position === undefined ? null : objectRow(
      "position",
      <label>Position</label>,
      position.kind === "key" ? (
        <span className="value axis-pack editable-position">
          {positionAxisControl("x", "X")}
          {positionAxisControl("y", "Y")}
        </span>
      ) : position.kind === "const" ? (
        <span className="value axis-pack">
          <i><b>X</b> {position.x}</i>
          <i><b>Y</b> {position.y}</i>
        </span>
      ) : <span className="value">animated</span>,
      typeof onAddPositionKey === "function" ? (
        <button
          type="button"
          className="automation-mark"
          aria-label="Add Position Key"
          onClick={onAddPositionKey}
        >
          <span aria-hidden="true">◇</span>
        </button>
      ) : <span />,
    );
    return (
      <aside className="inspector" id="inspector">
        {panelHead}
        <div className="section">{targetIdentity}</div>
        {positionRow}
        {activeEffectSection}
      </aside>
    );
  }

  if (mode === "installed" && effectFocused) {
    return (
      <aside className="inspector" id="inspector">
        {panelHead}
        <div className="section">
          <div className="section-title">
            EDITING EFFECT <span>ON OBJECT</span>
          </div>
          <div className="identity">
            <div className="icon">◎</div>
            <div>
              <b>Echo Bloom</b>
              <small>Pulse rings · Effect</small>
            </div>
          </div>
        </div>
        <div className="section">
          <div className="section-title">
            ECHO BLOOM <span>HOST PANEL</span>
          </div>
          <p className="effect-parameter-description" id="effect-parameter-description">
            Layered light pulses that follow the selected object. Adjust Intensity
            and Spread while watching the Stage.
          </p>
          <div className="row">
            <label>Input</label>
            <span className="value">Pulse rings composite</span>
            <span className="tag">TEXTURE</span>
          </div>
          <EffectScrubRow
            param="intensity"
            label="Intensity"
            value={state.intensity}
            automation={state.automation}
            toggledEffect={toggledAutomationRef.current.effect}
            onToggleAutomation={toggleEffectAutomation}
            scrubProps={scrubProps}
          />
          <EffectScrubRow
            param="spread"
            label="Spread"
            value={state.spread}
            automation={state.automation}
            toggledEffect={toggledAutomationRef.current.effect}
            onToggleAutomation={toggleEffectAutomation}
            scrubProps={scrubProps}
          />
          <div className="row">
            <label>Blend</label>
            <span className="value">Screen</span>
            <span />
          </div>
        </div>
        <DevInfoEffectFocused />
      </aside>
    );
  }

  if (mode === "installed") {
    const installedObjectRow = (param, label, valueContent, keys, extraClass = "") => {
      const on = state.automation[param];
      return objectRow(
        param,
        <span className="param-label">
          {label}{" "}
          <button
            className={`automation-mark ${on ? "on" : ""}${extraClass}`}
            data-object-automation={param}
            aria-pressed={on}
            aria-label={objectAutomationAriaLabel(
              param,
              on,
              toggledAutomationRef.current.object.has(param),
            )}
            onClick={() => toggleObjectAutomation(param)}
          />
        </span>,
        valueContent,
        <ObjectAutoHint param={param} keys={keys} automation={state.automation} />,
      );
    };

    return (
      <aside className="inspector" id="inspector">
        {panelHead}
        <div className="section">
          <div className="section-title">
            SELECTED OBJECT <span />
          </div>
          {targetIdentity}
        </div>
        <div className="section">
          <div className="section-title">
            TRANSFORM <span>OBJECT</span>
          </div>
          {installedObjectRow(
            "position",
            "Position",
            <span className="value axis-pack">
              <i>
                <b>X</b> 0.124
              </i>
              <i>
                <b>Y</b> −0.082
              </i>
            </span>,
            "2 KEYS",
          )}
          {installedObjectRow(
            "depth",
            "Depth Z",
            <span className="value" id="depth-value">0.180</span>,
            "1 KEY",
            " at-key",
          )}
          {installedObjectRow(
            "scale",
            "Scale",
            <span className="value">1.000</span>,
            "",
          )}
          {installedObjectRow(
            "rotation",
            "Rotation Z",
            <span className="value">0.000 rad</span>,
            "",
          )}
          {installedObjectRow(
            "opacity",
            "Opacity",
            <span className="value">100%</span>,
            "2 KEYS",
          )}
        </div>
        <div className="section">
          <div className="section-title">
            APPEARANCE <span>OBJECT</span>
          </div>
          <div className="row">
            <label>Fill</label>
            <button
              className="color-chip"
              data-color-channel="Fill"
              data-label={state.appliedFill}
              style={{ "--chip": state.appliedFill }}
              aria-label="FillをColor Bookから選ぶ"
              onClick={() => {
                state.colorChannel = "Fill";
                state.selectedColor = state.appliedFill;
                syncColorBook();
                setSurface("colors", true);
                status("Color Book", `${state.colorChannel} · Preview`, "Esc");
              }}
            />
            <span className="tag">COLOR</span>
          </div>
          <div className="row">
            <label>Stroke</label>
            <button
              className="color-chip"
              data-color-channel="Stroke"
              data-label={state.appliedStroke}
              style={{ "--chip": state.appliedStroke }}
              aria-label="StrokeをColor Bookから選ぶ"
              onClick={() => {
                state.colorChannel = "Stroke";
                state.selectedColor = state.appliedStroke;
                syncColorBook();
                setSurface("colors", true);
                status("Color Book", `${state.colorChannel} · Preview`, "Esc");
              }}
            />
            <span className="tag">COLOR</span>
          </div>
        </div>
        <div className="section">
          <div className="section-title">
            GROUP COMPOSITION <span>EDIT SPACE</span>
          </div>
          <div className="row">
            <label>Z Occlusion</label>
            <span className="segmented">
              <span>OFF / Stack</span>
              <span className="on">ON / Group Z</span>
            </span>
            <span className="tag">Z</span>
          </div>
          <div className="row">
            <label>Composite</label>
            <span className="value">Child → Group bake point</span>
            <span />
          </div>
          <div className="row">
            <label>Link</label>
            <button
              className="value link-value"
              id="link-target"
              onClick={() => setStageTool("connect")}
            >
              <i />
              Position → target
            </button>
            <span className="tag">TYPED</span>
          </div>
        </div>
        <div className="section">
          <div className="section-title">
            DRIVER <span>2 ROUTES</span>
          </div>
          <div className="row">
            <label>Audio Low</label>
            <span className="driver-mini">
              <svg viewBox="0 0 180 18" preserveAspectRatio="none">
                <path d="M0 15 L14 13 27 16 42 4 55 11 72 8 89 15 106 5 124 12 144 3 160 11 180 7" />
              </svg>
            </span>
            <span className="tag">LIVE</span>
          </div>
        </div>
        <div className="section">
          <div className="section-title">
            APPLIED PLUGINS <span>＋</span>
          </div>
          <div className="applied-plugin">
            <span className="grip">::</span>
            <span className="plugin-mini">◎</span>
            <span>
              <b>Echo Bloom</b>
              <small>IN → Effect → OUT · selected</small>
            </span>
          </div>
        </div>
        <div className="section">
          <div className="section-title">
            ECHO BLOOM <span>HOST PANEL</span>
          </div>
          <div className="row">
            <label>Input</label>
            <span className="value">Pulse rings composite</span>
            <span className="tag">TEXTURE</span>
          </div>
          <EffectScrubRow
            param="intensity"
            label="Intensity"
            value={state.intensity}
            automation={state.automation}
            toggledEffect={toggledAutomationRef.current.effect}
            onToggleAutomation={toggleEffectAutomation}
            scrubProps={scrubProps}
          />
          <EffectScrubRow
            param="spread"
            label="Spread"
            value={state.spread}
            automation={state.automation}
            toggledEffect={toggledAutomationRef.current.effect}
            onToggleAutomation={toggleEffectAutomation}
            scrubProps={scrubProps}
          />
          <div className="row">
            <label>Blend</label>
            <span className="value">Screen</span>
            <span />
          </div>
        </div>
        <DevInfoInstalled />
      </aside>
    );
  }

  if (mode === "discover") {
    return (
      <aside className="inspector" id="inspector">
        {panelHead}
        <div className="section">
          <div className="section-title">
            DISCOVERY <span>NOT IN PROJECT</span>
          </div>
          <div className="identity">
            <div className="icon">字</div>
            <div>
              <b>Glyph Current</b>
              <small>Generator plugin · flowing type</small>
            </div>
          </div>
          <div className="action-strip">
            <button
              className="btn quiet"
              id="preview-vism"
              onClick={() =>
                status("Glyph Current", "Preview · Project変更ゼロ", "Esc")
              }
            >
              Preview
            </button>
            <button
              className="btn primary"
              id="add-vism"
              onClick={() => {
                state.pluginHistory = [
                  "discover",
                  ...state.pluginHistory.filter((entry) => entry !== "discover"),
                ];
                renderPluginHistory();
                setUndo("Add Glyph Current");
                status(
                  "Glyph Current",
                  "Project instanceを追加 · 1 Undo",
                  "⌘Z",
                );
              }}
            >
              Add to selected object
            </button>
          </div>
        </div>
        <DevInfoDiscover />
      </aside>
    );
  }

  if (mode === "blocked") {
    return (
      <aside className="inspector" id="inspector">
        {panelHead}
        <div className="section">
          <div className="section-title">
            DISCOVERY{" "}
            <span style={{ color: "var(--warning)" }}>UNAVAILABLE</span>
          </div>
          <div className="identity">
            <div className="icon">◇</div>
            <div>
              <b>Fold Field</b>
              <small>Effect plugin · local file</small>
            </div>
          </div>
          <div className="notice error">
            <strong>このHostでは評価できません</strong>
            <br />
            要求された能力が未対応です。近い既存Effectへ置換せず、非互換理由を表示します。
          </div>
          <div className="lifecycle">
            Project change<span>NONE</span>Install<span>NOT STARTED</span>
            Fallback<span>REFUSED</span>
          </div>
        </div>
        <div className="action-strip">
          <button
            className="btn quiet"
            id="inspect-reason"
            onClick={() =>
              status(
                "Unsupported capability",
                "要求: Feedback buffer / Host: 未対応",
                "Esc",
              )
            }
          >
            Inspect reason
          </button>
          <button className="btn" disabled>
            Add
          </button>
        </div>
        <DevInfoBlocked />
      </aside>
    );
  }

  return (
    <aside className="inspector" id="inspector">
      {panelHead}
      <div className="section">
        <div className="section-title">
          PROJECT INSTANCE{" "}
          <span style={{ color: "var(--warning)" }}>MISSING</span>
        </div>
        <div className="identity">
          <div
            className="icon"
            style={{ borderColor: "var(--warning)", color: "var(--warning)" }}
          >
            ?
          </div>
          <div>
            <b>Ribbon Array</b>
            <small>Plugin unavailable · Project instance retained</small>
          </div>
        </div>
        <div className="notice error">
          <strong>必要なプラグインを評価できません</strong>
          <br />
          identity、version要求、instance payloadを保持しています。欠落中はpayloadを解釈して似た設定へ変換しません。
        </div>
        <div className="lifecycle">
          Project open<span>SUCCEEDED</span>Unrelated edit<span>AVAILABLE</span>
          Required export<span>REFUSED</span>Payload<span>RETAINED</span>
        </div>
        <div className="action-strip">
          <button
            className="btn quiet"
            id="review-recovery"
            onClick={() => {
              // archived source に recovery 用の named command が無いため、legacy DOM 操作だけ残す。
              document.querySelector("#recovery")?.classList.add("open");
              status("Recovery candidate", "候補を照合 · installなし", "Esc");
            }}
          >
            Review recovery
          </button>
        </div>
      </div>
      <div className="section">
        <div className="section-title">
          STANDARD TRANSFORM <span>EDITABLE</span>
        </div>
        <div className="row">
          <label>Position</label>
          <span className="value">X 0.00 · Y 0.00</span>
          <span />
        </div>
        <div className="row">
          <label>Scale</label>
          <span className="value">100%</span>
          <span />
        </div>
      </div>
      <DevInfoMissing />
    </aside>
  );
}
