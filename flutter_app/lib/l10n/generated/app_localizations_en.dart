// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appName => 'KeyLessPass';

  @override
  String get appSubtitle => 'Local password derivation client';

  @override
  String get dashboard => 'Dashboard';

  @override
  String get records => 'Records';

  @override
  String get addRecord => 'Add Record';

  @override
  String get derivePassword => 'Derive Password';

  @override
  String get rotation => 'Rotation';

  @override
  String get recovery => 'Recovery';

  @override
  String get usbDevice => 'USB Device';

  @override
  String get security => 'Security';

  @override
  String get settings => 'Settings';

  @override
  String get about => 'About';

  @override
  String get initialized => 'Initialized';

  @override
  String get notInitialized => 'Setup required';

  @override
  String get platformProtected => 'Platform protection';

  @override
  String get reducedProtection => 'Reduced protection';

  @override
  String get localOnly => 'Local only';

  @override
  String get refresh => 'Refresh';

  @override
  String get status => 'Status';

  @override
  String get activeRecords => 'Active records';

  @override
  String get usbStatus => 'USB status';

  @override
  String get integrity => 'Integrity';

  @override
  String get lastCheck => 'Last check';

  @override
  String get ready => 'Ready';

  @override
  String get needsSetup => 'Needs setup';

  @override
  String get available => 'Available';

  @override
  String get notFound => 'Not found';

  @override
  String get ok => 'OK';

  @override
  String get actionAddRecord => 'Add Record';

  @override
  String get actionDerive => 'Derive';

  @override
  String get actionRotate => 'Rotate';

  @override
  String get actionRecovery => 'Recovery';

  @override
  String get quickActions => 'Quick actions';

  @override
  String get dashboardSubtitle =>
      'Manage records and derive passwords without storing a password vault.';

  @override
  String get safetyReminder =>
      'Passwords are derived on demand and are not saved.';

  @override
  String get search => 'Search';

  @override
  String get filter => 'Filter';

  @override
  String get all => 'All';

  @override
  String get active => 'Active';

  @override
  String get pending => 'Pending';

  @override
  String get retired => 'Retired';

  @override
  String get conflict => 'Conflict';

  @override
  String get error => 'Error';

  @override
  String get displayName => 'Display Name';

  @override
  String get serviceHint => 'Service Hint / URL';

  @override
  String get accountHint => 'Account Hint';

  @override
  String get notes => 'Notes';

  @override
  String get version => 'Version';

  @override
  String get state => 'State';

  @override
  String get lastUpdated => 'Updated';

  @override
  String get lastUsed => 'Last used';

  @override
  String get passwordRule => 'Password Rule';

  @override
  String get length => 'Length';

  @override
  String get requiredClasses => 'Required character classes';

  @override
  String get requireUppercase => 'Uppercase';

  @override
  String get requireLowercase => 'Lowercase';

  @override
  String get requireDigits => 'Digits';

  @override
  String get requireSymbols => 'Symbols';

  @override
  String get forbiddenCharacters => 'Forbidden characters';

  @override
  String get save => 'Save';

  @override
  String get cancel => 'Cancel';

  @override
  String get close => 'Close';

  @override
  String get editMetadata => 'Edit Metadata';

  @override
  String get viewIntegrity => 'View Integrity';

  @override
  String get noRecords => 'No records yet.';

  @override
  String get selectRecord => 'Select a record';

  @override
  String get recordCreated => 'Record created.';

  @override
  String get metadataSaved => 'Metadata saved.';

  @override
  String get metadataDoesNotChangePassword =>
      'Display fields do not change the derived password.';

  @override
  String get ruleChangeRequiresRotation =>
      'Changing password rules requires a new version.';

  @override
  String get recordDetails => 'Record details';

  @override
  String get advancedDetails => 'Advanced details';

  @override
  String get recordSequence => 'Record sequence';

  @override
  String get recordId => 'Record ID';

  @override
  String get salt => 'Salt';

  @override
  String get encodingRule => 'Encoding rule';

  @override
  String get mnemonicPhrase => 'Mnemonic phrase';

  @override
  String get mnemonicLanguage => 'Mnemonic language';

  @override
  String get englishMnemonic => 'English';

  @override
  String get chineseMnemonic => 'Simplified Chinese';

  @override
  String get generateMnemonic => 'Generate mnemonic';

  @override
  String get generatedMnemonicReady => 'Mnemonic generated locally.';

  @override
  String get mnemonicGeneratedLocally =>
      'Generated on this device only. Save it offline; KeyLessPass will not store it.';

  @override
  String get showMnemonic => 'Show mnemonic';

  @override
  String get hideMnemonic => 'Hide mnemonic';

  @override
  String get wordCount => 'Word count';

  @override
  String get usbPath => 'USB path';

  @override
  String get chooseUsb => 'Choose USB folder';

  @override
  String get showPassword => 'Show password';

  @override
  String get hidePassword => 'Hide password';

  @override
  String get copy => 'Copy';

  @override
  String get deriveAndCopy => 'Derive and copy';

  @override
  String get deriveMode => 'Verification mode';

  @override
  String get deriveModeThisDevice => 'This device';

  @override
  String get deriveModeUsbRecovery => 'USB recovery';

  @override
  String get deriveModeThisDeviceHelp =>
      'Use this device, the selected USB package, and the mnemonic.';

  @override
  String get deriveModeUsbRecoveryHelp =>
      'Rebuild local material from USB and mnemonic before deriving.';

  @override
  String get passwordCopied =>
      'Password copied. It will be cleared automatically.';

  @override
  String get passwordHidden => 'Password hidden';

  @override
  String get clipboardClearFailed =>
      'Clipboard clearing could not be confirmed.';

  @override
  String get clipboardTimeout => 'Clipboard timeout';

  @override
  String get seconds => 'seconds';

  @override
  String get clearOnLeave =>
      'The displayed password is cleared when you leave this page.';

  @override
  String get createPendingVersion => 'Create pending version';

  @override
  String get commitRotation => 'Commit rotation';

  @override
  String get cancelPending => 'Cancel pending';

  @override
  String get pendingCreated => 'Pending version created.';

  @override
  String get rotationCommitted => 'Rotation committed.';

  @override
  String get rotationCanceled => 'Pending version canceled.';

  @override
  String get oldVersionRemainsActive =>
      'The current active version remains valid until commit.';

  @override
  String get recoverLocal => 'Recover this device';

  @override
  String get recoverUsb => 'Rebuild USB package';

  @override
  String get resetMnemonic => 'Reset mnemonic';

  @override
  String get newMnemonicPhrase => 'New mnemonic phrase';

  @override
  String get resetMnemonicHelp =>
      'Use this device and the selected USB package to set a new mnemonic. Existing derived passwords remain unchanged.';

  @override
  String get mnemonicResetComplete => 'Mnemonic reset completed.';

  @override
  String get recoveryComplete => 'Recovery completed.';

  @override
  String get singleFactorNotEnough =>
      'A single factor is not enough to recover.';

  @override
  String get usbLostMode => 'USB lost';

  @override
  String get newDeviceMode => 'New device';

  @override
  String get runRecovery => 'Run recovery';

  @override
  String get detectedUsb => 'Detected USB volumes';

  @override
  String get rescanUsb => 'Rescan USB';

  @override
  String get usbPackage => 'USB package';

  @override
  String get cdrBackup => 'CDR backup';

  @override
  String get cdrBackupStatus => 'CDR backup status';

  @override
  String get localRecordCount => 'Local records';

  @override
  String get usbRecordCount => 'USB records';

  @override
  String get syncLocalToUsb => 'Sync local to USB';

  @override
  String get restoreLocalFromUsb => 'Restore local from USB';

  @override
  String get cdrBackupConsistent =>
      'Local records and USB backup are consistent.';

  @override
  String get cdrBackupNeedsAction =>
      'Local records and USB backup differ. Choose which copy to trust.';

  @override
  String get cdrSyncedToUsb => 'CDR backup synced to USB.';

  @override
  String get cdrRestoredFromUsb => 'Local records restored from USB backup.';

  @override
  String get confirmRestoreCdrTitle => 'Restore local records';

  @override
  String get confirmRestoreCdrBody =>
      'This replaces local CDR metadata with the USB backup. Derived passwords are not stored in either copy.';

  @override
  String get usbActions => 'USB actions';

  @override
  String get packageStatus => 'Package status';

  @override
  String get packageReadable => 'Package readable';

  @override
  String get packageMissing => 'No package yet';

  @override
  String get verifyUsbPackage => 'Verify USB package';

  @override
  String get rebuildUsbPackage => 'Rebuild USB package';

  @override
  String get usbVerified => 'USB package verified.';

  @override
  String get usbRebuilt => 'USB package rebuilt.';

  @override
  String get usbHelpHint =>
      'A standard USB drive is supported. Use the folder button if macOS asks for access.';

  @override
  String get manualUsbHint =>
      'If the drive is visible in Finder, you may enter its path manually, for example /Volumes/WD.';

  @override
  String get integrityCheck => 'Integrity Check';

  @override
  String get cdrMac => 'CDR MAC';

  @override
  String get usbAuthentication => 'USB authentication';

  @override
  String get logSafety => 'Log safety';

  @override
  String get clipboardClearing => 'Clipboard clearing';

  @override
  String get analytics => 'Analytics';

  @override
  String get disabled => 'Disabled';

  @override
  String get enabled => 'Enabled';

  @override
  String get language => 'Language';

  @override
  String get english => 'English';

  @override
  String get simplifiedChinese => 'Simplified Chinese';

  @override
  String get systemDefault => 'System default';

  @override
  String get theme => 'Theme';

  @override
  String get dark => 'Dark';

  @override
  String get light => 'Light';

  @override
  String get defaultPasswordLength => 'Default password length';

  @override
  String get defaultCharacterPolicy => 'Default character policy';

  @override
  String get advancedMode => 'Advanced mode';

  @override
  String get exportDiagnostics => 'Export diagnostics';

  @override
  String get diagnosticsReady => 'Diagnostics prepared without sensitive data.';

  @override
  String get diagnosticsTitle => 'Diagnostics';

  @override
  String get copyDiagnostics => 'Copy diagnostics';

  @override
  String get diagnosticsCopied => 'Diagnostics copied.';

  @override
  String get resetApplicationData => 'Reset application data';

  @override
  String get resetWarning =>
      'Deletes local KeyLessPass data on this device. USB packages are not erased.';

  @override
  String get resetConfirmTitle => 'Reset local data';

  @override
  String get resetConfirmationPrompt => 'Type RESET to confirm.';

  @override
  String get resetConfirmationMismatch => 'Confirmation did not match.';

  @override
  String get resetComplete =>
      'Local application data reset. Initialize again to continue.';

  @override
  String get setup => 'Setup';

  @override
  String get createFactors => 'Create factors';

  @override
  String get setupLocked => 'Setup is locked';

  @override
  String get setupLockedMessage =>
      'This device is already set up. Use Recovery to rebuild missing factor packages.';

  @override
  String get setupComplete => 'Setup completed.';

  @override
  String get supportEmail => 'Support: support@example.com';

  @override
  String get privacySummary =>
      'No cloud sync, no analytics, no service-password vault.';

  @override
  String get aboutBody =>
      'KeyLessPass is a local desktop client for enterprise password derivation. It is designed for controlled environments that still depend on legacy passwords.';

  @override
  String get operationFailed =>
      'The operation could not be completed. Check the required factors and try again.';

  @override
  String get coreUnavailable => 'The local security core could not be loaded.';

  @override
  String get requiredField => 'This field is required.';

  @override
  String recordsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count records',
      one: '1 record',
      zero: 'No records',
    );
    return '$_temp0';
  }

  @override
  String usbCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count USB volumes',
      one: '1 USB volume',
      zero: 'No USB volume',
    );
    return '$_temp0';
  }
}
