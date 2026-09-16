import json
import os

CURRENT_OS = "windows"  # windows, linux, osx
game_version = "26.2"
dependencies = ""


def put_dependency(name: str, version: str):
    global dependencies
    dependencies += f"\"{name}\" = \"{version}\"\n"


data = json.load(open(f"{game_version}.json"))
dps = {}

for lib in data["libraries"]:
    passes = True
    rules = lib.get("rules")

    if rules:
        for rule in rules:
            # only action type is allow right now lol
            if rule.get("os") and rule.get("os").get("name") != CURRENT_OS:
                passes = False
                break
            elif not rule.get("os") or not rule.get("os").get("name"):
                raise RuntimeError(f"unknown rule: {rule}")

    if passes:
        name = lib["name"]
        split = name.split(":")
        resolved = f"{split[0]}.{split[1]}"
        version = split[2]

        dps[resolved] = version

for d in dps:
    put_dependency(d, dps[d])

dependencies = dependencies.strip()
asset_index = data["assets"]

out = f"""[package]
name = "minecraft"
group = ""
version = "{game_version}"

[extra]
assetIndex = "{asset_index}"

[dependencies]
{dependencies}

[artifact]
runtime = [\"minecraft.jar\"]
sources = [\"minecraft-sources.jar\"]
"""

try:
    os.makedirs(f"../registry/minecraft/{game_version}")
except Exception:
    pass

with open(f"../registry/minecraft/{game_version}/grustle.toml", "w") as f:
    f.write(out)


out = f"""[package]
name = "minecraft-client"
group = ""
version = "{game_version}"

[dependencies]
{dependencies}

[artifact]
runtime = [\"minecraft-client.jar\"]
sources = [\"minecraft-client-sources.jar\"]
"""

try:
    os.makedirs(f"../registry/minecraft-client/{game_version}")
except Exception:
    pass

with open(f"../registry/minecraft-client/{game_version}/grustle.toml", "w") as f:
    f.write(out)
