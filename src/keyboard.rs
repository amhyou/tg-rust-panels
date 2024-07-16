use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn start_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("🛒 Purchase".to_owned(), "purchase".to_owned()),
            InlineKeyboardButton::callback("🔝 Deposit".to_owned(), "deposit".to_owned()),
        ],
        vec![InlineKeyboardButton::callback(
            "🆘 Support".to_owned(),
            "support".to_owned(),
        )],
    ])
}

pub fn deposit_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "✅ I paid".to_owned(),
            "paid".to_owned(),
        )],
        vec![InlineKeyboardButton::callback(
            "🔙 Back".to_owned(),
            "back".to_owned(),
        )],
    ])
}

pub fn purchase_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("1 pic".to_owned(), "buy_1".to_owned()),
            InlineKeyboardButton::callback("2 pics".to_owned(), "buy_2".to_owned()),
            InlineKeyboardButton::callback("5 pics".to_owned(), "buy_5".to_owned()),
        ],
        vec![
            InlineKeyboardButton::callback("10 pics".to_owned(), "buy_10".to_owned()),
            InlineKeyboardButton::callback("20 pics".to_owned(), "buy_20".to_owned()),
            InlineKeyboardButton::callback("50 Pics".to_owned(), "buy_50".to_owned()),
        ],
        vec![InlineKeyboardButton::callback(
            "🔙 Back".to_owned(),
            "back".to_owned(),
        )],
    ])
}

pub fn support_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "🔙 Back".to_owned(),
        "back".to_owned(),
    )]])
}
