use crate::error::{Result, ZecDevError};
use std::process::Command;

#[derive(Clone)]
pub struct DockerCompose {
    project_dir: String,
    use_prebuilt: bool,
}

impl DockerCompose {
    pub fn new() -> Result<Self> {
        Self::with_options(true) // Default to prebuilt
    }

    pub fn with_options(use_prebuilt: bool) -> Result<Self> {
        // Get project root (go up from cli/ directory)
        let current_dir = std::env::current_dir()?;
        let project_dir = if current_dir.ends_with("cli") {
            current_dir.parent().unwrap().to_path_buf()
        } else {
            current_dir
        };

        Ok(Self {
            project_dir: project_dir.to_string_lossy().to_string(),
            use_prebuilt,
        })
    }

    fn compose_file(&self) -> &str {
        if self.use_prebuilt {
            "docker-compose.prebuilt.yml"
        } else {
            "docker-compose.yml"
        }
    }

    pub fn up(&self, services: &[&str]) -> Result<()> {
        let mut cmd = Command::new("docker");
        cmd.arg("compose")
            .arg("-f")
            .arg(self.compose_file())
            .arg("up")
            .arg("-d")
            .current_dir(&self.project_dir);

        for service in services {
            cmd.arg(service);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ZecDevError::Docker(error.to_string()));
        }

        Ok(())
    }

    pub fn up_with_profile(&self, profile: &str) -> Result<()> {
        if self.use_prebuilt {
            // PULL IMAGES from GHCR (force amd64 for pre-built images)
            let pull_output = Command::new("docker")
                .arg("compose")
                .arg("-f")
                .arg(self.compose_file())
                .arg("--profile")
                .arg(profile)
                .arg("pull")
                .env("DOCKER_DEFAULT_PLATFORM", "linux/amd64")
                .current_dir(&self.project_dir)
                .output()?;

            if !pull_output.status.success() {
                let error = String::from_utf8_lossy(&pull_output.stderr);
                return Err(ZecDevError::Docker(format!("Image pull failed: {}", error)));
            }
        } else {
            // BUILD IMAGES locally
            let build_output = Command::new("docker")
                .arg("compose")
                .arg("-f")
                .arg(self.compose_file())
                .arg("--profile")
                .arg(profile)
                .arg("build")
                .current_dir(&self.project_dir)
                .output()?;

            if !build_output.status.success() {
                let error = String::from_utf8_lossy(&build_output.stderr);
                return Err(ZecDevError::Docker(format!(
                    "Image build failed: {}",
                    error
                )));
            }
        }

        // START SERVICES
        let output = Command::new("docker")
            .arg("compose")
            .arg("-f")
            .arg(self.compose_file())
            .arg("--profile")
            .arg(profile)
            .arg("up")
            .arg("-d")
            .current_dir(&self.project_dir)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ZecDevError::Docker(error.to_string()));
        }

        Ok(())
    }

    pub fn down(&self, volumes: bool) -> Result<()> {
        let mut cmd = Command::new("docker");
        cmd.arg("compose")
            .arg("-f")
            .arg(self.compose_file())
            .arg("down")
            .current_dir(&self.project_dir);

        if volumes {
            cmd.arg("-v");
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ZecDevError::Docker(error.to_string()));
        }

        Ok(())
    }

    pub fn ps(&self) -> Result<Vec<String>> {
        let output = Command::new("docker")
            .arg("compose")
            .arg("-f")
            .arg(self.compose_file())
            .arg("ps")
            .arg("--format")
            .arg("table")
            .current_dir(&self.project_dir)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ZecDevError::Docker(error.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<String> = stdout
            .lines()
            .skip(1) // Skip header
            .map(|l| l.to_string())
            .collect();

        Ok(lines)
    }

    pub fn logs(&self, service: &str, tail: usize) -> Result<Vec<String>> {
        let output = Command::new("docker")
            .arg("compose")
            .arg("-f")
            .arg(self.compose_file())
            .arg("logs")
            .arg("--tail")
            .arg(tail.to_string())
            .arg(service)
            .current_dir(&self.project_dir)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ZecDevError::Docker(error.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<String> = stdout.lines().map(|l| l.to_string()).collect();

        Ok(lines)
    }

    pub fn exec(&self, service: &str, command: &[&str]) -> Result<String> {
        let mut cmd = Command::new("docker");
        cmd.arg("compose")
            .arg("-f")
            .arg(self.compose_file())
            .arg("exec")
            .arg("-T") // Non-interactive
            .arg(service)
            .current_dir(&self.project_dir);

        for arg in command {
            cmd.arg(arg);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ZecDevError::Docker(error.to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn is_running(&self) -> bool {
        Command::new("docker")
            .arg("compose")
            .arg("-f")
            .arg(self.compose_file())
            .arg("ps")
            .arg("-q")
            .current_dir(&self.project_dir)
            .output()
            .map(|output| !output.stdout.is_empty())
            .unwrap_or(false)
    }
}
