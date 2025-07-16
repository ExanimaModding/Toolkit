use std::ffi::CString;

use bevy::{
	ecs::{component::HookContext, world::DeferredWorld},
	prelude::*,
	render::RenderApp,
	window::{PrimaryWindow, RawHandleWrapper, WindowWrapper},
};
use emtk_overlay::{MainWorldReceiver, RenderWorldSender};
use raw_window_handle::{
	DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawWindowHandle,
	Win32WindowHandle, WindowHandle,
};
use tracing::{debug, info};
use winapi::um::winuser::GetClassNameA;

pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
	fn build(&self, app: &mut App) {
		// let (s, r) = crossbeam_channel::unbounded();

		app.add_observer(Self::attach).add_observer(Self::detach);

		// let render_app = app
		// 	.insert_resource(MainWorldReceiver(r))
		// 	.sub_app_mut(RenderApp);

		// emtk_overlay::SENDER
		// 	.set(RenderWorldSender(s.clone()))
		// 	.unwrap();
		// render_app.insert_resource(RenderWorldSender(s));
	}
}

impl OverlayPlugin {
	fn attach(trigger: Trigger<Attach>, mut commands: Commands, attaching: Query<&Attaching>) {
		if let Some(target_handle) = emtk_overlay::WINDOW_HANDLE.get() {
			let class_name = unsafe {
				let mut class_name = vec![0u8; u8::MAX.try_into().unwrap()];
				let class_name_len = GetClassNameA(
					target_handle.hwnd.get() as _,
					class_name.as_mut_ptr() as _,
					u8::MAX as _,
				);

				// Add one to include a null terminator
				let class_name_len = class_name_len + 1;

				CString::from_vec_with_nul(class_name[..class_name_len as _].to_vec())
					.unwrap()
					.into_string()
					.unwrap()
			};

			struct Win32WindowWrapper(Win32WindowHandle);
			impl HasDisplayHandle for Win32WindowWrapper {
				fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
					Ok(DisplayHandle::windows())
				}
			}
			impl HasWindowHandle for Win32WindowWrapper {
				fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
					unsafe { Ok(WindowHandle::borrow_raw(RawWindowHandle::Win32(self.0))) }
				}
			}

			commands.entity(trigger.target()).insert((
				PrimaryWindow,
				Window {
					name: Some(class_name),
					..default()
				},
				RawHandleWrapper::new(&WindowWrapper::new(Win32WindowWrapper(*target_handle)))
					.unwrap(),
				Attached,
			));
			info!("Overlay attached")
		} else {
			let entity = trigger.target();
			if attaching.get(entity).is_err() {
				emtk_overlay::run();
				commands.entity(trigger.target()).insert(Attaching);
			}
			commands.trigger_targets(Attach, entity);
			debug!("Overlay attaching");
		};
	}

	fn detach(trigger: Trigger<Detach>, mut commands: Commands, attached: Query<&Attached>) {
		let entity = trigger.target();
		if attached.get(entity).is_ok() {
			commands
				.entity(entity)
				.remove::<(PrimaryWindow, Window, RawHandleWrapper)>()
				.insert(Detached);
		}
	}
}

#[derive(Default, Reflect)]
#[reflect(Default)]
pub enum State {
	// TODO: make use of detached to be able to toggle the overlay on and off
	/// Overlay window is detached from Exanima's window
	#[default]
	Detached,
	/// Overlay window is attaching to Exanima's window
	Attaching,
	/// Overlay window is attached to Exanima's window
	Attached,
}

/// A marker component to identify an entity as the overlay window.
#[derive(Default, Component, Reflect)]
#[reflect(Default, Component)]
pub struct Overlay {
	pub state: State,
}

/// Trigger to attach the overlay window to the Exanima window
#[derive(Event)]
pub struct Attach;

/// Trigger to detach the overlay window from the Exanima window
#[derive(Event)]
pub struct Detach;

#[derive(Component, Reflect)]
#[reflect(Component)]
#[component(on_add = Self::on_add)]
pub struct Detached;

impl Detached {
	fn on_add(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
		if let Some(mut overlay) = world.get_mut::<Overlay>(entity) {
			overlay.state = State::Detached
		}
		world
			.commands()
			.entity(entity)
			.remove::<(Attaching, Attached)>();
	}
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[component(on_add = Self::on_add)]
pub struct Attaching;

impl Attaching {
	fn on_add(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
		if let Some(mut overlay) = world.get_mut::<Overlay>(entity) {
			overlay.state = State::Attaching
		}
		world
			.commands()
			.entity(entity)
			.remove::<(Detached, Attached)>();
	}
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[component(on_add = Self::on_add)]
pub struct Attached;

impl Attached {
	fn on_add(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
		if let Some(mut overlay) = world.get_mut::<Overlay>(entity) {
			overlay.state = State::Attached
		}
		world
			.commands()
			.entity(entity)
			.remove::<(Detached, Attaching)>();
	}
}

// fn post_initialize(
// 	mut commands: Commands,
// 	// primary_window: Single<(Entity, &mut Window, &mut RawHandleWrapper), With<PrimaryWindow>>,
// 	// winit_windows: NonSendMut<WinitWindows>,
// 	// mut exanima_window: ResMut<NextState<State>>,
// ) {
// 	debug!("Running overlay post init");

// 	let Some(target_handle) = emtk_overlay::WINDOW_HANDLE.get() else {
// 		return;
// 	};

// 	// let (entity, mut window, mut handle_wrapper) = primary_window.into_inner();

// 	// unsafe {
// 	// 	handle_wrapper.set_window_handle(RawWindowHandle::Win32(*target_window));
// 	// }

// 	// let winit_window = winit_windows.get_window(entity).unwrap();

// 	// window.visible = true;
// 	// exanima_window.set(State::Initialized);
// }
