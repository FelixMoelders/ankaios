// Copyright (c) 2024 Elektrobit Automotive GmbH
//
// This program and the accompanying materials are made available under the
// terms of the Apache License, Version 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.
//
// SPDX-License-Identifier: Apache-2.0

use common::objects::{DeletedWorkload, FulfilledBy, WorkloadSpec};

use std::collections::HashMap;

use crate::parameter_storage::ParameterStorage;

pub type ReadyWorkloads = Vec<WorkloadSpec>;
pub type WaitingWorkloads = Vec<WorkloadSpec>;
pub type ReadyDeletedWorkloads = Vec<DeletedWorkload>;
pub type WaitingDeletedWorkloads = Vec<DeletedWorkload>;

type StartWorkloadQueue = HashMap<String, WorkloadSpec>;
type DeleteWorkloadQueue = HashMap<String, DeletedWorkload>;

pub struct DependencyScheduler {
    start_queue: StartWorkloadQueue,
    delete_queue: DeleteWorkloadQueue,
}

impl DependencyScheduler {
    pub fn new() -> Self {
        DependencyScheduler {
            start_queue: StartWorkloadQueue::new(),
            delete_queue: DeleteWorkloadQueue::new(),
        }
    }

    pub fn split_workloads_to_ready_and_waiting(
        new_workloads: Vec<WorkloadSpec>,
    ) -> (ReadyWorkloads, WaitingWorkloads) {
        let mut ready_to_start_workloads = Vec::new();
        let mut waiting_to_start_workloads = Vec::new();

        for workload in new_workloads {
            if workload.dependencies.is_empty() {
                ready_to_start_workloads.push(workload);
            } else {
                waiting_to_start_workloads.push(workload);
            }
        }
        (ready_to_start_workloads, waiting_to_start_workloads)
    }

    pub fn put_on_waiting_queue(&mut self, workloads: WaitingWorkloads) {
        self.start_queue.extend(
            workloads
                .into_iter()
                .map(|workload| (workload.name.clone(), workload)),
        );
    }

    pub fn split_deleted_workloads_to_ready_and_waiting(
        deleted_workloads: Vec<DeletedWorkload>,
    ) -> (ReadyDeletedWorkloads, WaitingDeletedWorkloads) {
        let mut ready_to_delete_workloads = Vec::new();
        let mut waiting_to_delete_workloads = Vec::new();

        for workload in deleted_workloads {
            if workload.dependencies.is_empty() {
                ready_to_delete_workloads.push(workload);
            } else {
                waiting_to_delete_workloads.push(workload);
            }
        }
        (ready_to_delete_workloads, waiting_to_delete_workloads)
    }

    pub fn put_on_delete_waiting_queue(&mut self, workloads: WaitingDeletedWorkloads) {
        self.delete_queue.extend(
            workloads
                .into_iter()
                .map(|workload| (workload.name.clone(), workload)),
        );
    }

    pub fn next_workloads_to_start(
        &mut self,
        workload_state_db: &ParameterStorage,
    ) -> ReadyWorkloads {
        let mut ready_workloads = Vec::new();
        for workload_spec in self.start_queue.values() {
            if workload_spec
                .dependencies
                .iter()
                .all(|(dependency_name, add_condition)| {
                    if let Some(wl_state) = workload_state_db.get_workload_state(dependency_name) {
                        add_condition.fulfilled_by(wl_state)
                    } else {
                        false
                    }
                })
            {
                ready_workloads.push(workload_spec.clone());
            }
        }

        for workload in ready_workloads.iter() {
            self.start_queue.remove(&workload.name);
        }

        ready_workloads
    }

    pub fn next_workloads_to_delete(
        &mut self,
        workload_state_db: &ParameterStorage,
    ) -> ReadyDeletedWorkloads {
        let mut ready_workloads = Vec::new();
        for deleted_workload in self.delete_queue.values() {
            if deleted_workload
                .dependencies
                .iter()
                .all(|(dependency_name, delete_condition)| {
                    if let Some(wl_state) = workload_state_db.get_workload_state(dependency_name) {
                        delete_condition.fulfilled_by(wl_state)
                    } else {
                        false
                    }
                })
            {
                ready_workloads.push(deleted_workload.clone());
            }
        }

        for workload in ready_workloads.iter() {
            self.delete_queue.remove(&workload.name);
        }
        ready_workloads
    }
}
