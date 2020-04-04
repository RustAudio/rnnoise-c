extern crate rnnoise_sys;

use rnnoise_sys as sys;

pub struct DenoiseState(* mut sys::DenoiseState);

pub const FRAME_SIZE :usize = 480;

unsafe impl Send for DenoiseState {}

impl DenoiseState {
	pub fn new() -> Self {
		let ds = unsafe {
			sys::rnnoise_create(std::ptr::null_mut())
		};
		Self(ds)
	}

	/// Performs denoising operations on `in_data` array into `out_data`
	///
	/// Panics if the passed slices are not equal to `FRAME_SIZE`.
	/// Assumes input data is mono 16 bit audio encoded at a sample rate of 48 kHz.
	///
	/// Note that the input floats should be in amplitude ranges between
	/// -32767.0 and 32767.0, instead of the more common -1.0 and 1.0
	/// boundaries. The output will be scaled in a similar fashion
	pub fn process_frame_mut(&mut self, in_data :&[f32], out_data :&mut [f32])  -> f32 {
		assert_eq!(in_data.len(), FRAME_SIZE);
		assert_eq!(out_data.len(), FRAME_SIZE);

		unsafe {
			let out_ptr = out_data.as_mut_ptr();
			let in_ptr = in_data.as_ptr();
			sys::rnnoise_process_frame(self.0, out_ptr, in_ptr)
		}
	}

	/// Performs denoising operations on `in_data` array into `out_data`
	///
	/// Panics if the passed slices are not equal to `FRAME_SIZE`.
	/// Assumes input data is mono 16 bit audio encoded at a sample rate of 48 kHz.
	///
	/// Note that the input floats should be in amplitude ranges between
	/// -32767.0 and 32767.0, instead of the more common -1.0 and 1.0
	/// boundaries. The output will be scaled in a similar fashion.
	pub fn process_frame_in_place(&mut self, data: &mut [f32])  -> f32 {
		assert_eq!(data.len(), FRAME_SIZE);

		unsafe {
			let ptr = data.as_mut_ptr();
			sys::rnnoise_process_frame(self.0, ptr, ptr)
		}
	}
}

impl Drop for DenoiseState {
	fn drop(&mut self) {
		unsafe {
			sys::rnnoise_destroy(self.0);
		}
	}
}
