// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Chinese (`zh`).
class AppLocalizationsZh extends AppLocalizations {
  AppLocalizationsZh([String locale = 'zh']) : super(locale);

  @override
  String get appName => 'KeyLessPass';

  @override
  String get appSubtitle => '本机密码派生客户端';

  @override
  String get dashboard => '首页';

  @override
  String get records => '记录';

  @override
  String get addRecord => '添加记录';

  @override
  String get derivePassword => '派生密码';

  @override
  String get rotation => '轮换';

  @override
  String get recovery => '恢复';

  @override
  String get usbDevice => 'U 盘设备';

  @override
  String get security => '安全';

  @override
  String get settings => '设置';

  @override
  String get about => '关于';

  @override
  String get initialized => '已初始化';

  @override
  String get notInitialized => '需要设置';

  @override
  String get platformProtected => '平台保护';

  @override
  String get reducedProtection => '降级保护';

  @override
  String get localOnly => '本机模式';

  @override
  String get refresh => '刷新';

  @override
  String get status => '状态';

  @override
  String get activeRecords => '活动记录';

  @override
  String get usbStatus => 'U 盘状态';

  @override
  String get integrity => '完整性';

  @override
  String get lastCheck => '最近检查';

  @override
  String get ready => '就绪';

  @override
  String get needsSetup => '需要设置';

  @override
  String get available => '可用';

  @override
  String get notFound => '未发现';

  @override
  String get ok => '正常';

  @override
  String get actionAddRecord => '添加记录';

  @override
  String get actionDerive => '派生';

  @override
  String get actionRotate => '轮换';

  @override
  String get actionRecovery => '恢复';

  @override
  String get quickActions => '快速操作';

  @override
  String get dashboardSubtitle => '管理记录并按需派生密码，不保存密码库。';

  @override
  String get safetyReminder => '密码仅在需要时派生，不会保存。';

  @override
  String get search => '搜索';

  @override
  String get filter => '筛选';

  @override
  String get all => '全部';

  @override
  String get active => '活动';

  @override
  String get pending => '待确认';

  @override
  String get retired => '已停用';

  @override
  String get conflict => '冲突';

  @override
  String get error => '错误';

  @override
  String get displayName => '显示名称';

  @override
  String get serviceHint => '服务提示 / URL';

  @override
  String get accountHint => '账号提示';

  @override
  String get notes => '备注';

  @override
  String get version => '版本';

  @override
  String get state => '状态';

  @override
  String get lastUpdated => '更新时间';

  @override
  String get lastUsed => '最近使用';

  @override
  String get passwordRule => '密码规则';

  @override
  String get length => '长度';

  @override
  String get requiredClasses => '必需字符类型';

  @override
  String get requireUppercase => '大写字母';

  @override
  String get requireLowercase => '小写字母';

  @override
  String get requireDigits => '数字';

  @override
  String get requireSymbols => '符号';

  @override
  String get forbiddenCharacters => '禁用字符';

  @override
  String get save => '保存';

  @override
  String get cancel => '取消';

  @override
  String get close => '关闭';

  @override
  String get editMetadata => '编辑信息';

  @override
  String get viewIntegrity => '查看完整性';

  @override
  String get noRecords => '还没有记录。';

  @override
  String get selectRecord => '选择记录';

  @override
  String get recordCreated => '记录已创建。';

  @override
  String get metadataSaved => '信息已保存。';

  @override
  String get metadataDoesNotChangePassword => '展示字段不会改变派生密码。';

  @override
  String get ruleChangeRequiresRotation => '修改密码规则前，需要创建新版本。';

  @override
  String get advancedDetails => '高级详情';

  @override
  String get recordSequence => '记录序号';

  @override
  String get recordId => '记录 ID';

  @override
  String get salt => '盐值';

  @override
  String get encodingRule => '编码规则';

  @override
  String get mnemonicPhrase => '助记短语';

  @override
  String get mnemonicLanguage => '助记短语语言';

  @override
  String get englishMnemonic => '英文';

  @override
  String get chineseMnemonic => '简体中文';

