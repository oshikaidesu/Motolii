import React, { useEffect, useRef, useState } from "react";
import Konva from "konva";
import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { TransformControls } from "three/addons/controls/TransformControls.js";

const nodeState = (node) => ({
  id: node.name(), x: node.x(), y: node.y(), scaleX: node.scaleX(), scaleY: node.scaleY(), rotation: node.rotation(),
});

export function ObjectHandleProbe() {
  const hostRef = useRef(null);
  const runtimeRef = useRef(null);
  const [state, setState] = useState({ commits: 0, cancels: 0, selection: 1, zoom: 1, visualCssPx: 14, hitCssPx: 30 });

  useEffect(() => {
    const stage = new Konva.Stage({ container: hostRef.current, width: 640, height: 300 });
    const layer = new Konva.Layer();
    const objects = [
      new Konva.Rect({ name: "object-a", x: 80, y: 82, width: 120, height: 76, fill: "#ffcd70", draggable: true }),
      new Konva.Rect({ name: "object-b", x: 310, y: 118, width: 100, height: 92, fill: "#59c7ff", draggable: true }),
    ];
    const transformer = new Konva.Transformer({
      nodes: [objects[0]], anchorSize: 14, rotateAnchorOffset: 32, flipEnabled: false,
      rotationSnaps: [0, 45, 90, 135, 180, 225, 270, 315], rotationSnapTolerance: 6,
      boundBoxFunc: (oldBox, newBox) => Math.abs(newBox.width) < 24 || Math.abs(newBox.height) < 24 ? oldBox : newBox,
      anchorStyleFunc: (anchor) => anchor.hitStrokeWidth(16),
    });
    let selected = [objects[0]];
    let snapshot = [];
    let cancelled = false;
    let zoom = 1;
    let commits = 0;
    let cancels = 0;

    const publish = (patch = {}) => setState((value) => ({
      ...value, commits, cancels, selection: selected.length, zoom, visualCssPx: transformer.anchorSize() * zoom,
      hitCssPx: (transformer.anchorSize() + 16 / zoom) * zoom, objects: objects.map(nodeState), ...patch,
    }));
    const save = () => { snapshot = objects.map(nodeState); cancelled = false; };
    const restore = () => snapshot.forEach((saved, index) => objects[index].setAttrs({
      x: saved.x, y: saved.y, scaleX: saved.scaleX, scaleY: saved.scaleY, rotation: saved.rotation,
    }));
    const commit = (action) => {
      if (!cancelled) { commits += 1; publish({ lastAction: action }); }
    };
    const select = (nodes) => {
      selected = nodes;
      transformer.nodes(selected);
      publish();
    };
    objects.forEach((object) => {
      object.on("click tap", (event) => {
        const multi = event.evt.shiftKey || event.evt.metaKey || event.evt.ctrlKey;
        select(multi
          ? (selected.includes(object) ? selected.filter((node) => node !== object) : [...selected, object])
          : [object]);
      });
      object.on("dragstart", save);
      object.on("dragend", () => commit("move"));
    });
    transformer.on("transformstart", save);
    transformer.on("transformend", () => commit(transformer.getActiveAnchor() === "rotater" ? "rotate" : "scale"));
    const cancel = (event) => {
      const dragging = objects.find((object) => object.isDragging());
      if (event.key !== "Escape" || (!transformer.isTransforming() && !dragging)) return;
      cancelled = true;
      transformer.stopTransform();
      dragging?.stopDrag();
      restore();
      transformer.forceUpdate();
      layer.batchDraw();
      cancels += 1;
      publish({ lastAction: "cancel" });
    };
    document.addEventListener("keydown", cancel);
    layer.add(...objects, transformer);
    stage.add(layer);

    runtimeRef.current = {
      select: (count) => select(objects.slice(0, count)),
      reset: () => {
        zoom = 1;
        stage.scale({ x: 1, y: 1 });
        transformer.anchorSize(14);
        transformer.rotateAnchorOffset(32);
        transformer.borderStrokeWidth(1);
        transformer.getChildren().forEach((child) => child.hitStrokeWidth(16));
        objects[0].setAttrs({ x: 80, y: 82, scaleX: 1, scaleY: 1, rotation: 0 });
        objects[1].setAttrs({ x: 310, y: 118, scaleX: 1, scaleY: 1, rotation: 0 });
        commits = 0; cancels = 0;
        select([objects[0]]); transformer.forceUpdate(); layer.batchDraw(); publish();
      },
      zoom: (next) => {
        zoom = next;
        stage.scale({ x: zoom, y: zoom });
        transformer.anchorSize(14 / zoom);
        transformer.rotateAnchorOffset(32 / zoom);
        transformer.borderStrokeWidth(1 / zoom);
        transformer.getChildren().forEach((child) => child.hitStrokeWidth(16 / zoom));
        transformer.forceUpdate(); layer.batchDraw(); publish();
      },
      keyboard: (action) => {
        selected.forEach((object) => {
          if (action === "move") object.x(object.x() + 1);
          if (action === "scale") object.scale({ x: object.scaleX() * 1.1, y: object.scaleY() * 1.1 });
          if (action === "rotate") object.rotation(object.rotation() + 15);
        });
        commits += 1;
        transformer.forceUpdate(); layer.batchDraw(); publish({ lastAction: action });
      },
    };
    publish();
    return () => { document.removeEventListener("keydown", cancel); stage.destroy(); };
  }, []);

  return (
    <section className="handle-section">
      <h2>2D object handles — Konva Transformer</h2>
      <div className="handle-controls">
        <button data-testid="handles-single" onClick={() => runtimeRef.current.select(1)}>Single</button>
        <button data-testid="handles-multi" onClick={() => runtimeRef.current.select(2)}>Multi</button>
        <button data-testid="handles-zoom" onClick={() => runtimeRef.current.zoom(state.zoom === 1 ? 2 : 1)}>Zoom {state.zoom}×</button>
        <button data-testid="handles-reset" onClick={() => runtimeRef.current.reset()}>Reset</button>
      </div>
      <div ref={hostRef} className="object-handle-stage" data-testid="object-handle-stage" role="img"
        aria-label={`2D object transform surface; ${state.selection} object${state.selection === 1 ? "" : "s"} selected`} />
      <output data-testid="object-handle-state">{JSON.stringify(state)}</output>
      <div role="toolbar" aria-label="Accessible object transform handles" className="handle-a11y-proxy">
        <button aria-label="Move selected objects right by one unit" onClick={() => runtimeRef.current.keyboard("move")}>Move</button>
        <button aria-label="Scale selected objects up ten percent" onClick={() => runtimeRef.current.keyboard("scale")}>Scale</button>
        <button aria-label="Rotate selected objects clockwise fifteen degrees" onClick={() => runtimeRef.current.keyboard("rotate")}>Rotate</button>
      </div>
    </section>
  );
}

