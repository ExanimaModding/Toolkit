# Blender Extension

This is the source for EMTK's blender extension. The `blender` submodule does not get
bundled with the `emtk` python module.

## Development

[uv](https://github.com/astral-sh/uv) is required for developing with the blender extension.
You'll also need to add blender to your PATH environment variable.

Blender ^4.2 is required due to the addition of blender extensions. The following commands
assume it's being ran at the root of the `Toolkit` project.

```bash
uv pip install -r ./bindings/python/emtk-py/emtk/blender/requirements.txt
cargo xtask blender
```

After building, go into Blender and toggle the extension locally via the toolbar with
`Edit` > `Preferences` > `Add-ons`. Find `Exanima Modding Toolkit` and toggle it to make
sure to re-register any new changes to take effect or restart blender entirely.
