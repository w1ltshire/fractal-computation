#[allow(dead_code)]
pub struct Gpu {
	device: wgpu::Device,
	queue: wgpu::Queue,
	pipeline: wgpu::ComputePipeline,
}

impl Gpu {
	#[allow(dead_code)]
	pub async fn new() -> Result<Gpu, Box<dyn std::error::Error>> {
		let instance = wgpu::Instance::new(&Default::default());
		let adapter = instance.request_adapter(&Default::default()).await?;
		let (device, queue) = adapter.request_device(&Default::default()).await?;
		let shader = device.create_shader_module(wgpu::include_wgsl!("compute.wgsl"));

		let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
			label: Some("compute_pipeline"),
			layout: None,
			module: &shader,
			entry_point: Some("main"),
			compilation_options: Default::default(),
			cache: Default::default(),
		});

		Ok(Gpu { device, queue, pipeline })
	}
}

