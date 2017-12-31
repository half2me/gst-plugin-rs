// Copyright (C) 2017 Sebastian Dröge <sebastian@centricular.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use glib;
use gst;
use gst::prelude::*;
use gst_video;

use gst_plugin::properties::*;
use gst_plugin::object::*;
use gst_plugin::element::*;
use gst_plugin::base_transform::*;

use std::i32;
use std::sync::Mutex;

const DEFAULT_STEPS: u32 = 256;

#[derive(Debug, Clone, Copy)]
struct Settings {
    pub steps: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            steps: DEFAULT_STEPS,
        }
    }
}

struct State {
    out_info: gst_video::VideoInfo,
}

struct Rgb2Grey {
    cat: gst::DebugCategory,
    settings: Mutex<Settings>,
    state: Mutex<Option<State>>,
}

static PROPERTIES: [Property; 1] = [
    Property::UInt(
        "steps",
        "Number of Steps",
        "Number of grey steps to use",
        (1, 256),
        DEFAULT_STEPS,
        PropertyMutability::ReadWrite,
    ),
];

impl Rgb2Grey {
    fn new(_transform: &BaseTransform) -> Self {
        Self {
            cat: gst::DebugCategory::new(
                "rsrgb2grey",
                gst::DebugColorFlags::empty(),
                "Rust RGB-GREY converter",
            ),
            settings: Mutex::new(Default::default()),
            state: Mutex::new(None),
        }
    }

    fn class_init(klass: &mut BaseTransformClass) {
        klass.set_metadata(
            "RGB-GREY Converter",
            "Filter/Effect/Converter/Video",
            "Converts RGB to GREY or greyscale RGB",
            "Sebastian Dröge <sebastian@centricular.com>",
        );

        let caps = gst::Caps::new_simple(
            "video/x-raw",
            &[
                (
                    "format",
                    &gst::List::new(&[
                        &gst_video::VideoFormat::Bgrx.to_string(),
                        &gst_video::VideoFormat::Gray8.to_string(),
                    ]),
                ),
                ("width", &gst::IntRange::<i32>::new(0, i32::MAX)),
                ("height", &gst::IntRange::<i32>::new(0, i32::MAX)),
                (
                    "framerate",
                    &gst::FractionRange::new(
                        gst::Fraction::new(0, 1),
                        gst::Fraction::new(i32::MAX, 1),
                    ),
                ),
            ],
        );
        let src_pad_template = gst::PadTemplate::new(
            "src",
            gst::PadDirection::Src,
            gst::PadPresence::Always,
            &caps,
        );
        klass.add_pad_template(src_pad_template);

        let caps = gst::Caps::new_simple(
            "video/x-raw",
            &[
                ("format", &gst_video::VideoFormat::Bgrx.to_string()),
                ("width", &gst::IntRange::<i32>::new(0, i32::MAX)),
                ("height", &gst::IntRange::<i32>::new(0, i32::MAX)),
                (
                    "framerate",
                    &gst::FractionRange::new(
                        gst::Fraction::new(0, 1),
                        gst::Fraction::new(i32::MAX, 1),
                    ),
                ),
            ],
        );
        let sink_pad_template = gst::PadTemplate::new(
            "sink",
            gst::PadDirection::Sink,
            gst::PadPresence::Always,
            &caps,
        );
        klass.add_pad_template(sink_pad_template);

        klass.install_properties(&PROPERTIES);

        klass.configure(BaseTransformMode::NeverInPlace, false, false);
    }

    fn init(element: &BaseTransform) -> Box<BaseTransformImpl<BaseTransform>> {
        let imp = Self::new(element);
        Box::new(imp)
    }
}

impl ObjectImpl<BaseTransform> for Rgb2Grey {
    fn set_property(&self, _obj: &glib::Object, id: u32, value: &glib::Value) {
        let prop = &PROPERTIES[id as usize];

        match *prop {
            Property::UInt("steps", ..) => {
                let mut settings = self.settings.lock().unwrap();
                settings.steps = value.get().unwrap();
            }
            _ => unimplemented!(),
        }
    }

    fn get_property(&self, _obj: &glib::Object, id: u32) -> Result<glib::Value, ()> {
        let prop = &PROPERTIES[id as usize];

        match *prop {
            Property::UInt("steps", ..) => {
                let settings = self.settings.lock().unwrap();
                Ok(settings.steps.to_value())
            }
            _ => unimplemented!(),
        }
    }
}

impl ElementImpl<BaseTransform> for Rgb2Grey {}

impl BaseTransformImpl<BaseTransform> for Rgb2Grey {
    fn transform_caps(
        &self,
        _element: &BaseTransform,
        direction: gst::PadDirection,
        caps: gst::Caps,
        filter: Option<&gst::Caps>,
    ) -> gst::Caps {
        let mut grey_caps = gst::Caps::new_empty();

        {
            let grey_caps = grey_caps.get_mut().unwrap();

            for s in caps.iter() {
                let mut s_grey = s.to_owned();
                s_grey.set("format", &gst_video::VideoFormat::Gray8.to_string());
                grey_caps.append_structure(s_grey);
            }
            grey_caps.append(caps);
        }

        if let Some(filter) = filter {
            grey_caps.intersect_with_mode(filter, gst::CapsIntersectMode::First)
        } else {
            grey_caps
        }
    }

    fn transform(
        &self,
        _element: &BaseTransform,
        inbuf: &gst::Buffer,
        outbuf: &mut gst::BufferRef,
    ) -> gst::FlowReturn {
        let mut settings = *self.settings.lock().unwrap();

        let mut state_guard = self.state.lock().unwrap();
        let state = match *state_guard {
            None => return gst::FlowReturn::NotNegotiated,
            Some(ref mut state) => state,
        };

        gst::FlowReturn::Ok
    }

    fn set_caps(&self, _element: &BaseTransform, incaps: &gst::Caps, outcaps: &gst::Caps) -> bool {
        let info = match gst_video::VideoInfo::from_caps(outcaps) {
            None => return false,
            Some(info) => info,
        };

        *self.state.lock().unwrap() = Some(State { out_info: info });

        true
    }

    fn stop(&self, _element: &BaseTransform) -> bool {
        // Drop state
        let _ = self.state.lock().unwrap().take();

        true
    }
}

struct Rgb2GreyStatic;

impl ImplTypeStatic<BaseTransform> for Rgb2GreyStatic {
    fn get_name(&self) -> &str {
        "Rgb2Grey"
    }

    fn new(&self, element: &BaseTransform) -> Box<BaseTransformImpl<BaseTransform>> {
        Rgb2Grey::init(element)
    }

    fn class_init(&self, klass: &mut BaseTransformClass) {
        Rgb2Grey::class_init(klass);
    }
}

pub fn register(plugin: &gst::Plugin) {
    let type_ = register_type(Rgb2GreyStatic);
    gst::Element::register(plugin, "rsrgb2grey", 0, type_);
}
