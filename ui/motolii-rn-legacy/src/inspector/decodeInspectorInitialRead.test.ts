import {
  decodeInspectorInitialRead,
  decodeInspectorInitialReadValue,
} from "./decodeInspectorInitialRead";

function validBase(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    version: 1,
    direction: "host-to-rn",
    role: "product-runtime-seat",
    host_handle: "1",
    revision: "0",
    projection_generation: "3",
    stage: {
      selection: [],
      bounds: [],
    },
    diagnostics: [],
    ...overrides,
  };
}

function encode(value: unknown): string {
  return JSON.stringify(value);
}

describe("decodeInspectorInitialRead", () => {
  test("no primary + empty selection → tagged no-selection", () => {
    const result = decodeInspectorInitialRead(encode(validBase()));
    expect(result).toEqual({
      ok: true,
      value: {
        tag: "no-selection",
        revision: "0",
        projectionGeneration: "3",
      },
    });
  });

  test("selected when primary equals sole selection and matching bounds", () => {
    const result = decodeInspectorInitialRead(
      encode(
        validBase({
          primary_layer_id: "42",
          revision: "7",
          projection_generation: "11",
          stage: {
            selection: [{ layer_id: "42" }],
            bounds: [
              { layer_id: "99", display_name: "Other" },
              { layer_id: "42", display_name: "Hero" },
            ],
          },
          diagnostics: [{ ignored: true }, "raw"],
        }),
      ),
    );
    expect(result).toEqual({
      ok: true,
      value: {
        tag: "selected",
        layerId: "42",
        displayName: "Hero",
        revision: "7",
        projectionGeneration: "11",
      },
    });
    if (result.ok) {
      expect(result.value).not.toHaveProperty("host_handle");
      expect(result.value).not.toHaveProperty("diagnostics");
      expect(result.value).not.toHaveProperty("hostHandle");
    }
  });

  test("rejects invalid JSON", () => {
    expect(decodeInspectorInitialRead("{")).toEqual({
      ok: false,
      error: { tag: "invalid-json" },
    });
  });

  test("rejects non-object root and arrays", () => {
    expect(decodeInspectorInitialRead("null")).toEqual({
      ok: false,
      error: { tag: "non-object" },
    });
    expect(decodeInspectorInitialRead('"x"')).toEqual({
      ok: false,
      error: { tag: "non-object" },
    });
    expect(decodeInspectorInitialRead("1")).toEqual({
      ok: false,
      error: { tag: "non-object" },
    });
    expect(decodeInspectorInitialRead("[]")).toEqual({
      ok: false,
      error: { tag: "non-object" },
    });
  });

  test("rejects unknown root field", () => {
    const result = decodeInspectorInitialReadValue(
      validBase({ extra: 1 }),
    );
    expect(result).toEqual({
      ok: false,
      error: { tag: "unknown-field", path: "", field: "extra" },
    });
  });

  test("rejects missing required root field", () => {
    const base = validBase();
    delete base.revision;
    expect(decodeInspectorInitialReadValue(base)).toEqual({
      ok: false,
      error: { tag: "missing-field", path: "", field: "revision" },
    });
  });

  test("rejects invalid expected constants", () => {
    expect(
      decodeInspectorInitialReadValue(validBase({ version: 2 })),
    ).toEqual({
      ok: false,
      error: { tag: "invalid-constant", path: "", field: "version" },
    });
    expect(
      decodeInspectorInitialReadValue(validBase({ direction: "rn-to-host" })),
    ).toEqual({
      ok: false,
      error: { tag: "invalid-constant", path: "", field: "direction" },
    });
    expect(
      decodeInspectorInitialReadValue(validBase({ role: "other" })),
    ).toEqual({
      ok: false,
      error: { tag: "invalid-constant", path: "", field: "role" },
    });
  });

  test("rejects invalid unsigned / positive decimals", () => {
    expect(
      decodeInspectorInitialReadValue(validBase({ revision: "01" })),
    ).toEqual({
      ok: false,
      error: { tag: "invalid-decimal", path: "", field: "revision" },
    });
    expect(
      decodeInspectorInitialReadValue(validBase({ revision: "" })),
    ).toEqual({
      ok: false,
      error: { tag: "invalid-decimal", path: "", field: "revision" },
    });
    expect(
      decodeInspectorInitialReadValue(validBase({ host_handle: "0" })),
    ).toEqual({
      ok: false,
      error: { tag: "invalid-decimal", path: "", field: "host_handle" },
    });
    expect(
      decodeInspectorInitialReadValue(
        validBase({ primary_layer_id: "0" }),
      ),
    ).toEqual({
      ok: false,
      error: {
        tag: "invalid-decimal",
        path: "",
        field: "primary_layer_id",
      },
    });
    expect(
      decodeInspectorInitialReadValue(
        validBase({
          stage: {
            selection: [{ layer_id: "0" }],
            bounds: [],
          },
        }),
      ),
    ).toEqual({
      ok: false,
      error: {
        tag: "invalid-decimal",
        path: "stage.selection[0]",
        field: "layer_id",
      },
    });
  });

  test("rejects unknown / missing fields on stage and entries", () => {
    expect(
      decodeInspectorInitialReadValue(
        validBase({
          stage: { selection: [], bounds: [], leftover: true },
        }),
      ),
    ).toEqual({
      ok: false,
      error: { tag: "unknown-field", path: "stage", field: "leftover" },
    });

    expect(
      decodeInspectorInitialReadValue(
        validBase({
          stage: {
            selection: [{ layer_id: "1", note: "x" }],
            bounds: [],
          },
        }),
      ),
    ).toEqual({
      ok: false,
      error: {
        tag: "unknown-field",
        path: "stage.selection[0]",
        field: "note",
      },
    });

    expect(
      decodeInspectorInitialReadValue(
        validBase({
          stage: {
            selection: [],
            bounds: [{ layer_id: "1" }],
          },
        }),
      ),
    ).toEqual({
      ok: false,
      error: {
        tag: "missing-field",
        path: "stage.bounds[0]",
        field: "display_name",
      },
    });
  });

  test("rejects non-array diagnostics", () => {
    expect(
      decodeInspectorInitialReadValue(validBase({ diagnostics: {} })),
    ).toEqual({
      ok: false,
      error: { tag: "invalid-type", path: "", field: "diagnostics" },
    });
  });

  test("rejects duplicate selection and bounds IDs", () => {
    expect(
      decodeInspectorInitialReadValue(
        validBase({
          primary_layer_id: "1",
          stage: {
            selection: [{ layer_id: "1" }, { layer_id: "1" }],
            bounds: [{ layer_id: "1", display_name: "A" }],
          },
        }),
      ),
    ).toEqual({
      ok: false,
      error: { tag: "duplicate-selection-id" },
    });

    expect(
      decodeInspectorInitialReadValue(
        validBase({
          primary_layer_id: "1",
          stage: {
            selection: [{ layer_id: "1" }],
            bounds: [
              { layer_id: "1", display_name: "A" },
              { layer_id: "1", display_name: "B" },
            ],
          },
        }),
      ),
    ).toEqual({
      ok: false,
      error: { tag: "duplicate-bounds-id" },
    });
  });

  test("rejects dangling primary", () => {
    expect(
      decodeInspectorInitialReadValue(
        validBase({
          primary_layer_id: "5",
          stage: { selection: [], bounds: [] },
        }),
      ),
    ).toEqual({
      ok: false,
      error: { tag: "dangling-primary" },
    });
  });

  test("rejects selection count other than zero or one", () => {
    expect(
      decodeInspectorInitialReadValue(
        validBase({
          primary_layer_id: "1",
          stage: {
            selection: [{ layer_id: "1" }, { layer_id: "2" }],
            bounds: [
              { layer_id: "1", display_name: "A" },
              { layer_id: "2", display_name: "B" },
            ],
          },
        }),
      ),
    ).toEqual({
      ok: false,
      error: { tag: "invalid-selection-count" },
    });
  });

  test("rejects mismatch cases", () => {
    // selection without primary
    expect(
      decodeInspectorInitialReadValue(
        validBase({
          stage: {
            selection: [{ layer_id: "1" }],
            bounds: [{ layer_id: "1", display_name: "A" }],
          },
        }),
      ),
    ).toEqual({
      ok: false,
      error: { tag: "mismatch" },
    });

    // primary ≠ sole selection
    expect(
      decodeInspectorInitialReadValue(
        validBase({
          primary_layer_id: "1",
          stage: {
            selection: [{ layer_id: "2" }],
            bounds: [
              { layer_id: "1", display_name: "A" },
              { layer_id: "2", display_name: "B" },
            ],
          },
        }),
      ),
    ).toEqual({
      ok: false,
      error: { tag: "mismatch" },
    });

    // primary=selection but no bounds for that id
    expect(
      decodeInspectorInitialReadValue(
        validBase({
          primary_layer_id: "1",
          stage: {
            selection: [{ layer_id: "1" }],
            bounds: [{ layer_id: "9", display_name: "Other" }],
          },
        }),
      ),
    ).toEqual({
      ok: false,
      error: { tag: "mismatch" },
    });
  });
});
