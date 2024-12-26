# Changelog

## [1.0.1](https://github.com/cedricziel/baserow-rs/compare/v1.0.0...v1.0.1) (2024-12-26)


### Bug Fixes

* make baserow Debug'able ([#32](https://github.com/cedricziel/baserow-rs/issues/32)) ([c2d2362](https://github.com/cedricziel/baserow-rs/commit/c2d2362a8545cb04d48cfcbc8fe5f54931d504ad))

## [1.0.0](https://github.com/cedricziel/baserow-rs/compare/v0.8.0...v1.0.0) (2024-12-25)


### ⚠ BREAKING CHANGES

* condense get_one_typed
* condense get method on table
* move request building to table operations
* move get into BaserowTable
* condense get_one_typed ([#30](https://github.com/cedricziel/baserow-rs/issues/30))

### Bug Fixes

* correct endpoint for view and add tests ([b379fde](https://github.com/cedricziel/baserow-rs/commit/b379fded9f62131a052c98f9524e3075b27fb747))


### Miscellaneous Chores

* condense get method on table ([b379fde](https://github.com/cedricziel/baserow-rs/commit/b379fded9f62131a052c98f9524e3075b27fb747))
* condense get_one_typed ([b379fde](https://github.com/cedricziel/baserow-rs/commit/b379fded9f62131a052c98f9524e3075b27fb747))
* condense get_one_typed ([#30](https://github.com/cedricziel/baserow-rs/issues/30)) ([b379fde](https://github.com/cedricziel/baserow-rs/commit/b379fded9f62131a052c98f9524e3075b27fb747))
* move get into BaserowTable ([b379fde](https://github.com/cedricziel/baserow-rs/commit/b379fded9f62131a052c98f9524e3075b27fb747))
* move request building to table operations ([b379fde](https://github.com/cedricziel/baserow-rs/commit/b379fded9f62131a052c98f9524e3075b27fb747))

## [0.8.0](https://github.com/cedricziel/baserow-rs/compare/v0.7.0...v0.8.0) (2024-12-25)


### Features

* allow retrieving typed-rows ([#29](https://github.com/cedricziel/baserow-rs/issues/29)) ([660444f](https://github.com/cedricziel/baserow-rs/commit/660444ff178c1c5b5f2517d84f7323a9ae926132))
* allow selection of views ([#26](https://github.com/cedricziel/baserow-rs/issues/26)) ([1e68076](https://github.com/cedricziel/baserow-rs/commit/1e680769bc6fd8d35395975f345efabc64067997))


### Bug Fixes

* extract field mapper ([#28](https://github.com/cedricziel/baserow-rs/issues/28)) ([8c10c34](https://github.com/cedricziel/baserow-rs/commit/8c10c3427da2b6c51f2b0516c0772fcb7ebe61fb))

## [0.7.0](https://github.com/cedricziel/baserow-rs/compare/v0.6.0...v0.7.0) (2024-12-24)


### Features

* add pagination ([#23](https://github.com/cedricziel/baserow-rs/issues/23)) ([6cdad0f](https://github.com/cedricziel/baserow-rs/commit/6cdad0f05ba5f1b54348381305370e175536b55e))

## [0.6.0](https://github.com/cedricziel/baserow-rs/compare/v0.5.1...v0.6.0) (2024-12-24)


### Features

* make library testable ([#21](https://github.com/cedricziel/baserow-rs/issues/21)) ([98acdf2](https://github.com/cedricziel/baserow-rs/commit/98acdf297bcda939cc4fb8a1f27efa2fff09551d))

## [0.5.1](https://github.com/cedricziel/baserow-rs/compare/v0.5.0...v0.5.1) (2024-12-19)


### Bug Fixes

* make fields public ([f1f711d](https://github.com/cedricziel/baserow-rs/commit/f1f711d5497b9da90dfaab0199b796472ca1344f))

## [0.5.0](https://github.com/cedricziel/baserow-rs/compare/v0.4.0...v0.5.0) (2024-12-13)


### Features

* implement table fields retrieval and mapping functionality ([#14](https://github.com/cedricziel/baserow-rs/issues/14)) ([3c31cd4](https://github.com/cedricziel/baserow-rs/commit/3c31cd47b098ac6cd7c494ef2e5f5f084dfab2c0))

## [0.4.0](https://github.com/cedricziel/baserow-rs/compare/v0.3.0...v0.4.0) (2024-12-12)


### Features

* add file upload functionality and update dependencies ([#12](https://github.com/cedricziel/baserow-rs/issues/12)) ([dad9de5](https://github.com/cedricziel/baserow-rs/commit/dad9de5d43d918eab6b99324a300774b3aeb3546))

## [0.3.0](https://github.com/cedricziel/baserow-rs/compare/v0.2.0...v0.3.0) (2024-12-12)


### Features

* add get one record ([#10](https://github.com/cedricziel/baserow-rs/issues/10)) ([45337f3](https://github.com/cedricziel/baserow-rs/commit/45337f3a0c7aef0a517419fb4dbfb8885b85abfe))
* delete one record ([#11](https://github.com/cedricziel/baserow-rs/issues/11)) ([a1af353](https://github.com/cedricziel/baserow-rs/commit/a1af3532be4487eb8db51c08c53654545386818f))
* update rows ([#8](https://github.com/cedricziel/baserow-rs/issues/8)) ([11eaa41](https://github.com/cedricziel/baserow-rs/commit/11eaa4117a210bfc4a65635a9ef8d321a2a556d8))

## [0.2.0](https://github.com/cedricziel/baserow-rs/compare/v0.1.4...v0.2.0) (2024-12-11)


### Features

* add ability to create a record ([0efc9d0](https://github.com/cedricziel/baserow-rs/commit/0efc9d053eeb2c63d8ed1533031b676c2302511a))

## [0.1.4](https://github.com/cedricziel/baserow-rs/compare/v0.1.3...v0.1.4) (2024-12-11)


### Bug Fixes

* allow auto-release ([e0e8673](https://github.com/cedricziel/baserow-rs/commit/e0e8673418160fb6ad90d922067b22d891bbf1f6))

## [0.1.3](https://github.com/cedricziel/baserow-rs/compare/v0.1.2...v0.1.3) (2024-12-11)


### Bug Fixes

* add cargo publishing ([1a78b0f](https://github.com/cedricziel/baserow-rs/commit/1a78b0fbfa1b24dd29fb78b0dd8f33e6c059e516))
* add categories ([f62f777](https://github.com/cedricziel/baserow-rs/commit/f62f7770bb86796efb08b98ec3e9a3e843b6b330))
* more info in the Cargo file ([33d566d](https://github.com/cedricziel/baserow-rs/commit/33d566d449bd7f127e9ffce048da59b65842ec8f))

## [0.1.2](https://github.com/cedricziel/baserow-rs/compare/v0.1.1...v0.1.2) (2024-12-11)


### Bug Fixes

* set license ([b34cf14](https://github.com/cedricziel/baserow-rs/commit/b34cf1453754105133849353b70a5b7b03019118))

## [0.1.1](https://github.com/cedricziel/baserow-rs/compare/v0.1.0...v0.1.1) (2024-12-11)


### Bug Fixes

* add example ([72a6ea2](https://github.com/cedricziel/baserow-rs/commit/72a6ea2ec5efd7d7a74be74a9bea637652229c8f))

## 0.1.0 (2024-12-11)


### Bug Fixes

* rust-edition ([f7adb61](https://github.com/cedricziel/baserow-rs/commit/f7adb61b642d3515c726fae3405259be91b342e3))
