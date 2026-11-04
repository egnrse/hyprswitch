//! helper functions for sorting tests
#![allow(clippy::print_stdout)]

use hyprland::data::{Workspace, WorkspaceBasic};
use hyprland::shared::MonitorId;
use random_color::RandomColor;
use svg::node::element::{Group, Rectangle, Text, SVG};

use crate::ClientAddress;
use crate::{ClientData, MonitorData};

mod many_windows;
mod multi_workspace_multi_monitor_horizontal;
mod multi_workspaces;
mod simple;

#[cfg(test)]
pub fn is_sorted(data: &[(ClientAddress, ClientData)]) -> bool {
	data.windows(2).all(|l| {
		l[0].0
			.to_string()
			.trim_start_matches("0x")
			.parse::<u16>()
			.unwrap() < l[1]
			.0
			.to_string()
			.trim_start_matches("0x")
			.parse::<u16>()
			.unwrap()
	})
}

#[cfg(test)]
pub fn create_svg_from_client_tests(
	clients: &[(ClientAddress, ClientData)],
	filename: &str,
	monitor_data: Vec<(MonitorId, MonitorData)>,
) {
	let mut a = filename.split('/').collect::<Vec<&str>>();
	let mut n = "";
	// remove tests dir
	a.remove(0);
	if monitor_data.len() == 1 {
		// dont create folder for 1 svg
		n = a.pop().expect("unable to pop filename");
	}
	let filename = "target/test-svgs/".to_owned() + &*a.join("/");
	std::fs::create_dir_all(filename.clone())
		.expect("unable to create target/test-svgs directory and subdirectories");

	for (iden, monitor) in &monitor_data {
		let cl: Vec<(usize, u16, u16, u16, u16, String)> = clients
			.iter()
			.enumerate()
			.filter(|(_, (_, c))| c.monitor == *iden)
			.map(|(i, (a, c))| {
				(
					i,
					c.x * 10,
					c.y * 10,
					c.width * 10,
					c.height * 10,
					a.to_string(),
				)
			})
			.map(|(i, x, y, w, h, iden)| (i, x as u16, y as u16, w as u16, h as u16, iden))
			.collect();

		// add iden to filename if there are multiple monitors
		let iden = if monitor_data.len() != 1 {
			iden.to_string()
		} else {
			n.to_owned()
		};

		// find the width of the svg by finding the max x+width value
		let wid = cl
			.iter()
			.map(|(_, x, _, w, _, _)| x + w)
			.max()
			.expect("no clients")
			+ 10;

		create_svg(
			cl,
			format!("{filename}/{iden}.svg"),
			0,
			0,
			wid,
			monitor.height * 10 + 10,
			2,
		);
	}
}

#[cfg(test)]
pub fn create_svg(
	rectangles: Vec<(usize, u16, u16, u16, u16, String)>,
	filename: String,
	x: u16,
	y: u16,
	width: u16,
	height: u16,
	stroke_width: u16,
) {
	let mut svg = SVG::new()
		.set("width", "100%")
		.set("height", "100%")
		.set("viewBox", format!("{} {} {} {}", x, y, width, height));

	// add background
	svg = svg.add(
		Rectangle::new()
			.set("x", x)
			.set("y", y)
			.set("width", width)
			.set("height", height)
			.set("fill", "grey"),
	);

	for (i, x, y, width, height, identifier) in rectangles {
		let color = RandomColor::new().to_hsl_array();
		let group = Group::new()
			.add(
				Rectangle::new()
					.set("x", x)
					.set("y", y)
					.set("width", width)
					.set("height", height)
					.set(
						"stroke",
						format!("hsl({} {}% {}% / {}%)", color[0], color[1], color[2], 65),
					)
					.set("stroke-width", stroke_width)
					.set("fill", "none"),
			)
			.add(
				Text::new(format!("{i}-{identifier}"))
					.set(
						"x",
						(x + width / 2) as i32
							- ((identifier.len() as u16 * (stroke_width * 4)) / 2) as i32,
					)
					.set(
						"y",
						(y + height / 2) as i32
							+ ((((stroke_width as f32 * color[0] as f32) / 90.0) as i32)
								- stroke_width as i32),
					)
					.set("font-size", stroke_width * 4)
					.set("fill", "white"),
			);

		svg = svg.add(group);
	}

	svg::save(filename.clone(), &svg).unwrap_or_else(|_| panic!("unable to save svg {filename}"));
}

/// create a fake [[hyprland::data::Workspace]] instance
#[cfg(test)]
pub(crate) fn fake_workspace(name: &str, monitor: &str) -> Workspace {
	serde_json::from_value(serde_json::json!({
		"id": 1,
		"name": name,
		"monitor": monitor,
		"monitorID": 0,
		"windows": 0,
		"hasfullscreen": false,
		"lastwindow": "0x0",
		"lastwindowtitle": "",
		"ispersistent": false,
		"tiledLayout": "dwindle"
	}))
	.expect("test workspace JSON must match the Workspace schema")
}
/// create a fake [[hyprland::data::WorkspaceBasic]] instance
#[cfg(test)]
pub(crate) fn fake_workspace_basic(name: &str) -> WorkspaceBasic {
	serde_json::from_value(serde_json::json!({
		"name": name
	}))
	.expect("test WorkspaceBasic JSON must match the schema")
}

