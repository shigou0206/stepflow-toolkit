use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, LogsOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions, WaitContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::CreateImageOptions;
use bollard::models::{ContainerSummary, HostConfig, Mount, MountTypeEnum, PortBinding};
use bollard::Docker;
use chrono::Utc;
use futures::stream::StreamExt;
use stepflow_database::SqliteDatabase;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::errors::*;
use crate::sandbox::ContainerManager;
use crate::types::*;

/// Docker 容器管理器配置
#[derive(Debug, Clone)]
pub struct ContainerManagerConfig {
    pub default_image: String,
    pub registry_url: String,
    pub registry_credentials: Option<RegistryCredentials>,
    pub container_timeout: Duration,
    pub enable_auto_cleanup: bool,
    pub max_containers: usize,
    pub default_network: String,
    pub default_cpu_limit: Option<f64>,
    pub default_memory_limit: Option<usize>,
}

impl Default for ContainerManagerConfig {
    fn default() -> Self {
        Self {
            default_image: "alpine:latest".to_string(),
            registry_url: "docker.io".to_string(),
            registry_credentials: None,
            container_timeout: Duration::from_secs(300),
            enable_auto_cleanup: true,
            max_containers: 1000,
            default_network: "bridge".to_string(),
            default_cpu_limit: Some(1.0),
            default_memory_limit: Some(512 * 1024 * 1024), // 512MB
        }
    }
}

/// 注册表凭据
#[derive(Debug, Clone)]
pub struct RegistryCredentials {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

/// Docker 容器管理器实现
pub struct ContainerManagerImpl {
    db: Arc<SqliteDatabase>,
    docker: Docker,
    config: ContainerManagerConfig,
}

impl ContainerManagerImpl {
    /// 创建新的容器管理器
    pub fn new(db: Arc<SqliteDatabase>, config: ContainerManagerConfig) -> ContainerResult<Self> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| ContainerError::DockerError(e.to_string()))?;
        
