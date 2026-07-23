import { StageSurface } from "../surfaces/StageSurface.jsx";
import { BrowserSurface } from "../surfaces/BrowserSurface.jsx";
import { InspectorSurface } from "../surfaces/InspectorSurface.jsx";
import { TimelineSurface } from "../surfaces/TimelineSurface.jsx";
import { Button } from "../primitives/index.jsx";
import { referenceFixtures } from "./fixtures.js";
import { loadReferenceFixtures } from "./loadReferenceFixtures.js";
import { ReferenceStateStrip } from "./ReferenceStateStrip.jsx";

export function StageFrameToolsReference() {
  const fixture = loadReferenceFixtures("stage-frame-tools", referenceFixtures);
  return (
    <main className="motolii-mock-app" data-reference-screen="stage-frame-tools">
      <header className="mock-titlebar">
        <strong className="wordmark">MOTOLII</strong><span>Stage frame and tools</span>
      </header>
      <nav className="mock-commandbar" aria-label="Stage tools">
        <Button pressed={fixture.scene.focus === "stage.selection"}>SELECT</Button>
        <Button
          data-semantic-id="camera"
          pressed={fixture.scene.focus === "stage.camera"}
          variant="quiet"
        >
          CAMERA{fixture.scene.focus === "stage.camera" ? " · focused" : ""}
        </Button>
        <Button
          data-semantic-id="hand"
          pressed={fixture.scene.focus === "stage.hand"}
          variant="quiet"
        >
          HAND{fixture.scene.hover === "stage.hand" ? " · hover" : ""}
        </Button>
      </nav>
      <section className="mock-workspace">
        <BrowserSurface fixture={fixture.browser} />
        <StageSurface
          composition={fixture.composition}
          plugin={fixture.plugin}
          frameState={{
            selected: fixture.inspector.object,
            outside: `layer ${fixture.scene.outsideFrame.join(", ")}`,
            scrimPercent: Math.round(fixture.scene.scrimOpacity * 100),
          }}
        />
        <InspectorSurface fixture={fixture.inspector} plugin={fixture.plugin} />
      </section>
      <TimelineSurface fixture={fixture.timeline} />
      <ReferenceStateStrip
        spacing={fixture.tokenSpacing}
        items={[{ label: "Frame states are projected into the Stage canvas" }]}
      />
    </main>
  );
}
