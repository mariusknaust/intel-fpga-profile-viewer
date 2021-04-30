use crate::data_model::*;

pub trait Interaction
{
	fn operation_type(&self) -> &OperationType;
	fn coalesced_memory(&self) -> &bool;
}

macro_rules! impl_interaction
{
	($struct_name:ident) =>
	{
		impl Interaction for $struct_name
		{
			fn operation_type(&self) -> &OperationType
			{
				&self.operation_type
			}

			fn coalesced_memory(&self) -> &bool
			{
				&self.coalesced_memory
			}
		}
	};
}

pub trait Occupancy
{
	fn occupancy_samples(&self) -> &[u64];
}

macro_rules! impl_occupancy
{
	($struct_name:ident) =>
	{
		impl Occupancy for $struct_name
		{
			fn occupancy_samples(&self) -> &[u64]
			{
				&self.occupancy_samples
			}
		}
	};
}

pub trait Stall
{
	fn stall_samples(&self) -> &[u64];
	fn idle_samples(&self) -> &[u64];
	fn activity_samples(&self) -> &[u64];
}

macro_rules! impl_stall
{
	($struct_name:ident) =>
	{
		impl Stall for $struct_name
		{
			fn stall_samples(&self) -> &[u64]
			{
				&self.stall_samples
			}

			fn idle_samples(&self) -> &[u64]
			{
				&self.idle_samples
			}

			fn activity_samples(&self) -> &[u64]
			{
				&self.activity_samples
			}
		}
	};
}

pub trait Bandwidth
{
	fn bandwidth_samples(&self) -> &[f32];
}

macro_rules! impl_bandwidth
{
	($struct_name:ident) =>
	{
		impl Bandwidth for $struct_name
		{
			fn bandwidth_samples(&self) -> &[f32]
			{
				&self.bandwidth_samples
			}
		}
	};
}

pub trait Effectiveness
{
	fn bandwidth_effective_samples(&self) -> &[f32];
	fn cache_hit_samples(&self) -> &[f32];
	fn average_burst_size(&self) -> &[f32];
}

macro_rules! impl_effectivness
{
	($struct_name:ident) =>
	{
		impl Effectiveness for $struct_name
		{
			fn bandwidth_effective_samples(&self) -> &[f32]
			{
				&self.bandwidth_effective_samples
			}

			fn cache_hit_samples(&self) -> &[f32]
			{
				&self.cache_hit_samples
			}

			fn average_burst_size(&self) -> &[f32]
			{
				&self.average_burst_size
			}
		}
	};
}

pub trait ChannelDepth
{
	fn average_channel_depth_samples(&self) -> &[f32];
	fn maximum_channel_depth_samples(&self) -> &[u32];
}

macro_rules! impl_channel_depth
{
	($struct_name:ident) =>
	{
		impl ChannelDepth for $struct_name
		{
			fn average_channel_depth_samples(&self) -> &[f32]
			{
				&self.average_channel_depth_samples
			}

			fn maximum_channel_depth_samples(&self) -> &[u32]
			{
				&self.maximum_channel_depth_samples
			}
		}
	};
}

impl_interaction!(Global);
impl_occupancy!(Global);
impl_stall!(Global);
impl_bandwidth!(Global);
impl_effectivness!(Global);

impl_interaction!(Local);
impl_occupancy!(Local);
impl_stall!(Local);

impl_interaction!(Channel);
impl_occupancy!(Channel);
impl_stall!(Channel);
impl_bandwidth!(Channel);
impl_channel_depth!(Channel);

impl_occupancy!(Loop);
