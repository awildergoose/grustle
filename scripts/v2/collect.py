from dataclasses import dataclass
import dataclasses
from typings import FabricLoaderResponse, Optional, VersionDict
from shared import VERSION, CURRENT_OS, EnhancedJSONEncoder, resolve_full_name
import json


@dataclass
class LibraryDef:
    group: str
    artifact: str
    version: str
    repo: Optional[str]


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
        group, artifact, version = resolve_full_name(name)
        identifier = f"{group}.{artifact}"

        libraries.append(LibraryDef(
            version=version, group=group, artifact=artifact, repo=None))

fabric_version_info: FabricLoaderResponse = json.load(
    open(f"{VERSION}-fabric.json"))
fabric = fabric_version_info[0]

name = fabric["loader"]["maven"]
group, artifact, version = resolve_full_name(name)

libraries.append(LibraryDef(
    version=fabric["loader"]["version"],
    group=group,
    artifact=artifact,
    repo=None
))

name = fabric["intermediary"]["maven"]
group, artifact, version = resolve_full_name(name)

libraries.append(LibraryDef(
    version=fabric["intermediary"]["version"],
    group=group,
    artifact=artifact,
    repo=None
))

for lib in fabric["launcherMeta"]["libraries"]["common"]:
    name = lib["name"]
    group, artifact, version = resolve_full_name(name)

    libraries.append(LibraryDef(
        version=version,
        group=group,
        artifact=artifact,
        repo=lib.get("url")
    ))

json.dump(libraries, open("collected.json", "w"),
          cls=EnhancedJSONEncoder, indent=4)
