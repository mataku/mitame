import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mitame_flutter/mitame_flutter.dart';
import 'package:mitame_flutter_example/login_form.dart';

void main() {
  testWidgets('login form', (tester) async {
    for (final variant in Mitame.matrix({
      'theme': ['light', 'dark'],
      'locale': ['ja', 'en'],
    })) {
      await tester.pumpWidget(
        MaterialApp(
          theme:
              variant['theme'] == 'dark' ? ThemeData.dark() : ThemeData.light(),
          locale: Locale(variant['locale']!),
          home: Scaffold(
            body: Center(child: SizedBox(width: 360, child: LoginForm())),
          ),
        ),
      );
      await tester.pumpAndSettle();
      await expectLater(
        find.byType(LoginForm),
        matchesGoldenFile(Mitame.golden('login_form', variant)),
      );
    }
  });

  testWidgets('login form default', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
          home: Scaffold(
              body: Center(child: SizedBox(width: 360, child: LoginForm())))),
    );
    await expectLater(
        find.byType(LoginForm), matchesGoldenFile('goldens/login_form.png'));
  });
}
