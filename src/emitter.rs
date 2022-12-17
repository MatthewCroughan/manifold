use mint::Vector3;
use stardust_xr_molecules::{
	fusion::{
		client::LogicStepInfo, core::values::Transform, drawable::Model, fields::BoxField,
		resource::NamespacedResource, spatial::Spatial,
	},
	Grabbable,
};

pub trait Emittable {
	const SIZE: Vector3<f32>;
	const EMIT_POINT: Vector3<f32>;
	fn model_resource() -> NamespacedResource;
	fn update(&mut self, info: LogicStepInfo);
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
		let field = BoxField::create(spatial_parent, Transform::default(), E::SIZE).unwrap();
		let grabbable = Grabbable::new(spatial_parent, &field, 0.1).unwrap();
		grabbable
			.content_parent()
			.set_position(
				None,
				Vector3::from([-E::EMIT_POINT.x, -E::EMIT_POINT.y, -E::EMIT_POINT.z]),
			)
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

	pub fn logic_step(&mut self, info: LogicStepInfo) {
		self.grabbable.update();
		self.contained.update(info);
	}
}
