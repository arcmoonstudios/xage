// src/omnixelerator/execution.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[OMNIXELERATOR]Xyn>=====S===t===u===d===i===o===s======[R|$>

use crate::omnixelerator::task_manager::{TaskMaster, TaskStatus, TaskMetadata};
use crate::omnixtracker::omnixerror::OmniXError;
use crate::constants::*;
use async_trait::async_trait;
use cuda_sys::{
    cuCtxCreate_v2, cuDeviceGet, cuInit, cuModuleGetFunction, cuLaunchKernel, cuMemAlloc_v2,
    cuMemcpyHtoD_v2, cuMemcpyDtoH_v2, cuMemFree_v2, CUcontext, CUdevice, CUfunction, CUmodule,
    CUresult, CUDA_SUCCESS,
};
use futures::future::BoxFuture;
use lazy_static::lazy_static;
use log::{error, warn, debug};
use nvrtc_sys::{
    nvrtcCompileProgram, nvrtcCreateProgram, nvrtcDestroyProgram, nvrtcGetPTX, nvrtcGetPTXSize,
    nvrtcProgram, nvrtcResult, NVRTC_SUCCESS,
};
use opencl3::{
    context::{Context, ContextProperties},
    device::{Device as OpenClDevice, CL_DEVICE_TYPE_GPU},
    platform::get_platforms,
    program::Program,
    queue::CommandQueue,
};
use parking_lot::Mutex;
use std::{
    ffi::{CStr, CString},
    mem::{MaybeUninit, size_of},
    ptr::null_mut,
    sync::Arc,
};
use tokio::task;
use wgpu::{Adapter, Device as WgpuDevice, Instance, Limits, PowerPreference, Queue as WgpuQueue, RequestAdapterOptions};

lazy_static! {
    static ref CUDA_CONTEXT: Arc<Mutex<CudaContext>> = Arc::new(Mutex::new(CudaContext::new().expect("Failed to initialize CUDA context")));
}

pub struct ExecutionContext {
    pub opencl_context: Option<Arc<Mutex<OpenClContext>>>,
    pub cuda_context: Option<Arc<Mutex<CudaContext>>>,
    pub wgpu_context: Option<Arc<Mutex<WgpuContext>>>,
}

impl ExecutionContext {
    pub async fn new() -> Result<Self, OmniXError> {
        let (cuda_ctx, opencl_ctx, wgpu_ctx) = tokio::try_join!(
            Self::initialize_cuda(),
            Self::initialize_opencl(),
            Self::initialize_wgpu()
        )?;
        
        Ok(Self {
            cuda_context: cuda_ctx,
            opencl_context: opencl_ctx,
            wgpu_context: wgpu_ctx,
        })
    }

    async fn initialize_cuda() -> Result<Option<Arc<Mutex<CudaContext>>>, OmniXError> {
        match CudaContext::new().await {
            Ok(context) => Ok(Some(Arc::new(Mutex::new(context)))),
            Err(e) => {
                warn!("Failed to initialize CUDA context: {}", e);
                Ok(None)
            }
        }
    }

    async fn initialize_opencl() -> Result<Option<Arc<Mutex<OpenClContext>>>, OmniXError> {
        match OpenClContext::new().await {
            Ok(context) => Ok(Some(Arc::new(Mutex::new(context)))),
            Err(e) => {
                warn!("Failed to initialize OpenCL context: {}", e);
                Ok(None)
            }
        }
    }

    async fn initialize_wgpu() -> Result<Option<Arc<Mutex<WgpuContext>>>, OmniXError> {
        match WgpuContext::new().await {
            Ok(context) => Ok(Some(Arc::new(Mutex::new(context)))),
            Err(e) => {
                warn!("Failed to initialize WGPU context: {}", e);
                Ok(None)
            }
        }
    }
}

pub struct CudaContext {
    context: CUcontext,
    device: CUdevice,
}

