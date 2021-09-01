mod data_model;
mod module_instance_details;

use data_model::*;
use module_instance_details::*;

#[derive(Debug, structopt::StructOpt)]
#[structopt(about = env!("CARGO_PKG_DESCRIPTION"))]
struct Options
{
	/// Path to the json profile file
	#[structopt(parse(from_os_str))]
	profile_file: std::path::PathBuf,
	/// Sets the kernels to be considered (all kernels if the option is absent)
	#[structopt(short, long)]
	kernels: Option<Vec<String>>,
	/// Expands the module instance section to show a more fine-grain breakdown
	#[structopt(short, long)]
	expand: bool,
}

fn main() -> anyhow::Result<()>
{
	use structopt::StructOpt as _;

	let options = Options::from_args();

	let file_content = std::fs::read_to_string(&options.profile_file)?;
	let profile: Profile = serde_json::from_str(&file_content)?;

	println!("{}: {} (aocx: {})",
		profile.json_type,
		profile.versions.profiler_json_version,
		profile.versions.aocx_version);

	print_boards(&profile);

	print_run_information(&profile);
	print_external_memory(&profile, &options);

	print_global_memory_module_instances(&profile, &options);
	print_local_memory_module_instances(&profile, &options);
	print_channel_module_instances(&profile, &options);
	print_loop_module_instances(&profile, &options);

	Ok(())
}

fn print_boards(profile: &Profile)
{
	let boards = profile.boards.nodes.iter()
		.filter_map(|node|
			match node
			{
				Node::Board(board) => Some(board),
				_ => None
			})
		.map(|board|
			{
				let global_memories = board.children.iter()
					.filter_map(|child|
						match child
						{
							Child::GlobalMemory(global_memory) => Some(global_memory),
							_ => None
						});

				(&board.board_type, global_memories)
			});

	println!("Boards:");

	for (board_type, global_memories) in boards
	{
		println!("\tType: {}", board_type);
		println!("\tGlobal memory:");

		for global_memory in global_memories
		{
			println!("\t\tMemory {}:", global_memory.global_memory_name);
			println!("\t\t\tMaximum theoretical global bandwidth: {} MB/s",
				global_memory.maximum_theoretical_global_memory_bandwidth);
			println!("\t\t\tMaximum burst: {}", global_memory.maximum_burst_count);
		}
	}
}

fn print_run_information(profile: &Profile)
{
	let run_informations = profile.run_information.nodes.iter()
		.filter_map(|node|
			match node
			{
				Node::RunInformation(run_information) => Some(run_information),
				_ => None
			});

	println!("Run information:");

	for run_information in run_informations
	{
		println!("\tFmax: {} MHz", run_information.fmax);
	}
}

fn print_external_memory(profile: &Profile, options: &Options)
{
	let structured_bandwidths = profile.kernels.nodes.iter()
		.filter_map(|node|
			match node
			{
				Node::Kernel(kernel) => Some(kernel),
				_ => None
			})
		.filter(|kernel|
			options.kernels.as_ref().map(|kernels| kernels.contains(&kernel.name)).unwrap_or(true))
		.flat_map(|kernel|
			kernel.children.iter()
				.filter_map(move |child|
					match child
					{
						Child::ExternalMemory(external_memory) => Some((kernel, external_memory)),
						_ => None
					}))
		.map(|(kernel, external_memory)|
			{
				let intervals = std::iter::once(&kernel.start_time)
					.chain(kernel.sample_timestamps.iter())
					.zip(kernel.sample_timestamps.iter());

				let samples = external_memory.global_used_bandwidth.iter()
					.zip(external_memory.average_write_burst.iter())
					.zip(external_memory.average_read_burst.iter());

				let (bandwidth, write_burst, read_burst) = intervals.zip(samples)
					.fold((0., 0., 0.),
						|(bandwidth_sum, write_burst_sum, read_burst_sum),
							((start_time, end_time), ((bandwidth, write_burst), read_burst))|
						{
							let time = (end_time - start_time) as f32;

							(bandwidth_sum + time * bandwidth,
								write_burst_sum + time * write_burst,
								read_burst_sum + time * read_burst)
						});

				let time = kernel.end_time - kernel.start_time;

				((&external_memory.name, &external_memory.port), time, bandwidth,
					write_burst, read_burst)
			})
		.fold(std::collections::BTreeMap::new(),
			|mut map, ((name, port), time, bandwidth, write_burst, read_burst)|
			{
				let (ref mut time_sum,
					(ref mut bandwidth_sum, ref mut write_burst_sum, ref mut read_burst_sum)) =
						map.entry(name).or_insert_with(std::collections::BTreeMap::new)
							.entry(port).or_insert((0, (0., 0., 0.)));

				*time_sum += time;
				*bandwidth_sum += bandwidth;
				*write_burst_sum += write_burst;
				*read_burst_sum += read_burst;

				map
			});

	println!("External memory:");

	for (name, ports) in structured_bandwidths.into_iter()
	{
		println!("\tMemory {}:", name);

		for (port, (time_sum, (bandwidth_sum, write_burst_sum, read_burst_sum)))
			in ports.into_iter()
		{
			println!("\t\tPort {}:", port);
			println!("\t\t\tBandwidth: {:.2} MB/s", bandwidth_sum / time_sum as f32);
			println!("\t\t\tWrite burst: {:.2}", write_burst_sum / time_sum as f32);
			println!("\t\t\tRead burst: {:.2}", read_burst_sum / time_sum as f32);
		}
	}
}

