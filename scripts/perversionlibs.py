import json
import os

CURRENT_OS = "windows" # windows, linux, osx
game_version = "26.2"
dependencies = ""

def put_dependency(name: str, version: str):
    global dependencies
    dependencies += f"\"{name}\" = \"{version}\"\n"

data = json.load(open("26.2.json"))
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

out = f"""[package]
name = "minecraft"
path = ""
ver = "{game_version}"

[dependencies]
{dependencies}

[version]
runtime = [\"minecraft.jar\"]
sources = [\"minecraft-sources.jar\"]
"""

try:
    os.makedirs(f"../registry/minecraft/{game_version}")
except Exception:
    pass

with open(f"../registry/minecraft/{game_version}/fabuild.toml", "w") as f:
    f.write(out)


out = f"""[package]
name = "minecraft-client"
path = ""

[dependencies]
{dependencies}

[version]
runtime = [\"minecraft-client.jar\"]
sources = [\"minecraft-client-sources.jar\"]
"""

try:
    os.makedirs(f"../registry/minecraft-client/{game_version}")
except Exception:
    pass

with open(f"../registry/minecraft-client/{game_version}/fabuild.toml", "w") as f:
    f.write(out)
