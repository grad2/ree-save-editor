import 'package:flutter_test/flutter_test.dart';
import 'package:ree_save_editor_flutter/main.dart';

void main() {
  testWidgets('shows conversion and cancel buttons', (tester) async {
    await tester.pumpWidget(const SaveConverterApp(arguments: []));

    expect(find.text('Конвертировать сейвы'), findsOneWidget);
    expect(find.text('Отмена'), findsOneWidget);
  });
}
