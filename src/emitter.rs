use mint::Vector3;
use stardust_xr_fusion::{
	client::FrameInfo,
	core::values::Transform,
	drawable::{Model, ResourceID},
	fields::BoxField,
	spatial::Spatial,
};
use stardust_xr_molecules::{GrabData, Grabbable};

pub trait Emittable {
	const SIZE: [f32; 3];
	const EMIT_POINT: [f32; 3];
	fn model_resource() -> ResourceID;
	fn update(&mut self, info: FrameInfo);
}

#[allow(dead_code)]
pub struct Emitter<E: Emittable> {
	field: BoxField,
	grabbable: Grabbable,
	model: Model,
	pub contained: E,
}
impl<E: Emittable> Emitter<E> {
	pub fn new<F>(spatial_parent: &Spatial, contain_fn: F) -> Self
	where
		F: FnOnce(&Spatial) -> E,
	{
		let field =
			BoxField::create(spatial_parent, Transform::default(), Vector3::from(E::SIZE)).unwrap();
		let grabbable = Grabbable::create(
			spatial_parent,
			Transform::default(),
			&field,
			GrabData {
				max_distance: 0.1,
				..Default::default()
			},
		)
		.unwrap();
		grabbable
			.content_parent()
			.set_position(None, E::EMIT_POINT.map(|n| -n))
			.unwrap();
		let model = Model::create(
			grabbable.content_parent(),
			Transform::default(),
			&E::model_resource(),
		)
		.unwrap();
		let contained = contain_fn(grabbable.content_parent());
		field
			.set_spatial_parent(grabbable.content_parent())
			.unwrap();
		Emitter {
			field,
			grabbable,
			model,
			contained,
		}
	}

	pub fn frame(&mut self, info: FrameInfo) {
		let _ = self.grabbable.update(&info);
		self.contained.update(info);
	}
}
