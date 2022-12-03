use crate::{emitter::Emitter, keyboard::Keyboard, mouse::Mouse};
use stardust_xr_molecules::fusion::client::{Client, LifeCycleHandler, LogicStepInfo};

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
impl LifeCycleHandler for Manifold {
	fn logic_step(&mut self, info: LogicStepInfo) {
		self.mouse.logic_step(info);
		self.keyboard.logic_step(info);
	}
}
