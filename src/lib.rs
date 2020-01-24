extern crate rnnoise_sys;

use rnnoise_sys as sys;

pub struct DenoiseState(* mut sys::DenoiseState);

impl DenoiseState {
	pub fn new() -> Self {
		let ds = unsafe {
			sys::rnnoise_create(std::ptr::null_mut())
		};
		Self(ds)
	}
}

impl Drop for DenoiseState {
	fn drop(&mut self) {
		unsafe {
			sys::rnnoise_destroy(self.0);
		}
	}
}
