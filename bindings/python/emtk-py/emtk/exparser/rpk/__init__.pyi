MAGIC = int

class Rpk:
    """Rayform Package"""

    entries: list[Entry]
    data: list[bytes]

class Entry:
    name: str
    offset: int
    size: int
