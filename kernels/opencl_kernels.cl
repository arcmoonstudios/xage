// kernels/opencl_kernels.cl

__kernel void vector_add(__global const float* a, __global const float* b, __global float* result, int num_elements) {
    int gid = get_global_id(0);
    if (gid < num_elements) {
        result[gid] = a[gid] + b[gid];
    }
}

__kernel void matrix_multiply(__global const float* a, __global const float* b, __global float* c, int m, int n, int k) {
    int row = get_global_id(0);
    int col = get_global_id(1);

    if (row < m && col < n) {
        float sum = 0.0f;
        for (int i = 0; i < k; ++i) {
            sum += a[row * k + i] * b[i * n + col];
        }
        c[row * n + col] = sum;
    }
}

__kernel void relu_activation(__global float* data, int num_elements) {
    int gid = get_global_id(0);
    if (gid < num_elements) {
        data[gid] = max(0.0f, data[gid]);
    }
}

__kernel void softmax(__global float* input, __global float* output, int num_elements) {
    __local float max_val;
    __local float sum;

    int gid = get_global_id(0);
    float thread_max = -INFINITY;

    if (gid < num_elements) {
        thread_max = input[gid];
    }

    for (int offset = get_local_size(0) / 2; offset > 0; offset /= 2) {
        if (get_local_id(0) < offset) {
            thread_max = max(thread_max, input[gid + offset]);
        }
        barrier(CLK_LOCAL_MEM_FENCE);
    }

    if (get_local_id(0) == 0) {
        max_val = thread_max;
    }
    barrier(CLK_LOCAL_MEM_FENCE);

    float thread_sum = 0.0f;
    if (gid < num_elements) {
        output[gid] = exp(input[gid] - max_val);
        thread_sum = output[gid];
    }

    for (int offset = get_local_size(0) / 2; offset > 0; offset /= 2) {
        if (get_local_id(0) < offset) {
            thread_sum += output[gid + offset];
        }
        barrier(CLK_LOCAL_MEM_FENCE);
    }

    if (get_local_id(0) == 0) {
        sum = thread_sum;
    }
    barrier(CLK_LOCAL_MEM_FENCE);

    if (gid < num_elements) {
        output[gid] /= sum;
    }
}