impl CudaContext {
    pub async fn new() -> Result<Self, OmniXError> {
        task::spawn_blocking(|| unsafe {
            check_cuda_error(cuInit(0))?;
            let mut device = MaybeUninit::<CUdevice>::uninit();
            check_cuda_error(cuDeviceGet(device.as_mut_ptr(), 0))?;
            let mut context = MaybeUninit::<CUcontext>::uninit();
            check_cuda_error(cuCtxCreate_v2(context.as_mut_ptr(), 0, device.assume_init()))?;
            Ok(CudaContext {
                context: context.assume_init(),
                device: device.assume_init(),
            })
        })
        .await
        .map_err(|e| OmniXError::CudaInitializationError(e.to_string()))?
    }
}

pub struct OpenClContext {
    context: Context,
    device: OpenClDevice,
    queue: CommandQueue,
    program: Program,
}

impl OpenClContext {
    pub async fn new() -> Result<Self, OmniXError> {
        task::spawn_blocking(|| {
            let platforms = get_platforms()
                .map_err(|e| OmniXError::OpenCLError(e.to_string()))?;
            let platform = platforms
                .first()
                .ok_or(OmniXError::DeviceNotSupported)?;
            let devices = platform
                .get_devices(CL_DEVICE_TYPE_GPU)
                .map_err(|e| OmniXError::OpenCLError(e.to_string()))?;
            let device = devices
                .first()
                .ok_or(OmniXError::DeviceNotSupported)?
                .clone();
            let context_properties = ContextProperties::new().platform(*platform);
            let context = Context::from_device(&device, &context_properties, None, None)
                .map_err(|e| OmniXError::OpenCLError(e.to_string()))?;
            let queue = CommandQueue::create(&context, device, None)
                .map_err(|e| OmniXError::OpenCLError(e.to_string()))?;
            let program_src = include_str!("../../kernels/opencl_kernels.cl");
            let program = Program::create_and_build_from_source(&context, program_src, "")
                .map_err(|e| OmniXError::KernelCompilationError(e.to_string()))?;
            Ok(OpenClContext {
                context,
                device,
                queue,
                program,
            })
        })
        .await
        .map_err(|e| OmniXError::OpenCLError(e.to_string()))?
    }
}

pub struct WgpuContext {
    device: WgpuDevice,
    queue: WgpuQueue,
}

impl WgpuContext {
    pub async fn new() -> Result<Self, OmniXError> {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(OmniXError::DeviceNotSupported)?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .map_err(|e| OmniXError::WgpuInitError(e.to_string()))?;
        Ok(WgpuContext { device, queue })
    }
}

#[async_trait]
pub trait TaskExecutionContext: Send + Sync {
    async fn execute(
        &self,
        task: Box<dyn TaskMaster>,
        metadata: Arc<Mutex<TaskMetadata>>,
    ) -> Result<(), OmniXError>;
}

#[async_trait]
impl TaskExecutionContext for CudaContext {
    async fn execute(
        &self,
        task: Box<dyn TaskMaster>,
        metadata: Arc<Mutex<TaskMetadata>>,
    ) -> Result<(), OmniXError> {
        let task_clone = task.clone();
        task::spawn_blocking(move || unsafe {
            if let Some(kernel_fn) = task_clone.get_cuda_kernel() {
                kernel_fn(null_mut(), null_mut(), 0);
            } else {
                return Err(OmniXError::CudaKernelExecutionError(
                    "No CUDA kernel provided".to_string(),
                ));
            }
            let mut meta = metadata.lock();
            meta.status = TaskStatus::Completed;
            Ok(())
        })
        .await
        .map_err(|e| OmniXError::TaskExecutionError(e.to_string()))?
    }
}

#[async_trait]
impl TaskExecutionContext for OpenClContext {
    async fn execute(
        &self,
        task: Box<dyn TaskMaster>,
        metadata: Arc<Mutex<TaskMetadata>>,
    ) -> Result<(), OmniXError> {
        let task_clone = task.clone();
        let program = self.program.clone();
        let queue = self.queue.clone();
        task::spawn_blocking(move || {
            if let Some(kernel_name) = task_clone.get_opencl_kernel() {
                let kernel = program
                    .create_kernel(kernel_name)
                    .map_err(|e| OmniXError::OpenCLError(e.to_string()))?;
                kernel
                    .enqueue()
                    .map_err(|e| OmniXError::OpenCLError(e.to_string()))?;
            } else {
                return Err(OmniXError::OpenCLError(
                    "No OpenCL kernel provided".to_string(),
                ));
            }
            let mut meta = metadata.lock();
            meta.status = TaskStatus::Completed;
            Ok(())
        })
        .await
        .map_err(|e| OmniXError::TaskExecutionError(e.to_string()))?
    }
}

