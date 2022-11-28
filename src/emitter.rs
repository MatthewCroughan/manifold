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
pub struct Emitter<T: Emittable> {
	field: BoxField,
	grabbable: Grabbable,
	model: Model,
	pub contained: T,
}
impl<T: Emittable> Emitter<T> {
	pub fn new<F>(spatial_parent: &Spatial, contain_fn: F) -> Self
	where
		F: FnOnce(&Spatial) -> T,
	{
		let field = BoxField::builder()
			.spatial_parent(spatial_parent)
			.size(Vector3::from(T::SIZE))
			.build()
			.unwrap();
		let grabbable = Grabbable::new(spatial_parent, &field).unwrap();
		let model = Model::builder()
			.spatial_parent(grabbable.content_parent())
			.resource(&T::model_resource())
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
