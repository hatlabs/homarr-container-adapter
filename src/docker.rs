//! Docker container status queries
//!
//! This module provides Docker container status checks for health monitoring.
//! App discovery is handled via static registry files in /etc/halos/webapps.d/
//!
//! Note: These functions are currently unused but kept for future health monitoring features.

#![allow(dead_code)]

use bollard::container::{InspectContainerOptions, ListContainersOptions};
use bollard::Docker;

use crate::config::Config;
use crate::error::{AdapterError, Result};

/// Check if a container is running
pub async fn is_container_running(config: &Config, container_name: &str) -> Result<bool> {
    let docker =
        Docker::connect_with_socket(&config.docker_socket, 120, bollard::API_DEFAULT_VERSION)
            .map_err(|e| AdapterError::Docker(format!("Failed to connect to Docker: {}", e)))?;

    // List running containers and check if our container is among them
    let options = ListContainersOptions::<String> {
        all: false, // Only running containers
        ..Default::default()
    };

    let containers = docker
        .list_containers(Some(options))
        .await
        .map_err(|e| AdapterError::Docker(format!("Failed to list containers: {}", e)))?;

    for container in containers {
        // Check container names (Docker prefixes with /)
        if let Some(names) = container.names {
            for name in names {
                let clean_name = name.trim_start_matches('/');
                if clean_name == container_name {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

/// Get container health status
pub async fn get_container_health(config: &Config, container_name: &str) -> Result<Option<String>> {
    let docker =
        Docker::connect_with_socket(&config.docker_socket, 120, bollard::API_DEFAULT_VERSION)
            .map_err(|e| AdapterError::Docker(format!("Failed to connect to Docker: {}", e)))?;

    let container = docker
        .inspect_container(container_name, None::<InspectContainerOptions>)
        .await
        .map_err(|e| AdapterError::Docker(format!("Failed to inspect container: {}", e)))?;

    // Get health status if available
    let health = container
        .state
        .and_then(|s| s.health)
        .and_then(|h| h.status)
        .map(|s| format!("{:?}", s));

    Ok(health)
}

#[cfg(test)]
mod tests {
    // Integration tests would require a running Docker daemon
    // Unit tests for this module are limited since most functionality
    // requires actual Docker API calls
}
