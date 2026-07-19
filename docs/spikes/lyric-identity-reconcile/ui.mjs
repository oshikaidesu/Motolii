import {
  createState,
  deleteRange,
  evaluateState,
  insertText,
  reconcileWholeText,
  replaceRange,
  stateText,
  withOverride,
} from "./reconcile.mjs";

const RUN_OVERRIDE = {
  offsetY: -36,
  visualScale: 1.4,
  timing: { mode: "pinned", value: 4 },
};

const stage = document.querySelector("#stage");
const timeline = document.querySelector("#timeline");
const identityTable = document.querySelector("#identity-table");
const reviewList = document.querySelector("#review-list");
const explanation = document.querySelector("#explanation");
const status = document.querySelector("#status");
const seedInput = document.querySelector("#seed");
const intervalInput = document.querySelector("#interval");

let state;

function preparedState() {
  return withOverride(createState("夜を走る"), "g3", RUN_OVERRIDE);
}

function describeCurrent(evaluated) {
  const road = evaluated.find((unit) => unit.grapheme === "道");
  const run = evaluated.find((unit) => unit.grapheme === "走");
  if (road && run) {
    return `
      <strong>道 = ${road.id} · AUTO ${road.start}f</strong>
      <span>新しい文字なので親Sequenceへ参加。Random Inも新IDから決定。</span>
      <strong>走 = ${run.id} · ${run.timingMode.toUpperCase()} ${run.start}f</strong>
      <span>同じIDのため、上方向offset・140%・Pinned Timingを保持。</span>
    `;
  }
  if (run) {
    return `
      <strong>走 = ${run.id} · ${run.timingMode.toUpperCase()} ${run.start}f</strong>
      <span>この文字だけ上へ移動し、Visual Scale 140%。親はRandom In / Interval ${intervalInput.value}f。</span>
    `;
  }
  return `
    <strong>手動調整は別文字へ移動していません</strong>
    <span>${state.needsReview.length}件をNeeds Reviewへ隔離。新しい文字は親SequenceのAuto値です。</span>
  `;
}

function renderStage(evaluated) {
  stage.replaceChildren();
  const centerX = Math.max(70, (stage.clientWidth - Math.max(0, evaluated.length - 1) * 76) / 2);
  const baseY = 185;

  for (const unit of evaluated) {
    const finalX = centerX + unit.final.x;
    const finalY = baseY + unit.final.y;
    const ghost = document.createElement("span");
    ghost.className = "glyph ghost";
    ghost.textContent = unit.grapheme;
    ghost.dataset.id = unit.id;
    ghost.style.left = `${finalX + unit.inPose.x}px`;
    ghost.style.top = `${finalY + unit.inPose.y}px`;
    ghost.style.transform = `translate(-50%, -50%) rotate(${unit.inPose.rotation}deg) scale(${unit.inPose.scale})`;
    stage.append(ghost);

    const final = document.createElement("span");
    final.className = "glyph final";
    if (unit.override) final.classList.add("overridden");
    final.textContent = unit.grapheme;
    final.dataset.id = unit.id;
    final.title = `${unit.grapheme} · ${unit.id} · ${unit.timingMode}`;
    final.style.left = `${finalX}px`;
    final.style.top = `${finalY}px`;
    final.style.transform = `translate(-50%, -50%) rotate(${unit.final.rotation}deg) scale(${unit.final.scale})`;
    stage.append(final);

    const guide = document.createElement("div");
    guide.className = "motion-guide";
    const dx = unit.inPose.x;
    const dy = unit.inPose.y;
    const length = Math.hypot(dx, dy);
    guide.style.width = `${length}px`;
    guide.style.left = `${finalX}px`;
    guide.style.top = `${finalY}px`;
    guide.style.transform = `rotate(${Math.atan2(dy, dx)}rad)`;
    stage.append(guide);
  }
}

function renderTimeline(evaluated) {
  timeline.replaceChildren();
  for (const unit of evaluated) {
    const row = document.createElement("div");
    row.className = "timeline-row";
    row.innerHTML = `
      <div class="row-label">
        <strong>${unit.grapheme}</strong>
        <span>${unit.id}</span>
        <em class="${unit.timingMode}">${unit.timingMode.toUpperCase()}</em>
      </div>
      <div class="track">
        <div class="bar ${unit.override ? "overridden" : ""}" style="left:${unit.start * 18}px;width:${unit.duration * 18}px">
          ${unit.start}f → ${unit.start + unit.duration}f
        </div>
      </div>
    `;
    timeline.append(row);
  }
}

function renderTable(evaluated) {
  const rows = evaluated
    .map(
      (unit) => `
        <tr>
          <td class="char">${unit.grapheme}</td>
          <td><code>${unit.id}</code></td>
          <td>${unit.override ? "manual" : "parent"}</td>
          <td>${unit.timingMode} ${unit.start}f</td>
          <td>${Math.round(unit.final.y)} / ${Math.round(unit.final.scale * 100)}%</td>
        </tr>
      `,
    )
    .join("");
  identityTable.innerHTML = `
    <table>
      <thead><tr><th>文字</th><th>ID</th><th>値の由来</th><th>Timing</th><th>Y / Scale</th></tr></thead>
      <tbody>${rows}</tbody>
    </table>
  `;
}

function renderReview() {
  if (state.needsReview.length === 0) {
    reviewList.className = "empty";
    reviewList.textContent = "0件 — 黙った誤接続なし";
    return;
  }
  reviewList.className = "review-items";
  reviewList.innerHTML = state.needsReview
    .map(
      (item) => `
        <div>
          <strong>${item.grapheme} · ${item.oldId}</strong>
          <span>${item.reason}</span>
          <small>別文字へ移さず停止</small>
        </div>
      `,
    )
    .join("");
}

function render() {
  const evaluated = evaluateState(state, {
    seed: Number(seedInput.value),
    interval: Number(intervalInput.value),
  });
  status.textContent = `${stateText(state)} · ${state.lastEdit}`;
  explanation.innerHTML = describeCurrent(evaluated);
  renderStage(evaluated);
  renderTimeline(evaluated);
  renderTable(evaluated);
  renderReview();
}

document.querySelector("#reset").addEventListener("click", () => {
  state = preparedState();
  render();
});

document.querySelector("#insert-road").addEventListener("click", () => {
  if (stateText(state) === "夜を走る") {
    state = insertText(state, 1, "道");
  }
  render();
});

document.querySelector("#replace-run").addEventListener("click", () => {
  const index = state.units.findIndex((unit) => unit.grapheme === "走");
  if (index >= 0) state = replaceRange(state, index, 1, "飛");
  render();
});

document.querySelector("#delete-run").addEventListener("click", () => {
  const index = state.units.findIndex((unit) => unit.grapheme === "走");
  if (index >= 0) state = deleteRange(state, index, 1);
  render();
});

document.querySelector("#apply-whole").addEventListener("click", () => {
  state = reconcileWholeText(state, document.querySelector("#whole-text").value);
  render();
});

seedInput.addEventListener("input", render);
intervalInput.addEventListener("input", render);
window.addEventListener("resize", render);

state = preparedState();
render();
