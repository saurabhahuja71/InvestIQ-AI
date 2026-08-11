import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:investiq_ai/features/ipo/presentation/ipo_analysis_section.dart';
import 'package:investiq_ai/features/ipo/presentation/ipo_providers.dart';

const _id = 'ipo-1';

Map<String, dynamic> _sampleAnalysis() => {
      'ipo_id': _id,
      'company_name': 'Acme Limited',
      'board': 'mainboard',
      'status': 'open',
      'overall_score': 82,
      'long_term_view': 'STRONG POSITIVE',
      'listing_view': 'POSITIVE',
      'confidence': 'HIGH',
      'data_completeness': 87,
      'financial_periods': 'FY2023–FY2025',
      'positive_factors': [
        {'factor': 'Revenue growth', 'detail': 'Revenue grew 25% YoY'},
        {'factor': 'Healthy margins', 'detail': 'PAT margin 18%'},
      ],
      'negative_factors': [
        {'factor': 'Valuation', 'detail': 'P/E of 45x is rich'},
      ],
      'missing_data': ['Promoter holding details', 'GMP'],
      'methodology_version': '1.0',
      'generated_at': '2026-08-11T10:00:00Z',
      'ipo_synced_at': '2026-08-11T09:00:00Z',
      'financials_retrieved_at': '2026-08-11T08:00:00Z',
      'subscription_updated_at': '2026-08-11T09:30:00Z',
    };

Widget _wrap(Map<String, dynamic> analysis) {
  final override = ipoAnalysisProvider.overrideWith(
    (ref, id) async => analysis,
  );
  return ProviderScope(
    overrides: [override],
    child: const MaterialApp(
      home: Scaffold(
        body: SingleChildScrollView(
          child: IpoAnalysisSection(ipoId: _id),
        ),
      ),
    ),
  );
}

void main() {
  testWidgets('renders views, confidence and score', (tester) async {
    await tester.pumpWidget(_wrap(_sampleAnalysis()));
    await tester.pump();

    expect(find.text('InvestIQ IPO View'), findsOneWidget);
    expect(find.text('STRONG POSITIVE'), findsOneWidget);
    expect(find.text('POSITIVE'), findsOneWidget);
    expect(find.text('HIGH'), findsOneWidget);
    expect(find.text('82'), findsOneWidget);
    expect(find.textContaining('Data completeness 87%'), findsWidgets);
    expect(find.textContaining('FY2023'), findsOneWidget);
  });

  testWidgets('renders why factors with details', (tester) async {
    await tester.pumpWidget(_wrap(_sampleAnalysis()));
    await tester.pump();

    expect(find.text('Why InvestIQ Thinks This'), findsOneWidget);
    expect(find.text('Revenue growth'), findsOneWidget);
    expect(find.text('Revenue grew 25% YoY'), findsOneWidget);
    expect(find.text('Valuation'), findsOneWidget);
    expect(find.text('P/E of 45x is rich'), findsOneWidget);
  });

  testWidgets('renders data quality metadata', (tester) async {
    await tester.pumpWidget(_wrap(_sampleAnalysis()));
    await tester.pump();

    expect(find.text('Data Quality'), findsOneWidget);
    expect(find.text('Missing / not evaluated'), findsOneWidget);
    expect(find.text('Promoter holding details'), findsOneWidget);
    expect(find.textContaining('Methodology v1.0'), findsOneWidget);
  });

  testWidgets('renders nothing when analysis is empty', (tester) async {
    await tester.pumpWidget(_wrap(<String, dynamic>{}));
    await tester.pump();

    expect(find.text('InvestIQ IPO View'), findsNothing);
    expect(find.text('Why InvestIQ Thinks This'), findsNothing);
    expect(find.text('Data Quality'), findsNothing);
  });
}