        Ok(Self {
            db,
            docker,
            config,
        })
    }

    /// 从 URL 连接到 Docker
    pub fn connect_with_url(db: Arc<SqliteDatabase>, url: &str, config: ContainerManagerConfig) -> ContainerResult<Self> {
        let docker = Docker::connect_with_http(url, 120, bollard::API_DEFAULT_VERSION)
            .map_err(|e| ContainerError::DockerError(e.to_string()))?;
        
        Ok(Self {
            db,
            docker,
            config,
        })
    }

    /// 构建容器配置
    fn build_container_config(&self, config: &ContainerConfig) -> ContainerResult<Config<String>> {
        let mut host_config = HostConfig::default();
        
        // 设置资源限制
        if let Some(memory) = config.resource_limits.memory_limit {
            host_config.memory = Some(memory as i64);
        }
        
        if let Some(cpu) = config.resource_limits.cpu_limit {
            host_config.cpu_quota = Some((cpu * 100000.0) as i64);
            host_config.cpu_period = Some(100000);
        }
        
        // 设置卷挂载
        let mut mounts = Vec::new();
        for volume in &config.volumes {
            let mount = Mount {
                target: Some(volume.container_path.clone()),
                source: Some(volume.host_path.clone()),
                typ: Some(MountTypeEnum::BIND),
                read_only: Some(volume.read_only),
                ..Default::default()
            };
            mounts.push(mount);
        }
        host_config.mounts = Some(mounts);
        
        // 设置端口映射
        let mut port_bindings = HashMap::new();
        for port in &config.ports {
            let port_key = format!("{}/{}", port.container_port, 
                match port.protocol {
                    Protocol::TCP => "tcp",
                    Protocol::UDP => "udp",
                });
            let port_binding = PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(port.host_port.to_string()),
            };
            port_bindings.insert(port_key, Some(vec![port_binding]));
        }
        host_config.port_bindings = Some(port_bindings);
        
        // 设置安全选项
        if !config.security_options.is_empty() {
            host_config.security_opt = Some(config.security_options.clone());
        }
        
        // 设置网络模式
        host_config.network_mode = Some(self.config.default_network.clone());
        
        let container_config = Config {
            image: Some(config.image.clone()),
            cmd: Some(config.command.clone()),
            env: Some(config.environment.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect()),
            labels: Some(config.labels.clone()),
            host_config: Some(host_config),
            ..Default::default()
        };
        
        Ok(container_config)
    }

    /// 拉取镜像
    async fn pull_image(&self, image: &str) -> ContainerResult<()> {
        info!("Pulling image: {}", image);
        
        let options = Some(CreateImageOptions {
            from_image: image,
            ..Default::default()
        });
        
        let mut stream = self.docker.create_image(options, None, None);
        
        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    debug!("Image pull progress: {:?}", info);
                }
                Err(e) => {
                    error!("Failed to pull image {}: {}", image, e);
                    return Err(ContainerError::ImagePullFailed(e.to_string()));
                }
            }
        }
        
        info!("Successfully pulled image: {}", image);
        Ok(())
    }

    /// 将容器摘要转换为容器信息
    fn container_summary_to_info(&self, summary: ContainerSummary) -> ContainerInfo {
        let status = match summary.state.as_deref() {
            Some("created") => ContainerStatus::Created,
            Some("running") => ContainerStatus::Running,
            Some("paused") => ContainerStatus::Paused,
            Some("restarting") => ContainerStatus::Running,
            Some("removing") => ContainerStatus::Removing,
            Some("exited") => ContainerStatus::Exited,
            Some("dead") => ContainerStatus::Dead,
            _ => ContainerStatus::Exited,
        };
        
        ContainerInfo {
            id: ContainerId::new(summary.id.unwrap_or_default()),
            name: summary.names.unwrap_or_default().first().unwrap_or(&"".to_string()).clone(),
            image: summary.image.unwrap_or_default(),
            status,
            created_at: Utc::now(), // 简化处理，实际应解析 summary.created
            started_at: None,
            finished_at: None,
            resource_usage: ResourceUsage::default(),
        }
    }

    /// 获取容器统计信息
    async fn get_container_stats_internal(&self, container_id: &ContainerId) -> ContainerResult<ResourceUsage> {
        let stats = self.docker.stats(container_id.as_str(), Some(bollard::container::StatsOptions {
            stream: false,
            one_shot: true,
        }));
        
        // 简化处理，实际应解析统计信息
        Ok(ResourceUsage::default())
    }
}

#[async_trait]
impl ContainerManager for ContainerManagerImpl {
    async fn create_container(&self, config: ContainerConfig) -> ContainerResult<ContainerId> {
        info!("Creating container with image: {}", config.image);
        
        // 拉取镜像
        self.pull_image(&config.image).await?;
        
        // 构建容器配置
        let container_config = self.build_container_config(&config)?;
        
        // 创建容器
        let options = CreateContainerOptions {
            name: format!("stepflow-{}", uuid::Uuid::new_v4()),
            platform: None,
        };
        
        let result = self.docker.create_container(Some(options), container_config).await
            .map_err(|e| ContainerError::ContainerCreationFailed(e.to_string()))?;
        
        let container_id = ContainerId::new(result.id);
        info!("Created container: {}", container_id.as_str());
        
        Ok(container_id)
    }

    async fn start_container(&self, container_id: &ContainerId) -> ContainerResult<()> {
        info!("Starting container: {}", container_id.as_str());
        
        self.docker.start_container(container_id.as_str(), None::<StartContainerOptions<String>>).await
            .map_err(|e| ContainerError::ContainerStartFailed(e.to_string()))?;
        
        info!("Started container: {}", container_id.as_str());
        Ok(())
    }

