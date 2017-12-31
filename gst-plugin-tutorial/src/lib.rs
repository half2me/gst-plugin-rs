// Copyright (C) 2017 Sebastian Dröge <sebastian@centricular.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![crate_type = "cdylib"]

extern crate glib;
#[macro_use]
extern crate gst_plugin;
extern crate gstreamer as gst;
extern crate gstreamer_base as gst_base;
extern crate gstreamer_video as gst_video;

mod rgb2gray;

fn plugin_init(plugin: &gst::Plugin) -> bool {
    rgb2grey::register(plugin);
    true
}

plugin_define!(
    b"rstutorial\0",
    b"Rust Tutorial Plugin\0",
    plugin_init,
    b"1.0\0",
    b"MIT/X11\0",
    b"rstutorial\0",
    b"rstutorial\0",
    b"https://github.com/sdroege/rsplugin\0",
    b"2017-12-30\0"
);
