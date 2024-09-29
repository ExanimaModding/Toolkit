use std::{collections::HashMap, fs, path::PathBuf, time::Instant};

use human_bytes::human_bytes;
use iced::{
	widget::{
		button, checkbox, container, horizontal_rule, horizontal_space, mouse_area, pick_list,
		scrollable, svg, text, text_input, Column, Row,
	},
	window, Alignment, Border, Element, Length, Size, Subscription, Task, Theme,
};
use lilt::{Animated, Easing};
use rfd::FileDialog;

use crate::{
	config::Config,
	gui::{
		constants::{CARGO_PKG_VERSION, FADE_DURATION},
		load_order, theme,
		widget::tooltip,
		Icon,
	},
};

pub enum Action {
	CloseModal,
	ConfigChanged(Config),
	Run(Task<Message>),
	ViewChangelog,
	None,
}

#[derive(Debug, Clone)]
pub struct Settings {
	cache_size: u64,
	config: Config,
	exanima_exe: String,
	fade: Animated<bool, Instant>,
	size: Option<Size>,
	theme: Theme,
	tooltip_fade: Animated<bool, Instant>,
}

#[derive(Debug, Clone)]
pub enum Message {
	CacheChecked,
	CacheCleared,
	CacheSize(u64),
	CacheOpened,
	Changelog,
	ConfigOpened,
	ConfigRefetched(Config),
	Confirm,
	DeveloperToggled(bool),
	ExanimaExe(PathBuf),
	ExanimaExeDialog,
	ExplainToggled(bool),
	FadeOut,
	SizeChanged(Size),
	ThemeChanged(Theme),
	Tick,
	TooltipShow,
	TooltipHide,
}

impl Settings {
	pub fn new(config: Config, theme: Theme, size: Option<Size>) -> (Self, Task<Message>) {
		let exanima_exe = match &config.exanima_exe {
			Some(exanima_exe) => exanima_exe.clone(),
			None => String::new(),
		};
		(
			Self {
				config,
				exanima_exe,
				size,
				theme,
				..Default::default()
			},
			Task::done(Message::CacheChecked),
		)
	}

