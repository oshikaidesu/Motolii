import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../foundation/metrics.dart';
import '../foundation/theme.dart';
import '../foundation/panel_controls.dart';
import '../session/editor_session.dart';

class StyledTextController extends TextEditingController {
  StyledTextController({required super.text});
  List<Map<String, dynamic>> styles = [], runs = [];

  /// The one family the box draws in — a glance at the face, nothing more.
  /// Null while the text carries more than one family, or while the family
  /// is not one the machine reported: an unknown name only costs a lookup
  /// and gives back the same fallback face.
  String? previewFamily;
  Set<int> highlighted = {};
  List<Map<String, dynamic>> characterStyles() {
    final count = text.characters.length;
    final byId = {for (final s in styles) s['id']: s};
    final fallback = styles.isEmpty ? <String, dynamic>{} : styles.first;
    final result = <Map<String, dynamic>>[];
    for (final run in runs) {
      final n = math.min((run['len'] as num).toInt(), count - result.length);
      result.addAll(List.filled(n, byId[run['style']] ?? fallback));
      if (result.length == count) break;
    }
    while (result.length < count) {
      result.add(fallback);
    }
    return result;
  }

  // The box is a text field, not the composition: one face, one UI size, one
  // line height. The class highlight is the only reason to break the string,
  // and while the IME is composing even that stands aside — so a keystroke
  // lays out one span, not one per grapheme.
  @override
  TextSpan buildTextSpan({
    required BuildContext context,
    TextStyle? style,
    required bool withComposing,
  }) {
    final base = previewFamily == null
        ? style
        : (style ?? const TextStyle()).copyWith(fontFamily: previewFamily);
    if (highlighted.isEmpty || (withComposing && value.composing.isValid))
      return super.buildTextSpan(
        context: context,
        style: base,
        withComposing: withComposing,
      );
    final marked = (base ?? const TextStyle()).copyWith(
      backgroundColor: EditorTheme.select,
      color: EditorTheme.selectInk,
    );
    final children = <TextSpan>[];
    final buffer = StringBuffer();
    bool? was;
    var index = 0;
    void flush() {
      if (buffer.isNotEmpty)
        children.add(
          TextSpan(text: buffer.toString(), style: was! ? marked : null),
        );
      buffer.clear();
    }

    for (final g in text.characters) {
      final on = highlighted.contains(index++);
      if (was != null && on != was) flush();
      was = on;
      buffer.write(g);
    }
    flush();
    return TextSpan(style: base, children: children);
  }
}

class RichTextEditor extends StatefulWidget {
  const RichTextEditor({
    super.key,
    required this.controller,
    required this.layer,
    required this.text,
  });
  final EditorSession controller;
  final Map<String, dynamic> layer, text;
  @override
  State<RichTextEditor> createState() => _RichTextEditorState();
}

class _RichTextEditorState extends State<RichTextEditor> {
  late final _text = StyledTextController(
    text: '${widget.text['content'] ?? ''}',
  );
  final _focus = FocusNode();
  String _scope = 'all', _baseline = '';
  List? _knownFrom;
  Set<String> _known = const {};

  /// The families the machine reported, as a set built once per list it
  /// hands over — a name outside it is never asked of the font machinery.
  Set<String> _knownFamilies(List? raw) {
    if (!identical(raw, _knownFrom)) {
      _knownFrom = raw;
      _known = (raw ?? const []).whereType<String>().toSet();
    }
    return _known;
  }

  bool _wasComposing = false;
  bool _dirty = false, _syncing = false, _committing = false;
  Future<void>? _commitFlight;
  late final _previews = EditorPreviewQueue<String>(
    (text) => c.command('previewText', {
      'layer': widget.layer['id'],
      'content': text,
      'interaction': _inputTag,
    }),
  );
  EditorSession get c => widget.controller;
  String get _inputTag =>
      'rich-input:${widget.layer['id']}:${identityHashCode(this)}';
  String get _formatTag =>
      'rich-format:${widget.layer['id']}:${identityHashCode(this)}';
  Future<void> _finish(String tag, bool cancel) {
    if (c.state.containsKey('previewInteraction') &&
        c.state['previewInteraction'] != tag)
      return Future.value();
    return c.command(cancel ? 'cancelPreview' : 'commitPreview', {
      if (c.state['previewOwner'] is num) 'owner': c.state['previewOwner'],
    });
  }