  @override
  String get generateMnemonic => '生成助记短语';

  @override
  String get generatedMnemonicReady => '助记短语已在本机生成。';

  @override
  String get mnemonicGeneratedLocally => '仅在本机生成。请离线保存，KeyLessPass 不会保存它。';

  @override
  String get showMnemonic => '显示助记短语';

  @override
  String get hideMnemonic => '隐藏助记短语';

  @override
  String get wordCount => '词数';

  @override
  String get usbPath => 'U 盘路径';

  @override
  String get chooseUsb => '选择 U 盘目录';

  @override
  String get showPassword => '显示密码';

  @override
  String get hidePassword => '隐藏密码';

  @override
  String get copy => '复制';

  @override
  String get deriveAndCopy => '派生并复制';

  @override
  String get deriveCurrentVersion => '派生当前版本';

  @override
  String get derivePreviousVersion => '派生上一版本';

  @override
  String get currentVersion => '当前版本';

  @override
  String get previousVersion => '上一版本';

  @override
  String get normalDerivationHelp => '正常派生使用助记短语 + 本机。U 盘包只在初始化和恢复时使用。';

  @override
  String get deriveMode => '验证方式';

  @override
  String get deriveModeThisDevice => '本机验证';

  @override
  String get deriveModeUsbRecovery => 'U 盘恢复验证';

  @override
  String get deriveModeThisDeviceHelp => '使用本机、所选 U 盘包和助记短语。';

  @override
  String get deriveModeUsbRecoveryHelp => '先通过 U 盘和助记短语重建本机材料，再派生密码。';

  @override
  String get passwordCopied => '密码已复制，将自动清除。';

  @override
  String get passwordHidden => '密码已隐藏';

  @override
  String get clipboardClearFailed => '无法确认剪贴板已清除。';

  @override
  String get clipboardTimeout => '剪贴板清除时间';

  @override
  String get seconds => '秒';

  @override
  String get clearOnLeave => '离开页面后会清除当前显示的密码。';

  @override
  String get createPendingVersion => '创建新版本';

  @override
  String get createNewVersion => '创建新版本';

  @override
  String get commitRotation => '提交轮换';

  @override
  String get cancelPending => '取消待确认版本';

  @override
  String get pendingCreated => '新版本已创建。';

  @override
  String get newVersionCreated => '新的当前版本已创建，上一版本仍可派生。';

  @override
  String get rotationCommitted => '轮换已提交。';

  @override
  String get rotationCanceled => '待确认版本已取消。';

  @override
  String get oldVersionRemainsActive => '保留当前版本和上一版本，两个版本都可以派生密码。';

  @override
  String get rotationCreateHelp =>
      '需要轮换目标系统密码时创建新的当前版本。KeyLessPass 会保留上一版本，便于回退核验。';

  @override
  String get rotationPendingHelp => '使用当前版本和上一版本按钮派生对应密码。';

  @override
  String get rotationEvidenceHelp =>
      '在远端修改密码后，分别记录新旧密码的认证结果。只有新密码成功且旧密码被明确拒绝时，才允许本地提交。';

  @override
  String get newPasswordWorks => '新密码可用';

  @override
  String get newPasswordFails => '新密码不可用';

  @override
  String get oldPasswordWorks => '旧密码仍可用';

  @override
  String get oldPasswordFails => '旧密码已失效';

  @override
  String get rotationEvidenceRecorded => '认证证据已记录。';

  @override
  String get rotationEvidenceRejected => '证据相互矛盾或尚不足以提交。';

  @override
  String get derivePendingPassword => '派生版本密码';

  @override
  String get rotationNoRecord => '请先选择要轮换的记录。';

  @override
  String get recoverLocal => '恢复本机';

  @override
  String get recoverUsb => '重建 U 盘包';

  @override
  String get resetMnemonic => '重置助记短语';

  @override
  String get newMnemonicPhrase => '新的助记短语';

  @override
  String get confirmNewMnemonicPhrase => '确认新的助记短语';

  @override
  String get newMnemonicMismatch => '两次输入的新助记短语不一致。';

