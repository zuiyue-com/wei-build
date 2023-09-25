
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let args: Vec<String> = std::env::args().collect();

    // let mut command = "";
    // if args.len() >= 2 {
    //     command = &args[1];
    // }

    build().await?;
    Ok(())
}

async fn build() -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::get("https://gitea.com/XIU2/TrackersListCollection/raw/branch/master/all.txt").await?;
    let trackers = response.text().await?;

    let os = match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "macos",
        "linux" => "ubuntu",
        _ => "ubuntu"
    };
    
    let contents = fs::read_to_string("../wei/Cargo.toml")
        .expect("Something went wrong reading the file");

    let value = contents.parse::<toml::Value>().unwrap();
    let package = value["package"].clone();
    let version = package["version"].to_string().replace("\"", "");
    
    // 写入 version.dat
    let mut file = File::create("./version.dat")?;
    file.write_all(version.as_bytes())?;
    println!("version:{}", version);
    let src = "./version.dat";
    let dest_dir = format!("../wei-release/{}/{}/data", os.clone(), version.clone());
    let dest_file = format!("../wei-release/{}/{}/data/version.dat", os.clone(), version.clone());
    if !Path::new(&dest_dir).exists() {
        fs::create_dir_all(&dest_dir)?;
    }
    fs::copy(src, &dest_file).unwrap();
    let dest_file = format!("../wei-release/{}/version.dat", os);
    fs::copy(src, &dest_file).unwrap();

    let content = std::fs::read_to_string("./build.dat")?;
    let map: serde_yaml::Value = serde_yaml::from_str(&content)?;

    if let serde_yaml::Value::Mapping(m) = map.clone() {
        for (k, v) in m {
            let name = k.as_str().unwrap();
            println!("build: {}", name);

            let mut cmd = std::process::Command::new("git");
            cmd.arg("pull");
            cmd.current_dir(format!("../{}", name));
            cmd.output().unwrap();

            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("build");
            cmd.arg("--release");
            cmd.current_dir(format!("../{}", name));
            cmd.output().unwrap();

            let suffix = match os {
                "windows" => ".exe",
                _ => ""
            };
            let src = format!("../{}/target/release/{}{}", name, name, suffix.clone());
            let dest_file = format!("../wei-release/{}/{}{}{}", os.clone(), version.clone(), v.as_str().unwrap(), suffix);
            println!("copy: {} -> {}", src, dest_file);
            fs::copy(src, &dest_file).unwrap();
        }
    }

    // 如果../wei-ui-vue文件件存在，则打包wei-ui-vue
    if Path::new("../wei-ui-vue").exists() {
        let mut cmd = std::process::Command::new("git");
        cmd.arg("pull");
        cmd.current_dir("../wei-ui-vue");
        cmd.output().unwrap();

        let mut cmd = std::process::Command::new("C:/Program Files (x86)/Yarn/bin/yarn.cmd");
        cmd.arg("run");
        cmd.arg("build");
        cmd.current_dir("../wei-ui-vue");
        cmd.output().unwrap();

        let src = "../wei-ui-vue/dist";
        let dest_file = format!("../wei-release/{}/{}/data/dist", os.clone(), version.clone());
        copy_files(src, &dest_file).expect("Failed to copy files");
    }
    
    // copy wei.ico
    std::fs::copy(
        format!("../wei/res/wei.ico"),
        format!("../wei-release/{}/{}/data/wei.ico", os.clone(), version.clone())
    ).expect("Failed to copy files");

    // copy wei.ico
    std::fs::copy(
        format!("../wei/res/wei.png"),
        format!("../wei-release/{}/{}/data/wei.png", os.clone(), version.clone())
    ).expect("Failed to copy files");

    // copy daemon.dat
    std::fs::copy(
        format!("./daemon.dat"),
        format!("../wei-release/{}/{}/data/daemon.dat", os.clone(), version.clone())
    ).expect("Failed to copy files");

    // copy daemon.dat
    std::fs::copy(
        format!("./kill.dat"),
        format!("../wei-release/{}/{}/data/kill.dat", os.clone(), version.clone())
    ).expect("Failed to copy files");

    // copy qbittorrent
    copy_files(
        format!("../wei-release/{}/qbittorrent", os.clone()),
        format!("../wei-release/{}/{}/data/qbittorrent", os.clone(), version.clone())
    ).expect("Failed to copy files");

    // copy dist to wei-ui/dist
    copy_files(
        format!("../wei-release/{}/{}/data/dist", os.clone(), version.clone()),
        format!("../wei-ui/dist")
    ).expect("Failed to copy files");

    let checksum_dir = std::path::PathBuf::from(format!("../wei-release/{}/{}", os.clone(), version.clone()));
    let mut checksum_file = File::create(format!("../wei-release/{}/{}/data/checksum.dat", os.clone(), version.clone()))?;
    write_checksums(&checksum_dir, &mut checksum_file, &checksum_dir).expect("Failed to write checksums");

    let from = format!("../wei-release/{}/{}", os.clone(), version.clone());
    let to = format!("../wei-release/{}/latest", os.clone());
    copy_files(from, to).expect("Failed to copy files");

    // make torrent
    let mut cmd = std::process::Command::new("../wei-release/windows/transmission/transmission-create");
    cmd.arg("-o");
    cmd.arg(format!("../wei-release/{}/{}.torrent", os.clone(), version.clone()));
    trackers.lines().filter(|line| !line.trim().is_empty()).for_each(|tracker| {
        cmd.arg("-t");
        cmd.arg(tracker.trim());
    });
    cmd.arg("-s");
    cmd.arg("8192");
    cmd.arg(format!("../wei-release/{}/{}", os.clone(), version.clone()));
    cmd.arg("-c");
    cmd.arg(version.clone());
    cmd.current_dir("../wei-release");
    let output = cmd.output().unwrap();
    println!("status: {}", output.status);
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    // git update
    let mut cmd = std::process::Command::new("git");
    cmd.arg("add");
    cmd.arg("*");
    cmd.current_dir("../wei-release");
    cmd.output().unwrap();

    let mut cmd = std::process::Command::new("git");
    cmd.arg("commit");
    cmd.arg("-am");
    cmd.arg(version);
    cmd.current_dir("../wei-release");
    cmd.output().unwrap();

    let mut cmd = std::process::Command::new("git");
    cmd.arg("push");
    cmd.current_dir("../wei-release");
    cmd.output().unwrap();

    Ok(())
}


