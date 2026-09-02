use std::collections::BTreeMap;

use ash::vk;
use next_contracts::project::AssetRevisionRefV1;

use super::{B0GpuContentError, FRAME_UNIFORM_SIZE, MINIMUM_BUFFER_SIZE, PreparedContent};

pub(crate) struct BufferAllocation {
    pub(crate) device: ash::Device,
    pub(crate) buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
    allocation_size: vk::DeviceSize,
}

impl BufferAllocation {
    pub(crate) fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        memory_properties: vk::MemoryPropertyFlags,
    ) -> Result<Self, B0GpuContentError> {
        let size = size.max(MINIMUM_BUFFER_SIZE);
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        // SAFETY: the create info contains no retained host pointers and the
        // returned buffer is owned and destroyed by this allocation.
        let buffer = unsafe { device.create_buffer(&buffer_info, None) }?;
        // SAFETY: the buffer was created by this live logical device.
        let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let memory_type_index = match memory_type_index(
            instance,
            physical_device,
            requirements.memory_type_bits,
            memory_properties,
        ) {
            Ok(index) => index,
            Err(error) => {
                // SAFETY: creation succeeded and no memory was bound.
                unsafe { device.destroy_buffer(buffer, None) };
                return Err(error);
            }
        };
        let allocation_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);
        // SAFETY: the allocation targets a supported memory type queried for
        // this physical device and retains no host pointer.
        let memory = match unsafe { device.allocate_memory(&allocation_info, None) } {
            Ok(memory) => memory,
            Err(error) => {
                // SAFETY: the unbound buffer remains owned by this device.
                unsafe { device.destroy_buffer(buffer, None) };
                return Err(error.into());
            }
        };
        // SAFETY: memory satisfies this buffer's exact requirements and is
        // bound at aligned offset zero exactly once.
        if let Err(error) = unsafe { device.bind_buffer_memory(buffer, memory, 0) } {
            // SAFETY: neither handle is used after this failed binding.
            unsafe {
                device.free_memory(memory, None);
                device.destroy_buffer(buffer, None);
            }
            return Err(error.into());
        }
        Ok(Self {
            device: device.clone(),
            buffer,
            memory,
            size,
            allocation_size: requirements.size,
        })
    }

    pub(super) const fn allocation_size(&self) -> vk::DeviceSize {
        self.allocation_size
    }

    /// Copies `bytes.len()` bytes out of a host-visible allocation. The
    /// caller must have established that no submitted work still writes it.
    pub(crate) fn read(
        &self,
        offset: vk::DeviceSize,
        bytes: &mut [u8],
    ) -> Result<(), B0GpuContentError> {
        if bytes.is_empty() {
            return Ok(());
        }
        let byte_count =
            u64::try_from(bytes.len()).map_err(|_| B0GpuContentError::CountOverflow)?;
        if offset
            .checked_add(byte_count)
            .is_none_or(|end| end > self.size)
        {
            return Err(B0GpuContentError::CountOverflow);
        }
        // SAFETY: this allocation was created HOST_VISIBLE for every caller of
        // `read`; the checked range lies within the allocation.
        let mapped = unsafe {
            self.device
                .map_memory(self.memory, offset, byte_count, vk::MemoryMapFlags::empty())
        }?;
        // SAFETY: Vulkan returned a readable mapping for `byte_count` bytes and
        // the destination slice is valid and non-overlapping.
        unsafe {
            std::ptr::copy_nonoverlapping(mapped.cast::<u8>(), bytes.as_mut_ptr(), bytes.len());
            self.device.unmap_memory(self.memory);
        }
        Ok(())
    }

    pub(crate) fn write(
        &self,
        offset: vk::DeviceSize,
        bytes: &[u8],
    ) -> Result<(), B0GpuContentError> {
        if bytes.is_empty() {
            return Ok(());
        }
        let byte_count =
            u64::try_from(bytes.len()).map_err(|_| B0GpuContentError::CountOverflow)?;
        if offset
            .checked_add(byte_count)
            .is_none_or(|end| end > self.size)
        {
            return Err(B0GpuContentError::CountOverflow);
        }
        // SAFETY: this allocation was created HOST_VISIBLE for every caller of
        // `write`; the checked range lies within the allocation.
        let mapped = unsafe {
            self.device
                .map_memory(self.memory, offset, byte_count, vk::MemoryMapFlags::empty())
        }?;
        // SAFETY: Vulkan returned a writable mapping for `byte_count` bytes
        // and the source slice is valid and non-overlapping.
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), mapped.cast::<u8>(), bytes.len());
            self.device.unmap_memory(self.memory);
        }
        Ok(())
    }
}

