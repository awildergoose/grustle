from typing import Literal, NotRequired, Required, TypedDict, Any, Literal, Optional, TypedDict


class VersionLibraryRuleOs(TypedDict):
    name: str


class VersionLibraryRule(TypedDict):
    action: Literal["allow"]
    os: VersionLibraryRuleOs


class VersionLibraryDownloadsArtifact(TypedDict):
    path: str
    sha1: str
    size: int
    url: str


class VersionLibraryDownloads(TypedDict):
    artifact: VersionLibraryDownloadsArtifact


class VersionLibrary(TypedDict):
    downloads: VersionLibraryDownloads
    name: str
    rules: Optional[list[VersionLibraryRule]]


class VersionDict(TypedDict):
    arguments: Any      # TODO
    assetIndex: Any     # TODO
    assets: str
    complianceLevel: int
    downloads: Any      # TODO
    id: str
    javaVersion: Any    # TODO
    libraries: list[VersionLibrary]
    logging: Any        # TODO
    mainClass: str
    minimumLauncherVersion: int
    releaseTime: str
    time: str
    type: str


class LoaderInfo(TypedDict):
    separator: str
    build: int
    maven: str
    version: str
    stable: bool


class IntermediaryInfo(TypedDict):
    maven: str
    version: str
    stable: bool


class Library(TypedDict):
    name: Required[str]
    url: NotRequired[str]

    md5: NotRequired[str]
    sha1: NotRequired[str]
    sha256: NotRequired[str]
    sha512: NotRequired[str]
    size: NotRequired[int]


class LauncherLibraries(TypedDict):
    client: Required[list[Library]]
    common: Required[list[Library]]
    server: Required[list[Library]]
    development: NotRequired[list[Library]]


class MainClassBySide(TypedDict):
    client: Required[str]
    server: Required[str]


MainClass = str | MainClassBySide


class LaunchArguments(TypedDict):
    client: Required[list[str]]
    common: Required[list[str]]
    server: Required[list[str]]


class LaunchWrapperTweakers(TypedDict):
    client: Required[list[str]]
    common: Required[list[str]]
    server: Required[list[str]]


class LaunchWrapper(TypedDict):
    tweakers: Required[LaunchWrapperTweakers]


class LauncherMeta(TypedDict):
    version: Required[int]
    libraries: Required[LauncherLibraries]

    min_java_version: NotRequired[int]
    mainClass: Required[MainClass]

    arguments: NotRequired[LaunchArguments]
    launchwrapper: NotRequired[LaunchWrapper]


class FabricLoaderVersion(TypedDict):
    loader: Required[LoaderInfo]
    intermediary: Required[IntermediaryInfo]
    launcherMeta: Required[LauncherMeta]


FabricLoaderResponse = list[FabricLoaderVersion]
