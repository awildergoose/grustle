from maven import parse_project
from shared import VERSION, CURRENT_OS, EnhancedJSONEncoder, LibraryDef, resolve_full_name, resolve_jar, resolve_pom

from concurrent.futures import ThreadPoolExecutor, as_completed
from typings import FabricLoaderResponse, VersionDict
import json
import requests
import time
import os

TEST_DOWNLOADS = False
ADD_FABRIC_API = True

FABRIC_MAVEN = "https://maven.fabricmc.net/"
MVNCEN_MAVEN = "https://repo1.maven.org/maven2/"

version_info: VersionDict = json.load(open(f"{VERSION}.json"))
libraries: list[LibraryDef] = []

for library in version_info["libraries"]:
    passes = True
    rules = library.get("rules")

    if rules:
        for rule in rules:
            # only action type is allow right now lol
            if rule["os"] and rule["os"]["name"] != CURRENT_OS:
                passes = False
                break

    if passes:
        name = library["name"]
        group, artifact, version, classifier = resolve_full_name(name)
        identifier = f"{group}.{artifact}"

        libraries.append(LibraryDef(
            version=version,
            group=group,
            artifact=artifact,
            classifier=classifier,
            dl=library["downloads"]["artifact"]["url"],
        ))

fabric_version_info: FabricLoaderResponse = json.load(
    open(f"{VERSION}-fabric.json"))
fabric = fabric_version_info[2]

name = fabric["loader"]["maven"]
group, artifact, version, classifier = resolve_full_name(name)

libraries.append(LibraryDef(
    version=fabric["loader"]["version"],
    group=group,
    artifact=artifact,
    classifier=classifier,
    dl=resolve_jar(FABRIC_MAVEN, group, artifact, version, classifier),
))

# fabric-loom adds these in
group = "org.jetbrains"
artifact = "annotations"
classifier = None
version = "26.0.2"

libraries.append(LibraryDef(
    group=group,
    artifact=artifact,
    classifier=classifier,
    version=version,
    dl=resolve_jar(MVNCEN_MAVEN, group, artifact, version, classifier)
))

group = "net.fabricmc"
artifact = "fabric-log4j-util"
version = "1.0.2"

libraries.append(LibraryDef(
    group=group,
    artifact=artifact,
    classifier=classifier,
    version=version,
    dl=resolve_jar(FABRIC_MAVEN, group, artifact, version, classifier)
))

artifact = "dev-launch-injector"
version = "0.2.1+build.8"

libraries.append(LibraryDef(
    group=group,
    artifact=artifact,
    classifier=classifier,
    version=version,
    dl=resolve_jar(FABRIC_MAVEN, group, artifact, version, classifier)
))

for sset in [
    fabric["launcherMeta"]["libraries"]["common"],
    fabric["launcherMeta"]["libraries"].get("development") or []
]:
    for lib in sset:
        name = lib["name"]
        group, artifact, version, classifier = resolve_full_name(name)
        repo = lib.get("url")
        if not repo:
            raise RuntimeError(f"{name} doesn't have a repo set")
        dl = resolve_jar(repo, group, artifact, version, classifier)

        libraries.append(LibraryDef(
            version=version,
            group=group,
            artifact=artifact,
            classifier=classifier,
            dl=dl
        ))

# fabric-api
if ADD_FABRIC_API:
    pom = resolve_pom(FABRIC_MAVEN, "net.fabricmc",
                      "fabric-api", "0.155.2+26.2", "fabric-api")
    filename = pom.split("/")[-1]
    if not os.path.exists(filename):
        res = requests.get(pom)
        res.raise_for_status()

        with open(filename, "wb") as f:
            f.write(res.content)

    with open(filename, "rb") as f:
        contents = f.read()
        project = parse_project(contents)
        libraries.append(LibraryDef(
            group=project.groupId,
            artifact=project.artifactId,
            version=project.version,
            classifier=None,
            dl=resolve_jar(FABRIC_MAVEN, project.groupId,
                           project.artifactId, project.version, None)
        ))
        dependencies = project.dependencies.dependency

    for dep in dependencies:
        if dep.artifactId == "fabric-api-deprecated":
            continue
        libraries.append(LibraryDef(
            group=dep.groupId,
            artifact=dep.artifactId,
            version=dep.version,
            classifier=dep.classifier,
            dl=resolve_jar(FABRIC_MAVEN, dep.groupId,
                           dep.artifactId, dep.version, dep.classifier)
        ))


if TEST_DOWNLOADS:
    def test(url):
        for attempt in range(5):
            response = requests.head(url, timeout=10)

            if response.status_code == 429:
                time.sleep(2 ** attempt)
                continue

            response.raise_for_status()
            return url

    with ThreadPoolExecutor(max_workers=3) as pool:
        futures = [pool.submit(test, lib.dl) for lib in libraries]

        for future in as_completed(futures):
            try:
                print(future.result())
            except Exception as exc:
                print(f"Request failed: {exc}")

json.dump(libraries, open("collected.json", "w"),
          cls=EnhancedJSONEncoder, indent=4)

out = ""

for lib in libraries:
    classifier = f"-{lib.classifier}" if lib.classifier else ""
    out += f"{lib.group}/{lib.artifact}/{lib.version}/{lib.artifact}-{lib.version}{classifier}.jar\n"

out = "\n".join(sorted(out.splitlines()))
out += "\n"

with open("sample.txt", "w") as f:
    f.write(out)
