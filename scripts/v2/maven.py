import subprocess

from pydantic_xml import BaseXmlModel, element
from shared import resolve_cache_path

NSMAP = {"maven": "http://maven.apache.org/POM/4.0.0"}


class Dependency(BaseXmlModel, tag="dependency", ns="maven", nsmap=NSMAP):
    groupId: str = element()
    artifactId: str = element()
    version: str = element()
    scope: str = element()
    classifier: str | None = element(default=None)


class Dependencies(BaseXmlModel, tag="dependencies", ns="maven", nsmap=NSMAP):
    dependency: list[Dependency] = element()


class Project(BaseXmlModel, tag="project", ns="maven", nsmap=NSMAP, search_mode="unordered"):
    groupId: str = element()
    artifactId: str = element()
    version: str = element()
    dependencies: Dependencies = element()


def parse_project(contents: bytes):
    return Project.from_xml(contents)


def normalize_pom(path: str) -> str:
    output = f"{".".join(path.split(".")[:-1])}-resolved.pom"
    subprocess.run([
        "cmd",
        "/C"
        "mvn.cmd",
        "-f",
        path,
        "org.apache.maven.plugins:maven-help-plugin:3.5.2:effective-pom",
        f"-Doutput={output.split("/")[-1]}"  # HACK
    ]).check_returncode()
    return output
