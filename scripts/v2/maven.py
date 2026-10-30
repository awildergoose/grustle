import subprocess

from pydantic_xml import BaseXmlModel, element

NSMAP = {"maven": "http://maven.apache.org/POM/4.0.0"}


class Dependency(BaseXmlModel, tag="dependency", ns="maven", nsmap=NSMAP):
    groupId: str = element()
    artifactId: str = element()
    version: str = element()
    scope: str | None = element(default=None)
    classifier: str | None = element(default=None)


class Dependencies(BaseXmlModel, tag="dependencies", ns="maven", nsmap=NSMAP):
    dependency: list[Dependency] = element()


class Project(BaseXmlModel, tag="project", ns="maven", nsmap=NSMAP, search_mode="unordered"):
    groupId: str = element()
    artifactId: str = element()
    version: str = element()
    dependencies: Dependencies = element(default=Dependencies(dependency=[]))


def parse_project(contents: bytes):
    return Project.from_xml(contents)


def get_resolved_pom_path(path: str) -> str:
    return f"{".".join(path.split(".")[:-1])}-resolved.pom"


def normalize_pom(path: str) -> str:
    output = get_resolved_pom_path(path)
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
