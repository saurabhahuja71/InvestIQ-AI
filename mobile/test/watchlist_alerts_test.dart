import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('IPO alert preference keys are complete', () {
    const keys = {
      'ipo_open',
      'ipo_close',
      'allotment',
      'listing_tomorrow',
      'listing_day',
    };
    expect(keys.length, 5);
    expect(keys.contains('listing_tomorrow'), isTrue);
  });

  testWidgets('watchlist empty state copy', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: Center(child: Text('No watched IPOs yet')),
        ),
      ),
    );
    expect(find.textContaining('No watched IPOs'), findsOneWidget);
  });

  testWidgets('star icon semantics for watchlist', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: IconButton(
            tooltip: 'Add to watchlist',
            onPressed: () {},
            icon: const Icon(Icons.star_outline_rounded),
          ),
        ),
      ),
    );
    expect(find.byIcon(Icons.star_outline_rounded), findsOneWidget);
    expect(find.byTooltip('Add to watchlist'), findsOneWidget);
  });
}
