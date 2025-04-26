use anyhow::{Context as AnyhowContext, Result};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use log::{debug, error, info};
use rusb::{Context, Device, DeviceHandle, Direction, TransferType, UsbContext};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AX206Error {
    #[error("USB error: {0}")]
    UsbError(#[from] rusb::Error),

    #[error("Device not found")]
    DeviceNotFound,

    #[error("Failed to get device dimensions")]
    DimensionError,

    #[error("Invalid brightness value: {0}")]
    InvalidBrightness(u8),

    #[error("SCSI command failed: {0}")]
    ScsiCommandFailed(u8),

    #[error("Image processing error: {0}")]
    ImageError(String),
}

pub struct AX206LCD {
    device: DeviceHandle<Context>,
    pub width: u16,
    pub height: u16,
    debug: bool,
}

impl AX206LCD {
    const VID: u16 = 0x1908;
    const PID: u16 = 0x0102;
    const BLACK: (u8, u8, u8) = (0, 0, 0);

    pub fn new(debug: bool) -> Result<Self, AX206Error> {
        let context = Context::new()?;

        // Find the device
        let device = context
            .devices()?
            .iter()
            .find(|device| {
                if let Ok(desc) = device.device_descriptor() {
                    desc.vendor_id() == Self::VID && desc.product_id() == Self::PID
                } else {
                    false
                }
            })
            .ok_or(AX206Error::DeviceNotFound)?;

        let mut handle = device.open()?;

        // Check if kernel driver is active
        if handle.kernel_driver_active(0)? {
            // Detach kernel driver
            handle.detach_kernel_driver(0)?;
        }

        handle.set_active_configuration(1)?;

        // Get LCD dimensions
        let cmd = [0xcd, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut buf = [0u8; 5];

        let status = Self::wrap_scsi(&mut handle, &cmd, Direction::In, Some(&mut buf), debug)?;
        if status != 0 {
            return Err(AX206Error::ScsiCommandFailed(status));
        }

        let width = u16::from_le_bytes([buf[0], buf[1]]);
        let height = u16::from_le_bytes([buf[2], buf[3]]);

        info!("AX206LCD: got LCD dimensions: {}x{}", width, height);

        Ok(Self {
            device: handle,
            width,
            height,
            debug,
        })
    }

    pub fn set_backlight(&mut self, brightness: u8) -> Result<(), AX206Error> {
        if brightness > 7 {
            return Err(AX206Error::InvalidBrightness(brightness));
        }

        let mut cmd = [0xcd, 0x00, 0x00, 0x00, 0x00, 0x06, 0x01, 0x01, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        cmd[9] = brightness;

        let status = Self::wrap_scsi(&mut self.device, &cmd, Direction::Out, None, self.debug)?;
        if status != 0 {
            return Err(AX206Error::ScsiCommandFailed(status));
        }

        Ok(())
    }

    pub fn clear(&mut self, color: (u8, u8, u8)) -> Result<(), AX206Error> {
        // Convert RGB to RGB565
        let (r, g, b) = color;
        let rgb565 = [(((r & 0xf8)) | ((g & 0xe0) >> 5)), (((g & 0x1c) << 3) | ((b & 0xf8) >> 3))];

        let out_size = self.width as usize * self.height as usize * 2;
        let mut out_img = vec![0u8; out_size];

        for n in (0..out_size).step_by(2) {
            out_img[n] = rgb565[0];
            out_img[n + 1] = rgb565[1];
        }

        let mut cmd = [0xcd, 0x00, 0x00, 0x00, 0x00, 0x06, 0x12, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00];

        // Pack coordinates (0, 0, width-1, height-1)
        let x1 = 0u16.to_le_bytes();
        let y1 = 0u16.to_le_bytes();
        let x2 = (self.width - 1).to_le_bytes();
        let y2 = (self.height - 1).to_le_bytes();

        cmd[7] = x1[0];
        cmd[8] = x1[1];
        cmd[9] = y1[0];
        cmd[10] = y1[1];
        cmd[11] = x2[0];
        cmd[12] = x2[1];
        cmd[13] = y2[0];
        cmd[14] = y2[1];

        let status = Self::wrap_scsi(&mut self.device, &cmd, Direction::Out, Some(&mut out_img), self.debug)?;
        if status != 0 {
            return Err(AX206Error::ScsiCommandFailed(status));
        }

        Ok(())
    }

    pub fn draw(&mut self, image: &DynamicImage) -> Result<(), AX206Error> {
        let resized_image = self.resize_image(image);
        let flipped_image = resized_image; //resized_image.flipv(); // Vertical flip (equivalent to horizontal flip in Python)

        let width = flipped_image.width() as u16;
        let height = flipped_image.height() as u16;

        let out_size = width as usize * height as usize * 2;
        let mut out_img = vec![0u8; out_size];

        // Convert image to RGB565 format
        for (x, y, pixel) in flipped_image.pixels() {
            let n = ((y * width as u32 + x) * 2) as usize;

            // RGBA to RGB565
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];

            out_img[n] = (((r & 0xf8)) | ((g & 0xe0) >> 5));
            out_img[n + 1] = (((g & 0x1c) << 3) | ((b & 0xf8) >> 3));
        }

        let mut cmd = [0xcd, 0x00, 0x00, 0x00, 0x00, 0x06, 0x12, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00];

        // Pack coordinates (0, 0, width-1, height-1)
        let x1 = 0u16.to_le_bytes();
        let y1 = 0u16.to_le_bytes();
        let x2 = (width - 1).to_le_bytes();
        let y2 = (height - 1).to_le_bytes();

        cmd[7] = x1[0];
        cmd[8] = x1[1];
        cmd[9] = y1[0];
        cmd[10] = y1[1];
        cmd[11] = x2[0];
        cmd[12] = x2[1];
        cmd[13] = y2[0];
        cmd[14] = y2[1];

        let status = Self::wrap_scsi(&mut self.device, &cmd, Direction::Out, Some(&mut out_img), self.debug)?;
        if status != 0 {
            return Err(AX206Error::ScsiCommandFailed(status));
        }

        Ok(())
    }

    fn resize_image(&self, image: &DynamicImage) -> DynamicImage {
        let (img_width, img_height) = (image.width(), image.height());

        // Calculate resize ratio
        let x_ratio = self.width as f32 / img_width as f32;
        let y_ratio = self.height as f32 / img_height as f32;

        let (resize_width, resize_height) = if x_ratio < y_ratio {
            (self.width as u32, (img_height as f32 * x_ratio) as u32)
        } else {
            ((img_width as f32 * y_ratio) as u32, self.height as u32)
        };

        // Resize the image
        let resized = image.resize_exact(resize_width, resize_height, image::imageops::FilterType::Nearest);

        // Create a new black image with the LCD dimensions
        let mut new_image = DynamicImage::new_rgba8(self.width as u32, self.height as u32);

        // Fill with black
        for pixel in new_image.as_mut_rgba8().unwrap().pixels_mut() {
            *pixel = Rgba([0, 0, 0, 255]);
        }

        // Calculate centering position
        let x = ((self.width as u32 - resize_width) / 2) as u32;
        let y = ((self.height as u32 - resize_height) / 2) as u32;

        // Copy the resized image onto the new image
        image::imageops::overlay(&mut new_image, &resized, x as i64, y as i64);

        new_image
    }

    fn wrap_scsi(
        handle: &mut DeviceHandle<Context>,
        cmd: &[u8],
        direction: Direction,
        mut buf: Option<&mut [u8]>,
        debug: bool,
    ) -> Result<u8, AX206Error> {
        if debug {
            debug!("wrap_scsi cmd:{:?} dir:{:?}, buf:{:?}", cmd, direction, buf.as_ref().map(|b| b.len()));
        }

        // Create Command Block Wrapper (CBW)
        let mut cbw = [
            b'U', b'S', b'B', b'C',  // Signature
            0xde, 0xad, 0xbe, 0xef,  // Tag
            0x00, 0x00, 0x00, 0x00,  // Data transfer length
            0x00,                    // Flags
            0x00,                    // LUN
            0x10,                    // Command length
        ];

        cbw[14] = cmd.len() as u8;

        if let Some(buf) = buf.as_ref() {
            let len_bytes = (buf.len() as u32).to_le_bytes();
            cbw[8] = len_bytes[0];
            cbw[9] = len_bytes[1];
            cbw[10] = len_bytes[2];
            cbw[11] = len_bytes[3];
        }

        // Set direction flag
        if direction == Direction::In {
            cbw[12] = 0x80;
        }

        // Combine CBW and command
        let mut out = Vec::with_capacity(cbw.len() + cmd.len());
        out.extend_from_slice(&cbw);
        out.extend_from_slice(cmd);

        if debug {
            debug!("cmd bulk write: {:?}", out);
        }

        // Write command
        handle.write_bulk(0x01, &out, Duration::from_millis(1000))?;

        // Handle data transfer
        match direction {
            Direction::Out => {
                if let Some(buf) = buf {
                    if debug {
                        debug!("buf bulk write: {:?}", buf);
                    }
                    handle.write_bulk(0x01, buf, Duration::from_millis(3000))?;
                }
            }
            Direction::In => {
                if let Some(buf) = buf.as_mut() {
                    if debug {
                        debug!("cmd bulk reading: {}", buf.len());
                    }
                    let bytes_read = handle.read_bulk(0x81, buf, Duration::from_millis(4000))?;
                    if debug {
                        debug!("cmd bulk read: {:?}", &buf[..bytes_read]);
                    }

                    if bytes_read != buf.len() {
                        error!("cmd bulk read length mismatch. expected:{} got:{}", buf.len(), bytes_read);
                    }
                }
            }
        }

        // Get Command Status Wrapper (CSW)
        if debug {
            debug!("ack bulk reading");
        }

        let mut csw = [0u8; 13];
        let bytes_read = handle.read_bulk(0x81, &mut csw, Duration::from_millis(5000))?;

        if debug {
            debug!("ack bulk read: {:?}", &csw[..bytes_read]);
        }

        if bytes_read != 13 {
            error!("ack read length mismatch. expected:13 got:{}", bytes_read);
        }

        // Check CSW signature
        if &csw[0..4] != b"USBS" {
            error!("NO ACK. {:?}", &csw[0..4]);
        }

        Ok(csw[12]) // bCSWStatus
    }
}

impl Drop for AX206LCD {
    fn drop(&mut self) {
        // Clean up resources when the object is dropped
        if let Err(e) = self.device.release_interface(0) {
            error!("Failed to release interface: {}", e);
        }

        // Try to reattach kernel driver if it was active
        if let Err(e) = self.device.attach_kernel_driver(0) {
            error!("Failed to reattach kernel driver: {}", e);
        }
    }
}
