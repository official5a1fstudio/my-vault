pub mod auth_view;
pub mod settings_view;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Authenticator,
    Settings,
}
