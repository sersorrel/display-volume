use libpulse_binding::{
    self as pulse,
    callbacks::ListResult,
    context::{self, Context},
    mainloop::standard::{IterateResult, Mainloop},
    operation, volume,
};
use notify_rust::{Hint, Notification, Timeout};
use num_traits::{cast, Float};

const VOLUME_0: u32 = volume::VOLUME_MUTED.0;
const VOLUME_100: u32 = volume::VOLUME_NORM.0;

fn unlerp<T, V>(min: T, max: T, val: V) -> V
where
    T: Float,
    V: Float,
{
    (val - cast(min).unwrap()) / cast(max - min).unwrap()
}

fn main() {
    let mut mainloop = Mainloop::new().expect("Could not create mainloop");
    let mut context = Context::new(&mainloop, "display-volume").expect("Could not create context");
    context
        .connect(None, pulse::context::flags::NOFLAGS, None)
        .expect("Could not initiate context connection");
    // XXX: this busyloop is meh
    loop {
        match mainloop.iterate(false) {
            IterateResult::Quit(n) => panic!("Mainloop quit: {:?}", n),
            IterateResult::Err(e) => panic!("Mainloop error: {}", e),
            IterateResult::Success(_) => {}
        }
        match context.get_state() {
            context::State::Ready => break,
            e @ context::State::Failed | e @ context::State::Terminated => {
                panic!("Context disconnected: {:?}", e)
            }
            _ => {}
        }
    }
    let introspector = context.introspect();
    let op = introspector.get_sink_info_by_name("@DEFAULT_SINK@", |result| match result {
        ListResult::Item(si) => {
            let volume = si.volume.get()[0];
            let muted = si.mute;
            let display_volume = unlerp(VOLUME_0 as f32, VOLUME_100 as f32, volume.0 as f32);
            let icon = if muted {
                "audio-volume-muted"
            } else if display_volume < 0.2 {
                "audio-volume-low"
            } else if display_volume < 1.0 {
                "audio-volume-medium"
            } else {
                "audio-volume-high"
            };
            Notification::new()
                .timeout(Timeout::Milliseconds(2000))
                .hint(Hint::Custom("synchronous".into(), "volume".into()))
                .summary(&format!(
                    "{}{}",
                    volume.print().trim(),
                    if muted { " (muted)" } else { "" }
                ))
                .icon(icon)
                .hint(Hint::CustomInt(
                    "value".into(),
                    (unlerp(VOLUME_0 as f32, VOLUME_100 as f32, volume.0 as f32) * 100.0).round()
                        as i32,
                ))
                .show()
                .unwrap();
        }
        ListResult::End => {}
        ListResult::Error => panic!("Error getting sink info"),
    });
    // XXX: this busyloop is meh
    loop {
        match mainloop.iterate(false) {
            IterateResult::Quit(n) => panic!("Mainloop quit: {:?}", n),
            IterateResult::Err(e) => panic!("Mainloop error: {}", e),
            IterateResult::Success(_) => {}
        }
        match op.get_state() {
            operation::State::Done => break,
            operation::State::Running => {}
            operation::State::Cancelled => panic!("get_sink_info operation cancelled"),
        }
    }
}
