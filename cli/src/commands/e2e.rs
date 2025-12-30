use crate::error::{Result, ZecDevError};
use colored::*;
use std::process::Command;

pub async fn execute() -> Result<()> {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "  ZecKit - Golden E2E Flow Tests".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();

    // Get project root
    let current_dir = std::env::current_dir()?;
    let project_dir = if current_dir.ends_with("cli") {
        current_dir.parent().unwrap().to_path_buf()
    } else {
        current_dir
    };

    let e2e_script = project_dir.join("tests/e2e/golden-flow.sh");

    if !e2e_script.exists() {
        return Err(ZecDevError::Config(
            "E2E test script not found at tests/e2e/golden-flow.sh".into(),
        ));
    }

    println!("Running golden E2E flow tests...");
    println!();

    let output = Command::new("bash")
        .arg(&e2e_script)
        .current_dir(&project_dir)
        .status()?;

    if output.success() {
        println!();
        println!("{}", "✓ E2E tests completed successfully!".green().bold());
        Ok(())
    } else {
        Err(ZecDevError::HealthCheck("E2E tests failed".into()))
    }
}
