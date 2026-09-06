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

pub(super) struct ImageAllocation {
    device: ash::Device,
    image: vk::Image,
    memory: vk::DeviceMemory,
    allocation_size: vk::DeviceSize,
}

impl ImageAllocation {
    pub(super) fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        extent: vk::Extent3D,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
    ) -> Result<Self, B0GpuContentError> {
        Self::new_layered(instance, physical_device, device, extent, format, usage, 1)
    }

    /// Scene look L2 (plan `look/02`): a 2D image with `array_layers`
    /// layers (the cascaded shadow map).
    pub(super) fn new_layered(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        extent: vk::Extent3D,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        array_layers: u32,
    ) -> Result<Self, B0GpuContentError> {
        Self::new_with_levels(
            instance,
            physical_device,
            device,
            extent,
            format,
            usage,
            array_layers,
            1,
        )
    }

    /// Scene look L5 (plan `look/05`): a 2D image with `mip_levels` levels.
    pub(super) fn new_mipped(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        extent: vk::Extent3D,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        mip_levels: u32,
    ) -> Result<Self, B0GpuContentError> {
        Self::new_with_levels(
            instance,
            physical_device,
            device,
            extent,
            format,
            usage,
            1,
            mip_levels,
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Vulkan ownership inputs are explicit at the private adapter boundary"
    )]
    fn new_with_levels(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        extent: vk::Extent3D,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        array_layers: u32,
        mip_levels: u32,
    ) -> Result<Self, B0GpuContentError> {
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(extent)
            .mip_levels(mip_levels)
            .array_layers(array_layers)
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

    pub(super) const fn allocation_size(&self) -> vk::DeviceSize {
        self.allocation_size
    }

    pub(super) const fn image(&self) -> vk::Image {
        self.image
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
    /// Scene look L5 (plan `look/05`): the image in the texture's own
    /// format with `mip_levels` levels.
    pub(super) fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        extent: vk::Extent3D,
        format: vk::Format,
        mip_levels: u32,
    ) -> Result<Self, B0GpuContentError> {
        Self::new_with_view(
            instance,
            physical_device,
            device,
            extent,
            format,
            mip_levels,
            1,
            vk::ImageViewType::TYPE_2D,
        )
    }

    /// Scene look L6a (plan `look/06a`): a material texture is a 2D array
    /// (one layer for a plain texture), viewed as such.
    pub(super) fn new_array(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        extent: vk::Extent3D,
        format: vk::Format,
        mip_levels: u32,
        array_layers: u32,
    ) -> Result<Self, B0GpuContentError> {
        Self::new_with_view(
            instance,
            physical_device,
            device,
            extent,
            format,
            mip_levels,
            array_layers,
            vk::ImageViewType::TYPE_2D_ARRAY,
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Vulkan ownership inputs are explicit at the private adapter boundary"
    )]
    fn new_with_view(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        extent: vk::Extent3D,
        format: vk::Format,
        mip_levels: u32,
        array_layers: u32,
        view_type: vk::ImageViewType,
    ) -> Result<Self, B0GpuContentError> {
        let image = ImageAllocation::new_with_levels(
            instance,
            physical_device,
            device,
            extent,
            format,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            array_layers,
            mip_levels,
        )?;
        let subresource = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(mip_levels)
            .base_array_layer(0)
            .layer_count(array_layers);
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image.image)
            .view_type(view_type)
            .format(format)
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
    /// Whether the image also carries sampled usage (the water pass reads
    /// the scene depth); false when the format cannot be sampled.
    sampled: bool,
}

pub(super) const SHADOW_MAP_EXTENT: u32 = 2_048;
/// Scene look L2 (plan `look/02`): the cascades of the shadow map.
pub(super) const SHADOW_CASCADES: u32 = 3;

