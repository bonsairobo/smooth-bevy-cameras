#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! define_on_controller_enabled_changed(($ControllerStruct:ty) => {
        fn on_controller_enabled_changed(
            mut look_transforms: Query<(&mut LookTransform, &$ControllerStruct), Changed<$ControllerStruct>>,
        ) {
            for (mut look_tfm, controller) in look_transforms.iter_mut() {
                look_tfm.enabled = controller.enabled;
            }
        }
    });
}

pub mod fps;
pub mod orbit;
pub mod unreal;
