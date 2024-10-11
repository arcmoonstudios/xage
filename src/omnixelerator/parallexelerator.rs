// src/omnixelerator/parallexelerator.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[OMNIXELERATOR]Xyn>=====S===t===u===d===i===o===s======[R|$>

use crate::task::{TaskMaster, TaskMetadata, TaskStatus, TaskWrapper, Hyperparameters, TunerConfig, TaskProgress};
use crate::execution::{ExecutionContext, CudaContext, OpenClContext, WgpuContext};
use crate::omnixtracker::omnixerror::{OmniXError, OmniXErrorManager};
use crate::persistence::{PersistenceManager, ComplexTaskState};
use crate::resource_monitor::{SystemHardware, GPUInfo};
use tokio::sync::{mpsc, oneshot, Semaphore};
use opencl3::device::Device as OpenCLDevice;
use log::{debug, error, info, trace, warn};
use futures::stream::FuturesUnordered;
use serde::{Serialize, Deserialize};
use nvml_wrapper::{Nvml, Device};
use async_trait::async_trait;
use tokio::task::JoinHandle;
use sha2::{Digest, Sha256};
use warp::reject::Reject;
use std::time::Duration;
use parking_lot::Mutex;
use futures::FutureExt;
use rayon::prelude::*;
use std::ffi::CString;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use rand::Rng;
use bincode;

#[async_trait]
pub trait OmniXurge: Send + Sync {
    async fn parallelize_task<T: TaskMaster + 'static>(&self, task: T) -> Result<Uuid, OmniXError>;
    async fn get_parallelized_task_status(&self, task_id: Uuid) -> Option<TaskMetadata>;
    async fn get_resource_utilization(&self) -> Result<ResourceMonitor, OmniXError>;
    async fn tune_hyperparameters<H: Hyperparameters>(&self, config: TunerConfig<H>) -> Result<H, OmniXError>;
    async fn accelerate(&self) -> Result<(), OmniXError>;
    async fn decelerate(&self) -> Result<(), OmniXError>;
    async fn collect_metrics(&self) -> Result<Metrics, OmniXError>;
    async fn shutdown(&self) -> Result<(), OmniXError>;
    async fn submit_task_with_progress<T: TaskMaster + 'static, F: Fn(TaskProgress) + Send + 'static>(
        &self,
        task: T,
        progress_callback: F,
    ) -> Result<Uuid, OmniXError>;
    async fn recover_and_resume_tasks(&self) -> Result<(), OmniXError>;
}

#[async_trait]
pub trait OmniXurge: Send + Sync {
    async fn parallelize_task<T: TaskMaster + 'static>(&self, task: T) -> Result<Uuid, OmniXError>;
    async fn get_parallelized_task_status(&self, task_id: Uuid) -> Option<TaskMetadata>;
    async fn get_resource_utilization(&self) -> Result<ResourceMonitor, OmniXError>;
    async fn tune_hyperparameters<H: Hyperparameters>(&self, config: TunerConfig<H>) -> Result<H, OmniXError>;
    async fn accelerate(&self) -> Result<(), OmniXError>;
    async fn decelerate(&self) -> Result<(), OmniXError>;
    async fn collect_metrics(&self) -> Result<Metrics, OmniXError>;
    async fn shutdown(&self) -> Result<(), OmniXError>;
    async fn submit_task_with_progress<T: TaskMaster + 'static, F: Fn(TaskProgress) + Send + 'static>(
        &self,
        task: T,
        progress_callback: F,
    ) -> Result<Uuid, OmniXError>;
    async fn recover_and_resume_tasks(&self) -> Result<(), OmniXError>;
}

pub struct ParalleXelerator {
    task_sender: mpsc::Sender<TaskWrapper>,
    task_store: Arc<dashmap::DashMap<Uuid, TaskMetadata>>,
    db: sled::Db,
    resource_monitor: Arc<Mutex<ResourceMonitor>>,
    hardware: SystemHardware,
    task_concurrency: Arc<AtomicUsize>,
}

#[async_trait]
impl OmniXurge for ParalleXelerator {
    async fn parallelize_task<T: TaskMaster + 'static>(&self, task: T) -> Result<Uuid, OmniXError> {
        let task_id = Uuid::new_v4();
        let metadata = TaskMetadata {
            id: task_id,
            submitted_at: Utc::now(),
            started_at: None,
            completed_at: None,
            dependencies: task.dependencies(),
            gpu_compatible: task.is_gpu_compatible(),
            status: TaskStatus::Queued,
            complexity: task.estimated_complexity(),
            priority: task.priority(),
        };

        let task_wrapper = TaskWrapper {
            task: Box::new(task),
            metadata: metadata.clone(),
            cancel_token: None,
        };

        self.task_sender.send(task_wrapper).await
            .map_err(|e| OmniXError::TaskSubmissionError(format!("Failed to submit task: {}", e)))?;