/// One renderer-owned sampled depth target. It is deliberately independent
/// from swapchain depth so resize does not change the fixed shadow texel
/// footprint. Unsupported sampled-depth formats and bounded allocation
/// failures use the explicit no-shadow pipeline instead.
pub(super) struct ShadowMap {
    device: ash::Device,
    /// The array view the lit programs sample.
    view: vk::ImageView,
    /// One attachment view per cascade layer.
    layer_views: Vec<vk::ImageView>,
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
        let image = ImageAllocation::new_layered(
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
            SHADOW_CASCADES,
        )?;
        let subresource = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::DEPTH)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(SHADOW_CASCADES);
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image.image)
            .view_type(vk::ImageViewType::TYPE_2D_ARRAY)
            .format(format)
            .subresource_range(subresource);
        // SAFETY: image is live and uses the exact queried depth format.
        let view = unsafe { device.create_image_view(&view_info, None) }?;
        // Scene look L2: one attachment view per cascade layer.
        let mut layer_views = Vec::with_capacity(SHADOW_CASCADES as usize);
        for layer in 0..SHADOW_CASCADES {
            let layer_info = vk::ImageViewCreateInfo::default()
                .image(image.image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::DEPTH)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(layer)
                        .layer_count(1),
                );
            // SAFETY: the image is live and the layer is inside its range.
            match unsafe { device.create_image_view(&layer_info, None) } {
                Ok(layer_view) => layer_views.push(layer_view),
                Err(error) => {
                    // SAFETY: the views created so far have no dependants.
                    unsafe {
                        for layer_view in layer_views {
                            device.destroy_image_view(layer_view, None);
                        }
                        device.destroy_image_view(view, None);
                    }
                    return Err(error.into());
                }
            }
        }
        // Scene look L2: linear compare taps (hardware 2x2 PCF per tap).
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
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
                // SAFETY: the views have no dependants.
                unsafe {
                    for layer_view in layer_views {
                        device.destroy_image_view(layer_view, None);
                    }
                    device.destroy_image_view(view, None);
                }
                return Err(error.into());
            }
        };
        Ok(Self {
            device: device.clone(),
            view,
            layer_views,
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

    /// Scene look L2: the attachment view of one cascade layer.
    pub(super) fn layer_view(&self, cascade: usize) -> Option<vk::ImageView> {
        self.layer_views.get(cascade).copied()
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
            for layer_view in self.layer_views.drain(..) {
                self.device.destroy_image_view(layer_view, None);
            }
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
        // SAFETY: read-only capability query on a device of this instance.
        let properties =
            unsafe { instance.get_physical_device_format_properties(physical_device, format) };
        let sampled = properties
            .optimal_tiling_features
            .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE);
        let mut usage = vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
        if sampled {
            usage |= vk::ImageUsageFlags::SAMPLED;
        }
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
            usage,
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
            sampled,
        })
    }

    pub(crate) const fn sampled(&self) -> bool {
        self.sampled
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

        // Scene look L5: every mip level of every texture; L6a: every layer.
        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(vk::REMAINING_MIP_LEVELS)
            .base_array_layer(0)
            .layer_count(vk::REMAINING_ARRAY_LAYERS);
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
            let region: Vec<vk::BufferImageCopy> = texture
                .mips
                .iter()
                .enumerate()
                .map(|(level, mip)| {
                    vk::BufferImageCopy::default()
                        .buffer_offset(mip.staging_offset)
                        .buffer_row_length(0)
                        .buffer_image_height(0)
                        .image_subresource(
                            vk::ImageSubresourceLayers::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .mip_level(level as u32)
                                .base_array_layer(0)
                                .layer_count(texture.layers),
                        )
                        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                        .image_extent(mip.extent)
                })
                .collect();
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

/// Scene look L3 (plan `look/03`): a `1 x 1` white `R8` image, the
/// occlusion binding's placeholder.
pub(super) struct WhiteTexture {
    device: ash::Device,
    image: ImageAllocation,
    view: vk::ImageView,
    /// Scene look L5: the colour the initialisation clears to.
    color: [f32; 4],
}

impl WhiteTexture {
    pub(super) fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
    ) -> Result<Self, B0GpuContentError> {
        Self::solid(
            instance,
            physical_device,
            device,
            vk::Format::R8_UNORM,
            [1.0, 1.0, 1.0, 1.0],
        )
    }

    /// Scene look L5: a `1 x 1` image of `format` cleared to `color`.
    pub(super) fn solid(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        format: vk::Format,
        color: [f32; 4],
    ) -> Result<Self, B0GpuContentError> {
        let image = ImageAllocation::new(
            instance,
            physical_device,
            device,
            vk::Extent3D {
                width: 1,
                height: 1,
                depth: 1,
            },
            format,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        )?;
        // Scene look L6a: a one-layer array view, as every material texture.
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image.image())
            .view_type(vk::ImageViewType::TYPE_2D_ARRAY)
            .format(format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );
        // SAFETY: the image is live and uses this exact format.
        let view = unsafe { device.create_image_view(&view_info, None) }?;
        Ok(Self {
            device: device.clone(),
            image,
            view,
            color,
        })
    }

    pub(super) const fn image(&self) -> vk::Image {
        self.image.image()
    }

    pub(super) const fn color(&self) -> [f32; 4] {
        self.color
    }

    pub(super) const fn view(&self) -> vk::ImageView {
        self.view
    }
}