const spatialState = (object) => ({
  position: object.position.toArray(), rotation: object.rotation.toArray().slice(0, 3), scale: object.scale.toArray(),
});

export function SpatialGizmoProbe() {
  const hostRef = useRef(null);
  const runtimeRef = useRef(null);
  const [state, setState] = useState({ commits: 0, cancels: 0, mode: "translate", space: "world", cameraEnabled: true });

  useEffect(() => {
    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(640, 360, false);
    hostRef.current.append(renderer.domElement);
    const scene = new THREE.Scene();
    scene.background = new THREE.Color("#101720");
    const camera = new THREE.PerspectiveCamera(45, 640 / 360, 0.1, 100);
    camera.position.set(4, 3, 6);
    camera.lookAt(0, 0, 0);
    const object = new THREE.Mesh(new THREE.BoxGeometry(1.4, 1, 0.8), new THREE.MeshNormalMaterial());
    scene.add(object, new THREE.GridHelper(8, 8, 0x33465b, 0x263445));
    const orbit = new OrbitControls(camera, renderer.domElement);
    orbit.enableDamping = false;
    const controls = new TransformControls(camera, renderer.domElement);
    controls.setMode("translate");
    controls.setSpace("world");
    controls.setSize(0.85);
    controls.setTranslationSnap(0.25);
    controls.setRotationSnap(THREE.MathUtils.degToRad(15));
    controls.setScaleSnap(0.1);
    controls.attach(object);
    scene.add(controls.getHelper());
    let snapshot = spatialState(object);
    let cancelled = false;
    let commits = 0;
    let cancels = 0;
    let frame = 0;
    const publish = () => setState({
      commits, cancels, mode: controls.getMode(), space: controls.space, cameraEnabled: orbit.enabled,
      activeAxis: controls.axis, object: spatialState(object), helperSize: controls.size,
    });
    controls.addEventListener("mouseDown", () => {
      snapshot = spatialState(object); cancelled = false; orbit.enabled = false; publish();
    });
    controls.addEventListener("mouseUp", () => {
      orbit.enabled = true;
      if (!cancelled) commits += 1;
      publish();
    });
    const cancel = (event) => {
      if (event.key !== "Escape" || !controls.dragging) return;
      cancelled = true;
      controls.reset();
      object.position.fromArray(snapshot.position);
      object.rotation.set(...snapshot.rotation);
      object.scale.fromArray(snapshot.scale);
      cancels += 1;
      publish();
    };
    document.addEventListener("keydown", cancel);
    const render = () => { renderer.render(scene, camera); frame = requestAnimationFrame(render); };
    render();
    const project = (axis) => {
      const point = new THREE.Vector3(axis === "x" ? 0.72 : 0, axis === "y" ? 0.72 : 0, axis === "z" ? 0.72 : 0)
        .project(camera);
      return { x: (point.x * 0.5 + 0.5) * 640, y: (-point.y * 0.5 + 0.5) * 360 };
    };
    runtimeRef.current = {
      mode: (mode) => { controls.setMode(mode); publish(); },
      space: () => { controls.setSpace(controls.space === "world" ? "local" : "world"); publish(); },
      reset: () => { object.position.set(0, 0, 0); object.rotation.set(0, 0, 0); object.scale.set(1, 1, 1); commits = 0; cancels = 0; publish(); },
    };
    window.g09SpatialGizmo = { project };
    publish();
    return () => {
      document.removeEventListener("keydown", cancel); cancelAnimationFrame(frame); controls.dispose(); orbit.dispose();
      renderer.dispose(); renderer.domElement.remove(); delete window.g09SpatialGizmo;
    };
  }, []);

  return (
    <section className="handle-section">
      <h2>3D object gizmo — Three.js TransformControls</h2>
      <div className="handle-controls" role="toolbar" aria-label="3D transform modes">
        <button data-testid="gizmo-translate" onClick={() => runtimeRef.current.mode("translate")}>Translate</button>
        <button data-testid="gizmo-rotate" onClick={() => runtimeRef.current.mode("rotate")}>Rotate</button>
        <button data-testid="gizmo-scale" onClick={() => runtimeRef.current.mode("scale")}>Scale</button>
        <button data-testid="gizmo-space" onClick={() => runtimeRef.current.space()}>{state.space}</button>
        <button data-testid="gizmo-reset" onClick={() => runtimeRef.current.reset()}>Reset</button>
      </div>
      <div ref={hostRef} className="spatial-gizmo-stage" data-testid="spatial-gizmo-stage" role="img"
        aria-label={`3D object ${state.mode} gizmo in ${state.space} space`} />
      <output data-testid="spatial-gizmo-state">{JSON.stringify(state)}</output>
    </section>
  );
}
