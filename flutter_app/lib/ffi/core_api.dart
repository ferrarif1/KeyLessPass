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

  Future<CredentialRecord> addCredential({
    required String displayName,
    required String serviceHint,
    required String accountHint,
    required int length,
  }) {
    final descriptor = defaultEncodingDescriptor()..['length'] = length;
    return _core.invoke(
      'addCredential',
      {
        'displayName': displayName,
        'serviceHint': serviceHint,
        'accountHint': accountHint,
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

  Future<void> recoverUsb({required String mnemonic, required String usbPath}) {
    return _core.invoke('recoverUsb', {'mnemonic': mnemonic, 'usbPath': usbPath}, (_) {});
  }

  Future<void> recoverLocal({required String mnemonic, required String usbPath}) {
    return _core.invoke('recoverLocal', {'mnemonic': mnemonic, 'usbPath': usbPath}, (_) {});
  }
}

Map<String, Object?> defaultEncodingDescriptor() => {
      'length': 18,
      'alphabetProfile': 'enterprise-balanced',
      'allowedAlphabet': 'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789!@#\$%*-_=+',
      'requiredClasses': [
        {'name': 'upper', 'alphabet': 'ABCDEFGHJKLMNPQRSTUVWXYZ', 'position': 1},
        {'name': 'lower', 'alphabet': 'abcdefghijkmnopqrstuvwxyz', 'position': 5},
        {'name': 'digit', 'alphabet': '23456789', 'position': 9},
        {'name': 'symbol', 'alphabet': '!@#\$%*-_=+', 'position': 13},
      ],
      'fixedPositions': <Object>[],
      'normalization': 'none',
      'forbiddenChars': '"\'`\\/:;?&<>{}[]()|, ',
      'ruleVersion': 1,
    };
