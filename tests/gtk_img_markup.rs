use gtk4 as gtk;
use gtk::prelude::*;

fn main() {
    let app = gtk::Application::builder().application_id("com.test.img").build();
    app.connect_activate(|app| {
        let label = gtk::Label::new(None);
        label.set_use_markup(true);
        // Let's see what happens with img tag
        label.set_markup("<a href=\"#\"><img src=\"test.png\" alt=\"test\"></a>");
        let window = gtk::ApplicationWindow::builder().application(app).child(&label).build();
        window.present();
        
        let app_clone = app.clone();
        // Stop after a second
        gtk::glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
            app_clone.quit();
            gtk::glib::ControlFlow::Break
        });
    });
    app.run();
}
