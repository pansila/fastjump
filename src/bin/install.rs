use anyhow::{anyhow, bail, Result};
use const_format::concatcp;
use fastjump::common::opts::InstallOpts;
use fastjump::common::utils::{get_app_path, get_install_path, into_level};
use fastjump::{copy_in, format_path};
use log::{debug, info};
use std::borrow::Cow;
#[cfg(target_family = "windows")]
use std::fs::read;
use std::fs::{create_dir_all, remove_dir_all, remove_file, OpenOptions};
use std::io::ErrorKind;
use std::io::Write;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
#[cfg(target_family = "unix")]
use {log::warn, std::ffi::OsStr, std::fs::copy, std::fs::read_to_string, std::io::LineWriter};

const PKGNAME: &str = env!("CARGO_PKG_NAME");
#[cfg(target_family = "unix")]
const SUPPORTED_SHELLS: &[&str] = &["bash", "zsh", "fish", "tcsh"];

#[derive(Default, Debug)]
struct Config {
    prefix: String,
    install_dir: PathBuf,

    bin_dir: PathBuf,
    etc_dir: PathBuf,
    doc_dir: PathBuf,
    share_dir: PathBuf,

    #[cfg(target_family = "unix")]
    zshshare_dir: PathBuf,
    #[cfg(target_family = "windows")]
    clink_dir: PathBuf,

    custom_install: bool,
}

impl Config {
    pub fn new() -> Self {
        let mut config = Config {
            prefix: "".to_string(),
            install_dir: get_install_path(),
            ..Default::default()
        };
        debug!("Default install location: {}", config.install_dir.display());

        #[cfg(target_family = "windows")]
        {
            let parent = config
                .install_dir
                .parent()
                .expect("parent of the installation location doesn't exist");
            config.clink_dir = parent.join("clink");

            if !config.clink_dir.exists() {
                panic!(
                    "clink has not been installed, expecting at {}",
                    config.clink_dir.display()
                );
            }
        }
        #[cfg(target_family = "unix")]
        {
            config.zshshare_dir = config.install_dir.join("functions");
        }

        config
    }

    pub fn update(&mut self) {
        self.bin_dir = self.install_dir.join(&self.prefix).join("bin");
        self.etc_dir = self.install_dir.join("etc").join("profile.d");
        self.doc_dir = self
            .install_dir
            .join(&self.prefix)
            .join("share")
            .join("man")
            .join("man1");
        self.share_dir = self
            .install_dir
            .join(&self.prefix)
            .join("share")
            .join(PKGNAME);

        #[cfg(target_family = "unix")]
        {
            self.zshshare_dir = self.install_dir.join("functions");
        }
    }

    pub fn update_from_opts(&mut self, opts: &InstallOpts) -> Result<()> {
        if let Some(Some(install)) = &opts.install {
            if Path::new(install) != self.install_dir {
                self.custom_install = true;
                self.install_dir = PathBuf::from(install);
            }
            // TODO: create it by default?
            if !self.install_dir.exists() {
                bail!("Destination install directory doesn't exist");
            }
        }
        if let Some(prefix) = &opts.prefix {
            if prefix != &self.prefix {
                self.custom_install = true;
            }
            self.prefix = prefix.clone();
        }

        #[cfg(target_family = "unix")]
        if opts.system {
            self.install_dir = PathBuf::from("/");
            self.prefix = "/usr/local".to_string();
        }

        self.update();

        // TODO: normalize path
        #[cfg(target_family = "windows")]
        if let Some(clinkdir) = &opts.clinkdir {
            if clinkdir != &self.clink_dir {
                self.custom_install = true;
            }
            self.clink_dir = clinkdir.clone();
            if !self.clink_dir.exists() {
                bail!("Specified clink directory doesn't exist");
            }
        }
        #[cfg(target_family = "unix")]
        if let Some(zshshare) = &opts.zshshare {
            if zshshare != &self.zshshare_dir {
                self.custom_install = true;
            }
            self.zshshare_dir = zshshare.clone();
            if !self.zshshare_dir.exists() {
                bail!("Specified zshshare directory doesn't exist");
            }
        }
        #[cfg(target_family = "unix")]
        if opts.system {
            if self.custom_install {
                bail!("Custom paths incompatible with --system option.");
            }
            self.zshshare_dir = PathBuf::from("/usr/share/zsh/site-functions");
        }

        debug!("config after updating from opts: {:#?}", self);

        Ok(())
    }
}

