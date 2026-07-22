# 商业授权

English: [COMMERCIAL.md](COMMERCIAL.md)

KeyLessPass 是源码可见软件，可用于评估、安全审查、学习和非商业测试，但不是开源软件。

以下用途需要另行取得书面商业授权：

- 企业生产部署；
- 商业使用；
- 二次分发；
- OEM 或白标集成；
- 渠道或代理销售；
- 托管服务；
- 安全服务商打包；
- 使用 KeyLessPass 交付收费咨询或实施服务；
- 处理真实客户或企业生产凭据；
- 集成到付费产品、设备、平台或服务包。

## 可合作方式

KeyLessPass 可支持：

1. 企业授权；
2. 离线或内网部署授权；
3. OEM 或白标授权；
4. 渠道或代理合作；
5. 安全服务商集成；
6. 定制企业支持；
7. 联合 PoC。

## 设备批量授权

商业部署应使用“组织授权 + 单设备授权书”的签名授权模型。授权层只处理商业元数据，不接收、不保存助记短语、`Kmaster`、`deviceSecret`、`usbSecret`、CDR secret、服务密码或派生密码。

实际操作看：

- 设计说明：[docs/commercial/device-batch-authorization.md](docs/commercial/device-batch-authorization.md)
- 中文使用指南：[docs/commercial/device-batch-authorization-implementation.zh-CN.md](docs/commercial/device-batch-authorization-implementation.zh-CN.md)
- 商业发布加固：[docs/commercial/commercial-release-hardening.zh-CN.md](docs/commercial/commercial-release-hardening.zh-CN.md)
- 授权后台：[admin_backend/README.zh-CN.md](admin_backend/README.zh-CN.md)

## PoC 规则

- 不要使用真实生产密码或企业秘密。
- 尽量使用测试账号和非生产环境。
- 未取得授权前，不要作为生产凭据管理系统部署。
- 不要向第三方分发软件。
- 不要移除版权、许可或归属信息。

## 联系方式

商业授权、OEM、渠道或合作咨询：revanton@icloud.com
