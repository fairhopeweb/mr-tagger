#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

use crate::cmd::AppArg;
use std::thread;
use tauri::api::{dialog, shell};
use tauri::{
	AboutMetadata, AppHandle, CustomMenuItem, Manager, Menu, MenuEntry, MenuItem, Submenu,
	WindowBuilder, WindowUrl,
};

mod cmd;
mod files;
mod frames;
mod image;

#[macro_export]
macro_rules! throw {
  ($($arg:tt)*) => {{
    return Err(format!($($arg)*))
  }};
}

fn main() {
	let ctx = tauri::generate_context!();
	let app = tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![
			cmd::error_popup,
			cmd::get_app,
			cmd::show,
			cmd::close_window,
			cmd::get_page,
			files::open_files,
			files::close_file,
			files::save_file,
			image::get_image,
			image::remove_image,
			image::set_image,
		])
		.setup(|app| {
			let _ = WindowBuilder::new(app, "main", WindowUrl::default())
				.title("Mr Tagger")
				.resizable(true)
				.decorations(true)
				.always_on_top(false)
				.inner_size(800.0, 550.0)
				.min_inner_size(400.0, 200.0)
				.skip_taskbar(false)
				.build()
				.expect("Unable to create window");
			Ok(())
		})
		.manage(cmd::AppState(Default::default()))
		.menu(Menu::with_items([
			#[cfg(target_os = "macos")]
			MenuEntry::Submenu(Submenu::new(
				&ctx.package_info().name,
				Menu::with_items([
					MenuItem::About(ctx.package_info().name.clone(), AboutMetadata::default())
						.into(),
					MenuItem::Separator.into(),
					MenuItem::Services.into(),
					MenuItem::Separator.into(),
					MenuItem::Hide.into(),
					MenuItem::HideOthers.into(),
					MenuItem::ShowAll.into(),
					MenuItem::Separator.into(),
					MenuItem::Quit.into(),
				]),
			)),
			MenuEntry::Submenu(Submenu::new(
				"File",
				Menu::with_items([
					CustomMenuItem::new("Open...", "Open...")
						.accelerator("cmdOrControl+O")
						.into(),
					MenuItem::Separator.into(),
					CustomMenuItem::new("Close", "Close")
						.accelerator("cmdOrControl+W")
						.into(),
					CustomMenuItem::new("Save", "Save")
						.accelerator("cmdOrControl+S")
						.into(),
					CustomMenuItem::new("Save As...", "Save As...")
						.accelerator("shift+cmdOrControl+S")
						.into(),
				]),
			)),
			MenuEntry::Submenu(Submenu::new(
				"Edit",
				Menu::with_items([
					MenuItem::Undo.into(),
					MenuItem::Redo.into(),
					MenuItem::Separator.into(),
					MenuItem::Cut.into(),
					MenuItem::Copy.into(),
					MenuItem::Paste.into(),
					#[cfg(not(target_os = "macos"))]
					MenuItem::Separator.into(),
					MenuItem::SelectAll.into(),
				]),
			)),
			MenuEntry::Submenu(Submenu::new(
				"View",
				Menu::with_items([MenuItem::EnterFullScreen.into()]),
			)),
			MenuEntry::Submenu(Submenu::new(
				"Window",
				Menu::with_items([MenuItem::Minimize.into(), MenuItem::Zoom.into()]),
			)),
			// You should always have a Help menu on macOS because it will automatically
			// show a menu search field
			MenuEntry::Submenu(Submenu::new(
				"Help",
				Menu::with_items([CustomMenuItem::new("Learn More", "Learn More").into()]),
			)),
		]))
		.on_menu_event(|event| {
			let event_name = event.menu_item_id();
			event.window().emit("menu", event_name).unwrap();
			match event_name {
				"Learn More" => {
					let link = "https://github.com/probablykasper/mr-tagger".to_string();
					shell::open(&event.window().shell_scope(), link, None).unwrap();
				}
				_ => {}
			}
		})
		.build(ctx)
		.expect("error while running tauri app");
	app.run(|_app_handle, e| match e {
		tauri::RunEvent::WindowEvent { label, event, .. } => match event {
			tauri::WindowEvent::CloseRequested { api, .. } => {
				if label == "main" && is_dirty(&_app_handle) {
					api.prevent_close();
					handle_close_requested(_app_handle.clone())
				}
			}
			_ => {}
		},
		_ => {}
	});
}

fn is_dirty(app: &AppHandle) -> bool {
	let app: AppArg<'_> = app.state();
	let app = app.0.lock().unwrap();
	for file in &app.files {
		if file.dirty {
			return true;
		}
	}
	false
}

fn handle_close_requested(app_handle: AppHandle) {
	thread::spawn(move || {
		let w = app_handle.get_window("main").unwrap();
		dialog::ask(
			Some(&w),
			"You have unsaved changes. Close without saving?",
			"",
			move |res| {
				let w = app_handle.get_window("main").unwrap();
				if res == true {
					w.close().unwrap();
				}
			},
		);
	});
}
