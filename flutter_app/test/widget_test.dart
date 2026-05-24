import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:keylesspass_desktop/app/keylesspass_app.dart';

void main() {
  testWidgets('KeylessPass desktop app builds', (WidgetTester tester) async {
    await tester.binding.setSurfaceSize(const Size(1280, 800));
    await tester.pumpWidget(const KeylessPassDesktopApp());
    await tester.pumpAndSettle();
    expect(find.text('KeyLessPass'), findsWidgets);
    addTearDown(() => tester.binding.setSurfaceSize(null));
  });

  testWidgets('settings language switch updates visible labels', (WidgetTester tester) async {
    await tester.binding.setSurfaceSize(const Size(1280, 800));
    await tester.pumpWidget(const KeylessPassDesktopApp());
    await tester.pumpAndSettle();

    await _showNavLabel(tester, 'Settings');
    await _tapNavLabel(tester, 'Settings');
    await tester.pumpAndSettle();
    expect(find.text('Language'), findsWidgets);

    await tester.tap(find.text('System default').first);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Simplified Chinese').last);
    await tester.pumpAndSettle();

    expect(find.text('设置'), findsWidgets);
    expect(find.text('语言'), findsWidgets);
    addTearDown(() => tester.binding.setSurfaceSize(null));
  });

  testWidgets('main navigation exposes product workflows', (WidgetTester tester) async {
    await tester.binding.setSurfaceSize(const Size(1280, 800));
    await tester.pumpWidget(const KeylessPassDesktopApp());
    await tester.pumpAndSettle();

    for (final label in ['Dashboard', 'Setup', 'Records', 'USB Device', 'Security', 'Settings', 'About']) {
      await _showNavLabel(tester, label);
      expect(find.text(label), findsWidgets);
    }
    addTearDown(() => tester.binding.setSurfaceSize(null));
  });
}

Future<void> _showNavLabel(WidgetTester tester, String label) async {
  final nav = find.byKey(const ValueKey('main-navigation'));
  for (var i = 0; i < 8 && find.text(label).hitTestable().evaluate().isEmpty; i++) {
    await tester.drag(nav, const Offset(0, -160));
    await tester.pumpAndSettle();
  }
}

Future<void> _tapNavLabel(WidgetTester tester, String label) async {
  await _showNavLabel(tester, label);
  final tile = find.ancestor(of: find.text(label).hitTestable(), matching: find.byType(ListTile)).first;
  await tester.tap(tile);
  await tester.pumpAndSettle();
}