impl Drop for BufferAllocation {
    fn drop(&mut self) {
        // SAFETY: these handles were created together by this allocation and
        // the buffer is destroyed before its bound memory is freed.
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}

struct ImageAllocation {
    device: ash::Device,
    image: vk::Image,
    memory: vk::DeviceMemory,
    allocation_size: vk::DeviceSize,
}

impl ImageAllocation {
    fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        extent: vk::Extent3D,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
    ) -> Result<Self, B0GpuContentError> {
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);
        // SAFETY: the create info is self-contained and the image is retained
        // by this allocation until child views have been destroyed.
        let image = unsafe { device.create_image(&image_info, None) }?;
        // SAFETY: the image belongs to this device and is not concurrently
        // mutated during construction.
        let requirements = unsafe { device.get_image_memory_requirements(image) };
        let memory_type_index = match memory_type_index(
            instance,
            physical_device,
            requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        ) {
            Ok(index) => index,
            Err(error) => {
                // SAFETY: image creation succeeded and memory is not bound.
                unsafe { device.destroy_image(image, None) };
                return Err(error);
            }
        };
        let allocation_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);
        // SAFETY: allocation size and type came from this image's exact
        // requirements.
        let memory = match unsafe { device.allocate_memory(&allocation_info, None) } {
            Ok(memory) => memory,
            Err(error) => {
                // SAFETY: the image has no bound memory after allocation
                // failure.
                unsafe { device.destroy_image(image, None) };
                return Err(error.into());
            }
        };
        // SAFETY: memory meets the image requirements and is bound at the
        // required aligned offset zero exactly once.
        if let Err(error) = unsafe { device.bind_image_memory(image, memory, 0) } {
            // SAFETY: neither handle is referenced after the failed binding.
            unsafe {
                device.free_memory(memory, None);
                device.destroy_image(image, None);
            }
            return Err(error.into());
        }
        Ok(Self {
            device: device.clone(),
            image,
            memory,
            allocation_size: requirements.size,
        })
    }

    const fn allocation_size(&self) -> vk::DeviceSize {
        self.allocation_size
    }
}

impl Drop for ImageAllocation {
    fn drop(&mut self) {
        // SAFETY: the owning texture/depth resource destroys child views
        // first, then this image is destroyed before freeing bound memory.
        unsafe {
            self.device.destroy_image(self.image, None);
            self.device.free_memory(self.memory, None);
        }
    }
}

pub(super) struct TextureResource {
    device: ash::Device,
    view: vk::ImageView,
    image: ImageAllocation,
}

impl TextureResource {
    pub(super) fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        extent: vk::Extent3D,
    ) -> Result<Self, B0GpuContentError> {
        let image = ImageAllocation::new(
            instance,
            physical_device,
            device,
            extent,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        )?;
        let subresource = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image.image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_SRGB)
            .subresource_range(subresource);
        // SAFETY: image is live, format-compatible, and remains owned by this
        // resource until after the view is destroyed.
        let view = unsafe { device.create_image_view(&view_info, None) }?;
        Ok(Self {
            device: device.clone(),
            view,
            image,
        })
    }

    pub(super) const fn allocation_size(&self) -> vk::DeviceSize {
        self.image.allocation_size()
    }

    pub(super) const fn view(&self) -> vk::ImageView {
        self.view
    }

    pub(super) const fn image(&self) -> vk::Image {
        self.image.image
    }
}

/// One device-local depth target. Swapchain ownership keeps one allocation per
/// color image so a reacquired image never aliases depth work still in flight.
pub(crate) struct DepthAttachment {
    device: ash::Device,
    view: vk::ImageView,
    image: ImageAllocation,
}

pub(super) const SHADOW_MAP_EXTENT: u32 = 2_048;

/// One renderer-owned sampled depth target. It is deliberately independent
/// from swapchain depth so resize does not change the fixed shadow texel
/// footprint. Unsupported sampled-depth formats and bounded allocation
/// failures use the explicit no-shadow pipeline instead.
pub(super) struct ShadowMap {
    device: ash::Device,
    view: vk::ImageView,
    sampler: vk::Sampler,
    image: ImageAllocation,
    format: vk::Format,
}