  @override
  String get resetMnemonicHelp => '使用本机 + U 盘包恢复并设置新的助记短语；不需要旧助记短语，已有派生密码保持不变。';

  @override
  String get resetMnemonicFactorHelp =>
      '此路径使用本机 + U 盘包，通过 W_CU 恢复主密钥；不需要旧助记短语。';

  @override
  String get usbLostHelp => '使用本机 + 助记短语重建 U 盘包。单独本机因子不能恢复主密钥。';

  @override
  String get recoveryPathMnemonicComputer => '助记短语 + 本机';

  @override
  String get recoveryPathMnemonicUsb => '助记短语 + U 盘包';

  @override
  String get recoveryPathComputerUsb => '本机 + U 盘包';

  @override
  String get rebuildUsbExplanation => '使用助记短语 + 本机，通过 W_MC 恢复主密钥并重建配对 U 盘包。';

  @override
  String get recoverComputerExplanation =>
      '使用助记短语 + U 盘包，通过 W_MU 恢复主密钥并重建本机因子。';

  @override
  String get resetMnemonicExplanation =>
      '使用本机 + U 盘包，通过 W_CU 恢复主密钥并设置新的助记短语。不需要旧助记短语，已有派生密码保持不变。';

  @override
  String get mnemonicResetComplete => '助记短语已重置。';

  @override
  String get recoveryComplete => '恢复已完成。';

  @override
  String get singleFactorNotEnough => '单个因子不足以恢复。';

  @override
  String get usbLostMode => 'U 盘丢失';

  @override
  String get newDeviceMode => '更换本机';

  @override
  String get runRecovery => '执行恢复';

  @override
  String get detectedUsb => '检测到的 U 盘卷';

  @override
  String get rescanUsb => '重新扫描 U 盘';

  @override
  String get usbPackage => 'U 盘包';

  @override
  String get cdrBackup => '记录备份';

  @override
  String get cdrBackupStatus => '记录备份状态';

  @override
  String get localRecordCount => '本机记录';

  @override
  String get usbRecordCount => 'U 盘记录';

  @override
  String get syncLocalToUsb => '同步本机到 U 盘';

  @override
  String get restoreLocalFromUsb => '用 U 盘恢复本机';

  @override
  String get cdrBackupConsistent => '本机记录与 U 盘备份一致。';

  @override
  String get cdrBackupNeedsAction => '本机记录与 U 盘备份不一致。请选择信任哪一份。';

  @override
  String get cdrSyncedToUsb => '记录备份已同步到 U 盘。';

  @override
  String get cdrRestoredFromUsb => '已从 U 盘备份恢复本机记录。';

  @override
  String get confirmRestoreCdrTitle => '恢复本机记录';

  @override
  String get confirmRestoreCdrBody => '这会用 U 盘备份替换本机 CDR 元数据。两份数据都不包含派生密码。';

  @override
  String get usbActions => 'U 盘操作';

  @override
  String get packageStatus => '包状态';

  @override
  String get packageReadable => '包可读取';

  @override
  String get packageMissing => '尚无包';

  @override
  String get verifyUsbPackage => '校验 U 盘包';

  @override
  String get rebuildUsbPackage => '重建 U 盘包';

  @override
  String get usbVerified => 'U 盘包校验通过。';

  @override
  String get usbRebuilt => 'U 盘包已重建。';

  @override
  String get usbHelpHint => '支持普通 U 盘。校验只检查包结构和完整性，不需要助记短语；重建需要助记短语 + 本机。';

  @override
  String get usbFactorContainerHelp =>
      'U 盘包是可复制的因子容器，保存 U 盘因子材料和成对加密包装，不保存明文服务密码或明文 Kmaster。';

  @override
  String get manualUsbHint => '如果 Finder 中能看到 U 盘，也可以手动输入路径，例如 /Volumes/WD。';

  @override
  String get integrityCheck => '完整性检查';

  @override
  String get cdrMac => '记录 MAC';

  @override
  String get usbAuthentication => 'U 盘认证';