#[cfg(target_family = "unix")]
#[link(name = "c")]
extern "C" {
    fn geteuid() -> u32;
}

/// Checks if any files are present within a directory and all sub-directories.
fn is_empty_dir(path: &Path) -> Result<bool> {
    let next = path.read_dir()?.next();
    if let Some(dir) = next {
        let entry = dir?;
        if entry.file_type()?.is_file() {
            Ok(false)
        } else {
            Ok(is_empty_dir(&entry.path())?)
        }
    } else {
        Ok(true)
    }
}

#[cfg(target_family = "unix")]
fn get_shell() -> String {
    Path::new(
        shellexpand::env("$SHELL")
            .unwrap_or_else(|_| Cow::from(""))
            .as_ref(),
    )
    .file_name()
    .unwrap_or_else(|| OsStr::new(""))
    .to_str()
    .unwrap_or("")
    .to_string()
}

fn check_opts(opts: &InstallOpts) -> Result<()> {
    if opts.force {
        return Ok(());
    }

    if opts.system {
        #[cfg(target_family = "windows")]
        bail!("System-wide installation is not supported on Windows.");
        #[cfg(target_family = "unix")]
        if unsafe { geteuid() != 0 } {
            bail!("Please rerun as root for system-wide installation.");
        }
    }
    #[cfg(target_family = "unix")]
    {
        let shell = get_shell();
        if !SUPPORTED_SHELLS.contains(&shell.as_str()) {
            bail!(
                "Unsupported shell: {}, we currently only support {:?}",
                shell,
                SUPPORTED_SHELLS
            );
        }
    }
    Ok(())
}

fn copy_in_dryrun(file: &Path, path: &Path, dryrun: bool) -> Result<()> {
    info!("Copying {} => {}", file.display(), path.display());
    if !dryrun {
        copy_in!(file, path)?;
    }
    Ok(())
}

fn create_dir_dryrun(dir: &Path, dryrun: bool) -> Result<()> {
    info!("Creating the path {}", dir.display());
    if !dryrun {
        create_dir_all(dir)?;
    }
    Ok(())
}

#[cfg(target_family = "unix")]
fn get_rc_file(etc_dir: &Path, share_dir: &Path) -> (String, String) {
    let rcfile;
    let source_msg;
    if get_shell() == "fish" {
        let aj_shell = format!("{}/{}.fish", share_dir.display(), PKGNAME);
        source_msg = format!("if test -f {}; . {}; end", aj_shell, aj_shell);
        rcfile = "~/.config/fish/config.fish".to_string();
    } else {
        let aj_shell = format!("{}/{}.sh", etc_dir.display(), PKGNAME);
        source_msg = format!("[[ -s {} ]] && source {}", aj_shell, aj_shell);

        if cfg!(target_os = "macos") && get_shell() == "bash" {
            rcfile = "~/.profile".to_string();
        } else {
            rcfile = format!("~/.{}rc", get_shell());
        }
    }
    (rcfile, source_msg)
}

fn post_install(_etc_dir: &Path, _share_dir: &Path, _bin_dir: &Path, _dryrun: bool) -> Result<()> {
    #[cfg(target_family = "windows")]
    println!(
        "\nPlease manually add {} to your user 'PATH'",
        _bin_dir.display()
    );
    #[cfg(target_family = "unix")]
    {
        let (rcfile, source_msg) = get_rc_file(_etc_dir, _share_dir);

        if get_shell() == "zsh" {
            println!("\n\tautoload -U compinit && compinit -u");
        }

        info!("Add {} to the rcfile {}", PKGNAME, rcfile);
        if let Err(e) = modify_bin_rcfile(&rcfile, &source_msg, _dryrun, true) {
            debug!("{}", e);
            warn!("Failed to add {} to {}", PKGNAME, rcfile);
            info!("Please manually add the following line(s) to {}:", rcfile);
            info!("{}", source_msg);
        }
        info!("");
        info!("Please restart terminal(s) to take effect.");
        info!("");
        info!(
            "If you want to try '{}' in the current shell, please run the following line manually.",
            PKGNAME
        );
        info!(
            "source {}",
            source_msg
                .split_whitespace()
                .last()
                .unwrap_or("Error: no source file found")
        );
    }

    Ok(())
}

