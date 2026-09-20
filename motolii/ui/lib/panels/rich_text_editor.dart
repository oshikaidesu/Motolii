import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:flutter/services.dart';

import '../foundation/metrics.dart';
import '../foundation/theme.dart';
import '../foundation/panel_controls.dart';
import '../session/editor_session.dart';
import '../foundation/leaves.dart';

class StyledTextController extends TextEditingController {
  StyledTextController({required super.text});
  List<Map<String, dynamic>> styles = [], runs = [];

  /// The families the machine listed; any other name would only cost a
  /// lookup and give back the same fallback face.
  Set<String> known = const {};

  /// The face of the text's first style when the machine knows it — the one
  /// the box borrows for its base and for the caret's own line.
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

  /// The box is a preview of proportion, not of the composition: the first
  /// style lands at the panel's own size and every other run keeps its share
  /// of it, in its own face when the machine knows it. A run is one span, so
  /// a keystroke lays out a handful, not one per grapheme; while the IME is
  /// composing even that stands aside and the whole text is one span.
  @override
  TextSpan buildTextSpan({
    required BuildContext context,
    TextStyle? style,
    required bool withComposing,
  }) {
    final base = (style ?? const TextStyle()).copyWith(
      fontFamily: previewFamily,
      fontSize: EditorMetrics.title,
    );
    if (withComposing && value.composing.isValid)
      return super.buildTextSpan(
        context: context,
        style: base,
        withComposing: withComposing,
      );
    final unit = (styles.firstOrNull?['size'] as num?)?.toDouble() ?? 1;
    final perCharacter = characterStyles();
    final children = <TextSpan>[];
    final buffer = StringBuffer();
    Map<String, dynamic>? run;
    var lit = false;
    void flush() {
      if (buffer.isEmpty) return;
      final family = '${EditorSession.map(run?['font'])['family'] ?? ''}';
      final size = (run?['size'] as num?)?.toDouble() ?? unit;
      children.add(
        TextSpan(
          text: buffer.toString(),
          style: TextStyle(
            fontFamily: known.contains(family) ? family : null,
            fontSize: EditorMetrics.title * size / (unit < 1 ? 1 : unit),
            backgroundColor: lit ? EditorTheme.of(context).select : null,
            color: lit ? EditorTheme.of(context).selectInk : null,
          ),
        ),
      );
      buffer.clear();
    }

    var index = 0;
    for (final g in text.characters) {
      final here = perCharacter[index];
      final on = highlighted.contains(index++);
      if (run != null && (here != run || on != lit)) flush();
      run = here;
      lit = on;
      buffer.write(g);
    }
    flush();
    return TextSpan(style: base, children: children);
  }
}

/// What the text says, drawn in proportion. The face, the size of a class of
/// characters and the alignment are the Fonts shelf's business; the box only
/// highlights the class the shelf is dressing. No span of characters is ever
/// held: a range is the animator's business.
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
  String _baseline = '';

  /// The characters the Fonts shelf is dressing for this layer; `all` unless
  /// the shelf said otherwise. Only the highlight reads it.
  String get _scope {
    final held = c.textStyleTarget.value;
    return held != null && held['layer'] == widget.layer['id']
        ? '${held['scope'] ?? 'all'}'
        : 'all';
  }

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
  Map<String, dynamic> get _target => {
    ...?c.textStyleTarget.value?['layer'] == widget.layer['id']
        ? c.textStyleTarget.value
        : null,
    'layer': widget.layer['id'],
    'scope': _scope,
    'text': _text.text,
  };
  void _scopeChanged() {
    if (mounted) setState(() {});
  }

  @override
  void initState() {
    super.initState();
    _baseline = _text.text;
    c.pendingEditors.add(_flush);
    c.textStyleTarget.addListener(_scopeChanged);
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

  @override
  void didUpdateWidget(covariant RichTextEditor old) {
    super.didUpdateWidget(old);
    if (old.controller != c) {
      old.controller.pendingEditors.remove(_flush);
      old.controller.textStyleTarget.removeListener(_scopeChanged);
      c.pendingEditors.add(_flush);
      c.textStyleTarget.addListener(_scopeChanged);
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
    c.textStyleTarget.removeListener(_scopeChanged);
    if (_dirty && !_committing)
      _previews.finish(true, () => _finish(_inputTag, true));
    _text.removeListener(_changed);
    _focus.removeListener(_focused);
    _text.dispose();
    _focus.dispose();
    super.dispose();
  }

  /// A class label from the machine (`latin-upper`) names a script and a
  /// case; the scope may be either half.
  static bool inScope(String kind, String scope) {
    final dash = kind.indexOf('-');
    return dash < 0
        ? kind == scope
        : kind.substring(0, dash) == scope || kind.substring(dash + 1) == scope;
  }

  @override
  Widget build(BuildContext context) {
    final classes = widget.text['classes'] as List? ?? const [];
    _text.highlighted = {
      for (final (i, kind) in classes.indexed)
        if (inScope('$kind', _scope)) i,
    };
    final known = _knownFamilies(c.state['fontFamilies'] as List?);
    final first =
        '${EditorSession.map(_text.styles.firstOrNull?['font'])['family'] ?? ''}';
    _text
      ..known = known
      ..previewFamily = known.contains(first) ? first : null;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        EditorFieldFrame(
          focus: _focus,
          height: null,
          minHeight: EditorMetrics.s96,
          maxHeight: EditorMetrics.s200,
          padding: const EdgeInsets.all(EditorMetrics.s8),
          child: EditorTextField(
            key: const ValueKey('rich-text-content'),
            controller: _text,
            focusNode: _focus,
            enabled: widget.layer['locked'] != true,
            maxLines: null,
            style: TextStyle(
              fontSize: EditorMetrics.title,
              height: 1.3,
              color: EditorTheme.of(context).ink,
            ),
            hint: 'Type here',
            onChanged: _typed,
          ),
        ),
      ],
    );
  }
}
