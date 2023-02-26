use crate::emitter::Emittable;
use color::{rgba, Rgba};
use mint::{Vector2, Vector3};
use parking_lot::MutexGuard;
use rustc_hash::FxHashMap;
use stardust_xr_fusion::{
	client::FrameInfo,
	core::values::Transform,
	data::{NewReceiverInfo, PulseReceiver, PulseSender, PulseSenderHandler},
	drawable::{LinePoint, Lines, ResourceID},
	fields::UnknownField,
	node::NodeType,
	spatial::Spatial,
	HandlerWrapper,
};
use stardust_xr_molecules::mouse::{MouseEvent, MOUSE_MASK};
use std::sync::Arc;

static MOUSE_COLOR: Rgba<f32> = rgba!(0.141, 0.886, 0.521, 1.0);

#[derive(Clone)]
pub struct Mouse(Arc<HandlerWrapper<PulseSender, MouseHandler>>);
impl Mouse {
	pub fn create(spatial_parent: &Spatial) -> Self {
		let pulse_sender = PulseSender::create(
			spatial_parent,
			Transform::from_position(Self::EMIT_POINT),
			&MOUSE_MASK,
		)
		.unwrap();
		let keyboard_handler = MouseHandler::new(pulse_sender.alias());
		Mouse(Arc::new(pulse_sender.wrap(keyboard_handler).unwrap()))
	}
	pub fn lock(&self) -> MutexGuard<MouseHandler> {
		self.0.lock_wrapped()
	}
}
impl Emittable for Mouse {
	const SIZE: [f32; 3] = [0.018, 0.027379, 0.004];
	const EMIT_POINT: [f32; 3] = [0.0, 0.017667, 0.0];

	fn model_resource() -> ResourceID {
		ResourceID::new_namespaced("manifold", "mouse")
	}
	fn update(&mut self, info: FrameInfo) {
		self.lock().frame(info);
	}
}

pub struct MouseHandler {
	pulse_sender: PulseSender,
	receivers_info: FxHashMap<String, MouseReceiverInfo>,
}
impl MouseHandler {
	fn new(pulse_sender: PulseSender) -> Self {
		MouseHandler {
			pulse_sender,
			receivers_info: FxHashMap::default(),
		}
	}
	pub fn frame(&mut self, _info: FrameInfo) {
		for receiver_info in self.receivers_info.values_mut() {
			receiver_info.update_sender(&self.pulse_sender);
		}

		// let receivers = self.pulse_sender.receivers();
		// let receivers: Vec<&PulseReceiver> = self
		// 	.receivers_info
		// 	.iter()
		// 	.filter(|(_, info)| info.connected() && !info.sent_keymap)
		// 	.filter_map(|(uid, _)| receivers.get(uid).map(|(rx, _)| rx))
		// 	.collect();
		// if !receivers.is_empty() {
		// 	let event = KeyboardEvent::new(, None, None);
		// 	event.send_event(&self.pulse_sender, &receivers);
		// 	for receiver in self.receivers_info.values_mut() {
		// 		receiver.sent_keymap = true;
		// 	}
		// }
	}

	pub fn send_event(
		&self,
		delta: Option<Vector2<f32>>,
		scroll_distance: Option<Vector2<f32>>,
		scroll_steps: Option<Vector2<f32>>,
		buttons_up: Option<Vec<u32>>,
		buttons_down: Option<Vec<u32>>,
	) {
		let event = MouseEvent::new(
			delta,
			scroll_distance,
			scroll_steps,
			buttons_up,
			buttons_down,
		);
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
impl PulseSenderHandler for MouseHandler {
	fn new_receiver(
		&mut self,
		info: NewReceiverInfo,
		receiver: PulseReceiver,
		_field: UnknownField,
	) {
		let mut keyboard_info = MouseReceiverInfo::new(receiver.alias());
		keyboard_info.connect(); // temporary
		self.receivers_info.insert(info.uid, keyboard_info);
	}
	fn drop_receiver(&mut self, uid: &str) {
		self.receivers_info.remove(uid);
	}
}
unsafe impl Send for MouseHandler {}
unsafe impl Sync for MouseHandler {}

struct MouseReceiverInfo {
	lines: Option<Arc<Lines>>,
	receiver: PulseReceiver,
}
impl MouseReceiverInfo {
	fn new(receiver: PulseReceiver) -> Self {
		MouseReceiverInfo {
			lines: None,
			receiver,
		}
	}
	fn connected(&self) -> bool {
		self.lines.is_some()
	}
	fn connect(&mut self) {
		self.lines = Some(Arc::new(
			Lines::create(&self.receiver, Transform::default(), &[], false).unwrap(),
		));
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
								color: MOUSE_COLOR,
							},
							LinePoint {
								point: position,
								thickness: Self::LINE_THICKNESS,
								color: MOUSE_COLOR,
							},
						])
						.unwrap();
				}
			});
		}
	}
}
unsafe impl Send for MouseReceiverInfo {}
unsafe impl Sync for MouseReceiverInfo {}