	pub fn update(&mut self, message: Message) -> Action {
		let now = Instant::now();

		match message {
			Message::CacheChecked => {
				if let Some(exanima_exe) = self.config.exanima_exe.clone() {
					return Action::Run(Task::perform(
						cache_size(cache_path(exanima_exe)),
						Message::CacheSize,
					));
				}
			}
			Message::CacheCleared => {
				return Action::Run(Task::perform(
					clear_cache(cache_path(self.config.exanima_exe.clone().unwrap())),
					|_| Message::CacheChecked,
				));
			}
			Message::CacheSize(cache_size) => self.cache_size = cache_size,
			Message::CacheOpened => {
				open::that(cache_path(self.config.exanima_exe.clone().unwrap())).unwrap()
			}
			Message::Changelog => return Action::ViewChangelog,
			Message::ConfigOpened => {
				open::that(dirs::config_dir().unwrap().join("Exanima Modding Toolkit")).unwrap()
			}
			Message::ConfigRefetched(config) => self.config = config,
			Message::Confirm => {
				return Action::CloseModal;
			}
			Message::DeveloperToggled(developer) => {
				if let Some(launcher) = &mut self.config.launcher {
					launcher.developer = developer;
					return Action::ConfigChanged(self.config.clone());
				}
			}
			Message::ExanimaExe(path) => {
				let path_str = path.to_str().unwrap().to_string();
				self.exanima_exe = path_str.clone();
				if !path.is_file() {
					return Action::None;
				}
				self.config.exanima_exe = Some(path_str);
				if self.config.load_order.is_empty() {
					self.config.load_order = load_order(&path);
				} else {
					let load_order = load_order(&path);

					// append any new mods not found in config
					self.config.load_order.append(
						&mut load_order
							.iter()
							.filter_map(|(maybe_mod_id, enabled)| {
								if self
									.config
									.load_order
									.iter()
									.any(|(mod_id, _enabled)| mod_id == maybe_mod_id)
								{
									None
								} else {
									Some((maybe_mod_id.clone(), *enabled))
								}
							})
							.collect::<Vec<_>>(),
					);
				}
				return Action::ConfigChanged(self.config.clone());
			}
			Message::ExanimaExeDialog => {
				if let Some(path) = FileDialog::new()
					.add_filter("Exanima Executable", &["exe"])
					.pick_file()
				{
					return Action::Run(Task::done(Message::ExanimaExe(path)));
				}
			}
			Message::ExplainToggled(explain) => {
				if let Some(launcher) = &mut self.config.launcher {
					launcher.explain = explain;
					return Action::ConfigChanged(self.config.clone());
				}
			}
			Message::FadeOut => self.fade.transition(false, now),
			Message::SizeChanged(size) => self.size = Some(size),
			Message::ThemeChanged(theme) => {
				self.theme = theme.to_owned();
				let theme_setting = match theme {
					Theme::Light => "light",
					Theme::Dark => "dark",
					Theme::Dracula => "dracula",
					Theme::Nord => "nord",
					Theme::SolarizedLight => "solarized_light",
					Theme::SolarizedDark => "solarized_dark",
					Theme::GruvboxLight => "gruvbox_light",
					Theme::GruvboxDark => "gruvbox_dark",
					Theme::CatppuccinLatte => "catppuccin_latte",
					Theme::CatppuccinFrappe => "catppuccin_frappe",
					Theme::CatppuccinMacchiato => "catppuccin_macchiato",
					Theme::CatppuccinMocha => "catppuccin_mocha",
					Theme::TokyoNight => "tokyo_night",
					Theme::TokyoNightStorm => "tokyo_night_storm",
					Theme::TokyoNightLight => "tokyo_night_light",
					Theme::KanagawaWave => "kanagawa_wave",
					Theme::KanagawaDragon => "kanagawa_dragon",
					Theme::KanagawaLotus => "kanagawa_lotus",
					Theme::Moonfly => "moonfly",
					Theme::Nightfly => "nightfly",
					Theme::Oxocarbon => "oxocarbon",
					Theme::Ferra => "ferra",
					Theme::Custom(_custom) => "custom",
				};
				self.config.launcher.as_mut().unwrap().theme = theme_setting.to_string();
				return Action::ConfigChanged(self.config.clone());
			}
			Message::Tick => (),
			Message::TooltipHide => {
				if self.config.exanima_exe.is_none() && !self.tooltip_fade.in_progress(now) {
					self.tooltip_fade.transition_instantaneous(false, now);
				}
			}
			Message::TooltipShow => {
				if self.config.exanima_exe.is_none() && !self.tooltip_fade.in_progress(now) {
					self.tooltip_fade.transition(true, now);
				}
			}
		};

		Action::None
	}

