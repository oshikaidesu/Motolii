# Color themes

Open **Settings → Color theme → Load JSON…** and select a theme file. No build or restart is needed. The selection is shared with other open Motolii windows through the existing window-state notifications. **Reload** reads edits from the same file. **Default** returns to Chromatic Workshop. **Copy JSON** copies the complete active theme as a starting point for your own file.

The application stores both the source path and a validated snapshot in its existing local settings. Reopening the application restores the snapshot even if the source file has moved. File edits become active only when you choose Reload; there is no background polling. A failed load leaves the last valid theme active and displays the error.

Start with [velvet.json](velvet.json), which demonstrates partial overrides. Omitted roles inherit Chromatic Workshop. [theme.schema.json](theme.schema.json) lists all available keys for schema version 1. Unknown keys, unsupported versions, malformed colors and empty palettes are rejected. Files must be at most 128 KB.

```json
{
  "schemaVersion": 1,
  "name": "My theme",
  "colors": {
    "panel": "#303231",
    "ink": "#f1f0e9",
    "tab": "#59c9df"
  }
}
```

Colors use **#RRGGBB** or **#RRGGBBAA**, with alpha last. For example, `#59c9df66` is cyan at 40% opacity. This differs from Flutter's integer ARGB notation.

| Group | What it paints |
|---|---|
| `colors.app`, `panel`, `raised`, `hover` | Application and control surfaces |
| `colors.line`, `border` | Dividers and outlined controls |
| `colors.ink`, `muted`, `disabledInk`, `inkDisabled` | Normal, secondary and disabled lettering |
| `colors.tab`, `tabInk` | Active dock tab and its lettering |
| `colors.accent`, `animate`, `caret`, `selection` | Active controls, animation accent and text editing |
| `colors.menu`, `menuEdge`, `select`, `selectInk`, `tooltip` | Menus, menu selection and tooltips |
| `colors.spatial`, `amount`, `time`, `count`, `seed`, `angle` | Parameter families |
| `colors.kindText`, `kindShape`, `kindPath`, `kindVideo`, `kindAudio`, `kind3d`, `kindHdr`, `kindImage`, `kindOther` | Browser category marks |
| Remaining `colors` keys | Error, overlay, scrollbar, disabled and slider states; Copy JSON exports them all |
| `colors.timelineWash` | A neutral overlay on Timeline layer colors only. Default `#6060604d` softens brightness and saturation; `#00000000` displays the original identity colors |
| `identityColors` | 1–64 repeating layer identity colors; the same identity is used across panels |
| `drawing` | Timeline lanes and grids, camera guides, focus ring, Ease desk and transparency checkerboard |
| `collectionColors` | Exactly seven Browser collection colors |

These are **editor appearance** colors. They do not change artwork fills, source thumbnails, effect parameters, renders, exports, or Document/Undo history. Typography and geometry remain application rules rather than part of the color-theme format.

Choose foreground/background pairs together. Check the actual window, menus, selected/unselected rows, disabled controls and all panels you use. Import validation checks the file format; it does not certify the contrast or aesthetic quality of a third-party palette.

Implementation: [EditorTheme](../lib/foundation/theme.dart) is a Flutter `ThemeExtension`; widgets depend on the inherited theme and painters receive a theme snapshot. Theme switches redraw the existing widget tree without recreating the editing session.