use std::io;
fn copy_files<P: AsRef<Path>>(from: P, to: P) -> io::Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    
    if !to.exists() {
        match fs::create_dir_all(&to) {
            Ok(_) => {},
            Err(e) => {
                println!("create dir error: {}", e);
            }
        }
    }

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            match fs::copy(&path, to.join(path.file_name().unwrap())) {
                Ok(_) => {},
                Err(e) => {
                    println!("copy file error: {}", e);                    
                }
            }
        } else if path.is_dir() {
            copy_files(&path, &to.join(path.file_name().unwrap()))?;
        }
    }

    Ok(())
}


use std::fs::{File};
use std::io::{Write, Read};
use sha2::{Sha256, Digest};
fn calculate_sha256<P: AsRef<Path>>(file_path: P) -> io::Result<String> {
    let mut file = File::open(file_path.as_ref())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut hasher = Sha256::new();
    hasher.update(buffer);
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}


fn write_checksums<P: AsRef<Path>>(dir: P
    , checksum_file: &mut File, prefix: &Path) -> io::Result<()> {
    let dir = dir.as_ref();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let relative_path = path.strip_prefix(prefix).unwrap().to_path_buf();
            let sha256 = calculate_sha256(&path)?;
            writeln!(checksum_file, "{}|||{}", relative_path.display(), sha256)?;
        } else if path.is_dir() {
            write_checksums(&path, checksum_file, prefix)?;
        }
    }

    Ok(())
}