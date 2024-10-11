// kernels/cuda_kernels.cu

extern "C" __global__ void vector_add(float* a, float* b, float* result, int num_elements) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < num_elements) {
        result[idx] = a[idx] + b[idx];
    }
}

extern "C" __global__ void matrix_multiply(float* a, float* b, float* c, int m, int n, int k) {
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

extern "C" __global__ void relu_activation(float* data, int num_elements) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < num_elements) {
        data[idx] = fmaxf(0.0f, data[idx]);
    }
}

extern "C" __global__ void softmax(float* input, float* output, int num_elements) {
    __shared__ float max_val;
    __shared__ float sum;
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    float thread_max = -INFINITY;
    if (idx < num_elements) {
        thread_max = input[idx];
    }
    for (int offset = blockDim.x / 2; offset > 0; offset >>= 1) {
        if (threadIdx.x < offset) {
            thread_max = fmaxf(thread_max, __shfl_down_sync(0xffffffff, thread_max, offset));
        }
    }
    if (threadIdx.x == 0) {
        max_val = thread_max;
    }
    __syncthreads();
    float thread_sum = 0.0f;
    if (idx < num_elements) {
        output[idx] = expf(input[idx] - max_val);
        thread_sum = output[idx];
    }
    for (int offset = blockDim.x / 2; offset > 0; offset >>= 1) {
        if (threadIdx.x < offset) {
            thread_sum += __shfl_down_sync(0xffffffff, thread_sum, offset);
        }
    }
    if (threadIdx.x == 0) {
        sum = thread_sum;
    }
    __syncthreads();
    if (idx < num_elements) {
        output[idx] /= sum;
    }
}
