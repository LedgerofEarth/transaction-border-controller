//! Agent coordination

use anyhow::Result;

/// Agent identifier
pub type AgentId = String;

/// Agent status
#[derive(Debug, Clone)]
pub enum AgentStatus {
    Active,
    Inactive,
    Busy,
}

/// Agent coordinator
pub struct Agent {
    pub id: AgentId,
    pub status: AgentStatus,
}

impl Agent {
    pub fn new(id: AgentId) -> Self {
        Self {
            id,
            status: AgentStatus::Active,
        }
    }
    
    /// Check if agent is available
    pub fn is_available(&self) -> bool {
        matches!(self.status, AgentStatus::Active)
    }
    
    /// Assign order to agent
    pub async fn assign_order(&mut self, _order_id: &str) -> Result<()> {
        self.status = AgentStatus::Busy;
        Ok(())
    }
}