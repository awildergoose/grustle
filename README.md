# Grustle

Grustle is a build system specialized for Minecraft mods, it is currently very early in development.

## Features

- Compiling Minecraft mods using `javac`
- Bundling Minecraft mods using `jar`
- Preprocessing Java code and mod resources
- Pretty error reporting!
- Generating an IntelliJ project
- Generating game sources using [cachyflower](https://github.com/awildergoose/cachyflower)
- Applying classtweakers using [quick-tweak](https://github.com/awildergoose/quick-tweak)
- Splitting original Minecraft client and server jars into client only and shared jars
- Easily swapping game versions
- Running the game client, server, and datagen
- Also is way faster than using Gradle

## Future goals

- Obfuscated version support
- NeoForge support

## Creating a new project

`grustle new --namespace me.yourname --identifier myawesomemod --classname MyAwesomeMod --display_name "my awesome mod"`

## Preprocessing

### Resources

For resources, the defines are as follows:

```
project.name
project.path
project.version
project.extra.whatever-keys-you-put-in-the-config
project.packages.whatever-package-from-the-tree.version
```

You can use these defines by surrounding them with `${}` (e.g. `${project.name}` becomes `thenameofyourproject`)

### Sources

For sources, you have the following defines:

```
GRUSTLE (always true)
PROFILE
DEBUG
RELEASE
```

For sources though, you can also use directives like so:

```java
void someFunction() {
    // #if version(minecraft) >= 26.2
    // do something cool here
    // #endif

    // #ifdef DEBUG
    // do something debug only here
    // #endif
}
```

All currently available directives are as follows:

- if
- ifdef
- endif
- define

`if` functions:

- version

###### _grustle... It's like grindr but for business people -dappercore_
