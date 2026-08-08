import os

reg = open("registry.txt", "r").readlines()

for line in reg:
    line = line.split("F:\\Other\\steve\\.gradle\\caches\\modules-2\\files-2.1\\")[1].strip()
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
    
    with open(f"../registry/{javaPath}/{pkgName}/{pkgVersion}/fabuild.toml", "w") as f:
        f.write(toml)
        f.close()
