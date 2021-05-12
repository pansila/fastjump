use anyhow::{bail, Result};
use downcast_rs::{impl_downcast, Downcast};
use std::io::{self, Write};
use std::ops::Deref;
use std::path::Path;
use std::process::{Command, Output};

struct Test {
    // test_name: &'static str, TODO
    test_name: String,
    ignore: bool,
}

struct TestRunner {
    test: Box<dyn TestSteps>,
}

type TestOutput = Option<Output>;

trait TestFlow {
    fn run(&self) -> Result<TestOutput>;
}

fn extract_output(output: &Output) -> Result<(String, String, bool)> {
    let stdout: String = output.stdout.iter().map(|&c| c as char).collect();
    let stderr: String = output.stderr.iter().map(|&c| c as char).collect();
    let success = output.status.success();
    if !success {
        println!("stdout:");
        io::stdout().write_all(stdout.as_bytes())?;
        eprintln!("stderr:");
        io::stdout().write_all(stderr.as_bytes())?;
    }
    Ok((stdout, stderr, success))
}

impl TestFlow for TestRunner {
    fn run(&self) -> Result<TestOutput> {
        if self.test.is_ignore() {
            println!("\nignoring the test '{}'", self.test.get_name());
            return Ok(None);
        }
        println!("\nrunning the test '{}'", self.test.get_name());
        let mut args: Vec<String> = Vec::new();
        // TODO:
        // let test = self
        //     .test
        //     .downcast_ref::<Test>()
        //     .context("Failed to downcast to Test")?;

        let output = self.test.pre_test()?;
        if let Some(o) = &output {
            let (stdout, _, success) = extract_output(o)?;
            if !success {
                bail!(
                    "failed to prepare the test environment for '{}'",
                    self.test.get_name()
                );
            }
            args = stdout.split_whitespace().map(|x| x.to_string()).collect();
        }

        let output = self.test.do_test(&args)?;
        if let Some(o) = &output {
            let (stdout, _, success) = extract_output(o)?;
            if !success {
                bail!(
                    "failed to do the test: '{}'",
                    self.test.get_name()
                );
            }
            args = stdout.split_whitespace().map(|x| x.to_string()).collect();
        }

        let output = self.test.post_test(&args)?;
        if let Some(o) = &output {
            let (_, _, success) = extract_output(o)?;
            if !success {
                bail!(
                    "failed to post process the test: '{}'",
                    self.test.get_name()
                );
            }
        }

        println!("test '{}' passes", self.test.get_name());
        Ok(output)
    }
}

trait TestSteps: Downcast 
{
    fn get_name(&self) -> &str;
    fn is_ignore(&self) -> bool;

    fn pre_test(&self) -> Result<TestOutput> {
        Ok(None)
    }

    fn do_test(&self, args: &Vec<String>) -> Result<TestOutput>;

    fn post_test(&self, _args: &Vec<String>) -> Result<TestOutput> {
        Ok(None)
    }
}
impl_downcast!(TestSteps);

// make Test compatible with trait TestSteps to downcast
impl TestSteps for Test {
    fn get_name(&self) -> &str {
        self.test_name.as_str()
    }
    fn is_ignore(&self) -> bool {
        self.ignore
    }

    fn do_test(&self, _: &Vec<String>) -> Result<TestOutput> {
        Ok(None)
    }
}

/// producea new type and its methods of trait TestSteps, eg.
///
/// ```
/// struct BashTest(Test);
/// impl Deref for BashTest {}
/// impl TestSteps for BashTest {}
/// ```
macro_rules! expand_to_test {
    ($shell: expr, $pre_test_file: expr, $do_test_file: expr, $post_test_file: expr) => {
        paste::item! {
            #[repr(transparent)]
            struct [< Test $shell >] (Test);

            impl Deref for [< Test $shell >] {
                type Target = Test;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl TestSteps for [< Test $shell >] {
                fn get_name(&self) -> &str {
                    self.test_name.as_str()
                }
                fn is_ignore(&self) -> bool {
                    self.ignore
                }
        
                fn pre_test(&self) -> Result<TestOutput> {
                    let output = Command::new($shell)
                        .arg($pre_test_file)
                        .output()?;
                    Ok(Some(output))
                }
        
                fn do_test(&self, args: &Vec<String>) -> Result<TestOutput> {
                    let output = Command::new($shell)
                        .arg($do_test_file)
                        .args(args)
                        .output()?;
                    Ok(Some(output))
                }
        
                fn post_test(&self, args: &Vec<String>) -> Result<TestOutput> {
                    let output = Command::new($shell)
                        .arg($post_test_file)
                        .args(args)
                        .output()?;
                    Ok(Some(output))
                }
            }
        }
    };
}

expand_to_test!(
    "bash",
    Path::new("scripts").join("tests").join("install.bash"),
    Path::new("scripts").join("tests").join("test.bash"),
    Path::new("scripts").join("tests").join("uninstall.bash")
);

expand_to_test!(
    "fish",
    Path::new("scripts").join("tests").join("install.fish"),
    Path::new("scripts").join("tests").join("test.fish"),
    Path::new("scripts").join("tests").join("uninstall.fish")
);

expand_to_test!(
    "zsh",
    Path::new("scripts").join("tests").join("install.zsh"),
    Path::new("scripts").join("tests").join("test.zsh"),
    Path::new("scripts").join("tests").join("uninstall.zsh")
);

expand_to_test!(
    "tcsh",
    Path::new("scripts").join("tests").join("install.tcsh"),
    Path::new("scripts").join("tests").join("test.tcsh"),
    Path::new("scripts").join("tests").join("uninstall.tcsh")
);

#[cfg(target_family = "unix")]
fn run_tests_for_unix() -> Result<()> {
    let bash_test = Testbash(Test {
        test_name: "bash test".to_owned(),
        ignore: false,
    });
    let fish_test = Testfish(Test {
        test_name: "fish test".to_owned(),
        ignore: true,
    });
    let zsh_test = Testzsh(Test {
        test_name: "zsh test".to_owned(),
        ignore: true,
    });
    let tcsh_test = Testtcsh(Test {
        test_name: "tcsh test".to_owned(),
        ignore: true,
    });

    let tests: Vec<Box<dyn TestSteps>> = vec![
        Box::new(bash_test),
        Box::new(fish_test),
        Box::new(zsh_test),
        Box::new(tcsh_test),
    ];
    let len = tests.len();

    for test in tests.into_iter() {
        TestRunner { test }.run()?;
    }

    println!("All {} tests pass.", len);
    Ok(())
}

#[cfg(target_family = "windows")]
fn run_tests_for_windows() -> Result<()> {
    Ok(())
}

#[test]
/// run the tests sequentially as they are sharing one database.
fn integration_tests() -> Result<()> {
    #[cfg(target_family = "unix")]
    run_tests_for_unix()?;

    #[cfg(target_family = "windows")]
    run_tests_for_windows()?;

    Ok(())
}
