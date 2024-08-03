# Exanima Modding Toolkit Python

This crate is dedicated to adding python bindings via pyo3. Currently, only the
exparser crate utilizes pyo3 and depends on the pyo3 crate which is re-exported
to this crate. A crate dedicated to re-exporting pyo3 should be made if multiple
crates depend on pyo3.

## Development

For developing with the python bindings.

```bash
uv venv
uv pip install maturin
./.venv/Scripts/activate.ps1
# run the next command any time new changes are added to the package
maturin develop --uv
python ./examples/char_ext.py
```

To see what is exposed to python, look in the `lib.rs` file and the `emtk` folder.
