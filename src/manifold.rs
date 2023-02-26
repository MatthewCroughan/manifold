use crate::{emitter::Emitter, keyboard::Keyboard, mouse::Mouse};
use stardust_xr_fusion::client::{Client, FrameInfo, RootHandler};

pub struct Manifold {
	keyboard: Emitter<Keyboard>,
	mouse: Emitter<Mouse>,
}
impl Manifold {
	pub fn new(client: &Client) -> Self {
		let keyboard = Emitter::new(client.get_root(), |grabbable| Keyboard::create(grabbable));
		let mouse = Emitter::new(client.get_root(), |grabbable| Mouse::create(grabbable));
		Manifold { keyboard, mouse }
	}
	pub fn keyboard(&self) -> Keyboard {
		self.keyboard.contained.clone()
	}
	pub fn mouse(&self) -> Mouse {
		self.mouse.contained.clone()
	}
}
impl RootHandler for Manifold {
	fn frame(&mut self, info: FrameInfo) {
		self.mouse.frame(info);
		self.keyboard.frame(info);
	}
}
