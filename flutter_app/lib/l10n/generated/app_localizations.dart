import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_zh.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'generated/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
      : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
    delegate,
    GlobalMaterialLocalizations.delegate,
    GlobalCupertinoLocalizations.delegate,
    GlobalWidgetsLocalizations.delegate,
  ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('zh')
  ];

  /// No description provided for @appName.
  ///
  /// In en, this message translates to:
  /// **'KeyLessPass'**
  String get appName;

  /// No description provided for @appSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Local password derivation client'**
  String get appSubtitle;

  /// No description provided for @dashboard.
  ///
  /// In en, this message translates to:
  /// **'Dashboard'**
  String get dashboard;

  /// No description provided for @records.
  ///
  /// In en, this message translates to:
  /// **'Records'**
  String get records;

  /// No description provided for @addRecord.
  ///
  /// In en, this message translates to:
  /// **'Add Record'**
  String get addRecord;

  /// No description provided for @derivePassword.
  ///
  /// In en, this message translates to:
  /// **'Derive Password'**
  String get derivePassword;

  /// No description provided for @rotation.
  ///
  /// In en, this message translates to:
  /// **'Rotation'**
  String get rotation;

  /// No description provided for @recovery.
  ///
  /// In en, this message translates to:
  /// **'Recovery'**
  String get recovery;

  /// No description provided for @usbDevice.
  ///
  /// In en, this message translates to:
  /// **'USB Device'**
  String get usbDevice;

  /// No description provided for @security.
  ///
  /// In en, this message translates to:
  /// **'Security'**
  String get security;

  /// No description provided for @settings.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get settings;

  /// No description provided for @about.
  ///
  /// In en, this message translates to:
  /// **'About'**
  String get about;

  /// No description provided for @initialized.
  ///
  /// In en, this message translates to:
  /// **'Initialized'**
  String get initialized;

  /// No description provided for @notInitialized.
  ///
  /// In en, this message translates to:
  /// **'Setup required'**
  String get notInitialized;

  /// No description provided for @platformProtected.
  ///
  /// In en, this message translates to:
  /// **'Platform protection'**
  String get platformProtected;

  /// No description provided for @reducedProtection.
  ///
  /// In en, this message translates to:
  /// **'Reduced protection'**
  String get reducedProtection;

  /// No description provided for @localOnly.
  ///
  /// In en, this message translates to:
  /// **'Local only'**
  String get localOnly;

  /// No description provided for @refresh.
  ///
  /// In en, this message translates to:
  /// **'Refresh'**
  String get refresh;

  /// No description provided for @status.
  ///
  /// In en, this message translates to:
  /// **'Status'**
  String get status;

  /// No description provided for @activeRecords.
  ///
  /// In en, this message translates to:
  /// **'Active records'**
  String get activeRecords;

  /// No description provided for @usbStatus.
  ///
  /// In en, this message translates to:
  /// **'USB status'**
  String get usbStatus;

  /// No description provided for @integrity.
  ///
  /// In en, this message translates to:
  /// **'Integrity'**
  String get integrity;

  /// No description provided for @lastCheck.
  ///
  /// In en, this message translates to:
  /// **'Last check'**
  String get lastCheck;

  /// No description provided for @ready.
  ///
  /// In en, this message translates to:
  /// **'Ready'**
  String get ready;

  /// No description provided for @needsSetup.
  ///
  /// In en, this message translates to:
  /// **'Needs setup'**
  String get needsSetup;

  /// No description provided for @available.
  ///
  /// In en, this message translates to:
  /// **'Available'**
  String get available;

  /// No description provided for @notFound.
  ///
  /// In en, this message translates to:
  /// **'Not found'**
  String get notFound;

  /// No description provided for @ok.
  ///
  /// In en, this message translates to:
  /// **'OK'**
  String get ok;

  /// No description provided for @actionAddRecord.
  ///
  /// In en, this message translates to:
  /// **'Add Record'**
  String get actionAddRecord;

  /// No description provided for @actionDerive.
  ///
  /// In en, this message translates to:
  /// **'Derive'**
  String get actionDerive;

  /// No description provided for @actionRotate.
  ///
  /// In en, this message translates to:
  /// **'Rotate'**
  String get actionRotate;

  /// No description provided for @actionRecovery.
  ///
  /// In en, this message translates to:
  /// **'Recovery'**
  String get actionRecovery;

  /// No description provided for @quickActions.
  ///
  /// In en, this message translates to:
  /// **'Quick actions'**
  String get quickActions;

  /// No description provided for @dashboardSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Manage records and derive passwords without storing a password vault.'**
  String get dashboardSubtitle;

  /// No description provided for @safetyReminder.
  ///
  /// In en, this message translates to:
  /// **'Passwords are derived on demand and are not saved.'**
  String get safetyReminder;

  /// No description provided for @search.
  ///
  /// In en, this message translates to:
  /// **'Search'**
  String get search;

  /// No description provided for @filter.
  ///
  /// In en, this message translates to:
  /// **'Filter'**
  String get filter;

  /// No description provided for @all.
  ///
  /// In en, this message translates to:
  /// **'All'**
  String get all;

  /// No description provided for @active.
  ///
  /// In en, this message translates to:
  /// **'Active'**
  String get active;

  /// No description provided for @pending.
  ///
  /// In en, this message translates to:
  /// **'Pending'**
  String get pending;

  /// No description provided for @retired.
  ///
  /// In en, this message translates to:
  /// **'Retired'**
  String get retired;

  /// No description provided for @conflict.
  ///
  /// In en, this message translates to:
  /// **'Conflict'**
  String get conflict;

  /// No description provided for @error.
  ///
  /// In en, this message translates to:
  /// **'Error'**
  String get error;

  /// No description provided for @displayName.
  ///
  /// In en, this message translates to:
  /// **'Display Name'**
  String get displayName;

  /// No description provided for @serviceHint.
  ///
  /// In en, this message translates to:
  /// **'Service Hint / URL'**
  String get serviceHint;

  /// No description provided for @accountHint.
  ///
  /// In en, this message translates to:
  /// **'Account Hint'**
  String get accountHint;

  /// No description provided for @notes.
  ///
  /// In en, this message translates to:
  /// **'Notes'**
  String get notes;

  /// No description provided for @version.
  ///
  /// In en, this message translates to:
  /// **'Version'**
  String get version;

  /// No description provided for @state.
  ///
  /// In en, this message translates to:
  /// **'State'**
  String get state;

  /// No description provided for @lastUpdated.
  ///
  /// In en, this message translates to:
  /// **'Updated'**
  String get lastUpdated;

  /// No description provided for @lastUsed.
  ///
  /// In en, this message translates to:
  /// **'Last used'**
  String get lastUsed;

  /// No description provided for @passwordRule.
  ///
  /// In en, this message translates to:
  /// **'Password Rule'**
  String get passwordRule;

  /// No description provided for @length.
  ///
  /// In en, this message translates to:
  /// **'Length'**
  String get length;

  /// No description provided for @requiredClasses.
  ///
  /// In en, this message translates to:
  /// **'Required character classes'**
  String get requiredClasses;

  /// No description provided for @requireUppercase.
  ///
  /// In en, this message translates to:
  /// **'Uppercase'**
  String get requireUppercase;

  /// No description provided for @requireLowercase.
  ///
  /// In en, this message translates to:
  /// **'Lowercase'**
  String get requireLowercase;

  /// No description provided for @requireDigits.
  ///
  /// In en, this message translates to:
  /// **'Digits'**
  String get requireDigits;

  /// No description provided for @requireSymbols.
  ///
  /// In en, this message translates to:
  /// **'Symbols'**
  String get requireSymbols;

  /// No description provided for @forbiddenCharacters.
  ///
  /// In en, this message translates to:
  /// **'Forbidden characters'**
  String get forbiddenCharacters;

  /// No description provided for @save.
  ///
  /// In en, this message translates to:
  /// **'Save'**
  String get save;

  /// No description provided for @cancel.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get cancel;

  /// No description provided for @close.
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get close;

  /// No description provided for @editMetadata.
  ///
  /// In en, this message translates to:
  /// **'Edit Metadata'**
  String get editMetadata;

  /// No description provided for @viewIntegrity.
  ///
  /// In en, this message translates to:
  /// **'View Integrity'**
  String get viewIntegrity;

  /// No description provided for @noRecords.
  ///
  /// In en, this message translates to:
  /// **'No records yet.'**
  String get noRecords;

  /// No description provided for @selectRecord.
  ///
  /// In en, this message translates to:
  /// **'Select a record'**
  String get selectRecord;

  /// No description provided for @recordCreated.
  ///
  /// In en, this message translates to:
  /// **'Record created.'**
  String get recordCreated;

  /// No description provided for @metadataSaved.
  ///
  /// In en, this message translates to:
  /// **'Metadata saved.'**
  String get metadataSaved;

  /// No description provided for @metadataDoesNotChangePassword.
  ///
  /// In en, this message translates to:
  /// **'Display fields do not change the derived password.'**
  String get metadataDoesNotChangePassword;

  /// No description provided for @ruleChangeRequiresRotation.
  ///
  /// In en, this message translates to:
  /// **'Changing password rules requires a new version.'**
  String get ruleChangeRequiresRotation;

  /// No description provided for @recordDetails.
  ///
  /// In en, this message translates to:
  /// **'Record details'**
  String get recordDetails;

  /// No description provided for @advancedDetails.
  ///
  /// In en, this message translates to:
  /// **'Advanced details'**
  String get advancedDetails;

  /// No description provided for @recordSequence.
  ///
  /// In en, this message translates to:
  /// **'Record sequence'**
  String get recordSequence;

  /// No description provided for @recordId.
  ///
  /// In en, this message translates to:
  /// **'Record ID'**
  String get recordId;

  /// No description provided for @salt.
  ///
  /// In en, this message translates to:
  /// **'Salt'**
  String get salt;

  /// No description provided for @encodingRule.
  ///
  /// In en, this message translates to:
  /// **'Encoding rule'**
  String get encodingRule;

  /// No description provided for @mnemonicPhrase.
  ///
  /// In en, this message translates to:
  /// **'Mnemonic phrase'**
  String get mnemonicPhrase;

  /// No description provided for @mnemonicLanguage.
  ///
  /// In en, this message translates to:
  /// **'Mnemonic language'**
  String get mnemonicLanguage;

  /// No description provided for @englishMnemonic.
  ///
  /// In en, this message translates to:
  /// **'English'**
  String get englishMnemonic;

  /// No description provided for @chineseMnemonic.
  ///
  /// In en, this message translates to:
  /// **'Simplified Chinese'**
  String get chineseMnemonic;

  /// No description provided for @generateMnemonic.
  ///
  /// In en, this message translates to:
  /// **'Generate mnemonic'**
  String get generateMnemonic;

  /// No description provided for @generatedMnemonicReady.
  ///
  /// In en, this message translates to:
  /// **'Mnemonic generated locally.'**
  String get generatedMnemonicReady;

  /// No description provided for @mnemonicGeneratedLocally.
  ///
  /// In en, this message translates to:
  /// **'Generated on this device only. Save it offline; KeyLessPass will not store it.'**
  String get mnemonicGeneratedLocally;

  /// No description provided for @showMnemonic.
  ///
  /// In en, this message translates to:
  /// **'Show mnemonic'**
  String get showMnemonic;

  /// No description provided for @hideMnemonic.
  ///
  /// In en, this message translates to:
  /// **'Hide mnemonic'**
  String get hideMnemonic;

  /// No description provided for @wordCount.
  ///
  /// In en, this message translates to:
  /// **'Word count'**
  String get wordCount;

  /// No description provided for @usbPath.
  ///
  /// In en, this message translates to:
  /// **'USB path'**
  String get usbPath;

  /// No description provided for @chooseUsb.
  ///
  /// In en, this message translates to:
  /// **'Choose USB folder'**
  String get chooseUsb;

  /// No description provided for @showPassword.
  ///
  /// In en, this message translates to:
  /// **'Show password'**
  String get showPassword;

  /// No description provided for @hidePassword.
  ///
  /// In en, this message translates to:
  /// **'Hide password'**
  String get hidePassword;

  /// No description provided for @copy.
  ///
  /// In en, this message translates to:
  /// **'Copy'**
  String get copy;

  /// No description provided for @deriveAndCopy.
  ///
  /// In en, this message translates to:
  /// **'Derive and copy'**
  String get deriveAndCopy;

  /// No description provided for @deriveMode.
  ///
  /// In en, this message translates to:
  /// **'Verification mode'**
  String get deriveMode;

  /// No description provided for @deriveModeThisDevice.
  ///
  /// In en, this message translates to:
  /// **'This device'**
  String get deriveModeThisDevice;

  /// No description provided for @deriveModeUsbRecovery.
  ///
  /// In en, this message translates to:
  /// **'USB recovery'**
  String get deriveModeUsbRecovery;

  /// No description provided for @deriveModeThisDeviceHelp.
  ///
  /// In en, this message translates to:
  /// **'Use this device, the selected USB package, and the mnemonic.'**
  String get deriveModeThisDeviceHelp;

  /// No description provided for @deriveModeUsbRecoveryHelp.
  ///
  /// In en, this message translates to:
  /// **'Rebuild local material from USB and mnemonic before deriving.'**
  String get deriveModeUsbRecoveryHelp;

  /// No description provided for @passwordCopied.
  ///
  /// In en, this message translates to:
  /// **'Password copied. It will be cleared automatically.'**
  String get passwordCopied;

  /// No description provided for @passwordHidden.
  ///
  /// In en, this message translates to:
  /// **'Password hidden'**
  String get passwordHidden;

  /// No description provided for @clipboardClearFailed.
  ///
  /// In en, this message translates to:
  /// **'Clipboard clearing could not be confirmed.'**
  String get clipboardClearFailed;

  /// No description provided for @clipboardTimeout.
  ///
  /// In en, this message translates to:
  /// **'Clipboard timeout'**
  String get clipboardTimeout;

  /// No description provided for @seconds.
  ///
  /// In en, this message translates to:
  /// **'seconds'**
  String get seconds;

  /// No description provided for @clearOnLeave.
  ///
  /// In en, this message translates to:
  /// **'The displayed password is cleared when you leave this page.'**
  String get clearOnLeave;

  /// No description provided for @createPendingVersion.
  ///
  /// In en, this message translates to:
  /// **'Create pending version'**
  String get createPendingVersion;

  /// No description provided for @commitRotation.
  ///
  /// In en, this message translates to:
  /// **'Commit rotation'**
  String get commitRotation;

  /// No description provided for @cancelPending.
  ///
  /// In en, this message translates to:
  /// **'Cancel pending'**
  String get cancelPending;

  /// No description provided for @pendingCreated.
  ///
  /// In en, this message translates to:
  /// **'Pending version created.'**
  String get pendingCreated;

  /// No description provided for @rotationCommitted.
  ///
  /// In en, this message translates to:
  /// **'Rotation committed.'**
  String get rotationCommitted;

  /// No description provided for @rotationCanceled.
  ///
  /// In en, this message translates to:
  /// **'Pending version canceled.'**
  String get rotationCanceled;

  /// No description provided for @oldVersionRemainsActive.
  ///
  /// In en, this message translates to:
  /// **'The current active version remains valid until commit.'**
  String get oldVersionRemainsActive;

  /// No description provided for @recoverLocal.
  ///
  /// In en, this message translates to:
  /// **'Recover this device'**
  String get recoverLocal;

  /// No description provided for @recoverUsb.
  ///
  /// In en, this message translates to:
  /// **'Rebuild USB package'**
  String get recoverUsb;

  /// No description provided for @resetMnemonic.
  ///
  /// In en, this message translates to:
  /// **'Reset mnemonic'**
  String get resetMnemonic;

  /// No description provided for @newMnemonicPhrase.
  ///
  /// In en, this message translates to:
  /// **'New mnemonic phrase'**
  String get newMnemonicPhrase;

  /// No description provided for @resetMnemonicHelp.
  ///
  /// In en, this message translates to:
  /// **'Use this device and the selected USB package to set a new mnemonic. Existing derived passwords remain unchanged.'**
  String get resetMnemonicHelp;

  /// No description provided for @mnemonicResetComplete.
  ///
  /// In en, this message translates to:
  /// **'Mnemonic reset completed.'**
  String get mnemonicResetComplete;

  /// No description provided for @recoveryComplete.
  ///
  /// In en, this message translates to:
  /// **'Recovery completed.'**
  String get recoveryComplete;

  /// No description provided for @singleFactorNotEnough.
  ///
  /// In en, this message translates to:
  /// **'A single factor is not enough to recover.'**
  String get singleFactorNotEnough;

  /// No description provided for @usbLostMode.
  ///
  /// In en, this message translates to:
  /// **'USB lost'**
  String get usbLostMode;

  /// No description provided for @newDeviceMode.
  ///
  /// In en, this message translates to:
  /// **'New device'**
  String get newDeviceMode;

  /// No description provided for @runRecovery.
  ///
  /// In en, this message translates to:
  /// **'Run recovery'**
  String get runRecovery;

  /// No description provided for @detectedUsb.
  ///
  /// In en, this message translates to:
  /// **'Detected USB volumes'**
  String get detectedUsb;

  /// No description provided for @rescanUsb.
  ///
  /// In en, this message translates to:
  /// **'Rescan USB'**
  String get rescanUsb;

  /// No description provided for @usbPackage.
  ///
  /// In en, this message translates to:
  /// **'USB package'**
  String get usbPackage;

  /// No description provided for @cdrBackup.
  ///
  /// In en, this message translates to:
  /// **'CDR backup'**
  String get cdrBackup;

  /// No description provided for @cdrBackupStatus.
  ///
  /// In en, this message translates to:
  /// **'CDR backup status'**
  String get cdrBackupStatus;

  /// No description provided for @localRecordCount.
  ///
  /// In en, this message translates to:
  /// **'Local records'**
  String get localRecordCount;

  /// No description provided for @usbRecordCount.
  ///
  /// In en, this message translates to:
  /// **'USB records'**
  String get usbRecordCount;

  /// No description provided for @syncLocalToUsb.
  ///
  /// In en, this message translates to:
  /// **'Sync local to USB'**
  String get syncLocalToUsb;

  /// No description provided for @restoreLocalFromUsb.
  ///
  /// In en, this message translates to:
  /// **'Restore local from USB'**
  String get restoreLocalFromUsb;

  /// No description provided for @cdrBackupConsistent.
  ///
  /// In en, this message translates to:
  /// **'Local records and USB backup are consistent.'**
  String get cdrBackupConsistent;

  /// No description provided for @cdrBackupNeedsAction.
  ///
  /// In en, this message translates to:
  /// **'Local records and USB backup differ. Choose which copy to trust.'**
  String get cdrBackupNeedsAction;

  /// No description provided for @cdrSyncedToUsb.
  ///
  /// In en, this message translates to:
  /// **'CDR backup synced to USB.'**
  String get cdrSyncedToUsb;

  /// No description provided for @cdrRestoredFromUsb.
  ///
  /// In en, this message translates to:
  /// **'Local records restored from USB backup.'**
  String get cdrRestoredFromUsb;

  /// No description provided for @confirmRestoreCdrTitle.
  ///
  /// In en, this message translates to:
  /// **'Restore local records'**
  String get confirmRestoreCdrTitle;

  /// No description provided for @confirmRestoreCdrBody.
  ///
  /// In en, this message translates to:
  /// **'This replaces local CDR metadata with the USB backup. Derived passwords are not stored in either copy.'**
  String get confirmRestoreCdrBody;

  /// No description provided for @usbActions.
  ///
  /// In en, this message translates to:
  /// **'USB actions'**
  String get usbActions;

  /// No description provided for @packageStatus.
  ///
  /// In en, this message translates to:
  /// **'Package status'**
  String get packageStatus;

  /// No description provided for @packageReadable.
  ///
  /// In en, this message translates to:
  /// **'Package readable'**
  String get packageReadable;

  /// No description provided for @packageMissing.
  ///
  /// In en, this message translates to:
  /// **'No package yet'**
  String get packageMissing;

  /// No description provided for @verifyUsbPackage.
  ///
  /// In en, this message translates to:
  /// **'Verify USB package'**
  String get verifyUsbPackage;

  /// No description provided for @rebuildUsbPackage.
  ///
  /// In en, this message translates to:
  /// **'Rebuild USB package'**
  String get rebuildUsbPackage;

  /// No description provided for @usbVerified.
  ///
  /// In en, this message translates to:
  /// **'USB package verified.'**
  String get usbVerified;

  /// No description provided for @usbRebuilt.
  ///
  /// In en, this message translates to:
  /// **'USB package rebuilt.'**
  String get usbRebuilt;

  /// No description provided for @usbHelpHint.
  ///
  /// In en, this message translates to:
  /// **'A standard USB drive is supported. Use the folder button if macOS asks for access.'**
  String get usbHelpHint;

  /// No description provided for @manualUsbHint.
  ///
  /// In en, this message translates to:
  /// **'If the drive is visible in Finder, you may enter its path manually, for example /Volumes/WD.'**
  String get manualUsbHint;

  /// No description provided for @integrityCheck.
  ///
  /// In en, this message translates to:
  /// **'Integrity Check'**
  String get integrityCheck;

  /// No description provided for @cdrMac.
  ///
  /// In en, this message translates to:
  /// **'CDR MAC'**
  String get cdrMac;

  /// No description provided for @usbAuthentication.
  ///
  /// In en, this message translates to:
  /// **'USB authentication'**
  String get usbAuthentication;

  /// No description provided for @logSafety.
  ///
  /// In en, this message translates to:
  /// **'Log safety'**
  String get logSafety;

  /// No description provided for @clipboardClearing.
  ///
  /// In en, this message translates to:
  /// **'Clipboard clearing'**
  String get clipboardClearing;

  /// No description provided for @analytics.
  ///
  /// In en, this message translates to:
  /// **'Analytics'**
  String get analytics;

  /// No description provided for @disabled.
  ///
  /// In en, this message translates to:
  /// **'Disabled'**
  String get disabled;

  /// No description provided for @enabled.
  ///
  /// In en, this message translates to:
  /// **'Enabled'**
  String get enabled;

  /// No description provided for @language.
  ///
  /// In en, this message translates to:
  /// **'Language'**
  String get language;

  /// No description provided for @english.
  ///
  /// In en, this message translates to:
  /// **'English'**
  String get english;

  /// No description provided for @simplifiedChinese.
  ///
  /// In en, this message translates to:
  /// **'Simplified Chinese'**
  String get simplifiedChinese;

  /// No description provided for @systemDefault.
  ///
  /// In en, this message translates to:
  /// **'System default'**
  String get systemDefault;

  /// No description provided for @theme.
  ///
  /// In en, this message translates to:
  /// **'Theme'**
  String get theme;

  /// No description provided for @dark.
  ///
  /// In en, this message translates to:
  /// **'Dark'**
  String get dark;

  /// No description provided for @light.
  ///
  /// In en, this message translates to:
  /// **'Light'**
  String get light;

  /// No description provided for @defaultPasswordLength.
  ///
  /// In en, this message translates to:
  /// **'Default password length'**
  String get defaultPasswordLength;

  /// No description provided for @defaultCharacterPolicy.
  ///
  /// In en, this message translates to:
  /// **'Default character policy'**
  String get defaultCharacterPolicy;

  /// No description provided for @advancedMode.
  ///
  /// In en, this message translates to:
  /// **'Advanced mode'**
  String get advancedMode;

  /// No description provided for @exportDiagnostics.
  ///
  /// In en, this message translates to:
  /// **'Export diagnostics'**
  String get exportDiagnostics;

  /// No description provided for @diagnosticsReady.
  ///
  /// In en, this message translates to:
  /// **'Diagnostics prepared without sensitive data.'**
  String get diagnosticsReady;

  /// No description provided for @diagnosticsTitle.
  ///
  /// In en, this message translates to:
  /// **'Diagnostics'**
  String get diagnosticsTitle;

  /// No description provided for @copyDiagnostics.
  ///
  /// In en, this message translates to:
  /// **'Copy diagnostics'**
  String get copyDiagnostics;

  /// No description provided for @diagnosticsCopied.
  ///
  /// In en, this message translates to:
  /// **'Diagnostics copied.'**
  String get diagnosticsCopied;

  /// No description provided for @resetApplicationData.
  ///
  /// In en, this message translates to:
  /// **'Reset application data'**
  String get resetApplicationData;

  /// No description provided for @resetWarning.
  ///
  /// In en, this message translates to:
  /// **'Deletes local KeyLessPass data on this device. USB packages are not erased.'**
  String get resetWarning;

  /// No description provided for @resetConfirmTitle.
  ///
  /// In en, this message translates to:
  /// **'Reset local data'**
  String get resetConfirmTitle;

  /// No description provided for @resetConfirmationPrompt.
  ///
  /// In en, this message translates to:
  /// **'Type RESET to confirm.'**
  String get resetConfirmationPrompt;

  /// No description provided for @resetConfirmationMismatch.
  ///
  /// In en, this message translates to:
  /// **'Confirmation did not match.'**
  String get resetConfirmationMismatch;

  /// No description provided for @resetComplete.
  ///
  /// In en, this message translates to:
  /// **'Local application data reset. Initialize again to continue.'**
  String get resetComplete;

  /// No description provided for @setup.
  ///
  /// In en, this message translates to:
  /// **'Setup'**
  String get setup;

  /// No description provided for @createFactors.
  ///
  /// In en, this message translates to:
  /// **'Create factors'**
  String get createFactors;

  /// No description provided for @setupLocked.
  ///
  /// In en, this message translates to:
  /// **'Setup is locked'**
  String get setupLocked;

  /// No description provided for @setupLockedMessage.
  ///
  /// In en, this message translates to:
  /// **'This device is already set up. Use Recovery to rebuild missing factor packages.'**
  String get setupLockedMessage;

  /// No description provided for @setupComplete.
  ///
  /// In en, this message translates to:
  /// **'Setup completed.'**
  String get setupComplete;

  /// No description provided for @supportEmail.
  ///
  /// In en, this message translates to:
  /// **'Support: support@example.com'**
  String get supportEmail;

  /// No description provided for @privacySummary.
  ///
  /// In en, this message translates to:
  /// **'No cloud sync, no analytics, no service-password vault.'**
  String get privacySummary;

  /// No description provided for @aboutBody.
  ///
  /// In en, this message translates to:
  /// **'KeyLessPass is a local desktop client for enterprise password derivation. It is designed for controlled environments that still depend on legacy passwords.'**
  String get aboutBody;

  /// No description provided for @operationFailed.
  ///
  /// In en, this message translates to:
  /// **'The operation could not be completed. Check the required factors and try again.'**
  String get operationFailed;

  /// No description provided for @coreUnavailable.
  ///
  /// In en, this message translates to:
  /// **'The local security core could not be loaded.'**
  String get coreUnavailable;

  /// No description provided for @requiredField.
  ///
  /// In en, this message translates to:
  /// **'This field is required.'**
  String get requiredField;

  /// No description provided for @recordsCount.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =0{No records} =1{1 record} other{{count} records}}'**
  String recordsCount(int count);

  /// No description provided for @usbCount.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =0{No USB volume} =1{1 USB volume} other{{count} USB volumes}}'**
  String usbCount(int count);
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en', 'zh'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
    case 'zh':
      return AppLocalizationsZh();
  }

  throw FlutterError(
      'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
      'an issue with the localizations generation tool. Please file an issue '
      'on GitHub with a reproducible sample app and the gen-l10n configuration '
      'that was used.');
}
