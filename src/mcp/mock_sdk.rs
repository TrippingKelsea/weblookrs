// This is a mock implementation of the MCP SDK for development purposes
// until the real SDK is available

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

// Server-side types
pub mod server {
    use super::*;
    
    pub struct Server {
        actions: Arc<Mutex<HashMap<String, ContextAction>>>,
        addr: SocketAddr,
    }
    
    impl Server {
        pub fn new(config: ServerConfig) -> Self {
            Server {
                actions: Arc::new(Mutex::new(HashMap::new())),
                addr: config.addr,
            }
        }
        
        pub fn clone(&self) -> Self {
            Server {
                actions: self.actions.clone(),
                addr: self.addr,
            }
        }
        
        pub fn register_action(&mut self, action: ContextAction) -> Result<()> {
            let mut actions = self.actions.lock().unwrap();
            actions.insert(action.name.clone(), action);
            Ok(())
        }
        
        pub async fn serve(&self) -> Result<()> {
            // In a real implementation, this would start an HTTP server
            // For now, we just simulate it
            println!("Mock MCP server started on {}", self.addr);
            
            // Wait indefinitely
            let (_tx, rx) = oneshot::channel::<()>();
            let _ = rx.await;
            
            Ok(())
        }
        
        pub async fn shutdown(&self) -> Result<()> {
            println!("Mock MCP server shutting down");
            Ok(())
        }
    }
    
    pub struct ServerConfig {
        addr: SocketAddr,
        auth_disabled: bool,
    }
    
    impl ServerConfig {
        pub fn new() -> Self {
            ServerConfig {
                addr: "127.0.0.1:8000".parse().unwrap(),
                auth_disabled: false,
            }
        }
        
        pub fn with_addr(mut self, addr: SocketAddr) -> Self {
            self.addr = addr;
            self
        }
        
        pub fn with_auth_disabled(mut self) -> Self {
            self.auth_disabled = true;
            self
        }
    }
    
    pub mod context_action {
        use super::*;
        
        pub type ContextAction = super::ContextAction;
        
        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct Parameter {
            pub name: String,
            pub description: String,
            pub parameter_type: ParameterType,
            pub required: bool,
        }
        
        impl Parameter {
            pub fn new(name: &str, description: &str, parameter_type: ParameterType, required: bool) -> Self {
                Parameter {
                    name: name.to_string(),
                    description: description.to_string(),
                    parameter_type,
                    required,
                }
            }
        }
        
        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub enum ParameterType {
            String,
            Integer,
            Float,
            Boolean,
            Object,
            Array,
        }
    }
}

// Client-side types
pub mod client {
    use super::*;
    
    pub struct Client {
        endpoint: String,
        timeout: std::time::Duration,
    }
    
    impl Client {
        pub async fn new(config: ClientConfig) -> Result<Self> {
            Ok(Client {
                endpoint: config.endpoint,
                timeout: config.timeout,
            })
        }
        
        pub async fn get_available_actions(&self) -> Result<Vec<ActionInfo>> {
            // In a real implementation, this would make an HTTP request
            // For now, we just return some mock data
            Ok(vec![
                ActionInfo {
                    name: "capture_screenshot".to_string(),
                    description: "Capture a screenshot of a web page".to_string(),
                },
                ActionInfo {
                    name: "record_interaction".to_string(),
                    description: "Record an animated GIF of a web page".to_string(),
                },
            ])
        }
        
        pub async fn invoke_action(&self, action_name: &str, _params: Value) -> Result<Value> {
            // In a real implementation, this would make an HTTP request
            // For now, we just simulate it based on the action name
            match action_name {
                "capture_screenshot" => {
                    // Return a mock response with a tiny 1x1 transparent PNG in base64
                    Ok(serde_json::json!({
                        "image_data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==",
                        "format": "png"
                    }))
                },
                "record_interaction" => {
                    // Return a mock response with a tiny 1x1 GIF in base64
                    Ok(serde_json::json!({
                        "image_data": "R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7",
                        "format": "gif"
                    }))
                },
                "invalid_action" => {
                    // Simulate an error
                    Err(anyhow::anyhow!("Action not found: {}", action_name))
                },
                "slow_action" => {
                    // Simulate a slow response
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    Ok(serde_json::json!({"result": "ok"}))
                },
                _ => {
                    // Default response
                    Ok(serde_json::json!({"result": "ok"}))
                }
            }
        }
    }
    
    pub struct ClientConfig {
        endpoint: String,
        timeout: std::time::Duration,
        auth_disabled: bool,
    }
    
    impl ClientConfig {
        pub fn new() -> Self {
            ClientConfig {
                endpoint: "http://localhost:8000".to_string(),
                timeout: std::time::Duration::from_secs(30),
                auth_disabled: false,
            }
        }
        
        pub fn with_endpoint(mut self, endpoint: &str) -> Self {
            self.endpoint = endpoint.to_string();
            self
        }
        
        pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
            self.timeout = timeout;
            self
        }
        
        pub fn with_auth_disabled(mut self) -> Self {
            self.auth_disabled = true;
            self
        }
    }
    
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct ActionInfo {
        pub name: String,
        pub description: String,
    }
}

// Shared types
#[derive(Clone)]
pub struct ContextAction {
    pub name: String,
    pub description: String,
    pub parameters: Vec<server::context_action::Parameter>,
    pub handler: Arc<dyn Fn(Value) -> Result<Value> + Send + Sync>,
}

impl ContextAction {
    pub fn new(
        name: &str,
        description: &str,
        parameters: Vec<server::context_action::Parameter>,
        handler: Arc<dyn Fn(Value) -> Result<Value> + Send + Sync>,
    ) -> Self {
        ContextAction {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
            handler,
        }
    }
}