impl Drop for WhiteTexture {
    fn drop(&mut self) {
        // SAFETY: the owner waits for device idle before dropping.
        unsafe { self.device.destroy_image_view(self.view, None) };
    }
}

/// Clears the white texture to one and leaves it in shader-read layout.
pub(super) fn initialize_white_texture(
    device: &ash::Device,
    queue: vk::Queue,
    queue_family_index: u32,
    white: &WhiteTexture,
) -> Result<(), B0GpuContentError> {
    let pool_info = vk::CommandPoolCreateInfo::default()
        .queue_family_index(queue_family_index)
        .flags(vk::CommandPoolCreateFlags::TRANSIENT);
    // SAFETY: a transient pool on the live device for one submission.
    let pool = unsafe { device.create_command_pool(&pool_info, None) }?;
    let result = (|| {
        let allocation_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        // SAFETY: the pool is live; one primary buffer is recorded and
        // submitted once.
        let command = unsafe { device.allocate_command_buffers(&allocation_info) }?[0];
        let begin = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        let subresource = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let to_clear = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::NONE)
            .src_access_mask(vk::AccessFlags2::NONE)
            .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
            .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .image(white.image())
            .subresource_range(subresource)];
        let to_sampled = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
            .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::FRAGMENT_SHADER)
            .dst_access_mask(vk::AccessFlags2::SHADER_SAMPLED_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image(white.image())
            .subresource_range(subresource)];
        let white_value = vk::ClearColorValue {
            float32: white.color(),
        };
        let ranges = [subresource];
        // SAFETY: the buffer records one clear between two barriers on a
        // live image and is submitted once, waited on, then freed with its
        // pool.
        unsafe {
            device.begin_command_buffer(command, &begin)?;
            device.cmd_pipeline_barrier2(
                command,
                &vk::DependencyInfo::default().image_memory_barriers(&to_clear),
            );
            device.cmd_clear_color_image(
                command,
                white.image(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &white_value,
                &ranges,
            );
            device.cmd_pipeline_barrier2(
                command,
                &vk::DependencyInfo::default().image_memory_barriers(&to_sampled),
            );
            device.end_command_buffer(command)?;
            let commands = [command];
            let submits = [vk::SubmitInfo::default().command_buffers(&commands)];
            device.queue_submit(queue, &submits, vk::Fence::null())?;
            device.queue_wait_idle(queue)?;
        }
        Ok(())
    })();
    // SAFETY: the queue idled; the pool and its buffer have no users.
    unsafe { device.destroy_command_pool(pool, None) };
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
    /// Scene look L5: the set of each material (its maps or placeholders).
    pub(super) material_sets: BTreeMap<AssetRevisionRefV1, vk::DescriptorSet>,
    pub(super) shadow_set: Option<vk::DescriptorSet>,
}

/// Scene look L5: which textures a material binds (`None` reads the flat
/// placeholder) and its uniform UV scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MaterialMapsV1 {
    pub(super) base_color: AssetRevisionRefV1,
    pub(super) metallic_roughness: Option<AssetRevisionRefV1>,
    pub(super) normal: Option<AssetRevisionRefV1>,
    /// Scene look L6a: the splat control map of a splat material.
    pub(super) splat_control: Option<AssetRevisionRefV1>,
    pub(super) uv_scale: f32,
}

