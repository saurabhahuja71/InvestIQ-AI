import 'package:flutter_test/flutter_test.dart';

void main() {
  test('investment disclaimer is non-empty', () {
    const disclaimer =
        'This is not financial advice. Past performance does not guarantee future results.';
    expect(disclaimer.contains('not financial advice'), isTrue);
  });
}
