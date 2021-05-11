use anyhow::{bail, Result};
use downcast_rs::{impl_downcast, Downcast};
use std::io::{self, Write};
use std::ops::Deref;
use std::path::Path;
use std::process::{Command, Output};

struct Test {
    // test_name: &'static str, TODO
    test_name: String,
}

struct TestRunner {
    test: Box<dyn TestSteps>,
}

enum TestOutput {
    RetCode(i32),
    Output(Output),
}

impl TestOutput {
    pub fn is_success(&self) -> bool {
        match self {
            TestOutput::RetCode(ret) => ret == &0i32,
            TestOutput::Output(output) => output.status.success(),
        }
    }
}

trait TestFlow {
    fn run(&self) -> Result<TestOutput>;
}

impl TestFlow for TestRunner {
    fn run(&self) -> Result<TestOutput> {
        println!("\nrunning the test {}", self.test.get_name());
        let mut stdout = "".to_string();
        let mut stderr = "".to_string();
        // TODO:
        // let test = self
        //     .test
        //     .downcast_ref::<Test>()
        //     .context("Failed to downcast to Test")?;

        let output = self.test.pre_test()?;
        if let TestOutput::Output(o) = &output {
            stdout = o.stdout.iter().map(|&c| c as char).collect();
            stderr = o.stderr.iter().map(|&c| c as char).collect();
        }
        if !output.is_success() {
            io::stdout().write_all(stdout.as_bytes())?;
            io::stdout().write_all(stderr.as_bytes())?;
            bail!(
                "failed to prepare the test environment for {}",
                self.test.get_name()
            );
        }

        let output = self.test.do_test(stdout.split_whitespace().collect::<Vec<_>>())?;
        if let TestOutput::Output(o) = &output {
            stdout = o.stdout.iter().map(|&c| c as char).collect();
            stderr = o.stderr.iter().map(|&c| c as char).collect();
        }
        if !output.is_success() {
            io::stdout().write_all(stdout.as_bytes())?;
            io::stdout().write_all(stderr.as_bytes())?;
            bail!("failed to do the test: '{}'", self.test.get_name());
        }

        let output = self.test.post_test(stdout.split_whitespace().collect::<Vec<_>>())?;
        if let TestOutput::Output(o) = &output {
            stdout = o.stdout.iter().map(|&c| c as char).collect();
            stderr = o.stderr.iter().map(|&c| c as char).collect();
        }
        if !output.is_success() {
            io::stdout().write_all(stdout.as_bytes())?;
            io::stdout().write_all(stderr.as_bytes())?;
            bail!("failed to post process the test '{}'", self.test.get_name());
        }

        println!("Test {} passes.", self.test.get_name());
        Ok(output)
    }
}

trait TestSteps: Downcast 
{
    fn get_name(&self) -> &str;

    fn pre_test(&self) -> Result<TestOutput> {
        Ok(TestOutput::RetCode(0))
    }

    fn do_test(&self, args: Vec<&str>) -> Result<TestOutput>;

    fn post_test(&self, _args: Vec<&str>) -> Result<TestOutput> {
        Ok(TestOutput::RetCode(0))
    }
}
impl_downcast!(TestSteps);

// make Test compatible with trait TestSteps to downcast
impl TestSteps for Test {
    fn get_name(&self) -> &str {
        self.test_name.as_str()
    }

    fn do_test(&self, _: Vec<&str>) -> Result<TestOutput> {
        Ok(TestOutput::RetCode(0))
    }
}

#[repr(transparent)]
struct BashTest(Test);