	pub fn view(&self, icons: &HashMap<Icon, svg::Handle>) -> Element<Message> {
		let now = Instant::now();

		let spacing = 6;
		let category_size = 24;
		let animate_alpha = if self.size.is_none() {
			1.
		} else {
			self.fade.animate_bool(0., 1., now)
		};
		let content = container(scrollable(
			container(
				Column::new()
					.push(
						Column::new()
							.push(text("Settings").size(36))
							.push(
								horizontal_rule(1)
									.style(move |theme| theme::rule(theme, animate_alpha)),
							)
							.spacing(spacing),
					)
					.push(
						Column::new()
							.push(text("Exanima").size(category_size))
							.push(
								Column::new()
									.push(Row::new().push(text("Game Executable Path")).push_maybe(
										if self.size.is_none() {
											None
										} else {
											Some(text("*").style(move |theme| {
												text::Style {
													color: Some(
														text::danger(theme)
															.color
															.unwrap()
															.scale_alpha(animate_alpha),
													),
												}
											}))
										},
									))
									.push(
										Row::new()
											.push(
												text_input(
													r"C:\Program Files (x86)\Steam\steamapps\common\Exanima\Exanima.exe",
													&self.exanima_exe,
												)
												.on_input(|s| Message::ExanimaExe(PathBuf::from(s)))
												.padding(5)
												.style(move |theme, status| {
													let mut style = text_input::default(theme, status);
													style.background = style.background.scale_alpha(animate_alpha);
													style.border = style.border.color(style.border.color.scale_alpha(animate_alpha));
													style.icon = style.icon.scale_alpha(animate_alpha);
													style.placeholder = style.placeholder.scale_alpha(animate_alpha);
													style.value = style.value.scale_alpha(animate_alpha);
													style
												}),
											)
											.push(
												button(
													Row::new()
														.push(text("Browse"))
														.push(
															container(
																svg(icons
																	.get(&Icon::SquareArrowOutUpRight)
																	.unwrap()
																	.clone())
																.width(Length::Shrink)
																.height(Length::Fixed(16.))
																.opacity(animate_alpha)
																.style(theme::svg_button),
															)
															.height(Length::Fixed(21.))
															.align_y(Alignment::Center),
														)
														.spacing(2),
												)
												.on_press(Message::ExanimaExeDialog)
												.style(move |theme, status| {
													let mut style = button::primary(theme, status);
													style.text_color =
														style.text_color.scale_alpha(animate_alpha);
													style.with_background(
														style
															.background
															.unwrap()
															.scale_alpha(animate_alpha),
													)
												}),
											)
											.spacing(1),
									)
									.spacing(3),
							)
							.push(
								horizontal_rule(1)
									.style(move |theme| theme::rule(theme, animate_alpha)),
							)
							.spacing(spacing),
					)
					.push(
						Column::new()
							.push(text("Appearance").size(category_size))
							.push(
								pick_list(
									Theme::ALL,
									Some(self.theme.to_owned()),
									Message::ThemeChanged,
								)
								.style(move |theme, status| {
									theme::pick_list(theme, status, animate_alpha)
								}),
							)
							.push(
								horizontal_rule(1)
									.style(move |theme| theme::rule(theme, animate_alpha)),
							)
							.spacing(spacing),
					)
					.push_maybe(if self.size.is_none() {
						Some(
							Column::new()
								.push(text("About").size(category_size))
								.push(Column::new().push(text(format!("Version: {}", CARGO_PKG_VERSION))))
								.push(button("View Changelog").on_press(Message::Changelog))
								.push(horizontal_rule(1))
								.spacing(spacing),
						)
					} else {
						None
					})
					.push_maybe(if self.size.is_none() {
						Some(
							Column::new()
								.push(text("Cache").size(category_size))
								.push(
									button(
										Row::new()
											.push(text("Open Cache"))
											.push(
												container(
													svg(icons
														.get(&Icon::SquareArrowOutUpRight)
														.unwrap()
														.clone())
													.width(Length::Shrink)
													.height(Length::Fixed(16.))
													.style(theme::svg_button),
												)
												.height(Length::Fixed(21.))
												.align_y(Alignment::Center),
											)
											.spacing(2),
									)
									.on_press(Message::CacheOpened),
								)
								.push(
									Row::new()
										.push(
											button("Clear Cache")
												.on_press_maybe(if self.cache_size == 0 {
													None
												} else {
													Some(Message::CacheCleared)
												})
												.style(button::danger),
										)
										.push(
											container(text(format!(
												"Size: {}",
												human_bytes(self.cache_size as f64)
											)))
											.padding(5),
										)
										.align_y(Alignment::Center),
								)
								.push(horizontal_rule(1))
								.spacing(spacing),
						)
					} else {
						None
					})
					.push(
						Column::new()
							.push(text("Developer").size(category_size))
							.push(
								Row::new().push(
									checkbox(
										"Developer Mode",
										self.config.launcher.as_ref().unwrap().developer,
									)
									.on_toggle(Message::DeveloperToggled)
									.style(move |theme, status| {
										theme::checkbox(theme, status, animate_alpha)
									}),
								),
							)
							.push_maybe(if self.config.launcher.as_ref().unwrap().developer {
								Some(
									Row::new().push(
										checkbox(
											"Explain UI Layout",
											self.config.launcher.as_ref().unwrap().explain,
										)
										.on_toggle(Message::ExplainToggled)
										.style(move |theme, status| {
											theme::checkbox(theme, status, animate_alpha)
										}),
									),
								)
							} else {
								None
							})
							.push_maybe(if self.config.launcher.as_ref().unwrap().developer {
								Some(
									button(
										Row::new()
											.push(text("Open Config"))
											.push(
												container(
													svg(icons
														.get(&Icon::SquareArrowOutUpRight)
														.unwrap()
														.clone())
													.width(Length::Shrink)
													.height(Length::Fixed(16.))
													.opacity(animate_alpha)
													.style(theme::svg_button),
												)
												.height(Length::Fixed(21.))
												.align_y(Alignment::Center),
											)
											.spacing(2),
									)
									.on_press(Message::ConfigOpened)
								)
							} else {
								None
							})
							.push(
								horizontal_rule(1)
									.style(move |theme| theme::rule(theme, animate_alpha)),
							)
							.spacing(spacing),
					)
					.push_maybe(if self.size.is_none() {
						None
					} else {
						Some(
							Row::new().push(horizontal_space()).push(
								mouse_area(
									tooltip(
										button("Confirm")
											.on_press_maybe(
												if self.config.exanima_exe.is_none() {
													None
												} else {
													Some(Message::Confirm)
												},
											)
											.style(move |theme, status| {
												let mut style = button::success(theme, status);
												style.text_color =
													style.text_color.scale_alpha(animate_alpha);
												style.with_background(
													style
														.background
														.unwrap()
														.scale_alpha(animate_alpha),
												)
											}),
										"Fill out the required fields",
									)
									.style(move |theme| {
										theme::tooltip(
											theme,
											if self.size.is_none() {
												1.
											} else {
												self.tooltip_fade.animate_bool(0., 1., now)
											},
										)
									}),
								)
								.on_enter(Message::TooltipShow)
								.on_move(|_| Message::TooltipShow)
								.on_exit(Message::TooltipHide),
							),
						)
					})
					.spacing(12),
			)
			.padding(12),
		))
		.style(move |theme: &Theme| {
			let palette = theme.extended_palette();
			container::Style::default()
				.color(palette.background.base.text.scale_alpha(animate_alpha))
				.background(palette.background.base.color.scale_alpha(animate_alpha))
				.border(Border::default().rounded(8))
		});

		if let Some(size) = self.size {
			content.width(size.width).max_height(size.height)
		} else {
			content
		}
		.into()
	}

