from typings import Optional
import dataclasses
import json
import os

GRADLE_ROOT = os.environ["GRUSTLE_GRADLE_ROOT"]
CURRENT_OS = "windows"  # windows, linux, osx
VERSION = "26.2"


@dataclasses.dataclass
class LibraryDef:
    group: str
    artifact: str
    version: str
    classifier: Optional[str]
    dl: str


def resolve_full_name(name: str) -> tuple[str, str, str, Optional[str]]:
    split = name.split(":")
    group = split[0]
    artifact = split[1]
    version = split[2]
    classifier = split[3] if len(split) >= 4 else None

    return (group, artifact, version, classifier)


def resolve_jar(repo: str, group: str, artifact: str, version: str, classifier: Optional[str]) -> str:
    def rep(s: str) -> str:
        return s.replace(".", "/")

    cf = f"{classifier}/" if classifier else ""

    # https://maven.fabricmc.net/
    # org/ow2/asm
    # /
    # asm-util
    # /
    # 9.10.1
    # /
    # asm-util-9.10.1.jar
    return f"{repo}{rep(group)}/{artifact}/{cf}{version}/{artifact}-{version}.jar"


def resolve_pom(repo: str, group: str, artifact: str, version: str, classifier: Optional[str]) -> str:
    def rep(s: str) -> str:
        return s.replace(".", "/")

    cf = f"{classifier}/" if classifier else ""

    # https://maven.fabricmc.net/
    # org/ow2/asm
    # /
    # asm-util
    # /
    # 9.10.1
    # /
    # asm-util-9.10.1.pom
    return f"{repo}{rep(group)}/{artifact}/{cf}{version}/{artifact}-{version}.pom"


def resolve_cache_path(filename: str) -> str:
    return f"cache/{filename}"


class EnhancedJSONEncoder(json.JSONEncoder):
    def default(self, o):
        if dataclasses.is_dataclass(o):
            return dataclasses.asdict(o)  # type: ignore
        return super().default(o)
