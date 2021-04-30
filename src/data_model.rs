#[derive(Debug, Clone, serde::Deserialize)]
pub struct Profile
{
	pub json_type: String,
	pub versions: Versions,
	pub kernels: Nodes,
	pub boards: Nodes,
	#[serde(rename = "memtransfers")]
	pub memory_transfers: Nodes,
	pub channels: Nodes,
	#[serde(rename = "run_info")]
	pub run_information: Nodes,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Versions
{
	pub profiler_json_version: String,
	pub aocx_version: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Nodes
{
	#[serde(deserialize_with = "empty_string_as_empty_vec")]
	pub nodes: Vec<Node>,
}

fn empty_string_as_empty_vec<'de, Deserializer, Value>(deserializer: Deserializer)
	-> Result<Vec<Value	>, Deserializer::Error>
where
	Deserializer: serde::Deserializer<'de>,
	Value: serde::Deserialize<'de>,
{
	use serde::Deserialize as _;
	use serde::de::Error as _;

	#[derive(Debug, Clone, serde::Deserialize)]
	#[serde(untagged)]
	enum StringOrVec<Value>
	{
		String(String),
		Vec(Vec<Value>),
	}

	match StringOrVec::deserialize(deserializer)?
	{
		StringOrVec::String(string) =>
		{
			match string.as_ref()
			{
				"" => Ok(vec![]),
				_ => Err(Deserializer::Error::custom(
					"non-empty string instead of array not supported"))
			}
		}
		StringOrVec::Vec(vec) => Ok(vec),
	}
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Node
{
	Kernel(Kernel),
	Board(Board),
	#[serde(rename = "memtransfers")]
	MemoryTransfers(MemoryTransfers),
	#[serde(rename = "runinfo")]
	RunInformation(RunInformation),
}

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Kernel
{
	pub name: String,
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub compute_unit: u32,
	#[serde(rename = "sourcefile")]
	pub source_file: FileReference,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub device_ids: Vec<u32>,
	#[serde(default)]
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub command_queue_ids: Vec<u32>,
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub start_time: u64,
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub end_time: u64,
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub num_samples: u32,
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub shared_counter_run_type: i32,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub sample_timestamps: Vec<u64>,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub total_cycles_between_samples: Vec<u64>,
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub is_autorun: bool,
	pub children: Vec<Child>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Board
{
	pub board_type: String,
	pub children: Vec<Child>,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MemoryTransfers
{
	pub type_transfer: String,
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub device_id: u32,
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub command_queue_id: u32,
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub start_time: u64,
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub end_time: u64,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RunInformation
{
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub fmax: f32,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Child
{
	#[serde(rename = "moduleinst")]
	ModuleInstance(ModuleInstance),
	#[serde(rename = "globalmem")]
	GlobalMemory(GlobalMemory),
	#[serde(rename = "extmem")]
	ExternalMemory(ExternalMemory),
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ModuleInstance
{
	pub name: String,
	#[serde(rename = "sourcefiles")]
	pub source_files: Vec<FileReference>,
	#[serde(rename = "module_inst_details")]
	pub module_instance_details: ModuleInstanceDetails,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GlobalMemory
{
	pub global_memory_name: String,
	#[serde(rename = "max_theoretical_globalmem_bw")]
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub maximum_theoretical_global_memory_bandwidth: f32,
	#[serde(rename = "max_burst_count")]
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub maximum_burst_count: f32,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExternalMemory
{
	pub name: String,
	pub interface: String,
	pub port: String,
	#[serde(rename = "global_used_bw")]
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub global_used_bandwidth: Vec<f32>,
	#[serde(rename = "avg_write_burst")]
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub average_write_burst: Vec<f32>,
	#[serde(rename = "avg_read_burst")]
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub average_read_burst: Vec<f32>,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize)]
pub struct FileReference
{
	#[serde(rename = "filename")]
	pub file_name: std::path::PathBuf,
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub line: u32,
	#[serde(rename = "column_num", default)]
	#[serde_as(as = "Option<serde_with::DisplayFromStr>")]
	pub column_number: Option<u32>,
	#[serde(default)]
	pub callsite: Vec<FileReference>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "mem_type")]
pub enum ModuleInstanceDetails
{
	#[serde(rename = "__global")]
	Global(Global),
	#[serde(rename = "__local")]
	Local(Local),
	#[serde(rename = "__channel")]
	Channel(Channel),
	#[serde(rename = "__loop")]
	Loop(Loop),
}

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Global
{
	pub operation_type: OperationType,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub occupancy_samples: Vec<u64>,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub stall_samples: Vec<u64>,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub idle_samples: Vec<u64>,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub activity_samples: Vec<u64>,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub bandwidth_samples: Vec<f32>,
	#[serde(rename = "bandwidth_eff_samples")]
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub bandwidth_effective_samples: Vec<f32>,
	#[serde(default)]
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub cache_hit_samples: Vec<f32>,
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub coalesced_memory: bool,
	#[serde(rename = "global_mem_name")]
	pub global_memory_name: String,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub average_burst_size: Vec<f32>,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Local
{
	pub operation_type: OperationType,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub occupancy_samples: Vec<u64>,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub stall_samples: Vec<u64>,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub idle_samples: Vec<u64>,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub activity_samples: Vec<u64>,
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub coalesced_memory: bool,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Channel
{
	pub operation_type: OperationType,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub occupancy_samples: Vec<u64>,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub stall_samples: Vec<u64>,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub idle_samples: Vec<u64>,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub average_channel_depth_samples: Vec<f32>,
	#[serde(rename = "max_channel_depth_samples")]
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub maximum_channel_depth_samples: Vec<u32>,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub activity_samples: Vec<u64>,
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub bandwidth_samples: Vec<f32>,
	#[serde_as(as = "serde_with::DisplayFromStr")]
	pub coalesced_memory: bool,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Loop
{
	#[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
	pub occupancy_samples: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationType
{
	Read,
	Write,
}
