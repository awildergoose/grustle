import dataclasses
import json
import os

GRADLE_ROOT = os.environ["GRUSTLE_GRADLE_ROOT"]
CURRENT_OS = "windows"  # windows, linux, osx
VERSION = "26.2"


def resolve_full_name(name: str) -> tuple[str, str, str]:
    split = name.split(":")
    group = split[0]
    artifact = split[1]
    version = split[2]

    return (group, artifact, version)


class EnhancedJSONEncoder(json.JSONEncoder):
    def default(self, o):
        if dataclasses.is_dataclass(o):
            return dataclasses.asdict(o)
        return super().default(o)
