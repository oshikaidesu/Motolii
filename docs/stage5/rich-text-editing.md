# Direct rich text editing

2026-09-09 user correction: the TEXT area is an extension of the text box, not a preview. Select and type in it; change font and size for selected characters; select Hiragana, Katakana and Han characters as a group.

Contract: TextRun.len counts extended grapheme clusters (UAX #29). Flutter selection is UTF-16; native snaps ranges to grapheme boundaries. Unicode Script supplies character classes, including inherited kana marks. Runs, styles and property sources remain in Document; UI owns only focus, selection and composing drafts. Content edits preserve unaffected runs. Formatting creates one undoable transaction, preserving properties not being changed.

Rendering: use cosmic_text Buffer.set_rich_text with per-span family/metrics and glyph metadata, then the existing vector rasterizer. Stage and export share that renderer. The editable Flutter text field uses span styles and an explicitly uniform editing zoom; it does not replace the Stage renderer.

Verification: mixed-script/combining/emoji selections; replacement/insertion/deletion run preservation; style change + Undo/save; mixed font/size raster output; editing and IME composing; locked layer rejection. Existing Blend readability work remains in progress and its behavior tests must also be brought back to passing.

References inspected: store/text.rs TextRun/TextDocument, editor/text.rs, resolved_text_document, engine/text.rs, cosmic-text Buffer.set_rich_text/Attrs.metadata; unicode-segmentation 1.13.3 and unicode-script 0.5.8 already in Cargo.lock; Flutter TextEditingController.buildTextSpan and TextField composing/selection ownership.

## Implemented and checked

- Inspector uses one styled TextField; the image specimen plus separate plain-content input has been removed. Selection/All/Hiragana/Katakana/Kanji/Latin scope, font search and direct size editing share the text-format route. A stable editing zoom preserves visible size changes.
- Typing previews are coalesced, IME composition is held until committed, Escape cancels, and pendingEditors flushes focused input before Save/close. Font choices from the Fonts shelf also carry the current text selection.
- Native formatting splits/coalesces runs, copies unchanged property sources when splitting a style, and preserves content keys. Insertion inherits adjacent formatting; UTF-16 selection snaps to grapheme boundaries. Renderer uses rich spans and the texture key includes runs.
- Native text-format tests (3): script-only sizes, selected font/size changes reflected in raster output, insertion preservation, empty text, stale selection rejection, save/load and Undo. Unicode test (1): combining kana, surrogate pairs and kana prolonged marks. Renderer cache test (1): changed style boundaries invalidate textures. All passed.
- Flutter focused regression (8) passed; final rich-text input/IME/save tests (3) passed after span merging. Stage 5 and whitespace checks passed. App and native builds completed; the final native revision is rebuilt after validation. Live-window interaction is left to the user as requested. Existing Lottie export restrictions on multi-style text remain; .rrd and the shared raster/video path carry the runs.
