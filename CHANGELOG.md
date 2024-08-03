# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.1] - 2024-08-03

### Added

- Add example mods for reference - ([1269b8c](https://codeberg.org/ExanimaModding/Toolkit/commit/1269b8ca00da90c9ab3d637592052d3bcaa200f6))
- Add xtask commands for development workflow (#25) - ([bb72642](https://codeberg.org/ExanimaModding/Toolkit/commit/bb72642f69d4012f7b6639648e46757368f0f064))
- Add workspace resolver version to /Cargo.toml - ([a89d80a](https://codeberg.org/ExanimaModding/Toolkit/commit/a89d80ab165f0065d60e46ce8ce2a9918460a15a))
- Add git-cliff and woodpecker to automate build releases (#28) - ([25eaeba](https://codeberg.org/ExanimaModding/Toolkit/commit/25eaeba85b2b6b3a88b9160562af2bef2efc1148))
- Add workspace package to the project (#31) - ([7d4b50c](https://codeberg.org/ExanimaModding/Toolkit/commit/7d4b50c0f5850a6874fe59478248d356103beed5))
- Add imgui overlay (#35) - ([40c707b](https://codeberg.org/ExanimaModding/Toolkit/commit/40c707b96fb00a9f8ccb684c53cbb5af99b4c50c))
- Add 'implement' keyword to cliff.toml - ([dcf4816](https://codeberg.org/ExanimaModding/Toolkit/commit/dcf4816aa74c91806fec65d230ea244311603c10))
- Add rust bindings for plugins (#43) - ([c9b060d](https://codeberg.org/ExanimaModding/Toolkit/commit/c9b060d9fdff006136ea93f9cea8289c9befd4cd))

### Changed

- Infinite health (kind of) - ([757bc9b](https://codeberg.org/ExanimaModding/Toolkit/commit/757bc9b97ded0912754cb1b834fa5920ed1f32d9))
- Bug and considerations - ([771b947](https://codeberg.org/ExanimaModding/Toolkit/commit/771b947d015e47716139d94f8d251ab714fbd863))
- Mod injector in rust 🚀🚀 (#2) - ([6dbd9ef](https://codeberg.org/ExanimaModding/Toolkit/commit/6dbd9efee59b9f6a2084a694835d035db3da82db))
- Repacker cli 🚀🚀 - ([6b07abe](https://codeberg.org/ExanimaModding/Toolkit/commit/6b07abe6afb55fee7c9b5cf1e1e8f81b51596704))
- Async unpack - ([46f7b38](https://codeberg.org/ExanimaModding/Toolkit/commit/46f7b38ceb1785cc8e927a494fecc0180d875a99))
- Unoptimized async - ([afcac94](https://codeberg.org/ExanimaModding/Toolkit/commit/afcac94e6b1147ecf34bdf7780c258f9cc9d9445))
- Rfi to dds - ([4f8b284](https://codeberg.org/ExanimaModding/Toolkit/commit/4f8b284b3a502db15f17e3e931f775bcdf967531))
- Setup editorconfig and rustfmt - ([2128911](https://codeberg.org/ExanimaModding/Toolkit/commit/2128911274837ea1424382b4e31a229906881794))
- Format detours, emf, emtk - ([dcdc048](https://codeberg.org/ExanimaModding/Toolkit/commit/dcdc048547c3368d51d5ac0897ff26bb65a094f5))
- Format - ([7629827](https://codeberg.org/ExanimaModding/Toolkit/commit/76298279beb971b29f8ab3f4fe084c0b26baf4d5))
- 100% lua modding - ([e694431](https://codeberg.org/ExanimaModding/Toolkit/commit/e69443135a232c8d3cbc51905cb9099573be0c94))
- Create and attach hooks in lua - ([1557e16](https://codeberg.org/ExanimaModding/Toolkit/commit/1557e1688be7b47b99a87c474c58a723bfc6e32b))
- Recursive unpacking - ([2a4da9f](https://codeberg.org/ExanimaModding/Toolkit/commit/2a4da9f663feb9285506436e7faddb5dca231370))
- Update readme - ([4d02e62](https://codeberg.org/ExanimaModding/Toolkit/commit/4d02e6228dc98f9348787c58210154f6177f6c73))
- Imgui & lua console 🚀 - ([5926b3f](https://codeberg.org/ExanimaModding/Toolkit/commit/5926b3f5e43ec548570e4ddbd0e5556f88210924))
- Formatting indentation - ([f96166b](https://codeberg.org/ExanimaModding/Toolkit/commit/f96166b4179b8bc2c1208527a1c5841aa19e13ab))
- ANSI support for console - ([571fe0d](https://codeberg.org/ExanimaModding/Toolkit/commit/571fe0da9cfc1b1b1ae864a337fd4cca75dcdfc6))
- Loading mods from folder - ([0900d4c](https://codeberg.org/ExanimaModding/Toolkit/commit/0900d4c6a49d7aa9c8e939093d41d587831882f4))
- Ignore require("types") - ([1fca404](https://codeberg.org/ExanimaModding/Toolkit/commit/1fca40441e1bf1c4c6b486fb19940d3e2c799c33))
- Expose sig scanner to lua - ([45fbec1](https://codeberg.org/ExanimaModding/Toolkit/commit/45fbec15dfe7b402af3ad5da10723d18d8193162))
- BIG CHANGES (64bit is real (sort of)) :rocket: - ([bd4d2a7](https://codeberg.org/ExanimaModding/Toolkit/commit/bd4d2a7c0cd2fa869701113f677170c59654696a))
- Find game exe via env or current path - ([faf1de8](https://codeberg.org/ExanimaModding/Toolkit/commit/faf1de87f8871ab1cd88615c9acb88b8f34096ef))
- Find game exe via env - ([7199ea2](https://codeberg.org/ExanimaModding/Toolkit/commit/7199ea29d17f4bff7491b3212482678236ee24f4))
- Change EXANIMA_PATH to EXANIMA_EXE - ([d5a8f79](https://codeberg.org/ExanimaModding/Toolkit/commit/d5a8f79b647a38e89d07bddeca36f601f8aaeb20))
- Format tomls with taplo - ([0e5318a](https://codeberg.org/ExanimaModding/Toolkit/commit/0e5318adad51426c0e195b18977fa14ab47fa714))
- Restructured project (#27) - ([88c900d](https://codeberg.org/ExanimaModding/Toolkit/commit/88c900de9ea9c5e70ffa88572cec3eccde668fa5))
- Clean up workspace dependencies - ([c569ee3](https://codeberg.org/ExanimaModding/Toolkit/commit/c569ee37d0e6f943c0e17e34df16ceefd661011e))
- Refactor logs for repacker - ([2bb1909](https://codeberg.org/ExanimaModding/Toolkit/commit/2bb19092ce6722b9d2b019d80dfa5da86fc062c5))
- Replace toml with ron for metadata in repacker (#32) - ([1518607](https://codeberg.org/ExanimaModding/Toolkit/commit/1518607b5e6233d452a6989606561423bf7756e9))
- Move back to upstream hudhook - ([4d6344e](https://codeberg.org/ExanimaModding/Toolkit/commit/4d6344e8c03fc16ab7e20771157e8245bb43d709))
- Clone detours over http instead of ssh - ([74b09d9](https://codeberg.org/ExanimaModding/Toolkit/commit/74b09d9925ff7e059b1d3768d94fab935166013f))
- Use ssh key for git - ([43ecc78](https://codeberg.org/ExanimaModding/Toolkit/commit/43ecc78496dd98f98dc58fee4d4d729b46b4a015))
- Move artifact files to root of zip file - ([d864731](https://codeberg.org/ExanimaModding/Toolkit/commit/d864731dcca8637f3d72e3ffd4c9a04e8d103bc4))
- Export send_message function, update setting functions - ([bb95ad3](https://codeberg.org/ExanimaModding/Toolkit/commit/bb95ad3a2cb53a881de11094a5c9bb577603d1d7))

### Fixed

- File exceptions - ([d27af9a](https://codeberg.org/ExanimaModding/Toolkit/commit/d27af9ae23b1d9a57d5d37a722c3d627b0595978))
- Filetype prioritization - ([1500018](https://codeberg.org/ExanimaModding/Toolkit/commit/15000183a4d04cd95d83d5e9863340093c1e9788))
- Setting envvar for SteamAppId - ([ab399e2](https://codeberg.org/ExanimaModding/Toolkit/commit/ab399e21d8aaa08891ffc15de0aab947636bd2ff))
- Formatting - ([332c964](https://codeberg.org/ExanimaModding/Toolkit/commit/332c964a3cfd326e67fad1641bd802560ca40eca))
- Formatting issues, share workspace dependencies - ([a02a762](https://codeberg.org/ExanimaModding/Toolkit/commit/a02a76247da6f36e38d32249d1d1e29af1b60772))
- Create mods folder if it's missing (#24) - ([41c44c7](https://codeberg.org/ExanimaModding/Toolkit/commit/41c44c7513678cc62456f4cc441734db39855322))
- Prevent overwriting config.toml - ([a40757b](https://codeberg.org/ExanimaModding/Toolkit/commit/a40757be63f42ce782ff88a3ba0013f07f715eb0))
- Refactor tasks after project restructure - ([c21dd12](https://codeberg.org/ExanimaModding/Toolkit/commit/c21dd12149dfc722181377f690da0c2789bfc94a))
- Add missing closing quote to CI workflow - ([09204c5](https://codeberg.org/ExanimaModding/Toolkit/commit/09204c583bec47024af1622e6d3d03c735a10b92))
- Fix settings not being boxed before being sent to ffi - ([c28e00e](https://codeberg.org/ExanimaModding/Toolkit/commit/c28e00ec225ce9352392ecafd599357d1943fa28))

### Removed

- Remove licensure - ([2fc9b40](https://codeberg.org/ExanimaModding/Toolkit/commit/2fc9b404aea705b3ba1c244a277c7e382acc2c07))
- Remove trivial_bounds feature - ([a195a8e](https://codeberg.org/ExanimaModding/Toolkit/commit/a195a8e11a81561316f039d51e15ea1b9afdfbce))
- Remove unused "emf.h" file - ([903b6b6](https://codeberg.org/ExanimaModding/Toolkit/commit/903b6b65eedbae65f325c057b5b37915e6d6f55c))
- Remove CHANGELOG.md - ([e2a0bde](https://codeberg.org/ExanimaModding/Toolkit/commit/e2a0bde9cf6f852083d43ea10ff945d346f14f7f))


<!-- generated by git-cliff -->