#[async_trait]
impl TaskExecutionContext for WgpuContext {
    async fn execute(
        &self,
        task: Box<dyn TaskMaster>,
        metadata: Arc<Mutex<TaskMetadata>>,
    ) -> Result<(), OmniXError> {
        let task_clone = task.clone();
        let device = self.device.clone();
        let queue = self.queue.clone();
        task::spawn_blocking(move || {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("WGPU Task Execution"),
            });
            device.push_error_scope(wgpu::ErrorFilter::Validation);
            task_clone
                .run()
                .map_err(|e| OmniXError::TaskExecutionError(e.to_string()))?;
            let command_buffer = encoder.finish();
            queue.submit(std::iter::once(command_buffer));
            match device.pop_error_scope(){
                Some(wgpu::Error::Validation { description }) => Err(
                    OmniXError::TaskExecutionError(format!(
                        "WGPU validation error: {}",
                        description
                    )),
                ),
                Some(other_error) => Err(OmniXError::TaskExecutionError(format!(
                    "WGPU error: {:?}",
                    other_error
                ))),
                None => {
                    let mut meta = metadata.lock();
                    meta.status = TaskStatus::Completed;
                    Ok(())
                }
            }
        })
        .await
        .map_err(|e| OmniXError::TaskExecutionError(e.to_string()))?
    }
}

#[async_trait]
impl TaskExecutionContext for () {
    async fn execute(
        &self,
        task: Box<dyn TaskMaster>,
        metadata: Arc<Mutex<TaskMetadata>>,
    ) -> Result<(), OmniXError> {
        task.run().await?;
        let mut meta = metadata.lock();
        meta.status = TaskStatus::Completed;
        Ok(())
    }
}

unsafe fn check_cuda_error(result: CUresult) -> Result<(), OmniXError> {
    if result != CUDA_SUCCESS {
        Err(OmniXError::CudaInitializationError(format!(
            "CUDA error code: {}",
            result
        )))
    } else {
        Ok(())
    }
}

unsafe fn check_nvrtc_error(result: nvrtcResult) -> Result<(), OmniXError> {
    if result != NVRTC_SUCCESS {
        Err(OmniXError::KernelCompilationError(format!(
            "NVRTC error code: {:?}",
            result
        )))
    } else {
        Ok(())
    }
}

pub struct KernelCompiler {
    pub context: Arc<Mutex<CudaContext>>,
}

impl KernelCompiler {
    pub fn new(context: Arc<Mutex<CudaContext>>) -> Self {
        KernelCompiler { context }
    }

