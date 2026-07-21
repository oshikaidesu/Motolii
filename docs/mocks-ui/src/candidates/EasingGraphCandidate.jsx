import { useEffect, useMemo, useRef, useState } from "react";
import "./easing-graph-candidate.css";
import {
  ADVANCED_SPECS,
  PLOT,
  advancedPathPoints,
  clamp,
  makeInitialAdvancedParameters,
  pointFrom,
  snap,
  viewForOvershoot,
  xOf,
  yOf,
} from "./easing-graph-model.js";

const BASIC_CURVES = [
  {
    name: "Linear",
    description: "一定速度",
    thumbnail: "M3 21 L37 3",
    values: [0, 0, 1, 1],
  },
  {
    name: "Smooth",
    description: "なめらかな加減速",
    thumbnail: "M3 21 C14 21 16 3 37 3",
    values: [0.4, 0, 0.2, 1],
  },
  {
    name: "Ease In",
    description: "ゆっくり始まる",
    thumbnail: "M3 21 C24 21 27 3 37 3",
    values: [0.42, 0, 1, 1],
  },
  {
    name: "Ease Out",
    description: "ゆっくり止まる",
    thumbnail: "M3 21 C13 21 16 3 37 3",
    values: [0, 0, 0.58, 1],
  },
];

const ADVANCED_CURVES = [
  {
    name: "Bounce",
    description: "重力で落下し、終点で跳ね返る",
    thumbnail:
      "M3 21 C12 21 13 3 22 3 C27 3 27 13 31 13 C35 13 35 3 37 3",
  },
  {
    name: "Elastic",
    description: "終点を越えてバネ状に収束する",
    thumbnail: "M3 21 C11 21 12 -4 21 5 S30 3 37 3",
  },
  {
    name: "Cyclic",
    description: "Sine状に進行を反復する",
    thumbnail: "M3 21 C7 5 11 5 15 21 S23 37 27 21 S33 5 37 3",
  },
  {
    name: "Random",
    description: "Brownian motion状の制御された乱れ",
    thumbnail: "M3 21 L8 16 L13 19 L18 10 L23 14 L28 6 L33 9 L37 3",
  },
  {
    name: "Steps",
    description: "区間を離散的な段階へ量子化する",
    thumbnail: "M3 21 H10 V16 H17 V11 H24 V7 H31 V3 H37",
  },
  {
    name: "Elastic Steps",
    description: "各段階に弾性の跳ね返りを加える",
    thumbnail: "M3 21 H11 V13 H16 V17 H22 V5 H27 V9 H32 V3 H37",
  },
];

// AM実機と同じく、Timeline上のplayhead位置を点線縦線で示す。
const PLAYHEAD_U = 0.46;

function bezierPath(values, view) {
  const [x1, y1, x2, y2] = values;
  return `M${xOf(0).toFixed(1)},${yOf(0, view).toFixed(1)} C${xOf(x1).toFixed(
    1,
  )},${yOf(y1, view).toFixed(1)} ${xOf(x2).toFixed(1)},${yOf(y2, view).toFixed(
    1,
  )} ${xOf(1).toFixed(1)},${yOf(1, view).toFixed(1)}`;
}

function advancedPath(name, parameters, view, allowOvershoot) {
  return advancedPathPoints(name, parameters)
    .map((point, index) => {
      const value = allowOvershoot
        ? clamp(point.v, view.bottom, view.top)
        : clamp(point.v, 0, 1);
      return `${index === 0 ? "M" : "L"}${xOf(point.u).toFixed(1)},${yOf(
        value,
        view,
      ).toFixed(1)}`;
    })
    .join(" ");
}

