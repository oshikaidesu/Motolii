export const allSurfacesFixture = {
  id: "all-surfaces",
  project: "night_drive.mv",
  composition: "Pulse rings",
  plugin: {
    name: "Echo Bloom",
    state: "installed",
    input: "TEXTURE",
    output: "TEXTURE",
  },
  browser: {
    activeTab: "plugins",
    items: [
      { name: "Echo Bloom", purpose: "Pulse texture", state: "installed" },
      { name: "Glyph Current", purpose: "Typography", state: "available" },
      { name: "Fold Field", purpose: "Feedback", state: "blocked" },
      { name: "Ribbon Array", purpose: "Generator", state: "missing" },
    ],
  },
  inspector: {
    object: "Pulse rings",
    depth: 0.18,
    parameters: [
      { name: "Intensity", value: "72%", automation: "on" },
      { name: "Spread", value: "38%", automation: "off" },
      { name: "Fill", value: "#df8b4d", automation: "off" },
    ],
  },
  timeline: {
    timecode: "00:54.2",
    bpm: 120,
    bars: [
      {
        name: "Pulse rings",
        kind: "GROUP",
        left: 4,
        width: 88,
        depth: "+.18",
        flow: "IN → Echo Bloom → OUT",
      },
      {
        name: "NIGHT DRIVE",
        kind: "TEXT",
        left: 18,
        width: 54,
        depth: "0",
        flow: "",
      },
    ],
  },
};