#[cfg(target_family = "unix")]
fn modify_bin_rcfile(rcfile: &str, source_msg: &str, dryrun: bool, install: bool) -> Result<()> {
    debug!("Modifying the rcfile {}", rcfile);
    if dryrun {
        return Ok(());
    }
    let rcfile = shellexpand::tilde(rcfile);
    if install {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(rcfile.as_ref())?;
        writeln!(file, "\n{}\n", source_msg)?;
    } else {
        copy(rcfile.as_ref(), (rcfile.to_owned() + ".bak").as_ref())?;

        let lines = read_to_string(rcfile.as_ref())?;
        let lines = lines.lines().filter(|line| !line.contains(source_msg));

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(rcfile.as_ref())?;
        let mut buffer = LineWriter::new(file);
        for line in lines {
            buffer.write_all(line.as_bytes())?;
            buffer.write_all(&[b'\n'])?;
        }
        buffer.flush()?;
    }
    Ok(())
}

#[cfg(target_family = "windows")]
/// Prepend custom FASTJUMP_BIN_DIR definition to fastjump.lua
fn modify_bin_lua(clink_dir: &Path, bin_dir: &Path, dryrun: bool) -> Result<()> {
    debug!("modifying the lua script");
    if dryrun {
        return Ok(());
    }

    let custom_install = format!(
        "local {}_BIN_DIR = \"{}\"\n",
        PKGNAME.to_ascii_uppercase(),
        bin_dir.display().to_string().replace("\\", "\\\\"),
    );

    let clink_file = clink_dir.join(format!("{}.lua", PKGNAME));
    let original = read(clink_file.as_path())?;
    let mut file = OpenOptions::new().write(true).open(clink_file)?;
    file.write_all(custom_install.as_bytes())?;
    file.write_all(original.as_ref())?;
    Ok(())
}

#[cfg(target_family = "unix")]
/// Append custom installation path to fastjump.sh
fn modify_bin_sh(etc_dir: &Path, share_dir: &Path, dryrun: bool) -> Result<()> {
    debug!("modifying the sh script");
    if dryrun {
        return Ok(());
    }

    let custom_install = format!(
        "\
        \n# check custom install \
        \nif [ -s {}/{}.${{shell}} ]; then \
        \n    source {}/{}.${{shell}} \
        \nfi\n",
        share_dir.display(),
        PKGNAME,
        share_dir.display(),
        PKGNAME
    );

    let etc_file = etc_dir.join(concatcp!(PKGNAME, ".sh"));
    let mut file = OpenOptions::new().write(true).open(etc_file)?;
    file.write_all(custom_install.as_bytes())?;

    Ok(())
}

