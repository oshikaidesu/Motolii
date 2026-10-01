# design-sense-lab が今持っているウィジェット(126 use case)

`book/lib/main.dart` と `book/lib/sets/*.dart` から機械的に抽出(2026-10-02)。


## main.dart
- **Shell**
  - Window
  - Top bar
- **Timeline**
  - Six layers
- **Browser**
  - Objects, generators, relations
- **Inspector**
  - Relations tab
- **Stage chrome**
  - Tools and bars
- **Relations**
  - Tiles
  - Gadgets
  - Card
- **Controls**
  - Value well
  - Switch
  - Segmented
- **Tokens**
  - Families and greys

## sets/transport.dart
- **transport**
  - Transport buttons / states
  - Transport cluster
  - Timecode readout
  - FPS chip
  - Mode switch
  - Wordmark block
  - Menu bar
  - Window title bar
  - Zoom control
  - Camera view dropdown
  - Undo / redo pair
  - Loop / range toggle
  - Master volume + meter
  - Top bar (assembled)

## sets/inputs.dart
- **inputs**
  - Scrub number
  - Vec3 row
  - Link / lock ratio
  - Text field
  - Search with clear
  - Dropdown
  - Checkbox / radio
  - Slider with ticks
  - Range slider
  - Colour swatch + hex
  - Tag input
  - Stepper
  - Toggle button group
  - Curve preset picker

## sets/navigation.dart
- **navigation**
  - Tab strip (underline)
  - Tab strip (pill)
  - Panel header
  - Dock tab (close + grip)
  - Breadcrumb
  - Tree view
  - List row
  - Section header
  - Context menu
  - Popover with arrow
  - Command palette
  - Sidebar nav item
  - Accordion group
  - Dialog sheet

## sets/timeline_parts.dart
- **timeline_parts**
  - Layer label cell
  - Label cell / all states
  - Keyframe diamonds
  - Layer bars
  - Playhead
  - Marker flag
  - Work area bar
  - Ruler / zoom-adaptive ticks
  - Mini curve editor
  - Property lane / Position X Y
  - Time zoom slider
  - Scrub / transport strip
  - Lane header
  - In context / full timeline

## sets/relation_set.dart
- **relation_set**
  - Gadgets 1-4 · Space (Falloff, Direction, Scatter, Region)
  - Gadgets 5 · Path and time (Along Path, Stagger, Follow, Curve)
  - Gadgets 6 · Distribution (Scale, Rotation, Color, Repeat)
  - Gadgets 7 · Signal (Noise, Graph / Link, Audio)
  - Gadget · one, large
  - Gadget wall · all 15
  - Chip · compact, for lists
  - Badge · on a layer row
  - Add relation menu
  - Card · collapsed / expanded
  - Card · disabled / in error
  - Connection line · drag the layers
  - Empty state · no relations yet
  - Flow · add, tune, see
  - Card list · inspector column

## sets/feedback.dart
- **feedback**
  - Progress bar
  - Spinner
  - Status bar
  - Toast stack
  - Inline banner
  - Empty state
  - Skeleton loader
  - Tooltip with shortcut
  - Keyboard shortcut chip
  - Badge and change dot
  - Error row with Why?
  - Render queue row
  - Export sheet
  - Undo history

## sets/media_set.dart
- **media_set**
  - Asset tile
  - Asset list row
  - Thumbnail grid / masonry
  - Source chips
  - Type filter strip
  - Search + filter bar
  - Folder tree
  - Preset row
  - Font row
  - Colour palette strip
  - Drag ghost
  - Empty state
  - Import drop zone
  - Asset details card

## sets/stage_set.dart
- **stage_set**
  - Tool column
  - Tool options bar
  - Selection box
  - Transform gizmo
  - Pivot / anchor marker
  - Guides and rulers
  - Grid and safe area
  - Camera frustum (top view)
  - Zoom HUD and corner label
  - Cursor tooltip
  - Snapping indicator
  - Checkerboard backdrop
  - Minimap / navigator
  - Composition frame