fn print_module_instances<'a, Filter, Compute, Type>(profile: &'a Profile, options: &Options,
	filter: Filter, compute: Compute)
where
	Filter: Fn(&'a ModuleInstanceDetails) -> Option<&'a Type>,
	Compute: for<'b> Fn(&'b [(&'a data_model::Kernel, &'a Type)]) -> String,
	Type: 'a,
{
	let structured_samples = profile.kernels.nodes.iter()
		.filter_map(|node|
			match node
			{
				Node::Kernel(kernel) => Some(kernel),
				_ => None
			})
		.filter(|kernel|
			options.kernels.as_ref().map(|kernels| kernels.contains(&kernel.name)).unwrap_or(true))
		.flat_map(|kernel|
			kernel.children.iter()
				.filter_map(|child|
					match child
					{
						Child::ModuleInstance(module_instance) => Some(module_instance),
						_ => None
					})
				.filter_map(|module_instance| filter(&module_instance.module_instance_details)
					.map(|module_instance_details| (module_instance, module_instance_details)))
				.map(move |(module_instance, module_instance_details)|
					(kernel, module_instance, module_instance_details)))
		.fold(std::collections::BTreeMap::new(), |mut map, (kernel, module_instance, sample)|
			{
				map.entry(&module_instance.source_files).or_insert_with(Vec::new)
					.push((kernel, module_instance, sample));

				map
			});


	for (source_files, samples) in structured_samples.into_iter()
	{
		println!("{}:", format_file_references(source_files, 0).lines()
			.map(|line| format!("\t{}", line))
			.collect::<Vec<_>>().join("\n"));

		if options.expand
		{
			let structured_samples = samples.into_iter()
				.fold(std::collections::BTreeMap::new(),
					|mut map, (kernel, module_instance, sample)|
					{
						map.entry(&kernel.name).or_insert_with(std::collections::BTreeMap::new)
							.entry(&module_instance.name).or_insert_with(Vec::new)
								.push((kernel, sample));

						map
					});

			for (kernal_name, unrolls) in structured_samples.into_iter()
			{
				println!("\t\tKernel {}:", kernal_name);

				if unrolls.len() > 1
				{
					for (id, (_, samples)) in unrolls.into_iter().enumerate()
					{
						println!("\t\t\tInstance {}:", id + 1);

						for line in compute(&samples).lines()
						{
							println!("\t\t\t\t{}", line);
						}
					}
				}
				else if let Some((_, samples)) = unrolls.iter().next()
				{
					for line in compute(&samples).lines()
					{
						println!("\t\t\t{}", line);
					}
				}
			}
		}
		else
		{
			let samples = samples.into_iter()
				.map(|(kernel, _, sample)| (kernel, sample))
				.collect::<Vec<_>>();

			for line in compute(&samples).lines()
			{
				println!("\t\t{}", line);
			}
		}
	}
}

fn format_file_references(file_references: &[FileReference], level: usize) -> String
{
	let preamble = (level > 0)
		.then(||
			{
				let indentation = std::iter::repeat(' ').take((level - 1) * 2).collect::<String>();

				format!("\n{}⮤ ", indentation)
			})
		.unwrap_or_default();

	file_references.iter()
		.map(|file_reference|
			{
				let callsite = format_file_references(&file_reference.callsite, level + 1);

				format!("{}{} (line: {}{}){}",
					preamble,
					file_reference.file_name.clone().into_os_string().into_string().unwrap(),
					file_reference.line,
					file_reference.column_number
						.map(|column| format!(", column: {}", column))
						.unwrap_or_default(),
					callsite)
			})
		.collect::<Vec<_>>().join("\n")
}

