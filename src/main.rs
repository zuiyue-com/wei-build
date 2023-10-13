
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let mut command = "";
    if args.len() >= 2 {
        command = &args[1];
    }

    match command {
        "build" => {
            let mut product_name = "wei";
            if args.len() >= 3 {
                product_name = &args[2];
            }

            build(&product_name).await?;
        }
        "test" => {
            let mut product_name = "wei";
            if args.len() >= 3 {
                product_name = &args[2];
            }

            test(&product_name).await?;
        }
        _ => {
            help();
        }
    }

    Ok(())
}

async fn test(product_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let os = match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "macos",
        "linux" => "ubuntu",
        _ => "ubuntu"
    };

    let config_path = format!("./data/{}/{}/", product_name, os);
    let path = Path::new(&config_path);
    if !path.exists() {
        println!("配置文件不存在，需要创建./data/{}/{}，具体配置请参考README.md", product_name, os);
        return Ok(());
    } 

    let build_path = format!("{}build.dat", config_path);
    let content = std::fs::read_to_string(&build_path)?;
    let map: serde_yaml::Value = serde_yaml::from_str(&content)?;

    if let serde_yaml::Value::Mapping(m) = map.clone() {
        for (k, _) in m {
            let name = k.as_str().unwrap();
            println!("test: {}", name);

            let suffix = match os {
                "windows" => ".exe",
                _ => ""
            };
            let src = format!("../{}/target/debug/{}{}", name, name, suffix);
            
            let output = std::process::Command::new("../wei-release/windows/virustotal/vt.exe")
                .arg("scan")
                .arg("file")
                .arg(src)
                .output()?;

            // 输出返回的 stdout 和 stderr
            if !output.stdout.is_empty() {
                let s = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = s.split(' ').collect();
                let result = format!("https://www.virustotal.com/gui/file-analysis/{}", parts[1]);
                println!("stdout: {}", result);
            }
            if !output.stderr.is_empty() {
                println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
    }

    Ok(())
}

fn help() {
    let args: Vec<String> = std::env::args().collect();
    println!("Usage:");
    println!("  {} build <product>", args[0]);
    println!("  {} test", args[0]);
}

async fn build(product_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::get("https://cdn.jsdelivr.net/gh/ngosang/trackerslist@master/trackers_all.txt").await?;
    let trackers = response.text().await?;

    let os = match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "macos",
        "linux" => "ubuntu",
        _ => "ubuntu"
    };

    let config_path = format!("./data/{}/{}/", product_name, os);
    let path = Path::new(&config_path);
    // println!("{:?},{}", path,path.exists());
    if !path.exists() {
        println!("配置文件不存在，需要创建./data/{}/{}，具体配置请参考README.md", product_name, os);
        return Ok(());
    } 

    let version_path = format!("{}version.dat", config_path);
    let version = fs::read_to_string(&version_path).expect("Something went wrong reading the file");
    let version = version.trim();

    let release_path = format!("../wei-release/{}/{}/{}/", product_name, os, version);
    let release_data_path = format!("{}data/", release_path);
    let release_os_path = format!("../wei-release/{}/{}/", product_name, os);
    
    println!("version:{}", version);
    let src = version_path;
    let dest_dir = release_data_path.clone();
    let dest_file = format!("{}version.dat", release_data_path.clone());
    if !Path::new(&dest_dir).exists() {
        fs::create_dir_all(&dest_dir)?;
    }
    fs::copy(src.clone(), &dest_file).unwrap();
    let dest_file = format!("../wei-release/{}/{}/version.dat", product_name, os);
    fs::copy(src, &dest_file).unwrap();

    let build_path = format!("{}build.dat", config_path);
    let content = std::fs::read_to_string(&build_path)?;
    let map: serde_yaml::Value = serde_yaml::from_str(&content)?;

    if let serde_yaml::Value::Mapping(m) = map.clone() {
        for (k, v) in m {
            let name = k.as_str().unwrap();
            println!("build: {}", name);
            let suffix = match os {
                "windows" => ".exe",
                _ => ""
            };

            // 先检测 product/windows/wei-updater.exe 是否存在，如果存在则不再编译,复制wei-updater.exe到 product/windows/version/data/wei-updater.exe
            let src = format!("{}stable/{}{}", release_os_path.clone(), name, suffix);
            let dest_file = format!("{}{}{}", release_path, v.as_str().unwrap(), suffix);
            
            if Path::new(&src).exists() {
                println!("copy: {} -> {}", src, dest_file);
                fs::copy(src, &dest_file).unwrap();
                continue;
            }
            
            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("build");
            // cmd.arg("--release");
            cmd.current_dir(format!("../{}", name));
            cmd.output().unwrap();

            let src = format!("../{}/target/debug/{}{}", name, name, suffix);
            println!("copy: {} -> {}", src, dest_file);
            fs::copy(src, &dest_file).unwrap();
        }
    }

    // 如果../wei-ui-vue文件件存在，则打包wei-ui-vue
    if !Path::new("../wei-ui-vue/dist").exists() {
        let mut cmd = std::process::Command::new("git");
        cmd.arg("pull");
        cmd.current_dir("../wei-ui-vue");
        cmd.output().unwrap();

        let mut cmd = std::process::Command::new("C:/Program Files (x86)/Yarn/bin/yarn.cmd");
        cmd.arg("run");
        cmd.arg("build");
        cmd.current_dir("../wei-ui-vue");
        cmd.output().unwrap();
    }

    let src = "../wei-ui-vue/dist";
    let dest_file = format!("{}dist", release_data_path.clone());
    copy_files(src, &dest_file).expect("Failed to copy files");
    
    std::fs::copy(
        format!("../wei/res/wei.ico"),
        format!("{}wei.ico", release_data_path.clone())
    ).expect("Failed to copy files");

    std::fs::copy(
        format!("../wei/res/wei.png"),
        format!("{}wei.png", release_data_path.clone())
    ).expect("Failed to copy files");

    std::fs::copy(
        format!("../wei-daemon/wei-daemon.ps1"),
        format!("{}wei-daemon.ps1", release_data_path.clone())
    ).expect("Failed to copy files");

    std::fs::copy(
        format!("../wei-daemon/wei-daemon-close.ps1"),
        format!("{}wei-daemon-close.ps1", release_data_path.clone())
    ).expect("Failed to copy files");

    std::fs::copy(
        format!("../wei-run/wei-close.ps1"),
        format!("{}wei-close.ps1", release_data_path.clone())
    ).expect("Failed to copy files");

    std::fs::copy(
        format!("../wei-updater/wei-updater.ps1"),
        format!("{}wei-updater.ps1", release_data_path.clone())
    ).expect("Failed to copy files");

    std::fs::copy(
        format!("../wei-ui/Webview2.exe"),
        format!("{}Webview2.exe", release_data_path.clone())
    ).expect("Failed to copy files");    

    std::fs::copy(
        format!("{}daemon.dat", config_path),
        format!("{}daemon.dat", release_data_path.clone())
    ).expect("Failed to copy files");

    std::fs::copy(
        format!("{}download.dat", config_path),
        format!("{}download.dat", release_data_path.clone())
    ).expect("Failed to copy files");

    std::fs::copy(
        format!("{}product.dat", config_path),
        format!("{}product.dat", release_data_path.clone())
    ).expect("Failed to copy files");

    std::fs::copy(
        format!("{}build.dat", config_path),
        format!("{}build.dat", release_data_path.clone())
    ).expect("Failed to copy files");

    std::fs::copy(
        format!("{}kill.dat", config_path),
        format!("{}kill.dat", release_data_path.clone())
    ).expect("Failed to copy files");

    copy_files(
        format!("../wei-release/{}/qbittorrent", os),
        format!("{}qbittorrent", release_data_path.clone())
    ).expect("Failed to copy files");

    copy_files(
        format!("{}dist", release_data_path.clone()),
        format!("../wei-ui/dist")
    ).expect("Failed to copy files");

    let checksum_dir = std::path::PathBuf::from(release_path.clone());
    let mut checksum_file = File::create(format!("{}checksum.dat", release_data_path.clone()))?;
    write_checksums(&checksum_dir, &mut checksum_file, &checksum_dir).expect("Failed to write checksums");

    let from = release_path.clone();
    let to = format!("../wei-release/{}/{}/latest", product_name, os);
    fs::create_dir_all(to.clone())?;
    fs::remove_dir_all(to.clone())?;
    copy_files(from, to).expect("Failed to copy files");

    // make torrent
    let mut cmd = std::process::Command::new("../wei-release/windows/transmission/transmission-create");
    cmd.arg("-o");
    cmd.arg(format!("../wei-release/{}/{}/{}.torrent", product_name, os, version));
    trackers.lines().filter(|line| !line.trim().is_empty()).for_each(|tracker| {
        cmd.arg("-t");
        cmd.arg(tracker.trim());
    });
    cmd.arg("-s");
    cmd.arg("8192");
    cmd.arg(release_path.clone());
    cmd.arg("-c");
    cmd.arg(version);
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