# Security policy

NextEngine является local pre-release bootstrap и пока не обещает public
support window или формальный security-response SLA.

Не добавляйте в Git secrets, credentials, user installations, imported game
data, datasets, checkpoints, training runs, generated models или caches.

## Reporting during bootstrap

Сообщите о чувствительной проблеме maintainer через доступный приватный канал.
Не коммитьте exploit payloads, credentials, personal data или protected bytes.
Если приватного канала пока нет, сохраните локальное redacted описание и
запросите контакт без публикации деталей.

## Scope

Runtime safety rules находятся в
`docs/architecture/11-security-licensing-and-governance.md`: bounded input
parsing, version/hash validation, plugin capabilities/fuel, no partial mutation
and no secrets in artifacts. Они проверяются соответствующими `fast` и
`content-package` ProductCheck.