impl ShadowMap {
    pub(super) fn try_new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
    ) -> Result<Option<Self>, B0GpuContentError> {
        for format in [vk::Format::D32_SFLOAT, vk::Format::D16_UNORM] {
            // SAFETY: the physical device belongs to this instance and this is
            // a read-only capability query.
            let properties =
                unsafe { instance.get_physical_device_format_properties(physical_device, format) };
            let required = vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT
                | vk::FormatFeatureFlags::SAMPLED_IMAGE;
            if !properties.optimal_tiling_features.contains(required) {
                continue;
            }
            match Self::new_with_format(instance, physical_device, device, format) {
                Ok(shadow) => return Ok(Some(shadow)),
                Err(B0GpuContentError::Graphics(error)) if shadow_fallback_error(error) => {
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
        Ok(None)
    }

    fn new_with_format(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        format: vk::Format,
    ) -> Result<Self, B0GpuContentError> {
        let image = ImageAllocation::new(
            instance,
            physical_device,
            device,
            vk::Extent3D {
                width: SHADOW_MAP_EXTENT,
                height: SHADOW_MAP_EXTENT,
                depth: 1,
            },
            format,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
        )?;
        let subresource = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::DEPTH)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image.image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(subresource);
        // SAFETY: image is live and uses the exact queried depth format.
        let view = unsafe { device.create_image_view(&view_info, None) }?;
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::NEAREST)
            .min_filter(vk::Filter::NEAREST)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .compare_enable(true)
            .compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
            .min_lod(0.0)
            .max_lod(0.0);
        // SAFETY: the compare sampler uses core, non-anisotropic features.
        let sampler = match unsafe { device.create_sampler(&sampler_info, None) } {
            Ok(sampler) => sampler,
            Err(error) => {
                // SAFETY: the view has no descriptors or submissions yet.
                unsafe { device.destroy_image_view(view, None) };
                return Err(error.into());
            }
        };
        Ok(Self {
            device: device.clone(),
            view,
            sampler,
            image,
            format,
        })
    }

    pub(super) const fn image(&self) -> vk::Image {
        self.image.image
    }

    pub(super) const fn view(&self) -> vk::ImageView {
        self.view
    }

    pub(super) const fn sampler(&self) -> vk::Sampler {
        self.sampler
    }

    pub(super) const fn format(&self) -> vk::Format {
        self.format
    }

    pub(super) const fn allocation_size(&self) -> vk::DeviceSize {
        self.image.allocation_size()
    }
}

impl Drop for ShadowMap {
    fn drop(&mut self) {
        // SAFETY: both child handles are idle before content teardown and are
        // destroyed before the backing image allocation.
        unsafe {
            self.device.destroy_sampler(self.sampler, None);
            self.device.destroy_image_view(self.view, None);
        }
    }
}

const fn shadow_fallback_error(error: vk::Result) -> bool {
    matches!(
        error,
        vk::Result::ERROR_FORMAT_NOT_SUPPORTED
            | vk::Result::ERROR_FEATURE_NOT_PRESENT
            | vk::Result::ERROR_OUT_OF_DEVICE_MEMORY
            | vk::Result::ERROR_OUT_OF_HOST_MEMORY
    )
}

impl DepthAttachment {
    pub(crate) fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        format: vk::Format,
        extent: vk::Extent2D,
    ) -> Result<Self, B0GpuContentError> {
        let image = ImageAllocation::new(
            instance,
            physical_device,
            device,
            vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            },
            format,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        )?;
        let subresource = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::DEPTH)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image.image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(subresource);
        // SAFETY: the image is live, was created with this exact depth format,
        // and remains owned until after the view is destroyed.
        let view = unsafe { device.create_image_view(&view_info, None) }?;
        Ok(Self {
            device: device.clone(),
            view,
            image,
        })
    }

    pub(crate) fn image(&self) -> vk::Image {
        self.image.image
    }

    pub(crate) fn view(&self) -> vk::ImageView {
        self.view
    }

    pub(crate) const fn allocation_size(&self) -> vk::DeviceSize {
        self.image.allocation_size()
    }
}

impl Drop for DepthAttachment {
    fn drop(&mut self) {
        // SAFETY: the view belongs to this device and is destroyed before the
        // backing image allocation is dropped automatically.
        unsafe { self.device.destroy_image_view(self.view, None) };
    }
}