  bool get _composing =>
      _text.value.composing.isValid && !_text.value.composing.isCollapsed;
  bool get _enabled =>
      widget.layer['locked'] != true &&
      c.supports('styleText') &&
      !_composing &&
      !_committing;
  Map<String, dynamic> get _target => {
    'layer': widget.layer['id'],
    'scope': _scope,
    'text': _text.text,
    'start': math.max(
      0,
      math.min(_text.selection.baseOffset, _text.selection.extentOffset),
    ),
    'end': math.max(
      0,
      math.max(_text.selection.baseOffset, _text.selection.extentOffset),
    ),
  };
  @override
  void initState() {
    super.initState();
    _baseline = _text.text;
    c.pendingEditors.add(_flush);
    _formats();
    _text.addListener(_changed);
    _focus.addListener(_focused);
    _focus.onKeyEvent = (_, event) {
      if (event is KeyDownEvent &&
          event.logicalKey == LogicalKeyboardKey.escape &&
          !_composing) {
        _cancel();
        return KeyEventResult.handled;
      }
      return KeyEventResult.ignored;
    };
  }

  void _formats() {
    _text.styles = EditorSession.maps(widget.text['styles']);
    if (_text.styles.isEmpty)
      _text.styles = [
        {
          'id': 0,
          'size': widget.text['size'] ?? 24,
          'font': {'family': widget.text['fontFamily']},
        },
      ];
    _text.runs = EditorSession.maps(widget.text['runs']);
  }

  void _changed() {
    if (_syncing) return;
    if (_wasComposing && !_composing && _dirty && c.supports('previewText'))
      _previews.add(_text.text);
    _wasComposing = _composing;
    if (!_text.selection.isCollapsed && _text.selection.isValid)
      _scope = 'selection';
    if (_scope == 'selection' && _text.selection.isCollapsed) _scope = 'all';
    c.textStyleTarget.value = _target;
    setState(() {});
  }

  void _typed(String value) {
    _dirty = true;
    if (!_composing && c.supports('previewText')) _previews.add(value);
  }

  void _focused() {
    if (_focus.hasFocus) {
      if (!_dirty) _baseline = _text.text;
      c.textStyleTarget.value = _target;
    } else if (_dirty) {
      if (_composing)
        _text.value = _text.value.copyWith(composing: TextRange.empty);
      _commit();
    }
  }

  Future<void> _flush() async {
    if (_composing)
      _text.value = _text.value.copyWith(composing: TextRange.empty);
    await _commit();
  }

  Future<void> _commit() {
    if (_commitFlight != null) return _commitFlight!;
    if (!_dirty || _composing) return Future.value();
    _committing = true;
    final content = _text.text;
    return _commitFlight = () async {
      try {
        if (c.supports('previewText')) {
          _previews.add(content);
          await _previews.finish(false, () => _finish(_inputTag, false));
        } else {
          await c.command('setText', {
            'layer': widget.layer['id'],
            'content': content,
          });
        }
        _dirty = _text.text != content || c.error.value != null;
        _baseline = content;
      } finally {
        _committing = false;
        _commitFlight = null;
        if (mounted) setState(() {});
      }
    }();
  }

  Future<void> _cancel() async {
    if (!_dirty) return;
    await _previews.finish(true, () => _finish(_inputTag, true));
    if (!mounted) return;
    _syncing = true;
    _text.value = TextEditingValue(
      text: _baseline,
      selection: TextSelection.collapsed(offset: _baseline.length),
    );
    _syncing = false;
    setState(() => _dirty = false);
    c.textStyleTarget.value = _target;
  }

  Future<void> _format(Map<String, dynamic> patch) async {
    if (_composing || widget.layer['locked'] == true) return;
    await _commit();
    if (!mounted || _dirty) return;
    await c.command('styleText', {
      ..._target,
      ...patch,
      'interaction': _formatTag,
    });
  }

  @override
  void didUpdateWidget(covariant RichTextEditor old) {
    super.didUpdateWidget(old);
    if (old.controller != c) {
      old.controller.pendingEditors.remove(_flush);
      c.pendingEditors.add(_flush);
    }
    _formats();
    final incoming = '${widget.text['content'] ?? ''}';
    if (!_dirty && incoming != _text.text) {
      _syncing = true;
      final selection = _text.selection;
      _text.value = TextEditingValue(
        text: incoming,
        selection: TextSelection(
          baseOffset: selection.baseOffset.clamp(0, incoming.length),
          extentOffset: selection.extentOffset.clamp(0, incoming.length),
        ),
      );
      _baseline = incoming;
      _syncing = false;
    }
  }

