// The window's keys as Ableton Live 12 has them (its reference manual, chapter 42 "Live Keyboard Shortcuts":
// www.ableton.com/en/manual/live-keyboard-shortcuts/), each said beside the section it comes from; where Live has no
// such key, the tool that has one is named. The commands here are what each does to the window's state; keys.dart
// says which key says which, and the maker's keybindings.json may say otherwise.
import 'package:flutter/foundation.dart' show ValueNotifier;

import 'key_drag.dart';
import 'ws.dart';

/// What the timeline is asked to do with how it looks at the document (its zoom, its loop): it keeps those itself.
final wsTimelineAsk = ValueNotifier<(String, int)>(('', 0));
void wsAskTimeline(String what) => wsTimelineAsk.value = (what, wsTimelineAsk.value.$2 + 1);

/// The picked thing's name asked for, in its row (Live: Cmd+R renames, 42.5 and 42.19).
final wsAskRename = ValueNotifier<int>(0);

/// The mark (locator) picked by a click on its flag: Cmd+R names it, Delete takes it away (Live 6.4: a locator is
/// selected, then renamed or deleted like anything else).
final wsMarkPicked = ValueNotifier<(double, String)?>(null);

/// Where playing began (Live 42.20: Space plays from the start marker and stops back there; Shift+Space goes on from
/// where it stopped).
int? _playFrom;

Map<String, void Function()> wsAbletonCommands(Ws ws) {
  void step(int by) {
    ws.playing = false;
    ws.frame = ws.frame + by;
  }

  /// The playhead on the nearest stop that way (After Effects' J and K).
  void hop(int way) {
    final all = ws.stops().where((f) => (f - ws.frame) * way > 0).toList()..sort();
    if (all.isNotEmpty) step((way > 0 ? all.first : all.last) - ws.frame);
  }

  /// The picked keys, else the picked thing's bar, carried [by] frames (Live 42.16: Nudge Selection Left/Right); with
  /// nothing picked, the playhead (Live moves its insert marker so).
  void nudge(int by) {
    final m = ws.inside;
    if (m == null) return;
    if (ws.keys.isNotEmpty) {
      wsSlideKeys(ws, ws.keys.toList(), by);
      ws.pickKeys([for (final k in ws.keys) (layer: k.layer, frame: (k.frame + by).clamp(0, Ws.duration))]);   // (still picked, where they now are)
      return;
    }
    final t = int.tryParse(ws.selected ?? '');
    if (t == null) return step(by);
    m.take(t);
    m.carry('bar', by / Ws.fps);
    ws.after(m.later('let_go', number: 1));
  }

  /// The row above or below the picked one picked (Live 42.15/42.16: the arrow keys move between tracks).
  void row(int by) {
    if (ws.layers.isEmpty) return;
    final i = ws.layers.indexWhere((l) => l.id == ws.selected);
    ws.select(ws.layers[(i < 0 ? 0 : i + by).clamp(0, ws.layers.length - 1)].id);
  }

  return {
    'play.fromStart': () {   // (Live 42.20 Space: play from the start marker / stop, back at it)
      if (ws.playing) {
        ws.playing = false;
        if (_playFrom case final f?) ws.frame = f;
      } else {
        _playFrom = ws.frame;
        ws.playing = true;
      }
    },
    'play.continue': () {   // (Live 42.20 Shift+Space: continue play from the stop point)
      _playFrom = null;
      ws.playing = !ws.playing;
    },
    'nudge.left': () => nudge(-1),
    'nudge.right': () => nudge(1),
    'nudge.left10': () => nudge(-10),
    'nudge.right10': () => nudge(10),
    'layer.previous': () => row(-1),
    'layer.next': () => row(1),
    'thing.rename': () => ws.selected == null && wsMarkPicked.value == null ? null : wsAskRename.value++,   // (Live 42.5/42.19 Cmd+R: the picked track, or the picked locator)
    'thing.solo': () {   // (Live 42.15/42.19 S: solo selected tracks)
      final t = int.tryParse(ws.selected ?? ''), m = ws.inside;
      if (t == null || m == null) return;
      ws.after(m.later('set', thing: t, name: 'solo', number: ws.layer?.solo == true ? 0 : 1));
    },
    'thing.lanes': () {   // (After Effects' U: the picked layers' keyed properties shown, pressed again gone)
      final ids = {...ws.many, ?ws.selected};
      if (ids.isNotEmpty) ws.twirl(ids);
    },
    'loop.toggle': () => wsAskTimeline('loop'),   // (Live 42.16 Cmd+L: toggle loop brace)
    'zoom.in': () => wsAskTimeline('in'),   // (Live 42.9 +: zoom in time ruler)
    'zoom.out': () => wsAskTimeline('out'),   // (Live 42.9 -)
    'zoom.selection': () => wsAskTimeline('selection'),   // (Live 42.16 Z: zoom to arrangement time selection)
    'zoom.back': () => wsAskTimeline('back'),   // (Live 42.16 X: zoom back)
    'zoom.width': () => wsAskTimeline('fit'),   // (Live 42.16 W: optimize arrangement width)
    'frame.previous': () => step(-1),   // (Live has no frame step; Rive's and Jitter's , and .)
    'frame.next': () => step(1),
    'frame.previousStop': () => hop(-1),
    'frame.nextStop': () => hop(1),
    'frame.back10': () => step(-10),
    'frame.forward10': () => step(10),
  };
}
