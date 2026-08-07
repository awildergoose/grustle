import os
import xmltodict
import toml

reg = open("registry.txt", "r").readlines()

FILES = "F:\\Other\\steve\\.gradle\\caches\\modules-2\\files-2.1\\"

for line in reg:
    line = line.split(FILES)[1].strip()
    javaPath = line.split("\\")[0]
    pkgName = line.split("\\")[1]
    pkgVersion = line.split("\\")[2]
    filename = line.split("\\")[4]
    
    # find the pomFile
    path = f"{FILES}\\{javaPath}\\{pkgName}\\{pkgVersion}"
    pomPath = None

    for folder in os.listdir(path):
        for file in os.listdir(f"{path}\\{folder}"):
            if file.endswith(".pom"):
                pomPath = f"{path}\\{folder}\\{file}"

    dps = {}

    if not pomPath:
        print(f"didnt find pom file for {javaPath}.{pkgName}")
    else:
        with open(pomPath, "r") as pomFile:
            pomTextContent = pomFile.read()
            parsed = xmltodict.parse(pomTextContent)
            if parsed["project"].get("dependencies") == None:
                continue
            dependencies = parsed["project"]["dependencies"]["dependency"]
            if str(type(dependencies)) == "<class 'dict'>":
                dependencies = [dependencies]
            for dep in dependencies:
                if dep.get("scope") == None or dep["scope"] != "test":
                    if (dep.get("optional") == None or dep.get("optional") == False):
                        groupId = dep.get("groupId")
                        artifactId = dep.get("artifactId")
                        if groupId == "${project.groupId}":
                            groupId = javaPath
                        fullName = f"{groupId}.{artifactId}"

                        version = dep.get("version")
                        
                        # I HATE EVERYTHING
                        if fullName == "net.fabricmc.fabric-api.fabric-api-deprecated":
                            continue
                        elif fullName == "org.mockito.mockito-core":
                            continue
                        elif fullName == "org.reflections.reflections":
                            continue
                        elif fullName == "org.osgi.org.osgi.core":
                            continue
                        elif fullName == "com.google.errorprone.error_prone_annotations":
                            continue
                        elif fullName == "com.google.j2objc.j2objc-annotations":
                            continue
                        elif fullName == "io.netty.netty-varhandle-stub":
                            continue
                        elif fullName == "io.netty.netty-jfr-stub":
                            continue
                        elif fullName == "com.google.guava.listenablefuture":
                            continue
                        elif fullName == "org.jboss.jdk-misc":
                            continue
                        elif fullName == "ca.weblite.java-objc-bridge":
                            continue
                        elif fullName == "com.google.code.findbugs.jsr305":
                            continue

                        if fullName == "commons-codec.commons-codec":
                            version = "1.22.0"
                        if fullName == "commons-io.commons-io":
                            version = "2.20.0"
                        elif fullName == "org.apache.commons.commons-lang3":
                            version = "3.20.0"
                        elif fullName == "com.google.code.gson.gson":
                            version = "2.14.0"
                        elif fullName == "org.jetbrains.annotations":
                            version = "26.0.2"
                        elif fullName == "org.slf4j.slf4j-api":
                            version = "2.0.17"
                        elif fullName == "com.google.guava.guava":
                            version = "33.6.0-jre"
                        elif fullName == "it.unimi.dsi.fastutil":
                            version = "8.5.18"
                        elif fullName == "org.ow2.asm.asm-tree":
                            version = "9.10.1"
                        elif fullName == "org.ow2.asm.asm-commons":
                            version = "9.10.1"
                        elif fullName == "org.ow2.asm.asm-util":
                            version = "9.10.1"

                        if not version:
                            # this sucks
                            print(f"guessing latest version for {groupId}.{artifactId} ({javaPath}.{pkgName})")

                            for line2 in reg:
                                line2 = line2.split(FILES)[1].strip()
                                javaPath2 = line2.split("\\")[0]
                                pkgName2 = line2.split("\\")[1]
                                pkgVersion2 = line2.split("\\")[2]
                                filename2 = line2.split("\\")[4]

                                if javaPath2 == groupId and pkgName2 == artifactId:
                                    version = pkgVersion2
                                    break
                            if not version:
                                raise RuntimeError(f"failed to guess version for {groupId}.{artifactId}") 

                        if "$" in version:
                            if version == "${project.version}":
                                version = parsed["project"].get("version") or parsed["project"]["parent"]["version"]
                            else:
                                # i hate this
                                if version == "${slf4j.version}":
                                    version = "2.0.17"
                                elif version == "${failureaccess.version}":
                                    version = "1.0.3"
                                else:
                                    print(f"{"="*50} odd version: {version} ({groupId}.{artifactId})")
                        dps[f"{groupId}.{artifactId}"] = version
    
    f = open(f"../registry/{javaPath}/{pkgName}/fabuild.toml", "r")
    data = toml.loads(f.read())
    f.close()
    data["dependencies"] = dps

    with open(f"../registry/{javaPath}/{pkgName}/fabuild.toml", "w") as f:
        f.write(toml.dumps(data))

