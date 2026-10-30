from maven import Dependency
from typings import Optional
import dataclasses
import json
import os

GRADLE_ROOT = os.environ["GRUSTLE_GRADLE_ROOT"]
CURRENT_OS = "windows"  # windows, linux, osx
VERSION = "26.2"


@dataclasses.dataclass
class S1LibraryDef:
    repo: str
    group: str
    artifact: str
    version: str
    classifier: Optional[str]
    dl: str


@dataclasses.dataclass
class S2LibraryDef:
    repo: str
    group: str
    artifact: str
    version: str

    classifiers: list[str]
    dependencies: list[str]

    runtime: list[str]
    sources: list[str]
    natives: list[str]


def resolve_full_name(name: str) -> tuple[str, str, str, Optional[str]]:
    split = name.split(":")
    group = split[0]
    artifact = split[1]
    version = split[2]
    classifier = split[3] if len(split) >= 4 else None

    return (group, artifact, version, classifier)


# excludes classifier
def get_full_name(lib: S1LibraryDef) -> str:
    return f"{lib.group}:{lib.artifact}:{lib.version}"


# excludes classifier
def get_full_name_mvn(lib: Dependency) -> str:
    return f"{lib.groupId}:{lib.artifactId}:{lib.version}"


def resolve_jar(repo: str, group: str, artifact: str, version: str, classifier: Optional[str]) -> str:
    def rep(s: str) -> str:
        return s.replace(".", "/")

    cf = f"{classifier}/" if classifier else ""
    return f"{repo}{rep(group)}/{artifact}/{cf}{version}/{artifact}-{version}.jar"


def resolve_pom(repo: str, group: str, artifact: str, version: str, classifier: Optional[str]) -> str:
    def rep(s: str) -> str:
        return s.replace(".", "/")

    cf = f"{classifier}/" if classifier else ""

    return f"{repo}{rep(group)}/{artifact}/{cf}{version}/{artifact}-{version}.pom"


def resolve_cache_path(filename: str) -> str:
    return f"cache/{filename}"


class EnhancedJSONEncoder(json.JSONEncoder):
    def default(self, o):
        if dataclasses.is_dataclass(o):
            return dataclasses.asdict(o)  # type: ignore
        return super().default(o)
