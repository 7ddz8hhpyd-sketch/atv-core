"""Python 3.14 compatible wrapper around pyatv's atvremote script.

pyatv's atvremote calls asyncio.get_event_loop() at startup, which raises on
Python >= 3.12 when no loop exists. Create one first, then delegate.
Set PYATV_DIR to the pyatv repo checkout (default: sibling of this repo).
"""
import asyncio
import os
import sys

PYATV_DIR = os.environ.get(
    "PYATV_DIR", "/Users/corvofeng/GitRepo/pyatv"
)
sys.path.insert(0, PYATV_DIR)
asyncio.set_event_loop(asyncio.new_event_loop())

from pyatv.scripts.atvremote import main

if __name__ == "__main__":
    sys.exit(main())
