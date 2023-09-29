use std::path::PathBuf;

use crate::{control_interface::PipesChannelContext, runtime::RuntimeError};
use common::objects::WorkloadSpec;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum WorkloadCommand {
    Stop,
    Update(Box<WorkloadSpec>, Option<PathBuf>),
}

// #[derive(Debug)]
pub struct Workload {
    channel: mpsc::Sender<WorkloadCommand>,
    control_interface: Option<PipesChannelContext>,
}

impl Workload {
    pub fn new(
        channel: mpsc::Sender<WorkloadCommand>,
        control_interface: Option<PipesChannelContext>,
    ) -> Self {
        Workload {
            channel,
            control_interface,
        }
    }

    pub async fn update(
        &mut self,
        spec: WorkloadSpec,
        control_interface: Option<PipesChannelContext>,
    ) -> Result<(), RuntimeError> {
        if let Some(control_interface) = self.control_interface.take() {
            control_interface.abort_pipes_channel_task()
        }
        self.control_interface = control_interface;

        let control_interface_path = self
            .control_interface
            .as_ref()
            .map(|control_interface| control_interface.get_api_location());

        self.channel
            .send(WorkloadCommand::Update(
                Box::new(spec),
                control_interface_path,
            ))
            .await
            .map_err(|err| RuntimeError::Update(err.to_string()))
    }

    pub async fn delete(self) -> Result<(), RuntimeError> {
        if let Some(control_interface) = self.control_interface {
            control_interface.abort_pipes_channel_task()
        }

        self.channel
            .send(WorkloadCommand::Stop)
            .await
            .map_err(|err| RuntimeError::Delete(err.to_string()))
    }
}
