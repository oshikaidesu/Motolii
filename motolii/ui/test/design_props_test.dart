// Design Mode's property model: from a widget call's source, every number, switch, named choice and colour it holds (where it is
// written, whether it is a token, an exception or a literal, which declaration a name leads to), the properties the widget could be given,
// and the one-place text edit that changes each.
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_ui/dev/design_core.dart';
import 'package:motolii_ui/dev/design_props.dart';

const metrics = '''
abstract final class Surface {
  static const base = N.g10, raised = N.g13, muted = N.g56, well = N.g07;
  static double get workRow => px(19);
  static double get sectionGap => px(4.5);
  static double get controlRadius => px(4);
}
abstract final class Dn {
  static double get nameSize => Surface.px(11);
  static double get valueSize => Surface.px(11.5);
  static double get valueHeroSize => Surface.px(14);
  static TextStyle name([Color c = N.g91, FontWeight w = FontWeight.w500]) => _t('Inter', nameSize, c, w, .05);
}
''';
const neutral = '''
abstract final class N {
  static const g07 = Color(0xFF131313);
  static const g10 = Color(0xFF191919);
  static const g13 = Color(0xFF202020);
  static const g20 = Color(0xFF343434);
  static const g56 = Color(0xFF8E8E8E);
  static const g91 = Color(0xFFE7E7E7);
}
''';

Analyzer analyzerFor(Map<String, String> files) {
  final reg = Registry(
    tokens: parseTokens(metrics),
    colors: parseColorTokens(neutral: neutral, metrics: metrics),
  );
  return Analyzer((f) => files[f]!, reg);
}

(int, int) at(String src, String needle) {
  final i = src.indexOf(needle);
  final before = src.substring(0, i);
  final line = '\n'.allMatches(before).length + 1;
  return (line, i - (before.lastIndexOf('\n') + 1) + 1);
}

/// The file after the edit, or null.
String? applied(Analyzer an, Map<String, String> files, Prop p, String value, {Scope scope = Scope.instance}) {
  final c = edit(an, p, value, scope: scope);
  return c == null ? null : (c[p.file] ?? files[p.file]);
}

