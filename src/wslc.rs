use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub created: String,
    pub status: String,
    pub ports: String,
    pub is_running: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageInfo {
    pub repository: String,
    pub tag: String,
    pub image_id: String,
    pub created: String,
    pub size: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VolumeInfo {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInfo {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContainerDetail {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub created: String,
    pub started: String,
    pub ports: Vec<PortMapping>,
    pub env: Vec<String>,
    pub mounts: Vec<MountInfo>,
    pub cmd: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortMapping {
    pub host_port: String,
    pub container_port: String,
    pub protocol: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MountInfo {
    pub source: String,
    pub destination: String,
    pub mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub wslc_version: String,
    pub container_count: usize,
    pub image_count: usize,
    pub running_containers: usize,
    pub volume_count: usize,
    pub network_count: usize,
}

pub struct WslcClient;

impl WslcClient {
    pub fn new() -> Self {
        Self
    }

    fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
        let output = Command::new(cmd)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if stderr.is_empty() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(stderr)
            }
        }
    }

    pub fn get_version(&self) -> Result<String, String> {
        let output = Self::run_command("wslc", &["--version"])?;
        Ok(output.trim().to_string())
    }

    pub fn list_containers(&self, all: bool) -> Result<Vec<ContainerInfo>, String> {
        let mut args = vec!["container", "ps"];
        if all {
            args.push("-a");
        }

        let output = Self::run_command("wslc", &args)?;
        let mut containers = Vec::new();
        let lines: Vec<&str> = output.lines().collect();
        
        if lines.is_empty() {
            return Ok(containers);
        }
        
        let header = &lines[0];
        let id_pos = header.find("CONTAINER ID").unwrap_or(0);
        let names_pos = header.find("NAMES").unwrap_or(12);
        let image_pos = header.find("IMAGE").unwrap_or(24);
        let created_pos = header.find("CREATED").unwrap_or(45);
        let status_pos = header.find("STATUS").unwrap_or(60);
        let ports_pos = header.find("PORTS").unwrap_or(78);
        
        for i in 1..lines.len() {
            let line = lines[i];
            if line.trim().is_empty() || line.starts_with("CONTAINER") {
                continue;
            }
            
            if line.len() >= names_pos {
                let id = line[id_pos..names_pos].trim().to_string();
                let name = if line.len() >= image_pos {
                    line[names_pos..image_pos].trim().to_string()
                } else {
                    line[names_pos..].trim().to_string()
                };
                let image = if line.len() >= created_pos {
                    line[image_pos..created_pos].trim().to_string()
                } else {
                    line[image_pos..].trim().to_string()
                };
                let created = if line.len() >= status_pos {
                    line[created_pos..status_pos].trim().to_string()
                } else {
                    String::from("-")
                };
                let status_str = if line.len() >= ports_pos {
                    line[status_pos..ports_pos].trim().to_string()
                } else {
                    line[status_pos..].trim().to_string()
                };
                let ports = if line.len() > ports_pos {
                    line[ports_pos..].trim().to_string()
                } else {
                    String::from("-")
                };
                
                let is_running = status_str.to_lowercase().contains("running") || status_str.contains("Up");
                
                containers.push(ContainerInfo {
                    id,
                    name,
                    image,
                    created,
                    status: status_str,
                    ports,
                    is_running,
                });
            }
        }

        Ok(containers)
    }

    pub fn list_images(&self) -> Result<Vec<ImageInfo>, String> {
        let output = Self::run_command("wslc", &["image", "ls"])?;
        let mut images = Vec::new();
        let lines: Vec<&str> = output.lines().collect();
        
        if lines.is_empty() {
            return Ok(images);
        }
        
        // Parse header to find column positions
        let header = &lines[0];
        let tag_pos = header.find("TAG").unwrap_or(40);
        let id_pos = header.find("IMAGE ID").unwrap_or(49);
        let created_pos = header.find("CREATED").unwrap_or(62);
        let size_pos = header.find("SIZE").unwrap_or(77);
        
        for i in 1..lines.len() {
            let line = lines[i];
            if line.trim().is_empty() || line.starts_with("REPOSITORY") {
                continue;
            }
            
            if line.len() >= size_pos {
                let repo = line[..tag_pos].trim().to_string();
                let tag = line[tag_pos..id_pos].trim().to_string();
                let image_id = line[id_pos..created_pos].trim().to_string();
                let created = line[created_pos..size_pos].trim().to_string();
                let size = line[size_pos..].trim().to_string();
                
                images.push(ImageInfo {
                    repository: repo,
                    tag,
                    image_id,
                    created,
                    size,
                });
            }
        }

        Ok(images)
    }

    pub fn list_volumes(&self) -> Result<Vec<VolumeInfo>, String> {
        let output = Self::run_command("wslc", &["volume", "ls"])?;
        let mut volumes = Vec::new();
        let lines: Vec<&str> = output.lines().collect();
        
        if lines.is_empty() {
            return Ok(volumes);
        }
        
        let header = &lines[0];
        let name_pos = header.find("VOLUME NAME").unwrap_or(header.find("NAME").unwrap_or(0));
        let driver_pos = header.find("DRIVER").unwrap_or(name_pos + 20);
        let mountpoint_pos = header.find("MOUNTPOINT").unwrap_or(driver_pos + 10);
        
        for i in 1..lines.len() {
            let line = lines[i];
            if line.trim().is_empty() || line.starts_with("VOLUME") || line.starts_with("NAME") {
                continue;
            }
            
            if line.len() >= name_pos {
                let name = line[name_pos..driver_pos].trim().to_string();
                let driver = if line.len() >= mountpoint_pos {
                    line[driver_pos..mountpoint_pos].trim().to_string()
                } else {
                    "local".to_string()
                };
                let mountpoint = if line.len() > mountpoint_pos {
                    line[mountpoint_pos..].trim().to_string()
                } else {
                    "-".to_string()
                };
                
                volumes.push(VolumeInfo {
                    name,
                    driver,
                    mountpoint,
                    created: "-".to_string(),
                });
            }
        }

        Ok(volumes)
    }

    pub fn list_networks(&self) -> Result<Vec<NetworkInfo>, String> {
        let output = Self::run_command("wslc", &["network", "ls"])?;
        let mut networks = Vec::new();
        let lines: Vec<&str> = output.lines().collect();
        
        if lines.is_empty() {
            return Ok(networks);
        }
        
        let header = &lines[0];
        let id_pos = header.find("NETWORK ID").unwrap_or(header.find("ID").unwrap_or(0));
        let name_pos = header.find("NAME").unwrap_or(id_pos + 12);
        let driver_pos = header.find("DRIVER").unwrap_or(name_pos + 20);
        let scope_pos = header.find("SCOPE").unwrap_or(driver_pos + 10);
        
        for i in 1..lines.len() {
            let line = lines[i];
            if line.trim().is_empty() || line.starts_with("NETWORK") || line.starts_with("ID") {
                continue;
            }
            
            if line.len() >= name_pos {
                let id = line[id_pos..name_pos].trim().to_string();
                let name = if line.len() >= driver_pos {
                    line[name_pos..driver_pos].trim().to_string()
                } else {
                    line[name_pos..].trim().to_string()
                };
                let driver = if line.len() >= scope_pos {
                    line[driver_pos..scope_pos].trim().to_string()
                } else {
                    "bridge".to_string()
                };
                let scope = if line.len() > scope_pos {
                    line[scope_pos..].trim().to_string()
                } else {
                    "local".to_string()
                };
                
                networks.push(NetworkInfo {
                    id,
                    name,
                    driver,
                    scope,
                });
            }
        }

        Ok(networks)
    }

    pub fn inspect_container(&self, name: &str) -> Result<ContainerDetail, String> {
        let output = Self::run_command("wslc", &["inspect", name])?;
        
        if let Ok(arr) = serde_json::from_str::<serde_json::Value>(&output) {
            // wslc inspect returns an array, take the first element
            let json = if arr.is_array() {
                arr.as_array().and_then(|a| a.first()).cloned().unwrap_or(arr)
            } else {
                arr
            };
            
            let name_val = json["Name"].as_str().unwrap_or(name).trim_start_matches('/').to_string();
            let id = json["Id"].as_str().unwrap_or("-").to_string();
            // Image can be at top level or under Config
            let image = json["Image"].as_str()
                .or_else(|| json["Config"]["Image"].as_str())
                .unwrap_or("-")
                .to_string();
            
            let state = &json["State"];
            let status = if state["Running"].as_bool().unwrap_or(false) {
                "running".to_string()
            } else {
                state["Status"].as_str().unwrap_or("unknown").to_string()
            };
            
            let created = json["Created"].as_str().unwrap_or("-").to_string();
            let started = state["StartedAt"].as_str().unwrap_or("-").to_string();
            
            let cmd = json["Config"]["Cmd"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            
            let env = json["Config"]["Env"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            
            let empty_vec = vec![];
            // Ports can be at top level or under NetworkSettings
            let ports_obj = json["Ports"].as_object()
                .or_else(|| json["NetworkSettings"]["Ports"].as_object());
            
            let ports = ports_obj
                .map(|obj| {
                    obj.iter().flat_map(|(container_port, bindings)| {
                        let bindings_arr = bindings.as_array().unwrap_or(&empty_vec);
                        bindings_arr.iter().filter_map(|b| {
                            Some(PortMapping {
                                host_port: b["HostPort"].as_str().unwrap_or("-").to_string(),
                                container_port: container_port.split('/').next().unwrap_or("-").to_string(),
                                protocol: container_port.split('/').last().unwrap_or("tcp").to_string(),
                            })
                        }).collect::<Vec<_>>()
                    }).collect()
                })
                .unwrap_or_default();
            
            let mounts = json["Mounts"]
                .as_array()
                .map(|arr| {
                    arr.iter().map(|m| MountInfo {
                        source: m["Source"].as_str().unwrap_or("-").to_string(),
                        destination: m["Destination"].as_str().unwrap_or("-").to_string(),
                        mode: m["Mode"].as_str().unwrap_or("-").to_string(),
                    }).collect()
                })
                .unwrap_or_default();
            
            Ok(ContainerDetail {
                id,
                name: name_val,
                image,
                status,
                created,
                started,
                ports,
                env,
                mounts,
                cmd,
            })
        } else {
            Err("Failed to parse container inspect output".to_string())
        }
    }

    pub fn run_container(
        &self,
        image: &str,
        name: Option<&str>,
        ports: Option<&str>,
        volumes: Option<&str>,
        envs: Option<Vec<String>>,
        detached: bool,
        command: Option<&str>,
    ) -> Result<String, String> {
        let mut args = vec!["run".to_string()];

        if detached {
            args.push("-d".to_string());
        } else {
            args.push("--rm".to_string());
        }

        if let Some(n) = name {
            args.push("--name".to_string());
            args.push(n.to_string());
        }

        if let Some(p) = ports {
            args.push("-p".to_string());
            args.push(p.to_string());
        }

        if let Some(v) = volumes {
            args.push("-v".to_string());
            args.push(v.to_string());
        }

        if let Some(env) = envs {
            for e in env {
                args.push("-e".to_string());
                args.push(e);
            }
        }

        args.push(image.to_string());

        if let Some(cmd) = command {
            args.push("bash".to_string());
            args.push("-c".to_string());
            args.push(cmd.to_string());
        }

        let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        Self::run_command("wslc", &str_args)
    }

    pub fn stop_container(&self, name: &str) -> Result<String, String> {
        Self::run_command("wslc", &["container", "stop", name])
    }

    pub fn start_container(&self, name: &str) -> Result<String, String> {
        Self::run_command("wslc", &["container", "start", name])
    }

    pub fn remove_container(&self, name: &str, force: bool) -> Result<String, String> {
        let mut args = vec!["container", "rm"];
        if force {
            args.push("-f");
        }
        args.push(name);
        Self::run_command("wslc", &args)
    }

    pub fn restart_container(&self, name: &str) -> Result<String, String> {
        self.stop_container(name)?;
        // Wait a bit for container to stop
        std::thread::sleep(std::time::Duration::from_millis(500));
        self.start_container(name)
    }

    pub fn get_logs(&self, name: &str, lines: Option<u32>, follow: bool) -> Result<String, String> {
        let mut args = vec!["container".to_string(), "logs".to_string()];
        if let Some(n) = lines {
            args.push("--tail".to_string());
            args.push(n.to_string());
        }
        if follow {
            args.push("-f".to_string());
        }
        args.push(name.to_string());
        let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        Self::run_command("wslc", &str_args)
    }

    pub fn exec_command(&self, name: &str, command: &str) -> Result<String, String> {
        Self::run_command("wslc", &["exec", name, "bash", "-c", command])
    }

    pub fn pull_image(&self, image: &str) -> Result<String, String> {
        Self::run_command("wslc", &["pull", image])
    }

    pub fn remove_image(&self, image: &str, force: bool) -> Result<String, String> {
        let mut args = vec!["rmi"];
        if force {
            args.push("-f");
        }
        args.push(image);
        Self::run_command("wslc", &args)
    }

    pub fn get_container_stats(&self, name: &str) -> Result<String, String> {
        Self::run_command("wslc", &["stats", name, "--no-stream"])
    }

    pub fn get_container_top(&self, name: &str) -> Result<String, String> {
        Self::run_command("wslc", &["exec", name, "ps", "aux"])
    }

    pub fn export_container(&self, name: &str, output: &str) -> Result<String, String> {
        Self::run_command("wslc", &["export", name, "-o", output])
    }

    #[allow(dead_code)]
    pub fn import_image(&self, name: &str, file: &str, cmd: Option<&str>) -> Result<String, String> {
        let mut args = vec!["import", name, file];
        if let Some(c) = cmd {
            args.push("-c");
            args.push(c);
        }
        Self::run_command("wslc", &args)
    }

    pub fn commit_container(&self, name: &str, image: &str) -> Result<String, String> {
        Self::run_command("wslc", &["commit", name, image])
    }

    #[allow(dead_code)]
    pub fn copy_from_container(&self, name: &str, src: &str, dest: &str) -> Result<String, String> {
        Self::run_command("wslc", &["cp", &format!("{}:{}", name, src), dest])
    }

    #[allow(dead_code)]
    pub fn copy_to_container(&self, name: &str, src: &str, dest: &str) -> Result<String, String> {
        Self::run_command("wslc", &["cp", src, &format!("{}:{}", name, dest)])
    }

    pub fn rename_container(&self, old_name: &str, new_name: &str) -> Result<String, String> {
        Self::run_command("wslc", &["container", "rename", old_name, new_name])
    }

    #[allow(dead_code)]
    pub fn container_exists(&self, name: &str) -> bool {
        self.inspect_container(name).is_ok()
    }

    #[allow(dead_code)]
    pub fn image_exists(&self, name: &str) -> bool {
        self.list_images()
            .map(|imgs| imgs.iter().any(|i| i.repository == name || format!("{}:{}", i.repository, i.tag) == name))
            .unwrap_or(false)
    }

    pub fn create_volume(&self, name: &str) -> Result<String, String> {
        Self::run_command("wslc", &["volume", "create", name])
    }

    pub fn remove_volume(&self, name: &str) -> Result<String, String> {
        Self::run_command("wslc", &["volume", "rm", name])
    }

    pub fn create_network(&self, name: &str, driver: Option<&str>) -> Result<String, String> {
        let mut args = vec!["network", "create"];
        if let Some(d) = driver {
            args.push("--driver");
            args.push(d);
        }
        args.push(name);
        Self::run_command("wslc", &args)
    }

    pub fn remove_network(&self, name: &str) -> Result<String, String> {
        Self::run_command("wslc", &["network", "rm", name])
    }

    pub fn get_system_info(&self) -> Result<SystemInfo, String> {
        let version = self.get_version().unwrap_or_default();
        let containers = self.list_containers(true).unwrap_or_default();
        let images = self.list_images().unwrap_or_default();
        let volumes = self.list_volumes().unwrap_or_default();
        let networks = self.list_networks().unwrap_or_default();

        let running = containers.iter().filter(|c| c.is_running).count();

        Ok(SystemInfo {
            wslc_version: version,
            container_count: containers.len(),
            image_count: images.len(),
            running_containers: running,
            volume_count: volumes.len(),
            network_count: networks.len(),
        })
    }
}
