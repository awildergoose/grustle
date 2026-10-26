use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use zip::{ZipArchive, write::SimpleFileOptions};

use crate::{jregistry::GrustleJRegistry, project::GrustleProject};

// https://github.com/FabricMC/fabric-loom/blob/dev/1.19/src/main/java/net/fabricmc/loom/configuration/providers/minecraft/MinecraftJarSplitter.java
#[allow(clippy::too_many_lines)]
pub fn invoke_splitter(
    _root: &Path,
    _profile: &str,
    project: &GrustleProject,
    jregistry: &GrustleJRegistry,
) -> anyhow::Result<()> {
    #[allow(clippy::type_complexity)]
    fn get_entries(
        path: &Path,
        archive: &mut ZipArchive<File>,
    ) -> anyhow::Result<(HashSet<PathBuf>, HashMap<PathBuf, Vec<u8>>)> {
        let mut entries = HashSet::new();
        let mut data = HashMap::new();

        for index in 0..archive.len() {
            let mut file = archive.by_index(index)?;

            // block out symlinks
            if !file.is_file() {
                continue;
            }

            let filename = file.enclosed_name().context(format!(
                "unsafe zip file entry was found in {}",
                path.display()
            ))?;

            if filename.starts_with("META-INF/") {
                continue;
            }

            let mut fdata = vec![];
            file.read_to_end(&mut fdata)?;

            entries.insert(filename.clone());
            data.insert(filename.clone(), fdata);
        }

        Ok((entries, data))
    }

    fn copy_entries(
        entries: HashSet<PathBuf>,
        data: &HashMap<PathBuf, Vec<u8>>,
        output: &Path,
        env: &str,
    ) -> anyhow::Result<()> {
        let options = SimpleFileOptions::default();

        let buf = Cursor::new(Vec::new());
        let mut zip = zip::write::ZipWriter::new(buf);

        for entry in entries {
            let data = data.get(&entry).ok_or_else(|| unreachable!())?;
            zip.start_file_from_path(entry, options)?;
            zip.write_all(data)?;
        }

        let manifest =
            format!("Manifest-Version: 1.0\nFabric-Loom-Split-Environment-Name: {env}\n\n");
        zip.start_file("META-INF/MANIFEST.MF", options)?;
        zip.write_all(&manifest.into_bytes())?;

        let bytes = zip.finish()?.into_inner();
        #[allow(clippy::unnecessary_debug_formatting)] // Clippy is stupid!
        std::fs::write(output, bytes).context(format!("writing jar to {output:?}"))?;

        Ok(())
    }

    let game_version = &project
        .dependencies
        .get("minecraft")
        .ok_or_else(|| anyhow::anyhow!("minecraft is not in the dependency list!"))?
        .version
        .clone();
    let game_client_version = &project
        .dependencies
        .get("minecraft-client")
        .ok_or_else(|| anyhow::anyhow!("minecraft-client is not in the dependency list!"))?
        .version
        .clone();

    let client = jregistry
        .resolve_path("minecraft-raw", game_version)?
        .join("client.jar");
    let server = jregistry
        .resolve_path("minecraft-raw", game_version)?
        .join("server.jar");

    let mut client_zip = zip::read::ZipArchive::new(
        File::open(&client).context(format!("opening jar at {}", client.display()))?,
    )
    .context(format!("parsing jar at {}", client.display()))?;
    let mut server_zip = zip::read::ZipArchive::new(
        File::open(&server).context(format!("opening jar at {}", server.display()))?,
    )
    .context(format!("parsing jar at {}", server.display()))?;

    let (client_entries, client_data) = get_entries(&client, &mut client_zip)?;
    let (server_entries, server_data) = get_entries(&server, &mut server_zip)?;

    let mut common_entries: HashSet<PathBuf> = client_entries
        .intersection(&server_entries)
        .cloned()
        .collect();
    let mut client_only_entries: HashSet<PathBuf> = client_entries
        .difference(&server_entries)
        .cloned()
        .collect();

    // https://github.com/FabricMC/fabric-loom/blob/dev/1.19/src/main/java/net/fabricmc/loom/configuration/providers/minecraft/SplitMinecraftProvider.java#L82
    let shared_entries: Vec<PathBuf> = [
        "version.json",
        "assets/.mcassetsroot",
        "assets/minecraft/lang/en_us.json",
    ]
    .iter()
    .map(std::convert::Into::into)
    .collect();

    for entry in shared_entries {
        common_entries.insert(entry.clone());
        client_only_entries.insert(entry);
    }

    if cfg!(debug_assertions) {
        let server_only_entries: HashSet<PathBuf> = server_entries
            .difference(&client_entries)
            .cloned()
            .collect(); // 0
        anyhow::ensure!(
            server_only_entries.is_empty(),
            "this should always be 0, otherwise, we need to split into 3 jars"
        );
    }

    let output_common = jregistry
        .resolve_path("minecraft", game_version)?
        .join("minecraft.jar");
    let output_client = jregistry
        .resolve_path("minecraft-client", game_client_version)?
        .join("minecraft-client.jar");

    copy_entries(common_entries, &server_data, &output_common, "common")?;
    copy_entries(client_only_entries, &client_data, &output_client, "client")?;

    Ok(())
}