impl Deref for BashTest {
    type Target = Test;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TestSteps for BashTest {
    fn get_name(&self) -> &str {
        self.test_name.as_str()
    }

    fn pre_test(&self) -> Result<TestOutput> {
        let output = Command::new("bash")
            .arg(Path::new("scripts").join("tests").join("install.bash"))
            .output()?;
        Ok(TestOutput::Output(output))
    }

    fn do_test(&self, args: Vec<&str>) -> Result<TestOutput> {
        let output = Command::new("bash")
            .arg(Path::new("scripts").join("tests").join("test.bash"))
            .args(args)
            .output()?;
        Ok(TestOutput::Output(output))
    }

    fn post_test(&self, args: Vec<&str>) -> Result<TestOutput> {
        let output = Command::new("bash")
            .arg(Path::new("scripts").join("tests").join("uninstall.bash"))
            .args(args)
            .output()?;
        Ok(TestOutput::Output(output))
    }
}

#[repr(transparent)]
struct FishTest(Test);

impl Deref for FishTest {
    type Target = Test;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TestSteps for FishTest {
    fn get_name(&self) -> &str {
        self.test_name.as_str()
    }

    fn pre_test(&self) -> Result<TestOutput> {
        let output = Command::new("echo")
            .arg(Path::new("scripts").join("tests").join("install.fish"))
            .output()?;
        Ok(TestOutput::Output(output))
    }

    fn do_test(&self, args: Vec<&str>) -> Result<TestOutput> {
        let output = Command::new("echo")
            .arg(Path::new("scripts").join("tests").join("test.fish"))
            .args(args)
            .output()?;
        Ok(TestOutput::Output(output))
    }

    fn post_test(&self, args: Vec<&str>) -> Result<TestOutput> {
        let output = Command::new("echo")
            .arg(Path::new("scripts").join("tests").join("uninstall.fish"))
            .args(args)
            .output()?;
        Ok(TestOutput::Output(output))
    }
}

#[repr(transparent)]
struct ZshTest(Test);

impl Deref for ZshTest {
    type Target = Test;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TestSteps for ZshTest {
    fn get_name(&self) -> &str {
        self.test_name.as_str()
    }

    fn pre_test(&self) -> Result<TestOutput> {
        let output = Command::new("echo")
            .arg(Path::new("scripts").join("tests").join("install.zsh"))
            .output()?;
        Ok(TestOutput::Output(output))
    }

    fn do_test(&self, args: Vec<&str>) -> Result<TestOutput> {
        let output = Command::new("echo")
            .arg(Path::new("scripts").join("tests").join("test.zsh"))
            .args(args)
            .output()?;
        Ok(TestOutput::Output(output))
    }

    fn post_test(&self, args: Vec<&str>) -> Result<TestOutput> {
        let output = Command::new("echo")
            .arg(Path::new("scripts").join("tests").join("uninstall.zsh"))
            .args(args)
            .output()?;
        Ok(TestOutput::Output(output))
    }
}

#[repr(transparent)]
struct TcshTest(Test);

impl Deref for TcshTest {
    type Target = Test;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TestSteps for TcshTest {
    fn get_name(&self) -> &str {
        self.test_name.as_str()
    }

    fn pre_test(&self) -> Result<TestOutput> {
        let output = Command::new("echo")
            .arg(Path::new("scripts").join("tests").join("install.tcsh"))
            .output()?;
        Ok(TestOutput::Output(output))
    }

    fn do_test(&self, args: Vec<&str>) -> Result<TestOutput> {
        let output = Command::new("echo")
            .arg(Path::new("scripts").join("tests").join("test.tcsh"))
            .args(args)
            .output()?;
        Ok(TestOutput::Output(output))
    }

    fn post_test(&self, args: Vec<&str>) -> Result<TestOutput> {
        let output = Command::new("echo")
            .arg(Path::new("scripts").join("tests").join("uninstall.tcsh"))
            .args(args)
            .output()?;
        Ok(TestOutput::Output(output))
    }
}

#[cfg(target_family = "unix")]
fn run_tests_for_unix() -> Result<()> {
    let bash_test = BashTest(Test {
        test_name: "bast test".to_owned(),
    });
    let fish_test = FishTest(Test {
        test_name: "fish test".to_owned(),
    });
    let zsh_test = ZshTest(Test {
        test_name: "zsh test".to_owned(),
    });
    let tcsh_test = TcshTest(Test {
        test_name: "tcsh test".to_owned(),
    });

    let tests: Vec<Box<dyn TestSteps>> = vec![
        Box::new(bash_test),
        Box::new(fish_test),
        Box::new(zsh_test),
        Box::new(tcsh_test),
    ];

    for test in tests.into_iter() {
        TestRunner { test }.run()?;
    }
    Ok(())
}

#[cfg(target_family = "windows")]
fn run_tests_for_windows() -> Result<()> {
    Ok(())
}

#[test]
/// run the tests sequentially as they are sharing one database.
fn run_all_tests() -> Result<()> {
    #[cfg(target_family = "unix")]
    run_tests_for_unix()?;

    #[cfg(target_family = "windows")]
    run_tests_for_windows()?;

    println!("All tests pass.");

    Ok(())
}
