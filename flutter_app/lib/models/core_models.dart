class PlatformSecurityStatus {
  const PlatformSecurityStatus({
    required this.platform,
    required this.provider,
    required this.systemKeystoreAvailable,
    required this.degraded,
    required this.message,
  });

  final String platform;
  final String provider;
  final bool systemKeystoreAvailable;
  final bool degraded;
  final String message;

  factory PlatformSecurityStatus.fromJson(Map<String, Object?> json) {
    return PlatformSecurityStatus(
      platform: json['platform'] as String? ?? '-',
      provider: json['provider'] as String? ?? '-',
      systemKeystoreAvailable:
          json['systemKeystoreAvailable'] as bool? ?? false,
      degraded: json['degraded'] as bool? ?? true,
      message: json['message'] as String? ?? '',
    );
  }
}

class AppStatus {
  const AppStatus({
    required this.enrolled,
    required this.securityStatus,
    this.config,
    this.recovery,
  });

  final bool enrolled;
  final PlatformSecurityStatus securityStatus;
  final Map<String, Object?>? config;
  final Map<String, Object?>? recovery;

  String get passwordDerivationAlgorithm =>
      config?['passwordDerivationAlgorithm'] as String? ?? 'hkdf-sha256';

  bool get hasStoredPasswordDerivationAlgorithm =>
      config?.containsKey('passwordDerivationAlgorithm') ?? false;

  factory AppStatus.fromJson(Object? value) {
    final json = (value as Map).cast<String, Object?>();
    return AppStatus(
      enrolled: json['enrolled'] as bool? ?? false,
      securityStatus: PlatformSecurityStatus.fromJson(
        (json['securityStatus'] as Map?)?.cast<String, Object?>() ?? const {},
      ),
      config: (json['config'] as Map?)?.cast<String, Object?>(),
      recovery: (json['recovery'] as Map?)?.cast<String, Object?>(),
    );
  }
}

class CredentialRecord {
  const CredentialRecord({
    required this.recordId,
    required this.recordSeq,
    required this.displayName,
    required this.serviceHint,
    required this.accountHint,
    required this.notes,
    required this.version,
    required this.salt,
    required this.encodingDescriptor,
    required this.state,
    required this.createdAt,
    required this.updatedAt,
  });

  final String recordId;
  final int recordSeq;
  final String displayName;
  final String serviceHint;
  final String accountHint;
  final String notes;
  final int version;
  final String salt;
  final Map<String, Object?> encodingDescriptor;
  final String state;
  final String createdAt;
  final String updatedAt;

  factory CredentialRecord.fromJson(Object? value) {
    final json = (value as Map).cast<String, Object?>();
    return CredentialRecord(
      recordId: json['recordId'] as String,
      recordSeq: (json['recordSeq'] as num).toInt(),
      displayName: json['displayName'] as String? ?? '',
      serviceHint: json['serviceHint'] as String? ?? '',
      accountHint: json['accountHint'] as String? ?? '',
      notes: json['notes'] as String? ?? '',
      version: (json['version'] as num).toInt(),
      salt: json['salt'] as String? ?? '',
      encodingDescriptor:
          (json['encodingDescriptor'] as Map).cast<String, Object?>(),
      state: json['state'] as String? ?? 'active',
      createdAt: json['createdAt'] as String? ?? '',
      updatedAt: json['updatedAt'] as String? ?? '',
    );
  }

  CredentialRecord copyWith({
    String? displayName,
    String? serviceHint,
    String? accountHint,
    String? notes,
  }) {
    return CredentialRecord(
      recordId: recordId,
      recordSeq: recordSeq,
      displayName: displayName ?? this.displayName,
      serviceHint: serviceHint ?? this.serviceHint,
      accountHint: accountHint ?? this.accountHint,
      notes: notes ?? this.notes,
      version: version,
      salt: salt,
      encodingDescriptor: encodingDescriptor,
      state: state,
      createdAt: createdAt,
      updatedAt: updatedAt,
    );
  }
}

class UsbCandidate {
  const UsbCandidate({
    required this.rootPath,
    required this.packagePath,
    required this.readable,
    required this.message,
  });

  final String rootPath;
  final String packagePath;
  final bool readable;
  final String message;

  factory UsbCandidate.fromJson(Object? value) {
    final json = (value as Map).cast<String, Object?>();
    return UsbCandidate(
      rootPath: json['rootPath'] as String? ?? '',
      packagePath: json['packagePath'] as String? ?? '',
      readable: json['readable'] as bool? ?? false,
      message: json['message'] as String? ?? '',
    );
  }
}

class UsbCdrStatus {
  const UsbCdrStatus({
    required this.status,
    required this.backupPath,
    required this.localRecordCount,
    required this.usbRecordCount,
    required this.message,
  });

  final String status;
  final String backupPath;
  final int localRecordCount;
  final int usbRecordCount;
  final String message;

  bool get needsAction => status != 'consistent';

  factory UsbCdrStatus.fromJson(Object? value) {
    final json = (value as Map).cast<String, Object?>();
    return UsbCdrStatus(
      status: json['status'] as String? ?? 'invalid',
      backupPath: json['backupPath'] as String? ?? '',
      localRecordCount: (json['localRecordCount'] as num?)?.toInt() ?? 0,
      usbRecordCount: (json['usbRecordCount'] as num?)?.toInt() ?? 0,
      message: json['message'] as String? ?? '',
    );
  }
}

class LicenseStatus {
  const LicenseStatus({
    required this.status,
    required this.authorized,
    required this.commercialDeviceId,
    required this.deviceFingerprint,
    required this.features,
    required this.message,
    this.organizationId,
    this.organizationName,
    this.licenseId,
    this.grantId,
    this.plan,
    this.seatLabel,
    this.validUntil,
  });

  final String status;
  final bool authorized;
  final String commercialDeviceId;
  final String deviceFingerprint;
  final String? organizationId;
  final String? organizationName;
  final String? licenseId;
  final String? grantId;
  final String? plan;
  final String? seatLabel;
  final String? validUntil;
  final List<String> features;
  final String message;

  factory LicenseStatus.fromJson(Object? value) {
    final json = (value as Map).cast<String, Object?>();
    return LicenseStatus(
      status: json['status'] as String? ?? 'unlicensed',
      authorized: json['authorized'] as bool? ?? false,
      commercialDeviceId: json['commercialDeviceId'] as String? ?? '',
      deviceFingerprint: json['deviceFingerprint'] as String? ?? '',
      organizationId: json['organizationId'] as String?,
      organizationName: json['organizationName'] as String?,
      licenseId: json['licenseId'] as String?,
      grantId: json['grantId'] as String?,
      plan: json['plan'] as String?,
      seatLabel: json['seatLabel'] as String?,
      validUntil: json['validUntil'] as String?,
      features: ((json['features'] as List?) ?? const [])
          .map((value) => '$value')
          .toList(),
      message: json['message'] as String? ?? '',
    );
  }
}
