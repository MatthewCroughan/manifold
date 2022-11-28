use crate::{emitter::Emitter, keyboard::Keyboard};
use stardust_xr_molecules::fusion::client::{Client, LifeCycleHandler, LogicStepInfo};

pub struct Manifold {
	keyboard: Emitter<Keyboard>,
}
impl Manifold {
	pub fn new(client: &Client) -> Self {
		let keyboard = Emitter::new(client.get_root(), |grabbable| Keyboard::create(grabbable));
		Manifold { keyboard }
	}
	pub fn keyboard(&self) -> Keyboard {
		self.keyboard.contained.clone()
	}
}
impl LifeCycleHandler for Manifold {
	fn logic_step(&mut self, info: LogicStepInfo) {
		self.keyboard.logic_step(info);
	}
}