fn handle_install(config: &Config, opts: &InstallOpts) -> Result<()> {
    if opts.dryrun {
        info!(
            "Installing {} to {} (DRYRUN)...",
            PKGNAME,
            config.install_dir.display()
        );
    } else {
        info!(
            "Installing {} to {}...",
            PKGNAME,
            config.install_dir.display()
        );
    }

    create_dir_dryrun(&config.bin_dir, opts.dryrun)?;
    create_dir_dryrun(&config.doc_dir, opts.dryrun)?;
    create_dir_dryrun(&config.share_dir, opts.dryrun)?;
    #[cfg(target_family = "unix")]
    {
        create_dir_dryrun(&config.zshshare_dir, opts.dryrun)?;
        create_dir_dryrun(&config.etc_dir, opts.dryrun)?;
    }

    let target_dirs;
    #[cfg(target_family = "unix")]
    {
        let target_dir = shellexpand::env("$target_dir").unwrap_or_else(|_| Cow::from(""));
        target_dirs = [
            format_path!("target", target_dir.as_ref(), "release", PKGNAME),
            format_path!("target", target_dir.as_ref(), "debug", PKGNAME),
        ];
    }
    #[cfg(target_family = "windows")]
    {
        let target = concatcp!(PKGNAME, ".exe");
        let target_dir = shellexpand::env("$TARGET").unwrap_or_else(|_| Cow::from(""));
        target_dirs = [
            format_path!("target", target_dir.as_ref(), "release", target),
            format_path!("target", target_dir.as_ref(), "debug", target),
        ];
    }
    let mut found = false;
    for target in &target_dirs {
        if target.exists() {
            copy_in_dryrun(target.as_path(), &config.bin_dir, opts.dryrun)?;
            found = true;
            break;
        }
    }
    if !found {
        bail!("target not found in the dirs {:?}", target_dirs);
    }

    copy_in_dryrun(
        format_path!("assets", "icon.png").as_path(),
        &config.share_dir,
        opts.dryrun,
    )?;
    copy_in_dryrun(
        format_path!("doc", concatcp!(PKGNAME, ".1")).as_path(),
        &config.doc_dir,
        opts.dryrun,
    )?;

    #[cfg(target_family = "windows")]
    {
        copy_in_dryrun(
            format_path!("scripts", concatcp!(PKGNAME, ".lua")).as_path(),
            &config.clink_dir,
            opts.dryrun,
        )?;
        copy_in_dryrun(
            format_path!("scripts", "j.bat").as_path(),
            &config.bin_dir,
            opts.dryrun,
        )?;
        copy_in_dryrun(
            format_path!("scripts", "jc.bat").as_path(),
            &config.bin_dir,
            opts.dryrun,
        )?;
        copy_in_dryrun(
            format_path!("scripts", "jo.bat").as_path(),
            &config.bin_dir,
            opts.dryrun,
        )?;
        copy_in_dryrun(
            format_path!("scripts", "jco.bat").as_path(),
            &config.bin_dir,
            opts.dryrun,
        )?;

        if config.custom_install {
            modify_bin_lua(&config.clink_dir, &config.bin_dir, opts.dryrun)?;
        }
    }
    #[cfg(target_family = "unix")]
    {
        copy_in_dryrun(
            format_path!("scripts", concatcp!(PKGNAME, ".sh")).as_path(),
            &config.etc_dir,
            opts.dryrun,
        )?;
        copy_in_dryrun(
            format_path!("scripts", concatcp!(PKGNAME, ".bash")).as_path(),
            &config.share_dir,
            opts.dryrun,
        )?;
        copy_in_dryrun(
            format_path!("scripts", concatcp!(PKGNAME, ".fish")).as_path(),
            &config.share_dir,
            opts.dryrun,
        )?;
        copy_in_dryrun(
            format_path!("scripts", concatcp!(PKGNAME, ".zsh")).as_path(),
            &config.share_dir,
            opts.dryrun,
        )?;
        copy_in_dryrun(
            format_path!("scripts", "_j").as_path(),
            &config.zshshare_dir,
            opts.dryrun,
        )?;

        if config.custom_install {
            modify_bin_sh(&config.etc_dir, &config.share_dir, opts.dryrun)?;
        }
    }

    post_install(
        &config.etc_dir,
        &config.share_dir,
        &config.bin_dir,
        opts.dryrun,
    )?;

    Ok(())
}

fn rmdir_dryrun(path: &Path, dryrun: bool) -> Result<()> {
    info!("Remove the whole directory {}", path.display());
    if !dryrun {
        if let Err(err) = remove_dir_all(path) {
            if err.kind() != ErrorKind::NotFound {
                bail!(err);
            }
        }
    }
    Ok(())
}

fn rm_dryrun(file: &Path, dryrun: bool) -> Result<()> {
    info!("Remove the file {}", file.display());
    if !dryrun {
        if let Err(err) = remove_file(file) {
            if err.kind() != ErrorKind::NotFound {
                bail!(err);
            }
        }
    }
    Ok(())
}

fn remove_default_installation(dryrun: bool) -> Result<()> {
    let mut config = Config::new();
    config.update();
    if config.install_dir.exists() {
        info!("Found default installation...");
        rmdir_dryrun(&config.install_dir, dryrun)?;

        #[cfg(target_family = "windows")]
        if config.clink_dir.exists() {
            rm_dryrun(&config.clink_dir.join(format!("{}.lua", PKGNAME)), dryrun)?;
        }
        #[cfg(target_family = "unix")]
        {
            rm_dryrun(&config.zshshare_dir.join("_j"), dryrun)?;
        }
    }
    Ok(())
}

fn remove_custom_installation(config: &Config, dryrun: bool) -> Result<()> {
    if config.install_dir.exists() {
        info!("Found custom installation...");
        rmdir_dryrun(&config.install_dir, dryrun)?;

        #[cfg(target_family = "windows")]
        if config.clink_dir.exists() {
            rm_dryrun(&config.clink_dir.join(format!("{}.lua", PKGNAME)), dryrun)?;
        }
        #[cfg(target_family = "unix")]
        {
            rm_dryrun(&config.zshshare_dir.join("_j"), dryrun)?;
        }
    }
    Ok(())
}