/// (x, y, width, height, workspace, monitor) => <increment>: ClientData { x, y, width, height, workspace, monitor, address: ClientAddress::new(<increment>), focus_history_id: <increment>, class: "test".to_string(), title: "test".to_string(), floating: false, active: true, ws: fake_workspace }
///
/// ```rust
/// let clients = client_vec![
///    (1, 1, 1, 3, 0, 0),
///    (3, 1, 2, 3, 0, 0),
///    (1, 5, 2, 2, 0, 0),
///    (4, 5, 1, 2, 0, 0),
/// ];
///```
#[cfg(test)]
macro_rules! client_vec {
    //($($x:expr),+ $(,)?) => {{
    ($(($x:expr, $y:expr, $w:expr, $h:expr, $ws:expr, $mon:expr $(, $name:expr)?)),+ $(,)?) => {{
        use hyprland::shared::Address;
        use crate::{ClientAddress, WorkspaceAddress};
        use crate::ClientData;

        const EXPECT_NAME: &str = "name should be defined in the macro";
        let mut map = Vec::new();
        let mut count = 0;
        $(
            count += 1;
            let name: Option<_> = Some("test".to_string()) $(.or(Some($name)))?;
            map.push((ClientAddress::from(Address::new(count)), ClientData {
                x: $x,
                y: $y,
                width: $w,
                height: $h,
                workspace: WorkspaceAddress::from(Address::new($ws)),
                monitor: $mon,
                focus_history_id: count as i8,
                class: name.clone().expect(EXPECT_NAME),
                title: name.clone().expect(EXPECT_NAME),
                floating: false,
                enabled: true,
                pid: 0,
                ws: crate::handle::sort::tests::fake_workspace_basic(&name.expect(EXPECT_NAME)),
            }));
        )+
        map
    }};
}

///
/// (x, y, width, height) => <increment>: MonitorData { x, y, width, height, connector: "test", active: true }
///
/// ```rust
/// let clients = monitor_map![
///    (0, 0, 4, 7),
/// ];
///```
#[cfg(test)]
macro_rules! monitor_map {
	($($x:expr),+ $(,)?) => {{
		use hyprland::shared::MonitorId;
		use crate::MonitorData;

		let mut map = Vec::new();
		let mut count = -1;
		$(
			count += 1;
			map.push((count as MonitorId, MonitorData {
				x: $x.0,
				y: $x.1,
				width: $x.2,
				height: $x.3,
				connector: "test".to_string(),
				enabled: true,
			}));
		)+
		map
	}};
}

/// (x, y, monitor) => <increment>: WorkspaceData { x, y, width: 0, height: 0, id: <increment>, name: "test", monitor, active: true, ws: fake_workspace }
///
/// ```rust
/// let clients = workspace_map![
///    (0, 0, 0),
/// ];
///```
#[cfg(test)]
macro_rules! workspace_map {
	($($x:expr),+ $(,)?) => {{
		use hyprland::shared::Address;
		use crate::WorkspaceAddress;
		use crate::WorkspaceData;

		let mut map = Vec::new();
		let mut count = -1;
		$(
			count += 1;
			let (x, y, monitor) = $x;
			map.push((WorkspaceAddress::from(Address::new(count)), WorkspaceData {
				x,
				y,
				width: 0,
				height: 0,
				name: "test".to_string(),
				monitor,
				enabled: true,
				ws: crate::handle::sort::tests::fake_workspace("test", &monitor.to_string()),
			}));
		)+
		map
		}};
}

#[cfg(test)]
macro_rules! function {
	() => {{
		fn f() {}
		fn type_name_of<T>(_: T) -> &'static str {
			std::any::type_name::<T>()
		}
		let name = type_name_of(f);

		let mut filename = file!().split("/").collect::<Vec<&str>>();
		if let Some(last) = filename.last() {
			if *last == "mod.rs" {
				filename.pop();
			}
		}
		let filename = filename.join("/");

		// Find and cut the rest of the path
		&(filename
			+ "/" + match &name[..name.len() - 3].rfind(':') {
			Some(pos) => &name[pos + 1..name.len() - 3],
			None => &name[..name.len() - 3],
		})
	}};
}

// used for tests
#[allow(unused_imports)]
pub(crate) use client_vec;
#[allow(unused_imports)]
pub(crate) use function;
#[allow(unused_imports)]
pub(crate) use monitor_map;
#[allow(unused_imports)]
pub(crate) use workspace_map;
