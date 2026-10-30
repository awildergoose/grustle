from pydantic_xml import BaseXmlModel, element

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
