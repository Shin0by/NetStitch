use crate::AppConnector;

pub mod chrome;
pub mod discord;
pub mod firefox;
pub mod manual;
pub mod telegram;
pub mod whatsapp;
pub mod yandex_browser;

pub fn all_connectors() -> Vec<Box<dyn AppConnector>> {
    vec![
        Box::new(discord::DiscordConnector),
        Box::new(telegram::TelegramConnector),
        Box::new(whatsapp::WhatsAppConnector),
        Box::new(chrome::ChromeConnector),
        Box::new(firefox::FirefoxConnector),
        Box::new(yandex_browser::YandexBrowserConnector),
        Box::new(manual::ManualPathConnector),
    ]
}
