#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! define_on_controller_enabled_changed(($ControllerStruct:ty) => {
        fn on_controller_enabled_changed(
            mut smoothers: Query<(&mut Smoother, &$ControllerStruct), Changed<$ControllerStruct>>,
        ) {
            for (mut smoother, controller) in smoothers.iter_mut() {
                smoother.set_enabled(controller.enabled);
            }
        }
    });
}

pub mod fps;
pub mod orbit;
pub mod unreal;
