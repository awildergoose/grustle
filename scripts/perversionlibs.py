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

[dependencies]
{dependencies}

[versions."{game_version}"]
runtime = [\"minecraft.jar\"]
sources = [\"minecraft-sources.jar\"]
"""

try:
    os.makedirs("../registry/minecraft")
except Exception:
    pass

with open("../registry/minecraft/fabuild.toml", "w") as f:
    f.write(out)


out = f"""[package]
name = "minecraft-client"
path = ""

[dependencies]
{dependencies}

[versions."{game_version}"]
runtime = [\"minecraft-client.jar\"]
sources = [\"minecraft-client-sources.jar\"]
"""

try:
    os.makedirs("../registry/minecraft-client")
except Exception:
    pass

with open("../registry/minecraft-client/fabuild.toml", "w") as f:
    f.write(out)
