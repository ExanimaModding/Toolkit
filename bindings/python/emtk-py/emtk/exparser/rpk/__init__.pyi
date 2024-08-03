MAGIC = int

class Rpk:
    """Rayform Package"""

    entries: list[Entry]
    data: bytes

class Entry:
    name: str
    offset: int
    size: int
