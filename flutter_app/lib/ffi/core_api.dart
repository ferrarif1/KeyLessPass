import '../models/core_models.dart';
import 'rust_core.dart';

class CoreApi {
  CoreApi(this._core);

  final RustCore _core;

  Future<AppStatus> getAppStatus() {
    return _core.invoke('getAppStatus', const {}, AppStatus.fromJson);
  }

  Future<List<CredentialRecord>> listCredentials() {
    return _core.invoke('listCredentials', const {}, (value) {
      return (value as List).map(CredentialRecord.fromJson).toList();
    });
  }

  Future<List<UsbCandidate>> listUsbCandidates() {
    return _core.invoke('listUsbCandidates', const {}, (value) {
      return (value as List).map(UsbCandidate.fromJson).toList();
    });
  }

  Future<void> enroll({required String mnemonic, required String usbPath}) {
    return _core.invoke('enroll', {'mnemonic': mnemonic, 'usbPath': usbPath}, (_) {});
  }

  Future<Map<String, Object?>> generateMnemonic({
    required String language,
    int wordCount = 20,
  }) {
    return _core.invoke(
      'generateMnemonic',
      {'language': language, 'wordCount': wordCount},
      (value) => (value as Map).cast<String, Object?>(),
    );
  }

  Future<CredentialRecord> addCredential({
    required String displayName,
    required String serviceHint,
    required String accountHint,
    String notes = '',
    required int length,
    bool requireUpper = true,
    bool requireLower = true,
    bool requireDigit = true,
    bool requireSymbol = true,
    String forbiddenChars = '"\'`\\/:;?&<>{}[]()|, ',
  }) {
    final descriptor = defaultEncodingDescriptor(
      length: length,
      requireUpper: requireUpper,
      requireLower: requireLower,
      requireDigit: requireDigit,
      requireSymbol: requireSymbol,
      forbiddenChars: forbiddenChars,
    );
    return _core.invoke(
      'addCredential',
      {
        'displayName': displayName,
        'serviceHint': serviceHint,
        'accountHint': accountHint,
        'notes': notes,
        'encodingDescriptor': descriptor,
      },
      CredentialRecord.fromJson,
    );
  }

  Future<CredentialRecord> updateCredentialDisplay(CredentialRecord record) {
    return _core.invoke(
      'updateCredentialDisplay',
      {
        'recordId': record.recordId,
        'version': record.version,
        'displayName': record.displayName,
        'serviceHint': record.serviceHint,
        'accountHint': record.accountHint,
        'notes': record.notes,
        'encodingDescriptor': record.encodingDescriptor,
      },
      CredentialRecord.fromJson,
    );
  }

  Future<Map<String, Object?>> derivePassword({
    required String recordId,
    required int version,
    required String mnemonic,
    required String usbPath,
  }) {
    return _core.invoke(
      'derivePassword',
      {
        'recordId': recordId,
        'version': version,
        'mnemonic': mnemonic,
        'usbPath': usbPath,
      },
      (value) => (value as Map).cast<String, Object?>(),
    );
  }

  Future<CredentialRecord> rotateCredential({
    required CredentialRecord record,
    required int length,
  }) {
    final descriptor = Map<String, Object?>.from(record.encodingDescriptor)..['length'] = length;
    return _core.invoke(
      'rotateCredential',
      {
        'recordId': record.recordId,
        'encodingDescriptor': descriptor,
      },
      CredentialRecord.fromJson,
    );
  }

  Future<void> confirmRotation({required String recordId, required int version}) {
    return _core.invoke('confirmRotation', {'recordId': recordId, 'version': version}, (_) {});
  }

  Future<void> cancelRotation({required String recordId, required int version}) {
    return _core.invoke('cancelRotation', {'recordId': recordId, 'version': version}, (_) {});
  }

  Future<void> recoverUsb({required String mnemonic, required String usbPath}) {
    return _core.invoke('recoverUsb', {'mnemonic': mnemonic, 'usbPath': usbPath}, (_) {});
  }

  Future<void> recoverLocal({required String mnemonic, required String usbPath}) {
    return _core.invoke('recoverLocal', {'mnemonic': mnemonic, 'usbPath': usbPath}, (_) {});
  }

  Future<Map<String, Object?>> verifyUsbPackage({required String mnemonic, required String usbPath}) {
    return _core.invoke(
      'verifyUsbPackage',
      {'mnemonic': mnemonic, 'usbPath': usbPath},
      (value) => (value as Map).cast<String, Object?>(),
    );
  }
}

Map<String, Object?> defaultEncodingDescriptor({
  int length = 18,
  bool requireUpper = true,
  bool requireLower = true,
  bool requireDigit = true,
  bool requireSymbol = true,
  String forbiddenChars = '"\'`\\/:;?&<>{}[]()|, ',
}) {
  final requiredClasses = <Map<String, Object?>>[];
  final positions = [1, 5, 9, 13];
  var index = 0;
  void addClass(bool enabled, String name, String alphabet) {
    if (!enabled) return;
    requiredClasses.add({
      'name': name,
      'alphabet': alphabet,
      'position': positions[index < positions.length ? index : positions.length - 1],
    });
    index += 1;
  }

  addClass(requireUpper, 'upper', 'ABCDEFGHJKLMNPQRSTUVWXYZ');
  addClass(requireLower, 'lower', 'abcdefghijkmnopqrstuvwxyz');
  addClass(requireDigit, 'digit', '23456789');
  addClass(requireSymbol, 'symbol', '!@#\$%*-_=+');

  return {
    'length': length,
    'alphabetProfile': 'enterprise-balanced',
    'allowedAlphabet': 'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789!@#\$%*-_=+',
    'requiredClasses': requiredClasses,
    'fixedPositions': <Object>[],
    'normalization': 'none',
    'forbiddenChars': forbiddenChars,
    'ruleVersion': 1,
  };
}
