// kernels/wgpu_shaders.wgsl

@group(0) @binding(0) var<storage, read> a: array<f32>;
@group(0) @binding(1) var<storage, read> b: array<f32>;
@group(0) @binding(2) var<storage, read_write> result: array<f32>;

@compute @workgroup_size(256)
fn vector_add(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx < arrayLength(&result)) {
        result[idx] = a[idx] + b[idx];
    }
}

@group(0) @binding(0) var<storage, read> matrix_a: array<f32>;
@group(0) @binding(1) var<storage, read> matrix_b: array<f32>;
@group(0) @binding(2) var<storage, read_write> matrix_c: array<f32>;
@group(0) @binding(3) var<uniform> dimensions: vec3<u32>;  // m, n, k

@compute @workgroup_size(16, 16)
fn matrix_multiply(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let row = global_id.y;
    let col = global_id.x;
    let m = dimensions.x;
    let n = dimensions.y;
    let k = dimensions.z;

    if (row < m && col < n) {
        var sum = 0.0;
        for (var i = 0u; i < k; i = i + 1u) {
            sum = sum + matrix_a[row * k + i] * matrix_b[i * n + col];
        }
        matrix_c[row * n + col] = sum;
    }
}

@group(0) @binding(0) var<storage, read_write> data: array<f32>;

@compute @workgroup_size(256)
fn relu_activation(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx < arrayLength(&data)) {
        data[idx] = max(0.0, data[idx]);
    }
}

@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(256)
fn softmax(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>) {
    let idx = global_id.x;
    let local_idx = local_id.x;
    let num_elements = arrayLength(&input);

    var max_val = -1e38;
    var sum = 0.0;

    // Find max value
    if (idx < num_elements) {
        max_val = input[idx];
    }
    for (var stride = 128u; stride > 0u; stride >>= 1u) {
        if (local_idx < stride) {
            max_val = max(max_val, input[idx + stride]);
        }
        workgroupBarrier();
    }

    // Calculate exp and sum
    if (idx < num_elements) {
        output[idx] = exp(input[idx] - max_val);
        sum = output[idx];
    }
    for (var stride = 128u; stride > 0u; stride >>= 1u) {
        if (local_idx < stride) {
            sum += output[idx + stride];
        }
        workgroupBarrier();
    }

    // Normalize
    if (idx < num_elements) {
        output[idx] /= sum;
    }
}