function currentChannelKeys(intervalContext) {
  if (intervalContext) {
    const stack = [...document.querySelectorAll(".candidate-automation-stack")]
      .find(
        (entry) => entry.dataset.objectId === intervalContext.objectId,
      );
    const row = [...(stack?.querySelectorAll(".candidate-automation-row") ?? [])]
      .find((entry) => entry.dataset.channel === intervalContext.channel);
    return [...(row?.querySelectorAll(".candidate-automation-key") ?? [])].sort(
      (left, right) =>
        Number.parseFloat(left.style.left) -
        Number.parseFloat(right.style.left),
    );
  }
  const selectedBar = document.querySelector(".time-bar.selected");
  if (!selectedBar) return [];
  return [
    ...selectedBar.querySelectorAll(
      '.inline-key[data-channel="Intensity"]:not(.channel-hidden)',
    ),
  ].sort(
    (left, right) =>
      Number.parseFloat(left.style.left) - Number.parseFloat(right.style.left),
  );
}

function currentIntervalStart(intervalContext) {
  if (intervalContext) {
    return currentChannelKeys(intervalContext)[intervalContext.startIndex] ?? null;
  }
  return (
    document.querySelector(
      ".time-bar.selected .inline-key.key-selected",
    ) ?? currentChannelKeys()[0]
  );
}

function setUndoLabel(label) {
  const undo = document.querySelector("#undo-state");
  if (undo) undo.textContent = `Undo 1 · ${label}`;
}

function writeCurveToIntervalStart(target, curve) {
  target.dataset.easing = curve.label;
  target.dataset.curve = curve.serialized;
  if (curve.serialized.startsWith("interval:")) {
    target.dataset.interpolationKind = curve.label;
    target.dataset.previewParams = curve.previewParams ?? "";
  } else {
    delete target.dataset.interpolationKind;
    delete target.dataset.previewParams;
  }
}

function formatParameter(value, step) {
  return step >= 1 ? String(Math.round(value)) : value.toFixed(2);
}

