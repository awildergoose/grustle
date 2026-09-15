import os

reg = open("registry2.txt", "r").readlines()

FILES = "F:\\Other\\steve\\.gradle\\caches\\modules-2\\files-2.1\\"

for line in reg:
    line = line.strip()
    if line.endswith("mappings.jar"):
        continue
    if "\\remapped\\" in line:
        continue
    line = line.split(FILES)[1].strip()
    javaPath = line.split("\\")[0]
    pkgName = line.split("\\")[1]
    pkgVersion = line.split("\\")[2]
    filename = line.split("\\")[4]

    with open(f"../registry/{javaPath}/{pkgName}/{pkgVersion}/grustle.toml", "r") as f:
        content = f.read()
    
    content = content.replace(f"""[version.natives]
x64 = []
x86 = []
arm64 = []
""", "")
    with open(f"../registry/{javaPath}/{pkgName}/{pkgVersion}/grustle.toml", "w") as f:
        f.write(content)