        self.task_store.insert(task_id, metadata.clone());
        self.db.insert(task_id.to_string(), bincode::serialize(&metadata)?)?;
        info!("Task submitted successfully with ID: {}", task_id);
        Ok(task_id)
    }

    async fn get_parallelized_task_status(&self, task_id: Uuid) -> Option<TaskMetadata> {
        self.task_store.get(&task_id).map(|entry| entry.value().clone())
    }

    async fn get_resource_utilization(&self) -> Result<ResourceMonitor, OmniXError> {
        let monitor = self.resource_monitor.lock().clone();
        Ok(monitor)
    }

    async fn tune_hyperparameters<H: Hyperparameters>(&self, config: TunerConfig<H>) -> Result<H, OmniXError> {
        let mut rng = rand::thread_rng();
        let mut population: Vec<H> = (0..config.population_size)
            .map(|_| config.initial_hyperparameters.clone().mutate())
            .collect();
        let mut global_best = config.initial_hyperparameters.clone();
        let mut global_best_score = (config.objective_function)(&global_best);
        
        for _ in 0..config.iterations {
            let (new_population, scores): (Vec<H>, Vec<f64>) = population.into_par_iter()
                .map(|particle| {
                    let score = (config.objective_function)(&particle);
                    (particle, score)
                })
                .unzip();
            population = new_population;

            let (best_particle, best_score) = population.iter()
                .zip(scores.iter())
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(CmpOrdering::Equal))
                .map(|(p, s)| (p.clone(), *s))
                .unwrap();

            if best_score > global_best_score {
                global_best = best_particle;
                global_best_score = best_score;
            }

            population = population.into_par_iter()
                .map(|p| p.crossover(&global_best).mutate())
                .collect();
        }
        Ok(global_best)
    }

    async fn accelerate(&self) -> Result<(), OmniXError> {
        let mut resource = self.resource_monitor.lock();
        let cpu_usage = resource.cpu_usage.iter().sum::<f32>() / resource.cpu_usage.len() as f32;
        let gpu_usage = resource.gpu_usage.iter().sum::<f32>() / resource.gpu_usage.len() as f32;
    
        if cpu_usage < 70.0 && gpu_usage < 70.0 {
            self.task_concurrency.fetch_add(1, Ordering::SeqCst);
            info!("Accelerated: Increased task concurrency");
        } else {
            warn!("Cannot accelerate: Resource usage is too high");
        }
        Ok(())
    }
    
    async fn decelerate(&self) -> Result<(), OmniXError> {
        let current_concurrency = self.task_concurrency.load(Ordering::SeqCst);
        if current_concurrency > 1 {
            self.task_concurrency.fetch_sub(1, Ordering::SeqCst);
            info!("Decelerated: Decreased task concurrency");
        } else {
            warn!("Cannot decelerate: Task concurrency is already at minimum");
        }
        Ok(())
    }

    async fn collect_metrics(&self) -> Result<Metrics, OmniXError> {
        let resource = self.resource_monitor.lock();
        let cpu_average = resource.cpu_usage.iter().sum::<f32>() / resource.cpu_usage.len() as f32;
        let gpu_average = if !resource.gpu_usage.is_empty() {
            resource.gpu_usage.iter().sum::<f32>() / resource.gpu_usage.len() as f32
        } else {
            0.0
        };
        let memory_usage_ratio = (self.hardware.total_memory - resource.available_memory) as f32 / self.hardware.total_memory as f32;
        let active_tasks = self.task_store.iter().filter(|entry| {
            matches!(entry.value().status, TaskStatus::Running | TaskStatus::Queued)
        }).count();
        let queued_tasks = self.task_store.iter().filter(|entry| {
            matches!(entry.value().status, TaskStatus::Queued)
        }).count();

        Ok(Metrics {
            cpu_usage_average: cpu_average,
            gpu_usage_average: gpu_average,
            memory_usage_ratio,
            active_tasks,
            queued_tasks,
        })
    }

    async fn shutdown(&self) -> Result<(), OmniXError> {
        // Implement graceful shutdown logic
        info!("Initiating ParalleXelerator shutdown");
        // Cancel all running tasks
        for entry in self.task_store.iter() {
            if let TaskStatus::Running = entry.value().status {
                if let Some(cancel_token) = &entry.value().cancel_token {
                    let _ = cancel_token.send(());
                }
            }
        }
        // Wait for all tasks to complete or timeout
        tokio::time::timeout(Duration::from_secs(30), self.wait_for_tasks_completion()).await
            .map_err(|_| OmniXError::ShutdownError("Timed out waiting for tasks to complete".to_string()))?;
        
        // Persist any remaining state
        self.db.flush_async().await
            .map_err(|e| OmniXError::ShutdownError(format!("Failed to flush database: {}", e)))?;
        
        info!("ParalleXelerator shutdown completed successfully");
        Ok(())
    }

    async fn submit_task_with_progress<T: TaskMaster + 'static, F: Fn(TaskProgress) + Send + 'static>(
        &self,
        task: T,
        progress_callback: F,
    ) -> Result<Uuid, OmniXError> {
        let task_id = Uuid::new_v4();
        let (progress_sender, mut progress_receiver) = mpsc::channel(100);
    
        let wrapped_task = Box::new(move || {
            let result = task.run();
            if let Err(ref e) = result {
                progress_callback(TaskProgress {
                    task_id,
                    progress: 1.0,
                    status: TaskStatus::Failed,
                    message: Some(e.to_string()),
                });
            } else {
                progress_callback(TaskProgress {
                    task_id,
                    progress: 1.0,
                    status: TaskStatus::Completed,
                    message: None,
                });
            }
            result
        }) as Box<dyn FnOnce() -> Result<(), OmniXError> + Send + 'static>;
    
        let task_wrapper = TaskWrapper {
            task: wrapped_task,
            metadata: TaskMetadata {
                id: task_id,
                submitted_at: Utc::now(),
                started_at: None,
                completed_at: None,
                dependencies: task.dependencies(),
                gpu_compatible: task.is_gpu_compatible(),
                status: TaskStatus::Queued,
                complexity: task.estimated_complexity(),
                priority: task.priority(),
            },
            cancel_token: None,
        };
    
        self.task_sender.send(task_wrapper).await
            .map_err(|e| OmniXError::TaskSubmissionError(format!("Failed to submit task: {}", e)))?;
    
        tokio::spawn(async move {
            while let Some(progress) = progress_receiver.recv().await {
                progress_callback(progress);
            }
        });
    
        Ok(task_id)
    }

    async fn recover_and_resume_tasks(&self) -> Result<(), OmniXError> {
        let tasks = self.db.iter()
            .filter_map(|res| res.ok())
            .filter_map(|(key, value)| {
                let task_id = Uuid::parse_str(std::str::from_utf8(&key).ok()?).ok()?;
                let metadata: TaskMetadata = bincode::deserialize(&value).ok()?;
                if matches!(metadata.status, TaskStatus::Running | TaskStatus::Queued) {
                    Some((task_id, metadata))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for (task_id, mut metadata) in tasks {
            metadata.status = TaskStatus::Queued;
            let task_wrapper = TaskWrapper {
                task: Box::new(RecoverableTask { id: task_id }),
                metadata: metadata.clone(),
                cancel_token: None,
            };

            self.task_sender.send(task_wrapper).await
                .map_err(|e| OmniXError::TaskSubmissionError(format!("Failed to resume task: {}", e)))?;

            self.task_store.insert(task_id, metadata.clone());
            self.db.insert(task_id.to_string(), bincode::serialize(&metadata)?)?;
            info!("Resumed task with ID: {}", task_id);
        }

        Ok(())
    }
}

impl ParalleXelerator {
    async fn wait_for_tasks_completion(&self) -> Result<(), OmniXError> {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let active_tasks = self.task_store.iter().filter(|entry| {
                matches!(entry.value().status, TaskStatus::Running | TaskStatus::Queued)
            }).count();
            
            if active_tasks == 0 {
                break;
            }
            
            info!("Waiting for {} active tasks to complete", active_tasks);
        }
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), OmniXError> {
        info!("Initiating ParalleXelerator shutdown");
        
        // Stop accepting new tasks
        // (Assuming we have a method to close the task_sender)
        self.close_task_sender().await?;
        
        // Cancel all running tasks
        for entry in self.task_store.iter_mut() {
            if let TaskStatus::Running = entry.value().status {
                if let Some(cancel_token) = &entry.value().cancel_token {
                    let _ = cancel_token.send(());
                    entry.value_mut().status = TaskStatus::Cancelled;
                }
            }
        }
        
        // Wait for all tasks to complete or timeout
        match tokio::time::timeout(Duration::from_secs(30), self.wait_for_tasks_completion()).await {
            Ok(_) => info!("All tasks completed successfully"),
            Err(_) => {
                warn!("Timed out waiting for tasks to complete. Some tasks may not have finished.");
                // Optionally, we could forcefully terminate remaining tasks here
            }
        }
        
        // Persist final state
        self.persist_final_state().await?;
        
        // Close database connection
        self.db.flush_async().await
            .map_err(|e| OmniXError::ShutdownError(format!("Failed to flush database: {}", e)))?;
        
        info!("ParalleXelerator shutdown completed successfully");
        Ok(())
    }

    async fn close_task_sender(&self) -> Result<(), OmniXError> {
        // Implementation depends on how we've set up the task_sender
        // For example, if it's wrapped in an Arc<Mutex<Option<mpsc::Sender<...>>>>:
        if let Some(sender) = self.task_sender.lock().await.take() {
            sender.close();
        }
        Ok(())
    }

    async fn persist_final_state(&self) -> Result<(), OmniXError> {
        for (task_id, metadata) in self.task_store.iter() {
            self.db.insert(task_id.to_string(), bincode::serialize(metadata)?)?;
        }
        Ok(())
    }
}