  @override
  String get logSafety => '日志安全';

  @override
  String get clipboardClearing => '剪贴板清除';

  @override
  String get analytics => '分析数据';

  @override
  String get disabled => '已关闭';

  @override
  String get enabled => '已启用';

  @override
  String get language => '语言';

  @override
  String get english => 'English';

  @override
  String get simplifiedChinese => '简体中文';

  @override
  String get systemDefault => '跟随系统';

  @override
  String get theme => '主题';

  @override
  String get dark => '深色';

  @override
  String get light => '浅色';

  @override
  String get defaultPasswordLength => '默认密码长度';

  @override
  String get defaultCharacterPolicy => '默认字符策略';

  @override
  String get derivationAlgorithm => '派生算法';

  @override
  String get exactDomainAlgorithm => '精确策略空间 v3（HKDF-SHA256 + FF1）';

  @override
  String get exactDomainAlgorithmHelp =>
      '新凭据采用精确策略计数、Rank/Unrank 和由 generation 索引的域置换。旧 KDF 元数据仅用于复现历史记录。';

  @override
  String get algorithmAppliesOnNextSetup => '该选择会用于下一次初始化新的本机配置。';

  @override
  String get algorithmLockedUntilReset => '当前数据使用此算法。如需更改，请重置本机数据后重新初始化。';

  @override
  String get legacyHkdfDetected =>
      '当前数据没有保存算法字段，KeyLessPass 按旧版 HKDF-SHA256 处理。';

  @override
  String get advancedMode => '高级模式';

  @override
  String get exportDiagnostics => '导出诊断信息';

  @override
  String get diagnosticsReady => '诊断信息已生成，不包含敏感数据。';

  @override
  String get diagnosticsTitle => '诊断信息';

  @override
  String get copyDiagnostics => '复制诊断信息';

  @override
  String get diagnosticsCopied => '诊断信息已复制。';

  @override
  String get resetApplicationData => '重置应用数据';

  @override
  String get resetWarning => '删除此设备上的 KeyLessPass 本地数据。不会擦除 U 盘包。';

  @override
  String get resetConfirmTitle => '重置本机数据';

  @override
  String get resetConfirmationPrompt => '请输入 RESET 以确认。';

  @override
  String get resetConfirmationMismatch => '确认文本不匹配。';

  @override
  String get resetComplete => '本机应用数据已重置。请重新初始化后继续。';

  @override
  String get setup => '初始化';

  @override
  String get createFactors => '创建因子';

  @override
  String get setupStartTitle => '在这台电脑上开始';

  @override
  String get setupStartSubtitle => '可以创建新的资料，也可以使用 U 盘包和助记短语恢复已有资料。';

  @override
  String get createNewProfile => '创建新资料';

  @override
  String get recoverExistingProfile => '恢复已有资料';

  @override
  String get recoverLocalHelp =>
      '在更换本机时使用助记短语 + U 盘重建这台电脑的本机因子，并在 U 盘记录备份可用时恢复本机记录。';

  @override
  String get setupLocked => '初始化已锁定';

  @override
  String get setupLockedMessage => '此设备已初始化。请使用恢复功能重建缺失的因子包。';

  @override
  String get setupComplete => '初始化已完成。';

  @override
  String get supportEmail => '支持邮箱：revanton@icloud.com';

  @override
  String get privacySummary => '无云同步、无分析采集、无服务密码库。';

  @override
  String get aboutBody => 'KeyLessPass 是面向企业场景的本机密码派生客户端，适用于仍依赖传统密码的受控环境。';

  @override
  String get operationFailed => '操作未完成。请检查所需因子后重试。';

  @override
  String get coreUnavailable => '无法加载本机安全核心。';

  @override
  String get requiredField => '此项必填。';

  @override
  String recordsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count 条记录',
      one: '1 条记录',
      zero: '无记录',
    );
    return '$_temp0';
  }

  @override
  String usbCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count 个 U 盘卷',
      one: '1 个 U 盘卷',
      zero: '未发现 U 盘',
    );
    return '$_temp0';
  }
}
