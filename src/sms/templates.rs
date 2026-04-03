const OPT_OUT_FOOTER: &str = "\n\nText PEACE BE STILL or STOP to unsubscribe";

pub fn magic_link_sms(event_name: &str, link: &str) -> String {
    format!(
        "Edit your RSVP for {event_name}: {link}{OPT_OUT_FOOTER}"
    )
}

pub fn reminder_sms(event_name: &str, message: &str) -> String {
    format!(
        "Reminder for {event_name}: {message}{OPT_OUT_FOOTER}"
    )
}
