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
      id={param}
      data-param={param}
      style={{ "--dial-shift": value * 2 }}
      aria-label={`${param === "intensity" ? "Intensity" : "Spread"}。無限目盛を左右dragして変更`}
      onPointerDown={(event) => {
        event.preventDefault();
        event.currentTarget.setPointerCapture(event.pointerId);
        onScrubStart(param, event.clientX, event.currentTarget);
      }}
      onPointerMove={(event) => {
        onScrubMove(param, event.clientX, event.currentTarget);
      }}
      onPointerUp={(event) => {
        onScrubEnd(param, event.currentTarget, triggerSettling);
      }}
      onPointerCancel={(event) => {
        onScrubCancel(param, event.currentTarget);
      }}
      onAnimationEnd={() => {
        settlingRef.current?.classList.remove("settling");
      }}
      onKeyDown={(event) => {
        if (!["ArrowLeft", "ArrowRight"].includes(event.key)) return;
        event.preventDefault();
        const step = event.shiftKey ? 10 : 1;
        const delta = event.key === "ArrowRight" ? step : -step;
        onScrubKey(param, delta, triggerSettling);
      }}
    >
      <span className="scrub-dial" aria-hidden="true" />
      <output id={`${param}-read`}>{`${value}%`}</output>
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
}) {
  const [, syncRender] = useReducer((n) => n + 1, 0);
  const scrubSessionRef = useRef(null);
  const toggledAutomationRef = useRef({ object: new Set(), effect: new Set() });

  useEffect(() => {
    toggledAutomationRef.current = { object: new Set(), effect: new Set() };
  }, [mode]);

  const bump = useCallback(() => {
    syncRender();
  }, []);

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

  const panelHead = <div className="panel-head">Inspector</div>;

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
    const objectRow = (param, label, valueContent, keys, extraClass = "") => {
      const on = state.automation[param];
      return (
        <div className="row" key={param}>
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
          </span>
          {valueContent}
          <ObjectAutoHint param={param} keys={keys} automation={state.automation} />
        </div>
      );
    };

    return (
      <aside className="inspector" id="inspector">
        {panelHead}
        <div className="section">
          <div className="section-title">
            SELECTED OBJECT <span />
          </div>
          <div className="identity">
            <div className="icon">G</div>
            <div>
              <b>{selectedObjectName}</b>
              <small>{selectedObjectKind}</small>
            </div>
          </div>
        </div>
        <div className="section">
          <div className="section-title">
            TRANSFORM <span>OBJECT</span>
          </div>
          {objectRow(
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
          {objectRow(
            "depth",
            "Depth Z",
            <span className="value" id="depth-value">0.180</span>,
            "1 KEY",
            " at-key",
          )}
          {objectRow(
            "scale",
            "Scale",
            <span className="value">1.000</span>,
            "",
          )}
          {objectRow(
            "rotation",
            "Rotation Z",
            <span className="value">0.000 rad</span>,
            "",
          )}
          {objectRow(
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
