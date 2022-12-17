use color::{rgba, Rgba};
use mint::Vector3;
use parking_lot::MutexGuard;
use rustc_hash::FxHashMap;
use stardust_xr_molecules::{
	fusion::{
		client::LogicStepInfo,
		core::values::Transform,
		data::{NewReceiverInfo, PulseReceiver, PulseSender, PulseSenderHandler},
		drawable::{LinePoint, Lines},
		fields::UnknownField,
		node::NodeType,
		resource::NamespacedResource,
		spatial::Spatial,
		HandlerWrapper,
	},
	keyboard::{xkb::State, KeyboardEvent, KEYBOARD_MASK},
};
use std::sync::Arc;
use xkbcommon::xkb::Keymap;

use crate::emitter::Emittable;

static KEYBOARD_COLOR: Rgba<f32> = rgba!(0.576, 0.38, 0.91, 1.0);

#[derive(Clone)]
pub struct Keyboard(Arc<HandlerWrapper<PulseSender, KeyboardHandler>>);
impl Keyboard {
	pub fn create(spatial_parent: &Spatial) -> Self {
		let pulse_sender = PulseSender::create(
			spatial_parent,
			Transform {
				position: Vector3::from(Self::EMIT_POINT),
				..Default::default()
			},
			&KEYBOARD_MASK,
		)
		.unwrap();
		let keyboard_handler = KeyboardHandler::new(pulse_sender.alias());
		Keyboard(Arc::new(pulse_sender.wrap(keyboard_handler).unwrap()))
	}
	pub fn lock(&self) -> MutexGuard<KeyboardHandler> {
		self.0.lock_wrapped()
	}
}
impl Emittable for Keyboard {
	const SIZE: Vector3<f32> = Vector3 {
		x: 0.05,
		y: 0.03,
		z: 0.004,
	};
	const EMIT_POINT: Vector3<f32> = Vector3 {
		x: 0.0,
		y: 0.017667,
		z: 0.0,
	};

	fn model_resource() -> NamespacedResource {
		NamespacedResource {
			namespace: "manifold".to_string(),
			path: "keyboard".to_string(),
		}
	}
	fn update(&mut self, info: LogicStepInfo) {
		self.lock().logic_step(info);
	}
}

pub struct KeyboardHandler {
	pulse_sender: PulseSender,
	receivers_info: FxHashMap<String, KeyboardReceiverInfo>,
	keymap: Option<Keymap>,
}
impl KeyboardHandler {
	fn new(pulse_sender: PulseSender) -> Self {
		KeyboardHandler {
			pulse_sender,
			receivers_info: FxHashMap::default(),
			keymap: None,
		}
	}
	pub fn logic_step(&mut self, _info: LogicStepInfo) {
		for receiver_info in self.receivers_info.values_mut() {
			receiver_info.update_sender(&self.pulse_sender);
		}

		if self.keymap.is_some() {
			let receivers = self.pulse_sender.receivers();
			let receivers: Vec<&PulseReceiver> = self
				.receivers_info
				.iter()
				.filter(|(_, info)| info.connected() && !info.sent_keymap)
				.filter_map(|(uid, _)| receivers.get(uid).map(|(rx, _)| rx))
				.collect();
			if !receivers.is_empty() {
				let event = KeyboardEvent::new(self.keymap.as_ref(), None, None);
				event.send_event(&self.pulse_sender, &receivers);
				for receiver in self.receivers_info.values_mut() {
					receiver.sent_keymap = true;
				}
			}
		}
	}

	pub fn set_keymap(&mut self, keymap: Keymap) {
		for receiver_info in self.receivers_info.values_mut() {
			receiver_info.state = Some(State::new(&keymap));
			receiver_info.sent_keymap = false;
		}
		self.keymap = Some(keymap);
	}

	pub fn send_key(&self, key: u32, state: bool) {
		let keys_down = state.then_some(vec![key]);
		let keys_up = (!state).then_some(vec![key]);
		let event = KeyboardEvent::new(None, keys_up, keys_down);
		let receivers = self.pulse_sender.receivers();
		let receivers: Vec<&PulseReceiver> = self
			.receivers_info
			.iter()
			.filter(|(_, info)| info.connected())
			.filter_map(|(uid, _)| receivers.get(uid).map(|(rx, _)| rx))
			.collect();
		event.send_event(&self.pulse_sender, &receivers);
	}
}
impl PulseSenderHandler for KeyboardHandler {
	fn new_receiver(
		&mut self,
		info: NewReceiverInfo,
		receiver: PulseReceiver,
		_field: UnknownField,
	) {
		let mut keyboard_info = KeyboardReceiverInfo::new(self.keymap.as_ref(), receiver.alias());
		keyboard_info.connect(&self.pulse_sender, self.keymap.as_ref()); // temporary
		self.receivers_info.insert(info.uid, keyboard_info);
	}
	fn drop_receiver(&mut self, uid: &str) {
		self.receivers_info.remove(uid);
	}
}
unsafe impl Send for KeyboardHandler {}
unsafe impl Sync for KeyboardHandler {}

struct KeyboardReceiverInfo {
	lines: Option<Arc<Lines>>,
	state: Option<State>,
	receiver: PulseReceiver,
	sent_keymap: bool,
}
impl KeyboardReceiverInfo {
	fn new(keymap: Option<&Keymap>, receiver: PulseReceiver) -> Self {
		KeyboardReceiverInfo {
			lines: None,
			state: keymap.map(State::new),
			receiver,
			sent_keymap: false,
		}
	}
	fn connected(&self) -> bool {
		self.lines.is_some()
	}
	fn connect(&mut self, sender: &PulseSender, keymap: Option<&Keymap>) {
		self.lines = Some(Arc::new(
			Lines::create(&self.receiver, Transform::default(), &[], false).unwrap(),
		));
		if keymap.is_some() {
			let keymap_event = KeyboardEvent::new(keymap, None, None);
			keymap_event.send_event(sender, &[&self.receiver]);
		}
	}
	const LINE_THICKNESS: f32 = 0.005;
	fn update_sender(&mut self, sender: &PulseSender) {
		if let Some(lines) = self.lines.clone() {
			let future = sender.get_position_rotation_scale(&lines).unwrap();
			tokio::task::spawn(async move {
				if let Ok((position, _, _)) = future.await {
					lines
						.update_points(&[
							LinePoint {
								point: Vector3::from([0.0; 3]),
								thickness: Self::LINE_THICKNESS,
								color: KEYBOARD_COLOR,
							},
							LinePoint {
								point: position,
								thickness: Self::LINE_THICKNESS,
								color: KEYBOARD_COLOR,
							},
						])
						.unwrap();
				}
			});
		}
	}
}
unsafe impl Send for KeyboardReceiverInfo {}
unsafe impl Sync for KeyboardReceiverInfo {}
