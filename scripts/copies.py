import os
import toml
import shutil

reg = open("registry.txt", "r").readlines()

FILES = "F:\\Other\\steve\\.gradle\\caches\\modules-2\\files-2.1\\"

for line in reg:
    line = line.split(FILES)[1].strip()
    javaPath = line.split("\\")[0]
    pkgName = line.split("\\")[1]
    pkgVersion = line.split("\\")[2]
    filename = line.split("\\")[4]
    
    # find the jars
    path = f"{FILES}\\{javaPath}\\{pkgName}\\{pkgVersion}"
    jars = []

    for folder in os.listdir(path):
        for file in os.listdir(f"{path}\\{folder}"):
            if file.endswith(".jar"):
                jars.append(f"{path}\\{folder}\\{file}")


    for jar in jars:
        src = jar
        dst = f"G:/steve/.fabuild/{javaPath}/{pkgName}/{pkgVersion}"
        
        try:
            os.makedirs(dst)
        except Exception:
            pass

        dst += f"/{jar.split("\\")[-1]}"

        # print(f"src: {src}, dst: {dst}")
        shutil.copyfile(src, dst)
        # break
    
    allJars = []
    
    runtimeJars = []
    sourcesJars = []
    
    nativesX64Jars = []
    nativesX86Jars = []
    nativesArm64Jars = []

    for jar in jars:
        allJars.append(jar.split("\\")[-1])
    
    for jar in allJars:
        if jar.endswith("-sources.jar"):
            sourcesJars.append(jar)
            continue
        if "-natives-" in jar:
            if "arm64" in jar:
                nativesArm64Jars.append(jar)
            elif "x86" in jar or "jtracy" in jar: # special case for com.mojang.jtracy
                nativesX86Jars.append(jar)
            else:
                nativesX64Jars.append(jar)
                if "lwjgl" not in jar:
                    print(f"guessing x64 for {jar} ({javaPath}.{pkgName})")
            continue

        runtimeJars.append(jar)

    f = open(f"../registry/{javaPath}/{pkgName}/fabuild.toml", "r")
    data = toml.loads(f.read())
    f.close()
    data["versions"][pkgVersion]["runtime"] = runtimeJars
    data["versions"][pkgVersion]["sources"] = sourcesJars
    data["versions"][pkgVersion]["natives"]["x64"] = nativesX64Jars
    data["versions"][pkgVersion]["natives"]["x86"] = nativesX86Jars
    data["versions"][pkgVersion]["natives"]["arm64"] = nativesArm64Jars

    with open(f"../registry/{javaPath}/{pkgName}/fabuild.toml", "w") as f:
        out = toml.dumps(data)
        
        # Prettify
        out = out.replace("\",]\n", "\" ]\n")
        
        f.write(out)