void main() {
  test('a Text: its style taken apart, what it could be given, and each edit', () {
    const src = '''
Widget build() => Text(
  label,
  style: sans(Dn.nameSize, c: Surface.muted, w: FontWeight.w500),
  overflow: TextOverflow.ellipsis,
);
''';
    final files = {'a.dart': src, 'lib/theme/metrics.dart': metrics};
    final an = analyzerFor(files);
    final (l, c) = at(src, 'Text(');
    final props = {for (final p in an.analyze('a.dart', l, c)) p.name: p};
    expect(props['fontSize']!.origin, Origin.token);
    expect(props['fontSize']!.token, 'Dn.nameSize');
    expect(props['color']!.kind, PropKind.color);
    expect(props['color']!.options, contains('Surface.base'));
    expect(props['color']!.options, contains('N.g20'));
    expect(props['fontWeight']!.kind, PropKind.option);
    expect(props['fontWeight']!.options.length, 9);
    expect(props['overflow']!.options, contains('TextOverflow.fade'));
    expect(props['textAlign']!.origin, Origin.unset);
    expect(props['letterSpacing']!.origin, Origin.unset, reason: 'sans takes `ls:`');
    expect(props['letterSpacing']!.copyWith, isFalse);
    expect(props['height']!.copyWith, isTrue, reason: 'sans has no line height: an instance adds .copyWith');

    expect(applied(an, files, props['fontWeight']!, 'FontWeight.w700'), contains('w: FontWeight.w700)'));
    expect(applied(an, files, props['color']!, 'Surface.base'), contains('c: Surface.base,'));
    expect(applied(an, files, props['overflow']!, 'TextOverflow.fade'), contains('overflow: TextOverflow.fade,'));
    expect(applied(an, files, props['fontSize']!, '12'), contains('sans(12, c:'), reason: 'here: the reference is replaced');
    final tokenEdit = edit(an, props['fontSize']!, '12', scope: Scope.token)!;
    expect(tokenEdit['lib/theme/metrics.dart'], contains('get nameSize => Surface.px(12);'));
    expect(tokenEdit.containsKey('a.dart'), isFalse, reason: 'the token edit touches the token only');
    final align = applied(an, files, props['textAlign']!, 'TextAlign.center')!;
    expect(align, contains('overflow: TextOverflow.ellipsis,\n  textAlign: TextAlign.center,\n)'));
    expect(applied(an, files, props['letterSpacing']!, '0.5'), contains(', ls: 0.5)'));
    expect(applied(an, files, props['height']!, '1.3'), contains('w: FontWeight.w500).copyWith(height: 1.3)'));
  });

  test('a style role: the size is its token, an instance adds .copyWith', () {
    const src = 'Widget b() => Text(t, style: Dn.name(ink));\n';
    final files = {'a.dart': src, 'lib/theme/metrics.dart': metrics};
    final an = analyzerFor(files);
    final (l, c) = at(src, 'Text(');
    final props = {for (final p in an.analyze('a.dart', l, c)) p.name: p};
    final size = props['fontSize']!;
    expect(size.token, 'Dn.nameSize');
    expect(size.origin, Origin.token);
    expect(applied(an, files, size, '12'), contains('Dn.name(ink).copyWith(fontSize: 12)'));
    expect(edit(an, size, '12', scope: Scope.token)!['lib/theme/metrics.dart'], contains('Surface.px(12)'));
    expect(applied(an, files, props['fontWeight']!, 'FontWeight.w700'), contains('Dn.name(ink).copyWith(fontWeight: FontWeight.w700)'));
  });

  test('a copyWith that is already there takes the new argument', () {
    const src = 'Widget b() => Text(t, style: Dn.name(ink).copyWith(fontSize: 13));\n';
    final files = {'a.dart': src, 'lib/theme/metrics.dart': metrics};
    final an = analyzerFor(files);
    final (l, c) = at(src, 'Text(');
    final props = {for (final p in an.analyze('a.dart', l, c)) p.name: p};
    expect(props['fontSize']!.origin, Origin.literal);
    expect(applied(an, files, props['fontSize']!, '12'), contains('copyWith(fontSize: 12)'));
    expect(applied(an, files, props['letterSpacing']!, '0.2'), contains('copyWith(fontSize: 13, letterSpacing: 0.2)'));
  });

  test('a Container: padding by side, the decoration, the radius, the border, the alignment', () {
    const src = '''
Widget b() => Container(
  height: Surface.workRow,
  alignment: Alignment.centerLeft,
  padding: EdgeInsets.symmetric(horizontal: Surface.sectionGap, vertical: 3),
  decoration: BoxDecoration(
    color: Surface.raised,
    borderRadius: BorderRadius.circular(Surface.controlRadius),
    border: Border.all(color: N.g20, width: 1),
  ),
  child: x,
);
''';
    final files = {'a.dart': src, 'lib/theme/metrics.dart': metrics};
    final an = analyzerFor(files);
    final (l, c) = at(src, 'Container(');
    final props = {for (final p in an.analyze('a.dart', l, c)) p.name: p};
    expect(props['height']!.token, 'Surface.workRow');
    expect(props['alignment']!.options.length, 9);
    expect(props['padding.horizontal']!.token, 'Surface.sectionGap');
    expect(props['padding.vertical']!.origin, Origin.literal);
    expect(props['decoration.color']!.kind, PropKind.color);
    expect(props['decoration.borderRadius']!.token, 'Surface.controlRadius');
    expect(props['decoration.border.color']!.token, 'N.g20');
    expect(props['decoration.border.width']!.text, '1');
    expect(applied(an, files, props['alignment']!, 'Alignment.center'), contains('alignment: Alignment.center,'));
    expect(applied(an, files, props['padding.vertical']!, '5'), contains('vertical: 5)'));
    expect(applied(an, files, props['decoration.color']!, 'Surface.well'), contains('color: Surface.well,'));
    expect(applied(an, files, props['decoration.border.width']!, '2'), contains('width: 2)'));
    expect(applied(an, files, props['decoration.borderRadius']!, '6'), contains('BorderRadius.circular(6)'));
    expect(props.containsKey('width'), isTrue, reason: 'a Container can be given a width');
  });

  test('a Row and an Expanded: alignment, size, flex, fit are there even when the source does not say', () {
    const src =
        'Widget b() => Row(mainAxisAlignment: MainAxisAlignment.center, children: [Expanded(flex: 3, child: a), Flexible(child: b)]);\n';
    final files = {'a.dart': src, 'lib/theme/metrics.dart': metrics};
    final an = analyzerFor(files);
    var (l, c) = at(src, 'Row(');
    final row = {for (final p in an.analyze('a.dart', l, c)) p.name: p};
    expect(row['mainAxisAlignment']!.text, 'MainAxisAlignment.center');
    expect(row['crossAxisAlignment']!.origin, Origin.unset);
    expect(row['mainAxisSize']!.options, ['MainAxisSize.min', 'MainAxisSize.max']);
    expect(
      applied(an, files, row['crossAxisAlignment']!, 'CrossAxisAlignment.start'),
      contains('children: [Expanded(flex: 3, child: a), Flexible(child: b)], crossAxisAlignment: CrossAxisAlignment.start)'),
    );
    (l, c) = at(src, 'Expanded(');
    final ex = {for (final p in an.analyze('a.dart', l, c)) p.name: p};
    expect(ex['flex']!.text, '3');
    expect(applied(an, files, ex['flex']!, '5'), contains('Expanded(flex: 5,'));
    (l, c) = at(src, 'Flexible(');
    final fl = {for (final p in an.analyze('a.dart', l, c)) p.name: p};
    expect(fl['flex']!.origin, Origin.unset);
    expect(fl['fit']!.options, ['FlexFit.tight', 'FlexFit.loose']);
    expect(applied(an, files, fl['fit']!, 'FlexFit.tight'), contains('Flexible(child: b, fit: FlexFit.tight)'));
  });

  test('a name is followed to its declaration, a conditional gives both branches, a parameter says who sets it', () {
    const src = '''
Widget build(BuildContext context, {double gap = 6}) {
  final size = widget.hero ? Dn.valueHeroSize : Dn.valueSize;
  final pad = Surface.px(7);
  return Text(t, style: TextStyle(fontSize: size, letterSpacing: gap, height: pad));
}
''';
    final files = {'a.dart': src, 'lib/theme/metrics.dart': metrics};
    final an = analyzerFor(files);
    final (l, c) = at(src, 'Text(');
    final all = an.analyze('a.dart', l, c);
    final sizes = all.where((p) => p.name == 'fontSize').toList();
    expect(sizes.length, 2);
    expect(sizes.map((p) => p.condition), ['if widget.hero', 'else']);
    expect(sizes.map((p) => p.token), ['Dn.valueHeroSize', 'Dn.valueSize']);
    expect(sizes.every((p) => p.via == 'size'), isTrue);
    final gap = all.firstWhere((p) => p.name == 'letterSpacing');
    expect(gap.origin, Origin.literal);
    expect(gap.text, '6', reason: 'a parameter default is a declaration too');
    final pad = all.firstWhere((p) => p.name == 'height');
    expect(pad.origin, Origin.exception);
    expect(applied(an, files, pad, '9'), contains('final pad = Surface.px(9);'), reason: 'the declaration is what is edited');
  });

  test('a stale span is refused; a read-only value has no edit; BorderRadius.only and EdgeInsets.fromLTRB name their parts', () {
    const src = '''
Widget b() => Container(
  padding: EdgeInsets.fromLTRB(1, 2, 3, 4),
  decoration: BoxDecoration(borderRadius: BorderRadius.only(topLeft: Radius.circular(8), bottomRight: Radius.circular(2))),
  child: Text(widget.title, maxLines: lines),
);
''';
    final files = {'a.dart': src, 'lib/theme/metrics.dart': metrics};
    final an = analyzerFor(files);
    final (l, c) = at(src, 'Container(');
    final props = {for (final p in an.analyze('a.dart', l, c)) p.name: p};
    expect(['padding.left', 'padding.top', 'padding.right', 'padding.bottom'].every(props.containsKey), isTrue);
    expect(props['decoration.borderRadius.topLeft']!.text, '8');
    expect(props['decoration.borderRadius.bottomRight']!.text, '2');
    final p = props['padding.top']!;
    files['a.dart'] = src.replaceFirst('fromLTRB(1, 2', 'fromLTRB(11, 2');
    an.forget();
    expect(edit(an, p, '5'), isNull, reason: 'the source moved under the span');
    final (l2, c2) = at(src, 'Text(');
    final t = {for (final q in an.analyze('a.dart', l2, c2)) q.name: q};
    expect(t['maxLines']!.kind, PropKind.readOnly);
    expect(edit(an, t['maxLines']!, '2'), isNull);
  });

  test('colours: a token switches to another token; a literal is its hex; the alias token is re-pointed', () {
    const src =
        'Widget b() => Container(color: Surface.base, decoration: BoxDecoration(color: Color(0xFF123456), border: Border.all(color: N.g20.withValues(alpha: 0.5))));\n';
    final files = {'a.dart': src, 'lib/theme/metrics.dart': metrics, 'lib/theme/neutral.dart': neutral};
    final an = analyzerFor(files);
    final (l, c) = at(src, 'Container(');
    final props = {for (final p in an.analyze('a.dart', l, c)) p.name: p};
    expect(props['color']!.token, 'Surface.base');
    expect(applied(an, files, props['color']!, 'Surface.well'), contains('Container(color: Surface.well,'));
    final tk = edit(an, props['color']!, 'N.g20', scope: Scope.token)!;
    expect(tk['lib/theme/metrics.dart'], contains('base = N.g20'));
    expect(props['decoration.color']!.origin, Origin.literal);
    expect(applied(an, files, props['decoration.color']!, 'Color(0xFF654321)'), contains('color: Color(0xFF654321),'));
    expect(props['decoration.border.color.opacity']!.text, '0.5');
    expect(props['decoration.border.color']!.token, 'N.g20');
    final n = edit(an, props['decoration.border.color']!, 'Color(0xFF111111)', scope: Scope.token)!;
    expect(n['lib/theme/neutral.dart'], contains('g20 = Color(0xFF111111)'));
  });

  test('taking back an argument this session added: inline, on its own line, last, and an emptied copyWith', () {
    Map<String, String> files(String src) => {'a.dart': src, 'lib/theme/metrics.dart': metrics};
    String? roundTrip(String src, String needle, String prop, String value) {
      final f = files(src);
      var an = analyzerFor(f);
      final (l, c) = at(src, needle);
      final p0 = an.analyze('a.dart', l, c).firstWhere((p) => p.name == prop);
      final added = edit(an, p0, value)!['a.dart']!;
      f['a.dart'] = added;
      an = analyzerFor(f);
      final (l2, c2) = at(added, needle);
      final p1 = an.analyze('a.dart', l2, c2).firstWhere((p) => p.name == prop);
      expect(p1.origin, isNot(Origin.unset));
      return removeProp(an, p1)?['a.dart'];
    }

    expect(roundTrip('Widget b() => Expanded(child: a);\n', 'Expanded(', 'flex', '2'), 'Widget b() => Expanded(child: a);\n');
    expect(
      roundTrip('Widget b() => Row(\n  children: [a],\n);\n', 'Row(', 'mainAxisAlignment', 'MainAxisAlignment.center'),
      'Widget b() => Row(\n  children: [a],\n);\n',
    );
    expect(
      roundTrip('Widget b() => Text(t, style: Dn.name(ink));\n', 'Text(', 'fontWeight', 'FontWeight.w700'),
      'Widget b() => Text(t, style: Dn.name(ink));\n',
    );
    expect(
      roundTrip('Widget b() => Text(t, style: sans(12));\n', 'Text(', 'textAlign', 'TextAlign.center'),
      'Widget b() => Text(t, style: sans(12));\n',
    );
  });

  test('a line of the file as it was when the session began, after a line was added', () {
    final disk = {'a.dart': 'one\ntwo(1)\nthree\nfour(2)\n'};
    final s = DesignSession((f) => disk[f]!, (f, t) => disk[f] = t);
    s.apply('a.dart', 'one\ntwo(1)\nadded: 5,\nthree\nfour(3)\n', 'x');
    expect(s.startLine('a.dart', 1), 1);
    expect(s.startLine('a.dart', 2), 2);
    expect(s.startLine('a.dart', 3), isNull, reason: 'written in this session');
    expect(s.startLine('a.dart', 4), 3);
    expect(s.startLine('a.dart', 5), 4, reason: 'a changed line between two matches keeps its place');
  });
}
