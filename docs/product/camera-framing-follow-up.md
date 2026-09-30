# Camera Framing: two meanings, one follow-up (2026-09-30)

Not part of the direct-manipulation PR. Recorded so it is decided once, on its own.

## What exists today
- **Resolver and Stage gizmo** (`resolve/camera.rs` `framed_camera`, `snapshot.rs` `camera_gizmos` / `camera_box`, engine `camera_of_scene_layer_in`): with a Target layer and `camera.framing` > 0, Center, Target Z and Distance are derived (the camera frames the target's size; Zoom feeds the Distance solve). The Stage box and frustum show the framed camera.
- **Production render camera** (`frame_graph/camera_program.rs`): reads the Framing row but never uses it; Distance stays as authored. Pinned as a known divergence in `camera_pose_parity.rs` ("framing size on a target shape", "composition camera track alone").
- Also divergent inside the pinned set: three definitions of the target point (anchor in the program, bounds centre in the engine, pivot in the resolver) and the Transition glide of centre / z (lost after the engine re-aims at the bounds centre).

## Why it is not decided here
The existing parity test says the divergence is *observed, not desired*, but neither side is named as the truth. Choosing changes what the picture shows for any document that already has Framing > 0.

## What the user sees
- The product UI Camera instrument has no Framing Size or Near Fade row (`CameraStore.rows` has seven). Framing cannot be set, seen or cleared from the product UI; a document that carries it silently overrides Distance for the resolver and the Stage box.
- Distance stays editable while Framing overrides it, and the "Frames the target at ×zoom/distance" line is then wrong.

## To bring Framing into the production render
- The render graph needs the target's bounds (it has none today; the engine path reads them from the scene), and one definition of "the target point".
- Then remove the two entries from `KNOWN_DIVERGENCE_*` in `camera_pose_parity.rs` and add the Framing row (and gate Distance while Framing > 0 and a Target is set, as Center / Target Z are gated by a Target).

## Left as they are on purpose (this pass)
- Orbit sensitivity: Stage 0.3 deg/px, Inspector face 0.8 deg/px (Shift 0.1x). No document justifies either; the face is a ~132 px schematic. Unchanged.
- Pitch clamp ±85 only in `StageSession.orbitBy`. Native has no range on `camera.orbit`; a camera layer may pitch past 90 (the image flips over the pole, no NaN). No reason on record; unchanged.
- Pitch sign is now one convention (composition y down: +pitch puts the eye below the target; pinned in `motolii-doc` `camera.rs` and `test/camera_orbit_sign_test.dart`).