    pub async fn compile_kernel(
        &self,
        kernel_code: &str,
        kernel_name: &str,
    ) -> Result<CUmodule, OmniXError> {
        let kernel_code = kernel_code.to_string();
        let kernel_name = kernel_name.to_string();
        let context = Arc::clone(&self.context);
        task::spawn_blocking(move || -> Result<CUmodule, OmniXError> {
            unsafe {
                let kernel_code_cstring = CString::new(kernel_code)
                    .map_err(|e| OmniXError::KernelCompilationError(e.to_string()))?;
                let kernel_name_cstring = CString::new(kernel_name)
                    .map_err(|e| OmniXError::KernelCompilationError(e.to_string()))?;
                let mut program = MaybeUninit::<nvrtcProgram>::uninit();
                check_nvrtc_error(nvrtcCreateProgram(
                    program.as_mut_ptr(),
                    kernel_code_cstring.as_ptr(),
                    kernel_name_cstring.as_ptr(),
                    0,
                    null_mut(),
                    null_mut(),
                ))?;
                let program = program.assume_init();
                let options = [
                    CString::new("--gpu-architecture=compute_70").unwrap(),
                    CString::new("--use_fast_math").unwrap(),
                    CString::new("--std=c++14").unwrap(),
                ];
                let option_ptrs: Vec<*const i8> = options.iter().map(|o| o.as_ptr()).collect();
                let result = nvrtcCompileProgram(program, option_ptrs.len() as i32, option_ptrs.as_ptr());
                if result != NVRTC_SUCCESS {
                    let mut log_size = 0;
                    nvrtc_sys::nvrtcGetProgramLogSize(program, &mut log_size);
                    let mut log = vec![0u8; log_size as usize];
                    nvrtc_sys::nvrtcGetProgramLog(program, log.as_mut_ptr() as *mut i8);
                    let log_str = CStr::from_ptr(log.as_ptr() as *const i8).to_string_lossy();
                    nvrtcDestroyProgram(&mut program);
                    return Err(OmniXError::KernelCompilationError(
                        log_str.to_string(),
                    ));
                }
                let mut ptx_size = 0;
                check_nvrtc_error(nvrtcGetPTXSize(program, &mut ptx_size))?;
                let mut ptx = vec![0u8; ptx_size as usize];
                check_nvrtc_error(nvrtcGetPTX(program, ptx.as_mut_ptr() as *mut i8))?;
                nvrtcDestroyProgram(&mut program);
                let mut module = MaybeUninit::<CUmodule>::uninit();
                check_cuda_error(cuModuleLoadData(
                    module.as_mut_ptr(),
                    ptx.as_ptr() as *const std::ffi::c_void,
                ))?;
                Ok(module.assume_init())
            }
        })
        .await
        .map_err(|e| OmniXError::KernelCompilationError(e.to_string()))?
    }
}

pub async fn execute_cuda_kernel_async(
    module: CUmodule,
    function_name: &str,
    params: &[*mut std::ffi::c_void],
    grid_size: u32,
    block_size: u32,
) -> Result<(), OmniXError> {
    task::spawn_blocking(move || {
        unsafe {
            let function = get_function(module, function_name)?;
            check_cuda_error(cuLaunchKernel(
                function,
                grid_size,
                1,
                1,
                block_size,
                1,
                1,
                0,
                null_mut(),
                params.as_ptr() as *mut *mut std::ffi::c_void,
                null_mut(),
            ))?;
        }
        Ok(())
    })
    .await
    .map_err(|e| OmniXError::CudaKernelExecutionError(e.to_string()))?
}

unsafe fn get_function(module: CUmodule, name: &str) -> Result<CUfunction, OmniXError> {
    let mut function = MaybeUninit::<CUfunction>::uninit();
    let c_name = CString::new(name)
        .map_err(|e| OmniXError::KernelCompilationError(e.to_string()))?;
    check_cuda_error(cuModuleGetFunction(function.as_mut_ptr(), module, c_name.as_ptr()))?;
    Ok(function.assume_init())
}

pub struct CudaArray {
    data: CUdeviceptr,
    size: usize,
}

impl CudaArray {
    pub fn new(size: usize) -> Result<Self, OmniXError> {
        let mut data = MaybeUninit::<CUdeviceptr>::uninit();
        unsafe {
            check_cuda_error(cuMemAlloc_v2(
                data.as_mut_ptr(),
                size * std::mem::size_of::<f32>(),
            ))?;
        }
        Ok(CudaArray {
            data: unsafe { data.assume_init() },
            size,
        })
    }

    pub fn from_slice(slice: &[f32]) -> Result<Self, OmniXError> {
        let mut array = Self::new(slice.len())?;
        unsafe {
            check_cuda_error(cuMemcpyHtoD_v2(
                array.data,
                slice.as_ptr() as *const std::ffi::c_void,
                slice.len() * std::mem::size_of::<f32>(),
            ))?;
        }
        Ok(array)
    }