/// Scene look L5: the flat `1 x 1` placeholders of the material bindings.
pub(super) struct MaterialPlaceholdersV1 {
    pub(super) white: WhiteTexture,
    pub(super) metallic_roughness: WhiteTexture,
    pub(super) normal: WhiteTexture,
    /// Scene look L6a: the control map of a plain material (layer 0).
    pub(super) splat_control: WhiteTexture,
}

impl DescriptorState {
    /// Scene look L3: the plain texture sampler (nearest, repeat).
    pub(super) const fn sampler(&self) -> vk::Sampler {
        self.sampler
    }

    /// Scene look L3: rewrites set 2 binding 1 (the occlusion target); a
    /// no-op without a shadow set. The device must be idle.
    pub(super) fn write_ambient_occlusion(&self, view: vk::ImageView, sampler: vk::Sampler) {
        let Some(shadow_set) = self.shadow_set else {
            return;
        };
        let info = [vk::DescriptorImageInfo::default()
            .sampler(sampler)
            .image_view(view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
        let writes = [vk::WriteDescriptorSet::default()
            .dst_set(shadow_set)
            .dst_binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&info)];
        // SAFETY: the set, view and sampler are live and no recorded frame
        // references the set (the caller idled the device).
        unsafe { self.device.update_descriptor_sets(&writes, &[]) };
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the B0 descriptor inputs are explicit at the private adapter boundary"
    )]
    pub(super) fn new(
        device: &ash::Device,
        frame_uniforms: &[BufferAllocation],
        lighting_uniforms: &[BufferAllocation],
        textures: &BTreeMap<AssetRevisionRefV1, TextureResource>,
        shadow: Option<&ShadowMap>,
        white: &WhiteTexture,
        material_maps: &BTreeMap<AssetRevisionRefV1, MaterialMapsV1>,
        placeholders: &MaterialPlaceholdersV1,
    ) -> Result<Self, B0GpuContentError> {
        if lighting_uniforms.len() != frame_uniforms.len() {
            return Err(B0GpuContentError::InvalidCatalog(
                "lighting ring must match the frame ring",
            ));
        }
        let frame_count =
            u32::try_from(frame_uniforms.len()).map_err(|_| B0GpuContentError::CountOverflow)?;
        if frame_count == 0 {
            return Err(B0GpuContentError::InvalidCatalog(
                "descriptor frame ring must be non-empty",
            ));
        }
        // Scene look L1: binding 1 is the lighting block.
        let frame_bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT),
        ];
        // Scene look L5: base colour, metallic-roughness and normal maps;
        // L6a: the splat control map.
        let texture_bindings = [0, 1, 2, 3].map(|binding| {
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        });
        let frame_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&frame_bindings);
        let texture_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&texture_bindings);
        // Scene look L3: binding 1 of the shadow set is the occlusion target.
        let shadow_bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        ];
        let shadow_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&shadow_bindings);
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
        // Scene look L5: trilinear over the mip chains, repeating.
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .min_lod(0.0)
            .max_lod(vk::LOD_CLAMP_NONE);
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

        // Scene look L5: one set per material and one per base texture,
        // three samplers each.
        let texture_count =
            u32::try_from(textures.len()).map_err(|_| B0GpuContentError::CountOverflow)?;
        let material_count =
            u32::try_from(material_maps.len()).map_err(|_| B0GpuContentError::CountOverflow)?;
        let material_set_count = texture_count
            .checked_add(material_count)
            .ok_or(B0GpuContentError::CountOverflow)?;
        let mut pool_sizes = vec![vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: frame_count * 2,
        }];
        let shadow_count = u32::from(shadow.is_some());
        if material_set_count != 0 || shadow_count != 0 {
            pool_sizes.push(vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: material_set_count
                    .checked_mul(4)
                    .and_then(|value| value.checked_add(shadow_count * 2))
                    .ok_or(B0GpuContentError::CountOverflow)?,
            });
        }
        let max_sets = material_set_count
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
                .checked_add(material_maps.len())
                .and_then(|value| value.checked_add(frame_uniforms.len()))
                .and_then(|value| value.checked_add(usize::from(shadow.is_some())))
                .ok_or(B0GpuContentError::CountOverflow)?,
        );
        layouts.extend(std::iter::repeat_n(frame_layout, frame_uniforms.len()));
        layouts.extend(std::iter::repeat_n(
            texture_layout,
            textures.len() + material_maps.len(),
        ));
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
        for ((frame_uniform, lighting_uniform), frame_set) in frame_uniforms
            .iter()
            .zip(lighting_uniforms)
            .zip(&frame_sets)
        {
            let frame_info = [vk::DescriptorBufferInfo::default()
                .buffer(frame_uniform.buffer)
                .offset(0)
                .range(FRAME_UNIFORM_SIZE)];
            let lighting_info = [vk::DescriptorBufferInfo::default()
                .buffer(lighting_uniform.buffer)
                .offset(0)
                .range(crate::sky::LIGHTING_UNIFORM_SIZE)];
            let frame_writes = [
                vk::WriteDescriptorSet::default()
                    .dst_set(*frame_set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&frame_info),
                vk::WriteDescriptorSet::default()
                    .dst_set(*frame_set)
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&lighting_info),
            ];
            // SAFETY: destination set and uniform buffer are live; Vulkan
            // copies descriptor values during this call.
            unsafe { device.update_descriptor_sets(&frame_writes, &[]) };
        }

        let info = |view: vk::ImageView| {
            [vk::DescriptorImageInfo::default()
                .sampler(sampler)
                .image_view(view)
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)]
        };
        let write_material_set = |set: vk::DescriptorSet,
                                  base: vk::ImageView,
                                  metallic_roughness: vk::ImageView,
                                  normal: vk::ImageView,
                                  splat_control: vk::ImageView| {
            let infos = [
                info(base),
                info(metallic_roughness),
                info(normal),
                info(splat_control),
            ];
            let writes = [0, 1, 2, 3].map(|binding| {
                vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(binding)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&infos[binding as usize])
            });
            // SAFETY: exact image views, the shared sampler and the set are
            // live; descriptor payload is copied synchronously.
            unsafe { device.update_descriptor_sets(&writes, &[]) };
        };
        let mut texture_sets = BTreeMap::new();
        for ((revision, texture), descriptor_set) in textures
            .iter()
            .zip(sets.iter().copied().skip(frame_set_count))
        {
            write_material_set(
                descriptor_set,
                texture.view,
                placeholders.metallic_roughness.view(),
                placeholders.normal.view(),
                placeholders.splat_control.view(),
            );
            texture_sets.insert(*revision, descriptor_set);
        }
        // Scene look L5: one set per material with its maps.
        let mut material_sets = BTreeMap::new();
        for ((revision, maps), descriptor_set) in material_maps
            .iter()
            .zip(sets.iter().copied().skip(frame_set_count + textures.len()))
        {
            let view_of = |texture: Option<AssetRevisionRefV1>, fallback: vk::ImageView| {
                texture
                    .and_then(|texture| textures.get(&texture))
                    .map_or(fallback, |texture| texture.view)
            };
            write_material_set(
                descriptor_set,
                view_of(Some(maps.base_color), placeholders.white.view()),
                view_of(
                    maps.metallic_roughness,
                    placeholders.metallic_roughness.view(),
                ),
                view_of(maps.normal, placeholders.normal.view()),
                view_of(maps.splat_control, placeholders.splat_control.view()),
            );
            material_sets.insert(*revision, descriptor_set);
        }

        let shadow_set = shadow.map(|shadow| {
            let descriptor_set = sets[frame_set_count + textures.len() + material_maps.len()];
            let image_info = [vk::DescriptorImageInfo::default()
                .sampler(shadow.sampler())
                .image_view(shadow.view())
                .image_layout(vk::ImageLayout::DEPTH_READ_ONLY_OPTIMAL)];
            let white_info = [vk::DescriptorImageInfo::default()
                .sampler(sampler)
                .image_view(white.view())
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
            let writes = [
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_set)
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&white_info),
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&image_info),
            ];
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
            material_sets,
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
