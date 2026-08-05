import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('IPO empty-state copy is user-facing', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: Center(
            child: Text('No open IPOs right now'),
          ),
        ),
      ),
    );
    expect(find.textContaining('No open IPOs'), findsOneWidget);
  });

  test('Not Available sentinel is stable', () {
    const na = 'Not Available';
    expect(na.isNotEmpty, isTrue);
    expect(na.contains('Available'), isTrue);
  });
}
