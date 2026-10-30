"""Portable paths in B03 reports and captured evidence."""
import re
from common import ROOT

# Reports include command lines and tool output as well as path fields. Keep
# paths under ROOT relative and remove machine-specific paths everywhere else.
HOST_PATH = re.compile(r"(?<![A-Za-z0-9_.:/-])(?:[A-Za-z]:[\\/]|/)[^\s\"'<>]+")

def portable(value):
    if isinstance(value, dict):
        return {key: portable(item) for key, item in value.items()}
    if isinstance(value, list):
        return [portable(item) for item in value]
    if not isinstance(value, str):
        return value
    def replace(match):
        path = match.group(0)
        root = str(ROOT)
        if path == root:
            return "."
        if path.startswith((root + "/", root + "\\")):
            return path[len(root) + 1:].replace("\\", "/")
        if match.start() == 0 and match.end() == len(value):
            return path.replace("\\", "/").rsplit("/", 1)[-1] or "<host-path>"
        return "<host-path>"
    return HOST_PATH.sub(replace, value)