impl Drop for TextureResource {
    fn drop(&mut self) {
        // SAFETY: the view belongs to this device and is destroyed before the
        // `image` field is dropped.
        unsafe { self.device.destroy_image_view(self.view, None) };
    }
}

fn memory_type_index(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    type_bits: u32,
    required: vk::MemoryPropertyFlags,
) -> Result<u32, B0GpuContentError> {
    // SAFETY: the physical-device handle was enumerated from this instance and
    // the query only returns value data.
    let properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
    properties
        .memory_types
        .iter()
        .enumerate()
        .take(properties.memory_type_count as usize)
        .find(|(index, memory_type)| {
            type_bits & (1_u32 << index) != 0 && memory_type.property_flags.contains(required)
        })
        .map(|(index, _)| index as u32)
        .ok_or(B0GpuContentError::MemoryTypeUnavailable)
}

#[allow(
    clippy::too_many_arguments,
    reason = "the one-shot upload explicitly names every owned transfer resource"
)]
pub(super) fn upload_content(
    device: &ash::Device,
    queue: vk::Queue,
    queue_family_index: u32,
    staging: &BufferAllocation,
    geometry: &BufferAllocation,
    indirect: &BufferAllocation,
    textures: &BTreeMap<AssetRevisionRefV1, TextureResource>,
    prepared: &PreparedContent,
) -> Result<(), B0GpuContentError> {
    let pool_info = vk::CommandPoolCreateInfo::default()
        .queue_family_index(queue_family_index)
        .flags(vk::CommandPoolCreateFlags::TRANSIENT);
    // SAFETY: the queue family belongs to this live logical device.
    let command_pool = unsafe { device.create_command_pool(&pool_info, None) }?;
    let mut upload_completion_known = true;
    let result = (|| {
        let allocation_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        // SAFETY: the transient pool is live and owned for the whole upload.
        let command_buffer = unsafe { device.allocate_command_buffers(&allocation_info) }?[0];
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        // SAFETY: the newly allocated primary command buffer is in the initial
        // state and is recorded exactly once before submission.
        unsafe { device.begin_command_buffer(command_buffer, &begin_info) }?;

        if prepared.geometry_payload_size != 0 {
            let regions = [vk::BufferCopy::default()
                .src_offset(prepared.geometry_staging_offset)
                .dst_offset(0)
                .size(prepared.geometry_payload_size)];
            // SAFETY: source and destination buffers are live, non-overlapping
            // allocations and the region lies within their checked sizes.
            unsafe {
                device.cmd_copy_buffer(command_buffer, staging.buffer, geometry.buffer, &regions);
            }
        }
        if prepared.indirect_payload_size != 0 {
            let regions = [vk::BufferCopy::default()
                .src_offset(prepared.indirect_staging_offset)
                .dst_offset(0)
                .size(prepared.indirect_payload_size)];
            // SAFETY: both buffers are live and the initialized indirect bytes
            // fit the device-local destination.
            unsafe {
                device.cmd_copy_buffer(command_buffer, staging.buffer, indirect.buffer, &regions);
            }
        }
        let mut buffer_barriers = Vec::with_capacity(2);
        if prepared.geometry_payload_size != 0 {
            buffer_barriers.push(
                vk::BufferMemoryBarrier2::default()
                    .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                    .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                    .dst_stage_mask(vk::PipelineStageFlags2::VERTEX_INPUT)
                    .dst_access_mask(
                        vk::AccessFlags2::VERTEX_ATTRIBUTE_READ | vk::AccessFlags2::INDEX_READ,
                    )
                    .buffer(geometry.buffer)
                    .offset(0)
                    .size(prepared.geometry_payload_size),
            );
        }
        if prepared.indirect_payload_size != 0 {
            buffer_barriers.push(
                vk::BufferMemoryBarrier2::default()
                    .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                    .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                    .dst_stage_mask(vk::PipelineStageFlags2::DRAW_INDIRECT)
                    .dst_access_mask(vk::AccessFlags2::INDIRECT_COMMAND_READ)
                    .buffer(indirect.buffer)
                    .offset(0)
                    .size(prepared.indirect_payload_size),
            );
        }
        if !buffer_barriers.is_empty() {
            let dependency = vk::DependencyInfo::default().buffer_memory_barriers(&buffer_barriers);
            // SAFETY: the transfer copies above initialize the complete buffer
            // ranges, and this barrier makes them visible to subsequent vertex,
            // index, and indirect reads on the same queue.
            unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency) };
        }

        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let to_transfer = prepared
            .textures
            .iter()
            .map(|texture| {
                let resource = textures
                    .get(&texture.revision)
                    .ok_or(B0GpuContentError::ResourceMissing("upload texture image"))?;
                Ok(vk::ImageMemoryBarrier2::default()
                    .src_stage_mask(vk::PipelineStageFlags2::NONE)
                    .src_access_mask(vk::AccessFlags2::NONE)
                    .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                    .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .image(resource.image.image)
                    .subresource_range(subresource_range))
            })
            .collect::<Result<Vec<_>, B0GpuContentError>>()?;
        if !to_transfer.is_empty() {
            let dependency = vk::DependencyInfo::default().image_memory_barriers(&to_transfer);
            // SAFETY: every image starts in UNDEFINED and each barrier targets
            // its sole color mip and layer in this recording command buffer.
            unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency) };
        }
        for texture in &prepared.textures {
            let resource = textures
                .get(&texture.revision)
                .ok_or(B0GpuContentError::ResourceMissing("upload texture image"))?;
            let region = [vk::BufferImageCopy::default()
                .buffer_offset(texture.staging_offset)
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_subresource(
                    vk::ImageSubresourceLayers::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .mip_level(0)
                        .base_array_layer(0)
                        .layer_count(1),
                )
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(texture.extent)];
            // SAFETY: the source offset is four-byte aligned, image extent
            // matches the exact RGBA8 mip payload, and layout is TRANSFER_DST.
            unsafe {
                device.cmd_copy_buffer_to_image(
                    command_buffer,
                    staging.buffer,
                    resource.image.image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &region,
                );
            }
        }
        let to_shader = prepared
            .textures
            .iter()
            .map(|texture| {
                let resource = textures
                    .get(&texture.revision)
                    .ok_or(B0GpuContentError::ResourceMissing("uploaded texture image"))?;
                Ok(vk::ImageMemoryBarrier2::default()
                    .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                    .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                    .dst_stage_mask(vk::PipelineStageFlags2::FRAGMENT_SHADER)
                    .dst_access_mask(vk::AccessFlags2::SHADER_SAMPLED_READ)
                    .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image(resource.image.image)
                    .subresource_range(subresource_range))
            })
            .collect::<Result<Vec<_>, B0GpuContentError>>()?;
        if !to_shader.is_empty() {
            let dependency = vk::DependencyInfo::default().image_memory_barriers(&to_shader);
            // SAFETY: every listed image was populated earlier in this command
            // buffer and remains alive through queue completion.
            unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency) };
        }
        // SAFETY: recording is active and all commands reference live resources.
        unsafe { device.end_command_buffer(command_buffer) }?;

        let command_buffers = [command_buffer];
        let submit_infos = [vk::SubmitInfo::default().command_buffers(&command_buffers)];
        let fence_info = vk::FenceCreateInfo::default();
        // SAFETY: fence creation retains no host pointer.
        let fence = unsafe { device.create_fence(&fence_info, None) }?;
        // SAFETY: command buffer is executable, queue belongs to its family,
        // and fence is unsignaled and used for this one submission.
        let submit_result = unsafe { device.queue_submit(queue, &submit_infos, fence) };
        let wait_result: Result<(), B0GpuContentError> = match submit_result {
            Err(error) => Err(error.into()),
            Ok(()) => {
                // SAFETY: the fence belongs to this device and will remain live
                // until this unbounded setup-time wait returns.
                match unsafe { device.wait_for_fences(&[fence], true, u64::MAX) } {
                    Ok(()) => Ok(()),
                    Err(vk::Result::ERROR_DEVICE_LOST) => Err(vk::Result::ERROR_DEVICE_LOST.into()),
                    Err(wait_error) => {
                        // SAFETY: this fallback waits for every submission on
                        // the same queue before any upload object is destroyed.
                        match unsafe { device.queue_wait_idle(queue) } {
                            Ok(()) => Err(wait_error.into()),
                            Err(vk::Result::ERROR_DEVICE_LOST) => {
                                Err(vk::Result::ERROR_DEVICE_LOST.into())
                            }
                            Err(idle_error) => {
                                upload_completion_known = false;
                                Err(B0GpuContentError::UploadCompletionUnknown(idle_error))
                            }
                        }
                    }
                }
            }
        };
        if upload_completion_known {
            // SAFETY: submission either never started, has completed, or the
            // device was lost; the fence is no longer in use.
            unsafe { device.destroy_fence(fence, None) };
        }
        wait_result?;
        Ok(())
    })();
    if upload_completion_known {
        // SAFETY: successful upload waits for completion; on an error, work
        // either was not submitted, was made idle by the fallback, or the
        // device was lost. Unknown pending work deliberately leaks this pool.
        unsafe { device.destroy_command_pool(command_pool, None) };
    }
    result
}