export function EasingGraphCandidate({ intervalContext = null }) {
  const [menuOpen, setMenuOpen] = useState(false);
  const [clipboard, setClipboard] = useState(null);
  const [favorite, setFavorite] = useState("Smooth");
  const [actionStatus, setActionStatus] = useState("");
  const [overshootEnabled, setOvershootEnabled] = useState(false);
  const [basicCurve, setBasicCurve] = useState("Smooth");
  const [bezier, setBezier] = useState([0.4, 0, 0.2, 1]);
  const [advancedCurve, setAdvancedCurve] = useState(null);
  const [advancedParameters, setAdvancedParameters] = useState(
    makeInitialAdvancedParameters,
  );
  const [hoveredHandle, setHoveredHandle] = useState(null);
  const [drag, setDrag] = useState(null);
  // legacy adapterがReact描画を上書きした後にsubtreeを取り直すための世代番号。
  const [legacySyncTick, setLegacySyncTick] = useState(0);
  const intervalCount = useMemo(
    () => Math.max(0, currentChannelKeys(intervalContext).length - 1),
    [intervalContext, menuOpen],
  );

  const activeAdvancedParameters = advancedCurve
    ? advancedParameters[advancedCurve]
    : null;
  const activeSpec = advancedCurve ? ADVANCED_SPECS[advancedCurve] : null;
  // drag確定時にrender遅延で古いparameterを書かないよう、最新値をrefにも持つ。
  const latestParametersRef = useRef(advancedParameters);
  latestParametersRef.current = advancedParameters;

  // overshoot型(Elastic / Elastic Steps)だけが0..1の外へ曲線を描ける。
  // それ以外の型はOvershoot toggleの状態に関わらず常にkey値範囲へ拘束する。
  const typeOvershoots = activeSpec?.overshoots ?? false;
  const showOvershootCurve = typeOvershoots && overshootEnabled;

  // curve内容では座標写像を変えない。Overshoot ON時だけ最大可動域へ
  // 一度に切り替え、handle操作中の再フィットを起こさない。
  const view = viewForOvershoot(overshootEnabled);

  const currentGraphPath = advancedCurve
    ? advancedPath(
        advancedCurve,
        activeAdvancedParameters,
        view,
        showOvershootCurve,
      )
    : bezierPath(bezier, view);

  useEffect(() => {
    const keys = [
      ...document.querySelectorAll(
        ".inline-key, .candidate-automation-key",
      ),
    ];
    const current = new Set(currentChannelKeys(intervalContext));
    keys.forEach((key) => {
      const isCurrent = current.has(key);
      const channel =
        key.dataset.channel ??
        key.closest(".candidate-automation-row")?.dataset.channel ??
        intervalContext?.channel ??
        "Automation";
      key.classList.toggle("current-channel", isCurrent);
      key.classList.toggle("context-only", !isCurrent);
      key.dataset.keyContext = isCurrent ? "current" : "context";
      if (key.classList.contains("inline-key")) {
        key.setAttribute(
          "aria-label",
          `${channel} keyframe · ${
            isCurrent ? "current channel" : "context only"
          }`,
        );
      }
    });
    return () => {
      keys.forEach((key) => {
        key.classList.remove("current-channel", "context-only");
        delete key.dataset.keyContext;
      });
    };
  }, [intervalContext]);

  useEffect(() => {
    if (!drag) return undefined;
    const cancelWithEscape = (event) => {
      if (event.key !== "Escape") return;
      if (drag.kind === "bezier") setBezier(drag.startBezier);
      if (drag.kind === "advanced") {
        latestParametersRef.current = {
          ...latestParametersRef.current,
          [drag.curve]: drag.startParameters,
        };
        setAdvancedParameters((current) => ({
          ...current,
          [drag.curve]: drag.startParameters,
        }));
      }
      setDrag(null);
      setActionStatus("Handle dragをCancel · Document変更ゼロ");
    };
    window.addEventListener("keydown", cancelWithEscape);
    return () => window.removeEventListener("keydown", cancelWithEscape);
  }, [drag]);

  // legacy fixture scriptは#interval-easingや[data-curve]ボタンへ直接
  // handlerを張り、renderEasingGraph()が旧座標系(0..100)で#graph-curveや
  // #graph-handle-a/bを上書きする。旧HTMLは変更しない方針のため、該当
  // クリックの直後にworkspace subtreeをkey差し替えで再マウントし、React側の
  // 描画所有権を取り戻す(fixture adapter限定の措置。egui実装へは持ち込まない)。
  useEffect(() => {
    const reclaimAfterLegacyWrite = (event) => {
      if (
        event.target.closest?.("#interval-easing") ||
        event.target.closest?.("[data-curve]")
      ) {
        requestAnimationFrame(() => setLegacySyncTick((tick) => tick + 1));
      }
    };
    document.addEventListener("click", reclaimAfterLegacyWrite);
    return () =>
      document.removeEventListener("click", reclaimAfterLegacyWrite);
  }, []);

  // legacyの#open-curve-shelf handlerは候補に存在しない#curve-shelfを参照して
  // 例外になるため、legacy初期化後に無効化して候補側の説明表示へ差し替える。
  useEffect(() => {
    const timer = setTimeout(() => {
      const shelfButton = document.querySelector("#open-curve-shelf");
      if (shelfButton) shelfButton.onclick = null;
    }, 0);
    return () => clearTimeout(timer);
  }, []);

  // •••メニューは外側クリックで閉じる。
  useEffect(() => {
    if (!menuOpen) return undefined;
    const closeOnOutsidePress = (event) => {
      if (
        event.target.closest?.(".candidate-easing-menu") ||
        event.target.closest?.(".candidate-easing-more")
      ) {
        return;
      }
      setMenuOpen(false);
    };
    document.addEventListener("pointerdown", closeOnOutsidePress);
    return () =>
      document.removeEventListener("pointerdown", closeOnOutsidePress);
  }, [menuOpen]);

  function currentCurve() {
    if (advancedCurve) {
      return {
        label: advancedCurve,
        serialized: `interval:${advancedCurve}`,
        previewParams: JSON.stringify(activeAdvancedParameters),
      };
    }
    return {
      label: basicCurve,
      serialized: `(${bezier.map((value) => value.toFixed(2)).join(", ")})`,
    };
  }

  function toggleOvershoot() {
    const outside = bezier[1] < 0 || bezier[1] > 1 || bezier[3] < 0 || bezier[3] > 1;
    if (overshootEnabled && !advancedCurve && outside) {
      return;
    }
    setOvershootEnabled((value) => !value);
    setActionStatus("");
  }

  function copyCurve() {
    const curve = currentCurve();
    setClipboard(curve);
    setActionStatus(`${curve.label}をCopy · Document / Undo変更なし`);
  }

  function pasteCurrent() {
    if (!clipboard) return;
    const target = currentIntervalStart(intervalContext);
    if (!target) {
      setActionStatus("対象区間がありません。変更ゼロ。");
      return;
    }
    writeCurveToIntervalStart(target, clipboard);
    setUndoLabel("Paste Easing");
    setActionStatus("現在区間へPaste · 1 Undo");
  }

  function pasteAll() {
    if (!clipboard) return;
    const keys = currentChannelKeys(intervalContext);
    if (keys.length < 2) {
      setActionStatus("対象区間がありません。変更ゼロ。");
      return;
    }
    keys
      .slice(0, -1)
      .forEach((key) => writeCurveToIntervalStart(key, clipboard));
    setUndoLabel("Paste Easing to channel");
    setActionStatus(
      `${keys.length - 1} intervals · IntensityだけへPaste · 1 Undo`,
    );
  }

  function applyBasicCurve(curve) {
    setBasicCurve(curve.name);
    setBezier([...curve.values]);
    setAdvancedCurve(null);
    const target = currentIntervalStart(intervalContext);
    if (target) {
      delete target.dataset.interpolationKind;
      delete target.dataset.previewParams;
    }
  }

  function applyAdvancedCurve(name) {
    const target = currentIntervalStart(intervalContext);
    if (!target) {
      setActionStatus("対象区間がありません。変更ゼロ。");
      return;
    }
    const parameters = advancedParameters[name];
    writeCurveToIntervalStart(target, {
      label: name,
      serialized: `interval:${name}`,
      previewParams: JSON.stringify(parameters),
    });
    setAdvancedCurve(name);
    // overshoot型は明示ON、それ以外の型へ切り替えたら明示OFFへ戻す。
    // (前の型のON状態が漏れて非overshoot型が範囲外表示になるのを防ぐ)
    const wantsOvershoot = ADVANCED_SPECS[name].overshoots === true;
    setOvershootEnabled(wantsOvershoot);
    const overshootNote = wantsOvershoot
      ? `${name}固有のOvershootを明示ON · `
      : overshootEnabled
        ? "Overshoot OFF(この型は0〜1に留まる) · "
        : "";
    setUndoLabel("Interval Easing");
    setActionStatus(
      `${overshootNote}${name}を現在区間へ適用 · keyframe数・時刻・値は不変 · 1 Undo`,
    );
  }

  // fraction側は意図的にclampしない: drag中のviewは固定なので、view上端で
  // parameterが頭打ちにならないようpointerがsvg外へ出ても外挿する。
  // parameterの可動域は各handleのapplyが確定する。
  function pointerCurvePoint(event, pointerView) {
    const svg = event.currentTarget.ownerSVGElement;
    const rect = svg.getBoundingClientRect();
    const point = pointFrom(
      (event.clientX - rect.left) / rect.width,
      (event.clientY - rect.top) / rect.height,
      pointerView,
    );
    return {
      u: clamp(point.u, -0.1, 1.1),
      v: clamp(point.v, -0.6, 2.4),
    };
  }

  function beginBezierDrag(event, handle) {
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    setDrag({ kind: "bezier", handle, startBezier: [...bezier], view });
    setActionStatus(
      `Bezier handle ${handle === "a" ? "1" : "2"} · Preview中`,
    );
  }

  function beginAdvancedDrag(event, descriptor) {
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    setDrag({
      kind: "advanced",
      curve: advancedCurve,
      descriptor,
      startParameters: { ...activeAdvancedParameters },
      view,
    });
    setActionStatus(`${descriptor.label} · Preview中`);
  }

  function moveHandle(event) {
    if (!drag) return;
    if (drag.kind === "bezier") {
      const point = pointerCurvePoint(event, drag.view);
      const next = [...bezier];
      const x = clamp(point.u, 0, 1);
      // 手動handleはAM-KG-07どおり、Overshoot OFFの間はkey値範囲へ拘束する。
      const y = overshootEnabled
        ? clamp(point.v, -0.35, 1.35)
        : clamp(point.v, 0, 1);
      if (drag.handle === "a") {
        next[0] = x;
        next[1] = y;
      } else {
        next[2] = x;
        next[3] = y;
      }
      setBasicCurve("Custom");
      setBezier(next);
      return;
    }

    const point = pointerCurvePoint(event, drag.view);
    const nextParameters = drag.descriptor.apply(
      latestParametersRef.current[drag.curve],
      point,
      drag,
    );
    latestParametersRef.current = {
      ...latestParametersRef.current,
      [drag.curve]: nextParameters,
    };
    setAdvancedParameters((current) => ({
      ...current,
      [drag.curve]: nextParameters,
    }));
  }

  function commitHandleDrag(event) {
    if (!drag) return;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    const target = currentIntervalStart(intervalContext);
    if (target && drag.kind === "bezier") {
      writeCurveToIntervalStart(target, {
        label: "Custom curve",
        serialized: `(${bezier.map((value) => value.toFixed(2)).join(", ")})`,
      });
    }
    if (target && drag.kind === "advanced") {
      writeCurveToIntervalStart(target, {
        label: drag.curve,
        serialized: `interval:${drag.curve}`,
        previewParams: JSON.stringify(latestParametersRef.current[drag.curve]),
      });
    }
    setUndoLabel("Interval Easing");
    setActionStatus(
      `${drag.kind === "advanced" ? drag.descriptor.label : "Bezier"}を確定 · 1 Undo`,
    );
    setDrag(null);
  }

  function cancelHandleDrag() {
    if (!drag) return;
    if (drag.kind === "bezier") setBezier(drag.startBezier);
    if (drag.kind === "advanced") {
      latestParametersRef.current = {
        ...latestParametersRef.current,
        [drag.curve]: drag.startParameters,
      };
      setAdvancedParameters((current) => ({
        ...current,
        [drag.curve]: drag.startParameters,
      }));
    }
    setDrag(null);
    setActionStatus("Handle dragをCancel · Document変更ゼロ");
  }

  function nudgeAdvancedHandle(descriptor, axis, direction) {
    if (!advancedCurve) return;
    // 単軸handleでは対応しない軸の矢印keyを無視する(誤操作防止)。
    const param = descriptor.params.find((entry) => entry.axis === axis);
    if (!param) return;
    const nextParameters = {
      ...activeAdvancedParameters,
      [param.key]: snap(
        activeAdvancedParameters[param.key] + param.step * direction,
        param.min,
        param.max,
        param.step,
      ),
    };
    setAdvancedParameters((current) => ({
      ...current,
      [advancedCurve]: nextParameters,
    }));
    const target = currentIntervalStart(intervalContext);
    if (target) {
      writeCurveToIntervalStart(target, {
        label: advancedCurve,
        serialized: `interval:${advancedCurve}`,
        previewParams: JSON.stringify(nextParameters),
      });
      setUndoLabel("Interval Easing");
    }
  }

  return (
    <aside
      className="easing-panel candidate-easing-panel"
      id="easing-panel"
      aria-hidden="true"
      aria-label="Interval Easing Editor"
      data-react-surface="easing-graph"
    >
      <div
        className="easing-head candidate-easing-head"
        data-info="Interval Easing|現在の区間を編集 · 1 Undo|Esc"
      >
        <b id="easing-target">
          {intervalContext
            ? `${intervalContext.objectName} · ${intervalContext.channel}`
            : "Pulse rings · Intensity"}
        </b>
        <button
          className="candidate-easing-more"
          aria-label="Easing actions"
          aria-expanded={menuOpen}
          onClick={(event) => {
            event.stopPropagation();
            setMenuOpen((value) => !value);
          }}
        >
          •••
        </button>
        <button id="close-easing" aria-label="Interval Easing Editorを閉じる">
          ×
        </button>
        {menuOpen ? (
          <div
            className="candidate-easing-menu"
            role="menu"
            aria-label="Easing actions"
          >
            <button
              role="menuitem"
              onClick={() => {
                const curve = currentCurve();
                setFavorite(curve.label);
                setActionStatus(
                  `${curve.label}をFavoriteへ設定 · User setting / Undoなし`,
                );
              }}
            >
              ◎ Set current as Favorite
            </button>
            <button role="menuitem" onClick={copyCurve}>
              Copy Curve
            </button>
            <button role="menuitem" disabled={!clipboard} onClick={pasteCurrent}>
              Paste Curve
            </button>
            <button
              role="menuitem"
              disabled={!clipboard || intervalCount === 0}
              onClick={pasteAll}
            >
              Paste to all in current channel · {intervalCount}
            </button>
            <p role="status" aria-live="polite">
              {actionStatus}
            </p>
          </div>
        ) : null}
      </div>

      <div className="easing-workspace" key={legacySyncTick}>
        <div className="candidate-easing-picker">
          <small>BEZIER</small>
          <div className="easing-curves" aria-label="Bezier curve shapes">
            {BASIC_CURVES.map((curve) => (
              <button
                key={curve.name}
                className={!advancedCurve && basicCurve === curve.name ? "on" : ""}
                data-curve={curve.name}
                data-favorite={favorite === curve.name ? "true" : undefined}
                aria-label={`${curve.name}${
                  favorite === curve.name ? " · Favorite" : ""
                }`}
                data-info={`${curve.name}|${curve.description}`}
                onClick={() => applyBasicCurve(curve)}
              >
                <svg viewBox="0 0 40 24" aria-hidden="true">
                  <path d={curve.thumbnail} />
                </svg>
                <span>{curve.name}</span>
              </button>
            ))}
            <button
              id="open-curve-shelf"
              aria-label="My curves"
              data-info="My curves|保存した形を開く|Esc"
            >
              <svg viewBox="0 0 40 24" aria-hidden="true">
                <path d="M5 19 C13 19 15 7 35 7 M5 14 C14 14 18 3 35 3 M5 22 H35" />
              </svg>
              <span>MY</span>
            </button>
          </div>
          <small>ADVANCED · INTERVAL</small>
          <div
            className="easing-curves easing-advanced-curves"
            aria-label="Advanced interval interpolation shapes"
          >
            {ADVANCED_CURVES.map((curve) => (
              <button
                key={curve.name}
                className={advancedCurve === curve.name ? "on" : ""}
                data-advanced-curve={curve.name}
                aria-label={
                  curve.name === "Cyclic" ? "Cyclic · Sine" : curve.name
                }
                data-info={`${curve.name}|${curve.description}|既存key間の補間だけを変更`}
                onClick={() => applyAdvancedCurve(curve.name)}
              >
                <svg viewBox="0 0 40 24" aria-hidden="true">
                  <path d={curve.thumbnail} />
                </svg>
                <span>
                  {curve.name === "Cyclic" ? "CYCLIC / SIN" : curve.name}
                </span>
              </button>
            ))}
          </div>
        </div>

        <div
          className={`easing-graph ${
            advancedCurve ? "advanced-active" : "bezier-active"
          }`}
          aria-label="Easing curve graph. X is source time, Y is remapped animation time, and slope represents speed."
          data-view-top={view.top.toFixed(2)}
          data-view-bottom={view.bottom.toFixed(2)}
        >
          <button
            className="candidate-overshoot-toggle"
            type="button"
            aria-label="Overshoot mode"
            aria-pressed={overshootEnabled}
            onClick={toggleOvershoot}
          >
            <svg viewBox="0 0 30 24" aria-hidden="true">
              <path className="candidate-overshoot-frame" d="M3 7 H27 V21 H3 Z" />
              <path
                className="candidate-overshoot-curve"
                d={
                  overshootEnabled
                    ? "M4 20 C9 20 10 3 16 3 C21 3 21 10 27 8"
                    : "M4 20 C10 20 13 8 26 8"
                }
              />
            </svg>
          </button>
          <svg
            viewBox={`0 0 ${PLOT.width} ${PLOT.height}`}
            preserveAspectRatio="none"
          >
            <line
              className="graph-guide"
              x1={xOf(0)}
              y1={yOf(1, view)}
              x2={xOf(1)}
              y2={yOf(1, view)}
            />
            <line
              className="graph-guide"
              x1={xOf(0)}
              y1={yOf(0, view)}
              x2={xOf(1)}
              y2={yOf(0, view)}
            />
            <line
              className="graph-playhead"
              x1={xOf(PLAYHEAD_U)}
              y1={yOf(1, view)}
              x2={xOf(PLAYHEAD_U)}
              y2={yOf(0, view)}
            />
            {advancedCurve
              ? activeSpec.decorations(activeAdvancedParameters).map(
                  (decoration) => (
                    <line
                      key={decoration.key}
                      className={decoration.className}
                      x1={xOf(decoration.from.u)}
                      y1={yOf(decoration.from.v, view)}
                      x2={xOf(decoration.to.u)}
                      y2={yOf(decoration.to.v, view)}
                    />
                  ),
                )
              : null}
            {advancedCurve
              ? (activeSpec.staticMarkers ?? []).map((marker) => (
                  <circle
                    key={marker.key}
                    className="candidate-graph-handle advanced-handle anchor-handle static-marker"
                    aria-hidden="true"
                    cx={xOf(marker.u)}
                    cy={yOf(marker.v, view)}
                    r="13"
                  />
                ))
              : null}
            {!advancedCurve ? (
              <>
                <line
                  className="graph-stem"
                  id="graph-stem-a"
                  x1={xOf(0)}
                  y1={yOf(0, view)}
                  x2={xOf(bezier[0])}
                  y2={yOf(bezier[1], view)}
                />
                <line
                  className="graph-stem"
                  id="graph-stem-b"
                  x1={xOf(1)}
                  y1={yOf(1, view)}
                  x2={xOf(bezier[2])}
                  y2={yOf(bezier[3], view)}
                />
              </>
            ) : null}
            <path
              className="graph-curve"
              id="graph-curve"
              d={currentGraphPath}
            />
            {!advancedCurve ? (
              <>
                {[
                  { key: "a", index: 1, x: bezier[0], y: bezier[1] },
                  { key: "b", index: 2, x: bezier[2], y: bezier[3] },
                ].map((handle) => (
                  <g key={handle.key}>
                    <circle
                      className="candidate-graph-handle"
                      aria-hidden="true"
                      cx={xOf(handle.x)}
                      cy={yOf(handle.y, view)}
                      r="12"
                    />
                    <circle
                      className="graph-handle-hit"
                      id={`graph-handle-${handle.key}`}
                      cx={xOf(handle.x)}
                      cy={yOf(handle.y, view)}
                      r="26"
                      aria-label={`Bezier handle ${handle.index}: ${handle.x.toFixed(
                        2,
                      )}, ${handle.y.toFixed(2)}`}
                      onPointerDown={(event) =>
                        beginBezierDrag(event, handle.key)
                      }
                      onPointerMove={moveHandle}
                      onPointerUp={commitHandleDrag}
                      onPointerCancel={cancelHandleDrag}
                    />
                  </g>
                ))}
              </>
            ) : (
              activeSpec.handles.map((descriptor) => {
                const anchor = descriptor.anchor(activeAdvancedParameters);
                const singleParam =
                  descriptor.params.length === 1 ? descriptor.params[0] : null;
                const isEngaged =
                  hoveredHandle === descriptor.id ||
                  (drag?.kind === "advanced" &&
                    drag.descriptor.id === descriptor.id);
                const cx = xOf(anchor.u);
                const cy = yOf(clamp(anchor.v, view.bottom, view.top), view);
                return (
                  <g key={descriptor.id}>
                    <circle
                      className={`candidate-graph-handle advanced-handle ${
                        descriptor.kind === "anchor" ? "anchor-handle" : ""
                      }${isEngaged ? " engaged" : ""}`}
                      aria-hidden="true"
                      cx={cx}
                      cy={cy}
                      r="13"
                    />
                    <circle
                      className="graph-handle-hit"
                      cx={cx}
                      cy={cy}
                      r="26"
                      role={descriptor.role}
                      tabIndex="0"
                      aria-label={`${descriptor.label} handle`}
                      aria-valuemin={singleParam ? singleParam.min : undefined}
                      aria-valuemax={singleParam ? singleParam.max : undefined}
                      aria-valuenow={
                        singleParam
                          ? activeAdvancedParameters[singleParam.key]
                          : undefined
                      }
                      onKeyDown={(event) => {
                        const moves = {
                          ArrowRight: ["x", 1],
                          ArrowLeft: ["x", -1],
                          ArrowUp: ["y", 1],
                          ArrowDown: ["y", -1],
                        };
                        const move = moves[event.key];
                        if (!move) return;
                        event.preventDefault();
                        nudgeAdvancedHandle(descriptor, move[0], move[1]);
                      }}
                      onPointerEnter={() => setHoveredHandle(descriptor.id)}
                      onPointerLeave={() => setHoveredHandle(null)}
                      onPointerDown={(event) =>
                        beginAdvancedDrag(event, descriptor)
                      }
                      onPointerMove={moveHandle}
                      onPointerUp={commitHandleDrag}
                      onPointerCancel={cancelHandleDrag}
                    />
                  </g>
                );
              })
            )}
            <circle
              className="graph-end"
              cx={xOf(0)}
              cy={yOf(0, view)}
              r="6"
            />
            <circle
              className="graph-end"
              cx={xOf(1)}
              cy={yOf(1, view)}
              r="6"
            />
          </svg>
        </div>

        <div
          className="easing-values"
          id="easing-values"
          aria-label={
            advancedCurve
              ? `${advancedCurve} ${
                  activeSpec.confirmed ? "confirmed" : "inferred"
                } handle parameters`
              : "Bezier handle values"
          }
        >
          {advancedCurve ? (
            <>
              <small className="candidate-parameter-source">
                {activeSpec.confirmed ? "USER-CONFIRMED" : "INFERRED"}
              </small>
              {activeSpec.handles.map((descriptor) => (
                <span
                  className={`handle-value candidate-advanced-value${
                    hoveredHandle === descriptor.id ||
                    (drag?.kind === "advanced" &&
                      drag.descriptor.id === descriptor.id)
                      ? " active-handle"
                      : ""
                  }`}
                  key={descriptor.id}
                  data-handle-id={descriptor.id}
                >
                  <small>{descriptor.label}</small>
                  {descriptor.params.map((param) => (
                    <b key={param.key} data-handle-parameter={param.key}>
                      {param.label}{" "}
                      {formatParameter(
                        activeAdvancedParameters[param.key],
                        param.step,
                      )}
                    </b>
                  ))}
                </span>
              ))}
            </>
          ) : (
            <>
              <span className="handle-value" data-info="Handle 1|X / Y">
                <span>1</span>
                <small>x</small>
                <b id="ease-x1">{bezier[0].toFixed(2)}</b>
                <small>y</small>
                <b id="ease-y1">{bezier[1].toFixed(2)}</b>
              </span>
              <span className="handle-value" data-info="Handle 2|X / Y">
                <span>2</span>
                <small>x</small>
                <b id="ease-x2">{bezier[2].toFixed(2)}</b>
                <small>y</small>
                <b id="ease-y2">{bezier[3].toFixed(2)}</b>
              </span>
            </>
          )}
        </div>
      </div>
    </aside>
  );
}