fn print_global_memory_module_instances(profile: &Profile, options: &Options)
{
	fn filter(module_instance_details: &ModuleInstanceDetails) -> Option<&Global>
	{
		match module_instance_details
		{
			ModuleInstanceDetails::Global(ref sample) => Some(sample),
			_ => None
		}
	}

	fn compute(samples: &[(&data_model::Kernel, &Global)]) -> String
	{
		compute_occupancy(samples.iter()) + "\n"
			+ compute_stall(samples.iter()).as_ref() + "\n"
			+ compute_bandwith(samples.iter()).as_ref() + "\n"
			+ compute_effectiveness(samples.iter()).as_ref()
	}

	println!("Global memory:");

	print_module_instances(profile, options, filter, compute);
}

fn print_local_memory_module_instances(profile: &Profile, options: &Options)
{
	fn filter(module_instance_details: &ModuleInstanceDetails) -> Option<&Local>
	{
		match module_instance_details
		{
			ModuleInstanceDetails::Local(ref sample) => Some(sample),
			_ => None
		}
	}

	fn compute(samples: &[(&data_model::Kernel, &Local)]) -> String
	{
		compute_occupancy(samples.iter()) + "\n"
			+ compute_stall(samples.iter()).as_ref() + "\n"
	}

	println!("Local memory:");

	print_module_instances(profile, options, filter, compute);
}

fn print_channel_module_instances(profile: &Profile, options: &Options)
{
	fn filter(module_instance_details: &ModuleInstanceDetails) -> Option<&Channel>
	{
		match module_instance_details
		{
			ModuleInstanceDetails::Channel(ref sample) => Some(sample),
			_ => None
		}
	}

	fn compute(samples: &[(&data_model::Kernel, &Channel)]) -> String
	{
		compute_occupancy(samples.iter()) + "\n"
			+ compute_stall(samples.iter()).as_ref() + "\n"
			+ compute_bandwith(samples.iter()).as_ref() + "\n"
			+ compute_channel_depth(samples.iter()).as_ref()
	}

	println!("Channel:");

	print_module_instances(profile, options, filter, compute);
}

fn print_loop_module_instances(profile: &Profile, options: &Options)
{
	fn filter(module_instance_details: &ModuleInstanceDetails) -> Option<&Loop>
	{
		match module_instance_details
		{
			ModuleInstanceDetails::Loop(ref sample) => Some(sample),
			_ => None
		}
	}

	fn compute(samples: &[(&data_model::Kernel, &Loop)]) -> String
	{
		compute_occupancy(samples.iter())
	}

	println!("Loop:");

	print_module_instances(profile, options, filter, compute);
}

fn compute_occupancy<'a, Samples, Sample>(samples: Samples) -> String
where
	Samples: Iterator<Item = &'a (&'a Kernel, &'a Sample)>,
	Sample: Occupancy + 'a,
{
	let (occupancy_sum, cycles_sum) = samples
		.flat_map(|(kernel, sample)|
			sample.occupancy_samples().iter()
				.zip(kernel.total_cycles_between_samples.iter().flatten()))
		.fold((0u64, 0u64), |(occupancy_sum, cycles_sum), (&occupancy, &cycles)|
			(occupancy_sum + occupancy, cycles_sum + cycles));

	format!("Occupancy: {:.2} %", occupancy_sum as f32 / cycles_sum as f32 * 100.)
}

fn compute_stall<'a, Samples, Sample>(samples: Samples) -> String
where
	Samples: Iterator<Item = &'a (&'a Kernel, &'a Sample)>,
	Sample: Stall + 'a,
{
	let (stall_sum, idle_sum, acitvity_sum, cycles_sum) = samples
		.flat_map(|(kernel, sample)|
			sample.stall_samples().iter()
				.zip(sample.idle_samples().iter())
				.zip(sample.activity_samples().iter())
				.zip(kernel.total_cycles_between_samples.iter().flatten()))
		.fold((0u64, 0u64, 0u64, 0u64),
			|(stall_sum, idle_sum, activity_sum, cycles_sum),
				(((&stall, &idle), &activity), &cycles)|
					(stall_sum + stall,
						idle_sum + idle,
						activity_sum + activity,
						cycles_sum + cycles));

	format!("Stall: {:.2} %\nIdle: {:.2} %\nAcitivity: {:.2} %",
		stall_sum as f32 / cycles_sum as f32 * 100.,
		idle_sum as f32 / cycles_sum as f32 * 100.,
		acitvity_sum as f32 / cycles_sum as f32 * 100.)
}