pub(super) struct DescriptorState {
    device: ash::Device,
    pool: vk::DescriptorPool,
    pub(super) frame_layout: vk::DescriptorSetLayout,
    pub(super) texture_layout: vk::DescriptorSetLayout,
    pub(super) shadow_layout: vk::DescriptorSetLayout,
    sampler: vk::Sampler,
    pub(super) frame_sets: Vec<vk::DescriptorSet>,
    pub(super) texture_sets: BTreeMap<AssetRevisionRefV1, vk::DescriptorSet>,
    pub(super) shadow_set: Option<vk::DescriptorSet>,
}

impl DescriptorState {
    pub(super) fn new(
        device: &ash::Device,
        frame_uniforms: &[BufferAllocation],
        textures: &BTreeMap<AssetRevisionRefV1, TextureResource>,
        shadow: Option<&ShadowMap>,
    ) -> Result<Self, B0GpuContentError> {
        let frame_count =
            u32::try_from(frame_uniforms.len()).map_err(|_| B0GpuContentError::CountOverflow)?;
        if frame_count == 0 {
            return Err(B0GpuContentError::InvalidCatalog(
                "descriptor frame ring must be non-empty",
            ));
        }
        let frame_bindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)];
        let texture_bindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)];
        let frame_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&frame_bindings);
        let texture_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&texture_bindings);
        let shadow_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&texture_bindings);
        // SAFETY: bindings are closed B0 values and no pointer is retained.
        let frame_layout =
            unsafe { device.create_descriptor_set_layout(&frame_layout_info, None) }?;
        // SAFETY: same ownership conditions as the frame layout.
        let texture_layout =
            match unsafe { device.create_descriptor_set_layout(&texture_layout_info, None) } {
                Ok(layout) => layout,
                Err(error) => {
                    // SAFETY: frame layout creation succeeded and it has no
                    // dependent pipeline or descriptor sets yet.
                    unsafe { device.destroy_descriptor_set_layout(frame_layout, None) };
                    return Err(error.into());
                }
            };
        // SAFETY: the sampled-depth binding has the same closed descriptor
        // shape as the texture set and retains no host pointers.
        let shadow_layout =
            match unsafe { device.create_descriptor_set_layout(&shadow_layout_info, None) } {
                Ok(layout) => layout,
                Err(error) => {
                    // SAFETY: neither prior layout has dependants yet.
                    unsafe {
                        device.destroy_descriptor_set_layout(texture_layout, None);
                        device.destroy_descriptor_set_layout(frame_layout, None);
                    }
                    return Err(error.into());
                }
            };
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::NEAREST)
            .min_filter(vk::Filter::NEAREST)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .min_lod(0.0)
            .max_lod(0.0);
        // SAFETY: sampler uses only core, non-anisotropic B0 features.
        let sampler = match unsafe { device.create_sampler(&sampler_info, None) } {
            Ok(sampler) => sampler,
            Err(error) => {
                // SAFETY: layouts have no dependants after sampler failure.
                unsafe {
                    device.destroy_descriptor_set_layout(shadow_layout, None);
                    device.destroy_descriptor_set_layout(texture_layout, None);
                    device.destroy_descriptor_set_layout(frame_layout, None);
                }
                return Err(error.into());
            }
        };

        let texture_count =
            u32::try_from(textures.len()).map_err(|_| B0GpuContentError::CountOverflow)?;
        let mut pool_sizes = vec![vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: frame_count,
        }];
        let shadow_count = u32::from(shadow.is_some());
        if texture_count != 0 || shadow_count != 0 {
            pool_sizes.push(vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: texture_count
                    .checked_add(shadow_count)
                    .ok_or(B0GpuContentError::CountOverflow)?,
            });
        }
        let max_sets = texture_count
            .checked_add(frame_count)
            .and_then(|value| value.checked_add(shadow_count))
            .ok_or(B0GpuContentError::CountOverflow)?;
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(max_sets)
            .pool_sizes(&pool_sizes);
        // SAFETY: pool sizes exactly cover the immutable B0 descriptor sets.
        let pool = match unsafe { device.create_descriptor_pool(&pool_info, None) } {
            Ok(pool) => pool,
            Err(error) => {
                // SAFETY: no sets or pipelines depend on these objects.
                unsafe {
                    device.destroy_sampler(sampler, None);
                    device.destroy_descriptor_set_layout(shadow_layout, None);
                    device.destroy_descriptor_set_layout(texture_layout, None);
                    device.destroy_descriptor_set_layout(frame_layout, None);
                }
                return Err(error.into());
            }
        };

        let mut layouts = Vec::with_capacity(
            textures
                .len()
                .checked_add(frame_uniforms.len())
                .and_then(|value| value.checked_add(usize::from(shadow.is_some())))
                .ok_or(B0GpuContentError::CountOverflow)?,
        );
        layouts.extend(std::iter::repeat_n(frame_layout, frame_uniforms.len()));
        layouts.extend(std::iter::repeat_n(texture_layout, textures.len()));
        if shadow.is_some() {
            layouts.push(shadow_layout);
        }
        let allocation_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool)
            .set_layouts(&layouts);
        // SAFETY: pool and every requested layout are live and owned by the
        // same device.
        let sets = match unsafe { device.allocate_descriptor_sets(&allocation_info) } {
            Ok(sets) => sets,
            Err(error) => {
                // SAFETY: failed allocation leaves no externally owned sets.
                unsafe {
                    device.destroy_descriptor_pool(pool, None);
                    device.destroy_sampler(sampler, None);
                    device.destroy_descriptor_set_layout(shadow_layout, None);
                    device.destroy_descriptor_set_layout(texture_layout, None);
                    device.destroy_descriptor_set_layout(frame_layout, None);
                }
                return Err(error.into());
            }
        };
        let frame_set_count = frame_uniforms.len();
        let frame_sets = sets[..frame_set_count].to_vec();
        for (frame_uniform, frame_set) in frame_uniforms.iter().zip(&frame_sets) {
            let frame_info = [vk::DescriptorBufferInfo::default()
                .buffer(frame_uniform.buffer)
                .offset(0)
                .range(FRAME_UNIFORM_SIZE)];
            let frame_writes = [vk::WriteDescriptorSet::default()
                .dst_set(*frame_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&frame_info)];
            // SAFETY: destination set and uniform buffer are live; Vulkan
            // copies descriptor values during this call.
            unsafe { device.update_descriptor_sets(&frame_writes, &[]) };
        }

        let mut texture_sets = BTreeMap::new();
        for ((revision, texture), descriptor_set) in textures
            .iter()
            .zip(sets.iter().copied().skip(frame_set_count))
        {
            let image_info = [vk::DescriptorImageInfo::default()
                .sampler(sampler)
                .image_view(texture.view)
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
            let writes = [vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_info)];
            // SAFETY: exact image view, shared sampler, and set are live;
            // descriptor payload is copied synchronously.
            unsafe { device.update_descriptor_sets(&writes, &[]) };
            texture_sets.insert(*revision, descriptor_set);
        }

        let shadow_set = shadow.map(|shadow| {
            let descriptor_set = sets[frame_set_count + textures.len()];
            let image_info = [vk::DescriptorImageInfo::default()
                .sampler(shadow.sampler())
                .image_view(shadow.view())
                .image_layout(vk::ImageLayout::DEPTH_READ_ONLY_OPTIMAL)];
            let writes = [vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_info)];
            // SAFETY: the sampled depth image, compare sampler and set remain
            // live for the complete descriptor-state lifetime.
            unsafe { device.update_descriptor_sets(&writes, &[]) };
            descriptor_set
        });

        Ok(Self {
            device: device.clone(),
            pool,
            frame_layout,
            texture_layout,
            shadow_layout,
            sampler,
            frame_sets,
            texture_sets,
            shadow_set,
        })
    }
}

impl Drop for DescriptorState {
    fn drop(&mut self) {
        // SAFETY: the graphics pipeline has already been destroyed. The pool
        // releases sets before their sampler and layouts are destroyed.
        unsafe {
            self.device.destroy_descriptor_pool(self.pool, None);
            self.device.destroy_sampler(self.sampler, None);
            self.device
                .destroy_descriptor_set_layout(self.shadow_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.texture_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.frame_layout, None);
        }
    }
}
