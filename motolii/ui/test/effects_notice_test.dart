import 'package:flutter_test/flutter_test.dart';
import '../lib/app/editor_window.dart';

void main() {
  test('the reasons the shelf refused an effect become one line', () {
    expect(effectsNotice({}), '');
    expect(effectsNotice({'catalogErrors': []}), '');
    expect(
      effectsNotice({
        'catalogErrors': ['glow: expected ; at line 4', 'twist: duplicate effect ID'],
      }),
      'Effects: glow: expected ; at line 4; twist: duplicate effect ID',
    );
  });
}
