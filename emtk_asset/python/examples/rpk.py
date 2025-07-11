import os
from pathlib import Path

from emtk_asset import Format
from emtk_asset.rpk import Rpk


def main() -> None:
    # Deserializing
    file_path = Path(os.environ["EXANIMA_EXE"]).parent.joinpath("Textures.rpk")
    print("Started deserializing from python")
    buf = file_path.read_bytes()
    format = Format.from_bytes(buf)
    print("Done deserializing from python")

    if isinstance(format._0, Rpk):
        rpk = format._0
        for entry in rpk.entries:
            print(entry.name)

    # Serializing
    file_path = file_path.parent.joinpath("Custom.rpk")
    print("Started serializing from python")
    buf = format.to_bytes()
    file_path.write_bytes(buf)
    print("Done serializing from python")


if __name__ == "__main__":
    main()
