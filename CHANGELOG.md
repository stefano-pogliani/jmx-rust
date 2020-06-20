# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
## [0.2.1]
### Changed
- Upgrade `crossbeam-channel` to version 0.4.2.
- Upgrade `j4rs` to version 0.11.2.

## [0.2.0]
### Changed
- **BREAKING**: Options can't be cloned anymore.
- **BREAKING**: Pass `JvmBuilder` to `MBeanClient` options.
- **BREAKING**: Stop sharing `Jvm` instances across clients.
- **BREAKING**: Upgrade to `failure` for errors.
- Upgrade `crossbeam-channel` to version 0.3.8.
- Upgrade `j4rs` to version 0.5.1.

## [0.1.3] - 2018-09-17
### Added
- Ability to create a threaded client without connecting to a server.

## [0.1.2] - 2018-09-15
### Added
- Derive `Clone` for MBean client and threaded client options.

## [0.1.1] - 2018-09-15
### Added
- MBeanThreadedClient supports re-connection.

## 0.1.0 - 2018-09-06
### Added
- Initial JMX client implementation


[Unreleased]: https://github.com/replicante-io/replicante/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/replicante-io/replicante/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/replicante-io/replicante/compare/v0.1.3...v0.2.0
[0.1.3]: https://github.com/replicante-io/replicante/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/replicante-io/replicante/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/replicante-io/replicante/compare/v0.1.0...v0.1.1
