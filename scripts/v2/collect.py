from dataclasses import dataclass
import requests
from typings import FabricLoaderResponse, Optional, VersionDict
from shared import VERSION, CURRENT_OS, EnhancedJSONEncoder, resolve_full_name, resolve_repo
import json

TEST_DOWNLOADS = False


@dataclass
class LibraryDef:
    group: str
    artifact: str
    version: str
    classifier: Optional[str]
    dl: Optional[str]


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
    dl=None,
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
libraries.append(LibraryDef(
    group="org.jetbrains",
    artifact="annotations",
    classifier=None,
    version="26.0.2",
    dl="https://repo1.maven.org/maven2/org/jetbrains/annotations/26.0.2/annotations-26.0.2.jar"
))
libraries.append(LibraryDef(
    group="net.fabricmc",
    artifact="fabric-log4j-util",
    classifier=None,
    version="1.0.2",
    dl="https://maven.fabricmc.net/net/fabricmc/fabric-log4j-util/1.0.2/fabric-log4j-util-1.0.2.jar"
))
libraries.append(LibraryDef(
    group="net.fabricmc",
    artifact="dev-launch-injector",
    classifier=None,
    version="0.2.1+build.8",
    dl="https://maven.fabricmc.net/net/fabricmc/dev-launch-injector/0.2.1+build.8/dev-launch-injector-0.2.1+build.8.jar"
))

for sset in [
    fabric["launcherMeta"]["libraries"]["common"],
    fabric["launcherMeta"]["libraries"].get("development") or []
]:
    for lib in sset:
        name = lib["name"]
        group, artifact, version, classifier = resolve_full_name(name)
        repo = lib.get("url")
        dl = None

        if repo:
            dl = resolve_repo(repo, group, artifact, version)

            if TEST_DOWNLOADS:
                dl_test = requests.head(dl)
                dl_test.raise_for_status()

        libraries.append(LibraryDef(
            version=version,
            group=group,
            artifact=artifact,
            classifier=classifier,
            dl=dl
        ))

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
