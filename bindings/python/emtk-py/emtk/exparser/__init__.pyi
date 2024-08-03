from typing import Union
from .fty import Fty
from .rfc import Rfc
from .rfi import Rfi
from .rpk import Rpk
from .wav import Wav

class Unknown:
    data: bytes

class Format:
    _0: Union[Fty, Rfc, Rfi, Rpk, Wav, Unknown]

    @staticmethod
    def from_bytes(data: bytes) -> Format: ...
    def to_bytes(self) -> bytes: ...