fn compute_bandwith<'a, Samples, Sample>(samples: Samples) -> String
where
	Samples: Iterator<Item = &'a (&'a Kernel, &'a Sample)> + Clone,
	Sample: Bandwidth + 'a,
{
	let bandwidth_sum = samples.clone()
		.map(|(kernel, sample)|
			{
				let intervals = std::iter::once(&kernel.start_time)
					.chain(kernel.sample_timestamps.iter())
					.zip(kernel.sample_timestamps.iter());

				intervals.zip(sample.bandwidth_samples().iter())
					.map(|((start_time, end_time), bandwidth)|
						{
							(end_time - start_time) as f32 * bandwidth
						})
					.sum::<f32>()
			})
		.sum::<f32>();

	let total_runtime = samples
		.map(|(kernel, _)| (kernel.start_time, kernel.end_time))
		.collect::<std::collections::BTreeSet<_>>()
		.into_iter()
		.fold(Vec::<(u64, u64)>::new(), |mut stack, (start_time, end_time)|
			{
				let extend = stack.iter_mut().rev().next()
					.filter(|(_, latest_end_time)| start_time <= *latest_end_time);

				if let Some((_, latest_end_time)) = extend
				{
					*latest_end_time = end_time;
				}
				else
				{
					stack.push((start_time, end_time));
				}

				stack
			})
		.into_iter()
		.fold(0, |sum, (start_time, end_time)| sum + (end_time - start_time));

	format!("Bandwidth: {:.2} MB/s", bandwidth_sum / total_runtime as f32)
}

fn compute_effectiveness<'a, Samples, Sample>(samples: Samples) -> String
where
	Samples: Iterator<Item = &'a (&'a Kernel, &'a Sample)> + Clone,
	Sample: Effectiveness + Occupancy + 'a,
{
	let (number_of_samples, bandwidth_effective_sum, average_burst_size_sum) = samples.clone()
		.flat_map(|(_, sample)|
			sample.bandwidth_effective_samples().iter()
				.zip(sample.average_burst_size().iter()))
		.fold((0, 0., 0.),
			|(number_of_samples, bandwidth_effective_sum, average_burst_size_sum),
				(&bandwidth_effective, &average_burst_size)|
					(number_of_samples + 1,
						bandwidth_effective_sum + bandwidth_effective,
						average_burst_size_sum + average_burst_size));

	let (cache_hit_samples_present, cache_hit_sum, occupancy_sum) = samples
		.flat_map(|(_, sample)|
			sample.cache_hit_samples().iter()
				.zip(sample.occupancy_samples().iter()))
		.fold((false, 0u64, 0u64),
			|(cache_hit_samples_present, cache_hit_sum, occupancy_sum), (&cache_hit, &occupancy)|
				(cache_hit_samples_present || true,
					cache_hit_sum + cache_hit, occupancy_sum + occupancy));

	let output = format!("Efficiency: {:.2} %\nBurst size: {:.2}",
		bandwidth_effective_sum / number_of_samples as f32 * 100.,
		average_burst_size_sum / number_of_samples as f32);

	if cache_hit_samples_present
	{
		format!("{}\nCache hit: {:.2} %", output,
			cache_hit_sum as f32 / occupancy_sum as f32 * 100.)
	}
	else
	{
		output
	}
}

fn compute_channel_depth<'a, Samples, Sample>(samples: Samples) -> String
where
	Samples: Iterator<Item = &'a (&'a Kernel, &'a Sample)>,
	Sample: ChannelDepth + 'a,
{
	let (number_of_samples, average_channel_depth_sum, maximum_channel_depth_overall) = samples
		.flat_map(|(_, sample)|
			sample.average_channel_depth_samples().iter()
				.zip(sample.maximum_channel_depth_samples().iter()))
		.fold((0, 0., 0),
			|(number_of_samples, average_channel_depth_sum, maximum_channel_depth_overall),
				(average_channel_depth, maximum_channel_depth)|
					(number_of_samples + 1,
						average_channel_depth_sum + average_channel_depth,
						maximum_channel_depth_overall.max(*maximum_channel_depth)));

	format!("Channel Depth: {:.2} (maximum: {})",
		average_channel_depth_sum / number_of_samples as f32,
		maximum_channel_depth_overall)
}
