import 'dart:math' as math;
import 'package:flutter/foundation.dart' show ValueListenable, ValueNotifier, listEquals, mapEquals, setEquals;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_effects_k_effects.dart';
import '../../parts/controls.dart';
import '../../tokens.dart';
import 'inspector_parts.dart';
part 'inspector_state_2.dart';

enum Editorial { a, b, c }

enum ModRule { shiftBig, shiftFine }

enum ResetRule { dbl, del, arrow, both, live }   // (live: Ableton Live's, Delete or double-click: its manual 42.4, 42.6)

enum KeyS { off, anim, at }

enum Kind { shape, text, camera, group, several, none }

enum Space { d2, d25, d3 }

const pickGrid = [
  ['v_x', 'v_y', 'v_z'],
  ['v_scale_x', 'v_scale_y', 'v_scale_z'],
  ['v_rotate_x', 'v_rotate_y', 'v_rotate'],
  ['v_skew_x', 'v_skew_y'],
  ['v_opacity'],
  ['depth'],
];

class Cfg {
  const Cfg({
    this.ed = Editorial.a,
    this.mod = ModRule.shiftBig,
    this.rightUp = true,
    this.reset = ResetRule.dbl,
    this.tabs = true,
    this.maybe = false,
    this.look = Look.quiet,
    this.locked = false,
  });
  final Editorial ed;
  final ModRule mod;
  final bool rightUp, tabs, maybe, locked;
  final ResetRule reset;
  final Look look;

  bool get resetDbl => reset == ResetRule.dbl || reset == ResetRule.both || reset == ResetRule.live;
  bool get resetDel => reset == ResetRule.del || reset == ResetRule.live;
  bool get typesOn => reset == ResetRule.live;   // (a digit typed on a field starts its number: Live 42.6 "Type In Value 0…9")
  bool get resetArrow => reset == ResetRule.arrow || reset == ResetRule.both;

  /// One rule for scrub, wheel and arrows (C4): the same keys mean the same thing in every control.
  double mult() {
    final k = HardwareKeyboard.instance;
    if (mod == ModRule.shiftBig) {
      if (k.isShiftPressed) return 10;
      if (k.isAltPressed) return .1;
    } else {
      if (k.isShiftPressed) return .1;
      if (k.isControlPressed) return 10;
    }
    return 1;
  }

  String get modWords => mod == ModRule.shiftBig ? 'Shift x10 · Alt fine' : 'Shift fine · Ctrl x10';
}

/// The editorial option (research/editorial.md): hairline alpha, caps/tracking, type ratio, how sections are told apart.
class Ed {
  const Ed(this.e);
  final Editorial e;
  double get hair => switch (e) {
    Editorial.a => .12,
    Editorial.b => .06,
    Editorial.c => .14,
  };
  Color get rule => Color.fromRGBO(255, 255, 255, hair);
  double get row => e == Editorial.b ? 22 : 24;
  double get gap => switch (e) {
    Editorial.a => 12,
    Editorial.b => 8,
    Editorial.c => 16,
  };

  /// Option A: caps section labels. Option C: caps row labels and big headings. Option B: none.
  String head(String s) => e == Editorial.a ? s.toUpperCase() : s;
  String lab(String s) => s;
  String kick(String s) => e == Editorial.b ? s : s.toUpperCase();
  TextStyle kickStyle(Color c) => e == Editorial.b ? T.label(c) : T.micro(c).copyWith(letterSpacing: .8);
  TextStyle subStyle() => T.name(Grey.g91);
  TextStyle headStyle() => switch (e) {
    Editorial.a => T.micro(Grey.g56).copyWith(letterSpacing: 1.0),
    Editorial.b => T.name(Grey.g63),
    Editorial.c => T.title(Grey.g91).copyWith(fontSize: T.px(13), fontWeight: FontWeight.w400),
  };
  TextStyle labStyle(Color c) => e == Editorial.c ? T.label(c).copyWith(letterSpacing: .3) : T.label(c);
  TextStyle nameStyle() => switch (e) {
    Editorial.a => T.title().copyWith(fontSize: T.px(14), fontWeight: FontWeight.w500),
    Editorial.b => T.title().copyWith(fontSize: T.px(13), fontWeight: FontWeight.w500),
    Editorial.c => T.title().copyWith(fontSize: T.px(17), fontWeight: FontWeight.w300),
  };
}

/// The property a value is part of, whose key mark is theirs together: a document's x, y, z are its position; its
/// scale_x .. scale_z its scale, rotate_x, rotate_y and rotate its rotation, anchor_x .. anchor_z its anchor.
String propOf(String id) => _place.hasMatch(id) ? 'v_position' : _axis.firstMatch(id)?.group(1) ?? id;
final _place = RegExp(r'^v_[xyz]$'), _axis = RegExp(r'^(v_\w+?)_[xyz]$');   // (made once: asked hundreds of times a picture)