fn remove_system_installation(config: &mut Config, dryrun: bool) -> Result<()> {
    if cfg!(target_family = "windows") {
        return Ok(());
    }

    config.install_dir = PathBuf::from("/");
    config.prefix = "/usr/local".to_string();
    config.update();

    if !config.bin_dir.join(PKGNAME).exists() {
        return Ok(());
    }

    info!("Found system installation...");

    #[cfg(target_family = "unix")]
    if unsafe { geteuid() != 0 } {
        bail!("Please rerun as root for system-wide uninstall, aborting...");
    }

    #[cfg(target_family = "unix")]
    {
        rm_dryrun(&config.bin_dir.join(PKGNAME), dryrun)?;
        rm_dryrun(&config.etc_dir.join(concatcp!(PKGNAME, ".sh")), dryrun)?;
        rm_dryrun(&config.share_dir.join(concatcp!(PKGNAME, ".bash")), dryrun)?;
        rm_dryrun(&config.share_dir.join(concatcp!(PKGNAME, ".fish")), dryrun)?;
        rm_dryrun(&config.share_dir.join(concatcp!(PKGNAME, ".tcsh")), dryrun)?;
        rm_dryrun(&config.share_dir.join(concatcp!(PKGNAME, ".zsh")), dryrun)?;
        rmdir_dryrun(&config.share_dir, dryrun)?;
        rm_dryrun(&config.zshshare_dir.join("_j"), dryrun)?;
    }
    #[cfg(target_family = "windows")]
    {
        rm_dryrun(&config.bin_dir.join(concatcp!(PKGNAME, ".exe")), dryrun)?;
        rm_dryrun(&config.bin_dir.join("j.bat"), dryrun)?;
        rm_dryrun(&config.bin_dir.join("jc.bat"), dryrun)?;
        rm_dryrun(&config.bin_dir.join("jco.bat"), dryrun)?;
        rm_dryrun(&config.bin_dir.join("jo.bat"), dryrun)?;
        if config.clink_dir.exists() {
            rm_dryrun(&config.clink_dir.join(format!("{}.lua", PKGNAME)), dryrun)?;
        }
    }
    rm_dryrun(&config.doc_dir.join(concatcp!(PKGNAME, ".1")), dryrun)?;
    if is_empty_dir(&config.install_dir)? {
        rmdir_dryrun(&config.install_dir, dryrun)?;
    }

    Ok(())
}

#[cfg(target_family = "unix")]
fn cleanup_source_file(config: &Config, dryrun: bool) -> Result<()> {
    let (rcfile, source_msg) = get_rc_file(&config.etc_dir, &config.share_dir);
    info!("Clean up {} stuff from rcfile {}", PKGNAME, rcfile);

    if let Err(e) = modify_bin_rcfile(&rcfile, &source_msg, dryrun, false) {
        warn!("Failed to revert changes from {}", rcfile);
        info!("{} has been saved to {}.bak", rcfile, rcfile);
        info!(
            "Please manually remove the following line(s) from {}",
            rcfile
        );
        // TODO: add colors
        info!("{}", source_msg);
        bail!(e);
    }
    Ok(())
}

fn remove_user_data(dryrun: bool) -> Result<()> {
    let data_home = get_app_path().join(PKGNAME);
    if data_home.exists() {
        info!("Found user data...");
        rmdir_dryrun(&data_home, dryrun)?;
    }
    Ok(())
}

fn handle_uninstall(config: &mut Config, opts: &InstallOpts) -> Result<()> {
    if opts.dryrun {
        info!("Uninstalling {} (DRYRUN)...", PKGNAME);
    } else {
        info!("Uninstalling {}...", PKGNAME);
    }

    #[cfg(target_family = "unix")]
    cleanup_source_file(config, opts.dryrun)?;
    remove_default_installation(opts.dryrun)?;
    remove_custom_installation(config, opts.dryrun)?;
    remove_system_installation(config, opts.dryrun)?;
    if opts.purge {
        remove_user_data(opts.dryrun)?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let opts = InstallOpts::from_args();

    let mut builder = env_logger::builder();
    #[cfg(not(debug_assertions))]
    let builder = builder.format_timestamp(None).format_module_path(false);
    builder
        .filter_level(into_level(log::LevelFilter::Info as u32 + opts.verbose))
        .parse_default_env()
        .init();

    let mut config = Config::new();
    check_opts(&opts)?;
    config.update_from_opts(&opts)?;

    if opts.install.is_some() {
        handle_install(&config, &opts)?;
    } else if opts.uninstall {
        handle_uninstall(&mut config, &opts)?;
    } else {
        // TODO
        // opts.print_help();
    }

    Ok(())
}