    pub fn to_vec(&self) -> Result<Vec<f32>, OmniXError> {
        let mut result = vec![0.0f32; self.size];
        unsafe {
            check_cuda_error(cuMemcpyDtoH_v2(
                result.as_mut_ptr() as *mut std::ffi::c_void,
                self.data,
                self.size * std::mem::size_of::<f32>(),
            ))?;
        }
        Ok(result)
    }
}

impl Drop for CudaArray {
    fn drop(&mut self) {
        unsafe {
            let _ = check_cuda_error(cuMemFree_v2(self.data));
        }
    }
}

pub async fn matrix_multiply(a: &[f32], b: &[f32], m: i32, n: i32, k: i32) -> Result<Vec<f32>, OmniXError> {
    let context = CUDA_CONTEXT.clone();
    let compiler = KernelCompiler::new(context);
    if (m * k) as usize != a.len() || (k * n) as usize != b.len() {
        return Err(OmniXError::InvalidDimensionsError("Invalid input dimensions".to_string()));
    }
    let a_gpu = CudaArray::from_slice(a)?;
    let b_gpu = CudaArray::from_slice(b)?;
    let mut c_gpu = CudaArray::new((m * n) as usize)?;
    let kernel_code = r#"
        extern "C" __global__ void matrix_multiply(float *a, float *b, float *c, int m, int n, int k) {
            int row = blockIdx.y * blockDim.y + threadIdx.y;
            int col = blockIdx.x * blockDim.x + threadIdx.x;
            if (row < m && col < n) {
                float sum = 0.0f;
                for (int i = 0; i < k; ++i) {
                    sum += a[row * k + i] * b[i * n + col];
                }
                c[row * n + col] = sum;
            }
        }
    "#;
    let module = compiler.compile_kernel(kernel_code, "matrix_multiply").await?;
    let function_name = "matrix_multiply";
    let params = [
        &a_gpu.data as *const _ as *mut std::ffi::c_void,
        &b_gpu.data as *const _ as *mut std::ffi::c_void,
        &c_gpu.data as *const _ as *mut std::ffi::c_void,
        &m as *const _ as *mut std::ffi::c_void,
        &n as *const _ as *mut std::ffi::c_void,
        &k as *const _ as *mut std::ffi::c_void,
    ];
    let block_size = 16;
    let grid_size_x = ((n as u32 + block_size - 1) / block_size).max(1);
    let grid_size_y = ((m as u32 + block_size - 1) / block_size).max(1);
    execute_cuda_kernel_async(module, function_name, &params, grid_size_x * grid_size_y, block_size).await?;
    let result = c_gpu.to_vec()?;
    Ok(result)
}

pub async fn relu_activation(data: &mut [f32]) -> Result<(), OmniXError> {
    let context = CUDA_CONTEXT.clone();
    let compiler = KernelCompiler::new(context);
    let mut data_gpu = CudaArray::from_slice(data)?;
    let kernel_code = r#"
        extern "C" __global__ void relu_activation(float *data, int n) {
            int idx = blockIdx.x * blockDim.x + threadIdx.x;
            if (idx < n) {
                data[idx] = fmaxf(data[idx], 0.0f);
            }
        }
    "#;
    let module = compiler.compile_kernel(kernel_code, "relu_activation").await?;
    let function_name = "relu_activation";
    let n = data.len() as i32;
    let params = [
        &data_gpu.data as *const _ as *mut std::ffi::c_void,
        &n as *const _ as *mut std::ffi::c_void,
    ];
    let block_size = 256;
    let grid_size = ((data.len() as u32 + block_size - 1) / block_size).max(1);
    execute_cuda_kernel_async(module, function_name, &params, grid_size, block_size).await?;
    let result = data_gpu.to_vec()?;
    data.copy_from_slice(&result);
    Ok(())
}

