struct Complex {
    re: f32,
    im: f32,
}

fn complex_add(a: Complex, b: Complex) -> Complex {
    return Complex(a.re + b.re, a.im + b.im);
}

fn complex_mult(a: Complex, b: Complex) -> Complex {
    return Complex(
        a.re * b.re - a.im * b.im,
        a.re * b.im + a.im * b.re,
    );
}

fn complex_norm_sqr(c: Complex) -> f32 {
    return c.re * c.re + c.im * c.im;
}

fn mandelbrot_color(c: Complex, upper: u32) -> u32 {
    var z: Complex = Complex(0.0, 0.0);

    for (var i: u32 = 0; i <= upper; i = i + 1) {
        z = complex_mult(z, z);
        z = complex_add(z, c);

        if (complex_norm_sqr(z) > 4.0) {
            return i;
        }
    }

    return upper;
}

@group(0) @binding(0) var<storage, read> input: array<u32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;

@compute
@workgroup_size(64)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let width: u32 = 800;
    let height: u32 = 600;

    let x = global_invocation_id.x;
    let y = global_invocation_id.y;

    if (x >= width || y >= height) {
        return;
    }

    let from_: Complex = Complex(-2.0, -1.5);
    let to: Complex = Complex(1.0, 1.5);

    let c: Complex = Complex(
        from_.re + (to.re - from_.re) * f32(x) / f32(width),
        from_.im + (to.im - from_.im) * f32(y) / f32(height),
    );

    let iter = mandelbrot_color(c, 255);

    let index = y * width + x;
    output[index] = u32(255 - iter) | (u32(255) << 24);
}