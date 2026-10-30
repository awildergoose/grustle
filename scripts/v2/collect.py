from maven import get_resolved_pom_path, normalize_pom, parse_project
from shared import VERSION, CURRENT_OS, EnhancedJSONEncoder, S1LibraryDef, S2LibraryDef, get_full_name, get_full_name_mvn, resolve_cache_path, resolve_full_name, resolve_jar, resolve_pom

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
MINECRAFT_MAVEN = "https://libraries.minecraft.net/"

version_info: VersionDict = json.load(open(f"{VERSION}.json"))
s1libraries: list[S1LibraryDef] = []

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

        s1libraries.append(S1LibraryDef(
            version=version,
            group=group,
            artifact=artifact,
            classifier=classifier,
            dl=library["downloads"]["artifact"]["url"],
            repo=MINECRAFT_MAVEN
        ))

fabric_version_info: FabricLoaderResponse = json.load(
    open(f"{VERSION}-fabric.json"))
fabric = fabric_version_info[2]

name = fabric["loader"]["maven"]
group, artifact, version, classifier = resolve_full_name(name)

s1libraries.append(S1LibraryDef(
    version=fabric["loader"]["version"],
    group=group,
    artifact=artifact,
    classifier=classifier,
    dl=resolve_jar(FABRIC_MAVEN, group, artifact, version, classifier),
    repo=FABRIC_MAVEN
))

# fabric-loom adds these in
group = "org.jetbrains"
artifact = "annotations"
classifier = None
version = "26.0.2"

s1libraries.append(S1LibraryDef(
    group=group,
    artifact=artifact,
    classifier=classifier,
    version=version,
    dl=resolve_jar(MVNCEN_MAVEN, group, artifact, version, classifier),
    repo=MVNCEN_MAVEN
))

group = "net.fabricmc"
artifact = "fabric-log4j-util"
version = "1.0.2"

s1libraries.append(S1LibraryDef(
    group=group,
    artifact=artifact,
    classifier=classifier,
    version=version,
    dl=resolve_jar(FABRIC_MAVEN, group, artifact, version, classifier),
    repo=FABRIC_MAVEN
))

artifact = "dev-launch-injector"
version = "0.2.1+build.8"

s1libraries.append(S1LibraryDef(
    group=group,
    artifact=artifact,
    classifier=classifier,
    version=version,
    dl=resolve_jar(FABRIC_MAVEN, group, artifact, version, classifier),
    repo=FABRIC_MAVEN
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

        assert dl.startswith(FABRIC_MAVEN)

        s1libraries.append(S1LibraryDef(
            version=version,
            group=group,
            artifact=artifact,
            classifier=classifier,
            dl=dl,
            repo=FABRIC_MAVEN
        ))


def do_maven_project(path: str):
    path = normalize_pom(path)

    with open(path, "rb") as f:
        contents = f.read()
        project = parse_project(contents)
        s1libraries.append(S1LibraryDef(
            group=project.groupId,
            artifact=project.artifactId,
            version=project.version,
            classifier=None,
            dl=resolve_jar(FABRIC_MAVEN, project.groupId,
                           project.artifactId, project.version, None),
            repo=FABRIC_MAVEN
        ))
        dependencies = project.dependencies.dependency

    for dep in dependencies:
        if dep.artifactId == "fabric-api-deprecated":
            continue
        s1libraries.append(S1LibraryDef(
            group=dep.groupId,
            artifact=dep.artifactId,
            version=dep.version,
            classifier=dep.classifier,
            dl=resolve_jar(FABRIC_MAVEN, dep.groupId,
                           dep.artifactId, dep.version, dep.classifier),
            repo=FABRIC_MAVEN
        ))


# fabric-api
if ADD_FABRIC_API:
    pom = resolve_pom(FABRIC_MAVEN, "net.fabricmc",
                      "fabric-api", "0.155.2+26.2", "fabric-api")
    path = resolve_cache_path(pom.split("/")[-1])

    if not os.path.exists(path):
        res = requests.get(pom)
        res.raise_for_status()

        with open(path, "wb") as f:
            f.write(res.content)

    if not os.path.exists(get_resolved_pom_path(path)):
        do_maven_project(path)

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
        futures = [pool.submit(test, lib.dl) for lib in s1libraries]

        for future in as_completed(futures):
            try:
                print(future.result())
            except Exception as exc:
                print(f"Request failed: {exc}")

pom_files: list[tuple[S1LibraryDef, str]] = []

for lib in s1libraries:
    pom = resolve_pom(lib.repo, lib.group, lib.artifact,
                      lib.version, None)
    path = resolve_cache_path(pom.split("/")[-1])

    if not os.path.exists(path):
        res = requests.get(pom)
        res.raise_for_status()

        with open(path, "wb") as f:
            f.write(res.content)

for lib in s1libraries:
    pom = resolve_pom(lib.repo, lib.group, lib.artifact,
                      lib.version, None)
    path = resolve_cache_path(pom.split("/")[-1])

    if not os.path.exists(get_resolved_pom_path(path)):
        do_maven_project(path)
    pom_files.append((lib, get_resolved_pom_path(path)))

full_names: list[str] = []

for l in s1libraries:
    full_names.append(get_full_name(l))

s2libraries: list[S2LibraryDef] = []

for (l, pom) in pom_files:
    # TODO: stop duplicates
    fl = get_full_name(l)
    lib = S2LibraryDef(
        repo=l.repo,
        group=l.group,
        artifact=l.artifact,
        version=l.version,
        classifiers=[ll.classifier for ll in s1libraries if get_full_name(
            ll) == fl and ll.classifier],
        natives=[],
        runtime=[],
        sources=[],
        dependencies=[],  # set later
    )

    with open(pom, "rb") as f:
        project = parse_project(f.read())
        for dep in project.dependencies.dependency:
            if dep.scope == "test":
                continue
            if dep.scope == "provided":
                continue
            full_name = get_full_name_mvn(dep)
            if full_name not in full_names:
                continue
            lib.dependencies.append(full_name)

    s2libraries.append(lib)

json.dump(s1libraries, open("s1.json", "w"),
          cls=EnhancedJSONEncoder, indent=4)
json.dump(s2libraries, open("s2.json", "w"),
          cls=EnhancedJSONEncoder, indent=4)

out = ""

for lib in s1libraries:
    classifier = f"-{lib.classifier}" if lib.classifier else ""
    out += f"{lib.group}/{lib.artifact}/{lib.version}/{lib.artifact}-{lib.version}{classifier}.jar\n"

out = "\n".join(sorted(out.splitlines()))
out += "\n"

with open("sample.txt", "w") as f:
    f.write(out)
