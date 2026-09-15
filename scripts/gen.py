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
    
    try:
        os.makedirs(f"../registry/{javaPath}/{pkgName}/{pkgVersion}")
    except Exception:
        pass
    
    toml = f"""[package]
name = "{pkgName}"
path = "{javaPath}"
ver = "{pkgVersion}"

[version]
runtime = []
sources = []

[version.natives]
x64 = []
x86 = []
arm64 = []

[dependencies]

"""
    
    with open(f"../registry/{javaPath}/{pkgName}/{pkgVersion}/grustle.toml", "w") as f:
        f.write(toml)
        f.close()
