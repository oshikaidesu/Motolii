import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/panel_controls.dart';

/// The state machine every continuous control shares, on a bare control.
class _Probe extends StatefulWidget {
  const _Probe(this.log, {this.watches = true});
  final List<String> log;
  final bool watches;
  @override
  State<_Probe> createState() => _ProbeState();
}

class _ProbeState extends State<_Probe>
    with WidgetsBindingObserver, EditorDragSession<int, _Probe> {
  bool discard = false;
  @override
  bool get watchesWindow => widget.watches;
  @override
  bool get discardsPreview => discard;
  @override
  Future<void> sendPreview(int value) async {
    widget.log.add('preview $value');
    await null;
  }

  @override
  Future<void> commitDrag() async => widget.log.add('commit');
  @override
  Future<void> cancelDrag() async => widget.log.add('cancel');
  @override
  void dragStopped(bool cancel) => widget.log.add('stopped cancel=$cancel');
  @override
  void dragSettled() => widget.log.add('settled');
  @override
  Widget build(BuildContext context) => const SizedBox();
}

void main() {
  testWidgets(
    'preview, then commit; the last value wins while one is in flight',
    (tester) async {
      final log = <String>[];
      await tester.pumpWidget(_Probe(log));
      final s = tester.state<_ProbeState>(find.byType(_Probe));
      s.beginDrag();
      expect(s.dragging, isTrue);
      s.queue.add(1);
      s.queue.add(2);
      s.queue.add(3);
      final done = s.endDrag(false);
      expect(s.dragging, isFalse);
      expect(s.ending, isTrue);
      await done;
      expect(s.ending, isFalse);
      expect(log, [
        'preview 1',
        'stopped cancel=false',
        'preview 3',
        'commit',
        'settled',
      ]);
    },
  );

  testWidgets('cancel throws the queued preview away', (tester) async {
    final log = <String>[];
    await tester.pumpWidget(_Probe(log));
    final s = tester.state<_ProbeState>(find.byType(_Probe));
    s.beginDrag();
    s.queue.add(1);
    s.queue.add(2);
    await s.endDrag(true);
    expect(log, ['preview 1', 'stopped cancel=true', 'cancel', 'settled']);
    log.clear();
    await s.endDrag(true);
    expect(log, isEmpty, reason: 'ending twice does nothing');
  });

  testWidgets('a commit that discards runs the cancel path', (tester) async {
    final log = <String>[];
    await tester.pumpWidget(_Probe(log));
    final s = tester.state<_ProbeState>(find.byType(_Probe));
    s.beginDrag();
    s.discard = true;
    s.queue.add(1);
    s.queue.add(2);
    await s.endDrag(false);
    expect(log, ['preview 1', 'stopped cancel=false', 'cancel', 'settled']);
  });

  testWidgets('the window going away cancels, unless the control opts out', (
    tester,
  ) async {
    final log = <String>[];
    await tester.pumpWidget(_Probe(log));
    final s = tester.state<_ProbeState>(find.byType(_Probe));
    s.beginDrag();
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
    await tester.pump();
    expect(log, ['stopped cancel=true', 'cancel', 'settled']);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);

    // A fresh control, not the first one re-configured: whether a control
    // watches the window is decided once, when it mounts.
    await tester.pumpWidget(const SizedBox());
    final quiet = <String>[];
    await tester.pumpWidget(_Probe(quiet, watches: false));
    final q = tester.state<_ProbeState>(find.byType(_Probe));
    q.beginDrag();
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
    await tester.pump();
    expect(q.dragging, isTrue, reason: 'opted out: only the drag itself ends');
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await q.endDrag(true);
  });

  testWidgets('going away mid-drag cancels without touching the tree', (
    tester,
  ) async {
    final log = <String>[];
    await tester.pumpWidget(_Probe(log));
    final s = tester.state<_ProbeState>(find.byType(_Probe));
    s.beginDrag();
    s.queue.add(1);
    await tester.pumpWidget(const SizedBox());
    await tester.pump();
    expect(log, ['preview 1', 'cancel']);
  });
}
