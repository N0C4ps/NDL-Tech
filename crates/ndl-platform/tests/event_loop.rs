use ndl_platform::event::{AppEvent, Event, EventLoop};

#[test]
fn poll_event_returns_pushed_events_in_order() {
    let mut event_loop: EventLoop<()> = EventLoop::new();
    event_loop.push_event(Event::App(AppEvent::Started));
    event_loop.push_event(Event::App(AppEvent::Resumed));

    assert_eq!(event_loop.poll_event(), Some(Event::App(AppEvent::Started)));
    assert_eq!(event_loop.poll_event(), Some(Event::App(AppEvent::Resumed)));
    assert_eq!(event_loop.poll_event(), None);
}

#[test]
fn proxy_sends_custom_user_events() {
    let mut event_loop: EventLoop<&'static str> = EventLoop::new();
    let proxy = event_loop.create_proxy();

    proxy.send_event("ping").expect("send");
    assert_eq!(event_loop.poll_event(), Some(Event::User("ping")));
}
