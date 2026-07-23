import { TimelineSurface } from "../surfaces/TimelineSurface.jsx";
import { BrowserSurface } from "../surfaces/BrowserSurface.jsx";
import { InspectorSurface } from "../surfaces/InspectorSurface.jsx";
import { StageSurface } from "../surfaces/StageSurface.jsx";
import { Button } from "../primitives/index.jsx";
import { referenceFixtures } from "./fixtures.js";
import { loadReferenceFixtures } from "./loadReferenceFixtures.js";
import { ReferenceStateStrip } from "./ReferenceStateStrip.jsx";

export function MixedTimelineReference() {
  const fixture = loadReferenceFixtures("mixed-timeline", referenceFixtures);
  return (
    <main className="motolii-mock-app" data-reference-screen="mixed-timeline">
      <header className="mock-titlebar">
        <strong className="wordmark">MOTOLII</strong>
        <span>Reference document · mixed objects</span>
        <span className="mock-grow" />
        <Button>Export</Button>
      </header>
      <nav className="mock-commandbar" aria-label="Timeline commands">
        <Button pressed>SELECT</Button><Button variant="quiet">KEYS</Button>
      </nav>
      <section className="mock-workspace">
        <BrowserSurface fixture={fixture.browser} />
        <StageSurface composition={fixture.composition} plugin={fixture.plugin} />
        <InspectorSurface fixture={fixture.inspector} plugin={fixture.plugin} />
      </section>
      <TimelineSurface fixture={fixture.timeline} />
      <ReferenceStateStrip
        spacing={fixture.tokenSpacing}
        items={[{ label: "Reference states are projected into the Timeline bars" }]}
      />
    </main>
  );
}
