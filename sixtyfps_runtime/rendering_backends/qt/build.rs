/* LICENSE BEGIN
    This file is part of the SixtyFPS Project -- https://sixtyfps.io
    Copyright (c) 2020 Olivier Goffart <olivier.goffart@sixtyfps.io>
    Copyright (c) 2020 Simon Hausmann <simon.hausmann@sixtyfps.io>

    SPDX-License-Identifier: GPL-3.0-only
    This file is also available under commercial licensing terms.
    Please contact info@sixtyfps.io for more information.
LICENSE END */
use std::process::Command;

fn qmake_query(var: &str) -> Option<String> {
    let qmake = std::env::var_os("QMAKE").unwrap_or("qmake".into());
    Command::new(qmake).env("QT_SELECT", "qt5").args(&["-query", var]).output().ok().map(|output| {
        String::from_utf8(output.stdout).expect("UTF-8 conversion from output of qmake failed")
    })
}
fn main() {
    println!("cargo:rerun-if-env-changed=SIXTYFPS_NO_QT");
    if std::env::var("TARGET").map_or(false, |t| t.starts_with("wasm"))
        || std::env::var("SIXTYFPS_NO_QT").is_ok()
    {
        println!("cargo:rustc-cfg=no_qt");
        return;
    }
    if qmake_query("QT_VERSION").is_none() {
        println!("cargo:rustc-cfg=no_qt");
        println!(
            "cargo:warning=Could not find a Qt installation. The Qt backend will not be functional. See https://github.com/sixtyfpsui/sixtyfps/blob/master/docs/install_qt.md for more info"
        );
        return;
    }

    let qt_include_path = qmake_query("QT_INSTALL_HEADERS").unwrap();
    let qt_library_path = qmake_query("QT_INSTALL_LIBS").unwrap();
    let mut config = cpp_build::Config::new();

    config.flag_if_supported("-std=c++17");
    config.flag_if_supported("/std:c++17");

    if cfg!(target_os = "macos") {
        config.flag("-F");
        config.flag(qt_library_path.trim());
    }

    config.include(qt_include_path.trim()).build("lib.rs");

    let macos_lib_search = if cfg!(target_os = "macos") { "=framework" } else { "" };
    let macos_lib_framework = if cfg!(target_os = "macos") { "" } else { "5" };

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-cdylib-link-arg=-Wl,-rpath,{}", qt_library_path.trim());
    }
    println!("cargo:rustc-link-search{}={}", macos_lib_search, qt_library_path.trim());
    println!("cargo:rustc-link-lib{}=Qt{}Widgets", macos_lib_search, macos_lib_framework);
    println!("cargo:rustc-link-lib{}=Qt{}Gui", macos_lib_search, macos_lib_framework);
    println!("cargo:rustc-link-lib{}=Qt{}Core", macos_lib_search, macos_lib_framework);

    println!("cargo:rerun-if-changed=qt_window.rs");
    println!("cargo:rerun-if-changed=widgets.rs");
    println!("cargo:rerun-if-changed=qttypes.rs");
    println!("cargo:rerun-if-changed=lib.rs");
}
