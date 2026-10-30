from typings import FabricLoaderResponse, VersionDict
from shared import VERSION, CURRENT_OS, EnhancedJSONEncoder, LibraryDef, resolve_full_name, resolve_repo
import json
import requests
import time
from concurrent.futures import ThreadPoolExecutor, as_completed

FABRIC_MAVEN = "https://maven.fabricmc.net/"
MVNCEN_MAVEN = "https://repo1.maven.org/maven2/"
TEST_DOWNLOADS = False

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
    dl=resolve_repo(FABRIC_MAVEN, group, artifact, version),
))

# name = fabric["intermediary"]["maven"]
# group, artifact, version, classifier = resolve_full_name(name)

# libraries.append(LibraryDef(
#     version=fabric["intermediary"]["version"],
#     group=group,
#     artifact=artifact,
#     classifier=classifier,
#     dl=None
# ))

# fabric-loom adds these in
# /org/jetbrains/annotations/26.0.2/annotations-26.0.2.jar"
group = "org.jetbrains"
artifact = "annotations"
classifier = None
version = "26.0.2"

libraries.append(LibraryDef(
    group=group,
    artifact=artifact,
    classifier=classifier,
    version=version,
    dl=resolve_repo(MVNCEN_MAVEN, group, artifact, version)
))

group = "net.fabricmc"
artifact = "fabric-log4j-util"
version = "1.0.2"

libraries.append(LibraryDef(
    group=group,
    artifact=artifact,
    classifier=classifier,
    version=version,
    dl=resolve_repo(FABRIC_MAVEN, group, artifact, version)
))

artifact = "dev-launch-injector"
version = "0.2.1+build.8"

libraries.append(LibraryDef(
    group=group,
    artifact=artifact,
    classifier=classifier,
    version=version,
    dl=resolve_repo(FABRIC_MAVEN, group, artifact, version)
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
        dl = resolve_repo(repo, group, artifact, version)

        libraries.append(LibraryDef(
            version=version,
            group=group,
            artifact=artifact,
            classifier=classifier,
            dl=dl
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


good = open("goodsample.txt", "r").read()