  @override
  void dispose() {
    c.pendingEditors.remove(_flush);
    if (_dirty && !_committing)
      _previews.finish(true, () => _finish(_inputTag, true));
    _text.removeListener(_changed);
    _focus.removeListener(_focused);
    _text.dispose();
    _focus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    _text.highlighted = {
      for (final (i, kind) in (widget.text['classes'] as List? ?? []).indexed)
        if (kind == _scope) i,
    };
    final formats = _text.characterStyles();
    final selection = _text.selection;
    var offset = 0, index = 0;
    final picked = <Map<String, dynamic>>[];
    for (final g in _text.text.characters) {
      final end = offset + g.length;
      if (_scope == 'all' ||
          (_scope == 'selection' &&
              offset < selection.end &&
              end > selection.start) ||
          (index < (widget.text['classes'] as List? ?? []).length &&
              (widget.text['classes'] as List)[index] == _scope))
        picked.add(formats[index]);
      offset = end;
      index++;
    }
    if (picked.isEmpty && _scope == 'all' && _text.styles.isNotEmpty)
      picked.add(_text.styles.first);
    final sizes = picked
        .map((s) => (s['size'] as num? ?? 24).toDouble())
        .toSet();
    final families = picked
        .map((s) => '${EditorSession.map(s['font'])['family'] ?? ''}')
        .toSet();
    final canFormat = _enabled && picked.isNotEmpty;
    final known = _knownFamilies(c.state['fontFamilies'] as List?);
    // The box draws in the text's own face only while the whole text wears
    // one — a mixed text falls back to the panel's, and so does a face the
    // machine does not list.
    final whole = _text.styles
        .map((s) => '${EditorSession.map(s['font'])['family'] ?? ''}')
        .toSet();
    _text.previewFamily = whole.length == 1 && known.contains(whole.single)
        ? whole.single
        : null;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Container(
          constraints: const BoxConstraints(
            minHeight: EditorMetrics.s96,
            maxHeight: EditorMetrics.s200,
          ),
          padding: const EdgeInsets.all(EditorMetrics.s8),
          decoration: BoxDecoration(
            color: EditorTheme.app,
            border: Border.all(
              color: _focus.hasFocus ? EditorTheme.accent : EditorTheme.border,
            ),
          ),
          child: TextField(
            key: const ValueKey('rich-text-content'),
            controller: _text,
            focusNode: _focus,
            enabled: widget.layer['locked'] != true,
            maxLines: null,
            style: const TextStyle(
              fontSize: EditorMetrics.title,
              height: 1.3,
              color: EditorTheme.ink,
            ),
            decoration: const InputDecoration.collapsed(hintText: 'Type here'),
            onChanged: _typed,
          ),
        ),
        const SizedBox(height: EditorMetrics.s6),
        Row(
          children: [
            Expanded(
              child: EditorChoice<String>(
                value: _scope,
                choices: const [
                  MapEntry('all', 'All text'),
                  MapEntry('selection', 'Selection'),
                  MapEntry('hiragana', 'Hiragana'),
                  MapEntry('katakana', 'Katakana'),
                  MapEntry('han', 'Kanji'),
                  MapEntry('latin', 'Latin'),
                ],
                onChanged: !_enabled
                    ? null
                    : (scope) => setState(() {
                        _scope = scope;
                        c.textStyleTarget.value = _target;
                      }),
              ),
            ),
          ],
        ),
        const SizedBox(height: EditorMetrics.s6),
        EditorTooltip(
          message: 'Font',
          child: Container(
            height: EditorMetrics.row,
            alignment: Alignment.centerLeft,
            decoration: BoxDecoration(
              color: EditorTheme.app,
              border: Border.all(color: EditorTheme.line),
            ),
            padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s5),
            child: Autocomplete<String>(
              key: ValueKey('font:${families.join('|')}'),
              initialValue: TextEditingValue(
                text: families.length == 1 ? families.single : '',
              ),
              optionsBuilder: (value) => !canFormat || value.text.isEmpty
                  ? const Iterable<String>.empty()
                  : known
                        .where(
                          (n) => n.toLowerCase().contains(
                            value.text.toLowerCase(),
                          ),
                        )
                        .take(40),
              onSelected: (family) => _format({'family': family}),
              fieldViewBuilder: (context, text, focus, submit) => TextField(
                controller: text,
                focusNode: focus,
                enabled: canFormat,
                style: const TextStyle(
                  fontSize: EditorMetrics.font,
                  color: EditorTheme.ink,
                ),
                decoration: InputDecoration(
                  isDense: true,
                  contentPadding: EdgeInsets.zero,
                  border: InputBorder.none,
                  hintText: families.length > 1 ? 'Mixed fonts' : 'Font',
                ),
                onSubmitted: (_) => submit(),
              ),
            ),
          ),
        ),
        const SizedBox(height: EditorMetrics.s6),
        EditorNumericField(
          key: const ValueKey('rich-text-size'),
          value: sizes.isEmpty ? 24 : sizes.first,
          label: 'Character size',
          unit: 'px',
          mixed: sizes.length > 1,
          min: 1,
          max: 1000,
          enabled: canFormat,
          onPreview: (size) => _format({'size': size, 'preview': true}),
          onCommit: (size) => _format({'size': size}),
          onFinish: () => _finish(_formatTag, false),
          onCancel: () => _finish(_formatTag, true),
        ),
      ],
    );
  }
}