    async fn stop_container(&self, container_id: &ContainerId) -> ContainerResult<()> {
        info!("Stopping container: {}", container_id.as_str());
        
        let options = StopContainerOptions {
            t: 10, // 10 seconds timeout
        };
        
        self.docker.stop_container(container_id.as_str(), Some(options)).await
            .map_err(|e| ContainerError::ContainerStopFailed(e.to_string()))?;
        
        info!("Stopped container: {}", container_id.as_str());
        Ok(())
    }

    async fn delete_container(&self, container_id: &ContainerId) -> ContainerResult<()> {
        info!("Deleting container: {}", container_id.as_str());
        
        let options = RemoveContainerOptions {
            force: true,
            v: true,
            link: false,
        };
        
        self.docker.remove_container(container_id.as_str(), Some(options)).await
            .map_err(|e| ContainerError::ContainerDeleteFailed(e.to_string()))?;
        
        info!("Deleted container: {}", container_id.as_str());
        Ok(())
    }

    async fn get_container_status(&self, container_id: &ContainerId) -> ContainerResult<ContainerStatus> {
        let inspect = self.docker.inspect_container(container_id.as_str(), None).await
            .map_err(|e| ContainerError::ContainerNotFound(e.to_string()))?;
        
        let status = match inspect.state.and_then(|s| s.status) {
            Some(bollard::models::ContainerStateStatusEnum::CREATED) => ContainerStatus::Created,
            Some(bollard::models::ContainerStateStatusEnum::RUNNING) => ContainerStatus::Running,
            Some(bollard::models::ContainerStateStatusEnum::PAUSED) => ContainerStatus::Paused,
            Some(bollard::models::ContainerStateStatusEnum::RESTARTING) => ContainerStatus::Running,
            Some(bollard::models::ContainerStateStatusEnum::REMOVING) => ContainerStatus::Removing,
            Some(bollard::models::ContainerStateStatusEnum::EXITED) => ContainerStatus::Exited,
            Some(bollard::models::ContainerStateStatusEnum::DEAD) => ContainerStatus::Dead,
            _ => ContainerStatus::Exited,
        };
        
        Ok(status)
    }

    async fn get_container_info(&self, container_id: &ContainerId) -> ContainerResult<ContainerInfo> {
        let inspect = self.docker.inspect_container(container_id.as_str(), None).await
            .map_err(|e| ContainerError::ContainerNotFound(e.to_string()))?;
        
        let status = match inspect.state.and_then(|s| s.status) {
            Some(bollard::models::ContainerStateStatusEnum::CREATED) => ContainerStatus::Created,
            Some(bollard::models::ContainerStateStatusEnum::RUNNING) => ContainerStatus::Running,
            Some(bollard::models::ContainerStateStatusEnum::PAUSED) => ContainerStatus::Paused,
            Some(bollard::models::ContainerStateStatusEnum::RESTARTING) => ContainerStatus::Running,
            Some(bollard::models::ContainerStateStatusEnum::REMOVING) => ContainerStatus::Removing,
            Some(bollard::models::ContainerStateStatusEnum::EXITED) => ContainerStatus::Exited,
            Some(bollard::models::ContainerStateStatusEnum::DEAD) => ContainerStatus::Dead,
            _ => ContainerStatus::Exited,
        };
        
        let info = ContainerInfo {
            id: container_id.clone(),
            name: inspect.name.unwrap_or_default(),
            image: inspect.config.and_then(|c| c.image).unwrap_or_default(),
            status,
            created_at: Utc::now(), // 简化处理
            started_at: None,
            finished_at: None,
            resource_usage: ResourceUsage::default(),
        };
        
        Ok(info)
    }

    async fn list_containers(&self) -> ContainerResult<Vec<ContainerInfo>> {
        let options = ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        };
        
        let containers = self.docker.list_containers(Some(options)).await
            .map_err(|e| ContainerError::DockerError(e.to_string()))?;
        
        let container_infos = containers.into_iter()
            .map(|c| self.container_summary_to_info(c))
            .collect();
        
