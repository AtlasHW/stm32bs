use assert_cmd::prelude::*;
use indoc::indoc;
use std::fs;
use std::io::Write;
use std::process::Command;
use std::str;

use crate::helpers::project::Project;
use tempfile::TempDir;

pub struct ProjectBuilder {
    files: Vec<(String, String)>,
    submodules: Vec<(String, String)>,
    root: TempDir,
    git: bool,
    branch: Option<String>,
    tag: Option<String>,
}

pub fn tempdir() -> ProjectBuilder {
    ProjectBuilder {
        files: Vec::new(),
        submodules: Vec::new(),
        root: tempfile::Builder::new()
            .prefix("stm32bs")
            .tempdir()
            .unwrap(),
        git: false,
        branch: None,
        tag: None,
    }
}

impl ProjectBuilder {
    /// builds a template with
    /// - one file `Cargo.toml` in it
    /// - one placeholder `project-name`
    pub fn init_default_template(self) -> Self {
        self.with_default_manifest().init_git()
    }

    /// creates a `Cargo.toml` manifest with a `project-name` placeholder
    pub fn with_default_manifest(self) -> Self {
        self.file(
            "stm32bs.toml",
            indoc! {r#"
                [template]
                description = "A wonderful project"
                version = ">=0.0.3"
            "#},
        ).file(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#},
        ).file(
            "build.rs",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#},
        ).file(
            "memory.x",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#},
        ).file(
            ".cargo/config.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#},
        ).file(
            "src/main.rs",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#},
        ).file(
            "ProductsList.csv", 
            indoc! {r#"
                Microcontrollers & microprocessors,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,
Part Number,General Description,Marketing Status,Package,Core,Operating Frequency (MHz),FPU,Co-Processor type,Co-Processor frequency (MHz) max,Flash Size (kB) (Prog),Dual-bank Flash,Data E2PROM (B) nom,RAM Size (kB),CCM RAM (I/D) (kByte) typ,ITCM/DTCM RAM (kB),Timers (16-bit) typ,Timers (32-bit) typ,Other timer functions,A/D Converters 12-bit,,A/D Converters 14-bit,,A/D Converters 16-bit,,D/A Converters (12-bit) typ,Comparator,Integrated op-amps,I/Os (High Current),SMPS,Display controller,Graphic accelerator,CAN (FD),CAN (2.0),I2C typ,SPI typ,I2S typ,USART typ,UART typ,Ethernet,Ethernet ports typ,I3C typ,Additional Interfaces,External Memory Interfaces,USB Type,Cryptography,Security Functions,Supply Voltage (V) min,Supply Voltage (V) max,Supply Current (µA) (@ Lowest Power) typ,Supply Current (µA) (Run Mode (per MHz)) typ,Operating Temperature (°C) min,Operating Temperature (°C) max,Output Power (dBm) min,Output Power (dBm) max,RX current (mA) typ,RX sensitivity (dBm) typ,Standby Current (µA) typ,TX current (mA) (@ 0dBm) max,Output Power (dBm) (Step) typ,Buy On Line,Number of Cores nom,,,,,,
,,,,,,,,,,,,,,,,,,Number of A/D Converters typ,Number of Channels typ,Number of A/D Converters typ,Number of Channels typ,Number of A/D Converters typ,Number of Channels typ,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,
STM32C011D6,"Mainstream Arm Cortex-M0+ MCU with 32 Kbytes of Flash memory, 6 Kbytes RAM, 48 MHz CPU, 2x USART, timers, ADC, comm. I/F, 2-3.6V",Active,WLCSP 12 1.7x1.42x0.6 P 0.35 mm,Arm Cortex-M0+,48,-,-,-,-,32,No,-,6,-,-,5,-,"IWDG, SysTick, WWDG",1,9,-,-,-,-,-,-,-,10,-,-,-,-,-,-,1,1,1,2,-,-,-,-,-,-,No,-,-,"Silicon Device Lifecycle, Software IP Protection",2,3.6,-,-,-40,85,-,-,-,-,-,-,-,Yes,-,-,-,-
STM32C011F4,"Mainstream Arm Cortex-M0+ MCU with 16 Kbytes of Flash memory, 6 Kbytes RAM, 48 MHz CPU, 2x USART, timers, ADC, comm. I/F, 2-3.6V",Active,"TSSOP-20, UFQFPN 20 3x3x0.6 mm",Arm Cortex-M0+,48,-,-,-,-,16,No,-,6,-,-,5,-,"IWDG, SysTick, WWDG",1,15,-,-,-,-,-,-,-,18,-,-,-,-,-,-,1,1,1,2,-,-,-,-,-,-,No,-,-,"Silicon Device Lifecycle, Software IP Protection",2,3.6,-,-,-40,125,-,-,-,-,-,-,-,Yes,-,-,-,-
            "#},
        ).file("pac_info.csv", 
            indoc! {r#"
Part Number,PAC,VER,Feature
STM32C011D6,stm32c0,0.1.1,stm32c0xx
STM32C011F4,-,-,-
STM32C011F6,-,-,-
STM32C011J4,-,-,-
STM32C011J6,-,-,-
STM32C031C4,-,-,-
STM32C031C6,-,-,-
STM32C031F4,-,-,-
STM32C031F6,-,-,-
STM32C031G4,-,-,-
STM32C031G6,-,-,-
STM32C031K4,-,-,-
STM32C031K6,-,-,-
STM32C051C6,-,-,-
            "#},)
    }

    pub fn file(mut self, name: &str, contents: impl AsRef<str>) -> Self {
        if self.files.iter().any(|(f, _)| f == name) {
            self.files.retain(|(f, _)| f != name);
            self.files
                .push((name.to_string(), contents.as_ref().to_string()));
        } else {
            self.files
                .push((name.to_string(), contents.as_ref().to_string()));
        }
        self
    }

    pub fn init_git(mut self) -> Self {
        self.git = true;
        self
    }

    pub fn branch(mut self, branch: &str) -> Self {
        self.branch = Some(branch.to_owned());
        self
    }

    pub fn tag(mut self, tag: &str) -> Self {
        self.tag = Some(tag.to_owned());
        self
    }

    pub fn add_submodule(
        mut self,
        destination: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        self.submodules.push((destination.into(), path.into()));
        self
    }

    /// On Git >=2.28.0 `init.defaultBranch` can be set to change the default initial branch name
    /// to something other than `master`. Calling this function after the first commit makes sure
    /// the initial branch is named `main` in all our integration tests so that they're not
    /// effected by `init.defaultBranch`.
    fn rename_branch_to_main(&self) {
        Command::new("git")
            .arg("branch")
            .arg("--move")
            .arg("main")
            .current_dir(self.root.path())
            .assert()
            .success();
    }

    fn git_commit(message: &str, current_directory: impl AsRef<std::path::Path>) {
        Command::new("git")
            .args([
                "-c",
                "user.name='Foo Bar'",
                "-c",
                "user.email='foo@bar.com'",
            ])
            .arg("commit")
            .arg("--no-gpg-sign")
            .arg("--message")
            .arg(message)
            .current_dir(current_directory)
            .assert()
            .success();
    }

    pub fn build(self) -> Project {
        let path = self.root.path();

        for (file, contents) in self.files.iter() {
            let path = path.join(file);
            let parent = path
                .parent()
                .unwrap_or_else(|| panic!("couldn't find parent dir of {path:?}"));

            fs::create_dir_all(parent)
                .unwrap_or_else(|_| panic!("couldn't create {parent:?} directory"));

            fs::File::create(&path)
                .unwrap_or_else(|_| panic!("couldn't create file {path:?}"))
                .write_all(contents.as_ref())
                .unwrap_or_else(|_| panic!("couldn't write to file {path:?}: {contents:?}"));
        }

        if self.git {
            Command::new("git")
                .arg("init")
                .current_dir(path)
                .assert()
                .success();

            if let Some(ref branch) = self.branch {
                // Create dummy content in "main" branch to aid testing

                fs::File::create(path.join("dummy.txt"))
                    .expect("Failed to create dummy")
                    .write_all(b"main dummy")
                    .expect("Couldn't write out dummy text");

                Command::new("git")
                    .arg("add")
                    .arg("dummy.txt")
                    .current_dir(path)
                    .assert()
                    .success();

                Self::git_commit("initial main commit", path);

                self.rename_branch_to_main();

                Command::new("git")
                    .arg("checkout")
                    .arg("-b")
                    .arg(branch)
                    .current_dir(path)
                    .assert()
                    .success();
            }

            Command::new("git")
                .arg("add")
                .arg("--all")
                .current_dir(path)
                .assert()
                .success();

            self.submodules.iter().for_each(|(d, m)| {
                Command::new("git")
                    .arg("-c")
                    .arg("protocol.file.allow=always")
                    .arg("submodule")
                    .arg("add")
                    .arg(m)
                    .arg(d)
                    .current_dir(path)
                    .assert()
                    .success();
            });

            Self::git_commit("initial commit", path);

            if let Some(ref tag) = self.tag {
                Command::new("git")
                    .args([
                        "-c",
                        "user.name='Foo Bar'",
                        "-c",
                        "user.email='foo@bar.com'",
                    ])
                    .arg("tag")
                    .arg("-a")
                    .arg(tag)
                    .arg("-m")
                    .arg(format!("our test tag {tag}"))
                    .current_dir(path)
                    .assert()
                    .success();

                for (file, _) in self.files.iter() {
                    let path = path.join(file);
                    fs::remove_file(&path).unwrap_or_else(|_| {
                        panic!("couldn't remove file {path:?}, after committing tag {tag}")
                    });
                }

                Command::new("git")
                    .arg("add")
                    .arg("--all")
                    .current_dir(path)
                    .assert()
                    .success();

                Self::git_commit("dummy commit after tag", path);
            }

            if self.branch.is_some() {
                Command::new("git")
                    .arg("checkout")
                    .arg("main")
                    .current_dir(path)
                    .assert()
                    .success();
            } else {
                self.rename_branch_to_main();
            }
        }

        Project { root: self.root }
    }
}
