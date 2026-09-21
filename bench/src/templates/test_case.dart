
  testWidgets('golden __INDEX__', (tester) async {
    tester.view.physicalSize = const Size(__WIDTH__, __HEIGHT__);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(
      RepaintBoundary(key: const Key('subject'), child: BenchWidget(seed: __INDEX__, marker: marker)),
    );
    if (skipGolden) {
      return;
    }
    await expectLater(find.byKey(const Key('subject')), matchesGoldenFile('goldens/__GOLDEN__.png'));
  });
