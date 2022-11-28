use color::{rgba, Rgba};
use mint::Vector3;
use parking_lot::MutexGuard;
use rustc_hash::FxHashMap;
use stardust_xr_molecules::{
	fusion::{
		client::LogicStepInfo,
		data::{NewReceiverInfo, PulseReceiver, PulseSender, PulseSenderHandler},
		drawable::{LinePoint, Lines},
		fields::UnknownField,
		resource::NamespacedResource,
		spatial::Spatial,
		HandlerWrapper, WeakNodeRef,
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
		Keyboard(Arc::new(
			PulseSender::create(
				spatial_parent,
				Some(Vector3::from(Self::EMIT_POINT)),
				None,
				KEYBOARD_MASK.clone(),
				|pulse_sender, _| KeyboardHandler::new(pulse_sender),
			)
			.unwrap(),
		))
	}
	pub fn lock(&self) -> MutexGuard<KeyboardHandler> {
		self.0.lock_inner()
	}
}
impl Emittable for Keyboard {
	const SIZE: [f32; 3] = [0.05, 0.03, 0.004];
	const EMIT_POINT: [f32; 3] = [0.0, 0.017667, 0.0];

	fn model_resource() -> NamespacedResource {
		NamespacedResource {
			namespace: "manifold".to_string(),
			path: "keyboard.glb".to_string(),
		}
	}
	fn update(&mut self, info: LogicStepInfo) {
		self.0.lock_inner().logic_step(info);
	}
}

pub struct KeyboardHandler {
	pulse_sender: WeakNodeRef<PulseSender>,
	receivers_info: FxHashMap<String, KeyboardReceiverInfo>,
	keymap: Option<Keymap>,
}
impl KeyboardHandler {
	fn new(pulse_sender: WeakNodeRef<PulseSender>) -> Self {
		KeyboardHandler {
			pulse_sender,
			receivers_info: FxHashMap::default(),
			keymap: None,
		}
	}
	pub fn logic_step(&mut self, _info: LogicStepInfo) {
		self.pulse_sender.with_node(|sender| {
			for receiver_info in self.receivers_info.values_mut() {
				receiver_info.update_sender(&sender);
			}

			if self.keymap.is_some() {
				let receivers = sender.receivers();
				let receivers: Vec<&PulseReceiver> = self
					.receivers_info
					.iter()
					.filter(|(_, info)| info.connected() && !info.sent_keymap)
					.filter_map(|(uid, _)| receivers.get(uid).map(|(rx, _)| rx))
					.collect();
				if !receivers.is_empty() {
					let event = KeyboardEvent::new(self.keymap.as_ref(), None, None);
					event.send_event(sender, &receivers);
					for receiver in self.receivers_info.values_mut() {
						receiver.sent_keymap = true;
					}
				}
			}
		});
	}

	pub fn set_keymap(&mut self, keymap: Keymap) {
		for receiver_info in self.receivers_info.values_mut() {
			receiver_info.state = Some(State::new(&keymap));
			receiver_info.sent_keymap = false;
		}
		self.keymap = Some(keymap);
	}

	pub fn send_key(&self, key: u32, state: bool) {
		self.pulse_sender.with_node(|sender| {
			let keys_down = state.then_some(vec![key]);
			let keys_up = (!state).then_some(vec![key]);
			let event = KeyboardEvent::new(None, keys_up, keys_down);
			let receivers = sender.receivers();
			let receivers: Vec<&PulseReceiver> = self
				.receivers_info
				.iter()
				.filter(|(_, info)| info.connected())
				.filter_map(|(uid, _)| receivers.get(uid).map(|(rx, _)| rx))
				.collect();
			event.send_event(sender, &receivers);
		});
	}
}
impl PulseSenderHandler for KeyboardHandler {
	fn new_receiver(
		&mut self,
		receiver: &PulseReceiver,
		_field: &UnknownField,
		info: NewReceiverInfo,
	) {
		dbg!(&info);
		let mut keyboard_info = KeyboardReceiverInfo::new(self.keymap.as_ref());
		keyboard_info.connect(receiver); // temporary
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
	sent_keymap: bool,
}
impl KeyboardReceiverInfo {
	fn new(keymap: Option<&Keymap>) -> Self {
		KeyboardReceiverInfo {
			lines: None,
			state: keymap.map(State::new),
			sent_keymap: false,
		}
	}
	fn connected(&self) -> bool {
		self.lines.is_some()
	}
	fn connect(&mut self, receiver: &PulseReceiver) {
		self.lines = Some(Arc::new(
			Lines::builder()
				.spatial_parent(&receiver.spatial)
				.points(&[])
				.cyclic(false)
				.build()
				.unwrap(),
		));
	}
	const LINE_THICKNESS: f32 = 0.005;
	fn update_sender(&mut self, sender: &PulseSender) {
		if let Some(lines) = self.lines.clone() {
			let future = sender.get_translation_rotation_scale(&lines).unwrap();
			tokio::task::spawn(async move {
				if let Ok((position, _rotation, _scalee)) = future.await {
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