        Ok(container_infos)
    }

    async fn execute_in_container(&self, container_id: &ContainerId, command: Command) -> ContainerResult<ExecutionResult> {
        info!("Executing command in container {}: {} {:?}", 
              container_id.as_str(), command.program, command.args);
        
        let mut cmd = vec![command.program];
        cmd.extend(command.args);
        
        let exec_options = CreateExecOptions {
            cmd: Some(cmd),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            env: Some(command.environment.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect()),
            working_dir: command.working_directory,
            ..Default::default()
        };
        
        let exec = self.docker.create_exec(container_id.as_str(), exec_options).await
            .map_err(|e| ContainerError::DockerError(e.to_string()))?;
        
        let start_time = std::time::Instant::now();
        
        let start_exec = self.docker.start_exec(&exec.id, None).await
            .map_err(|e| ContainerError::DockerError(e.to_string()))?;
        
        let mut stdout = String::new();
        let mut stderr = String::new();
        
        match start_exec {
            StartExecResults::Attached { mut output, .. } => {
                while let Some(Ok(msg)) = output.next().await {
                    match msg {
                        bollard::container::LogOutput::StdOut { message } => {
                            stdout.push_str(&String::from_utf8_lossy(&message));
                        }
                        bollard::container::LogOutput::StdErr { message } => {
                            stderr.push_str(&String::from_utf8_lossy(&message));
                        }
                        _ => {}
                    }
                }
            }
            StartExecResults::Detached => {
                return Err(ContainerError::DockerError("Unexpected detached execution".to_string()));
            }
        }
        
        let execution_time = start_time.elapsed();
        
        // 获取退出代码
        let inspect = self.docker.inspect_exec(&exec.id).await
            .map_err(|e| ContainerError::DockerError(e.to_string()))?;
        
        let exit_code = inspect.exit_code.unwrap_or(-1) as i32;
        
        let result = ExecutionResult {
            exit_code,
            stdout,
            stderr,
            execution_time,
            resource_usage: ResourceUsage::default(),
        };
        
        info!("Command execution completed with exit code: {}", exit_code);
        Ok(result)
    }

    async fn get_container_logs(&self, container_id: &ContainerId, lines: Option<usize>) -> ContainerResult<Vec<String>> {
        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: lines.map(|n| n.to_string()).unwrap_or_else(|| "all".to_string()),
            ..Default::default()
        };
        
        let mut stream = self.docker.logs(container_id.as_str(), Some(options));
        let mut logs = Vec::new();
        
        while let Some(result) = stream.next().await {
            match result {
                Ok(log_output) => {
                    let log_line = match log_output {
                        bollard::container::LogOutput::StdOut { message } => {
                            String::from_utf8_lossy(&message).to_string()
                        }
                        bollard::container::LogOutput::StdErr { message } => {
                            String::from_utf8_lossy(&message).to_string()
                        }
                        _ => continue,
                    };
                    logs.push(log_line);
                }
                Err(e) => {
                    warn!("Error reading container logs: {}", e);
                    break;
                }
            }
        }
        
        Ok(logs)
    }

    async fn get_container_stats(&self, container_id: &ContainerId) -> ContainerResult<ResourceUsage> {
        self.get_container_stats_internal(container_id).await
    }

    async fn pause_container(&self, container_id: &ContainerId) -> ContainerResult<()> {
        info!("Pausing container: {}", container_id.as_str());
        
        self.docker.pause_container(container_id.as_str()).await
            .map_err(|e| ContainerError::DockerError(e.to_string()))?;
        
        info!("Paused container: {}", container_id.as_str());
        Ok(())
    }

    async fn resume_container(&self, container_id: &ContainerId) -> ContainerResult<()> {
        info!("Resuming container: {}", container_id.as_str());
        
        self.docker.unpause_container(container_id.as_str()).await
            .map_err(|e| ContainerError::DockerError(e.to_string()))?;
        
        info!("Resumed container: {}", container_id.as_str());
        Ok(())
    }
} 