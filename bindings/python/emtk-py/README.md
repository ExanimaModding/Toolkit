# Exanima Modding Toolkit Python

This crate is dedicated to adding python bindings via pyo3. Currently, only the
exparser crate utilizes pyo3 and depends on the pyo3 crate which is re-exported
to this crate. A crate dedicated to re-exporting pyo3 should be made if multiple
crates depend on pyo3.

## Development

[uv](https://github.com/astral-sh/uv) is required for developing with the python bindings.
The same applies to the blender extension.

For developing with the python bindings. The following commands assume it's being ran
at the root of the `Toolkit` project.

```bash
cargo xtask python

# Test if it works by running an example
python ./examples/char_ext.py
```

To see what is exposed to python, look in the `lib.rs` file and the `emtk` folder.
