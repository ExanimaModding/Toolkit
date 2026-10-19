# emtk_asset

emtk_asset is for deserializing/serializing the binary formats for the game, Exanima, by Bare Mettle Entertainment. The
goal is to make programmatically accessing the game assets very convenient, safe, and fast for use in generic contexts
with the intention of modding the game.

The current state of the project is heavily work-in-progress and there is a lot of work to do.

## Roadmap

The following table is not an exhaustive list of formats that Exanima uses, and this table may grow in the future
after more discovery and better understanding of the game assets.

| Name | Extension | Magic | Rust | Python |
| - | :-: | :-: | :-: | :-: |
| Factory | `.fty` | `0xAFCE0F01`<br/>`0xAFCE0F00` | ❌ | ❌ |
| Power | `.pwr` | `0xAFCE01CE` | ❌ | ❌ |
| Rayform Content | `.rfc` | `0x3D23AFCF`<br/>`0x3D21AFCF` | ❌ | ❌ |
| Rayform Database | `.rdb` | | ❌| ❌ |
| Rayform Image | `.rfi` | `0x1D2D3DC6` | ❌ | ❌ |
| Rayform Package | `.rpk`<br/>`.fds`<br/>`.flb`<br/>`.rml` | `0xAFBF0C01` | ✅ | 🚧 |
| Rayform Project | `.rfp` | `0xAFDFBD10` | ❌ | ❌ |
| Rayform Save Game<br/>Rayform Checkpoint | `.rsg`<br/>`.rcp` | `0xDA7AEA02` | ❌ | ❌ |
| Rayform Sequence | `.rsq` | | ❌ | ❌ |
| Settings | `.set` | `0x6440A401` | ❌ | ❌ |
| Terrain Palette | `.ftb` | `0x3EEFBD01` | ❌ | ❌ |

## Disclaimer

This project is not affiliated with Bare Mettle.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