pub async fn softmax(data: &mut [f32]) -> Result<(), OmniXError> {
    let context = CUDA_CONTEXT.clone();
    let compiler = KernelCompiler::new(context);
    let mut data_gpu = CudaArray::from_slice(data)?;
    let kernel_code = r#"
        extern "C" __global__ void softmax(float *data, int n) {
            __shared__ float max_val;
            __shared__ float sum;
            int idx = blockIdx.x * blockDim.x + threadIdx.x;
            float thread_max = -INFINITY;
            if (idx < n) {
                thread_max = data[idx];
            }
            
            for (int stride = blockDim.x / 2; stride > 0; stride >>= 1) {
                if (threadIdx.x < stride) {
                    thread_max = fmaxf(thread_max, __shfl_down_sync(0xffffffff, thread_max, stride));
                }
            }
            if (threadIdx.x == 0) {
                max_val = thread_max;
            }
            __syncthreads();
            float thread_sum = 0.0f;
            if (idx < n) {
                data[idx] = expf(data[idx] - max_val);
                thread_sum = data[idx];
            }
            for (int stride = blockDim.x / 2; stride > 0; stride >>= 1) {
                if (threadIdx.x < stride) {
                    thread_sum += __shfl_down_sync(0xffffffff, thread_sum, stride);
                }
            }
            if (threadIdx.x == 0) {
                sum = thread_sum;
            }
            __syncthreads();
            if (idx < n) {
                data[idx] /= sum;
            }
        }
    "#;
    let module = compiler.compile_kernel(kernel_code, "softmax").await?;
    let function_name = "softmax";
    let n = data.len() as i32;
    let params = [
        &data_gpu.data as *const _ as *mut std::ffi::c_void,
        &n as *const _ as *mut std::ffi::c_void,
    ];
    let block_size = 256;
    let grid_size = ((data.len() as u32 + block_size - 1) / block_size).max(1);
    execute_cuda_kernel_async(module, function_name, &params, grid_size, block_size).await?;
    let result = data_gpu.to_vec()?;
    data.copy_from_slice(&result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[tokio::test]
    async fn test_matrix_multiply() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let m = 2;
        let n = 2;
        let k = 2;
        let result = matrix_multiply(&a, &b, m, n, k).await.unwrap();
        assert_eq!(result, vec![19.0, 22.0, 43.0, 50.0]);
    }

    #[tokio::test]
    async fn test_relu_activation() {
        let mut data = vec![-1.0, 0.0, 1.0, 2.0];
        relu_activation(&mut data).await.unwrap();
        assert_eq!(data, vec![0.0, 0.0, 1.0, 2.0]);
    }

    #[tokio::test]
    async fn test_softmax() {
        let mut data = vec![1.0, 2.0, 3.0, 4.0];
        softmax(&mut data).await.unwrap();
        let sum: f32 = data.iter().sum();
        assert_approx_eq!(sum, 1.0, 1e-6);
    }

    #[tokio::test]
    async fn test_cuda_kernel_execution() {
        let context = CUDA_CONTEXT.clone();
        let compiler = KernelCompiler::new(context);
        let kernel_code = r#"
            extern "C" __global__ void add_one(float *data, int n) {
                int idx = blockIdx.x * blockDim.x + threadIdx.x;
                if (idx < n) {
                    data[idx] += 1.0f;
                }
            }
        "#;
        let module = compiler.compile_kernel(kernel_code, "add_one").await.unwrap();
        let data = vec![1.0f32, 2.0, 3.0, 4.0];
        let mut data_gpu = CudaArray::from_slice(&data).unwrap();
        let n = data.len() as i32;
        let params = [
            &data_gpu.data as *const _ as *mut std::ffi::c_void,
            &n as *const _ as *mut std::ffi::c_void,
        ];
        execute_cuda_kernel_async(module, "add_one", &params, 1, 256).await.unwrap();
        let result = data_gpu.to_vec().unwrap();
        assert_eq!(result, vec![2.0, 3.0, 4.0, 5.0]);
    }

    #[tokio::test]
    async fn test_matrix_multiply_large() {
        let size = 1000;
        let a = vec![1.0; size * size];
        let b = vec![2.0; size * size];
        let result = matrix_multiply(&a, &b, size as i32, size as i32, size as i32).await.unwrap();
        assert_eq!(result.len(), size * size);
        assert!(result.iter().all(|&x| (x - (size as f32 * 2.0)).abs() < 1e-6));
    }
}