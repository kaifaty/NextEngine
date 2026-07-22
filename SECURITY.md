# Security policy

NextEngine является local pre-release bootstrap и пока не имеет public support window или permanent private disclosure endpoint.

- `security_disclosure_status: BootstrapOutOfBand`
- `public_contact_status: AwaitingCapability`
- `supported_release_status: PreReleaseOnly`

Не добавляйте в Git secrets, signing/reviewer keys, user installations, imported game data, datasets, checkpoints, training runs, models или evidence media.

## Reporting during bootstrap

Создайте локальный redacted report без exploit payloads/protected bytes и уведомите repository owner по существующему out-of-band каналу. Не публикуйте детали до triage. Bootstrap owner стремится подтвердить получение ≤3 business days и выполнить initial triage ≤7 days, но это не является public SLA.

## Public-release blocker

До public release MUST быть настроен permanent monitored private disclosure contact, supported-version policy, severity/embargo/credit process и rotation/revocation runbook. Пока contact отсутствует, `GOV-01` возвращает `AwaitingCapability`, а не `PASS`.

Security-sensitive boundaries и exact gates определены в `docs/architecture/11-security-licensing-and-governance.md`. Importer publication дополнительно требует отдельного written legal approval.
