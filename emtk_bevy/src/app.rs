use std::{ops::Deref, time::Duration};

use bevy::{
	app::ScheduleRunnerPlugin,
	prelude::*,
	render::{
		RenderPlugin,
		settings::{Backends, RenderCreation, WgpuSettings},
	},
	winit::{EventLoopProxyWrapper, WakeUp, WinitPlugin},
};
use emtk_overlay::{EVENT_LOOP_PROXY, OverlayEvent};
use tracing::info;

use crate::overlay::{self, Overlay, OverlayPlugin};

pub fn run() -> AppExit {
	info!("Running {}", env!("CARGO_PKG_NAME"));

	let mut app = App::new();

	let mut winit_plugin = WinitPlugin::<OverlayEvent>::default();
	// let mut winit_plugin = WinitPlugin::<WakeUp>::default();
	// Exanima is supported on Windows only so there's no need to worry about
	// cross-platform compatibility issues when running on any thread
	winit_plugin.run_on_any_thread = true;

	// TODO: explore app.set_runner() for customizing how bevy runs

	app.add_plugins((
		DefaultPlugins
			.build()
			.disable::<WinitPlugin<WakeUp>>()
			.add(winit_plugin)
			.set(WindowPlugin {
				// Prevent bevy from creating a window on start up but according to
				// ScheduleRunnerPlugin, the window is never created making this not strictly
				// necessary.
				primary_window: None,
				..default()
			})
			// .set(winit_plugin)
			// .add_after::<WindowPlugin>(winit_plugin)
			.set(RenderPlugin {
				render_creation: RenderCreation::Automatic(WgpuSettings {
					// Exanima uses OpenGl which makes the overlay plugin dependent on this setting.
					backends: Some(Backends::GL),
					..default()
				}),
				..default()
			}),
		// .disable::<WinitPlugin>(),
		// ScheduleRunnerPlugin::run_loop(
		// 	// TODO: explore what this does
		// 	// Run 60 times per second.
		// 	Duration::from_secs_f64(1. / 60.),
		// ),
		OverlayPlugin,
	));

	// app.add_plugins((
	// 	PanicHandlerPlugin,
	// 	MinimalPlugins
	// 		.build()
	// 		.add_after::<TimePlugin>(TransformPlugin)
	// 		.add_after::<TransformPlugin>(DiagnosticsPlugin)
	// 		.add_after::<DiagnosticsPlugin>(InputPlugin),
	// 	AccessibilityPlugin,
	// 	TerminalCtrlCHandlerPlugin,
	// 	AssetPlugin::default(),
	// 	ScenePlugin,
	// 	OverlayPlugin,
	// 	ImagePlugin::default(),
	// 	PipelinedRenderingPlugin,
	// 	CorePipelinePlugin,
	// 	SpritePlugin,
	// 	TextPlugin,
	// 	UiPlugin::default(),
	// 	PbrPlugin::default(),
	// 	GltfPlugin::default(),
	// 	AudioPlugin::default(),
	// 	GilrsPlugin,
	// 	AnimationPlugin,
	// 	GizmoPlugin,
	// 	StatesPlugin,
	// 	DefaultPickingPlugins,
	// ));

	app.add_systems(Startup, start);

	app.run()
}

fn start(
	mut commands: Commands,
	event_loop_proxy: Res<EventLoopProxyWrapper<OverlayEvent>>,
	// event_loop_proxy: Res<EventLoopProxyWrapper<WakeUp>>
) {
	EVENT_LOOP_PROXY
		.set(event_loop_proxy.into_inner().deref().clone())
		.unwrap();

	let overlay = commands.spawn(Overlay::default()).id();
	commands.trigger_targets(overlay::Attach, overlay);
}
