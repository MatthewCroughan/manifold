use mint::Vector3;
use stardust_xr_molecules::{
	fusion::{
		client::LogicStepInfo, drawable::Model, fields::BoxField, resource::NamespacedResource,
		spatial::Spatial,
	},
	Grabbable,
};

pub trait Emittable {
	const SIZE: [f32; 3];
	const EMIT_POINT: [f32; 3];
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
		let field = BoxField::builder()
			.spatial_parent(spatial_parent)
			.size(Vector3::from(E::SIZE))
			.build()
			.unwrap();
		let grabbable = Grabbable::new(spatial_parent, &field).unwrap();
		grabbable
			.content_parent()
			.set_position(None, Vector3::from(E::EMIT_POINT.map(|n| -n)))
			.unwrap();
		let model = Model::builder()
			.spatial_parent(grabbable.content_parent())
			.resource(&E::model_resource())
			.build()
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