	pub fn subscription(&self) -> Subscription<Message> {
		let now = Instant::now();

		if self.tooltip_fade.in_progress(now) {
			window::frames().map(|_| Message::Tick)
		} else {
			Subscription::none()
		}
	}
}

impl Default for Settings {
	fn default() -> Self {
		let now = Instant::now();

		Self {
			cache_size: u64::default(),
			config: Config::default(),
			exanima_exe: String::default(),
			fade: Animated::new(false)
				.duration(FADE_DURATION as f32)
				.easing(Easing::EaseOut)
				.delay(0.)
				.auto_start(true, now),
			size: Some(Size::default()),
			theme: Theme::default(),
			tooltip_fade: Animated::new(false)
				.duration(FADE_DURATION as f32)
				.easing(Easing::EaseOut)
				.delay(500.),
		}
	}
}

// TODO: move cache_path into a more appropriate file
pub fn cache_path(exanima_exe: String) -> PathBuf {
	let path = PathBuf::from(exanima_exe)
		.parent()
		.unwrap()
		.join(".emtk")
		.join("cache");

	if !path.is_dir() {
		fs::create_dir_all(&path).unwrap();
	}

	path
}

pub async fn cache_size(cache_path: PathBuf) -> u64 {
	if !cache_path.is_dir() {
		return 0;
	}

	let mut total_size = 0;

	for entry in cache_path.read_dir().unwrap().flatten() {
		total_size += entry.metadata().unwrap().len();
	}

	total_size
}

pub async fn clear_cache(cache_path: PathBuf) {
	if !cache_path.is_dir() {
		return;
	}

	fs::remove_dir_all(cache_path).unwrap();
}
