from flask import get_flashed_messages
from enum import Enum


class FlashType(Enum):
    INFO = "info"
    ERROR = "error"


def flash_collect():
    flashed_error = [
        ("error", msg) for msg in get_flashed_messages(False, FlashType.ERROR.name)
    ]

    flashed_info = [
        ("info", msg) for msg in get_flashed_messages(False, FlashType.INFO.name)
    ]

    return flashed_error + flashed_info
