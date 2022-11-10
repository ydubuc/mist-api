use crate::{app, users::models::user::User};

pub fn request_password_update_template(
    user: &User,
    access_token: &str,
    frontend_url: &str,
) -> (String, String) {
    let url = format!("{}/auth/password/{}", frontend_url, access_token);

    (
        format!("{} password update", app::config::APP_NAME),
        format!(
            "
        <p>Hello {},</p>
        <p>We heard that you want to update your {} password.</p>
        <p>You can use the following link to change it:</p>
        <a href={}>{}</a>
        <p>This link will expire in 1 hour.</p>
        <p>If you did not request this, ignore this email.</p>
        <p>Your friends at {}</p>
        ",
            user.displayname,
            app::config::APP_NAME,
            url,
            url,
            app::config::APP_NAME
        ),
    )
}
