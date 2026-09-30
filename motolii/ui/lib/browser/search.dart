// The Search capability. Behaviour lives here once: query state, matching, filtering, keyboard.
// Presentation is not here. Each panel decides where a search face goes and how it folds.
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'parts.dart';
import '../theme/neutral.dart';
import '../theme/tokens.dart' show H;
import '../theme/surface.dart' show Dn, Surface;

class SearchCapability extends ChangeNotifier {
  SearchCapability({String query = ''}) : controller = TextEditingController(text: query) {
    controller.addListener(_changed);
  }
  final TextEditingController controller;
  final FocusNode focus = FocusNode();
  /// The panel's own focus, so keys work before the field has focus.
  final FocusNode panel = FocusNode(debugLabel: 'panel');
  String _last = '';

  String get query => controller.text;
  bool get active => query.trim().isNotEmpty;

  void _changed() {
    if (controller.text == _last) return;
    _last = controller.text;
    notifyListeners();
  }

  /// Every whitespace-separated token must appear (case-insensitive) in one of the item's fields.
  List<T> apply<T>(Iterable<T> items, Iterable<String> Function(T) fields) {
    final tokens = query.toLowerCase().split(RegExp(r'\s+')).where((t) => t.isNotEmpty).toList();
    if (tokens.isEmpty) return items.toList();
    return [
      for (final it in items)
        if (tokens.every((t) => fields(it).any((f) => f.toLowerCase().contains(t)))) it,
    ];
  }

  void clear() {
    controller.clear();
  }

  /// Asks whichever face is showing to take focus. A face that is folded away opens itself on this.
  final ValueNotifier<int> requests = ValueNotifier(0);
  void request() {
    requests.value++;
    focus.requestFocus();
  }

  /// "/" and Cmd or Ctrl+F focus the search. Escape clears it, then leaves it.
  KeyEventResult onKey(FocusNode node, KeyEvent e) {
    if (e is! KeyDownEvent) return KeyEventResult.ignored;
    final k = e.logicalKey;
    final mod = HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed;
    if (k == LogicalKeyboardKey.escape && (focus.hasFocus || active)) {
      if (active) {
        clear();
      } else {
        focus.unfocus();
      }
      return KeyEventResult.handled;
    }
    if (!focus.hasFocus && ((mod && k == LogicalKeyboardKey.keyF) || k == LogicalKeyboardKey.slash)) {
      request();
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  Widget keys(Widget child) => Focus(focusNode: panel, onKeyEvent: onKey, child: child);

  @override
  void dispose() {
    controller.removeListener(_changed);
    controller.dispose();
    requests.dispose();
    focus.dispose();
    panel.dispose();
    super.dispose();
  }
}


/// A face for the capability: a text field. One of several possible faces.
class SearchField extends StatelessWidget {
  const SearchField(this.search, this.hint, {super.key, this.trailing, this.height = 26});
  final SearchCapability search;
  final String hint;
  final Widget? trailing;
  final double height;
  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: search,
        builder: (_, __) => Container(
          height: height,
          padding: const EdgeInsets.only(left: 6, right: 4.5),
          decoration: BoxDecoration(color: Surface.raised, border: Border.all(color: search.focus.hasFocus ? N.g44 : Surface.dividerFine), borderRadius: BorderRadius.circular(2)),
          child: Row(children: [
            SizedBox(width: 10, height: 10, child: CustomPaint(painter: magnifier(Surface.muted))),
            const SizedBox(width: 5),
            Expanded(
              child: Stack(alignment: Alignment.centerLeft, children: [
                if (!search.active) Text(hint, softWrap: false, overflow: TextOverflow.clip, style: sans(Dn.nameSize, c: Surface.muted)),
                EditableText(
                  controller: search.controller,
                  focusNode: search.focus,
                  style: sans(Dn.nameSize, c: N.g91),
                  cursorColor: N.g91,
                  backgroundCursorColor: Surface.muted,
                  selectionColor: H.textSelection,
                  maxLines: 1,
                ),
              ]),
            ),
            if (search.active)
              GestureDetector(
                onTap: search.clear,
                child: SizedBox(width: 12, height: 12, child: CustomPaint(painter: _X())),
              ),
            if (trailing != null) ...[const SizedBox(width: 3), trailing!],
          ]),
        ),
      );
}

CustomPainter magnifier(Color c) => _Mag(c);

class _Mag extends CustomPainter {
  _Mag(this.c);
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final p = Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeCap = StrokeCap.round;
    cv.drawCircle(Offset(s.width * .42, s.height * .42), s.width * .3, p);
    cv.drawLine(Offset(s.width * .65, s.height * .65), Offset(s.width * .92, s.height * .92), p..strokeWidth = 1.7);
  }
  @override
  bool shouldRepaint(_Mag o) => o.c != c;
}

class _X extends CustomPainter {
  @override
  void paint(Canvas cv, Size s) {
    final p = Paint()..color = Surface.muted..strokeWidth = 1.4..strokeCap = StrokeCap.round;
    cv.drawLine(Offset(s.width * .3, s.height * .3), Offset(s.width * .7, s.height * .7), p);
    cv.drawLine(Offset(s.width * .7, s.height * .3), Offset(s.width * .3, s.height * .7), p);
  }
  @override
  bool shouldRepaint(_X o) => false;
}

/// Another face: a magnifier key that opens into the field. It stays open while there is a query.
class SearchKeyFace extends StatefulWidget {
  const SearchKeyFace(this.search, this.hint, {super.key});
  final SearchCapability search;
  final String hint;
  @override
  State<SearchKeyFace> createState() => _SearchKeyFaceState();
}

class _SearchKeyFaceState extends State<SearchKeyFace> {
  bool open = false;
  @override
  void initState() {
    super.initState();
    widget.search.focus.addListener(_focus);
  }

  void _focus() {
    if (!widget.search.focus.hasFocus && !widget.search.active && open) setState(() => open = false);
  }

  @override
  void dispose() {
    widget.search.focus.removeListener(_focus);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: widget.search,
        builder: (_, __) {
          if (open || widget.search.active) return SearchField(widget.search, widget.hint);
          return GestureDetector(
            onTap: () {
              setState(() => open = true);
              WidgetsBinding.instance.addPostFrameCallback((_) => widget.search.request());
            },
            child: Container(
              width: 19.5,
              height: 19.5,
              decoration: BoxDecoration(color: Surface.raised, border: Border.all(color: Surface.dividerFine), borderRadius: BorderRadius.circular(2)),
              child: Center(child: SizedBox(width: 10, height: 10, child: CustomPaint(painter: magnifier(Surface.muted)))),
            ),
          );
        },
      );
}

