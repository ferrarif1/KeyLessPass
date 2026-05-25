import 'dart:convert';
import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keylesspass_desktop/l10n/generated/app_localizations.dart';

void main() {
  test('English and Simplified Chinese resources expose the same message keys',
      () {
    final en = _messageKeys('lib/l10n/app_en.arb');
    final zh = _messageKeys('lib/l10n/app_zh.arb');

    expect(zh.difference(en), isEmpty,
        reason: 'Chinese ARB has keys not present in English.');
    expect(en.difference(zh), isEmpty,
        reason: 'English ARB has keys missing from Chinese.');
  });

  test('generated localizations load core product labels', () async {
    final en = await AppLocalizations.delegate.load(const Locale('en'));
    final zh = await AppLocalizations.delegate.load(const Locale('zh'));

    expect(en.addRecord, 'Add Record');
    expect(en.derivePassword, 'Derive Password');
    expect(en.generateMnemonic, 'Generate mnemonic');
    expect(zh.addRecord, '添加记录');
    expect(zh.derivePassword, '派生密码');
    expect(zh.generateMnemonic, '生成助记短语');
  });
}

Set<String> _messageKeys(String path) {
  final data =
      jsonDecode(File(path).readAsStringSync()) as Map<String, dynamic>;
  return data.keys.where((key) => !key.startsWith('@')).toSet();
}
